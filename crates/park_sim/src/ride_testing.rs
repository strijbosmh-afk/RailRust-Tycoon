//! On-demand, isolated coaster motion tests using the production train rules.
//! Results are derived presentation data, never saved or applied to guests.
use crate::{
    Park, RideId, RideOperatingError, RideOperatingSettings, TRACK_UNITS, TrackKind,
    TrackPieceSpec, Train, TrainError, TrainPhase,
};

/// Twenty simulated minutes, including the initial station wait. An unusually
/// long supported circuit can exceed this limit and returns a clear failure.
pub const MAX_COASTER_TEST_TICKS: u32 = 36_000;

pub const COASTER_TEST_LIMITATIONS: &str = "Mechanics test excludes passenger loading policy. Layout estimates use approximate motion; no G-force simulation, safety certification or guest-demand prediction.";

/// Comparable inputs for a UI cache. Layout/settings changes invalidate a
/// previous result naturally; a UI must also clear results after park loading.
/// Passenger dispatch policy is excluded because this is not a capacity test.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoasterTestInput {
    pub cars: u8,
    pub ride: RideId,
    pub specs: Vec<TrackPieceSpec>,
    pub settings: RideOperatingSettings,
}

impl CoasterTestInput {
    /// Cheap cache validation: no lap simulation, allocation or park mutation.
    /// An opened, moving, occupied, missing or faulted coaster invalidates it.
    pub fn matches(&self, park: &Park) -> bool {
        park.ride().is_some_and(|ride| ride.id == self.ride)
            && park.coaster_car_count(self.ride) == Some(self.cars)
            && park.ride_operating_settings_status(self.ride).is_ok()
            && park.ride_operating_settings(self.ride) == Some(self.settings)
            && park
                .circuit()
                .is_some_and(|circuit| circuit.specs() == self.specs.as_slice())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoasterLayoutMetrics {
    pub pieces: u32,
    pub curves: u32,
    pub lifts: u32,
    pub drops: u32,
    pub brakes: u32,
    pub ascent_steps: u32,
    pub descent_steps: u32,
    /// Sum of successive descending pieces, including across the input seam.
    pub longest_drop_steps: u32,
}

/// Original, transparent 0..=100 layout scores. These are not measured forces,
/// passenger reviews or operating-safety judgments and do not affect demand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoasterLayoutRatings {
    pub excitement: u8,
    pub intensity: u8,
    pub nausea: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoasterTestReport {
    pub input: CoasterTestInput,
    /// Sampled arc length: one tile is TRACK_UNITS; vertical steps contribute
    /// one quarter of a tile, exactly as in the production train simulation.
    pub length_units: u32,
    pub station_wait_ticks: u16,
    /// Includes the final, possibly partial, movement tick into the station.
    pub moving_ticks: u32,
    pub total_ticks: u32,
    /// Positive train speeds sampled after production motion steps. Parked
    /// zeroes and the completed lap's stop are excluded from both extrema.
    pub min_speed_units_per_tick: u32,
    pub max_speed_units_per_tick: u32,
    /// Exact layout extrema in abstract terrain-height steps, not metres.
    pub min_height: u8,
    pub max_height: u8,
    pub layout: CoasterLayoutMetrics,
    pub ratings: CoasterLayoutRatings,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoasterTestError {
    NoCoaster,
    Circuit(TrainError),
    Operating(RideOperatingError),
    TimedOut { ticks: u32 },
}

impl std::fmt::Display for CoasterTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCoaster => f.write_str("Build a complete coaster before testing"),
            Self::Circuit(error) => write!(f, "Coaster test unavailable: {error}"),
            Self::Operating(error) => error.fmt(f),
            Self::TimedOut { ticks } => write!(
                f,
                "Coaster test did not finish within {ticks} simulation ticks; no rating was produced"
            ),
        }
    }
}

impl std::error::Error for CoasterTestError {}

