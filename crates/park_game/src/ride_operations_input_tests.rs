//! Actual Rides buttons and production input/save systems, without a GPU.
use super::*;
use crate::{SaveSession, Speed, keyboard_input, persistence};
use park_sim::{RideOperatingSettings, TilePos};
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct SaveDirectory(PathBuf);

impl SaveDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "railrust-operations-{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }

    fn session(&self) -> SaveSession {
        SaveSession::from_arguments([
            OsString::from("--save-dir"),
            self.0.clone().into_os_string(),
        ])
    }

    fn store(&self) -> crate::save_store::SaveStore {
        crate::save_store::SaveStore::new(self.0.clone())
    }
}

impl Drop for SaveDirectory {
    fn drop(&mut self) {
        assert!(self.0.starts_with(std::env::temp_dir()));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn two_rides() -> (Park, RideId, RideId) {
    let mut park = crate::demonstration_park();
    park.set_train_open(false);
    let coaster = park.ride().unwrap().id;
    let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    (park, coaster, carousel)
}

fn app(directory: &SaveDirectory, park: Park, view: ViewState) -> App {
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(view)
        .insert_resource(directory.session())
        .insert_resource(Time::<()>::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                ui::button_actions,
                keyboard_input,
                apply_commands,
                persistence::apply_save_action,
                ui::rides::refresh,
                ui::refresh,
            )
                .chain(),
        );
    app.update();
    app
}

fn fixture() -> (App, SaveDirectory, RideId, RideId) {
    let directory = SaveDirectory::new();
    let (park, coaster, carousel) = two_rides();
    let app = app(
        &directory,
        park,
        ViewState {
            rides_open: true,
            selected_ride: Some(coaster),
            speed: Speed::Paused,
            ..default()
        },
    );
    (app, directory, coaster, carousel)
}

fn buttons(app: &mut App, actions: &[ui::Action]) {
    let mut query = app.world_mut().query::<(&ui::Action, &mut Interaction)>();
    for (action, mut interaction) in query.iter_mut(app.world_mut()) {
        *interaction = if actions.contains(action) {
            Interaction::Pressed
        } else {
            Interaction::None
        };
    }
}

fn frame(app: &mut App, actions: &[ui::Action], keys: &[KeyCode]) {
    buttons(app, actions);
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

fn click(app: &mut App, command: OperationCommand) {
    frame(app, &[ui::Action::RideOperating(command)], &[]);
    frame(app, &[], &[]);
}

fn settings(app: &App, id: RideId) -> RideOperatingSettings {
    app.world()
        .resource::<Simulation>()
        .0
        .ride_operating_settings(id)
        .unwrap()
}

#[test]
fn visible_settings_buttons_target_selected_ride_once_and_preserve_other_park_state() {
    let (mut app, _directory, coaster, carousel) = fixture();
    let mut expected = app.world().resource::<Simulation>().0.clone();
    let command = OperationCommand::Adjust(OperatingField::StationWait, 1);
    let mut requested = RideOperatingSettings::default();
    requested.station_wait_ticks += park_sim::STATION_WAIT_STEP_TICKS;
    expected
        .set_ride_operating_settings(coaster, requested)
        .unwrap();
    frame(&mut app, &[ui::Action::RideOperating(command)], &[]);
    app.update(); // A held button must not repeat an adjustment.
    assert_eq!(app.world().resource::<Simulation>().0, expected);
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
    assert_eq!(settings(&app, carousel), RideOperatingSettings::default());
    frame(&mut app, &[], &[KeyCode::ArrowDown]);
    assert_eq!(
        app.world().resource::<ViewState>().selected_ride,
        Some(carousel)
    );
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_operating_notice
            .is_none()
    );
    click(
        &mut app,
        OperationCommand::Adjust(OperatingField::CycleDuration, 1),
    );
    let mut carousel_settings = RideOperatingSettings::default();
    carousel_settings.cycle_ticks += park_sim::CAROUSEL_CYCLE_STEP_TICKS;
    expected
        .set_ride_operating_settings(carousel, carousel_settings)
        .unwrap();
    assert_eq!(app.world().resource::<Simulation>().0, expected);
    // A synthetic press on a hidden, unsupported row must also be harmless.
    click(
        &mut app,
        OperationCommand::Adjust(OperatingField::LiftSpeed, 1),
    );
    assert_eq!(app.world().resource::<Simulation>().0, expected);
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_operating_notice
            .as_deref()
            .unwrap()
            .contains("does not apply")
    );
}

