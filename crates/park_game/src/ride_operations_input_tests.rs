//! Actual Rides buttons and production input/save systems, without a GPU.
use super::*;
use crate::{SaveSession, Speed, keyboard_input, persistence};
use park_sim::RideOperatingSettings;
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
    let (coaster, carousel) = crate::save_check::ride_choice::install(&mut park).unwrap();
    park.set_ride_open(coaster, false).unwrap();
    park.set_ride_open(carousel, false).unwrap();
    (park, coaster, carousel)
}

#[test]
fn funds_closure_buttons_refuse_without_money_and_preserve_the_overdue_bill_after_recovery() {
    let (mut park, id, reserve) = crate::save_check::ride_costs::install().unwrap();
    crate::save_check::ride_costs::paid(&mut park, id).unwrap();
    crate::save_check::ride_costs::closed(&mut park, id).unwrap();
    let before = park.clone();
    let directory = SaveDirectory::new();
    let mut app = app(
        &directory,
        park,
        ViewState {
            rides_open: true,
            selected_ride: Some(id),
            speed: Speed::Paused,
            ..default()
        },
    );
    click(&mut app, OperationCommand::ToggleOpen);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_operating_notice
            .as_deref()
            .unwrap()
            .contains("overdue")
    );
    let mut labels = app.world_mut().query::<(&ui::rides::RideLabel, &Text)>();
    assert!(labels.iter(app.world()).any(|(kind, text)| matches!(
        kind,
        ui::rides::RideLabel::OperatingStatus
    ) && text.0.contains("Funds closure")));
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .remove_carousel(reserve)
        .unwrap();
    assert!(!app.world().resource::<Simulation>().0.ride_is_open(id));
    click(&mut app, OperationCommand::ToggleOpen);
    assert!(app.world().resource::<Simulation>().0.ride_is_open(id));
    assert_eq!(app.world().resource::<Simulation>().0.cash_cents(), 15_000);
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .advance(Speed::Normal);
    assert_eq!(app.world().resource::<Simulation>().0.cash_cents(), 14_900);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .ride_running_costs(id)
            .unwrap()
            .upkeep_cents,
        700
    );
    app.world()
        .resource::<Simulation>()
        .0
        .to_save_bytes()
        .unwrap();
}

