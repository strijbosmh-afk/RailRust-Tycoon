//! Exercise the real pointer gesture and existing owner-specific settings panels.
use super::*;
use park_sim::{ObjectId, PathDirection, PathSlope, RideId, ServiceId, ServiceKind};

fn fixtures() -> (Park, [RideId; 2], [ServiceId; 2], Vec<ObjectId>) {
    let mut park = Park::new(32, 32);
    let first = park.place_carousel(TilePos { x: 14, y: 14 }, 0).unwrap();
    let second = park.place_swing(TilePos { x: 19, y: 19 }, 0).unwrap();
    let food = park
        .place_service(
            TilePos { x: 11, y: 18 },
            0,
            PathDirection::North,
            ServiceKind::Food,
        )
        .unwrap();
    let drink = park
        .place_service(
            TilePos { x: 19, y: 13 },
            0,
            PathDirection::West,
            ServiceKind::Drink,
        )
        .unwrap();
    park.set_service_price(food, 150).unwrap();
    park.set_service_price(drink, 450).unwrap();
    let queue = park
        .place_queue_path(TilePos { x: 12, y: 14 }, 0, PathSlope::Flat, first)
        .unwrap();
    let objects = vec![
        park.carousel(first).unwrap().objects[0],
        park.carousel(second).unwrap().objects[0],
        park.service(food).unwrap().object.unwrap(),
        park.service(drink).unwrap().object.unwrap(),
        queue,
    ];
    (park, [first, second], [food, drink], objects)
}

fn visible_point(park: &Park, object: ObjectId, rotation: u8) -> Vec2 {
    let surface = SurfaceId::Object(object);
    let center = world::surface_center(park, surface, rotation).unwrap();
    let mut offsets: Vec<_> = (-16..=16)
        .flat_map(|x| (-16..=16).map(move |y| Vec2::new(x as f32 * 4.0, y as f32 * 4.0)))
        .collect();
    offsets.sort_by(|a, b| a.length_squared().total_cmp(&b.length_squared()));
    offsets
        .into_iter()
        .map(|offset| center + offset)
        .find(|point| {
            world::pick_surface(*point, park, rotation, world::ViewMode::Surface) == Some(surface)
        })
        .expect("fixture has an exposed face")
}

fn setup(park: Park, object: ObjectId, rotation: u8, dpi: f32, scale: f32) -> (App, Entity, Vec2) {
    let point = visible_point(&park, object, rotation);
    let (mut app, window) = input_app(
        park,
        ViewState {
            rotation,
            pan: point,
            zoom: 1.7,
            speed: Speed::Paused,
            ..default()
        },
        dpi,
    );
    app.insert_resource(UiScale(scale));
    let w = app.world().get::<Window>(window).unwrap();
    let view = app.world().resource::<ViewState>();
    let (camera_scale, center) =
        camera_layout_scaled(w, view, &app.world().resource::<Simulation>().0, scale);
    let relative = (point - center) / camera_scale;
    let cursor = Vec2::new(w.width() * 0.5 + relative.x, w.height() * 0.5 - relative.y);
    assert!(is_map_position_scaled(cursor, w, false, scale));
    set_cursor(&mut app, window, Some(cursor));
    (app, window, cursor)
}

fn right_click(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    frame(app);
    assert!(!app.world().resource::<ViewState>().rides_open);
    assert!(!app.world().resource::<ViewState>().services_open);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
    frame(app);
}

#[test]
fn right_click_opens_the_visible_owner_with_rotated_scaled_pointer_coordinates() {
    let (park, rides, services, objects) = fixtures();
    let before = park.to_save_bytes().unwrap();
    for rotation in 0..4 {
        for (dpi, scale) in [(1.0, 1.0), (2.0, 1.25)] {
            for (index, object) in objects.iter().enumerate() {
                let (mut app, _, _) = setup(park.clone(), *object, rotation, dpi, scale);
                app.add_systems(Startup, ui::setup).add_systems(
                    Update,
                    (ui::rides::refresh, ui::services::refresh)
                        .chain()
                        .after(pointer_input),
                );
                {
                    let mut view = app.world_mut().resource_mut::<ViewState>();
                    view.selected_ride = Some(rides[1]);
                    view.selected_service = Some(services[1]);
                    view.service_browse_pending = 5;
                    view.path_tool.set_mode(path_tools::PathTool::Demolish);
                }
                right_click(&mut app);
                let view = app.world().resource::<ViewState>();
                if index == 2 || index == 3 {
                    assert!(view.services_open && !view.rides_open);
                    assert_eq!(view.selected_service, Some(services[index - 2]));
                } else {
                    assert!(view.rides_open && !view.services_open);
                    assert_eq!(
                        view.selected_ride,
                        Some(if index == 1 { rides[1] } else { rides[0] })
                    );
                    assert_eq!(view.ride_tab, ui::rides::RideTab::Operations);
                }
                assert!(view.ui_pointer_consumed && view.edits_blocked());
                assert_eq!(
                    app.world()
                        .resource::<Simulation>()
                        .0
                        .to_save_bytes()
                        .unwrap(),
                    before
                );
            }
        }
    }
}

