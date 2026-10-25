//! Exercise the game input systems without a windowing backend or GPU.
//!
//! These tests supply Bevy input resources, not native Windows events. They do
//! not replace a normal-input check of the packaged desktop application.

use std::time::Duration;

use super::*;

#[path = "admission_input_tests.rs"]
mod admission_input_tests;
#[path = "economy_input_tests.rs"]
mod economy_input_tests;
#[path = "finance_input_tests.rs"]
mod finance_input_tests;
#[path = "guest_inspection_input_tests.rs"]
mod guest_inspection_input_tests;
#[path = "journey_input_tests.rs"]
mod journey_input_tests;
#[path = "path_input_tests.rs"]
mod path_input_tests;
#[path = "queue_input_tests.rs"]
mod queue_input_tests;
#[path = "ride_choice_input_tests.rs"]
mod ride_choice_input_tests;
#[path = "service_input_tests.rs"]
mod service_input_tests;
#[path = "slope_input_tests.rs"]
mod slope_input_tests;
#[path = "terrain_input_tests.rs"]
mod terrain_input_tests;
#[path = "track_input_tests.rs"]
mod track_input_tests;

fn crowd_park() -> Park {
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.apply_terrain_profile(TerrainProfile::Terraced);
    park.install_crowd_fixture(CROWD_SEED).unwrap();
    park
}

fn train_park() -> Park {
    let mut park = crowd_park();
    park.install_train_fixture().unwrap();
    park
}

#[test]
fn render_preview_blocks_keyboard_and_button_edits_but_allows_camera_and_speed() {
    use park_sim::diagnostics::DiagnosticCase;
    let original = train_park();
    let mut app = App::new();
    app.insert_resource(Simulation(original.clone()))
        .insert_resource(ViewState {
            diagnostic_case: Some(DiagnosticCase::SaveScale),
            selected: Some(SurfaceId::Object(original.objects()[0].id)),
            camera_span: 8.0,
            ..default()
        })
        .insert_resource(Time::<()>::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(
            Update,
            (
                ui::button_actions,
                keyboard_input,
                apply_terrain_profile,
                apply_path_actions,
                apply_train_actions,
            )
                .chain(),
        );
    for key in [
        KeyCode::KeyT,
        KeyCode::Delete,
        KeyCode::KeyR,
        KeyCode::KeyO,
        KeyCode::F5,
        KeyCode::F9,
    ] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }
    for action in [
        ui::Action::Terrain,
        ui::Action::RemovePath,
        ui::Action::RestorePaths,
        ui::Action::ToggleTrain,
        ui::Action::Save,
        ui::Action::Load,
        ui::Action::LoadBackup,
    ] {
        app.world_mut()
            .spawn((Button, Interaction::Pressed, action));
    }
    app.update();
    assert_eq!(app.world().resource::<Simulation>().0, original);
    let view = app.world().resource::<ViewState>();
    assert!(
        !view.terrain_change_pending && !view.remove_path_pending && !view.restore_paths_pending
    );
    assert!(!view.toggle_train_pending && view.save_pending.is_none());
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    for key in [KeyCode::KeyE, KeyCode::KeyU, KeyCode::KeyF, KeyCode::Digit2] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }
    app.update();
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.rotation, 1);
    assert_eq!(view.view_mode, world::ViewMode::Underground);
    assert_eq!(view.speed, Speed::Fast);
    assert_eq!(view.zoom, 12.0);
    assert_eq!(app.world().resource::<Simulation>().0, original);
}

fn test_window(width: u32, height: u32, dpi: f32) -> Window {
    Window {
        resolution: WindowResolution::new(
            (width as f32 * dpi) as u32,
            (height as f32 * dpi) as u32,
        )
        .with_scale_factor_override(dpi),
        focused: true,
        ..default()
    }
}

fn input_app(park: Park, view: ViewState, dpi: f32) -> (App, Entity) {
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(view)
        .insert_resource(Time::<()>::default())
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<MouseWheel>()
        .add_systems(Update, (keyboard_input, pointer_input).chain());
    let window = app
        .world_mut()
        .spawn((test_window(1280, 800, dpi), PrimaryWindow))
        .id();
    (app, window)
}

fn cursor_for_world(app: &App, window: Entity, point: Vec2) -> Vec2 {
    let window = app.world().get::<Window>(window).unwrap();
    let park = &app.world().resource::<Simulation>().0;
    let view = app.world().resource::<ViewState>();
    let (scale, center) = camera_layout(window, view, park);
    let relative = (point - center) / scale;
    Vec2::new(
        window.width() * 0.5 + relative.x,
        window.height() * 0.5 - relative.y,
    )
}

fn set_cursor(app: &mut App, window: Entity, cursor: Option<Vec2>) {
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(cursor);
}

fn frame(app: &mut App) {
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
}

fn click_world(app: &mut App, window: Entity, point: Vec2, expected: TilePos) {
    click_surface(app, window, point, SurfaceId::Terrain(expected));
}

fn click_surface(app: &mut App, window: Entity, point: Vec2, expected: SurfaceId) {
    let cursor = cursor_for_world(app, window, point);
    assert!(
        is_map_position(cursor, app.world().get::<Window>(window).unwrap(), false),
        "fixture point must be in the usable viewport: {cursor:?}"
    );
    set_cursor(app, window, Some(cursor));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    frame(app);
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.hovered, Some(expected));
    assert_eq!(view.selected, Some(expected));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
}

fn from_view_tile(x: u32, y: u32, size: u32, rotation: u8) -> TilePos {
    let last = size - 1;
    match rotation {
        0 => TilePos { x, y },
        1 => TilePos { x: y, y: last - x },
        2 => TilePos {
            x: last - x,
            y: last - y,
        },
        _ => TilePos { x: last - y, y: x },
    }
}

#[test]
fn pointer_selects_an_elevated_roof_after_rotation_pan_zoom_and_dpi_scaling() {
    let tile = TilePos { x: 15, y: 16 };
    for rotation in 0..4 {
        for dpi in [1.0, 2.0] {
            for (zoom, pan) in [(1.0, Vec2::ZERO), (1.7, Vec2::new(70.0, -30.0))] {
                let mut park = Park::new(MAP_SIZE, MAP_SIZE);
                park.set_terrain_height(tile, 6).unwrap();
                let roof = world::elevated_tile_center(&park, tile, rotation).unwrap();
                let view = ViewState {
                    rotation,
                    zoom,
                    pan,
                    ..default()
                };
                let (mut app, window) = input_app(park, view, dpi);
                click_world(&mut app, window, roof, tile);
            }
        }
    }
}