#[test]
fn opening_buttons_and_o_reject_missing_access_atomically_and_close_damaged_rides() {
    for panel in [false, true] {
        for keyboard in [false, true] {
            if panel && keyboard {
                continue;
            }
            let directory = SaveDirectory::new();
            let mut park = crate::demonstration_park();
            park.set_train_open(false);
            let id = park.ride().unwrap().id;
            let before = park.clone();
            let mut app = app(
                &directory,
                park,
                ViewState {
                    rides_open: panel,
                    selected_ride: Some(id),
                    speed: Speed::Paused,
                    ..default()
                },
            );
            if keyboard {
                frame(&mut app, &[], &[KeyCode::KeyO]);
            } else {
                frame(
                    &mut app,
                    &[if panel {
                        ui::Action::RideOperating(OperationCommand::ToggleOpen)
                    } else {
                        ui::Action::ToggleTrain
                    }],
                    &[],
                );
            }
            assert_eq!(app.world().resource::<Simulation>().0, before);
            let view = app.world().resource::<ViewState>();
            let notice = if panel {
                view.ride_operating_notice.as_deref()
            } else {
                view.price_notice.as_deref()
            }
            .unwrap();
            assert!(notice.contains("entrance"), "{notice}");
        }
    }
    let (mut app, _directory, coaster, _) = fixture();
    click(&mut app, OperationCommand::ToggleOpen);
    let exit = app
        .world()
        .resource::<Simulation>()
        .0
        .ride_by_id(coaster)
        .unwrap()
        .exit
        .unwrap()
        .object;
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .remove_object(exit);
    click(&mut app, OperationCommand::ToggleOpen);
    assert!(!app.world().resource::<Simulation>().0.ride_is_open(coaster));
    let before = app.world().resource::<Simulation>().0.clone();
    click(&mut app, OperationCommand::ToggleOpen);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_operating_notice
            .as_deref()
            .unwrap()
            .contains("exit")
    );
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
                crate::coaster_testing::apply_commands,
                crate::access_planning::apply_commands,
                apply_commands,
                ui::appearance::apply,
                crate::apply_train_actions,
                persistence::apply_save_action,
                ui::rides::refresh,
                ui::appearance::refresh,
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
fn car_buttons_change_visible_capacity_once_invalidate_tests_and_survive_save_load() {
    let (mut app, _directory, coaster, carousel) = fixture();
    let report = app
        .world()
        .resource::<Simulation>()
        .0
        .test_coaster()
        .unwrap();
    app.world_mut()
        .resource_mut::<ViewState>()
        .coaster_testing
        .report = Some(report);
    let command = OperationCommand::Adjust(OperatingField::Cars, -1);
    frame(&mut app, &[ui::Action::RideOperating(command)], &[]);
    app.update();
    let park = &app.world().resource::<Simulation>().0;
    assert_eq!(park.coaster_car_count(coaster), Some(2));
    assert_eq!(park.ride_capacity(coaster), Some(4));
    assert_eq!(park.train_cars().len(), 2);
    assert_eq!(park.ride_capacity(carousel), Some(6));
    assert!(
        app.world()
            .resource::<ViewState>()
            .coaster_testing
            .report
            .is_none()
    );
    frame(&mut app, &[ui::Action::CloseRides], &[]);
    frame(&mut app, &[], &[]);
    frame(&mut app, &[], &[KeyCode::F5]);
    frame(&mut app, &[ui::Action::Rides], &[]);
    frame(&mut app, &[], &[]);
    click(&mut app, command);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .coaster_car_count(coaster),
        Some(1)
    );
    frame(&mut app, &[ui::Action::CloseRides], &[]);
    frame(&mut app, &[], &[]);
    frame(&mut app, &[], &[KeyCode::F9]);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .coaster_car_count(coaster),
        Some(2)
    );
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
}