#[test]
fn tuning_bounds_reset_and_open_close_use_authoritative_simulation_guards() {
    let (mut app, _directory, coaster, _) = fixture();
    let max = RideOperatingSettings {
        station_wait_ticks: park_sim::MAX_STATION_WAIT_TICKS,
        ..default()
    };
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .set_ride_operating_settings(coaster, max)
        .unwrap();
    frame(&mut app, &[], &[]);
    let increase = OperationCommand::Adjust(OperatingField::StationWait, 1);
    assert!(!command_enabled(
        &app.world().resource::<Simulation>().0,
        app.world().resource::<ViewState>(),
        increase
    ));
    let before = app.world().resource::<Simulation>().0.clone();
    click(&mut app, increase);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    click(&mut app, OperationCommand::ResetDefaults);
    assert_eq!(settings(&app, coaster), RideOperatingSettings::default());
    click(&mut app, OperationCommand::ToggleOpen);
    let open = app.world().resource::<Simulation>().0.clone();
    assert!(open.ride_is_open(coaster));
    click(&mut app, increase);
    assert_eq!(app.world().resource::<Simulation>().0, open);
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_operating_notice
            .as_deref()
            .unwrap()
            .contains("Close the ride")
    );
    // No-op defaults preserve an operating ride, as specified by the sim API.
    click(&mut app, OperationCommand::ResetDefaults);
    assert_eq!(app.world().resource::<Simulation>().0, open);
    // An open/close command wins over settings pressed in the same frame.
    frame(
        &mut app,
        &[
            ui::Action::RideOperating(increase),
            ui::Action::RideOperating(OperationCommand::ToggleOpen),
        ],
        &[],
    );
    assert!(!app.world().resource::<Simulation>().0.ride_is_open(coaster));
    assert_eq!(settings(&app, coaster), RideOperatingSettings::default());
    frame(&mut app, &[], &[]);
    click(&mut app, increase);
    assert_eq!(settings(&app, coaster).station_wait_ticks, 120);
}

#[test]
fn modal_transitions_browsing_and_load_intent_discard_same_frame_settings() {
    let changes = [
        (Some(ui::Action::CloseRides), None),
        (Some(ui::Action::BuildCarousel), None),
        (Some(ui::Action::EditTrack), None),
        (Some(ui::Action::BrowseRide(1)), None),
        (None, Some(KeyCode::Escape)),
        (None, Some(KeyCode::KeyY)),
        (None, Some(KeyCode::KeyZ)),
        (None, Some(KeyCode::ArrowDown)),
        (None, Some(KeyCode::F9)),
    ];
    for (action, key) in changes {
        let (mut app, _directory, _, _) = fixture();
        let before = app.world().resource::<Simulation>().0.clone();
        let mut actions = vec![ui::Action::RideOperating(OperationCommand::Adjust(
            OperatingField::StationWait,
            1,
        ))];
        actions.extend(action);
        frame(&mut app, &actions, &key.into_iter().collect::<Vec<_>>());
        assert_eq!(app.world().resource::<Simulation>().0, before);
        assert!(
            app.world()
                .resource::<ViewState>()
                .ride_operating_commands
                .is_empty()
        );
        frame(&mut app, &[], &[]);
        assert_eq!(app.world().resource::<Simulation>().0, before);
    }
}

#[test]
fn readonly_modes_other_modals_and_pending_loads_consume_queued_settings() {
    for case in 0..8 {
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
                _ => view.modal_transition_pending = true,
            }
            queue(
                &mut view,
                OperationCommand::Adjust(OperatingField::StationWait, 1),
            );
        }
        frame(&mut app, &[], &[]);
        assert_eq!(
            app.world().resource::<Simulation>().0,
            before,
            "case {case}"
        );
        assert!(
            app.world()
                .resource::<ViewState>()
                .ride_operating_commands
                .is_empty()
        );
    }
}