#[test]
fn pointer_selects_the_visible_cliff_instead_of_the_ground_behind_it() {
    for rotation in 0..4 {
        let mut park = Park::new(8, 8);
        let low = from_view_tile(3, 3, 8, rotation);
        let blocker = from_view_tile(4, 4, 8, rotation);
        park.set_terrain_height(blocker, 6).unwrap();
        let hidden_ground =
            world::elevated_tile_center(&park, low, rotation).unwrap() + Vec2::new(8.0, 0.0);
        let roof = world::elevated_tile_center(&park, blocker, rotation).unwrap();
        let cliff = roof + Vec2::new(16.0, -8.0 - 3.0 * world::HEIGHT_STEP);
        let (mut app, window) = input_app(
            park,
            ViewState {
                rotation,
                ..default()
            },
            1.0,
        );
        click_world(&mut app, window, hidden_ground, blocker);
        click_world(&mut app, window, cliff, blocker);
        click_world(&mut app, window, roof, blocker);
    }
}

#[test]
fn ui_regions_help_and_consumed_clicks_block_selection_and_wheel_zoom() {
    let selected = TilePos { x: 9, y: 8 };
    let cases = [
        (Vec2::new(SIDEBAR, 400.0), false, false),
        (Vec2::new(640.0, HEADER), false, false),
        (Vec2::new(640.0, 800.0 - FOOTER), false, false),
        (Vec2::new(640.0, 400.0), true, false),
        (Vec2::new(640.0, 400.0), false, true),
    ];
    for (cursor, help, consumed) in cases {
        let (mut app, window) = input_app(
            Park::new(MAP_SIZE, MAP_SIZE),
            ViewState {
                selected: Some(SurfaceId::Terrain(selected)),
                hovered: Some(SurfaceId::Terrain(selected)),
                help,
                ui_pointer_consumed: consumed,
                ..default()
            },
            2.0,
        );
        set_cursor(&mut app, window, Some(cursor));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: 2.0,
                window,
            });
        frame(&mut app);
        let view = app.world().resource::<ViewState>();
        assert_eq!(view.selected, Some(SurfaceId::Terrain(selected)));
        assert_eq!(view.hovered, None);
        assert_eq!(view.zoom, 1.0);
    }
}

#[test]
fn focus_or_cursor_loss_cancels_drag_without_a_jump_on_return() {
    for lose_focus in [false, true] {
        let (mut app, window) = input_app(Park::new(MAP_SIZE, MAP_SIZE), ViewState::default(), 2.0);
        set_cursor(&mut app, window, Some(Vec2::new(600.0, 400.0)));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        frame(&mut app);
        assert!(app.world().resource::<ViewState>().dragging);
        assert_eq!(app.world().resource::<ViewState>().pan, Vec2::ZERO);

        set_cursor(&mut app, window, Some(Vec2::new(620.0, 410.0)));
        frame(&mut app);
        let pan = app.world().resource::<ViewState>().pan;
        assert_ne!(pan, Vec2::ZERO);
        if lose_focus {
            app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        } else {
            set_cursor(&mut app, window, None);
        }
        frame(&mut app);
        let view = app.world().resource::<ViewState>();
        assert!(!view.dragging);
        assert_eq!(view.hovered, None);
        assert_eq!(view.pan, pan);

        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        set_cursor(&mut app, window, Some(Vec2::new(1000.0, 600.0)));
        frame(&mut app);
        let view = app.world().resource::<ViewState>();
        assert!(!view.dragging);
        assert_eq!(view.pan, pan);
    }
}

#[test]
fn keyboard_camera_changes_and_wheel_zoom_preserve_pointer_selection() {
    let tile = TilePos { x: 15, y: 16 };
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.set_terrain_height(tile, 6).unwrap();
    let (mut app, window) = input_app(park, ViewState::default(), 1.0);
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::ArrowRight);
        keys.press(KeyCode::Equal);
        keys.press(KeyCode::KeyE);
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(50));
    frame(&mut app);
    {
        let view = app.world().resource::<ViewState>();
        assert_eq!(view.rotation, 1);
        assert!((view.zoom - 1.2).abs() < 0.001);
        assert!(view.pan.x > 0.0);
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    let roof =
        world::elevated_tile_center(&app.world().resource::<Simulation>().0, tile, 1).unwrap();
    click_world(&mut app, window, roof, tile);

    app.world_mut()
        .resource_mut::<Messages<MouseWheel>>()
        .write(MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: 0.0,
            y: 60.0,
            window,
        });
    frame(&mut app);
    assert!(app.world().resource::<ViewState>().zoom > 1.2);
    click_world(&mut app, window, roof, tile);
}

#[test]
fn fitting_keeps_terrain_roofs_in_the_viewport_at_maximum_height() {
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    for pos in [
        TilePos { x: 0, y: 0 },
        TilePos {
            x: MAP_SIZE - 1,
            y: 0,
        },
        TilePos {
            x: 0,
            y: MAP_SIZE - 1,
        },
        TilePos {
            x: MAP_SIZE - 1,
            y: MAP_SIZE - 1,
        },
    ] {
        park.set_terrain_height(pos, park_sim::MAX_TERRAIN_HEIGHT)
            .unwrap();
    }
    for (width, height, dpi) in [(1024, 680, 1.0), (1280, 800, 1.0), (1280, 800, 2.0)] {
        let window = test_window(width, height, dpi);
        for rotation in 0..4 {
            let view = ViewState {
                rotation,
                ..default()
            };
            let (scale, center) = camera_layout(&window, &view, &park);
            assert!(scale.is_finite() && scale > 0.0);
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    let roof =
                        world::elevated_tile_center(&park, TilePos { x, y }, rotation).unwrap();
                    for corner in [
                        Vec2::new(world::TILE_WIDTH * 0.5, 0.0),
                        Vec2::new(-world::TILE_WIDTH * 0.5, 0.0),
                        Vec2::new(0.0, world::TILE_HEIGHT * 0.5),
                        Vec2::new(0.0, -world::TILE_HEIGHT * 0.5),
                    ] {
                        let relative = (roof + corner - center) / scale;
                        let cursor = Vec2::new(
                            window.width() * 0.5 + relative.x,
                            window.height() * 0.5 - relative.y,
                        );
                        assert!(
                            is_map_position(cursor, &window, false) && cursor.x < window.width(),
                            "terrain clipped: rotation={rotation}, tile=({x},{y}), cursor={cursor:?}"
                        );
                    }
                }
            }
        }
    }
}

