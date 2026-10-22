//! Selected ride ownership, modal transitions and deliberate construction.
use super::*;
use park_sim::{CAROUSEL_BUILD_COST_CENTS, FinanceKind, PathDirection, PathSlope, RideId};

fn two_rides() -> (Park, RideId) {
    let mut park = train_park();
    let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    (park, id)
}

fn app(directory: &InputSaveDirectory, park: Park, view: ViewState) -> App {
    let (mut app, _) = persistence_input_app(directory, park, view);
    app.add_systems(Startup, ui::setup).add_systems(
        Update,
        (apply_path_actions, apply_train_actions, apply_price_actions)
            .chain()
            .after(keyboard_input)
            .before(pointer_input),
    );
    app.add_systems(
        Update,
        (ui::rides::refresh, ui::refresh)
            .chain()
            .after(persistence::apply_save_action),
    );
    app
}

#[test]
fn ride_modal_transitions_consume_world_edits_and_selector_wraps_by_identity() {
    let dir = InputSaveDirectory::new();
    let (park, carousel) = two_rides();
    let coaster = park.ride().unwrap().id;
    let mut app = app(
        &dir,
        park.clone(),
        ViewState {
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    persistence_key_frame(
        &mut app,
        &[
            KeyCode::KeyY,
            KeyCode::F7,
            KeyCode::KeyO,
            KeyCode::F5,
            KeyCode::Delete,
        ],
    );
    assert!(app.world().resource::<ViewState>().rides_open);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert!(!dir.store().save_path().exists());
    persistence_key_frame(
        &mut app,
        &[
            KeyCode::ArrowDown,
            KeyCode::F7,
            KeyCode::KeyO,
            KeyCode::PageUp,
        ],
    );
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.selected_ride, Some(carousel));
    assert_eq!(view.path_tool.height, 8);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    persistence_key_frame(&mut app, &[KeyCode::ArrowDown]);
    assert_eq!(
        app.world().resource::<ViewState>().selected_ride,
        Some(coaster)
    );
    persistence_key_frame(&mut app, &[KeyCode::ArrowUp]);
    assert_eq!(
        app.world().resource::<ViewState>().selected_ride,
        Some(carousel)
    );
    persistence_key_frame(
        &mut app,
        &[KeyCode::Escape, KeyCode::F7, KeyCode::KeyO, KeyCode::F9],
    );
    assert!(!app.world().resource::<ViewState>().rides_open);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    const {
        assert!(ui::rides::PANEL_WIDTH <= 1024.0);
        assert!(ui::rides::PANEL_HEIGHT <= 680.0 - HEADER - FOOTER);
    }
}

#[test]
fn selected_ride_price_open_focus_and_save_target_only_that_ride() {
    let dir = InputSaveDirectory::new();
    let (park, carousel) = two_rides();
    let coaster = park.ride().unwrap().clone();
    let mut app = app(
        &dir,
        park,
        ViewState {
            speed: Speed::Paused,
            selected_ride: Some(carousel),
            ..default()
        },
    );
    persistence_key_frame(
        &mut app,
        &[KeyCode::F7, KeyCode::F5, KeyCode::KeyO, KeyCode::KeyF],
    );
    let sim = &app.world().resource::<Simulation>().0;
    assert_eq!(sim.ride(), Some(&coaster));
    assert_eq!(sim.ride_by_id(carousel).unwrap().ticket_price_cents, 50);
    assert!(sim.ride_is_open(carousel));
    assert_eq!(sim.ticks(), 0);
    assert_eq!(dir.store().load().unwrap(), *sim);
    let view = app.world().resource::<ViewState>();
    let expected = world::ride_focus(sim, carousel, 0).unwrap()
        - Vec2::Y * f32::from(sim.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    assert_eq!(view.pan, expected);
    persistence_key_frame(&mut app, &[KeyCode::F7, KeyCode::F9]);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .ride_by_id(carousel)
            .unwrap()
            .ticket_price_cents,
        50
    );
    assert_eq!(app.world().resource::<ViewState>().selected_ride, None);
}

#[test]
fn removed_ride_never_redirects_a_stale_price_command_to_the_coaster() {
    let dir = InputSaveDirectory::new();
    let (mut park, carousel) = two_rides();
    park.remove_carousel(carousel).unwrap();
    let before = park.clone();
    let mut app = app(
        &dir,
        park,
        ViewState {
            selected_ride: Some(carousel),
            ..default()
        },
    );
    persistence_key_frame(&mut app, &[KeyCode::F7, KeyCode::KeyO]);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert_eq!(
        app.world().resource::<ViewState>().selected_ride,
        before.ride().map(|ride| ride.id)
    );
}

#[test]
fn carousel_build_button_closes_modal_without_clicking_through_and_builds_while_paused() {
    let dir = InputSaveDirectory::new();
    let park = train_park();
    let mut app = app(
        &dir,
        park.clone(),
        ViewState {
            rides_open: true,
            speed: Speed::Paused,
            ..default()
        },
    );
    let entity = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::BuildCarousel))
        .id();
    frame(&mut app);
    let view = app.world().resource::<ViewState>();
    assert!(!view.rides_open && view.modal_input_blocked);
    assert_eq!(view.path_tool.mode, path_tools::PathTool::Carousel);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    app.world_mut().despawn(entity);
    let point =
        world::path_build_center(TilePos { x: 19, y: 22 }, &park, 0, 8, PathSlope::Flat).unwrap();
    let mut view = app.world_mut().remove_resource::<ViewState>().unwrap();
    let mut sim = app.world_mut().resource_mut::<Simulation>();
    path_tools::point_at(&mut sim.0, &mut view, point, false);
    assert_eq!(sim.0, park, "a price preview never spends cash");
    assert!(
        view.path_tool
            .hint
            .contains("Build $300.00 / Cash $1000.00")
    );
    path_tools::point_at(&mut sim.0, &mut view, point, true);
    assert_eq!(sim.0.carousels().len(), 1);
    let carousel = &sim.0.carousels()[0];
    assert_eq!(carousel.objects.len(), 9);
    assert!(!carousel.open);
    assert_eq!(view.selected_ride, Some(carousel.ride.id));
    assert_eq!(view.speed, Speed::Paused);
    assert_eq!(sim.0.ticks(), 0);
    assert_eq!(
        sim.0.cash_cents(),
        park.cash_cents() - CAROUSEL_BUILD_COST_CENTS
    );
    assert_eq!(carousel.construction_cost_cents, CAROUSEL_BUILD_COST_CENTS);
    let entry = sim.0.finance_ledger().entries.last().unwrap();
    assert_eq!(entry.kind, FinanceKind::RideConstruction);
    assert_eq!(entry.ride, Some(carousel.ride.id));
    assert_eq!(entry.amount_cents, CAROUSEL_BUILD_COST_CENTS);
    assert!(
        view.path_notice
            .as_deref()
            .unwrap()
            .contains("Built for $300.00")
    );
    assert!(
        view.path_notice
            .as_deref()
            .unwrap()
            .contains("Cash $700.00")
    );
}