#[test]
fn minimum_load_blocks_car_removal_and_reset_restores_three_cars_atomically() {
    let (mut app, _directory, coaster, carousel) = fixture();
    click(&mut app, OperationCommand::Adjust(OperatingField::Cars, -1));
    for _ in 0..4 {
        click(
            &mut app,
            OperationCommand::Adjust(OperatingField::MinimumPassengers, 1),
        );
    }
    let before = app.world().resource::<Simulation>().0.clone();
    assert_eq!(
        before
            .ride_dispatch_policy(coaster)
            .unwrap()
            .minimum_passengers,
        4
    );
    click(&mut app, OperationCommand::Adjust(OperatingField::Cars, -1));
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_operating_notice
            .as_ref()
            .unwrap()
            .contains("Lower minimum load")
    );
    click(&mut app, OperationCommand::ResetDefaults);
    let park = &app.world().resource::<Simulation>().0;
    assert_eq!(park.coaster_car_count(coaster), Some(3));
    assert_eq!(
        park.ride_dispatch_policy(coaster),
        Some(RideDispatchPolicy::default())
    );
    assert_eq!(
        park.ride_operating_settings(carousel),
        before.ride_operating_settings(carousel)
    );
    frame(&mut app, &[], &[KeyCode::ArrowDown]);
    assert!(!command_enabled(
        &app.world().resource::<Simulation>().0,
        app.world().resource::<ViewState>(),
        OperationCommand::Adjust(OperatingField::Cars, -1)
    ));
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
            let visible = matches!(
                field.0,
                OperatingField::StationWait
                    | OperatingField::MinimumPassengers
                    | OperatingField::MaxExtraWait
            ) || (carousel && field.0 == OperatingField::CycleDuration)
                || (!carousel
                    && matches!(
                        field.0,
                        OperatingField::Cars
                            | OperatingField::LiftSpeed
                            | OperatingField::BrakeSpeed
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
                ui::rides::RideLabel::OperatingValue(OperatingField::MinimumPassengers) => {
                    assert!(text.0.contains("No minimum"))
                }
                ui::rides::RideLabel::OperatingValue(OperatingField::MaxExtraWait) => {
                    assert!(text.0.contains("30 s") && text.0.contains("1-60 s"))
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

#[test]
fn generated_loading_controls_target_each_ride_once_and_save_both_policy_fields() {
    let (mut app, directory, coaster, carousel) = fixture();
    let mut expected = app.world().resource::<Simulation>().0.clone();
    let minimum = OperationCommand::Adjust(OperatingField::MinimumPassengers, 1);
    let wait = OperationCommand::Adjust(OperatingField::MaxExtraWait, 1);
    frame(&mut app, &[ui::Action::RideOperating(minimum)], &[]);
    app.update();
    frame(&mut app, &[], &[]);
    click(&mut app, wait);
    expected
        .set_ride_dispatch_policy(
            coaster,
            RideDispatchPolicy {
                minimum_passengers: 1,
                maximum_extra_wait_ticks: 930,
            },
        )
        .unwrap();
    assert_eq!(app.world().resource::<Simulation>().0, expected);
    frame(&mut app, &[], &[KeyCode::ArrowDown]);
    click(&mut app, minimum);
    expected
        .set_ride_dispatch_policy(
            carousel,
            RideDispatchPolicy {
                minimum_passengers: 1,
                ..default()
            },
        )
        .unwrap();
    assert_eq!(app.world().resource::<Simulation>().0, expected);
    frame(&mut app, &[], &[KeyCode::Escape]);
    frame(&mut app, &[], &[KeyCode::F5]);
    assert_eq!(directory.store().load().unwrap(), expected);
}

#[test]
fn loading_control_bounds_disable_and_preserve_park_without_wrapping() {
    for (policy, direction) in [
        (
            RideDispatchPolicy {
                minimum_passengers: 0,
                maximum_extra_wait_ticks: park_sim::MIN_EXTRA_WAIT_TICKS,
            },
            -1,
        ),
        (
            RideDispatchPolicy {
                minimum_passengers: park_sim::MAX_DISPATCH_PASSENGERS,
                maximum_extra_wait_ticks: park_sim::MAX_EXTRA_WAIT_TICKS,
            },
            1,
        ),
    ] {
        let (mut app, _directory, coaster, _) = fixture();
        app.world_mut()
            .resource_mut::<Simulation>()
            .0
            .set_ride_dispatch_policy(coaster, policy)
            .unwrap();
        frame(&mut app, &[], &[]);
        let before = app.world().resource::<Simulation>().0.clone();
        for field in [
            OperatingField::MinimumPassengers,
            OperatingField::MaxExtraWait,
        ] {
            let command = OperationCommand::Adjust(field, direction);
            assert!(!command_enabled(
                &before,
                app.world().resource::<ViewState>(),
                command
            ));
            click(&mut app, command);
            assert_eq!(app.world().resource::<Simulation>().0, before);
        }
    }
}

#[test]
fn combined_reset_is_atomic_while_open_or_moving_and_resets_both_domains_when_ready() {
    let (mut app, _directory, coaster, _) = fixture();
    {
        let mut sim = app.world_mut().resource_mut::<Simulation>();
        sim.0
            .set_ride_operating_settings(
                coaster,
                RideOperatingSettings {
                    station_wait_ticks: 30,
                    ..default()
                },
            )
            .unwrap();
        sim.0
            .set_ride_dispatch_policy(
                coaster,
                RideDispatchPolicy {
                    minimum_passengers: 6,
                    maximum_extra_wait_ticks: 30,
                },
            )
            .unwrap();
        sim.0.set_train_open(true);
    }
    let open = app.world().resource::<Simulation>().0.clone();
    click(&mut app, OperationCommand::ResetDefaults);
    assert_eq!(app.world().resource::<Simulation>().0, open);
    {
        let mut sim = app.world_mut().resource_mut::<Simulation>();
        for _ in 0..2_000 {
            if sim.0.ride_is_running(coaster) {
                break;
            }
            sim.0.advance(Speed::Normal);
        }
        sim.0.set_train_open(false);
        assert!(sim.0.ride_is_running(coaster));
    }
    let closing = app.world().resource::<Simulation>().0.clone();
    click(&mut app, OperationCommand::ResetDefaults);
    assert_eq!(app.world().resource::<Simulation>().0, closing);
    {
        let mut sim = app.world_mut().resource_mut::<Simulation>();
        for _ in 0..10_000 {
            if sim.0.ride_operating_settings_status(coaster).is_ok() {
                break;
            }
            sim.0.advance(Speed::Normal);
        }
        assert!(sim.0.ride_operating_settings_status(coaster).is_ok());
    }
    let mut expected = app.world().resource::<Simulation>().0.clone();
    expected
        .set_ride_operating_settings(coaster, default())
        .unwrap();
    expected
        .set_ride_dispatch_policy(coaster, default())
        .unwrap();
    click(&mut app, OperationCommand::ResetDefaults);
    assert_eq!(app.world().resource::<Simulation>().0, expected);
}

#[test]
fn tab_changes_and_load_intent_consume_loading_controls_before_any_policy_write() {
    for action in [
        ui::Action::RideAccess(crate::access_planning::AccessCommand::ShowAccess),
        ui::Action::CoasterTest(crate::coaster_testing::TestCommand::Show),
    ] {
        let (mut app, _directory, _, _) = fixture();
        let before = app.world().resource::<Simulation>().0.clone();
        frame(
            &mut app,
            &[
                action,
                ui::Action::RideOperating(OperationCommand::Adjust(
                    OperatingField::MinimumPassengers,
                    1,
                )),
            ],
            &[],
        );
        frame(&mut app, &[], &[]);
        assert_eq!(app.world().resource::<Simulation>().0, before);
    }
    for keys in [vec![KeyCode::F9], vec![KeyCode::ShiftLeft, KeyCode::F9]] {
        let (mut app, _directory, _, _) = fixture();
        let before = app.world().resource::<Simulation>().0.clone();
        frame(
            &mut app,
            &[ui::Action::RideOperating(OperationCommand::Adjust(
                OperatingField::MaxExtraWait,
                -1,
            ))],
            &keys,
        );
        frame(&mut app, &[], &[]);
        assert_eq!(app.world().resource::<Simulation>().0, before);
    }
}

#[test]
fn colors_controls_preserve_selected_owner_money_and_atomic_browse_or_tab_transitions() {
    use park_sim::RidePaint;
    use ui::appearance::{Command as C, Part};
    let (mut app, _directory, coaster, carousel) = fixture();
    let before = app.world().resource::<Simulation>().0.clone();
    frame(
        &mut app,
        &[
            ui::Action::RidePaint(C::Show),
            ui::Action::RideOperating(OperationCommand::ToggleOpen),
        ],
        &[],
    );
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert_eq!(
        app.world().resource::<ViewState>().ride_tab,
        ui::rides::RideTab::Colors
    );
    frame(
        &mut app,
        &[ui::Action::RidePaint(C::Step(Part::Structure, 1))],
        &[],
    );
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .ride_appearance(coaster)
            .unwrap()
            .structure,
        RidePaint::Coral
    );
    assert_eq!(
        app.world().resource::<Simulation>().0.cash_cents(),
        before.cash_cents()
    );
    let painted = app.world().resource::<Simulation>().0.clone();
    frame(
        &mut app,
        &[
            ui::Action::BrowseRide(1),
            ui::Action::RidePaint(C::Step(Part::Vehicles, 1)),
        ],
        &[],
    );
    assert_eq!(
        app.world().resource::<ViewState>().selected_ride,
        Some(carousel)
    );
    assert_eq!(app.world().resource::<Simulation>().0, painted);
    frame(
        &mut app,
        &[
            ui::Action::RidePaint(C::Step(Part::Structure, 1)),
            ui::Action::RidePaint(C::Step(Part::Vehicles, 1)),
        ],
        &[],
    );
    assert_eq!(app.world().resource::<Simulation>().0, painted);
    frame(
        &mut app,
        &[ui::Action::RidePaint(C::Step(Part::Vehicles, 1))],
        &[],
    );
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .ride_appearance(carousel)
            .unwrap()
            .vehicles,
        RidePaint::Coral
    );
    frame(&mut app, &[ui::Action::RidePaint(C::Reset)], &[]);
    assert_eq!(app.world().resource::<Simulation>().0, painted);
    frame(&mut app, &[ui::Action::BrowseRide(-1)], &[]);
    frame(&mut app, &[ui::Action::RidePaint(C::Reset)], &[]);
    assert_eq!(app.world().resource::<Simulation>().0, before);
}
#[test]
fn color_tab_switch_close_and_capture_cannot_apply_conflicting_paint() {
    use ui::appearance::{Command as C, Part};
    let (mut app, _directory, _, _) = fixture();
    let before = app.world().resource::<Simulation>().0.clone();
    frame(
        &mut app,
        &[
            ui::Action::RidePaint(C::Show),
            ui::Action::RidePaint(C::Step(Part::Structure, 1)),
        ],
        &[],
    );
    assert_eq!(app.world().resource::<Simulation>().0, before);
    frame(&mut app, &[ui::Action::RidePaint(C::Show)], &[]);
    frame(
        &mut app,
        &[
            ui::Action::RideAccess(crate::access_planning::AccessCommand::ShowAccess),
            ui::Action::RidePaint(C::Step(Part::Structure, 1)),
        ],
        &[],
    );
    assert_eq!(app.world().resource::<Simulation>().0, before);
    frame(&mut app, &[ui::Action::RidePaint(C::Show)], &[]);
    frame(
        &mut app,
        &[
            ui::Action::CloseRides,
            ui::Action::RidePaint(C::Step(Part::Structure, 1)),
        ],
        &[],
    );
    assert!(!app.world().resource::<ViewState>().rides_open);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    {
        let mut v = app.world_mut().resource_mut::<ViewState>();
        v.rides_open = true;
        v.ride_tab = ui::rides::RideTab::Colors;
        v.capture_active = true;
    }
    frame(
        &mut app,
        &[ui::Action::RidePaint(C::Step(Part::Structure, 1))],
        &[],
    );
    assert_eq!(app.world().resource::<Simulation>().0, before);
}
#[test]
fn painted_ride_can_be_shown_and_saved_without_changing_pause_or_gameplay() {
    use ui::appearance::{Command as C, Part};
    let (mut app, directory, coaster, _) = fixture();
    frame(&mut app, &[ui::Action::RidePaint(C::Show)], &[]);
    frame(
        &mut app,
        &[ui::Action::RidePaint(C::Step(Part::Structure, 1))],
        &[],
    );
    let painted = app.world().resource::<Simulation>().0.clone();
    frame(&mut app, &[ui::Action::RidePaint(C::Find)], &[]);
    let v = app.world().resource::<ViewState>();
    assert!(!v.rides_open);
    assert_eq!(v.speed, Speed::Paused);
    assert_eq!(v.selected_ride, Some(coaster));
    assert_eq!(app.world().resource::<Simulation>().0, painted);
    // The close action deliberately consumes one transition frame.
    frame(&mut app, &[], &[]);
    frame(&mut app, &[], &[KeyCode::F5]);
    assert_eq!(directory.store().load().unwrap(), painted);
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .set_ride_appearance(park_sim::RideAppearance {
            ride: coaster,
            ..Default::default()
        })
        .unwrap();
    frame(&mut app, &[], &[KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, painted);
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
}
