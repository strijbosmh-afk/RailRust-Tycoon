//! Generated controls through the production modal/controller/save ordering.
use super::*;
use crate::ui::rides::RideTab;
use crate::{SaveSession, Speed, access_planning, keyboard_input, persistence, ride_operations};
use park_sim::{RideDispatchPolicy, RideOperatingSettings, TilePos};
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "railrust-test-input-{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }

    fn store(&self) -> crate::save_store::SaveStore {
        crate::save_store::SaveStore::new(self.0.clone())
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        assert!(self.0.starts_with(std::env::temp_dir()));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> (App, Directory, RideId, RideId) {
    let directory = Directory::new();
    let mut park = crate::demonstration_park();
    park.set_train_open(false);
    let coaster = park.ride().unwrap().id;
    let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(ViewState {
            rides_open: true,
            ride_tab: RideTab::Test,
            selected_ride: Some(coaster),
            speed: Speed::Paused,
            ..default()
        })
        .insert_resource(SaveSession::from_arguments([
            OsString::from("--save-dir"),
            directory.0.clone().into_os_string(),
        ]))
        .insert_resource(Time::<()>::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                ui::button_actions,
                keyboard_input,
                apply_commands,
                access_planning::apply_commands,
                ride_operations::apply_commands,
                persistence::apply_save_action,
                ui::rides::refresh,
                ui::refresh,
            )
                .chain(),
        );
    app.update();
    (app, directory, coaster, carousel)
}

fn frame(app: &mut App, actions: &[ui::Action], keys: &[KeyCode]) {
    let mut buttons = app.world_mut().query::<(&ui::Action, &mut Interaction)>();
    for (action, mut interaction) in buttons.iter_mut(app.world_mut()) {
        *interaction = if actions.contains(action) {
            Interaction::Pressed
        } else {
            Interaction::None
        };
    }
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for key in keys {
        input.press(*key);
    }
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
}

fn click(app: &mut App, command: TestCommand) {
    frame(app, &[ui::Action::CoasterTest(command)], &[]);
    frame(app, &[], &[]);
}

#[test]
fn generated_run_button_is_readonly_and_a_held_press_does_not_run_again() {
    let (mut app, _directory, _, _) = fixture();
    let before = app.world().resource::<Simulation>().0.clone();
    let bytes = before.to_save_bytes().unwrap();
    frame(&mut app, &[ui::Action::CoasterTest(TestCommand::Run)], &[]);
    assert!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .report
            .is_some()
    );
    app.world_mut()
        .resource_mut::<ViewState>()
        .coaster_testing
        .notice = Some("Retained while held".into());
    app.update();
    assert_eq!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .notice
            .as_deref(),
        Some("Retained while held")
    );
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .unwrap(),
        bytes
    );
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
}

#[test]
fn three_tabs_are_exclusive_and_selected_tab_remains_highlighted_in_captures() {
    let (mut app, _directory, _, _) = fixture();
    for (tab, action) in [
        (
            RideTab::Access,
            ui::Action::RideAccess(access_planning::AccessCommand::ShowAccess),
        ),
        (
            RideTab::Operations,
            ui::Action::RideAccess(access_planning::AccessCommand::ShowOperations),
        ),
        (RideTab::Test, ui::Action::CoasterTest(TestCommand::Show)),
    ] {
        frame(&mut app, &[action], &[]);
        frame(&mut app, &[], &[]);
        assert_eq!(app.world().resource::<ViewState>().ride_tab, tab);
        let mut sections = app.world_mut().query::<(&ui::rides::RideSection, &Node)>();
        for (section, node) in sections.iter(app.world()) {
            assert_eq!(
                node.display,
                if section.0 == tab {
                    Display::Flex
                } else {
                    Display::None
                }
            );
        }
        let mut colors = app.world_mut().query::<(&ui::Action, &BackgroundColor)>();
        let selected_color = colors
            .iter(app.world())
            .find(|(candidate, _)| **candidate == action)
            .unwrap()
            .1
            .0;
        app.world_mut().resource_mut::<ViewState>().capture_active = true;
        frame(&mut app, &[], &[]);
        let mut colors = app.world_mut().query::<(&ui::Action, &BackgroundColor)>();
        let color = colors
            .iter(app.world())
            .find(|(candidate, _)| **candidate == action)
            .unwrap()
            .1;
        assert_eq!(color.0, selected_color);
        let inactive_tab_color = colors
            .iter(app.world())
            .find(|(candidate, _)| {
                **candidate != action
                    && matches!(
                        candidate,
                        ui::Action::RideAccess(
                            access_planning::AccessCommand::ShowAccess
                                | access_planning::AccessCommand::ShowOperations
                        ) | ui::Action::CoasterTest(TestCommand::Show)
                    )
            })
            .unwrap()
            .1
            .0;
        assert_ne!(selected_color, inactive_tab_color);
        app.world_mut().resource_mut::<ViewState>().capture_active = false;
    }
}

#[test]
fn same_frame_tab_changes_browsing_modal_keys_and_load_keys_preempt_run() {
    let transitions = [
        (
            Some(ui::Action::RideAccess(
                access_planning::AccessCommand::ShowOperations,
            )),
            vec![],
        ),
        (
            Some(ui::Action::RideAccess(
                access_planning::AccessCommand::ShowAccess,
            )),
            vec![],
        ),
        (Some(ui::Action::CloseRides), vec![]),
        (Some(ui::Action::BuildCarousel), vec![]),
        (Some(ui::Action::EditTrack), vec![]),
        (Some(ui::Action::BrowseRide(1)), vec![]),
        (None, vec![KeyCode::ArrowDown]),
        (None, vec![KeyCode::Escape]),
        (None, vec![KeyCode::KeyY]),
        (None, vec![KeyCode::KeyZ]),
        (None, vec![KeyCode::F9]),
        (None, vec![KeyCode::ShiftLeft, KeyCode::F9]),
    ];
    for (action, keys) in transitions {
        let (mut app, _directory, _, _) = fixture();
        let before = app.world().resource::<Simulation>().0.clone();
        let mut actions = vec![ui::Action::CoasterTest(TestCommand::Run)];
        actions.extend(action);
        frame(&mut app, &actions, &keys);
        frame(&mut app, &[], &[]);
        assert_eq!(app.world().resource::<Simulation>().0, before);
        let state = &app.world().resource::<ViewState>().coaster_testing;
        assert!(
            state.report.is_none() && !state.run_pending && !state.show_pending,
            "pending test survived conflicting controls, keys={keys:?}"
        );
    }
}