impl Park {
    /// Requires a complete installed coaster that is closed, stopped and empty.
    /// Access paths/gates are unnecessary: this tests track motion, not whether
    /// the ride can open to guests. It never tests or builds the current draft.
    pub fn coaster_test_input(&self) -> Result<CoasterTestInput, CoasterTestError> {
        let ride = self.ride().ok_or(CoasterTestError::NoCoaster)?;
        self.validate_train_circuit()
            .map_err(CoasterTestError::Circuit)?;
        self.ride_operating_settings_status(ride.id)
            .map_err(CoasterTestError::Operating)?;
        let circuit = self.circuit().ok_or(CoasterTestError::NoCoaster)?;
        // Installed state already validates sampled lengths and envelopes;
        // explicitly retain the complete directed-loop contract for this API.
        crate::trains::validate_specs(self, circuit.specs()).map_err(CoasterTestError::Circuit)?;
        Ok(CoasterTestInput {
            cars: self.coaster_car_count(ride.id).expect("validated coaster"),
            ride: ride.id,
            specs: circuit.specs().to_vec(),
            settings: self.coaster_operating_settings(),
        })
    }

    /// Runs one mechanics lap with production equipment settings and physics.
    /// Passenger thresholds/extra loading waits are intentionally excluded: the
    /// synthetic train is empty, so no guest demand or capacity is measured.
    /// No authoritative clock, RNG, guest, other ride or finance system runs.
    pub fn test_coaster(&self) -> Result<CoasterTestReport, CoasterTestError> {
        self.test_coaster_with_limit(MAX_COASTER_TEST_TICKS)
    }

    fn test_coaster_with_limit(&self, limit: u32) -> Result<CoasterTestReport, CoasterTestError> {
        let input = self.coaster_test_input()?;
        let circuit = self.circuit().ok_or(CoasterTestError::NoCoaster)?.clone();
        let length_units = circuit.total_length;
        let (min_height, max_height, layout) = layout_metrics(&input.specs);
        // This temporary state is deliberately not a saved/playable park: only
        // step_train reads it. No guests, world objects or unrelated rides are
        // copied, admitted or advanced, and no other subsystem observes it.
        // Dispatch records deliberately stay empty: an isolated mechanics lap
        // uses base station dwell, without waiting for nonexistent passengers.
        let mut isolated = Park::new(1, 1);
        isolated.ride = self.ride.clone();
        isolated.ride_operating_settings = self
            .ride_operating_settings
            .iter()
            .filter(|record| record.ride == input.ride)
            .cloned()
            .collect();
        isolated.train = Some(Train {
            open: true,
            phase: TrainPhase::Station,
            speed: 0,
            distance: circuit.station_stop,
            laps: 0,
            dwell_ticks: input.settings.station_wait_ticks,
            piece_index: circuit.locate(circuit.station_stop).0,
            lap_progress: 0,
            departed: false,
        });
        isolated.circuit = Some(circuit);
        let mut min_speed = u32::MAX;
        let mut max_speed = 0;
        for total_ticks in 1..=limit {
            isolated.step_train();
            let train = isolated
                .train
                .as_ref()
                .expect("isolated train remains installed");
            if train.speed > 0 {
                min_speed = min_speed.min(train.speed);
                max_speed = max_speed.max(train.speed);
            }
            if train.laps == 1 {
                let station_wait_ticks = input.settings.station_wait_ticks;
                return Ok(CoasterTestReport {
                    input,
                    length_units,
                    station_wait_ticks,
                    moving_ticks: total_ticks - u32::from(station_wait_ticks),
                    total_ticks,
                    min_speed_units_per_tick: min_speed,
                    max_speed_units_per_tick: max_speed,
                    min_height,
                    max_height,
                    layout,
                    ratings: layout_ratings(length_units, max_speed, layout),
                });
            }
        }
        Err(CoasterTestError::TimedOut { ticks: limit })
    }
}