fn terrain_materials(park: &Park) -> Vec<park_sim::Terrain> {
    (0..park.height())
        .flat_map(|y| (0..park.width()).map(move |x| park.tile(TilePos { x, y }).unwrap().terrain))
        .collect()
}

#[test]
fn terrain_button_preserves_selected_identity_clock_and_materials() {
    let selected = TilePos { x: 12, y: 16 };
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.apply_terrain_profile(TerrainProfile::Terraced);
    park.advance(Speed::Fastest);
    let ticks = park.ticks();
    let materials = terrain_materials(&park);
    assert_eq!(park.max_terrain_height(), 6);
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(ViewState {
            selected: Some(SurfaceId::Terrain(selected)),
            ..default()
        })
        .add_systems(Update, (ui::button_actions, apply_terrain_profile).chain());
    let button = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::Terrain))
        .id();

    app.update();
    for expected in [TerrainProfile::Flat, TerrainProfile::Terraced] {
        let view = app.world().resource::<ViewState>();
        let park = &app.world().resource::<Simulation>().0;
        assert_eq!(view.selected, Some(SurfaceId::Terrain(selected)));
        assert_eq!(view.terrain_profile, expected);
        assert!(!view.terrain_change_pending);
        assert_eq!(park.ticks(), ticks);
        assert_eq!(terrain_materials(park), materials);
        assert_eq!(
            park.max_terrain_height(),
            if expected == TerrainProfile::Terraced {
                6
            } else {
                0
            }
        );
        if expected == TerrainProfile::Flat {
            app.world_mut().entity_mut(button).insert(Interaction::None);
            app.update();
            app.world_mut()
                .entity_mut(button)
                .insert(Interaction::Pressed);
            app.update();
        }
    }
}

fn selected_mesh_positions(app: &App) -> Vec<[f32; 3]> {
    let scene = app.world().resource::<SceneMeshes>();
    app.world()
        .resource::<Assets<Mesh>>()
        .get(&scene.selection)
        .unwrap()
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .to_vec()
}

#[test]
fn selection_mesh_follows_height_changes_without_changing_selected_tile_or_rotation() {
    let selected = TilePos { x: 12, y: 16 };
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.apply_terrain_profile(TerrainProfile::Terraced);
    let elevation = f32::from(park.tile(selected).unwrap().height) * world::HEIGHT_STEP;
    assert!(elevation > 0.0);
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(ViewState {
            selected: Some(SurfaceId::Terrain(selected)),
            rotation: 2,
            ..default()
        })
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (apply_terrain_profile, update_scene).chain());
    app.update();
    let raised_positions = selected_mesh_positions(&app);
    assert!(!raised_positions.is_empty());

    app.world_mut()
        .resource_mut::<ViewState>()
        .toggle_terrain_profile();
    app.update();
    let flat_positions = selected_mesh_positions(&app);
    assert_eq!(raised_positions.len(), flat_positions.len());
    for (raised, flat) in raised_positions.iter().zip(&flat_positions) {
        assert!((raised[0] - flat[0]).abs() < 0.001);
        assert!((raised[1] - flat[1] - elevation).abs() < 0.001);
    }
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.selected, Some(SurfaceId::Terrain(selected)));
    assert_eq!(view.rotation, 2);

    app.world_mut()
        .resource_mut::<ViewState>()
        .toggle_terrain_profile();
    app.update();
    assert_eq!(selected_mesh_positions(&app), raised_positions);
}

#[test]
fn layered_pointer_selection_and_visibility_survive_camera_and_dpi_changes() {
    use park_sim::{Axis, ObjectKind};
    let pos = TilePos { x: 15, y: 16 };
    for rotation in 0..4 {
        for dpi in [1.0, 2.0] {
            let mut park = Park::new(MAP_SIZE, MAP_SIZE);
            park.set_terrain_height(pos, 6).unwrap();
            let path = SurfaceId::Object(
                park.place_object(pos, ObjectKind::Path, Axis::X, 8, 2)
                    .unwrap(),
            );
            let track = SurfaceId::Object(
                park.place_object(pos, ObjectKind::Track, Axis::Y, 0, 2)
                    .unwrap(),
            );
            let path_center = world::surface_center(&park, path, rotation).unwrap();
            let track_center = world::surface_center(&park, track, rotation).unwrap();
            assert_ne!(
                world::pick_surface(track_center, &park, rotation, world::ViewMode::Surface),
                Some(track)
            );
            let (mut app, window) = input_app(
                park,
                ViewState {
                    rotation,
                    zoom: 1.5,
                    pan: Vec2::new(35.0, -20.0),
                    ..default()
                },
                dpi,
            );
            click_surface(&mut app, window, path_center, path);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyU);
            frame(&mut app);
            assert_eq!(
                app.world().resource::<ViewState>().view_mode,
                world::ViewMode::Underground
            );
            assert_eq!(app.world().resource::<ViewState>().selected, Some(path));
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            click_surface(&mut app, window, track_center, track);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyU);
            frame(&mut app);
            assert_eq!(
                app.world().resource::<ViewState>().view_mode,
                world::ViewMode::Surface
            );
            assert_eq!(app.world().resource::<ViewState>().selected, Some(track));
        }
    }
}