#[test]
fn footprint_rejection_is_atomic_and_world_inspection_selects_body_gate_and_queue_owner() {
    let mut park = train_park();
    let original = park.clone();
    let mut view = ViewState {
        path_tool: path_tools::ToolState {
            mode: path_tools::PathTool::Carousel,
            ..default()
        },
        ..default()
    };
    let bad =
        world::path_build_center(TilePos { x: 31, y: 31 }, &park, 0, 8, PathSlope::Flat).unwrap();
    path_tools::point_at(&mut park, &mut view, bad, true);
    assert_eq!(park, original);
    assert!(view.path_tool.carousel_preview.is_some_and(|p| !p.valid));
    let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    let gate = park
        .place_ride_entrance(TilePos { x: 18, y: 23 }, 8, PathDirection::East, carousel)
        .unwrap();
    let queue = park
        .place_queue_path(TilePos { x: 17, y: 23 }, 8, PathSlope::Flat, carousel)
        .unwrap();
    for object in [park.carousel(carousel).unwrap().objects[0], gate, queue] {
        view.path_tool.set_mode(path_tools::PathTool::Inspect);
        view.selected_ride = park.ride().map(|r| r.id);
        view.hovered = Some(SurfaceId::Object(object));
        path_tools::point_at(&mut park, &mut view, Vec2::ZERO, true);
        assert_eq!(view.selected_ride, Some(carousel));
    }
    let body = park.carousel(carousel).unwrap().objects[0];
    view.path_tool.set_mode(path_tools::PathTool::Demolish);
    view.hovered = Some(SurfaceId::Object(body));
    let before = park.clone();
    path_tools::point_at(&mut park, &mut view, Vec2::ZERO, false);
    assert_eq!(park, before);
    assert!(view.path_tool.hint.contains("Refund $150.00 / whole ride"));
    view.selected = Some(SurfaceId::Object(body));
    path_tools::remove_selected(&mut park, &mut view);
    assert!(park.carousel(carousel).unwrap().objects.is_empty());
    assert!(view.selected.is_none());
    assert!(
        park.object(gate).is_none(),
        "whole-ride demolition removes its gate"
    );
    assert!(
        park.object(queue).is_some(),
        "owned queue remains for a safe drain"
    );
    assert_eq!(
        park.cash_cents(),
        original.cash_cents() - CAROUSEL_BUILD_COST_CENTS / 2
    );
    let entry = park.finance_ledger().entries.last().unwrap();
    assert_eq!(entry.kind, FinanceKind::RideDemolitionRefund);
    assert_eq!(entry.ride, Some(carousel));
    assert_eq!(entry.amount_cents, CAROUSEL_BUILD_COST_CENTS / 2);
    assert!(
        view.path_notice
            .as_deref()
            .unwrap()
            .contains("refund $150.00")
    );
    let removed = park.clone();
    view.selected = Some(SurfaceId::Object(body));
    path_tools::remove_selected(&mut park, &mut view);
    assert_eq!(park, removed, "a stale selection never refunds twice");
}

