//! Exercise service controls through the production input and persistence order.
use super::*;
use park_sim::{PathDirection, ServiceId, ServiceKind};

fn service_park() -> (Park, ServiceId, ServiceId) {
    let mut park = train_park();
    let food = park
        .place_service(
            TilePos { x: 10, y: 21 },
            8,
            PathDirection::North,
            ServiceKind::Food,
        )
        .unwrap();
    let drink = park
        .place_service(
            TilePos { x: 15, y: 21 },
            8,
            PathDirection::North,
            ServiceKind::Drink,
        )
        .unwrap();
    (park, food, drink)
}

fn service_app(directory: &InputSaveDirectory, park: Park, view: ViewState) -> (App, Entity) {
    let (mut app, window) = persistence_input_app(directory, park, view);
    app.add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                apply_path_actions,
                apply_train_actions,
                apply_price_actions,
                apply_service_actions,
            )
                .chain()
                .after(keyboard_input)
                .before(pointer_input),
        )
        .add_systems(
            Update,
            (ui::refresh, ui::guests::refresh, ui::services::refresh)
                .chain()
                .after(persistence::apply_save_action),
        );
    (app, window)
}

fn click(app: &mut App, action: ui::Action) {
    let entity = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, action))
        .id();
    frame(app);
    app.world_mut().despawn(entity);
}

#[test]
fn service_modal_consumes_open_and_close_frames_and_browses_without_world_edits() {
    for keyboard in [true, false] {
        let directory = InputSaveDirectory::new();
        let (park, food, drink) = service_park();
        let (mut app, _) = service_app(&directory, park.clone(), ViewState::default());
        frame(&mut app);
        let entity = (!keyboard).then(|| {
            app.world_mut()
                .spawn((Button, Interaction::Pressed, ui::Action::Services))
                .id()
        });
        let mut keys = vec![
            KeyCode::F7,
            KeyCode::KeyO,
            KeyCode::F5,
            KeyCode::Delete,
            KeyCode::KeyP,
        ];
        if keyboard {
            keys.push(KeyCode::KeyM);
        }
        persistence_key_frame(&mut app, &keys);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        assert!(app.world().resource::<ViewState>().services_open);
        assert_eq!(
            app.world().resource::<ViewState>().selected_service,
            Some(food)
        );
        if let Some(entity) = entity {
            app.world_mut().despawn(entity);
        }
        persistence_key_frame(
            &mut app,
            &[
                KeyCode::ArrowDown,
                KeyCode::PageUp,
                KeyCode::KeyE,
                KeyCode::KeyP,
                KeyCode::F9,
            ],
        );
        let view = app.world().resource::<ViewState>();
        assert_eq!(view.selected_service, Some(drink));
        assert_eq!(view.rotation, 0);
        assert_eq!(view.pan, Vec2::ZERO);
        assert_eq!(view.path_tool.height, 8);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        persistence_key_frame(
            &mut app,
            &[
                KeyCode::Escape,
                KeyCode::F7,
                KeyCode::KeyO,
                KeyCode::F5,
                KeyCode::Delete,
            ],
        );
        assert!(!app.world().resource::<ViewState>().services_open);
        frame(&mut app);
        assert_eq!(
            app.world().resource::<Simulation>().0,
            park,
            "no deferred mutation"
        );
        assert!(!directory.store().save_path().exists());
    }
}

#[test]
fn service_price_and_open_controls_target_the_selected_facility_once_while_paused() {
    let directory = InputSaveDirectory::new();
    let (park, food, drink) = service_park();
    let (mut app, _) = service_app(
        &directory,
        park.clone(),
        ViewState {
            services_open: true,
            selected_service: Some(food),
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    let entity = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::ServicePrice(1)))
        .id();
    persistence_key_frame(&mut app, &[KeyCode::F7]);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .service(food)
            .unwrap()
            .price_cents,
        300
    );
    frame(&mut app);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .service(food)
            .unwrap()
            .price_cents,
        300
    );
    app.world_mut().despawn(entity);
    persistence_key_frame(&mut app, &[KeyCode::F6, KeyCode::F7]);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .service(food)
            .unwrap()
            .price_cents,
        300
    );
    persistence_key_frame(&mut app, &[KeyCode::KeyO]);
    assert!(
        !app.world()
            .resource::<Simulation>()
            .0
            .service(food)
            .unwrap()
            .open
    );
    click(&mut app, ui::Action::ToggleService);
    let sim = &app.world().resource::<Simulation>().0;
    assert!(sim.service(food).unwrap().open);
    assert_eq!(sim.service(drink), park.service(drink));
    assert_eq!(sim.guests(), park.guests());
    assert_eq!(sim.ticks(), park.ticks());
    assert_eq!(sim.cash_cents(), park.cash_cents());
    persistence_key_frame(&mut app, &[KeyCode::KeyM]);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let saved = directory.store().load().unwrap();
    assert_eq!(saved, app.world().resource::<Simulation>().0);
}

#[test]
fn choosing_a_service_consumes_the_click_and_retains_kind_while_rotating() {
    let directory = InputSaveDirectory::new();
    let (park, _, _) = service_park();
    let (mut app, _) = service_app(
        &directory,
        park.clone(),
        ViewState {
            services_open: true,
            ..default()
        },
    );
    frame(&mut app);
    click(&mut app, ui::Action::BuildService(ServiceKind::Toilet));
    let view = app.world().resource::<ViewState>();
    assert!(!view.services_open);
    assert!(view.modal_input_blocked && view.ui_pointer_consumed);
    assert_eq!(
        view.path_tool.mode,
        path_tools::PathTool::Service(ServiceKind::Toilet)
    );
    persistence_key_frame(&mut app, &[KeyCode::KeyC, KeyCode::PageUp]);
    let view = app.world().resource::<ViewState>();
    assert_eq!(
        view.path_tool.mode,
        path_tools::PathTool::Service(ServiceKind::Toilet)
    );
    assert_eq!(view.path_tool.service_facing, PathDirection::East);
    assert_eq!(view.path_tool.height, 9);
    assert_eq!(app.world().resource::<Simulation>().0, park);
}

#[test]
fn service_commands_are_discarded_by_help_guests_capture_diagnostics_and_pending_load() {
    for blocked in 0..6 {
        let directory = InputSaveDirectory::new();
        let (park, food, _) = service_park();
        let (mut app, _) = service_app(
            &directory,
            park.clone(),
            ViewState {
                services_open: blocked != 5,
                help: blocked == 0,
                guests_open: blocked == 1,
                capture_active: blocked == 2,
                diagnostic_case: (blocked == 3)
                    .then_some(park_sim::diagnostics::DiagnosticCase::SaveScale),
                save_pending: (blocked == 4).then_some(SaveAction::Load),
                service_commands: vec![
                    ui::services::ServiceCommand::PriceSteps {
                        service: food,
                        steps: 1,
                    },
                    ui::services::ServiceCommand::SetOpen {
                        service: food,
                        open: false,
                    },
                ],
                ..default()
            },
        );
        frame(&mut app);
        assert_eq!(
            app.world().resource::<Simulation>().0,
            park,
            "case {blocked}"
        );
        assert!(
            app.world()
                .resource::<ViewState>()
                .service_commands
                .is_empty()
        );
        assert!(!directory.store().save_path().exists());
    }
}