#[test]
fn underground_button_rebuilds_hidden_outlines_and_removal_clears_selection() {
    use park_sim::{Axis, ObjectKind};
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    let pos = TilePos { x: 13, y: 16 };
    park.set_terrain_height(pos, 6).unwrap();
    let id = park
        .place_object(pos, ObjectKind::Track, Axis::Y, 0, 2)
        .unwrap();
    let selected = SurfaceId::Object(id);
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(ViewState {
            selected: Some(selected),
            ..default()
        })
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (ui::button_actions, update_scene).chain());
    app.update();
    assert!(
        selected_mesh_positions(&app).is_empty(),
        "buried selections cannot show through terrain"
    );
    let button = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::Underground))
        .id();
    app.update();
    assert_eq!(app.world().resource::<ViewState>().selected, Some(selected));
    assert!(app.world().resource::<ViewState>().ui_pointer_consumed);
    assert!(!selected_mesh_positions(&app).is_empty());
    app.world_mut().entity_mut(button).insert(Interaction::None);
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .remove_object(id)
        .unwrap();
    app.world_mut().resource_mut::<ViewState>().scene_dirty = true;
    app.update();
    assert_eq!(app.world().resource::<ViewState>().selected, None);
    assert!(selected_mesh_positions(&app).is_empty());
}

#[test]
fn presets_keep_object_identity_and_reject_conflicting_terrain_atomically() {
    use park_sim::{Axis, ObjectKind};
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.install_overlap_fixture().unwrap();
    let selected = SurfaceId::Object(park.objects().last().unwrap().id);
    let objects = park.objects().to_vec();
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(ViewState {
            terrain_profile: TerrainProfile::Flat,
            selected: Some(selected),
            ..default()
        })
        .add_systems(Update, apply_terrain_profile);
    for expected in [TerrainProfile::Terraced, TerrainProfile::Flat] {
        app.world_mut()
            .resource_mut::<ViewState>()
            .toggle_terrain_profile();
        app.update();
        assert_eq!(app.world().resource::<Simulation>().0.objects(), objects);
        let view = app.world().resource::<ViewState>();
        assert_eq!(view.selected, Some(selected));
        assert_eq!(view.terrain_profile, expected);
        assert!(view.terrain_error.is_none());
    }
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .place_object(TilePos { x: 12, y: 15 }, ObjectKind::Path, Axis::X, 0, 2)
        .unwrap();
    let before = app.world().resource::<Simulation>().0.clone();
    app.world_mut()
        .resource_mut::<ViewState>()
        .toggle_terrain_profile();
    app.update();
    assert_eq!(app.world().resource::<Simulation>().0, before);
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.terrain_profile, TerrainProfile::Flat);
    assert!(view.terrain_error.is_some());
    assert!(!view.terrain_change_pending);
}

#[test]
fn fitting_includes_elevated_objects_above_flat_terrain() {
    use park_sim::{Axis, ObjectKind};
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    for pos in [
        TilePos { x: 0, y: 0 },
        TilePos { x: 31, y: 0 },
        TilePos { x: 0, y: 31 },
        TilePos { x: 31, y: 31 },
    ] {
        park.place_object(pos, ObjectKind::Path, Axis::X, 30, 2)
            .unwrap();
    }
    let window = test_window(1024, 680, 2.0);
    for rotation in 0..4 {
        let view = ViewState {
            rotation,
            ..default()
        };
        let (scale, center) = camera_layout(&window, &view, &park);
        for object in park.objects() {
            let point =
                world::surface_center(&park, SurfaceId::Object(object.id), rotation).unwrap();
            let relative = (point - center) / scale;
            let cursor = Vec2::new(
                window.width() * 0.5 + relative.x,
                window.height() * 0.5 - relative.y,
            );
            assert!(is_map_position(cursor, &window, false) && cursor.x < window.width());
        }
    }
}

#[test]
fn keyboard_deletion_and_restore_preserve_guests_time_and_other_objects() {
    let mut park = crowd_park();
    for _ in 0..90 {
        park.advance(Speed::Normal);
    }
    let removed = park.guests()[0].current.unwrap();
    let guest_ids: Vec<_> = park.guests().iter().map(|guest| guest.id).collect();
    let objects = park.objects().to_vec();
    let ticks = park.ticks();
    let (mut app, _) = input_app(
        park,
        ViewState {
            selected: Some(SurfaceId::Object(removed)),
            speed: Speed::Paused,
            ..default()
        },
        1.0,
    );
    app.add_systems(Update, apply_path_actions.after(keyboard_input));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Delete);
    frame(&mut app);
    let park = &app.world().resource::<Simulation>().0;
    assert!(park.object(removed).is_none());
    assert_eq!(park.ticks(), ticks);
    assert_eq!(
        park.guests().iter().map(|g| g.id).collect::<Vec<_>>(),
        guest_ids
    );
    assert!(
        park.guests()
            .iter()
            .all(|guest| guest.current != Some(removed)
                && guest.next != Some(removed)
                && guest.destination != Some(removed))
    );
    assert!(app.world().resource::<ViewState>().selected.is_none());
    assert!(app.world().resource::<ViewState>().scene_dirty);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    frame(&mut app);
    let park = &app.world().resource::<Simulation>().0;
    assert_eq!(park.ticks(), ticks);
    assert_eq!(park.objects().len(), objects.len());
    assert!(
        park.object(removed).is_none(),
        "restoration must allocate a new identity"
    );
    for object in objects.iter().filter(|o| o.id != removed) {
        assert_eq!(park.object(object.id), Some(object));
    }
    assert_eq!(
        park.guests().iter().map(|g| g.id).collect::<Vec<_>>(),
        guest_ids
    );
}

#[test]
fn delete_rejects_terrain_and_help_blocks_path_shortcuts() {
    let park = crowd_park();
    for (selected, help) in [
        (Some(SurfaceId::Terrain(TilePos { x: 12, y: 16 })), false),
        (
            Some(SurfaceId::Object(park.guests()[0].current.unwrap())),
            true,
        ),
    ] {
        let (mut app, _) = input_app(
            park.clone(),
            ViewState {
                selected,
                help,
                ..default()
            },
            1.0,
        );
        app.add_systems(Update, apply_path_actions.after(keyboard_input));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Delete);
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
    }
}

