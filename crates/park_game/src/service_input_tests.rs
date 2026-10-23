//! Exercise service controls through the production input and persistence order.
use super::*;
use park_sim::{PathDirection, ServiceId, ServiceKind};

fn label(app: &mut App, matches: impl Fn(&ui::services::ServiceLabel) -> bool) -> String {
    let world = app.world_mut();
    let mut labels = world.query::<(&ui::services::ServiceLabel, &Text)>();
    labels
        .iter(world)
        .find(|(kind, _)| matches(kind))
        .unwrap()
        .1
        .0
        .clone()
}

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

#[test]
fn service_mouse_purchase_and_delete_save_exactly_one_charge_and_refund_while_paused() {
    let directory = InputSaveDirectory::new();
    let original = Park::new(MAP_SIZE, MAP_SIZE);
    let pos = TilePos { x: 14, y: 17 };
    let (mut app, window) = service_app(
        &directory,
        original.clone(),
        ViewState {
            speed: Speed::Paused,
            zoom: 1.4,
            path_tool: path_tools::ToolState {
                mode: path_tools::PathTool::Service(ServiceKind::Food),
                ..default()
            },
            ..default()
        },
    );
    let point = world::path_build_center(pos, &original, 0, 8, park_sim::PathSlope::Flat).unwrap();
    let cursor = cursor_for_world(&app, window, point);
    set_cursor(&mut app, window, Some(cursor));
    frame(&mut app);
    assert_eq!(app.world().resource::<Simulation>().0, original);
    assert!(
        app.world()
            .resource::<ViewState>()
            .path_tool
            .hint
            .contains("Build $120.00 / Cash $1000.00")
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let built = app.world().resource::<Simulation>().0.clone();
    assert_eq!(built.cash_cents(), 88_000);
    assert_eq!(built.services()[0].construction_cost_cents, 12_000);
    assert_eq!(built.finance_ledger().entries.len(), 1);
    assert_eq!(directory.store().load().unwrap(), built);
    frame(&mut app);
    assert_eq!(
        app.world().resource::<Simulation>().0,
        built,
        "held mouse never paints a second purchase"
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    persistence_key_frame(&mut app, &[KeyCode::Delete, KeyCode::F5]);
    let removed = app.world().resource::<Simulation>().0.clone();
    assert_eq!(removed.cash_cents(), 94_000);
    assert!(removed.services()[0].object.is_none());
    assert_eq!(removed.services()[0].demolition_refund_cents, 6_000);
    assert_eq!(removed.finance_ledger().entries.len(), 2);
    assert_eq!(removed.ticks(), 0);
    assert_eq!(directory.store().load().unwrap(), removed);
    assert_eq!(directory.store().load_backup().unwrap(), built);
    persistence_key_frame(&mut app, &[KeyCode::Delete]);
    assert_eq!(app.world().resource::<Simulation>().0, removed);
}

#[test]
fn service_panel_distinguishes_operating_profit_from_purchase_net_cash_and_shows_refund() {
    let directory = InputSaveDirectory::new();
    let (park, food, _) = service_park();
    let (mut app, window) = service_app(
        &directory,
        park.clone(),
        ViewState {
            services_open: true,
            selected_service: Some(food),
            speed: Speed::Paused,
            ..default()
        },
    );
    *app.world_mut().get_mut::<Window>(window).unwrap() = test_window(1024, 680, 2.0);
    frame(&mut app);
    let details = label(&mut app, |kind| {
        matches!(kind, ui::services::ServiceLabel::Details)
    });
    assert!(
        details.contains("Operating profit +$0.00 / Net cash -$120.00"),
        "{details}"
    );
    assert!(details.contains("Paid $120.00 / Demolition refund $60.00"));
    assert_eq!(app.world().resource::<Simulation>().0, park);
    const {
        assert!(ui::services::PANEL_WIDTH <= 1024.0);
        assert!(ui::services::PANEL_HEIGHT <= 680.0 - HEADER - FOOTER);
    }
    let world = app.world_mut();
    let mut texts = world.query::<&Text>();
    for caption in [
        "Build Food $120.00",
        "Build Drinks $80.00",
        "Build Toilets $100.00",
    ] {
        assert!(
            texts.iter(world).any(|text| text.0 == caption),
            "missing {caption}"
        );
    }
}

#[test]
fn removing_a_facility_during_use_refunds_only_the_build_and_cancels_without_guest_payment() {
    let mut park = train_park();
    crate::save_check::services::install(&mut park).unwrap();
    let mut using = None;
    for _ in 0..24_000 {
        using =
            crate::save_check::services::guest_in_phase(&park, park_sim::ServiceVisitPhase::Using);
        if using.is_some() {
            break;
        }
        park.advance(Speed::Normal);
    }
    let guest = using.expect("natural service visit");
    let owner = park
        .guests()
        .iter()
        .find(|g| g.id == guest)
        .unwrap()
        .service_visit
        .as_ref()
        .unwrap()
        .service;
    let facility = park.service(owner).unwrap().clone();
    let object = facility.object.unwrap();
    let original = park.clone();
    let mut view = ViewState {
        speed: Speed::Paused,
        selected: Some(SurfaceId::Object(object)),
        hovered: Some(SurfaceId::Object(object)),
        path_tool: path_tools::ToolState {
            mode: path_tools::PathTool::Demolish,
            ..default()
        },
        ..default()
    };
    path_tools::point_at(&mut park, &mut view, Vec2::ZERO, false);
    assert_eq!(park, original);
    assert!(
        view.path_tool
            .hint
            .contains("Unfinished visits cancel free")
    );
    path_tools::point_at(&mut park, &mut view, Vec2::ZERO, true);
    assert_eq!(
        park.cash_cents(),
        original.cash_cents() + facility.construction_cost_cents / 2
    );
    assert_eq!(park.ticks(), original.ticks());
    for before in original.guests() {
        let after = park.guests().iter().find(|g| g.id == before.id).unwrap();
        assert_eq!(
            (
                after.wallet_cents,
                after.spent_cents,
                after.service_spent_cents,
                after.services_used,
                after.needs
            ),
            (
                before.wallet_cents,
                before.spent_cents,
                before.service_spent_cents,
                before.services_used,
                before.needs
            )
        );
        assert!(
            after
                .service_visit
                .as_ref()
                .is_none_or(|v| v.service != owner)
        );
    }
    let removed = park.service(owner).unwrap();
    assert!(removed.object.is_none());
    assert_eq!(
        (
            removed.served,
            removed.revenue_cents,
            removed.upkeep_cents,
            removed.supplies_cents
        ),
        (
            facility.served,
            facility.revenue_cents,
            facility.upkeep_cents,
            facility.supplies_cents
        )
    );
    let entry = park.finance_ledger().entries.last().unwrap();
    assert_eq!(
        (entry.kind, entry.service, entry.ride, entry.amount_cents),
        (
            park_sim::FinanceKind::ServiceDemolitionRefund,
            Some(owner),
            None,
            facility.construction_cost_cents / 2
        )
    );
    assert_eq!(
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
        park
    );
}
