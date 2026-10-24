//! Optional passenger loading thresholds, separate from mandatory station dwell.
//! The extra wait counts only otherwise safe departure opportunities.
use crate::{Park, RideId, RideOperatingError, RideVisitPhase};

pub const MAX_DISPATCH_PASSENGERS: u8 = crate::TRAIN_SEATS as u8;
pub const MIN_EXTRA_WAIT_TICKS: u16 = 30;
pub const MAX_EXTRA_WAIT_TICKS: u16 = 1800;
pub const EXTRA_WAIT_STEP_TICKS: u16 = 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideDispatchPolicy {
    /// Zero preserves the existing dispatch behavior without an occupancy hold.
    pub minimum_passengers: u8,
    /// Extra safe loading time; boarding, unloading and station dwell are excluded.
    pub maximum_extra_wait_ticks: u16,
}

impl Default for RideDispatchPolicy {
    fn default() -> Self {
        Self {
            minimum_passengers: 0,
            maximum_extra_wait_ticks: 900,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideDispatchError {
    Operating(RideOperatingError),
    InvalidMinimumPassengers,
    InvalidMaximumExtraWait,
}

impl std::fmt::Display for RideDispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Operating(error) => error.fmt(f),
            Self::InvalidMinimumPassengers => f.write_str("Minimum load must be 0 to 6 passengers"),
            Self::InvalidMaximumExtraWait => {
                f.write_str("Extra loading wait must be 1 to 60 whole seconds")
            }
        }
    }
}
impl std::error::Error for RideDispatchError {}
impl From<RideOperatingError> for RideDispatchError {
    fn from(error: RideOperatingError) -> Self {
        Self::Operating(error)
    }
}