#[test]
fn path_buttons_apply_once_and_capture_mode_ignores_them() {
    for capture_active in [false, true] {
        let park = crowd_park();
        let selected = park.guests()[0].current.unwrap();
        let count = park.objects().len();
        let mut app = App::new();
        app.insert_resource(Simulation(park))
            .insert_resource(ViewState {
                selected: Some(SurfaceId::Object(selected)),
                capture_active,
                ..default()
            })
            .add_systems(Update, (ui::button_actions, apply_path_actions).chain());
        let button = app
            .world_mut()
            .spawn((Button, Interaction::Pressed, ui::Action::RemovePath))
            .id();
        app.update();
        assert_eq!(
            app.world().resource::<Simulation>().0.objects().len(),
            count - usize::from(!capture_active)
        );
        assert_eq!(
            app.world().resource::<ViewState>().ui_pointer_consumed,
            !capture_active
        );
        app.update();
        assert!(!app.world().resource::<ViewState>().ui_pointer_consumed);
        app.world_mut()
            .entity_mut(button)
            .insert((ui::Action::RestorePaths, Interaction::None));
        app.update();
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        assert_eq!(
            app.world().resource::<Simulation>().0.objects().len(),
            count
        );
    }
}

#[test]
fn application_speed_advances_complete_crowd_steps_and_pause_freezes_them() {
    let mut expected = crowd_park();
    let mut app = App::new();
    app.insert_resource(Simulation(expected.clone()))
        .insert_resource(ViewState {
            speed: Speed::Fastest,
            ..default()
        })
        .add_systems(Update, tick_simulation);
    for _ in 0..150 {
        app.update();
        for _ in 0..4 {
            expected.advance(Speed::Normal);
        }
    }
    assert_eq!(app.world().resource::<Simulation>().0, expected);
    assert!(expected.crowd_stats().arrivals > 0);
    app.world_mut()
        .resource_mut::<ViewState>()
        .set_speed(Speed::Paused);
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(app.world().resource::<Simulation>().0, expected);
}

#[test]
fn moving_guests_update_their_mesh_without_rebuilding_static_terrain() {
    let mut app = App::new();
    app.insert_resource(Simulation(crowd_park()))
        .insert_resource(ViewState::default())
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_scene);
    app.update();
    let positions = |app: &App, guests: bool| {
        let scene = app.world().resource::<SceneMeshes>();
        let handle = if guests {
            &scene.guests
        } else {
            app.world()
                .resource::<static_landscape::StaticLandscape>()
                .mesh_handles()
                .next()
                .unwrap()
        };
        app.world()
            .resource::<Assets<Mesh>>()
            .get(handle)
            .unwrap()
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .to_vec()
    };
    let initial_guests = positions(&app, true);
    let initial_land = positions(&app, false);
    assert!(!initial_guests.is_empty());
    for _ in 0..90 {
        app.world_mut()
            .resource_mut::<Simulation>()
            .0
            .advance(Speed::Normal);
    }
    app.update();
    assert_ne!(positions(&app, true), initial_guests);
    assert_eq!(positions(&app, false), initial_land);
}

#[test]
fn scene_reuses_guest_resources_and_idle_frames_do_not_repaint() {
    let mut app = App::new();
    app.insert_resource(Simulation(train_park()))
        .insert_resource(ViewState::default())
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_scene);
    app.update();
    let initial = app.world().resource::<SceneMeshes>().guest_renderer.stats();
    let handle = app.world().resource::<SceneMeshes>().guests.id();
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(
        app.world()
            .resource::<SceneMeshes>()
            .guest_renderer
            .stats()
            .updates,
        initial.updates
    );
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .advance(Speed::Normal);
    app.update();
    let moving = app.world().resource::<SceneMeshes>().guest_renderer.stats();
    assert_eq!(moving.updates, initial.updates + 1);
    assert_eq!(moving.deck_rebuilds, initial.deck_rebuilds);
    assert_eq!(moving.template_builds, initial.template_builds);
    assert_eq!(app.world().resource::<SceneMeshes>().guests.id(), handle);
    app.world_mut().resource_mut::<ViewState>().rotate(1);
    app.update();
    let rotated = app.world().resource::<SceneMeshes>().guest_renderer.stats();
    assert_eq!(rotated.deck_rebuilds, moving.deck_rebuilds + 1);
    assert_eq!(rotated.template_builds, moving.template_builds);
    assert_eq!(app.world().resource::<SceneMeshes>().guests.id(), handle);
}

#[test]
fn junction_path_stays_selectable_with_crowds_in_all_rotations_and_dpi() {
    let mut park = crowd_park();
    for _ in 0..600 {
        park.advance(Speed::Normal);
    }
    let junction = park
        .objects()
        .iter()
        .find(|o| o.kind == ObjectKind::Path && o.pos == (TilePos { x: 13, y: 16 }))
        .unwrap()
        .id;
    for rotation in 0..4 {
        for dpi in [1.0, 2.0] {
            let point =
                world::surface_center(&park, SurfaceId::Object(junction), rotation).unwrap();
            let (mut app, window) = input_app(
                park.clone(),
                ViewState {
                    rotation,
                    zoom: 1.3,
                    ..default()
                },
                dpi,
            );
            click_surface(&mut app, window, point, SurfaceId::Object(junction));
        }
    }
}

#[test]
fn train_shortcuts_preserve_pause_and_help_and_capture_gate_actions() {
    for (help, capture_active) in [(false, false), (true, false), (false, true)] {
        let initial = train_park();
        let (mut app, _) = input_app(
            initial.clone(),
            ViewState {
                speed: Speed::Paused,
                help,
                capture_active,
                ..default()
            },
            1.0,
        );
        app.add_systems(Update, apply_train_actions.after(keyboard_input));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyO);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        frame(&mut app);
        let park = &app.world().resource::<Simulation>().0;
        let view = app.world().resource::<ViewState>();
        assert_eq!(park.ticks(), 0);
        assert_eq!(park.guests(), initial.guests());
        assert_eq!(park.objects(), initial.objects());
        assert_eq!(view.speed, Speed::Paused);
        if help || capture_active {
            assert_eq!(park, &initial);
            assert_eq!(view.zoom, 1.0);
        } else {
            assert!(!park.train().unwrap().open);
            assert_eq!(park.train().unwrap().phase, park_sim::TrainPhase::Closed);
            assert_eq!(view.zoom, 1.5);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::KeyO);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyO);
            frame(&mut app);
            assert!(!app.world().resource::<Simulation>().0.train().unwrap().open);
            assert!(
                app.world()
                    .resource::<ViewState>()
                    .price_notice
                    .as_deref()
                    .unwrap()
                    .contains("entrance")
            );
        }
    }
}

