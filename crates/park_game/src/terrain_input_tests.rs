//! Exercise terrain commands through the real input/save/render schedules.
use super::*;
use crate::path_tools::{PathTool, ToolState};
use park_sim::{Axis, MAX_TERRAIN_HEIGHT, Terrain};

fn tool_view(mode: PathTool, rotation: u8) -> ViewState {
    ViewState {
        rotation,
        zoom: 1.4,
        speed: Speed::Paused,
        path_tool: ToolState { mode, ..default() },
        ..default()
    }
}

fn aim_point(app: &mut App, window: Entity, point: Vec2) {
    let cursor = cursor_for_world(app, window, point);
    assert!(is_map_position(
        cursor,
        app.world().get::<Window>(window).unwrap(),
        false
    ));
    set_cursor(app, window, Some(cursor));
}

fn aim_tile(app: &mut App, window: Entity, pos: TilePos) {
    let park = &app.world().resource::<Simulation>().0;
    let rotation = app.world().resource::<ViewState>().rotation;
    let point = world::elevated_tile_center(park, pos, rotation).unwrap();
    aim_point(app, window, point);
}

fn press_left(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
}

fn release_left(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
}

#[test]
fn terrain_click_matches_preview_at_each_rotation_dpi_and_does_not_paint() {
    for rotation in 0..4 {
        for dpi in [1.0, 1.5, 2.0] {
            for (mode, expected_height) in
                [(PathTool::RaiseTerrain, 7), (PathTool::LowerTerrain, 5)]
            {
                let pos = TilePos { x: 15, y: 16 };
                let mut park = Park::new(MAP_SIZE, MAP_SIZE);
                park.set_terrain_height(pos, 6).unwrap();
                let mut expected = park.clone();
                expected.set_terrain_height(pos, expected_height).unwrap();
                let mut view = tool_view(mode, rotation);
                view.pan = Vec2::new(40.0, -20.0);
                let (mut app, window) = input_app(park.clone(), view, dpi);
                aim_tile(&mut app, window, pos);
                frame(&mut app);
                assert_eq!(app.world().resource::<Simulation>().0, park);
                let preview = app
                    .world()
                    .resource::<ViewState>()
                    .path_tool
                    .terrain_preview
                    .unwrap();
                assert_eq!(
                    (
                        preview.pos,
                        preview.height,
                        preview.target_height,
                        preview.valid
                    ),
                    (pos, 6, expected_height, true)
                );
                press_left(&mut app);
                frame(&mut app);
                assert_eq!(app.world().resource::<Simulation>().0, expected);
                let view = app.world().resource::<ViewState>();
                assert!(view.terrain_custom && view.scene_dirty);
                assert_eq!(view.selected, Some(SurfaceId::Terrain(pos)));
                assert!(view.path_tool.terrain_preview.is_none());
                aim_tile(&mut app, window, TilePos { x: 14, y: 16 });
                frame(&mut app);
                assert_eq!(
                    app.world().resource::<Simulation>().0,
                    expected,
                    "held click must not paint"
                );
            }
        }
    }
}

#[test]
fn terrain_cliff_click_edits_the_visible_column_instead_of_hidden_land() {
    for rotation in 0..4 {
        let mut park = Park::new(8, 8);
        let behind = from_view_tile(3, 3, 8, rotation);
        let front = from_view_tile(4, 4, 8, rotation);
        park.set_terrain_height(front, 6).unwrap();
        let point =
            world::elevated_tile_center(&park, behind, rotation).unwrap() + Vec2::new(8.0, 0.0);
        assert_eq!(
            world::pick_surface(point, &park, rotation, world::ViewMode::Surface),
            Some(SurfaceId::Terrain(front))
        );
        let mut expected = park.clone();
        expected.set_terrain_height(front, 7).unwrap();
        let (mut app, window) = input_app(park, tool_view(PathTool::RaiseTerrain, rotation), 1.0);
        aim_point(&mut app, window, point);
        press_left(&mut app);
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, expected);
    }
}