pub(crate) fn layout_metrics(specs: &[TrackPieceSpec]) -> (u8, u8, CoasterLayoutMetrics) {
    let station = specs
        .iter()
        .position(|spec| spec.kind == TrackKind::Station)
        .expect("validated circuit has a station");
    let mut metrics = CoasterLayoutMetrics {
        pieces: specs.len() as u32,
        ..Default::default()
    };
    let mut min_height = u8::MAX;
    let mut max_height = 0;
    let mut drop_run = 0;
    for spec in specs.iter().cycle().skip(station).take(specs.len()) {
        min_height = min_height.min(spec.entry.height).min(spec.exit.height);
        max_height = max_height.max(spec.entry.height).max(spec.exit.height);
        metrics.curves += u32::from(spec.kind == TrackKind::Curve);
        metrics.lifts += u32::from(spec.kind == TrackKind::Lift);
        metrics.drops += u32::from(spec.kind == TrackKind::Drop);
        metrics.brakes += u32::from(spec.kind == TrackKind::Brake);
        let rise = u32::from(spec.exit.height.saturating_sub(spec.entry.height));
        let drop = u32::from(spec.entry.height.saturating_sub(spec.exit.height));
        metrics.ascent_steps += rise;
        metrics.descent_steps += drop;
        drop_run = if drop > 0 { drop_run + drop } else { 0 };
        metrics.longest_drop_steps = metrics.longest_drop_steps.max(drop_run);
    }
    (min_height, max_height, metrics)
}