#[test]
fn ride_panel_blocks_map_selection_and_scroll_at_both_dpi_scales() {
    for dpi in [1.0, 2.0] {
        let (mut app, window) = input_app(train_park(), ViewState::default(), dpi);
        let cursor = Vec2::new(
            1280.0 - RIDE_PANEL_MARGIN - RIDE_PANEL_WIDTH * 0.5,
            HEADER + RIDE_PANEL_MARGIN + RIDE_PANEL_HEIGHT * 0.5,
        );
        assert!(!is_map_position(
            cursor,
            app.world().get::<Window>(window).unwrap(),
            false
        ));
        set_cursor(&mut app, window, Some(cursor));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window,
        });
        frame(&mut app);
        let view = app.world().resource::<ViewState>();
        assert_eq!(view.selected, None);
        assert_eq!(view.hovered, None);
        assert_eq!(view.zoom, 1.0);
    }
}

#[test]
fn train_button_is_consumed_once_without_pausing_guests() {
    let park = train_park();
    let mut app = App::new();
    app.insert_resource(Simulation(park.clone()))
        .insert_resource(ViewState::default())
        .add_systems(Update, (ui::button_actions, apply_train_actions).chain());
    app.world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::ToggleTrain));
    app.update();
    assert!(!app.world().resource::<Simulation>().0.train().unwrap().open);
    assert!(app.world().resource::<ViewState>().ui_pointer_consumed);
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Normal);
    app.update();
    assert!(!app.world().resource::<ViewState>().ui_pointer_consumed);
    assert!(!app.world().resource::<Simulation>().0.train().unwrap().open);
    let mut actual = app.world_mut().resource_mut::<Simulation>();
    for _ in 0..120 {
        actual.0.advance(Speed::Normal);
    }
    assert_ne!(actual.0.guests(), park.guests());
    assert_eq!(
        actual.0.train().unwrap().phase,
        park_sim::TrainPhase::Closed
    );
}

#[test]
fn fixed_update_train_and_crowd_are_equal_at_30_60_and_144_display_frames() {
    use bevy::time::{TimePlugin, TimeUpdateStrategy};

    let baseline = train_park();
    let mut expected = baseline.clone();
    for _ in 0..120 * TICKS_PER_SECOND {
        expected.advance(Speed::Normal);
    }
    assert!(expected.train().unwrap().laps > 0);
    for hz in [30_u64, 60, 144] {
        let mut app = App::new();
        app.add_plugins(TimePlugin)
            .insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND as f64))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .insert_resource(Simulation(baseline.clone()))
            .insert_resource(ViewState::default())
            .add_systems(FixedUpdate, tick_simulation);
        app.update(); // Initialize the real/virtual clock without advancing it.
        for frame in 0..120 * hz {
            // Distribute fractional nanoseconds, preserving exactly 120 seconds.
            let ns = (frame + 1) * 1_000_000_000 / hz - frame * 1_000_000_000 / hz;
            app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_nanos(ns)));
            app.update();
        }
        let actual = &app.world().resource::<Simulation>().0;
        assert_eq!(actual.ticks(), expected.ticks(), "{hz} Hz clock");
        assert_eq!(actual.train(), expected.train(), "{hz} Hz train");
        assert!(actual == &expected, "{hz} Hz full park state diverged");
        app.world_mut()
            .resource_mut::<ViewState>()
            .set_speed(Speed::Paused);
        for _ in 0..hz {
            app.update();
        }
        assert_eq!(app.world().resource::<Simulation>().0, expected);
    }
}

#[test]
fn moving_train_refreshes_only_dynamic_meshes_and_keeps_track_identity() {
    let mut park = train_park();
    for _ in 0..100 {
        park.advance(Speed::Normal);
    }
    let selected = park.circuit().unwrap().pieces[0].object;
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(ViewState {
            selected: Some(SurfaceId::Object(selected)),
            ..default()
        })
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_scene);
    app.update();
    let positions = |app: &App, train: bool| {
        let scene = app.world().resource::<SceneMeshes>();
        let handle = if train {
            &scene.train
        } else {
            app.world()
                .resource::<static_landscape::StaticLandscape>()
                .mesh_handles()
                .next()
                .unwrap()
        };
        app.world()
            .resource::<Assets<Mesh>>()
            .get(handle)
            .unwrap()
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .to_vec()
    };
    let initial_train = positions(&app, true);
    let initial_land = positions(&app, false);
    assert!(!initial_train.is_empty());
    for _ in 0..60 {
        app.world_mut()
            .resource_mut::<Simulation>()
            .0
            .advance(Speed::Normal);
    }
    app.update();
    assert_ne!(positions(&app, true), initial_train);
    assert_eq!(positions(&app, false), initial_land);
    assert_eq!(
        app.world().resource::<ViewState>().selected,
        Some(SurfaceId::Object(selected))
    );
}

struct InputSaveDirectory(std::path::PathBuf);

impl InputSaveDirectory {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "Parkbound input save é {} {timestamp} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn session(&self) -> SaveSession {
        SaveSession::from_arguments([
            std::ffi::OsString::from("--save-dir"),
            self.0.clone().into_os_string(),
        ])
    }

    fn store(&self) -> save_store::SaveStore {
        save_store::SaveStore::new(self.0.clone())
    }
}

