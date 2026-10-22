//! Deterministic preferences and event-driven feedback, independent of navigation RNG.
use crate::{Guest, GuestId, Park, Ride, RideId};

pub(crate) const MAX_RATINGS: u64 = u64::MAX / 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuestPreferences {
    pub max_ticket_price_cents: u32,
    pub comfortable_queue_ticks: u32,
}
impl Default for GuestPreferences {
    fn default() -> Self {
        Self {
            max_ticket_price_cents: 600,
            comfortable_queue_ticks: 750,
        }
    }
}
impl GuestPreferences {
    /// A stable, independent integer hash. No draw from the crowd's route RNG.
    pub fn for_guest(seed: u64, id: GuestId) -> Self {
        let mut value = seed ^ id.0.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^= value >> 31;
        Self {
            max_ticket_price_cents: [400, 600, 800, 1000][(value & 3) as usize],
            comfortable_queue_ticks: [450, 750, 1050, 1350][((value >> 8) & 3) as usize],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GuestThought {
    Exploring,
    HeadingToRide,
    Queueing,
    NotEnoughMoney,
    TooExpensive,
    QueueFull,
    CannotReachRide,
    WaitingTooLong,
    RideCancelled,
    EnjoyedRide,
    DisappointedByRide,
    Hungry,
    Thirsty,
    NeedsToilet,
    SeekingService,
    WaitingForService,
    UsingService,
    ServiceUnavailable,
    CannotAffordService,
    ServiceCancelled,
    AteFood,
    HadDrink,
    UsedToilet,
}
impl std::fmt::Display for GuestThought {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Exploring => "Exploring the park",
            Self::HeadingToRide => "Heading to the ride",
            Self::Queueing => "Waiting for the ride",
            Self::NotEnoughMoney => "I do not have enough money",
            Self::TooExpensive => "That ticket is too expensive for me",
            Self::QueueFull => "The queue is full",
            Self::CannotReachRide => "I cannot reach the ride",
            Self::WaitingTooLong => "I have been waiting too long",
            Self::RideCancelled => "My ride visit was cancelled",
            Self::EnjoyedRide => "I enjoyed that ride",
            Self::DisappointedByRide => "That ride was disappointing",
            Self::Hungry => "I am hungry",
            Self::Thirsty => "I am thirsty",
            Self::NeedsToilet => "I need a toilet",
            Self::SeekingService => "Heading to a service",
            Self::WaitingForService => "Waiting at the counter",
            Self::UsingService => "Using a service",
            Self::ServiceUnavailable => "I cannot find a service I need",
            Self::CannotAffordService => "I cannot afford a service I need",
            Self::ServiceCancelled => "That service is no longer available",
            Self::AteFood => "That food helped",
            Self::HadDrink => "That drink helped",
            Self::UsedToilet => "I feel better after using the toilet",
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RideExperienceStats {
    pub willing_guests: usize,
    pub interested_guests: usize,
    pub guest_count: usize,
    pub value_rejections: u64,
    pub rating_count: u64,
    pub rating_total: u64,
    pub average_rating_tenths: Option<u8>,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GuestExperienceSummary {
    pub average_satisfaction: Option<u8>,
    pub unhappy_guests: usize,
    pub guest_count: usize,
}

impl Guest {
    pub fn willing_to_pay(&self, cents: u32) -> bool {
        cents <= self.preferences.max_ticket_price_cents
    }
    pub(crate) fn think(&mut self, thought: GuestThought, tick: u64) {
        self.thought = thought;
        self.thought_tick = tick;
    }
    pub(crate) fn cancel_experience(&mut self, tick: u64) {
        self.satisfaction = self.satisfaction.saturating_sub(8);
        self.think(GuestThought::RideCancelled, tick);
    }
    pub(crate) fn rate_completed_trip(
        &mut self,
        ride: &mut Ride,
        fare: u32,
        waited: u32,
        tick: u64,
    ) {
        let price_penalty = if fare <= self.preferences.max_ticket_price_cents / 2 {
            0
        } else if self.willing_to_pay(fare) {
            1
        } else {
            2
        };
        let queue_penalty = (waited / self.preferences.comfortable_queue_ticks).min(2);
        let score = (5_u32.saturating_sub(price_penalty + queue_penalty)).clamp(1, 5) as u8;
        // A rating slot is reserved at boarding and validated on loading.
        self.ratings_given += 1;
        self.rating_total += u64::from(score);
        self.last_rating = Some(score);
        ride.rating_count += 1;
        ride.rating_total += u64::from(score);
        let history = self.ride_history_mut(ride.id);
        history.completed += 1;
        history.rating_count += 1;
        history.rating_total += u64::from(score);
        self.satisfaction = ((3 * u16::from(self.satisfaction) + u16::from(score) * 20) / 4) as u8;
        self.think(
            if score >= 3 {
                GuestThought::EnjoyedRide
            } else {
                GuestThought::DisappointedByRide
            },
            tick,
        );
    }
}
impl Park {
    pub fn ride_experience_stats(&self, id: RideId) -> RideExperienceStats {
        let Some(ride) = self.ride_by_id(id) else {
            return RideExperienceStats::default();
        };
        RideExperienceStats {
            willing_guests: self
                .guests
                .iter()
                .filter(|guest| guest.willing_to_pay(ride.ticket_price_cents))
                .count(),
            interested_guests: self
                .guests
                .iter()
                .filter(|guest| {
                    guest.willing_to_pay(ride.ticket_price_cents)
                        && guest.wallet_cents >= ride.ticket_price_cents
                })
                .count(),
            guest_count: self.guests.len(),
            value_rejections: ride.value_rejections,
            rating_count: ride.rating_count,
            rating_total: ride.rating_total,
            average_rating_tenths: (ride.rating_count > 0).then(|| {
                (u128::from(ride.rating_total) * 10 / u128::from(ride.rating_count)) as u8
            }),
        }
    }
    pub fn guest_experience_summary(&self) -> GuestExperienceSummary {
        GuestExperienceSummary {
            average_satisfaction: (!self.guests.is_empty()).then(|| {
                (self
                    .guests
                    .iter()
                    .map(|guest| u64::from(guest.satisfaction))
                    .sum::<u64>()
                    / self.guests.len() as u64) as u8
            }),
            unhappy_guests: self
                .guests
                .iter()
                .filter(|guest| guest.satisfaction < 40)
                .count(),
            guest_count: self.guests.len(),
        }
    }
    pub(crate) fn can_reserve_rating(&self, index: usize) -> bool {
        let Some(id) = self.guests[index]
            .ride_visit
            .as_ref()
            .map(|visit| visit.ride)
        else {
            return false;
        };
        let active = self
            .guests
            .iter()
            .filter(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.ride == id && visit.passenger.is_some())
            })
            .count() as u64;
        let all_active = self
            .guests
            .iter()
            .filter(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.passenger.is_some())
            })
            .count() as u128;
        let all_ratings: u128 = self
            .all_rides()
            .map(|ride| u128::from(ride.rating_count))
            .sum();
        self.guests[index].ratings_given < MAX_RATINGS
            && self.guests[index].rides_completed < u64::MAX
            && all_ratings + all_active < u128::from(MAX_RATINGS)
            && self
                .ride_by_id(id)
                .is_some_and(|ride| ride.rating_count < MAX_RATINGS - active)
    }
}

pub(crate) fn validate_experience(park: &Park) -> Result<(), String> {
    let mut count = 0_u64;
    let mut total = 0_u64;
    let mut owners = std::collections::BTreeMap::<RideId, (u64, u64)>::new();
    let mut active = std::collections::BTreeMap::<RideId, u64>::new();
    for guest in &park.guests {
        let passenger = guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| visit.passenger.is_some());
        if ![400, 600, 800, 1000].contains(&guest.preferences.max_ticket_price_cents)
            || ![450, 750, 1050, 1350].contains(&guest.preferences.comfortable_queue_ticks)
            || guest.satisfaction > 100
            || guest.thought_tick > park.ticks
            || guest.ratings_given > guest.rides_completed
            || guest.ratings_given > MAX_RATINGS - u64::from(passenger)
            || guest.rating_total < guest.ratings_given
            || guest.rating_total > guest.ratings_given * 5
            || (guest.ratings_given == 0 && guest.last_rating.is_some())
            || (guest.ratings_given > 0
                && guest
                    .last_rating
                    .is_none_or(|score| !(1..=5).contains(&score)))
            || guest
                .last_rating
                .is_some_and(|score| u64::from(score) > guest.rating_total)
        {
            return Err("guest preferences or feedback history is invalid".into());
        }
        if let Some(score) = guest.last_rating {
            let previous = guest.ratings_given - 1;
            if guest
                .rating_total
                .checked_sub(u64::from(score))
                .is_none_or(|remaining| remaining < previous || remaining > previous * 5)
            {
                return Err("last guest rating disagrees with rating history".into());
            }
        }
        count = count
            .checked_add(guest.ratings_given)
            .ok_or("rating counts overflow")?;
        total = total
            .checked_add(guest.rating_total)
            .ok_or("rating totals overflow")?;
        if passenger {
            *active
                .entry(guest.ride_visit.as_ref().unwrap().ride)
                .or_default() += 1;
        }
        let mut guest_count = 0_u64;
        let mut guest_total = 0_u64;
        for history in &guest.ride_history {
            if park.ride_by_id(history.ride).is_none()
                || history.rating_count > history.completed
                || history.rating_count > MAX_RATINGS
                || history.rating_total < history.rating_count
                || history.rating_total > history.rating_count * 5
            {
                return Err("attributed guest feedback history is invalid".into());
            }
            guest_count = guest_count
                .checked_add(history.rating_count)
                .ok_or("guest attributed rating count overflows")?;
            guest_total = guest_total
                .checked_add(history.rating_total)
                .ok_or("guest attributed rating total overflows")?;
            let entry = owners.entry(history.ride).or_default();
            entry.0 = entry
                .0
                .checked_add(history.rating_count)
                .ok_or("attributed rating count overflows")?;
            entry.1 = entry
                .1
                .checked_add(history.rating_total)
                .ok_or("attributed rating total overflows")?;
        }
        if guest_count != guest.ratings_given || guest_total != guest.rating_total {
            return Err("guest attributed ratings disagree with lifetime history".into());
        }
    }
    let mut ride_count = 0_u64;
    let mut ride_total = 0_u64;
    for ride in park.all_rides() {
        let reserved = active.get(&ride.id).copied().unwrap_or(0);
        if owners.get(&ride.id).copied().unwrap_or_default()
            != (ride.rating_count, ride.rating_total)
            || ride.rating_count > MAX_RATINGS - reserved
            || ride.rating_count > ride.completed
        {
            return Err("ride ratings disagree with guest feedback or reserved capacity".into());
        }
        ride_count = ride_count
            .checked_add(ride.rating_count)
            .ok_or("total ride rating counts overflow")?;
        ride_total = ride_total
            .checked_add(ride.rating_total)
            .ok_or("total ride rating totals overflow")?;
    }
    if (count, total) != (ride_count, ride_total)
        || count > MAX_RATINGS - active.values().sum::<u64>()
    {
        return Err("ride rating totals disagree with guest feedback".into());
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;
