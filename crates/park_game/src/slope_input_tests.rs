//! Sloped-path commands through the production input/save/render systems.
use super::*;
use crate::path_tools::{PathTool, ToolState};
use park_sim::PathSlope;

fn build_view(slope: PathSlope, rotation: u8) -> ViewState {
    ViewState {
        rotation,
        zoom: 1.4,
        speed: Speed::Paused,
        path_tool: ToolState {
            mode: PathTool::Build,
            height: 8,
            slope,
            ..default()
        },
        ..default()
    }
}

fn aim(app: &mut App, window: Entity, pos: TilePos) {
    let view = app.world().resource::<ViewState>();
    let point = world::path_build_center(
        pos,
        &app.world().resource::<Simulation>().0,
        view.rotation,
        view.path_tool.height,
        view.path_tool.slope,
    )
    .unwrap();
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
fn every_slope_shape_preview_commits_the_picked_tile_at_each_rotation_and_dpi() {
    for slope in [
        PathSlope::North,
        PathSlope::East,
        PathSlope::South,
        PathSlope::West,
    ] {
        for rotation in 0..4 {
            for dpi in [1.0, 1.5, 2.0] {
                let pos = TilePos { x: 15, y: 17 };
                let park = Park::new(MAP_SIZE, MAP_SIZE);
                let mut expected = park.clone();
                let id = expected.place_sloped_path(pos, 8, slope).unwrap();
                let (mut app, window) = input_app(park.clone(), build_view(slope, rotation), dpi);
                aim(&mut app, window, pos);
                frame(&mut app);
                assert_eq!(app.world().resource::<Simulation>().0, park);
                let preview = app
                    .world()
                    .resource::<ViewState>()
                    .path_tool
                    .preview
                    .unwrap();
                assert_eq!(
                    (preview.pos, preview.slope, preview.valid),
                    (pos, slope, true)
                );
                click(&mut app);
                frame(&mut app);
                assert_eq!(app.world().resource::<Simulation>().0, expected);
                assert_eq!(
                    app.world().resource::<ViewState>().selected,
                    Some(SurfaceId::Object(id))
                );
                aim(&mut app, window, TilePos { x: 14, y: 17 });
                frame(&mut app);
                assert_eq!(
                    app.world().resource::<Simulation>().0,
                    expected,
                    "held click must not paint ramps"
                );
            }
        }
    }
}

#[test]
fn shape_keyboard_and_button_cycle_clamp_height_and_respect_help_diagnostic_capture_gates() {
    let (mut app, _) = input_app(Park::new(MAP_SIZE, MAP_SIZE), ViewState::default(), 1.0);
    app.add_systems(Update, ui::button_actions.before(keyboard_input));
    app.world_mut().resource_mut::<ViewState>().path_tool.height = 30;
    for shape in [
        PathSlope::North,
        PathSlope::East,
        PathSlope::South,
        PathSlope::West,
        PathSlope::Flat,
    ] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyC);
        frame(&mut app);
        let view = app.world().resource::<ViewState>();
        assert_eq!(view.path_tool.slope, shape);
        assert_eq!(view.path_tool.height, 29);
        assert_eq!(view.path_tool.mode, PathTool::Build);
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    let button = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::CyclePathShape))
        .id();
    frame(&mut app);
    assert_eq!(
        app.world().resource::<ViewState>().path_tool.slope,
        PathSlope::North
    );
    assert!(app.world().resource::<ViewState>().ui_pointer_consumed);
    frame(&mut app); // Held button cannot repeat.
    assert_eq!(
        app.world().resource::<ViewState>().path_tool.slope,
        PathSlope::North
    );
    app.world_mut().despawn(button);
    for gate in 0..3 {
        {
            let mut view = app.world_mut().resource_mut::<ViewState>();
            view.help = gate == 0;
            view.capture_active = gate == 1;
            view.diagnostic_case =
                (gate == 2).then_some(park_sim::diagnostics::DiagnosticCase::SaveScale);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyC);
        frame(&mut app);
        assert_eq!(
            app.world().resource::<ViewState>().path_tool.slope,
            PathSlope::North
        );
    }
}