impl Drop for InputSaveDirectory {
    fn drop(&mut self) {
        if let (Ok(path), Ok(root)) = (
            std::fs::canonicalize(&self.0),
            std::fs::canonicalize(std::env::temp_dir()),
        ) && path.parent() == Some(root.as_path())
            && path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("Parkbound input save "))
        {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

fn persistence_input_app(
    directory: &InputSaveDirectory,
    park: Park,
    view: ViewState,
) -> (App, Entity) {
    let (mut app, window) = input_app(park, view, 1.0);
    app.insert_resource(directory.session())
        .add_systems(Update, ui::button_actions.before(keyboard_input))
        .add_systems(Update, persistence::apply_save_action.after(pointer_input));
    (app, window)
}

fn persistence_key_frame(app: &mut App, pressed: &[KeyCode]) {
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.reset_all();
        for key in pressed {
            keys.press(*key);
        }
    }
    frame(app);
    // frame() clears edge events, while reset_all() also releases held keys.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
}

fn saved_file_pair(store: &save_store::SaveStore) -> (Vec<u8>, Vec<u8>) {
    (
        std::fs::read(store.save_path()).unwrap(),
        std::fs::read(store.backup_path()).unwrap(),
    )
}

fn persistence_parks() -> (Park, Park, Park) {
    let first = train_park();
    let mut second = first.clone();
    for _ in 0..150 {
        second.advance(Speed::Normal);
    }
    let mut live = second.clone();
    for _ in 0..200 {
        live.advance(Speed::Normal);
    }
    (first, second, live)
}

#[test]
fn save_load_and_both_shift_backup_keyboard_gestures_restore_the_expected_park() {
    let directory = InputSaveDirectory::new();
    let store = directory.store();
    let (first, second, live) = persistence_parks();
    let (mut app, _) = persistence_input_app(
        &directory,
        first.clone(),
        ViewState {
            speed: Speed::Fast,
            ..default()
        },
    );
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    assert_eq!(store.load().unwrap(), first);
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Fast);
    assert_eq!(app.world().resource::<Simulation>().0, first);
    assert!(!store.backup_path().exists());

    app.world_mut().resource_mut::<Simulation>().0 = second.clone();
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    assert_eq!(store.load_backup().unwrap(), first);
    let files = saved_file_pair(&store);
    app.world_mut().resource_mut::<Simulation>().0 = live.clone();
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, second);
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
    assert_eq!(app.world().resource::<ViewState>().save_pending, None);
    for shift in [KeyCode::ShiftLeft, KeyCode::ShiftRight] {
        app.world_mut().resource_mut::<Simulation>().0 = live.clone();
        persistence_key_frame(&mut app, &[shift, KeyCode::F9]);
        assert_eq!(app.world().resource::<Simulation>().0, first);
        assert!(
            app.world()
                .resource::<ViewState>()
                .save_notice
                .starts_with("Backup loaded")
        );
        assert_eq!(saved_file_pair(&store), files);
    }
}

#[test]
fn simultaneous_keyboard_and_button_load_requests_never_save_over_their_source() {
    for (keys, button, backup) in [
        (vec![KeyCode::F5, KeyCode::F9], None, false),
        (
            vec![KeyCode::F5, KeyCode::ShiftLeft, KeyCode::F9],
            None,
            true,
        ),
        (vec![KeyCode::F9], Some(ui::Action::Save), false),
        (vec![KeyCode::F5], Some(ui::Action::LoadBackup), true),
    ] {
        let directory = InputSaveDirectory::new();
        let store = directory.store();
        let (first, second, live) = persistence_parks();
        store.save(&first).unwrap();
        store.save(&second).unwrap();
        let before = saved_file_pair(&store);
        let (mut app, _) = persistence_input_app(&directory, live, ViewState::default());
        if let Some(action) = button {
            app.world_mut()
                .spawn((Button, Interaction::Pressed, action));
        }
        persistence_key_frame(&mut app, &keys);
        assert_eq!(
            app.world().resource::<Simulation>().0,
            if backup { first } else { second }
        );
        assert_eq!(saved_file_pair(&store), before);
        assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
        assert_eq!(app.world().resource::<ViewState>().save_pending, None);
    }
}

#[test]
fn rapid_released_backup_chords_keep_the_modifier_and_do_not_latch_it_for_later_loads() {
    for (shift, held_before_update) in [
        (KeyCode::ShiftLeft, false),
        (KeyCode::ShiftRight, false),
        (KeyCode::ShiftLeft, true),
        (KeyCode::ShiftRight, true),
    ] {
        let directory = InputSaveDirectory::new();
        let (backup, primary, live) = persistence_parks();
        directory.store().save(&backup).unwrap();
        directory.store().save(&primary).unwrap();
        let files = saved_file_pair(&directory.store());
        let (mut app, _) = persistence_input_app(&directory, live, ViewState::default());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(shift);
        if held_before_update {
            frame(&mut app);
        }
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::F9);
            keys.release(KeyCode::F9);
            keys.release(shift);
        }
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, backup);
        assert!(
            app.world()
                .resource::<ViewState>()
                .save_notice
                .starts_with("Backup loaded")
        );
        assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
        // The modifier is an update-local edge, not a sticky backup mode.
        persistence_key_frame(&mut app, &[KeyCode::F9]);
        assert_eq!(app.world().resource::<Simulation>().0, primary);
        assert_eq!(saved_file_pair(&directory.store()), files);
    }
}

#[test]
fn help_and_capture_gate_all_persistence_inputs_without_deferred_actions() {
    for (help, capture_active) in [(true, false), (false, true)] {
        let directory = InputSaveDirectory::new();
        let store = directory.store();
        let (first, second, live) = persistence_parks();
        store.save(&first).unwrap();
        store.save(&second).unwrap();
        let before = saved_file_pair(&store);
        let (mut app, _) = persistence_input_app(
            &directory,
            live.clone(),
            ViewState {
                help,
                capture_active,
                speed: Speed::Fast,
                ..default()
            },
        );
        for keys in [
            vec![KeyCode::F5],
            vec![KeyCode::F9],
            vec![KeyCode::ShiftRight, KeyCode::F9],
        ] {
            persistence_key_frame(&mut app, &keys);
            assert_eq!(app.world().resource::<Simulation>().0, live);
            assert_eq!(saved_file_pair(&store), before);
            assert_eq!(app.world().resource::<ViewState>().save_pending, None);
        }
        for action in [ui::Action::Save, ui::Action::Load, ui::Action::LoadBackup] {
            let button = app
                .world_mut()
                .spawn((Button, Interaction::Pressed, action))
                .id();
            frame(&mut app);
            assert_eq!(app.world().resource::<Simulation>().0, live);
            assert_eq!(saved_file_pair(&store), before);
            assert_eq!(app.world().resource::<ViewState>().save_pending, None);
            app.world_mut().despawn(button);
        }
        {
            let mut view = app.world_mut().resource_mut::<ViewState>();
            assert_eq!(view.speed, Speed::Fast);
            view.help = false;
            view.capture_active = false;
        }
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, live);
        assert_eq!(saved_file_pair(&store), before);
        persistence_key_frame(&mut app, &[KeyCode::F9]);
        assert_eq!(app.world().resource::<Simulation>().0, second);
    }
}