#[test]
fn water_height_limits_and_underground_are_rejected_without_mutation() {
    for case in 0..4 {
        let mut park = Park::new(MAP_SIZE, MAP_SIZE);
        let mut pos = TilePos { x: 15, y: 16 };
        let mode = if case == 1 {
            PathTool::LowerTerrain
        } else {
            PathTool::RaiseTerrain
        };
        if case == 0 {
            park.set_terrain_height(pos, MAX_TERRAIN_HEIGHT).unwrap();
        }
        if case == 2 {
            pos = (0..MAP_SIZE)
                .flat_map(|y| (0..MAP_SIZE).map(move |x| TilePos { x, y }))
                .find(|&p| park.tile(p).unwrap().terrain == Terrain::Water)
                .unwrap();
        }
        let mut view = tool_view(mode, 0);
        view.zoom = 0.7;
        if case == 3 {
            view.view_mode = world::ViewMode::Underground;
        }
        let (mut app, window) = input_app(park.clone(), view, 1.0);
        aim_tile(&mut app, window, pos);
        press_left(&mut app);
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        let view = app.world().resource::<ViewState>();
        assert!(!view.scene_dirty && !view.terrain_custom);
        assert!(
            view.path_notice
                .as_ref()
                .unwrap()
                .starts_with("Terrain unchanged")
        );
        if case == 3 {
            assert!(view.path_tool.terrain_preview.is_none());
            assert!(view.path_tool.hint.contains("Surface"));
        } else {
            assert!(!view.path_tool.terrain_preview.unwrap().valid);
        }
    }
}

#[test]
fn terrain_clearance_rejects_raising_into_paths_and_lowering_into_buried_track() {
    let pos = TilePos { x: 15, y: 16 };
    for rotation in 0..4 {
        for lowering in [false, true] {
            let mut park = Park::new(MAP_SIZE, MAP_SIZE);
            let mode = if lowering {
                PathTool::LowerTerrain
            } else {
                PathTool::RaiseTerrain
            };
            let height = if lowering { 2 } else { 4 };
            park.set_terrain_height(pos, height).unwrap();
            if lowering {
                park.place_object(pos, ObjectKind::Track, Axis::X, 0, 2)
                    .unwrap();
            } else {
                park.place_path(pos, height).unwrap();
            }
            let roof = world::elevated_tile_center(&park, pos, rotation).unwrap();
            let point = if lowering {
                roof
            } else {
                roof + Vec2::new(16.0, -8.0 - 2.0 * world::HEIGHT_STEP)
            };
            assert_eq!(
                world::pick_surface(point, &park, rotation, world::ViewMode::Surface),
                Some(SurfaceId::Terrain(pos))
            );
            let bytes = park.to_save_bytes().unwrap();
            let (mut app, window) = input_app(park.clone(), tool_view(mode, rotation), 1.0);
            aim_point(&mut app, window, point);
            press_left(&mut app);
            frame(&mut app);
            assert_eq!(
                app.world()
                    .resource::<Simulation>()
                    .0
                    .to_save_bytes()
                    .unwrap(),
                bytes
            );
            let view = app.world().resource::<ViewState>();
            assert!(!view.path_tool.terrain_preview.unwrap().valid);
            assert!(
                view.path_notice
                    .as_ref()
                    .unwrap()
                    .contains("blocks terrain")
            );
            assert!(!view.terrain_custom);
        }
    }
}

#[test]
fn clicking_visible_objects_never_uses_the_ground_beneath_them() {
    let pos = TilePos { x: 15, y: 16 };
    for kind in [ObjectKind::Path, ObjectKind::Track] {
        let mut park = Park::new(MAP_SIZE, MAP_SIZE);
        let id = park.place_object(pos, kind, Axis::X, 8, 2).unwrap();
        let point = world::surface_center(&park, SurfaceId::Object(id), 0).unwrap();
        let (mut app, window) = input_app(park.clone(), tool_view(PathTool::RaiseTerrain, 0), 1.0);
        aim_point(&mut app, window, point);
        press_left(&mut app);
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        assert!(
            app.world()
                .resource::<ViewState>()
                .path_tool
                .terrain_preview
                .is_none()
        );
    }
}

