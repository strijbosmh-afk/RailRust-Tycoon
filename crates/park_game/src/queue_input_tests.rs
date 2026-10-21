//! Queue and gate construction through production input, persistence and meshes.
use super::*;
use crate::path_tools::{PathTool, ToolState};
use park_sim::{PathDirection, PathSlope};

const GATE: TilePos = TilePos { x: 8, y: 11 };
const QUEUE: TilePos = TilePos { x: 8, y: 12 };

fn tool_view(mode: PathTool, rotation: u8) -> ViewState {
    ViewState {
        rotation,
        zoom: 1.8,
        speed: Speed::Paused,
        path_tool: ToolState { mode, ..default() },
        ..default()
    }
}

fn aim(app: &mut App, window: Entity, pos: TilePos) {
    let view = app.world().resource::<ViewState>();
    let park = &app.world().resource::<Simulation>().0;
    let slope = if view.path_tool.mode == PathTool::Entrance {
        PathSlope::Flat
    } else {
        view.path_tool.slope
    };
    let center =
        world::path_build_center(pos, park, view.rotation, view.path_tool.height, slope).unwrap();
    // The track may cover the center at some rotations. Choose a visible point
    // on the requested construction plane, exactly as a human can do.
    let point = std::iter::once(center)
        .chain((-24..=24).step_by(4).flat_map(|y| {
            (-28..=28)
                .step_by(4)
                .map(move |x| center + Vec2::new(x as f32, y as f32))
        }))
        .find(|point| {
            world::pick_sloped_path_plane(*point, park, view.rotation, view.path_tool.height, slope)
                == Some(pos)
                && !world::sloped_path_plane_occluded(
                    *point,
                    park,
                    view.rotation,
                    view.path_tool.height,
                    slope,
                    view.view_mode,
                )
        })
        .unwrap();
    let pan = point - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    app.world_mut().resource_mut::<ViewState>().pan = pan;
    let cursor = cursor_for_world(app, window, point);
    assert!(is_map_position(
        cursor,
        app.world().get::<Window>(window).unwrap(),
        false
    ));
    set_cursor(app, window, Some(cursor));
}

fn click(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
}
fn release(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
}

#[test]
fn queue_and_entrance_previews_commit_once_at_every_rotation_and_dpi() {
    for mode in [PathTool::BuildQueue, PathTool::Entrance] {
        for rotation in 0..4 {
            for dpi in [1.0, 1.5, 2.0] {
                let park = train_park();
                let ride = park.ride().unwrap().id;
                let pos = if mode == PathTool::Entrance {
                    GATE
                } else {
                    QUEUE
                };
                let mut expected = park.clone();
                let id = if mode == PathTool::Entrance {
                    expected
                        .place_ride_entrance(pos, 8, PathDirection::North, ride)
                        .unwrap()
                } else {
                    expected
                        .place_queue_path(pos, 8, PathSlope::Flat, ride)
                        .unwrap()
                };
                let (mut app, window) = input_app(park.clone(), tool_view(mode, rotation), dpi);
                aim(&mut app, window, pos);
                frame(&mut app);
                assert_eq!(app.world().resource::<Simulation>().0, park);
                let preview = app
                    .world()
                    .resource::<ViewState>()
                    .path_tool
                    .preview
                    .unwrap();
                assert_eq!((preview.pos, preview.valid), (pos, true));
                assert_eq!(
                    preview.entrance_facing.is_some(),
                    mode == PathTool::Entrance
                );
                click(&mut app);
                frame(&mut app);
                assert_eq!(app.world().resource::<Simulation>().0, expected);
                assert_eq!(
                    app.world().resource::<ViewState>().selected,
                    Some(SurfaceId::Object(id))
                );
                aim(&mut app, window, TilePos { x: 8, y: 13 });
                frame(&mut app);
                assert_eq!(
                    app.world().resource::<Simulation>().0,
                    expected,
                    "held click cannot paint"
                );
            }
        }
    }
}

#[test]
fn queue_and_gate_keyboard_and_buttons_keep_their_modes_and_obey_edit_gates() {
    let (mut app, _) = input_app(train_park(), ViewState::default(), 1.0);
    app.add_systems(Update, ui::button_actions.before(keyboard_input));
    for (key, mode) in [
        (KeyCode::KeyJ, PathTool::BuildQueue),
        (KeyCode::KeyI, PathTool::Entrance),
    ] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        frame(&mut app);
        assert_eq!(app.world().resource::<ViewState>().path_tool.mode, mode);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyC);
        frame(&mut app);
        let tool = &app.world().resource::<ViewState>().path_tool;
        assert_eq!(tool.mode, mode);
        if mode == PathTool::Entrance {
            assert_eq!(tool.entrance_facing, PathDirection::East);
        } else {
            assert_eq!(tool.slope, PathSlope::North);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ViewState>()
            .path_tool
            .set_mode(PathTool::Inspect);
        let button = app
            .world_mut()
            .spawn((Button, Interaction::Pressed, ui::Action::SetPathTool(mode)))
            .id();
        frame(&mut app);
        assert_eq!(app.world().resource::<ViewState>().path_tool.mode, mode);
        app.world_mut().despawn(button);
        for gate in 0..4 {
            {
                let mut view = app.world_mut().resource_mut::<ViewState>();
                view.path_tool.set_mode(PathTool::Inspect);
                view.help = gate == 0;
                view.capture_active = gate == 1;
                view.diagnostic_case =
                    (gate == 2).then_some(park_sim::diagnostics::DiagnosticCase::SaveScale);
                view.save_pending = (gate == 3).then_some(SaveAction::Load);
            }
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            let button = app
                .world_mut()
                .spawn((Button, Interaction::Pressed, ui::Action::SetPathTool(mode)))
                .id();
            frame(&mut app);
            assert_eq!(
                app.world().resource::<ViewState>().path_tool.mode,
                PathTool::Inspect
            );
            app.world_mut().despawn(button);
        }
        let mut view = app.world_mut().resource_mut::<ViewState>();
        view.help = false;
        view.capture_active = false;
        view.diagnostic_case = None;
        view.save_pending = None;
    }
}

