//! Production systems exercised with Bevy inputs, not native Windows events.
use super::*;
use crate::path_tools::{PathTool, ToolState};

fn build_view(rotation: u8) -> ViewState {
    ViewState {
        rotation,
        speed: Speed::Paused,
        zoom: 1.4,
        path_tool: ToolState {
            mode: PathTool::Build,
            ..default()
        },
        ..default()
    }
}

fn aim(app: &mut App, window: Entity, pos: TilePos, height: u8) {
    let view = app.world().resource::<ViewState>();
    let point = world::tile_center(pos, MAP_SIZE, view.rotation)
        + Vec2::Y * (f32::from(height) + 0.22) * world::HEIGHT_STEP;
    let cursor = cursor_for_world(app, window, point);
    assert!(is_map_position(
        cursor,
        app.world().get::<Window>(window).unwrap(),
        false
    ));
    set_cursor(app, window, Some(cursor));
}

fn press_left(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
}

#[test]
fn path_click_matches_preview_under_rotation_dpi_and_camera_and_does_not_paint_while_held() {
    for rotation in 0..4 {
        for dpi in [1.0, 1.5, 2.0] {
            let pos = TilePos { x: 14, y: 17 };
            let mut view = build_view(rotation);
            view.pan = Vec2::new(40.0, -20.0);
            let (mut app, window) = input_app(Park::new(MAP_SIZE, MAP_SIZE), view, dpi);
            aim(&mut app, window, pos, 8);
            frame(&mut app);
            let preview = app
                .world()
                .resource::<ViewState>()
                .path_tool
                .preview
                .unwrap();
            assert_eq!((preview.pos, preview.height, preview.valid), (pos, 8, true));
            press_left(&mut app);
            frame(&mut app);
            let park = &app.world().resource::<Simulation>().0;
            let object = park.objects()[0];
            assert_eq!(
                (object.pos, object.height, object.kind),
                (pos, 8, ObjectKind::Path)
            );
            assert_eq!(park.ticks(), 0);
            assert!(
                app.world()
                    .resource::<ViewState>()
                    .path_tool
                    .preview
                    .is_none()
            );
            assert!(app.world().resource::<ViewState>().scene_dirty);
            aim(&mut app, window, TilePos { x: 15, y: 17 }, 8);
            frame(&mut app);
            assert_eq!(app.world().resource::<Simulation>().0.objects(), &[object]);
        }
    }
}

#[test]
fn rejected_build_keeps_authoritative_state_and_reports_the_reason() {
    let pos = TilePos { x: 14, y: 17 };
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.place_path(pos, 8).unwrap();
    let (mut app, window) = input_app(park.clone(), build_view(0), 1.0);
    aim(&mut app, window, pos, 8);
    press_left(&mut app);
    frame(&mut app);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    let view = app.world().resource::<ViewState>();
    assert!(!view.scene_dirty);
    assert!(!view.path_tool.preview.unwrap().valid);
    assert!(
        view.path_notice
            .as_ref()
            .unwrap()
            .contains("Tile 15, 18 is occupied")
    );
}

#[test]
fn help_ui_focus_drag_and_diagnostic_gates_leave_no_deferred_construction() {
    for block in 0..9 {
        let park = Park::new(MAP_SIZE, MAP_SIZE);
        let (mut app, window) = input_app(park.clone(), build_view(0), 1.0);
        aim(&mut app, window, TilePos { x: 14, y: 17 }, 8);
        match block {
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
                Some(Vec2::new(1100.0, ui::PATH_PANEL_TOP + 20.0)),
            ),
            8 => app.world_mut().resource_mut::<ViewState>().save_pending = Some(SaveAction::Load),
            _ => unreachable!(),
        }
        press_left(&mut app);
        frame(&mut app);
        assert_eq!(
            app.world().resource::<Simulation>().0,
            park,
            "block {block}"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
        app.world_mut().resource_mut::<ViewState>().help = false;
        app.world_mut().resource_mut::<ViewState>().capture_active = false;
        app.world_mut().resource_mut::<ViewState>().diagnostic_case = None;
        app.world_mut()
            .resource_mut::<ViewState>()
            .ui_pointer_consumed = false;
        app.world_mut().resource_mut::<ViewState>().save_pending = None;
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        aim(&mut app, window, TilePos { x: 14, y: 17 }, 8);
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
    }
}

