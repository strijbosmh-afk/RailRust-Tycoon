//! Integer-cent fares. Reservations remain guest-owned until actual departure.
use crate::{Guest, GuestRideHistory, Park, RideId, RideVisitPhase};
use std::collections::BTreeMap;

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

impl Guest {
    pub(crate) fn ride_history_mut(&mut self, ride: RideId) -> &mut GuestRideHistory {
        let index = self
            .ride_history
            .binary_search_by_key(&ride, |entry| entry.ride)
            .unwrap_or_else(|index| {
                assert!(self.ride_history.len() < crate::MAX_RIDE_RECORDS);
                self.ride_history.insert(
                    index,
                    GuestRideHistory {
                        ride,
                        spent_cents: 0,
                        completed: 0,
                        rating_count: 0,
                        rating_total: 0,
                    },
                );
                index
            });
        &mut self.ride_history[index]
    }
}

impl Park {
    pub fn cash_cents(&self) -> u64 {
        self.cash_cents
    }

    /// An immediate command, including when paused. Existing locked fares stay
    /// unchanged; the next admission and boarding checks use the new price.
    pub fn set_ride_ticket_price(&mut self, id: RideId, cents: u32) -> Result<(), EconomyError> {
        let ride = self
            .ride_by_id_mut(id)
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
        let Some(ride) = self.ride_by_id(id) else {
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
        let mut earned = BTreeMap::<RideId, u64>::new();
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
                .filter(|passenger| passenger.departed && !passenger.fare_paid)
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
            let fare = u64::from(passenger.fare_cents);
            let ride = visit.ride;
            *earned.entry(ride).or_default() += fare;
            passenger.fare_paid = true;
            guest.ride_history_mut(ride).spent_cents += fare;
        }
        for (id, amount) in earned {
            self.ride_by_id_mut(id)
                .expect("passengers belong to a ride")
                .revenue_cents += amount;
            self.record_ride_cash(id, amount);
        }
    }
}

pub(crate) fn validate_economy(park: &Park) -> Result<(), String> {
    let mut spent = 0_u64;
    let mut service_spent = 0_u64;
    let mut histories = BTreeMap::<RideId, (u64, u64)>::new();
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
        if guest.ride_history.len() > crate::MAX_RIDE_RECORDS {
            return Err("guest ride history exceeds its bound".into());
        }
        let mut previous = None;
        let mut history_spent = 0_u64;
        let mut history_completed = 0_u64;
        for entry in &guest.ride_history {
            if park.ride_by_id(entry.ride).is_none() || previous.is_some_and(|id| id >= entry.ride)
            {
                return Err("guest ride history owner or order is invalid".into());
            }
            previous = Some(entry.ride);
            history_spent = history_spent
                .checked_add(entry.spent_cents)
                .ok_or("guest ride spending overflows")?;
            history_completed = history_completed
                .checked_add(entry.completed)
                .ok_or("guest ride completions overflow")?;
            let active = guest
                .ride_visit
                .as_ref()
                .filter(|v| v.ride == entry.ride)
                .and_then(|v| v.passenger.as_ref())
                .filter(|p| p.fare_paid)
                .map_or(0, |p| u64::from(p.fare_cents));
            if entry.completed == 0 && entry.spent_cents != active {
                return Err("attributed spending has no completed or active paid ride".into());
            }
            let total = histories.entry(entry.ride).or_default();
            total.0 = total
                .0
                .checked_add(entry.spent_cents)
                .ok_or("ride spending sum overflows")?;
            total.1 = total
                .1
                .checked_add(entry.completed)
                .ok_or("ride completion sum overflows")?;
        }
        if history_spent != ride_spent || history_completed != guest.rides_completed {
            return Err("guest ride histories disagree with lifetime totals".into());
        }
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
        if let Some(visit) = &guest.ride_visit
            && let Some(passenger) = &visit.passenger
            && passenger.fare_paid
            && guest
                .ride_history
                .iter()
                .find(|h| h.ride == visit.ride)
                .is_none_or(|h| h.spent_cents < u64::from(passenger.fare_cents))
        {
            return Err("paid passenger lacks attributed ride history".into());
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
    let mut revenue = 0_u64;
    for ride in park.all_rides() {
        if ride.ticket_price_cents > MAX_TICKET_PRICE_CENTS
            || histories.get(&ride.id).copied().unwrap_or_default()
                != (ride.revenue_cents, ride.completed)
        {
            return Err("ride revenue or completions disagree with guest histories".into());
        }
        revenue = revenue
            .checked_add(ride.revenue_cents)
            .ok_or("ride revenue overflows")?;
    }
    let service_revenue = park
        .services
        .iter()
        .try_fold(0_u64, |sum, service| sum.checked_add(service.revenue_cents))
        .ok_or("service revenue overflows")?;
    if service_spent != service_revenue
        || revenue.checked_add(service_revenue) != Some(spent)
        || INITIAL_PARK_CASH_CENTS
            .checked_add(spent)
            .and_then(|funds| {
                park.carousels.iter().try_fold(funds, |cash, carousel| {
                    cash.checked_add(carousel.demolition_refund_cents)
                })
            })
            .and_then(|funds| {
                park.services.iter().try_fold(funds, |cash, service| {
                    cash.checked_add(service.demolition_refund_cents)
                })
            })
            .and_then(|funds| funds.checked_add(park.track_economy.refund_cents))
            .and_then(|funds| funds.checked_sub(park.track_economy.construction_cents))
            .and_then(|funds| {
                park.carousels.iter().try_fold(funds, |cash, carousel| {
                    cash.checked_sub(carousel.construction_cost_cents)
                })
            })
            .and_then(|funds| {
                park.services.iter().try_fold(funds, |cash, service| {
                    cash.checked_sub(service.construction_cost_cents)?
                        .checked_sub(service.upkeep_cents)?
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
