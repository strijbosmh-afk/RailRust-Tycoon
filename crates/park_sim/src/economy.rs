//! Integer-cent fares. Reservations remain guest-owned until actual departure.
use crate::{Park, RideId, RideVisitPhase};

pub const INITIAL_GUEST_WALLET_CENTS: u32 = 2_000;
pub const INITIAL_PARK_CASH_CENTS: u64 = 100_000;
pub const MAX_TICKET_PRICE_CENTS: u32 = 10_000;
pub const PRICE_STEP_CENTS: u32 = 50;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EconomyError {
    UnknownRide(RideId),
    PriceTooHigh { price: u32, max: u32 },
}
impl std::fmt::Display for EconomyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRide(id) => write!(f, "ride #{} does not exist", id.0),
            Self::PriceTooHigh { max, .. } => write!(
                f,
                "ticket price cannot exceed ${}.{:02}",
                max / 100,
                max % 100
            ),
        }
    }
}
impl std::error::Error for EconomyError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RideEconomyStats {
    pub ticket_price_cents: u32,
    pub revenue_cents: u64,
    pub unaffordable_rejections: u64,
    /// Guests whose current wallet covers the current price; includes passengers.
    pub affordable_guests: usize,
    pub guest_count: usize,
    /// Still owned by guests. This is not park cash or earned revenue.
    pub reserved_fares_cents: u64,
}

impl Park {
    pub fn cash_cents(&self) -> u64 {
        self.cash_cents
    }

    /// An immediate command, including when paused. Existing locked fares stay
    /// unchanged; the next admission and boarding checks use the new price.
    pub fn set_ride_ticket_price(&mut self, id: RideId, cents: u32) -> Result<(), EconomyError> {
        let ride = self
            .ride
            .as_mut()
            .filter(|ride| ride.id == id)
            .ok_or(EconomyError::UnknownRide(id))?;
        if cents > MAX_TICKET_PRICE_CENTS {
            return Err(EconomyError::PriceTooHigh {
                price: cents,
                max: MAX_TICKET_PRICE_CENTS,
            });
        }
        ride.ticket_price_cents = cents;
        Ok(())
    }

    pub fn ride_economy_stats(&self, id: RideId) -> RideEconomyStats {
        let Some(ride) = self.ride.as_ref().filter(|ride| ride.id == id) else {
            return RideEconomyStats::default();
        };
        RideEconomyStats {
            ticket_price_cents: ride.ticket_price_cents,
            revenue_cents: ride.revenue_cents,
            unaffordable_rejections: ride.unaffordable_rejections,
            affordable_guests: self
                .guests
                .iter()
                .filter(|guest| guest.wallet_cents >= ride.ticket_price_cents)
                .count(),
            guest_count: self.guests.len(),
            reserved_fares_cents: self
                .guests
                .iter()
                .filter_map(|guest| {
                    let visit = guest.ride_visit.as_ref().filter(|visit| visit.ride == id)?;
                    let passenger = visit.passenger.as_ref()?;
                    (!passenger.fare_paid).then_some(u64::from(passenger.fare_cents))
                })
                .sum(),
        }
    }

    pub(crate) fn settle_departure_fares(&mut self) {
        if self.train.as_ref().is_none_or(|train| !train.departed) {
            return;
        }
        let mut earned = 0_u64;
        for guest in &mut self.guests {
            let Some(visit) = guest
                .ride_visit
                .as_mut()
                .filter(|visit| visit.phase == RideVisitPhase::Riding)
            else {
                continue;
            };
            let Some(passenger) = visit
                .passenger
                .as_mut()
                .filter(|passenger| !passenger.fare_paid)
            else {
                continue;
            };
            // Loaded states and every boarding operation validate this reserved
            // amount. Ride and service visits are mutually exclusive, so a
            // service purchase cannot spend a reserved train fare.
            guest.wallet_cents = guest
                .wallet_cents
                .checked_sub(passenger.fare_cents)
                .expect("validated reserved fare must remain affordable");
            guest.spent_cents += u64::from(passenger.fare_cents);
            earned += u64::from(passenger.fare_cents);
            passenger.fare_paid = true;
        }
        if earned != 0 {
            self.ride
                .as_mut()
                .expect("passengers belong to a ride")
                .revenue_cents += earned;
            self.record_cash(crate::FinanceKind::RideRevenue, None, earned);
        }
    }
}

pub(crate) fn validate_economy(park: &Park) -> Result<(), String> {
    let mut spent = 0_u64;
    let mut service_spent = 0_u64;
    for guest in &park.guests {
        if guest.wallet_cents > INITIAL_GUEST_WALLET_CENTS
            || guest.spent_cents > u64::from(INITIAL_GUEST_WALLET_CENTS)
            || u64::from(guest.wallet_cents) + guest.spent_cents
                != u64::from(INITIAL_GUEST_WALLET_CENTS)
        {
            return Err("guest wallet and spending do not conserve initial funds".into());
        }
        spent = spent
            .checked_add(guest.spent_cents)
            .ok_or("total guest spending overflows")?;
        if guest.service_spent_cents > guest.spent_cents {
            return Err("guest service spending exceeds total spending".into());
        }
        service_spent = service_spent
            .checked_add(guest.service_spent_cents)
            .ok_or("service spending overflows")?;
        let ride_spent = guest.spent_cents - guest.service_spent_cents;
        let passenger = guest
            .ride_visit
            .as_ref()
            .and_then(|visit| visit.passenger.as_ref());
        if let Some(passenger) = passenger
            && (passenger.fare_cents > MAX_TICKET_PRICE_CENTS
                || passenger.fare_paid != passenger.departed
                || (!passenger.fare_paid && passenger.fare_cents > guest.wallet_cents)
                || (passenger.fare_paid && u64::from(passenger.fare_cents) > ride_spent))
        {
            return Err("passenger fare disagrees with wallet or departure".into());
        }
        // A paid trip either still occupies its protected seat or has reached
        // the public exit and incremented rides_completed. Before the first
        // completed return, no other historical spending can legitimately exist.
        let active_paid_fare = passenger
            .filter(|passenger| passenger.fare_paid)
            .map_or(0, |passenger| u64::from(passenger.fare_cents));
        if guest.rides_completed == 0 && ride_spent != active_paid_fare {
            return Err("guest spending has no completed or active paid journey".into());
        }
    }
    let revenue = park.ride.as_ref().map_or(0, |ride| ride.revenue_cents);
    let service_revenue = park
        .services
        .iter()
        .try_fold(0_u64, |sum, service| sum.checked_add(service.revenue_cents))
        .ok_or("service revenue overflows")?;
    if park
        .ride
        .as_ref()
        .is_some_and(|ride| ride.ticket_price_cents > MAX_TICKET_PRICE_CENTS)
        || service_spent != service_revenue
        || revenue.checked_add(service_revenue) != Some(spent)
        || INITIAL_PARK_CASH_CENTS
            .checked_add(spent)
            .and_then(|funds| {
                park.services.iter().try_fold(funds, |cash, service| {
                    cash.checked_sub(service.upkeep_cents)?
                        .checked_sub(service.supplies_cents)
                })
            })
            != Some(park.cash_cents)
    {
        return Err("ride price, revenue, and park cash do not conserve funds".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