#[test]
fn demolition_targets_the_visible_path_and_never_removes_track() {
    let pos = TilePos { x: 14, y: 17 };
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    let lower = park.place_path(pos, 8).unwrap();
    let track = park
        .place_object(pos, ObjectKind::Track, park_sim::Axis::X, 10, 2)
        .unwrap();
    let mut view = build_view(0);
    view.path_tool.mode = PathTool::Demolish;
    let (mut app, window) = input_app(park.clone(), view, 1.0);
    let point = world::surface_center(&park, SurfaceId::Object(track), 0).unwrap();
    let cursor = cursor_for_world(&app, window, point);
    set_cursor(&mut app, window, Some(cursor));
    press_left(&mut app);
    frame(&mut app);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    // Remove only the diagnostic occluder, then click the visible lower path.
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .remove_object(track);
    aim(&mut app, window, pos, 8);
    frame(&mut app);
    assert!(
        app.world()
            .resource::<ViewState>()
            .path_tool
            .preview
            .unwrap()
            .demolish
    );
    press_left(&mut app);
    frame(&mut app);
    assert!(
        app.world()
            .resource::<Simulation>()
            .0
            .object(lower)
            .is_none()
    );
    assert!(
        app.world()
            .resource::<ViewState>()
            .path_notice
            .as_ref()
            .unwrap()
            .contains("removed")
    );
}

#[test]
fn path_shortcuts_buttons_height_limits_and_escape_cancel_without_click_through() {
    let (mut app, window) = input_app(Park::new(MAP_SIZE, MAP_SIZE), ViewState::default(), 1.0);
    app.add_systems(Update, ui::button_actions.before(keyboard_input));
    let button = app
        .world_mut()
        .spawn((
            Button,
            Interaction::Pressed,
            ui::Action::SetPathTool(PathTool::Build),
        ))
        .id();
    frame(&mut app);
    assert_eq!(
        app.world().resource::<ViewState>().path_tool.mode,
        PathTool::Build
    );
    app.world_mut().entity_mut(button).insert(Interaction::None);
    app.world_mut().resource_mut::<ViewState>().path_tool.height = park_sim::MAX_PATH_HEIGHT;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::PageUp);
    frame(&mut app);
    assert_eq!(
        app.world().resource::<ViewState>().path_tool.height,
        park_sim::MAX_PATH_HEIGHT
    );
    app.world_mut().resource_mut::<ViewState>().path_tool.height = 0;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::PageDown);
    frame(&mut app);
    assert_eq!(app.world().resource::<ViewState>().path_tool.height, 0);
    aim(&mut app, window, TilePos { x: 14, y: 17 }, 0);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    press_left(&mut app);
    frame(&mut app);
    assert_eq!(
        app.world().resource::<ViewState>().path_tool.mode,
        PathTool::Inspect
    );
    assert!(app.world().resource::<Simulation>().0.objects().is_empty());
    assert!(app.world().resource::<ViewState>().selected.is_none());
}