impl RideDispatchPolicy {
    fn validate(self) -> Result<(), RideDispatchError> {
        if self.minimum_passengers > MAX_DISPATCH_PASSENGERS {
            return Err(RideDispatchError::InvalidMinimumPassengers);
        }
        if !(MIN_EXTRA_WAIT_TICKS..=MAX_EXTRA_WAIT_TICKS).contains(&self.maximum_extra_wait_ticks)
            || !self
                .maximum_extra_wait_ticks
                .is_multiple_of(EXTRA_WAIT_STEP_TICKS)
        {
            return Err(RideDispatchError::InvalidMaximumExtraWait);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RideDispatchRecord {
    pub(crate) ride: RideId,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub(crate) policy: RideDispatchPolicy,
    pub(crate) extra_waited_ticks: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideDispatchProgress {
    pub minimum_passengers: u8,
    pub seated_passengers: u8,
    pub maximum_extra_wait_ticks: u16,
    pub extra_waited_ticks: u16,
    pub waiting_for_load: bool,
}

impl Park {
    pub fn ride_dispatch_policy(&self, id: RideId) -> Option<RideDispatchPolicy> {
        self.ride_by_id(id)?;
        Some(
            self.ride_dispatch
                .iter()
                .find(|record| record.ride == id)
                .map_or_else(RideDispatchPolicy::default, |record| record.policy),
        )
    }

    pub fn ride_dispatch_policy_status(&self, id: RideId) -> Result<(), RideOperatingError> {
        self.ride_operating_settings_status(id)
    }

    pub fn set_ride_dispatch_policy(
        &mut self,
        id: RideId,
        policy: RideDispatchPolicy,
    ) -> Result<bool, RideDispatchError> {
        let current = self
            .ride_dispatch_policy(id)
            .ok_or(RideOperatingError::UnknownRide(id))?;
        policy.validate()?;
        if usize::from(policy.minimum_passengers) > self.ride_capacity(id).unwrap_or(0) {
            return Err(RideDispatchError::InvalidMinimumPassengers);
        }
        if current == policy {
            return Ok(false);
        }
        self.ride_dispatch_policy_status(id)?;
        self.ride_dispatch.retain(|record| record.ride != id);
        if policy != RideDispatchPolicy::default() {
            self.ride_dispatch.push(RideDispatchRecord {
                ride: id,
                policy,
                extra_waited_ticks: 0,
            });
            self.ride_dispatch.sort_by_key(|record| record.ride);
        }
        Ok(true)
    }

    pub fn ride_dispatch_progress(&self, id: RideId) -> Option<RideDispatchProgress> {
        let ride = self.ride_by_id(id)?;
        let policy = self.ride_dispatch_policy(id)?;
        let extra_waited_ticks = self
            .ride_dispatch
            .iter()
            .find(|r| r.ride == id)
            .map_or(0, |r| r.extra_waited_ticks);
        let seated_passengers = self.dispatch_seated_passengers(id);
        let dwell = if self.ride.as_ref().is_some_and(|r| r.id == id) {
            self.train.as_ref().map_or(0, |train| train.dwell_ticks)
        } else {
            self.carousel(id).map_or(0, |carousel| carousel.dwell_ticks)
        };
        Some(RideDispatchProgress {
            minimum_passengers: policy.minimum_passengers,
            maximum_extra_wait_ticks: policy.maximum_extra_wait_ticks,
            seated_passengers,
            extra_waited_ticks,
            waiting_for_load: self.ride_available(id)
                && self.ride_is_open(id)
                && self.ride_stationary(id)
                && !ride.draining
                && dwell == 0
                && !self.ride_holds(id)
                && seated_passengers < policy.minimum_passengers
                && extra_waited_ticks < policy.maximum_extra_wait_ticks,
        })
    }

    fn dispatch_seated_passengers(&self, id: RideId) -> u8 {
        self.guests
            .iter()
            .filter(|guest| {
                guest.ride_visit.as_ref().is_some_and(|visit| {
                    visit.ride == id
                        && visit.phase == RideVisitPhase::Riding
                        && visit.passenger.as_ref().is_some_and(|passenger| {
                            passenger.transfer.is_none()
                                && !passenger.departed
                                && !passenger.completed_lap
                        })
                })
            })
            .count() as u8
    }

    /// Call once at the existing departure point, after mandatory station dwell
    /// and physical holds. True consumes exactly one additional waiting tick.
    /// False changes no timer: the actual departure resets it explicitly.
    pub(crate) fn hold_for_dispatch_load(&mut self, id: RideId) -> bool {
        let Some(ride) = self.ride_by_id(id) else {
            return false;
        };
        if !self.ride_is_open(id) || ride.draining {
            return false;
        }
        let seated = self.dispatch_seated_passengers(id);
        let Some(record) = self
            .ride_dispatch
            .iter_mut()
            .find(|record| record.ride == id)
        else {
            return false;
        };
        if seated >= record.policy.minimum_passengers
            || record.extra_waited_ticks >= record.policy.maximum_extra_wait_ticks
        {
            return false;
        }
        record.extra_waited_ticks += 1;
        true
    }

    /// Closing, faulting, rebuilding or actual departure starts a new loading
    /// interval. Boarding's mandatory dwell reset deliberately does not call it.
    pub(crate) fn reset_ride_dispatch_wait(&mut self, id: RideId) {
        if let Some(record) = self
            .ride_dispatch
            .iter_mut()
            .find(|record| record.ride == id)
        {
            record.extra_waited_ticks = 0;
        }
    }
}

pub(crate) fn validate_saved_dispatch(park: &Park) -> Result<(), String> {
    if park.ride_dispatch.len() > crate::MAX_RIDE_RECORDS {
        return Err("too many ride dispatch records".into());
    }
    let mut previous = None;
    for record in &park.ride_dispatch {
        if previous.is_some_and(|id| id >= record.ride) {
            return Err("ride dispatch records must have unique sorted owners".into());
        }
        previous = Some(record.ride);
        if park.ride_by_id(record.ride).is_none() {
            return Err("ride dispatch record has an unknown owner".into());
        }
        record
            .policy
            .validate()
            .map_err(|error| error.to_string())?;
        if usize::from(record.policy.minimum_passengers)
            > park.ride_capacity(record.ride).unwrap_or(0)
        {
            return Err("minimum load exceeds the configured ride capacity".into());
        }
        if record.policy == RideDispatchPolicy::default() {
            return Err("default dispatch policies must not be stored as overrides".into());
        }
        if record.extra_waited_ticks > record.policy.maximum_extra_wait_ticks
            || u64::from(record.extra_waited_ticks) > park.ticks
            || (record.extra_waited_ticks != 0
                && (record.policy.minimum_passengers == 0
                    || !park.ride_available(record.ride)
                    || !park.ride_is_open(record.ride)
                    || park.ride_is_running(record.ride)))
        {
            return Err("ride extra loading wait is inconsistent with its policy or state".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RideOperatingSettings, Speed, TilePos};
    use serde_json::{Value, json};

    fn empty_rides() -> (Park, RideId, RideId) {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        park.set_train_open(false);
        let coaster = park.ride().unwrap().id;
        let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        (park, coaster, carousel)
    }
    fn advance(park: &mut Park, ticks: usize) {
        for _ in 0..ticks {
            park.advance(Speed::Normal);
        }
    }
    fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
        for _ in 0..24_000 {
            if predicate(park) {
                return;
            }
            park.advance(Speed::Normal);
        }
        panic!("dispatch test condition not reached");
    }
    fn wait(park: &Park, id: RideId) -> u16 {
        park.ride_dispatch_progress(id).unwrap().extra_waited_ticks
    }
    fn dwell(park: &Park, id: RideId) -> u16 {
        if park.ride().is_some_and(|r| r.id == id) {
            park.train().unwrap().dwell_ticks
        } else {
            park.carousel(id).unwrap().dwell_ticks
        }
    }
    fn saved(park: &Park) -> Park {
        let bytes = park.to_save_bytes().unwrap();
        let restored = Park::from_save_bytes(&bytes).unwrap();
        assert_eq!(park, &restored);
        assert_eq!(restored.to_save_bytes().unwrap(), bytes);
        restored
    }
    fn value(park: &Park) -> Value {
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap()
    }
    fn read(value: &Value) -> Result<Park, crate::SaveError> {
        Park::from_save_bytes(&serde_json::to_vec(value).unwrap())
    }
    fn policy(minimum: u8, maximum: u16) -> RideDispatchPolicy {
        RideDispatchPolicy {
            minimum_passengers: minimum,
            maximum_extra_wait_ticks: maximum,
        }
    }
    fn configure(park: &mut Park, id: RideId, minimum: u8, maximum: u16) {
        park.set_ride_open(id, false).unwrap();
        park.set_ride_operating_settings(
            id,
            RideOperatingSettings {
                station_wait_ticks: 30,
                ..Default::default()
            },
        )
        .unwrap();
        park.set_ride_dispatch_policy(id, policy(minimum, maximum))
            .unwrap();
    }

    #[test]
    fn editing_is_atomic_per_ride_bounded_and_preserves_money_and_existing_settings() {
        let (mut park, coaster, carousel) = empty_rides();
        assert_eq!(
            park.ride_dispatch_policy(coaster),
            Some(RideDispatchPolicy::default())
        );
        assert_eq!(park.ride_dispatch_policy(RideId(99)), None);
        let before = park.clone();
        for bad in [
            policy(7, 900),
            policy(255, 900),
            policy(1, 0),
            policy(1, 29),
            policy(1, 31),
            policy(1, 1801),
            policy(1, u16::MAX),
        ] {
            assert!(park.set_ride_dispatch_policy(coaster, bad).is_err());
            assert_eq!(park, before);
        }
        assert_eq!(
            park.set_ride_dispatch_policy(RideId(99), policy(1, 900)),
            Err(RideDispatchError::Operating(
                RideOperatingError::UnknownRide(RideId(99))
            ))
        );
        let before = value(&park);
        assert_eq!(
            park.set_ride_dispatch_policy(coaster, policy(6, 1800)),
            Ok(true)
        );
        let mut after = value(&park);
        after["park"]["ride_dispatch"] = before["park"]["ride_dispatch"].clone();
        assert_eq!(before, after);
        assert_eq!(
            park.ride_dispatch_policy(carousel),
            Some(RideDispatchPolicy::default())
        );
        park.set_ride_open(coaster, true).unwrap();
        let before = park.clone();
        assert_eq!(
            park.set_ride_dispatch_policy(coaster, policy(6, 1800)),
            Ok(false)
        );
        assert_eq!(
            park.set_ride_dispatch_policy(coaster, policy(1, 30)),
            Err(RideDispatchError::Operating(
                RideOperatingError::CloseAndStop
            ))
        );
        assert_eq!(park, before);
        park.set_ride_open(coaster, false).unwrap();
        assert_eq!(
            park.set_ride_dispatch_policy(coaster, RideDispatchPolicy::default()),
            Ok(true)
        );
        assert!(park.ride_dispatch.is_empty());
        saved(&park);
    }

    #[test]
    fn timeout_adds_exactly_selected_safe_ticks_to_each_rides_original_departure() {
        for coaster_target in [false, true] {
            for extra in [30, 60, 1800] {
                let (mut delayed, coaster, carousel) = empty_rides();
                let id = if coaster_target { coaster } else { carousel };
                configure(&mut delayed, id, 6, extra);
                let mut baseline = delayed.clone();
                baseline
                    .set_ride_dispatch_policy(id, RideDispatchPolicy::default())
                    .unwrap();
                baseline.set_ride_open(id, true).unwrap();
                delayed.set_ride_open(id, true).unwrap();
                let mut first_baseline = None;
                let mut first_delayed = None;
                for tick in 1..=u32::from(extra) + 40 {
                    baseline.advance(Speed::Normal);
                    delayed.advance(Speed::Normal);
                    if baseline.ride_is_running(id) && first_baseline.is_none() {
                        first_baseline = Some(tick);
                    }
                    if delayed.ride_is_running(id) {
                        first_delayed = Some(tick);
                        break;
                    }
                    assert!(wait(&delayed, id) <= extra);
                    if wait(&delayed, id) == extra / 2 {
                        saved(&delayed);
                    }
                }
                assert_eq!(
                    first_delayed,
                    first_baseline.map(|tick| tick + u32::from(extra))
                );
                assert!(first_delayed.is_some());
                assert_eq!(wait(&delayed, id), 0);
                saved(&delayed);
            }
        }
    }

    #[test]
    fn no_minimum_and_paused_or_grouped_ticks_preserve_existing_motion_and_wait_history() {
        let (mut base, coaster, carousel) = empty_rides();
        let mut optional = base.clone();
        for id in [coaster, carousel] {
            optional
                .set_ride_dispatch_policy(id, policy(0, 30))
                .unwrap();
            optional.set_ride_open(id, true).unwrap();
            base.set_ride_open(id, true).unwrap();
        }
        for _ in 0..2400 {
            base.advance(Speed::Normal);
            optional.advance(Speed::Normal);
            let mut projection = optional.clone();
            projection.ride_dispatch.clear();
            assert_eq!(base, projection);
        }
        let (mut waiting, coaster, carousel) = empty_rides();
        for id in [coaster, carousel] {
            configure(&mut waiting, id, 6, 1800);
            waiting.set_ride_open(id, true).unwrap();
        }
        advance(&mut waiting, 100);
        assert!(wait(&waiting, coaster) > 0 && wait(&waiting, carousel) > 0);
        let before = waiting.clone();
        advance_paused(&mut waiting, 20);
        assert_eq!(waiting, before);
        let mut normal = saved(&waiting);
        for _ in 0..800 {
            normal.advance(Speed::Normal);
        }
        for speed in [Speed::Fast, Speed::Fastest] {
            let mut grouped = saved(&waiting);
            for _ in 0..800 / speed.multiplier() {
                grouped.advance(speed);
            }
            assert_eq!(grouped, normal);
        }
    }
    fn advance_paused(park: &mut Park, times: usize) {
        for _ in 0..times {
            park.advance(Speed::Paused);
        }
    }

    fn guest_fixture(coaster_target: bool, minimum: u8) -> (Park, RideId) {
        let (mut park, carousel) = crate::carousel::tests::fixture(true);
        let coaster = park.ride().unwrap().id;
        park.set_ride_open(coaster, false).unwrap();
        park.set_ride_open(carousel, false).unwrap();
        // A single real guest makes threshold success vs timeout unambiguous.
        park.guests.truncate(1);
        let id = if coaster_target { coaster } else { carousel };
        configure(&mut park, id, minimum, 1800);
        park.set_ride_open(id, true).unwrap();
        (park, id)
    }

    #[test]
    fn boarding_pauses_extra_wait_without_erasing_it_and_threshold_or_timeout_charges_once() {
        for coaster_target in [false, true] {
            for minimum in [1, 3] {
                let (mut park, id) = guest_fixture(coaster_target, minimum);
                seek(&mut park, |p| {
                    wait(p, id) > 0
                        && p.guests[0].ride_visit.as_ref().is_some_and(|v| {
                            v.ride == id
                                && v.phase == RideVisitPhase::Boarding
                                && v.passenger
                                    .as_ref()
                                    .and_then(|p| p.transfer.as_ref())
                                    .is_some_and(|t| t.progress == 0)
                        })
                });
                let elapsed = wait(&park, id);
                let wallet = park.guests[0].wallet_cents;
                let revenue = park.ride_by_id(id).unwrap().revenue_cents;
                let fare = park.guests[0]
                    .ride_visit
                    .as_ref()
                    .unwrap()
                    .passenger
                    .as_ref()
                    .unwrap()
                    .fare_cents;
                assert!(elapsed < 1800);
                let mut restored = saved(&park);
                for _ in 0..60 {
                    park.advance(Speed::Normal);
                    restored.advance(Speed::Normal);
                    assert_eq!(park, restored);
                    assert_eq!(wait(&park, id), elapsed);
                    assert_eq!(park.guests[0].wallet_cents, wallet);
                }
                assert!(dwell(&park, id) > 0);
                assert_eq!(
                    park.ride_dispatch_progress(id).unwrap().seated_passengers,
                    1
                );
                assert_eq!(
                    park.guests[0].ride_visit.as_ref().unwrap().phase,
                    RideVisitPhase::Riding
                );
                saved(&park);
                for _ in 0..dwell(&park, id).saturating_sub(1) {
                    park.advance(Speed::Normal);
                    assert_eq!(wait(&park, id), elapsed);
                }
                let mut last_wait = wait(&park, id);
                for _ in 0..2000 {
                    if park.ride_is_running(id) {
                        break;
                    }
                    assert_eq!(park.guests[0].wallet_cents, wallet);
                    last_wait = wait(&park, id);
                    park.advance(Speed::Normal);
                }
                assert!(park.ride_is_running(id));
                if minimum == 1 {
                    assert_eq!(last_wait, elapsed);
                } else {
                    assert_eq!(last_wait, 1800);
                }
                assert_eq!(wait(&park, id), 0);
                assert_eq!(park.guests[0].wallet_cents, wallet - fare);
                assert_eq!(
                    park.ride_by_id(id).unwrap().revenue_cents,
                    revenue + u64::from(fare)
                );
                let paid_cash = park.cash_cents();
                advance(&mut park, 20);
                assert_eq!(park.cash_cents(), paid_cash);
                let mut restored = saved(&park);
                park.set_ride_open(id, false).unwrap();
                restored.set_ride_open(id, false).unwrap();
                for _ in 0..4000 {
                    park.advance(Speed::Normal);
                    restored.advance(Speed::Normal);
                    assert_eq!(park, restored);
                }
                assert!(
                    park.guests[0]
                        .ride_visit
                        .as_ref()
                        .is_none_or(|v| v.passenger.is_none())
                );
                assert_eq!(park.ride_by_id(id).unwrap().completed, 1);
                saved(&park);
            }
        }
    }

    #[test]
    fn closing_before_departure_cancels_unpaid_trip_and_editing_stays_blocked_until_empty() {
        for coaster_target in [false, true] {
            let (mut park, id) = guest_fixture(coaster_target, 3);
            seek(&mut park, |p| {
                p.ride_dispatch_progress(id).unwrap().seated_passengers == 1 && wait(p, id) > 0
            });
            let wallet = park.guests[0].wallet_cents;
            let revenue = park.ride_by_id(id).unwrap().revenue_cents;
            park.set_ride_open(id, false).unwrap();
            assert_eq!(wait(&park, id), 0);
            let before = park.clone();
            assert_eq!(
                park.set_ride_dispatch_policy(id, policy(1, 30)),
                Err(RideDispatchError::Operating(RideOperatingError::Passengers))
            );
            assert_eq!(park, before);
            let mut restored = saved(&park);
            for _ in 0..180 {
                park.advance(Speed::Normal);
                restored.advance(Speed::Normal);
                assert_eq!(park, restored);
            }
            assert_eq!(park.guests[0].wallet_cents, wallet);
            assert_eq!(park.ride_by_id(id).unwrap().revenue_cents, revenue);
            assert_eq!(park.ride_by_id(id).unwrap().completed, 0);
            assert_eq!(park.set_ride_dispatch_policy(id, policy(1, 30)), Ok(true));
            saved(&park);
        }
    }

    #[test]
    fn draining_a_new_unaffordable_queue_releases_reserved_riders_without_loading_timeout() {
        for coaster_target in [false, true] {
            let (mut park, carousel) = crate::carousel::tests::fixture(true);
            let coaster = park.ride().unwrap().id;
            park.set_ride_open(coaster, false).unwrap();
            park.set_ride_open(carousel, false).unwrap();
            let id = if coaster_target { coaster } else { carousel };
            configure(&mut park, id, 6, 1800);
            park.set_ride_open(id, true).unwrap();
            seek(&mut park, |p| {
                p.ride_dispatch_progress(id).unwrap().seated_passengers >= 1
            });
            let before_price_wait = wait(&park, id);
            assert!(before_price_wait > 0);
            park.set_ride_ticket_price(id, 5000).unwrap();
            // A paused save may happen in the same frame as a control change;
            // the accrued timer remains valid before the next production tick.
            assert_eq!(wait(&park, id), before_price_wait);
            saved(&park);
            seek(&mut park, |p| p.ride_by_id(id).unwrap().draining);
            saved(&park);
            assert!(park.ride_dispatch_progress(id).unwrap().seated_passengers < 6);
            assert!(wait(&park, id) < 1800);
            let mut maximum_observed_wait = wait(&park, id);
            for _ in 0..2000 {
                if park.ride_is_running(id) {
                    break;
                }
                maximum_observed_wait = maximum_observed_wait.max(wait(&park, id));
                park.advance(Speed::Normal);
            }
            assert!(park.ride_is_running(id));
            // No earlier threshold was reached: this phase was released by the
            // production drain path, and original reserved fares remain $1.
            assert_eq!(wait(&park, id), 0);
            assert!(park.ride_by_id(id).unwrap().revenue_cents > 0);
            assert!(
                park.guests
                    .iter()
                    .filter_map(|g| g.ride_visit.as_ref())
                    .filter(|v| v.ride == id)
                    .filter_map(|v| v.passenger.as_ref())
                    .all(|p| p.fare_cents == 100 && p.fare_paid)
            );
            assert!(maximum_observed_wait < 1800);
            saved(&park);
        }
    }

    #[test]
    fn fault_repair_close_and_demolition_clear_wait_but_retain_each_owners_policy() {
        let (mut park, coaster, carousel) = empty_rides();
        for id in [coaster, carousel] {
            configure(&mut park, id, 6, 1800);
            park.set_ride_open(id, true).unwrap();
        }
        advance(&mut park, 100);
        assert!(wait(&park, coaster) > 0 && wait(&park, carousel) > 0);
        let track = park.circuit().unwrap().pieces[4].object;
        park.remove_object(track).unwrap();
        assert_eq!(wait(&park, coaster), 0);
        assert_eq!(park.ride_dispatch_policy(coaster), Some(policy(6, 1800)));
        saved(&park);
        park.install_train_fixture().unwrap();
        assert_eq!(wait(&park, coaster), 0);
        assert_eq!(park.ride_dispatch_policy(coaster), Some(policy(6, 1800)));
        park.set_ride_open(carousel, false).unwrap();
        assert_eq!(wait(&park, carousel), 0);
        park.remove_carousel(carousel).unwrap();
        assert_eq!(park.ride_dispatch_policy(carousel), Some(policy(6, 1800)));
        let replacement = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        assert_eq!(
            park.ride_dispatch_policy(replacement),
            Some(RideDispatchPolicy::default())
        );
        assert_eq!(wait(&park, replacement), 0);
        saved(&park);
    }

    #[test]
    fn carousel_zero_dwell_requires_accrued_loading_wait() {
        for minimum in [1, MAX_DISPATCH_PASSENGERS] {
            let (mut park, _, carousel) = empty_rides();
            configure(&mut park, carousel, minimum, 30);
            park.set_ride_open(carousel, true).unwrap();
            assert_eq!(park.ticks(), 0);
            assert_eq!(dwell(&park, carousel), 30);
            assert_eq!(wait(&park, carousel), 0);
            let mut forged = value(&park);
            forged["park"]["carousels"][0]["dwell_ticks"] = json!(0);
            assert!(matches!(
                read(&forged),
                Err(crate::SaveError::InvalidState(reason))
                    if reason == "carousel operation counters or phase are invalid"
            ));
            saved(&park);
        }
    }

    #[test]
    fn carousel_loading_timeout_snapshot_resumes_at_next_safe_departure() {
        for maximum in [MIN_EXTRA_WAIT_TICKS, MAX_EXTRA_WAIT_TICKS] {
            let (mut park, _, carousel) = empty_rides();
            configure(&mut park, carousel, MAX_DISPATCH_PASSENGERS, maximum);
            park.set_ride_open(carousel, true).unwrap();
            // Carousel's legacy departure opportunity occurs on tick 30;
            // that opportunity consumes the first extra tick while held.
            advance(&mut park, 29 + usize::from(maximum));
            assert_eq!(
                park.carousel(carousel).unwrap().phase,
                crate::CarouselPhase::Station
            );
            assert_eq!(dwell(&park, carousel), 0);
            assert_eq!(wait(&park, carousel), maximum);
            let mut restored = saved(&park);
            let paused = restored.clone();
            restored.advance(Speed::Paused);
            assert_eq!(restored, paused);
            park.advance(Speed::Normal);
            restored.advance(Speed::Normal);
            assert_eq!(restored, park);
            assert!(restored.ride_is_running(carousel));
            assert_eq!(wait(&restored, carousel), 0);
            saved(&restored);
        }
    }

    #[test]
    fn strict_current_records_reject_wrong_owners_shapes_policy_ranges_and_timer_states() {
        let (mut park, coaster, carousel) = empty_rides();
        configure(&mut park, coaster, 1, 60);
        configure(&mut park, carousel, 3, 90);
        park.set_ride_open(coaster, true).unwrap();
        park.set_ride_open(carousel, true).unwrap();
        advance(&mut park, 45);
        let base = value(&park);
        assert_eq!(base["schema_version"], json!(crate::SAVE_SCHEMA_VERSION));
        let mut missing = base.clone();
        missing["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_dispatch");
        assert!(read(&missing).is_err());
        let record = base["park"]["ride_dispatch"][0].clone();
        for invalid in [
            json!(null),
            json!({}),
            json!([null]),
            json!([record.clone(), record.clone()]),
            json!([base["park"]["ride_dispatch"][1], record.clone()]),
            json!(vec![record.clone(); crate::MAX_RIDE_RECORDS + 1]),
        ] {
            let mut forged = base.clone();
            forged["park"]["ride_dispatch"] = invalid;
            assert!(read(&forged).is_err());
        }
        for (path, invalid) in [
            ("/park/ride_dispatch/0/ride", json!(999)),
            ("/park/ride_dispatch/0/policy", json!([1, 60])),
            ("/park/ride_dispatch/0/policy/minimum_passengers", json!(7)),
            ("/park/ride_dispatch/0/policy/minimum_passengers", json!(0)),
            (
                "/park/ride_dispatch/0/policy/maximum_extra_wait_ticks",
                json!(31),
            ),
            ("/park/ride_dispatch/0/extra_waited_ticks", json!(61)),
            ("/park/ride_dispatch/0/extra_waited_ticks", json!(60)),
            ("/park/ride_dispatch/0/extra_waited_ticks", json!(-1)),
            ("/park/ride_dispatch/0/extra_waited_ticks", json!(1.0)),
        ] {
            let mut forged = base.clone();
            *forged.pointer_mut(path).unwrap() = invalid;
            assert!(read(&forged).is_err(), "accepted {path}");
        }
        let mut redundant = base.clone();
        redundant["park"]["ride_dispatch"][0]["policy"] = json!(RideDispatchPolicy::default());
        redundant["park"]["ride_dispatch"][0]["extra_waited_ticks"] = json!(0);
        assert!(read(&redundant).is_err());
        for (path, fields) in [
            (
                "/park/ride_dispatch/0",
                vec!["ride", "policy", "extra_waited_ticks"],
            ),
            (
                "/park/ride_dispatch/0/policy",
                vec!["minimum_passengers", "maximum_extra_wait_ticks"],
            ),
        ] {
            for field in fields {
                let mut forged = base.clone();
                forged
                    .pointer_mut(path)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .remove(field);
                assert!(read(&forged).is_err());
            }
            let mut forged = base.clone();
            forged.pointer_mut(path).unwrap()["future"] = json!(0);
            assert!(read(&forged).is_err());
        }
        let text = serde_json::to_string(&base).unwrap();
        let timer_field = format!("\"extra_waited_ticks\":{}", record["extra_waited_ticks"]);
        let duplicate = text.replacen(&timer_field, &format!("{timer_field},{timer_field}"), 1);
        assert_ne!(duplicate, text);
        assert!(Park::from_save_bytes(duplicate.as_bytes()).is_err());
        park.set_ride_open(coaster, false).unwrap();
        let mut closed = value(&park);
        closed["park"]["ride_dispatch"][0]["extra_waited_ticks"] = json!(1);
        assert!(read(&closed).is_err());
        saved(&park);
    }

    #[test]
    fn schema_sixteen_preserves_variable_operating_history_and_defaults_only_dispatch() {
        let (mut park, coaster, carousel) = empty_rides();
        park.set_ride_operating_settings(
            coaster,
            RideOperatingSettings {
                station_wait_ticks: 900,
                lift_speed: 12,
                brake_speed: 8,
                ..Default::default()
            },
        )
        .unwrap();
        park.set_ride_operating_settings(
            carousel,
            RideOperatingSettings {
                station_wait_ticks: 30,
                cycle_ticks: 360,
                ..Default::default()
            },
        )
        .unwrap();
        park.set_ride_open(carousel, true).unwrap();
        advance(&mut park, 390);
        park.set_ride_open(carousel, false).unwrap();
        assert_eq!(park.carousel(carousel).unwrap().cycles, 1);
        let mut old = value(&park);
        old["schema_version"] = json!(16);
        old["park"].as_object_mut().unwrap().remove("coaster_cars");
        old["content_version"] = json!(16);
        old["park"].as_object_mut().unwrap().remove("ride_dispatch");
        let mut migrated = read(&old).unwrap();
        assert_eq!(migrated, park);
        assert!(migrated.ride_dispatch.is_empty());
        let mut projected = value(&migrated);
        projected["schema_version"] = json!(16);
        projected["park"]
            .as_object_mut()
            .unwrap()
            .remove("coaster_cars");
        projected["content_version"] = json!(16);
        projected["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_dispatch");
        assert_eq!(projected, old);
        let mut smuggled = old.clone();
        smuggled["park"]["ride_dispatch"] = json!([]);
        assert!(read(&smuggled).is_err());
        for id in [coaster, carousel] {
            park.set_ride_open(id, true).unwrap();
            migrated.set_ride_open(id, true).unwrap();
        }
        for _ in 0..1800 {
            park.advance(Speed::Normal);
            migrated.advance(Speed::Normal);
            assert_eq!(park, migrated);
        }
        saved(&park);
    }
}