#[test]
fn save_load_and_backup_buttons_consume_one_press_and_do_not_repeat_while_held() {
    for action in [ui::Action::Save, ui::Action::Load, ui::Action::LoadBackup] {
        let directory = InputSaveDirectory::new();
        let store = directory.store();
        let (first, second, live) = persistence_parks();
        store.save(&first).unwrap();
        store.save(&second).unwrap();
        let selected = Some(SurfaceId::Object(live.objects()[0].id));
        let (mut app, window) = persistence_input_app(
            &directory,
            live.clone(),
            ViewState {
                selected,
                ..default()
            },
        );
        app.world_mut()
            .spawn((Button, Interaction::Pressed, action));
        set_cursor(&mut app, window, Some(Vec2::new(600.0, 400.0)));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        frame(&mut app);
        let expected = match action {
            ui::Action::Save => &live,
            ui::Action::Load => &second,
            ui::Action::LoadBackup => &first,
            _ => unreachable!(),
        };
        assert_eq!(&app.world().resource::<Simulation>().0, expected);
        let view = app.world().resource::<ViewState>();
        assert!(view.ui_pointer_consumed);
        assert_eq!(view.save_pending, None);
        assert_eq!(
            view.selected,
            if action == ui::Action::Save {
                selected
            } else {
                None
            }
        );
        assert_eq!(view.hovered, None);
        let files = saved_file_pair(&store);

        // Change live state while the UI component remains Pressed. A second
        // execution would either reload the old state or rotate/replace files.
        let mut advanced = expected.clone();
        advanced.advance(Speed::Normal);
        app.world_mut().resource_mut::<Simulation>().0 = advanced.clone();
        frame(&mut app);
        assert!(!app.world().resource::<ViewState>().ui_pointer_consumed);
        assert_eq!(app.world().resource::<Simulation>().0, advanced);
        assert_eq!(saved_file_pair(&store), files);
    }
}

fn mesh_vertex_positions(mesh: &Mesh) -> Vec<[f32; 3]> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .to_vec()
}

fn all_scene_positions(app: &App) -> [Vec<[f32; 3]>; 4] {
    let scene = app.world().resource::<SceneMeshes>();
    let meshes = app.world().resource::<Assets<Mesh>>();
    [
        app.world()
            .resource::<static_landscape::StaticLandscape>()
            .mesh_handles()
            .next()
            .unwrap(),
        &scene.guests,
        &scene.train,
        &scene.selection,
    ]
    .map(|handle| mesh_vertex_positions(meshes.get(handle).unwrap()))
}

#[test]
fn same_tick_load_refreshes_land_guests_train_and_clears_the_selection_mesh() {
    let directory = InputSaveDirectory::new();
    let mut saved = train_park();
    saved.set_terrain_height(TilePos { x: 0, y: 0 }, 3).unwrap();
    let mut live = Park::new(MAP_SIZE, MAP_SIZE);
    live.apply_terrain_profile(TerrainProfile::Terraced);
    live.install_crowd_fixture(CROWD_SEED.wrapping_add(1))
        .unwrap();
    live.install_train_fixture().unwrap();
    live.set_train_open(false);
    for _ in 0..150 {
        saved.advance(Speed::Normal);
        live.advance(Speed::Normal);
    }
    assert_eq!(saved.ticks(), live.ticks());
    assert_ne!(saved.guests(), live.guests());
    assert_ne!(saved.train_cars(), live.train_cars());
    directory.store().save(&saved).unwrap();
    let selected = Some(SurfaceId::Object(live.circuit().unwrap().pieces[0].object));
    let (mut app, _) = persistence_input_app(
        &directory,
        live,
        ViewState {
            selected,
            ..default()
        },
    );
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_scene.after(persistence::apply_save_action));
    frame(&mut app);
    let before = all_scene_positions(&app);
    assert!(before.iter().all(|vertices| !vertices.is_empty()));
    assert!(!app.world().resource::<ViewState>().scene_dirty);
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    let after = all_scene_positions(&app);
    for index in 0..3 {
        assert_ne!(after[index], before[index], "mesh {index} stayed stale");
    }
    assert!(after[3].is_empty());
    assert_eq!(app.world().resource::<Simulation>().0, saved);
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.selected, None);
    assert_eq!(view.hovered, None);
    assert_eq!(view.speed, Speed::Paused);
    assert!(!view.scene_dirty);
    let expected = [
        world::build_scene(&saved, view.rotation, view.grid, view.view_mode),
        world::guest_mesh(&saved, view.rotation, view.view_mode),
        world::train_mesh(&saved, view.rotation, view.view_mode),
    ];
    for index in 0..3 {
        let expected_positions = mesh_vertex_positions(&expected[index]);
        if index == 1 {
            // Cached local guest geometry is translated in a different order.
            // Permit only subpixel f32 rounding, while retaining exact counts.
            assert_eq!(after[index].len(), expected_positions.len());
            for (actual, expected) in after[index].iter().zip(&expected_positions) {
                for coordinate in 0..3 {
                    assert!((actual[coordinate] - expected[coordinate]).abs() <= 0.001);
                }
            }
        } else {
            assert_eq!(after[index], expected_positions);
        }
    }
}

#[test]
fn f10_sound_modal_consumes_simultaneous_save_build_and_close_inputs() {
    let directory = InputSaveDirectory::new();
    let park = train_park();
    let before = park.clone();
    let (mut app, _) = persistence_input_app(&directory, park, ViewState::default());
    persistence_key_frame(&mut app, &[KeyCode::F10, KeyCode::F5, KeyCode::Delete]);
    let v = app.world().resource::<ViewState>();
    assert!(v.help && v.audio_view);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(!directory.store().save_path().exists());
    persistence_key_frame(&mut app, &[KeyCode::F5, KeyCode::KeyT, KeyCode::KeyO]);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(!directory.store().save_path().exists());
    persistence_key_frame(&mut app, &[KeyCode::Escape, KeyCode::F5, KeyCode::Delete]);
    assert!(!app.world().resource::<ViewState>().help);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(!directory.store().save_path().exists());
}