#[test]
fn terrain_edit_gates_leave_no_deferred_click_after_focus_help_ui_or_pan() {
    for gate in 0..10 {
        let park = Park::new(MAP_SIZE, MAP_SIZE);
        let pos = TilePos { x: 15, y: 16 };
        let (mut app, window) = input_app(park.clone(), tool_view(PathTool::RaiseTerrain, 0), 1.0);
        aim_tile(&mut app, window, pos);
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
                app.world_mut().resource_mut::<ViewState>().diagnostic_case =
                    Some(park_sim::diagnostics::DiagnosticCase::SaveScale)
            }
            5 => app
                .world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Right),
            6 => app
                .world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Middle),
            7 => set_cursor(
                &mut app,
                window,
                Some(Vec2::new(
                    1100.0,
                    ui::PATH_PANEL_TOP + ui::PATH_PANEL_HEIGHT - 2.0,
                )),
            ),
            8 => app.world_mut().resource_mut::<ViewState>().save_pending = Some(SaveAction::Load),
            9 => set_cursor(&mut app, window, None),
            _ => unreachable!(),
        }
        press_left(&mut app);
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park, "gate {gate}");
        assert!(
            app.world()
                .resource::<ViewState>()
                .path_tool
                .terrain_preview
                .is_none()
        );
        release_left(&mut app);
        *app.world_mut().resource_mut::<ViewState>() = tool_view(PathTool::RaiseTerrain, 0);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        aim_tile(&mut app, window, pos);
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
    }
}

#[test]
fn terrain_shortcuts_buttons_and_help_close_switch_without_click_through() {
    for (key, mode) in [
        (KeyCode::KeyB, PathTool::RaiseTerrain),
        (KeyCode::KeyV, PathTool::LowerTerrain),
    ] {
        let mut park = Park::new(MAP_SIZE, MAP_SIZE);
        let pos = TilePos { x: 15, y: 16 };
        park.set_terrain_height(pos, 6).unwrap();
        let (mut app, window) = input_app(park.clone(), ViewState::default(), 1.0);
        app.add_systems(Update, ui::button_actions.before(keyboard_input));
        aim_tile(&mut app, window, pos);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        frame(&mut app);
        assert_eq!(app.world().resource::<ViewState>().path_tool.mode, mode);
        assert!(
            app.world()
                .resource::<ViewState>()
                .path_tool
                .terrain_preview
                .is_some()
        );
        assert_eq!(app.world().resource::<Simulation>().0, park);
        let button = app
            .world_mut()
            .spawn((Button, Interaction::Pressed, ui::Action::SetPathTool(mode)))
            .id();
        press_left(&mut app);
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        assert!(
            app.world()
                .resource::<ViewState>()
                .path_tool
                .terrain_preview
                .is_none()
        );
        app.world_mut().despawn(button);
        release_left(&mut app);
        for close in [KeyCode::KeyH, KeyCode::F1, KeyCode::Escape] {
            app.world_mut().resource_mut::<ViewState>().help = true;
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(close);
            press_left(&mut app);
            frame(&mut app);
            assert_eq!(app.world().resource::<Simulation>().0, park);
            assert!(!app.world().resource::<ViewState>().help);
            release_left(&mut app);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyP);
        frame(&mut app);
        assert_eq!(
            app.world().resource::<ViewState>().path_tool.mode,
            PathTool::Build
        );
        assert!(
            app.world()
                .resource::<ViewState>()
                .path_tool
                .terrain_preview
                .is_none()
        );
    }
}

#[test]
fn terrain_same_update_save_backup_and_failed_load_preserve_edit_order() {
    let directory = InputSaveDirectory::new();
    let pos = TilePos { x: 3, y: 24 };
    let mut park = train_park();
    // Use an empty meadow tile, away from the fixture objects.
    park.set_terrain_height(pos, 4).unwrap();
    let (mut app, window) = persistence_input_app(
        &directory,
        park.clone(),
        tool_view(PathTool::RaiseTerrain, 0),
    );
    aim_tile(&mut app, window, pos);
    press_left(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F9]); // Missing slot still suppresses terrain edits.
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert!(
        app.world()
            .resource::<ViewState>()
            .save_notice
            .starts_with("Load failed")
    );
    release_left(&mut app);
    aim_tile(&mut app, window, pos);
    press_left(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let raised = app.world().resource::<Simulation>().0.clone();
    assert_eq!(raised.tile(pos).unwrap().height, 5);
    assert_eq!(directory.store().load().unwrap(), raised);
    release_left(&mut app);
    app.world_mut()
        .resource_mut::<ViewState>()
        .path_tool
        .set_mode(PathTool::LowerTerrain);
    aim_tile(&mut app, window, pos);
    press_left(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    assert_eq!(directory.store().load().unwrap(), park);
    assert_eq!(directory.store().load_backup().unwrap(), raised);
    release_left(&mut app);
    app.world_mut()
        .resource_mut::<ViewState>()
        .path_tool
        .set_mode(PathTool::LowerTerrain);
    aim_tile(&mut app, window, pos);
    press_left(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::ShiftLeft, KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, raised);
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.path_tool.mode, PathTool::Inspect);
    assert!(view.path_tool.terrain_preview.is_none() && view.selected.is_none());
    assert!(view.terrain_custom);
    assert_eq!(view.speed, Speed::Paused);
}