#[test]
fn slope_clicks_obey_ui_focus_pan_and_load_gates_without_deferred_edits() {
    for gate in 0..8 {
        let park = Park::new(MAP_SIZE, MAP_SIZE);
        let (mut app, window) = input_app(park.clone(), build_view(PathSlope::South, 0), 1.0);
        aim(&mut app, window, TilePos { x: 15, y: 17 });
        match gate {
            0 => app.world_mut().resource_mut::<ViewState>().help = true,
            1 => {
                app.world_mut()
                    .resource_mut::<ViewState>()
                    .ui_pointer_consumed = true
            }
            2 => app.world_mut().get_mut::<Window>(window).unwrap().focused = false,
            3 => app.world_mut().resource_mut::<ViewState>().capture_active = true,
            4 => app.world_mut().resource_mut::<ViewState>().save_pending = Some(SaveAction::Load),
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
                Some(Vec2::new(
                    1200.0,
                    ui::PATH_PANEL_TOP + ui::PATH_PANEL_HEIGHT - 2.0,
                )),
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
        aim(&mut app, window, TilePos { x: 15, y: 17 });
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
    }
}

#[test]
fn blocked_slope_preview_and_click_preserve_every_authoritative_field() {
    for rotation in 0..4 {
        let pos = TilePos { x: 15, y: 17 };
        let mut park = Park::new(MAP_SIZE, MAP_SIZE);
        park.place_path(pos, 10).unwrap(); // Ramp needs the complete [8,11) column.
        let before = park.to_save_bytes().unwrap();
        let (mut app, window) = input_app(park, build_view(PathSlope::East, rotation), 1.0);
        aim(&mut app, window, pos);
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
        assert!(
            app.world()
                .resource::<ViewState>()
                .path_notice
                .as_ref()
                .unwrap()
                .starts_with("No path placed")
        );
    }
}

#[test]
fn slope_build_save_demolish_backup_and_failed_load_follow_real_command_order() {
    let directory = InputSaveDirectory::new();
    let pos = TilePos { x: 15, y: 17 };
    let park = Park::new(MAP_SIZE, MAP_SIZE);
    let (mut app, window) =
        persistence_input_app(&directory, park.clone(), build_view(PathSlope::West, 0));
    aim(&mut app, window, pos);
    click(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    release(&mut app);
    aim(&mut app, window, pos);
    click(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let built = app.world().resource::<Simulation>().0.clone();
    assert_eq!(built.objects().len(), 1);
    assert_eq!(built.objects()[0].path_slope, PathSlope::West);
    assert_eq!(directory.store().load().unwrap(), built);
    release(&mut app);
    app.world_mut()
        .resource_mut::<ViewState>()
        .path_tool
        .set_mode(PathTool::Demolish);
    aim(&mut app, window, pos);
    click(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    assert!(app.world().resource::<Simulation>().0.objects().is_empty());
    assert_eq!(directory.store().load_backup().unwrap(), built);
    release(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::ShiftLeft, KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, built);
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.path_tool.mode, PathTool::Inspect);
    assert_eq!(view.path_tool.slope, PathSlope::Flat);
    assert_eq!(view.speed, Speed::Paused);
}

#[test]
fn paused_slope_build_and_load_refresh_static_meshes_and_reuse_preview_slot() {
    let directory = InputSaveDirectory::new();
    let park = Park::new(MAP_SIZE, MAP_SIZE);
    directory.store().save(&park).unwrap();
    let (mut app, window) =
        persistence_input_app(&directory, park.clone(), build_view(PathSlope::North, 2));
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_scene.after(persistence::apply_save_action));
    aim(&mut app, window, TilePos { x: 15, y: 17 });
    frame(&mut app);
    let mesh_count = app.world().resource::<Assets<Mesh>>().len();
    let original = all_scene_positions(&app);
    click(&mut app);
    frame(&mut app);
    let edited = &app.world().resource::<Simulation>().0;
    assert_eq!(edited.objects().len(), 1);
    assert_eq!(edited.ticks(), 0);
    let after = all_scene_positions(&app);
    assert_ne!(after[0], original[0]);
    assert_eq!(
        after[0],
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
    assert_eq!(app.world().resource::<ViewState>().rotation, 0);
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