#[test]
fn right_click_opens_coaster_settings_from_track_and_preserves_the_park() {
    let park = new_game_park();
    let ride = park.ride().unwrap().id;
    let object = park.circuit().unwrap().pieces[0].object;
    let before = park.to_save_bytes().unwrap();
    let (mut app, _, _) = setup(park, object, 0, 1.0, 1.0);
    right_click(&mut app);
    assert_eq!(
        app.world().resource::<ViewState>().selected_ride,
        Some(ride)
    );
    assert!(app.world().resource::<ViewState>().rides_open);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .unwrap(),
        before
    );
}

#[test]
fn dragging_with_either_button_and_returning_to_the_start_never_opens_settings() {
    let (park, _, _, objects) = fixtures();
    for button in [MouseButton::Right, MouseButton::Middle] {
        let (mut app, window, cursor) = setup(park.clone(), objects[0], 0, 2.0, 1.0);
        let initial_pan = app.world().resource::<ViewState>().pan;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(button);
        frame(&mut app);
        set_cursor(&mut app, window, Some(cursor + Vec2::new(22.0, 12.0)));
        frame(&mut app);
        assert_ne!(app.world().resource::<ViewState>().pan, initial_pan);
        set_cursor(&mut app, window, Some(cursor));
        frame(&mut app);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(button);
        frame(&mut app);
        let view = app.world().resource::<ViewState>();
        assert!(!view.rides_open && !view.services_open);
        assert_eq!(app.world().resource::<Simulation>().0, park);
    }
}

#[test]
fn focus_cursor_modal_camera_and_load_changes_cancel_a_pending_click() {
    let (park, _, _, objects) = fixtures();
    for cause in 0..6 {
        let (mut app, window, cursor) = setup(park.clone(), objects[0], 0, 1.0, 1.0);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        frame(&mut app);
        match cause {
            0 => app.world_mut().get_mut::<Window>(window).unwrap().focused = false,
            1 => set_cursor(&mut app, window, None),
            2 => app.world_mut().resource_mut::<ViewState>().help = true,
            3 => app.world_mut().resource_mut::<ViewState>().zoom = 2.0,
            4 => app.world_mut().resource_mut::<ViewState>().park_generation += 1,
            _ => set_cursor(&mut app, window, Some(Vec2::new(20.0, 20.0))),
        }
        frame(&mut app);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().resource_mut::<ViewState>().help = false;
        set_cursor(&mut app, window, Some(cursor));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Right);
        frame(&mut app);
        let view = app.world().resource::<ViewState>();
        assert!(!view.rides_open && !view.services_open, "cause {cause}");
        assert_eq!(app.world().resource::<Simulation>().0, park);
    }
}

#[test]
fn diagnostic_right_drag_still_pans_without_opening_settings() {
    let (park, _, _, objects) = fixtures();
    let (mut app, window, cursor) = setup(park.clone(), objects[0], 0, 1.0, 1.0);
    app.world_mut().resource_mut::<ViewState>().diagnostic_case =
        Some(park_sim::diagnostics::DiagnosticCase::SaveScale);
    let pan = app.world().resource::<ViewState>().pan;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    frame(&mut app);
    set_cursor(&mut app, window, Some(cursor + Vec2::new(20.0, 8.0)));
    frame(&mut app);
    assert_ne!(app.world().resource::<ViewState>().pan, pan);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
    frame(&mut app);
    let view = app.world().resource::<ViewState>();
    assert!(!view.rides_open && !view.services_open);
    assert_eq!(app.world().resource::<Simulation>().0, park);
}

#[test]
fn right_click_on_empty_ground_or_with_a_left_click_never_builds_or_opens_settings() {
    for both in [false, true] {
        let park = Park::new(32, 32);
        let point = world::tile_center(TilePos { x: 15, y: 16 }, 32, 0);
        let (mut app, window) = input_app(
            park.clone(),
            ViewState {
                pan: point,
                ..default()
            },
            1.0,
        );
        app.world_mut()
            .resource_mut::<ViewState>()
            .path_tool
            .set_mode(path_tools::PathTool::Build);
        let cursor = cursor_for_world(&app, window, point);
        set_cursor(&mut app, window, Some(cursor));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        if both {
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
        }
        frame(&mut app);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Right);
        frame(&mut app);
        let view = app.world().resource::<ViewState>();
        assert!(!view.rides_open && !view.services_open);
        assert_eq!(app.world().resource::<Simulation>().0, park);
    }
}

#[test]
fn small_pointer_jitter_opens_settings_without_panning_or_spending_tip_uses() {
    let (mut park, _, services, objects) = fixtures();
    park.retain_tip_uses(2).unwrap();
    let before = park.to_save_bytes().unwrap();
    let (mut app, window, cursor) = setup(park, objects[2], 0, 1.0, 1.0);
    let pan = app.world().resource::<ViewState>().pan;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    frame(&mut app);
    set_cursor(&mut app, window, Some(cursor + Vec2::new(1.0, 1.0)));
    frame(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
    frame(&mut app);
    let view = app.world().resource::<ViewState>();
    assert!(view.services_open);
    assert_eq!(view.selected_service, Some(services[0]));
    assert_eq!(view.pan, pan);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .unwrap(),
        before
    );
}