fn layout_ratings(
    length: u32,
    max_speed: u32,
    metrics: CoasterLayoutMetrics,
) -> CoasterLayoutRatings {
    // All terms are integer, bounded and visible from the report. Pace is a
    // percentage of this model's speed cap, not a physical speed/force rating.
    // Excitement: up to20 length +33 pace +20 descent +20 feature variety.
    // Intensity: half pace +3 per longest-drop step (max12) +half turn density.
    // Nausea: quarter intensity +third turn density +half descent (max20).
    let pace = max_speed.min(crate::trains::MAX_SPEED) * 100 / crate::trains::MAX_SPEED;
    let turn_density = metrics.curves.saturating_mul(100) / metrics.pieces.max(1);
    let variety = [metrics.curves, metrics.lifts, metrics.drops, metrics.brakes]
        .into_iter()
        .filter(|&count| count > 0)
        .count() as u32;
    let excitement = ((length / TRACK_UNITS as u32).min(120) / 6
        + pace / 3
        + metrics.descent_steps.min(20)
        + variety * 5)
        .min(100);
    let intensity =
        (pace / 2 + metrics.longest_drop_steps.min(12) * 3 + turn_density.min(100) / 2).min(100);
    let nausea =
        (intensity / 4 + turn_density.min(100) / 3 + metrics.descent_steps.min(20) / 2).min(100);
    CoasterLayoutRatings {
        excitement: excitement as u8,
        intensity: intensity as u8,
        nausea: nausea as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TilePos, TrackBuildKind, TrackDirection};

    fn fixture() -> Park {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(42).unwrap();
        park.install_train_fixture().unwrap();
        park.set_train_open(false);
        park.copy_installed_track_to_draft().unwrap();
        park
    }

    #[test]
    fn isolated_test_matches_actual_motion_with_each_supported_settings_extreme() {
        for (wait, lift, brake) in [(90, 24, 20), (30, 12, 8), (900, 48, 40)] {
            let mut park = fixture();
            let id = park.ride().unwrap().id;
            park.set_ride_operating_settings(
                id,
                RideOperatingSettings {
                    station_wait_ticks: wait,
                    lift_speed: lift,
                    brake_speed: brake,
                    ..Default::default()
                },
            )
            .unwrap();
            let before = park.to_save_bytes().unwrap();
            let report = park.test_coaster().unwrap();
            assert_eq!(park.to_save_bytes().unwrap(), before);
            assert!(report.input.matches(&park));
            let mut actual = park.clone();
            actual.set_train_open(true);
            let mut min_speed = u32::MAX;
            let mut max_speed = 0;
            for elapsed in 1..=report.total_ticks {
                actual.advance(Speed::Normal);
                let train = actual.train().unwrap();
                if train.speed > 0 {
                    min_speed = min_speed.min(train.speed);
                    max_speed = max_speed.max(train.speed);
                }
                assert_eq!(train.laps, u64::from(elapsed == report.total_ticks));
            }
            assert_eq!(report.station_wait_ticks, wait);
            assert_eq!(report.moving_ticks + u32::from(wait), report.total_ticks);
            assert_eq!(
                (
                    report.min_speed_units_per_tick,
                    report.max_speed_units_per_tick
                ),
                (min_speed, max_speed)
            );
            assert_eq!(report.length_units, park.circuit().unwrap().total_length);
            assert_eq!((report.min_height, report.max_height), (8, 16));
            assert_eq!(
                (report.layout.ascent_steps, report.layout.descent_steps),
                (8, 8)
            );
            assert_eq!(report.layout.longest_drop_steps, 8);
        }
    }

    #[test]
    fn wait_changes_total_time_and_equipment_changes_motion_without_changing_source() {
        let mut park = fixture();
        let id = park.ride().unwrap().id;
        let first = park.test_coaster().unwrap();
        park.set_ride_operating_settings(
            id,
            RideOperatingSettings {
                station_wait_ticks: 120,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!first.input.matches(&park));
        let waited = park.test_coaster().unwrap();
        assert_eq!(waited.total_ticks, first.total_ticks + 30);
        assert_eq!(waited.moving_ticks, first.moving_ticks);
        assert_eq!(waited.ratings, first.ratings);
        park.set_ride_operating_settings(
            id,
            RideOperatingSettings {
                station_wait_ticks: 120,
                lift_speed: 12,
                brake_speed: 8,
                ..Default::default()
            },
        )
        .unwrap();
        let slow = park.test_coaster().unwrap();
        assert!(slow.moving_ticks > waited.moving_ticks);
        assert!(slow.min_speed_units_per_tick < waited.min_speed_units_per_tick);
        let before = park.clone();
        assert_eq!(park.test_coaster().unwrap(), slow);
        assert_eq!(park, before);
        let restored = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(restored.test_coaster().unwrap(), slow);
    }

    #[test]
    fn empty_mechanics_test_excludes_dispatch_thresholds_and_extra_waits() {
        let mut park = fixture();
        let id = park.ride().unwrap().id;
        let baseline = park.test_coaster().unwrap();
        for (minimum_passengers, maximum_extra_wait_ticks) in [(6, 1800), (1, 30)] {
            park.set_ride_dispatch_policy(
                id,
                crate::RideDispatchPolicy {
                    minimum_passengers,
                    maximum_extra_wait_ticks,
                },
            )
            .unwrap();
            let before = park.clone();
            let bytes = park.to_save_bytes().unwrap();
            let progress = park.ride_dispatch_progress(id).unwrap();
            assert_eq!(progress.seated_passengers, 0);
            assert_eq!(progress.extra_waited_ticks, 0);
            assert!(baseline.input.matches(&park));
            assert_eq!(park.test_coaster().unwrap(), baseline);
            assert_eq!(park.ride_dispatch_progress(id), Some(progress));
            assert_eq!(park, before);
            assert_eq!(park.to_save_bytes().unwrap(), bytes);
            let restored = Park::from_save_bytes(&bytes).unwrap();
            assert_eq!(restored.test_coaster().unwrap(), baseline);
        }
        // Real operation still obeys the configured wait on an empty train.
        // The synthetic lap is not accidentally measuring this admission delay.
        park.set_train_open(true);
        for _ in 0..=baseline.station_wait_ticks {
            park.advance(Speed::Normal);
        }
        assert!(!park.train().unwrap().departed);
        assert_eq!(
            park.ride_dispatch_progress(id).unwrap().extra_waited_ticks,
            1
        );
        assert!(COASTER_TEST_LIMITATIONS.contains("excludes passenger loading policy"));
    }

    #[test]
    fn layout_changes_invalidate_signature_and_ratings_are_derived_without_guest_feedback() {
        let mut park = fixture();
        let original = park.test_coaster().unwrap();
        let mut piece = *park
            .track_draft()
            .iter()
            .find(|piece| piece.kind == TrackKind::Straight)
            .unwrap();
        piece.kind = TrackKind::Brake;
        park.set_track_draft_piece(piece).unwrap();
        assert!(original.input.matches(&park)); // An unapplied draft is not installed track.
        park.apply_track_draft().unwrap();
        assert!(!original.input.matches(&park));
        let changed = park.test_coaster().unwrap();
        assert_eq!(changed.layout.brakes, original.layout.brakes + 1);
        assert_eq!(park.ride().unwrap().rating_count, 0);
        assert_eq!(park.ride().unwrap().rating_total, 0);
        assert!(park.guests().iter().all(|guest| guest.ratings_given == 0));
    }

    #[test]
    fn missing_faulted_open_and_time_limited_tests_refuse_without_changes() {
        assert_eq!(
            Park::new(32, 32).test_coaster(),
            Err(CoasterTestError::NoCoaster)
        );
        let mut park = fixture();
        let input = park.coaster_test_input().unwrap();
        let before = park.clone();
        assert_eq!(
            park.test_coaster_with_limit(1),
            Err(CoasterTestError::TimedOut { ticks: 1 })
        );
        assert_eq!(park, before);
        park.set_train_open(true);
        let open = park.clone();
        assert_eq!(
            park.test_coaster(),
            Err(CoasterTestError::Operating(
                RideOperatingError::CloseAndStop
            ))
        );
        assert!(!input.matches(&park));
        assert_eq!(park, open);
        park.set_train_open(false);
        let object = park.circuit().unwrap().pieces[3].object;
        park.remove_object(object).unwrap();
        let faulted = park.clone();
        assert_eq!(
            park.test_coaster(),
            Err(CoasterTestError::Circuit(TrainError::EmptyCircuit))
        );
        assert!(!input.matches(&park));
        assert_eq!(park, faulted);
    }

    #[test]
    fn completed_passengers_block_testing_until_they_can_leave() {
        let (mut park, carousel) = crate::carousel::tests::fixture(true);
        let coaster = park.ride().unwrap().id;
        park.set_ride_open(carousel, false).unwrap();
        let mut boarded = false;
        for _ in 0..24_000 {
            park.advance(Speed::Normal);
            if park.ride_is_running(coaster) && park.ride_journey_stats(coaster).riding > 0 {
                boarded = true;
                break;
            }
        }
        assert!(boarded);
        let exit = park.ride().unwrap().exit.unwrap().object;
        park.remove_object(exit).unwrap();
        park.set_ride_open(coaster, false).unwrap();
        for _ in 0..6000 {
            if !park.ride_is_running(coaster) {
                break;
            }
            park.advance(Speed::Normal);
        }
        assert!(!park.ride_is_running(coaster));
        let before = park.to_save_bytes().unwrap();
        assert_eq!(
            park.test_coaster(),
            Err(CoasterTestError::Operating(RideOperatingError::Passengers))
        );
        assert_eq!(park.to_save_bytes().unwrap(), before);
    }

    #[test]
    fn level_loop_and_graded_circuit_have_distinct_bounded_estimates() {
        use TrackBuildKind::{CurveRight, Station, Straight};
        use TrackDirection::{East, North, South, West};
        let mut flat = Park::new(32, 32);
        let specs: Vec<_> = [
            (1, 0, Station, East),
            (2, 0, CurveRight, East),
            (2, 1, Straight, South),
            (2, 2, CurveRight, South),
            (1, 2, Straight, West),
            (0, 2, CurveRight, West),
            (0, 1, Straight, North),
            (0, 0, CurveRight, North),
        ]
        .into_iter()
        .map(|(x, y, kind, heading)| {
            kind.spec(
                TilePos {
                    x: x + 10,
                    y: y + 10,
                },
                8,
                heading,
            )
        })
        .collect();
        flat.install_train_circuit(&specs).unwrap();
        flat.set_train_open(false);
        let level = flat.test_coaster().unwrap();
        let graded = fixture().test_coaster().unwrap();
        assert_eq!(level.layout.descent_steps, 0);
        assert_eq!(level.min_height, level.max_height);
        assert!(graded.ratings.excitement > level.ratings.excitement);
        assert!(graded.ratings.intensity > level.ratings.intensity);
        for report in [level, graded] {
            assert!(
                [
                    report.ratings.excitement,
                    report.ratings.intensity,
                    report.ratings.nausea
                ]
                .into_iter()
                .all(|rating| rating <= 100)
            );
        }
        // Moving the input seam inside a drop must not split its metric.
        let park = fixture();
        let mut rotated = park.circuit().unwrap().specs().to_vec();
        let drop = rotated
            .iter()
            .position(|piece| piece.kind == TrackKind::Drop)
            .unwrap();
        rotated.rotate_left(drop + 1);
        assert_eq!(
            layout_metrics(&rotated),
            layout_metrics(park.circuit().unwrap().specs())
        );
        assert!(COASTER_TEST_LIMITATIONS.contains("no G-force"));
    }
}