#[test]
fn queue_and_gate_clicks_obey_focus_ui_pan_and_pending_load() {
    for mode in [PathTool::BuildQueue, PathTool::Entrance] {
        for gate in 0..8 {
            let park = train_park();
            let (mut app, window) = input_app(park.clone(), tool_view(mode, 0), 1.0);
            let pos = if mode == PathTool::Entrance {
                GATE
            } else {
                QUEUE
            };
            aim(&mut app, window, pos);
            match gate {
                0 => app.world_mut().resource_mut::<ViewState>().help = true,
                1 => {
                    app.world_mut()
                        .resource_mut::<ViewState>()
                        .ui_pointer_consumed = true
                }
                2 => app.world_mut().get_mut::<Window>(window).unwrap().focused = false,
                3 => app.world_mut().resource_mut::<ViewState>().capture_active = true,
                4 => {
                    app.world_mut().resource_mut::<ViewState>().save_pending =
                        Some(SaveAction::Load)
                }
                5 => app
                    .world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .press(MouseButton::Right),
                6 => app
                    .world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .press(MouseButton::Middle),
                _ => set_cursor(
                    &mut app,
                    window,
                    Some(Vec2::new(1200.0, ui::PATH_PANEL_TOP + 20.0)),
                ),
            }
            click(&mut app);
            frame(&mut app);
            assert_eq!(app.world().resource::<Simulation>().0, park);
            assert!(
                app.world()
                    .resource::<ViewState>()
                    .path_tool
                    .preview
                    .is_none()
            );
            {
                let mut view = app.world_mut().resource_mut::<ViewState>();
                view.help = false;
                view.ui_pointer_consumed = false;
                view.capture_active = false;
                view.save_pending = None;
            }
            app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
            release(&mut app);
            aim(&mut app, window, pos);
            frame(&mut app);
            assert_eq!(app.world().resource::<Simulation>().0, park);
        }
    }
}

#[test]
fn invalid_gate_and_missing_ride_are_atomic_through_pointer_input() {
    for mode in [PathTool::Entrance, PathTool::BuildQueue] {
        let park = if mode == PathTool::Entrance {
            train_park()
        } else {
            Park::new(MAP_SIZE, MAP_SIZE)
        };
        let before = park.to_save_bytes().unwrap();
        let (mut app, window) = input_app(park, tool_view(mode, 0), 1.0);
        aim(&mut app, window, QUEUE); // Not beside a station.
        frame(&mut app);
        assert!(
            !app.world()
                .resource::<ViewState>()
                .path_tool
                .preview
                .unwrap()
                .valid
        );
        click(&mut app);
        frame(&mut app);
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

#[test]
fn gate_build_save_demolish_backup_and_failed_load_follow_command_order() {
    let directory = InputSaveDirectory::new();
    let park = train_park();
    let (mut app, window) =
        persistence_input_app(&directory, park.clone(), tool_view(PathTool::Entrance, 0));
    app.add_systems(
        Update,
        apply_path_actions
            .after(keyboard_input)
            .before(pointer_input),
    );
    aim(&mut app, window, GATE);
    click(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    release(&mut app);
    aim(&mut app, window, GATE);
    click(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let built = app.world().resource::<Simulation>().0.clone();
    let gate = built.ride().unwrap().entrance.as_ref().unwrap().object;
    assert_eq!(directory.store().load().unwrap(), built);
    release(&mut app);
    app.world_mut().resource_mut::<ViewState>().selected = Some(SurfaceId::Object(gate));
    persistence_key_frame(&mut app, &[KeyCode::Delete, KeyCode::F5]);
    assert!(
        app.world()
            .resource::<Simulation>()
            .0
            .ride()
            .unwrap()
            .entrance
            .is_none()
    );
    assert_eq!(directory.store().load_backup().unwrap(), built);
    persistence_key_frame(&mut app, &[KeyCode::ShiftLeft, KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, built);
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.path_tool.mode, PathTool::Inspect);
    assert_eq!(view.speed, Speed::Paused);
}

#[test]
fn paused_queue_build_and_load_refresh_meshes_without_extra_assets() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    let ride = park.ride().unwrap().id;
    park.place_ride_entrance(GATE, 8, PathDirection::North, ride)
        .unwrap();
    directory.store().save(&park).unwrap();
    let (mut app, window) =
        persistence_input_app(&directory, park.clone(), tool_view(PathTool::BuildQueue, 2));
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_scene.after(persistence::apply_save_action));
    aim(&mut app, window, QUEUE);
    frame(&mut app);
    let mesh_count = app.world().resource::<Assets<Mesh>>().len();
    let original = all_scene_positions(&app);
    click(&mut app);
    frame(&mut app);
    let edited = &app.world().resource::<Simulation>().0;
    assert_eq!(edited.ticks(), 0);
    assert_ne!(all_scene_positions(&app)[0], original[0]);
    assert_eq!(
        all_scene_positions(&app)[0],
        mesh_vertex_positions(&world::build_scene(
            edited,
            2,
            true,
            world::ViewMode::Surface
        ))
    );
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
    release(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(
        all_scene_positions(&app)[0],
        mesh_vertex_positions(&world::build_scene(
            &park,
            0,
            true,
            world::ViewMode::Surface
        ))
    );
}
