//! Transient, read-only coaster test results and deliberate panel controls.
use crate::{SaveAction, Simulation, ViewState, ui};
use bevy::prelude::*;
use park_sim::{CoasterTestReport, Park, RideId, RideKind, TICKS_PER_SECOND, TRACK_UNITS};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TestCommand {
    Show,
    Run,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CoasterTesting {
    pub(crate) run_pending: bool,
    pub(crate) show_pending: bool,
    pub(crate) report: Option<CoasterTestReport>,
    pub(crate) notice: Option<String>,
}

pub(crate) fn queue(view: &mut ViewState, command: TestCommand) {
    match command {
        TestCommand::Show => view.coaster_testing.show_pending = true,
        TestCommand::Run => view.coaster_testing.run_pending = true,
    }
}

fn blocked(view: &ViewState) -> bool {
    !view.rides_open
        || view.help
        || view.guests_open
        || view.services_open
        || view.finance_open
        || view.designs_open
        || view.capture_active
        || view.diagnostic_case.is_some()
        || view.modal_input_blocked
        || view.modal_transition_pending
        || view
            .save_pending
            .is_some_and(|action| action != SaveAction::Save)
}

pub(crate) fn command_enabled(park: &Park, view: &ViewState, command: TestCommand) -> bool {
    !blocked(view)
        && view.ride_browse_pending == 0
        && (command == TestCommand::Show
            || (view.ride_tab == ui::rides::RideTab::Test
                && can_run(park, ui::rides::selected_id(park, view)).is_ok()))
}

pub(crate) fn apply_commands(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
) {
    let show = std::mem::take(&mut view.coaster_testing.show_pending);
    let run = std::mem::take(&mut view.coaster_testing.run_pending);
    let selected = ui::rides::selected_id(&sim.0, &view);
    view.coaster_testing.refresh(&sim.0, selected);
    if blocked(&view)
        || view.ride_browse_pending != 0
        || keys
            .as_ref()
            .is_some_and(|keys| keys.just_pressed(KeyCode::F9))
    {
        return;
    }
    if show {
        view.ride_tab = ui::rides::RideTab::Test;
        view.ride_access_commands.clear();
        view.ride_operating_commands.clear();
        return;
    }
    if run
        && view.ride_tab == ui::rides::RideTab::Test
        && view.ride_access_commands.is_empty()
        && view.ride_operating_commands.is_empty()
    {
        run_selected(&sim.0, &mut view.coaster_testing, selected);
    }
}

impl CoasterTesting {
    /// Call before displaying a cached report. Loading a park must replace the
    /// enclosing ViewState with Default, even if loaded inputs happen to match.
    pub(crate) fn refresh(&mut self, park: &Park, selected: Option<RideId>) {
        if self.report.as_ref().is_some_and(|report| {
            selected != Some(report.input.ride) || !report.input.matches(park)
        }) {
            self.report = None;
            self.notice = Some("The ride changed. Run a fresh test when it is ready.".into());
        }
    }
}

/// Cheap button eligibility. The production test validates the full installed
/// circuit again before running; draft geometry is never used here.
pub(crate) fn can_run(park: &Park, selected: Option<RideId>) -> Result<(), String> {
    let Some(id) = selected else {
        return Err("Select a coaster to test.".into());
    };
    if park.ride_kind(id) != Some(RideKind::Coaster) {
        return Err("Select the coaster; carousel testing is unavailable.".into());
    }
    park.ride_operating_settings_status(id)
        .map_err(|error| error.to_string())
}

/// Explicit Run boundary shared by future UI controls and process proofs.
/// Only presentation state changes; the authoritative park is borrowed immutably.
pub(crate) fn run_selected(
    park: &Park,
    state: &mut CoasterTesting,
    selected: Option<RideId>,
) -> bool {
    state.run_pending = false;
    state.refresh(park, selected);
    if let Err(reason) = can_run(park, selected) {
        state.notice = Some(reason);
        return false;
    }
    match park.test_coaster() {
        Ok(report) => {
            state.report = Some(report);
            state.notice =
                Some("One mechanics lap complete. Passenger loading policy was excluded.".into());
            true
        }
        Err(error) => {
            state.report = None;
            state.notice = Some(error.to_string());
            false
        }
    }
}

pub(crate) fn status_text(park: &Park, state: &CoasterTesting, selected: Option<RideId>) -> String {
    can_run(park, selected).map_or_else(
        |reason| reason,
        |()| {
            if state.report.is_some() {
                "Installed coaster tested; estimates below."
            } else {
                "Closed, stationary and empty. Ready for one test lap."
            }
            .into()
        },
    )
}

pub(crate) fn metrics_text(state: &CoasterTesting) -> String {
    let Some(report) = &state.report else {
        return "Run a test to measure the installed track.\nDraft changes must be applied before testing.\nThe live train, guests and park clock stay unchanged.".into();
    };
    let seconds = |ticks| f64::from(ticks) / TICKS_PER_SECOND as f64;
    let speed = |value| f64::from(value) * TICKS_PER_SECOND as f64 / f64::from(TRACK_UNITS);
    format!(
        "Length {:.2} tiles / {} pieces\nLap {:.1} s + wait {:.1} s = {:.1} s\nSpeed {:.2}-{:.2} tiles/s while moving\nHeight {}-{} terrain steps\nCurves {} / lifts {} / drops {} / brakes {}\nClimb {} steps / descent {} steps\nLongest continuous drop {} steps\nPassenger loading policy excluded.",
        f64::from(report.length_units) / f64::from(TRACK_UNITS),
        report.layout.pieces,
        seconds(report.moving_ticks),
        seconds(u32::from(report.station_wait_ticks)),
        seconds(report.total_ticks),
        speed(report.min_speed_units_per_tick),
        speed(report.max_speed_units_per_tick),
        report.min_height,
        report.max_height,
        report.layout.curves,
        report.layout.lifts,
        report.layout.drops,
        report.layout.brakes,
        report.layout.ascent_steps,
        report.layout.descent_steps,
        report.layout.longest_drop_steps,
    )
}

pub(crate) fn ratings_text(state: &CoasterTesting) -> String {
    state.report.as_ref().map_or_else(
        || "Layout estimates / 0-100\nNo result yet.".into(),
        |report| {
            format!(
                "Layout estimates / 0-100\nExcitement {} / intensity {} / nausea {}",
                report.ratings.excitement, report.ratings.intensity, report.ratings.nausea,
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::{RideOperatingSettings, Speed, TilePos};

    fn fixture() -> (Park, RideId) {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        park.set_train_open(false);
        let id = park.ride().unwrap().id;
        (park, id)
    }

    #[test]
    fn explicit_test_repeats_deterministically_without_changing_authoritative_state() {
        let (park, id) = fixture();
        let before = park.clone();
        let bytes = park.to_save_bytes().unwrap();
        let mut state = CoasterTesting {
            run_pending: true,
            ..Default::default()
        };
        assert!(run_selected(&park, &mut state, Some(id)));
        let first = state.report.clone().unwrap();
        assert!(first.length_units > 0 && first.moving_ticks > 0);
        assert_eq!(
            first.total_ticks,
            first.moving_ticks + u32::from(first.station_wait_ticks)
        );
        assert!(!state.run_pending);
        assert!(run_selected(&park, &mut state, Some(id)));
        assert_eq!(state.report.as_ref(), Some(&first));
        assert_eq!(park, before);
        assert_eq!(park.to_save_bytes().unwrap(), bytes);
    }

    #[test]
    fn selected_carousel_unknown_open_and_closing_coasters_refuse_without_mutation() {
        let (mut park, coaster) = fixture();
        let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        for selected in [None, Some(carousel), Some(RideId(999))] {
            let before = park.clone();
            let mut state = CoasterTesting::default();
            assert!(!run_selected(&park, &mut state, selected));
            assert!(state.report.is_none());
            assert_eq!(park, before);
        }
        park.set_train_open(true);
        let mut state = CoasterTesting::default();
        let open = park.clone();
        assert!(!run_selected(&park, &mut state, Some(coaster)));
        assert_eq!(park, open);
        for _ in 0..100 {
            park.advance(Speed::Normal);
        }
        park.set_train_open(false);
        assert!(park.ride_is_running(coaster));
        let closing = park.clone();
        assert!(!run_selected(&park, &mut state, Some(coaster)));
        assert_eq!(park, closing);
    }

    #[test]
    fn cached_reports_follow_inputs_without_rerunning_for_unrelated_park_changes() {
        let (mut park, id) = fixture();
        let mut state = CoasterTesting::default();
        assert!(run_selected(&park, &mut state, Some(id)));
        let report = state.report.clone();
        park.advance(Speed::Normal);
        park.set_ride_ticket_price(id, 50).unwrap();
        state.refresh(&park, Some(id));
        assert_eq!(state.report, report);
        park.set_ride_operating_settings(
            id,
            RideOperatingSettings {
                station_wait_ticks: 120,
                ..Default::default()
            },
        )
        .unwrap();
        state.refresh(&park, Some(id));
        assert!(state.report.is_none());
        assert!(run_selected(&park, &mut state, Some(id)));
        let rail = park.circuit().unwrap().pieces[3].object;
        park.remove_object(rail).unwrap();
        state.refresh(&park, Some(id));
        assert!(state.report.is_none());
    }

    #[test]
    fn report_uses_game_units_and_labels_estimates_separately_from_test_metrics() {
        let (park, id) = fixture();
        let mut state = CoasterTesting::default();
        assert!(run_selected(&park, &mut state, Some(id)));
        let metrics = metrics_text(&state);
        assert!(metrics.contains("tiles/s") && metrics.contains("terrain steps"));
        assert!(metrics.contains("Lap ") && metrics.contains(" + wait "));
        assert!(!metrics.contains("km/h") && !metrics.contains("metres"));
        assert!(ratings_text(&state).starts_with("Layout estimates / 0-100"));
        assert!(status_text(&park, &state, Some(id)).contains("estimates"));
        state.refresh(&park, None);
        assert!(state.report.is_none());
    }
}

#[cfg(test)]
#[path = "coaster_testing_input_tests.rs"]
mod input_tests;