#[test]
fn terrain_preset_consumes_the_same_frame_tool_click_and_resets_custom_label() {
    let pos = TilePos { x: 15, y: 16 };
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.set_terrain_height(pos, 5).unwrap();
    let mut view = tool_view(PathTool::RaiseTerrain, 0);
    view.terrain_custom = true;
    let (mut app, window) = input_app(park.clone(), view, 1.0);
    app.add_systems(
        Update,
        apply_terrain_profile
            .after(keyboard_input)
            .before(pointer_input),
    );
    aim_tile(&mut app, window, pos);
    press_left(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyT);
    frame(&mut app);
    park.try_apply_terrain_profile(TerrainProfile::Flat)
        .unwrap();
    assert_eq!(app.world().resource::<Simulation>().0, park);
    let view = app.world().resource::<ViewState>();
    assert!(!view.terrain_custom);
    assert!(view.path_tool.terrain_preview.is_none());
}

#[test]
fn paused_terrain_and_same_tick_load_refresh_land_preview_guests_and_train() {
    let directory = InputSaveDirectory::new();
    let pos = TilePos { x: 3, y: 24 };
    let park = train_park();
    directory.store().save(&park).unwrap();
    let (mut app, window) = persistence_input_app(
        &directory,
        park.clone(),
        tool_view(PathTool::RaiseTerrain, 0),
    );
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_scene.after(persistence::apply_save_action));
    aim_tile(&mut app, window, pos);
    frame(&mut app);
    let scene = app.world().resource::<SceneMeshes>();
    assert!(
        app.world()
            .resource::<Assets<Mesh>>()
            .get(&scene.path_preview)
            .unwrap()
            .count_vertices()
            > 0
    );
    let original_meshes = all_scene_positions(&app);
    let original_handles = app.world().resource::<Assets<Mesh>>().len();
    press_left(&mut app);
    frame(&mut app);
    let edited = &app.world().resource::<Simulation>().0;
    assert_eq!(edited.ticks(), park.ticks());
    assert_eq!(
        edited.tile(pos).unwrap().height,
        park.tile(pos).unwrap().height + 1
    );
    let meshes = all_scene_positions(&app);
    assert_ne!(meshes[0], original_meshes[0]);
    assert_eq!(
        meshes[0],
        mesh_vertex_positions(&world::build_scene(
            edited,
            0,
            true,
            world::ViewMode::Surface
        ))
    );
    assert_eq!(meshes[1], original_meshes[1]);
    let guest_oracle =
        mesh_vertex_positions(&world::guest_mesh(edited, 0, world::ViewMode::Surface));
    assert_eq!(meshes[1].len(), guest_oracle.len());
    for (actual, expected) in meshes[1].iter().zip(&guest_oracle) {
        // Cached local guest geometry changes f32 addition order only.
        assert!((actual[0] - expected[0]).abs() <= 0.001);
        assert!((actual[1] - expected[1]).abs() <= 0.001);
        assert_eq!(actual[2], expected[2]);
    }
    assert_eq!(
        meshes[2],
        mesh_vertex_positions(&world::train_mesh(edited, 0, world::ViewMode::Surface))
    );
    let scene = app.world().resource::<SceneMeshes>();
    assert_eq!(
        app.world()
            .resource::<Assets<Mesh>>()
            .get(&scene.path_preview)
            .unwrap()
            .count_vertices(),
        0
    );
    assert_eq!(
        app.world().resource::<Assets<Mesh>>().len(),
        original_handles
    );
    release_left(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    let restored = all_scene_positions(&app);
    assert_eq!(&restored[..3], &original_meshes[..3]);
    assert!(restored[3].is_empty());
    assert_eq!(
        app.world().resource::<ViewState>().path_tool.mode,
        PathTool::Inspect
    );
}
