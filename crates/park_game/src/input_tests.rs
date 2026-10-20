//! Exercise the game input systems without a windowing backend or GPU.
//!
//! These tests supply Bevy input resources, not native Windows events. They do
//! not replace a normal-input check of the packaged desktop application.

use std::time::Duration;

use super::*;

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
                selected: Some(selected),
                hovered: Some(selected),
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
        assert_eq!(view.selected, Some(selected));
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
            selected: Some(selected),
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
        assert_eq!(view.selected, Some(selected));
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
            selected: Some(selected),
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
    assert_eq!(view.selected, Some(selected));
    assert_eq!(view.rotation, 2);

    app.world_mut()
        .resource_mut::<ViewState>()
        .toggle_terrain_profile();
    app.update();
    assert_eq!(selected_mesh_positions(&app), raised_positions);
}