#[test]
fn same_update_build_and_demolition_are_included_in_save_and_load_cancels_edits() {
    let directory = InputSaveDirectory::new();
    let park = Park::new(MAP_SIZE, MAP_SIZE);
    let (mut app, window) = persistence_input_app(&directory, park, build_view(0));
    let pos = TilePos { x: 14, y: 17 };
    aim(&mut app, window, pos, 8);
    press_left(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let built = app.world().resource::<Simulation>().0.clone();
    assert_eq!(built.objects().len(), 1);
    assert_eq!(directory.store().load().unwrap(), built);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    app.world_mut().resource_mut::<ViewState>().path_tool.mode = PathTool::Demolish;
    aim(&mut app, window, pos, 8);
    press_left(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let removed = app.world().resource::<Simulation>().0.clone();
    assert!(removed.objects().is_empty());
    assert_eq!(directory.store().load().unwrap(), removed);
    assert_eq!(directory.store().load_backup().unwrap(), built);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    app.world_mut().resource_mut::<ViewState>().path_tool.mode = PathTool::Build;
    aim(&mut app, window, TilePos { x: 15, y: 17 }, 8);
    press_left(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::ShiftLeft, KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, built);
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.path_tool.mode, PathTool::Inspect);
    assert!(view.path_tool.preview.is_none() && view.selected.is_none());
    assert_eq!(view.speed, Speed::Paused);
}

#[test]
fn failed_load_suppresses_build_legacy_delete_restore_terrain_and_train_edits() {
    let directory = InputSaveDirectory::new(); // no saved file
    let park = train_park();
    let mut view = build_view(0);
    view.selected = Some(SurfaceId::Object(park.guests()[0].current.unwrap()));
    let (mut app, window) = persistence_input_app(&directory, park.clone(), view);
    app.add_systems(
        Update,
        (
            apply_terrain_profile,
            apply_path_actions,
            apply_train_actions,
        )
            .chain()
            .after(keyboard_input)
            .before(pointer_input),
    );
    aim(&mut app, window, TilePos { x: 14, y: 17 }, 8);
    press_left(&mut app);
    persistence_key_frame(
        &mut app,
        &[
            KeyCode::F9,
            KeyCode::Delete,
            KeyCode::KeyR,
            KeyCode::KeyT,
            KeyCode::KeyO,
        ],
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
    let view = app.world().resource::<ViewState>();
    assert!(view.save_notice.starts_with("Load failed"));
    assert!(!view.remove_path_pending && !view.restore_paths_pending && !view.toggle_train_pending);
    assert!(!view.terrain_change_pending);
    assert!(view.path_tool.preview.is_none());
}

#[test]
fn delete_wins_over_restore_and_consumes_a_simultaneous_build_click() {
    let park = crowd_park();
    let removed = park.guests()[0].current.unwrap();
    let mut view = build_view(0);
    view.selected = Some(SurfaceId::Object(removed));
    let (mut app, window) = input_app(park.clone(), view, 1.0);
    app.add_systems(
        Update,
        apply_path_actions
            .after(keyboard_input)
            .before(pointer_input),
    );
    aim(&mut app, window, TilePos { x: 14, y: 17 }, 8);
    press_left(&mut app);
    for key in [KeyCode::Delete, KeyCode::KeyR] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }
    frame(&mut app);
    let edited = &app.world().resource::<Simulation>().0;
    assert!(edited.object(removed).is_none());
    assert_eq!(edited.objects().len(), park.objects().len() - 1);
}

#[test]
fn paused_path_preview_build_and_removal_refresh_meshes_in_the_same_update() {
    let park = train_park();
    let (mut app, window) = input_app(park, build_view(0), 1.0);
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_scene.after(pointer_input));
    let pos = TilePos { x: 13, y: 21 };
    aim(&mut app, window, pos, 8);
    frame(&mut app);
    let counts = |app: &App| {
        let scene = app.world().resource::<SceneMeshes>();
        let meshes = app.world().resource::<Assets<Mesh>>();
        (
            meshes.get(&scene.path_preview).unwrap().count_vertices(),
            app.world()
                .resource::<static_landscape::StaticLandscape>()
                .report()["geometry_builds"]
                .as_u64()
                .unwrap(),
        )
    };
    assert!(counts(&app).0 > 0);
    assert_eq!(counts(&app).1, 1);
    let ticks = app.world().resource::<Simulation>().0.ticks();
    press_left(&mut app);
    frame(&mut app);
    assert_eq!(counts(&app), (0, 2));
    let id = app.world().resource::<ViewState>().selected.unwrap();
    let verify_landscape = |app: &App| {
        let park = &app.world().resource::<Simulation>().0;
        let view = app.world().resource::<ViewState>();
        let handle = app
            .world()
            .resource::<static_landscape::StaticLandscape>()
            .mesh_handles()
            .next()
            .unwrap();
        assert_eq!(
            mesh_vertex_positions(app.world().resource::<Assets<Mesh>>().get(handle).unwrap()),
            mesh_vertex_positions(&world::build_scene(
                park,
                view.rotation,
                view.grid,
                view.view_mode
            )),
        );
        assert_eq!(park.ticks(), ticks);
    };
    verify_landscape(&app);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    app.world_mut().resource_mut::<ViewState>().path_tool.mode = PathTool::Demolish;
    aim(&mut app, window, pos, 8);
    frame(&mut app);
    assert!(counts(&app).0 > 0);
    press_left(&mut app);
    frame(&mut app);
    assert_eq!(counts(&app), (0, 3));
    assert!(
        app.world()
            .resource::<Simulation>()
            .0
            .surface_position(id)
            .is_none()
    );
    verify_landscape(&app);
}

#[test]
fn occluded_plane_and_hidden_selected_path_cannot_be_edited() {
    let pos = TilePos { x: 14, y: 17 };
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.place_path(TilePos { x: 15, y: 18 }, 10).unwrap();
    let (mut app, window) = input_app(park.clone(), build_view(0), 1.0);
    aim(&mut app, window, pos, 8);
    press_left(&mut app);
    frame(&mut app);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert!(
        app.world()
            .resource::<ViewState>()
            .path_notice
            .as_ref()
            .unwrap()
            .contains("hidden")
    );
    assert!(
        app.world()
            .resource::<ViewState>()
            .path_tool
            .preview
            .is_none()
    );
    let hidden = park.place_path(pos, 8).unwrap();
    let mut view = build_view(0);
    view.selected = Some(SurfaceId::Object(hidden));
    crate::path_tools::remove_selected(&mut park, &mut view);
    assert!(park.object(hidden).is_some());
}

#[test]
fn keyboard_help_close_cannot_build_or_demolish_behind_the_overlay() {
    for mode in [PathTool::Build, PathTool::Demolish] {
        for key in [KeyCode::KeyH, KeyCode::F1] {
            let mut park = Park::new(MAP_SIZE, MAP_SIZE);
            let pos = TilePos { x: 14, y: 17 };
            if mode == PathTool::Demolish {
                park.place_path(pos, 8).unwrap();
            }
            let mut view = build_view(0);
            view.help = true;
            view.path_tool.mode = mode;
            let (mut app, window) = input_app(park.clone(), view, 1.0);
            aim(&mut app, window, pos, 8);
            press_left(&mut app);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            frame(&mut app);
            assert!(!app.world().resource::<ViewState>().help);
            assert!(app.world().resource::<ViewState>().ui_pointer_consumed);
            assert_eq!(app.world().resource::<Simulation>().0, park);
        }
    }
}
