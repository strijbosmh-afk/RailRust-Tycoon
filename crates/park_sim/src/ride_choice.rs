//! Deterministic ride choice; no additional random draws change public walking.
use crate::{GuestId, Park, RideAccess, RideId, RideKind};
use std::collections::BTreeMap;

impl Park {
    pub fn guest_preferred_ride_kind(&self, id: GuestId) -> Option<RideKind> {
        self.guests.iter().find(|g| g.id == id)?;
        Some(preference(self.crowd_seed.unwrap_or(0), id))
    }

    pub(crate) fn choose_guest_rides(
        &self,
        graph: &crate::crowds::PathGraph,
        accesses: &BTreeMap<RideId, RideAccess>,
    ) -> BTreeMap<GuestId, RideId> {
        let eligible: Vec<_> = accesses
            .iter()
            .filter_map(|(&id, access)| {
                let ride = self.ride_by_id(id)?;
                if !self.admission_eligible(id, access)
                    || ride.draining
                    || ride.next_queue_ticket == u64::MAX
                {
                    return None;
                }
                // Static geometry is scanned once per eligible ride, never
                // once per guest and never by running a mechanics test lap.
                Some((
                    ride,
                    access,
                    self.ride_queue_stats(id).queueing,
                    self.ride_appeal_profile(id)?,
                ))
            })
            .collect();
        let mut choices = BTreeMap::new();
        for guest in &self.guests {
            if self.visitors.leaving(guest.id)
                || guest.ride_visit.is_some()
                || guest.service_visit.is_some()
                || guest.ride_cooldown != 0
                || guest.next.is_some()
                || guest.wait_ticks != 0
            {
                continue;
            }
            let Some(current) = guest.current else {
                continue;
            };
            let preferred = preference(self.crowd_seed.unwrap_or(0), guest.id);
            let preferred_challenge =
                crate::ride_appeal::preferred_challenge(self.crowd_seed.unwrap_or(0), guest.id);
            let selected = eligible
                .iter()
                .filter_map(|(ride, access, queue, appeal)| {
                    let route = graph.route(current, access.public_entry?)?;
                    // First prefer affordable, acceptable fares, then balance travel,
                    // line length, price, personal taste, previous ride count,
                    // layout challenge and cumulative review reputation.
                    let refused = guest.wallet_cents < ride.ticket_price_cents
                        || !guest.willing_to_pay(ride.ticket_price_cents);
                    let history = guest
                        .ride_history
                        .iter()
                        .find(|h| h.ride == ride.id)
                        .map_or(0, |h| h.completed);
                    let score = route.len() as u128 * 10
                        + *queue as u128 * 10
                        + u128::from(ride.ticket_price_cents / 25)
                        + u128::from(self.ride_kind(ride.id) != Some(preferred)) * 80
                        + u128::from(history) * 20
                        + u128::from(appeal.for_preference(preferred_challenge).total_penalty);
                    Some((refused, score, ride.id))
                })
                .min();
            if let Some((_, _, id)) = selected {
                choices.insert(guest.id, id);
            }
        }
        choices
    }
}

fn preference(seed: u64, id: GuestId) -> RideKind {
    let mut value = seed ^ id.0.wrapping_mul(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    if (value ^ (value >> 31)) & 1 == 0 {
        RideKind::Coaster
    } else {
        RideKind::Carousel
    }
}