#[test]
fn saving_after_panel_close_keeps_settings_and_loading_resets_transient_ui() {
    let (mut app, directory, coaster, _) = fixture();
    click(
        &mut app,
        OperationCommand::Adjust(OperatingField::StationWait, 1),
    );
    let saved = app.world().resource::<Simulation>().0.clone();
    frame(&mut app, &[], &[KeyCode::F5]);
    assert!(
        !directory.store().save_path().exists(),
        "save shortcut is captured inside the modal"
    );
    frame(&mut app, &[], &[KeyCode::Escape]);
    frame(&mut app, &[], &[KeyCode::F5]);
    assert_eq!(directory.store().load().unwrap(), saved);
    frame(&mut app, &[], &[KeyCode::KeyY]);
    frame(&mut app, &[], &[]);
    click(
        &mut app,
        OperationCommand::Adjust(OperatingField::StationWait, 1),
    );
    assert_eq!(settings(&app, coaster).station_wait_ticks, 150);
    frame(&mut app, &[], &[KeyCode::Escape]);
    // A stale queued adjustment must not be replayed into the loaded park.
    queue(
        &mut app.world_mut().resource_mut::<ViewState>(),
        OperationCommand::Adjust(OperatingField::StationWait, 1),
    );
    frame(&mut app, &[], &[KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, saved);
    let view = app.world().resource::<ViewState>();
    assert!(!view.rides_open);
    assert_eq!(view.speed, Speed::Paused);
    assert!(view.selected_ride.is_none());
    assert!(view.ride_operating_notice.is_none() && view.ride_operating_commands.is_empty());
}

#[test]
fn operating_rows_show_family_specific_units_and_panel_fits_minimum_window() {
    const {
        assert!(ui::rides::PANEL_WIDTH <= 1024.0);
        assert!(ui::rides::PANEL_HEIGHT <= 680.0 - crate::HEADER - crate::FOOTER);
    }
    let (mut app, _directory, _, _) = fixture();
    for carousel in [false, true] {
        if carousel {
            frame(&mut app, &[], &[KeyCode::ArrowDown]);
        }
        let mut rows = app.world_mut().query::<(&ui::rides::OperatingRow, &Node)>();
        for (field, node) in rows.iter(app.world()) {
            let visible = field.0 == OperatingField::StationWait
                || (carousel && field.0 == OperatingField::CycleDuration)
                || (!carousel
                    && matches!(
                        field.0,
                        OperatingField::LiftSpeed | OperatingField::BrakeSpeed
                    ));
            assert_eq!(
                node.display,
                if visible {
                    Display::Flex
                } else {
                    Display::None
                }
            );
        }
        let mut labels = app.world_mut().query::<(&ui::rides::RideLabel, &Text)>();
        for (label, text) in labels.iter(app.world()) {
            match label {
                ui::rides::RideLabel::OperatingValue(OperatingField::StationWait) => {
                    assert!(text.0.contains("3 s"))
                }
                ui::rides::RideLabel::OperatingValue(
                    OperatingField::LiftSpeed | OperatingField::BrakeSpeed,
                ) => assert!(text.0.contains("tiles/s")),
                ui::rides::RideLabel::OperatingValue(OperatingField::CycleDuration) => {
                    assert!(text.0.contains("24 s"))
                }
                _ => {}
            }
        }
    }
}

#[test]
fn removed_selection_never_redirects_queued_settings_to_another_ride() {
    let directory = SaveDirectory::new();
    let (mut park, _, carousel) = two_rides();
    park.remove_carousel(carousel).unwrap();
    let before = park.clone();
    let mut view = ViewState {
        rides_open: true,
        selected_ride: Some(carousel),
        ..default()
    };
    queue(
        &mut view,
        OperationCommand::Adjust(OperatingField::StationWait, 1),
    );
    let app = app(&directory, park, view);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_operating_commands
            .is_empty()
    );
}

#[test]
fn faulted_coaster_open_control_refuses_without_false_success_notice() {
    let (mut app, _directory, coaster, _) = fixture();
    let rail = app
        .world()
        .resource::<Simulation>()
        .0
        .circuit()
        .unwrap()
        .pieces[3]
        .object;
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .remove_object(rail)
        .unwrap();
    frame(&mut app, &[], &[]);
    let before = app.world().resource::<Simulation>().0.clone();
    assert!(!command_enabled(
        &before,
        app.world().resource::<ViewState>(),
        OperationCommand::ToggleOpen
    ));
    click(&mut app, OperationCommand::ToggleOpen);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(!before.ride_is_open(coaster));
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_operating_notice
            .as_deref()
            .unwrap()
            .contains("Restore this ride")
    );
}