#[test]
fn hidden_readonly_and_load_states_drain_pending_test_requests() {
    for case in 0..10 {
        let (mut app, _directory, _, _) = fixture();
        let before = app.world().resource::<Simulation>().0.clone();
        {
            let mut view = app.world_mut().resource_mut::<ViewState>();
            match case {
                0 => view.capture_active = true,
                1 => view.diagnostic_case = Some(park_sim::diagnostics::DiagnosticCase::SaveScale),
                2 => view.help = true,
                3 => view.designs_open = true,
                4 => view.rides_open = false,
                5 => view.save_pending = Some(SaveAction::Load),
                6 => view.save_pending = Some(SaveAction::Backup),
                7 => view.modal_transition_pending = true,
                8 => view.ride_tab = RideTab::Operations,
                _ => view.ride_tab = RideTab::Access,
            }
            queue(&mut view, TestCommand::Run);
        }
        frame(&mut app, &[], &[]);
        let state = &app.world().resource::<ViewState>().coaster_testing;
        assert!(state.report.is_none() && !state.run_pending, "case {case}");
        assert_eq!(app.world().resource::<Simulation>().0, before);
    }
}

#[test]
fn test_tab_transition_discards_competing_tuning_and_never_runs_implicitly() {
    let (mut app, _directory, _, _) = fixture();
    app.world_mut().resource_mut::<ViewState>().ride_tab = RideTab::Operations;
    let before = app.world().resource::<Simulation>().0.clone();
    frame(
        &mut app,
        &[
            ui::Action::CoasterTest(TestCommand::Show),
            ui::Action::CoasterTest(TestCommand::Run),
            ui::Action::RideOperating(ride_operations::OperationCommand::Adjust(
                ride_operations::OperatingField::StationWait,
                1,
            )),
        ],
        &[],
    );
    assert_eq!(app.world().resource::<ViewState>().ride_tab, RideTab::Test);
    assert!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .report
            .is_none()
    );
    assert_eq!(app.world().resource::<Simulation>().0, before);
}

#[test]
fn cache_tracks_mechanical_inputs_and_selection_but_excludes_loading_policy() {
    let (mut app, _directory, coaster, carousel) = fixture();
    click(&mut app, TestCommand::Run);
    let report = app
        .world()
        .resource::<ViewState>()
        .coaster_testing
        .report
        .clone()
        .unwrap();
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .set_ride_dispatch_policy(
            coaster,
            RideDispatchPolicy {
                minimum_passengers: 6,
                maximum_extra_wait_ticks: 1800,
            },
        )
        .unwrap();
    frame(&mut app, &[], &[]);
    assert_eq!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .report
            .as_ref(),
        Some(&report)
    );
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .set_ride_operating_settings(
            coaster,
            RideOperatingSettings {
                station_wait_ticks: 120,
                ..default()
            },
        )
        .unwrap();
    frame(&mut app, &[], &[]);
    assert!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .report
            .is_none()
    );
    click(&mut app, TestCommand::Run);
    frame(&mut app, &[], &[KeyCode::ArrowDown]);
    assert_eq!(
        app.world().resource::<ViewState>().selected_ride,
        Some(carousel)
    );
    assert!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .report
            .is_none()
    );
    let before = app.world().resource::<Simulation>().0.clone();
    click(&mut app, TestCommand::Run);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .notice
            .as_deref()
            .unwrap()
            .contains("carousel")
    );
    frame(&mut app, &[], &[KeyCode::ArrowUp]);
    click(&mut app, TestCommand::Run);
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .set_train_open(true);
    frame(&mut app, &[], &[]);
    assert!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .report
            .is_none()
    );
}

#[test]
fn loading_identical_park_and_backup_clear_cached_report_notice_and_pending_actions() {
    for action in [SaveAction::Load, SaveAction::Backup] {
        let (mut app, directory, _, _) = fixture();
        let saved = app.world().resource::<Simulation>().0.clone();
        directory.store().save(&saved).unwrap();
        directory.store().save(&saved).unwrap();
        click(&mut app, TestCommand::Run);
        frame(&mut app, &[], &[KeyCode::Escape]);
        {
            let mut view = app.world_mut().resource_mut::<ViewState>();
            view.coaster_testing.run_pending = true;
            view.coaster_testing.show_pending = true;
        }
        frame(
            &mut app,
            &[],
            if action == SaveAction::Backup {
                &[KeyCode::ShiftLeft, KeyCode::F9]
            } else {
                &[KeyCode::F9]
            },
        );
        assert_eq!(app.world().resource::<Simulation>().0, saved);
        let view = app.world().resource::<ViewState>();
        assert_eq!(view.ride_tab, RideTab::Operations);
        assert!(!view.rides_open && view.selected_ride.is_none());
        assert!(view.coaster_testing.report.is_none() && view.coaster_testing.notice.is_none());
        assert!(!view.coaster_testing.run_pending && !view.coaster_testing.show_pending);
    }
}