#[test]
fn unaffordable_carousel_preview_and_click_preserve_all_state() {
    let mut park = train_park();
    let origin = TilePos { x: 19, y: 22 };
    for _ in 0..5 {
        let id = park.place_carousel(origin, 8).unwrap();
        park.remove_carousel(id).unwrap();
    }
    assert_eq!(park.cash_cents(), 25_000);
    let original = park.clone();
    let mut view = ViewState {
        speed: Speed::Paused,
        path_tool: path_tools::ToolState {
            mode: path_tools::PathTool::Carousel,
            ..default()
        },
        ..default()
    };
    let point = world::path_build_center(origin, &park, 0, 8, PathSlope::Flat).unwrap();
    for click in [false, true] {
        path_tools::point_at(&mut park, &mut view, point, click);
        assert_eq!(park, original);
        assert!(view.path_tool.carousel_preview.is_some_and(|p| !p.valid));
        assert!(view.path_tool.hint.contains("Build $300.00 / Cash $250.00"));
        assert!(view.path_tool.hint.contains("Need $50.00 more"));
        assert!(!view.scene_dirty);
    }
    assert!(
        view.path_notice
            .as_deref()
            .unwrap()
            .contains("Need $50.00 more")
    );
}

#[test]
fn carousel_mouse_build_debits_once_even_when_button_stays_held() {
    let original = Park::new(MAP_SIZE, MAP_SIZE);
    let origin = TilePos { x: 14, y: 17 };
    let view = ViewState {
        speed: Speed::Paused,
        zoom: 1.4,
        path_tool: path_tools::ToolState {
            mode: path_tools::PathTool::Carousel,
            ..default()
        },
        ..default()
    };
    let (mut app, window) = input_app(original.clone(), view, 1.0);
    let point = world::path_build_center(origin, &original, 0, 8, PathSlope::Flat).unwrap();
    let cursor = cursor_for_world(&app, window, point);
    set_cursor(&mut app, window, Some(cursor));
    frame(&mut app);
    assert_eq!(app.world().resource::<Simulation>().0, original);
    assert!(
        app.world()
            .resource::<ViewState>()
            .path_tool
            .hint
            .contains("Build $300.00")
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    frame(&mut app);
    let built = app.world().resource::<Simulation>().0.clone();
    assert_eq!(built.cash_cents(), 70_000);
    assert_eq!(built.finance_ledger().entries.len(), 1);
    assert_eq!(built.carousels()[0].origin, origin);
    frame(&mut app);
    assert_eq!(app.world().resource::<Simulation>().0, built);
}

#[test]
fn occupied_carousel_demolition_shows_no_current_refund_and_cannot_change_cash_or_passengers() {
    let mut park = train_park();
    let id = crate::save_check::ride_choice::install_carousel(&mut park).unwrap();
    for _ in 0..12_000 {
        if crate::save_check::ride_choice::paid_running(&park, id) {
            break;
        }
        park.advance(Speed::Normal);
    }
    assert!(crate::save_check::ride_choice::paid_running(&park, id));
    let body = park.carousel(id).unwrap().objects[0];
    let original = park.clone();
    let mut view = ViewState {
        speed: Speed::Paused,
        hovered: Some(SurfaceId::Object(body)),
        selected: Some(SurfaceId::Object(body)),
        path_tool: path_tools::ToolState {
            mode: path_tools::PathTool::Demolish,
            ..default()
        },
        ..default()
    };
    for click in [false, true] {
        path_tools::point_at(&mut park, &mut view, Vec2::ZERO, click);
        assert_eq!(park, original);
        assert!(
            view.path_tool
                .carousel_preview
                .is_some_and(|p| p.demolish && !p.valid)
        );
        assert!(view.path_tool.hint.contains("no refund now"));
        assert!(!view.scene_dirty);
    }
    assert!(
        ui::rides::construction_details(&park, id).contains("Demolition blocked while occupied")
    );
    assert!(view.path_notice.as_deref().unwrap().contains("not removed"));
}
