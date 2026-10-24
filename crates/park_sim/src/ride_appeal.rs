//! Derived, deterministic ride-choice factors. These are soft preferences,
//! separate from measured mechanics estimates, safety and admission eligibility.
use crate::{CoasterLayoutMetrics, GuestId, Park, RideId, RideKind};

/// The same installed-layout and cumulative-review inputs used by fresh choices.
/// Reputation includes five neutral three-star prior reviews; it is not the
/// actual review average shown in ride experience statistics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideAppealProfile {
    pub layout_challenge: u8,
    pub review_count: u64,
    pub reputation_rating_tenths: u8,
    pub reputation_penalty: u8,
}

/// These penalties are added to travel, queue, price, ride-family preference
/// and repeat-visit costs. They never override access or affordability priority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuestRideAppeal {
    pub profile: RideAppealProfile,
    pub preferred_challenge: u8,
    pub layout_penalty: u16,
    pub total_penalty: u16,
}

impl RideAppealProfile {
    pub(crate) fn for_preference(self, preferred_challenge: u8) -> GuestRideAppeal {
        let layout_penalty =
            u16::from(self.layout_challenge.saturating_sub(preferred_challenge)) * 2;
        GuestRideAppeal {
            profile: self,
            preferred_challenge,
            layout_penalty,
            total_penalty: layout_penalty + u16::from(self.reputation_penalty),
        }
    }
}

impl Park {
    /// Read-only static geometry scan; no train lap, park clone, random draw or
    /// cache mutation. Closed intact rides have profiles; unavailable rides do not.
    pub fn ride_appeal_profile(&self, id: RideId) -> Option<RideAppealProfile> {
        let ride = self.ride_by_id(id)?;
        if !self.ride_available(id) {
            return None;
        }
        let layout_challenge = match self.ride_kind(id)? {
            RideKind::Coaster => {
                let (_, _, metrics) = crate::ride_testing::layout_metrics(self.circuit()?.specs());
                layout_challenge(metrics)
            }
            RideKind::Carousel => 20,
        };
        let reputation_rating_tenths = reputation(ride.rating_count, ride.rating_total);
        Some(RideAppealProfile {
            layout_challenge,
            review_count: ride.rating_count,
            reputation_rating_tenths,
            reputation_penalty: 50 - reputation_rating_tenths,
        })
    }

    /// A stable soft preference derived from the existing seed and guest ID.
    /// It is not a safety limit and does not prevent choosing a challenging ride.
    pub fn guest_preferred_challenge(&self, id: GuestId) -> Option<u8> {
        self.guests.iter().find(|guest| guest.id == id)?;
        Some(preferred_challenge(self.crowd_seed.unwrap_or(0), id))
    }

    /// Read-only explanation using precisely the same factors as choice scoring.
    /// It describes preference costs, not whether the guest can currently join.
    pub fn guest_ride_appeal(&self, guest: GuestId, ride: RideId) -> Option<GuestRideAppeal> {
        Some(
            self.ride_appeal_profile(ride)?
                .for_preference(self.guest_preferred_challenge(guest)?),
        )
    }
}

fn layout_challenge(metrics: CoasterLayoutMetrics) -> u8 {
    let curve_density = metrics.curves.saturating_mul(100) / metrics.pieces.max(1);
    (metrics.longest_drop_steps.min(12) * 5 + curve_density.min(100) * 2 / 5).min(100) as u8
}

fn reputation(count: u64, total: u64) -> u8 {
    // Widen before adding the prior or multiplying lifetime counters.
    ((u128::from(total) * 10 + 150) / (u128::from(count) + 5)) as u8
}

pub(crate) fn preferred_challenge(seed: u64, id: GuestId) -> u8 {
    // Separate salt keeps this preference independent of family/price hashes.
    let mut value = seed ^ 0xa076_1d64_78bd_642f ^ id.0.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    [25, 45, 65, 85][((value >> 16) & 3) as usize]
}

#[cfg(test)]
mod tests;
