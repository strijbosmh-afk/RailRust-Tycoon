//! Track draft, UI transaction ordering and persistence through production inputs.
use super::*;
use crate::track_tools::{self, TrackCommand, TrackEditor};
use park_sim::{TrackBuildKind as Kind, TrackDirection as Direction};

fn view(rotation: u8) -> ViewState {
    ViewState {
        rotation,
        speed: Speed::Paused,
        track_editor: TrackEditor {
            active: true,
            ..default()
        },
        ..default()
    }
}

fn add_commands(app: &mut App) {
    app.add_systems(
        Update,
        track_tools::apply_commands
            .after(keyboard_input)
            .before(pointer_input),
    );
}

fn press_action(app: &mut App, command: TrackCommand) {
    let entity = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::Track(command)))
        .id();
    frame(app);
    app.world_mut().despawn(entity);
}

fn aim(app: &mut App, window: Entity, pos: TilePos, height: u8) {
    let view = app.world().resource::<ViewState>();
    let park = &app.world().resource::<Simulation>().0;
    let editor = &view.track_editor;
    let spec = editor.kind.spec(pos, height, editor.direction);
    let midpoint_shift =
        (f32::from(spec.exit.height) - f32::from(spec.entry.height)) * 0.5 * world::HEIGHT_STEP;
    let point =
        track_tools::build_center(park, pos, height, view.rotation) + Vec2::Y * midpoint_shift;
    let cursor = cursor_for_world(app, window, point);
    assert!(is_map_position(
        cursor,
        app.world().get::<Window>(window).unwrap(),
        false
    ));
    set_cursor(app, window, Some(cursor));
}

fn complete_draft(park: &mut Park) {
    for (x, y, kind, direction) in [
        (20, 20, Kind::CurveRight, Direction::North),
        (21, 20, Kind::Station, Direction::East),
        (22, 20, Kind::CurveRight, Direction::East),
        (22, 21, Kind::Straight, Direction::South),
        (22, 22, Kind::CurveRight, Direction::South),
        (21, 22, Kind::Straight, Direction::West),
        (20, 22, Kind::CurveRight, Direction::West),
        (20, 21, Kind::Straight, Direction::North),
    ] {
        park.set_track_draft_piece(kind.spec(TilePos { x, y }, 8, direction))
            .unwrap();
    }
}

#[test]
fn track_pointer_rotation_and_dpi_preserve_live_world_and_never_paint_while_held() {
    for rotation in 0..4 {
        for dpi in [1.0, 1.5, 2.0] {
            let park = train_park();
            let (mut app, window) = input_app(park.clone(), view(rotation), dpi);
            let pos = TilePos { x: 14, y: 17 };
            aim(&mut app, window, pos, 8);
            frame(&mut app);
            assert_eq!(app.world().resource::<Simulation>().0, park);
            let preview = app
                .world()
                .resource::<ViewState>()
                .track_editor
                .preview
                .unwrap();
            assert!(preview.valid);
            assert_eq!(preview.spec.pos, pos);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            frame(&mut app);
            assert_eq!(
                app.world().resource::<Simulation>().0.track_draft(),
                &[preview.spec]
            );
            let mut comparable = app.world().resource::<Simulation>().0.clone();
            comparable.clear_track_draft();
            assert_eq!(comparable, park);
            aim(&mut app, window, TilePos { x: 15, y: 17 }, 8);
            frame(&mut app);
            assert_eq!(
                app.world().resource::<Simulation>().0.track_draft().len(),
                1
            );
        }
    }
}

#[test]
fn track_apply_refuses_open_ride_then_replaces_closed_route_without_charging() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    complete_draft(&mut park);
    let before = park.clone();
    let (mut app, _) = persistence_input_app(&directory, park, view(0));
    add_commands(&mut app);
    press_action(&mut app, TrackCommand::Apply);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(
        app.world()
            .resource::<ViewState>()
            .track_editor
            .notice
            .starts_with("Apply refused")
    );
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .set_train_open(false);
    let closed = app.world().resource::<Simulation>().0.clone();
    press_action(&mut app, TrackCommand::Apply);
    let applied = &app.world().resource::<Simulation>().0;
    assert_eq!(applied.circuit().unwrap().pieces.len(), 8);
    assert_eq!(applied.ride(), closed.ride());
    assert_eq!(applied.cash_cents(), closed.cash_cents());
    assert_eq!(applied.ticks(), 0);
    assert!(app.world().resource::<ViewState>().scene_dirty);
    let applied = applied.clone();
    press_action(&mut app, TrackCommand::Apply);
    assert_eq!(app.world().resource::<Simulation>().0, applied);
}

#[test]
fn partial_track_click_is_saved_and_load_cancels_same_frame_apply() {
    let directory = InputSaveDirectory::new();
    let (mut app, window) = persistence_input_app(&directory, train_park(), view(0));
    add_commands(&mut app);
    aim(&mut app, window, TilePos { x: 14, y: 17 }, 8);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let saved = app.world().resource::<Simulation>().0.clone();
    assert_eq!(saved.track_draft().len(), 1);
    assert_eq!(directory.store().load().unwrap(), saved);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    {
        let mut sim = app.world_mut().resource_mut::<Simulation>();
        sim.0.clear_track_draft();
        complete_draft(&mut sim.0);
        sim.0.set_train_open(false);
    }
    app.world_mut().spawn((
        Button,
        Interaction::Pressed,
        ui::Action::Track(TrackCommand::Apply),
    ));
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    assert_eq!(app.world().resource::<Simulation>().0, saved);
    assert!(!app.world().resource::<ViewState>().track_editor.active);
    assert!(
        app.world()
            .resource::<ViewState>()
            .track_editor
            .pending
            .is_empty()
    );
}

#[test]
fn track_modal_transitions_and_done_discard_pending_commands_without_losing_the_plan() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    complete_draft(&mut park);
    let before = park.clone();
    let (mut app, _) = persistence_input_app(&directory, park, view(0));
    add_commands(&mut app);
    let clear = app
        .world_mut()
        .spawn((
            Button,
            Interaction::Pressed,
            ui::Action::Track(TrackCommand::Clear),
        ))
        .id();
    persistence_key_frame(&mut app, &[KeyCode::KeyH]);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(app.world().resource::<ViewState>().help);
    app.world_mut().despawn(clear);
    persistence_key_frame(&mut app, &[KeyCode::Escape]);
    frame(&mut app);
    app.world_mut()
        .resource_mut::<ViewState>()
        .track_editor
        .pending = vec![TrackCommand::Clear, TrackCommand::Apply, TrackCommand::Done];
    frame(&mut app);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(!app.world().resource::<ViewState>().track_editor.active);
    persistence_key_frame(&mut app, &[KeyCode::KeyZ, KeyCode::Delete, KeyCode::F5]);
    assert!(app.world().resource::<ViewState>().track_editor.active);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert!(!directory.store().save_path().exists());
}

#[test]
fn track_tools_turn_clamp_replace_erase_and_clear_only_saved_draft_pieces() {
    let directory = InputSaveDirectory::new();
    let park = train_park();
    let (mut app, _) = persistence_input_app(&directory, park.clone(), view(0));
    add_commands(&mut app);
    app.add_systems(
        Update,
        apply_path_actions
            .after(keyboard_input)
            .before(pointer_input),
    );
    app.world_mut().resource_mut::<ViewState>().selected =
        park.guests()[0].current.map(SurfaceId::Object);
    for action in [ui::Action::RemovePath, ui::Action::RestorePaths] {
        let button = app
            .world_mut()
            .spawn((Button, Interaction::Pressed, action))
            .id();
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        let state = app.world().resource::<ViewState>();
        assert!(!state.remove_path_pending && !state.restore_paths_pending);
        app.world_mut().despawn(button);
    }
    press_action(&mut app, TrackCommand::Piece(Kind::CurveLeft));
    persistence_key_frame(&mut app, &[KeyCode::KeyC]);
    assert_eq!(
        app.world().resource::<ViewState>().track_editor.direction,
        Direction::South
    );
    app.world_mut()
        .resource_mut::<ViewState>()
        .track_editor
        .height = 30;
    persistence_key_frame(&mut app, &[KeyCode::PageUp]);
    assert_eq!(app.world().resource::<ViewState>().track_editor.height, 30);
    app.world_mut()
        .resource_mut::<ViewState>()
        .track_editor
        .height = 0;
    persistence_key_frame(&mut app, &[KeyCode::PageDown]);
    assert_eq!(app.world().resource::<ViewState>().track_editor.height, 0);
    let mut state = view(0);
    let mut edited = park.clone();
    let point = track_tools::build_center(&edited, TilePos { x: 14, y: 17 }, 8, 0);
    track_tools::point_at(&mut edited, &mut state, point, true);
    state.track_editor.kind = Kind::CurveRight;
    track_tools::point_at(&mut edited, &mut state, point, true);
    assert_eq!(edited.track_draft().len(), 1);
    assert_eq!(edited.track_draft()[0].kind, park_sim::TrackKind::Curve);
    state.track_editor.erase = true;
    track_tools::point_at(&mut edited, &mut state, point, true);
    assert_eq!(edited, park);
    complete_draft(&mut app.world_mut().resource_mut::<Simulation>().0);
    press_action(&mut app, TrackCommand::Clear);
    assert_eq!(app.world().resource::<Simulation>().0, park);
}

#[test]
fn selecting_another_ride_exits_track_tools_without_discarding_the_draft() {
    let mut park = train_park();
    let coaster = park.ride().unwrap().id;
    let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    park.set_track_draft_piece(Kind::Station.spec(TilePos { x: 14, y: 17 }, 8, Direction::East))
        .unwrap();
    let before = park.clone();
    let mut view = view(0);
    view.selected_ride = Some(coaster);
    view.select_ride(carousel);
    assert!(!view.track_editor.active);
    assert_eq!(view.selected_ride, Some(carousel));
    assert_eq!(park, before);
}

#[test]
fn graded_piece_buttons_and_keyboard_height_limits_produce_valid_ports() {
    let directory = InputSaveDirectory::new();
    let (mut app, _) = persistence_input_app(&directory, train_park(), view(0));
    add_commands(&mut app);
    for (kind, low, high, grade) in [
        (Kind::Lift, 0, 29, 1_i16),
        (Kind::Drop, 1, 30, -1),
        (Kind::SteepDrop, 2, 30, -2),
        (Kind::Brake, 0, 30, 0),
    ] {
        app.world_mut()
            .resource_mut::<ViewState>()
            .track_editor
            .height = 0;
        press_action(&mut app, TrackCommand::Piece(kind));
        assert_eq!(app.world().resource::<ViewState>().track_editor.height, low);
        persistence_key_frame(&mut app, &[KeyCode::PageDown]);
        assert_eq!(app.world().resource::<ViewState>().track_editor.height, low);
        app.world_mut()
            .resource_mut::<ViewState>()
            .track_editor
            .height = 30;
        press_action(&mut app, TrackCommand::Piece(kind));
        assert_eq!(
            app.world().resource::<ViewState>().track_editor.height,
            high
        );
        persistence_key_frame(&mut app, &[KeyCode::PageUp]);
        let editor = &app.world().resource::<ViewState>().track_editor;
        assert_eq!(editor.height, high);
        let spec = editor
            .kind
            .spec(TilePos { x: 14, y: 17 }, editor.height, editor.direction);
        assert_eq!(
            i16::from(spec.exit.height) - i16::from(spec.entry.height),
            grade
        );
        assert!(
            app.world()
                .resource::<Simulation>()
                .0
                .validate_track_draft_piece(spec)
                .is_ok()
        );
    }
    assert!(
        app.world()
            .resource::<Simulation>()
            .0
            .track_draft()
            .is_empty()
    );
}

#[test]
fn graded_pointer_build_and_save_preserve_each_selected_kind_and_world_heading() {
    for rotation in 0..4 {
        for kind in [Kind::Lift, Kind::Drop, Kind::SteepDrop, Kind::Brake] {
            let directory = InputSaveDirectory::new();
            let original = train_park();
            let (mut app, window) =
                persistence_input_app(&directory, original.clone(), view(rotation));
            add_commands(&mut app);
            press_action(&mut app, TrackCommand::Piece(kind));
            persistence_key_frame(&mut app, &[KeyCode::KeyC]);
            let pos = TilePos { x: 14, y: 17 };
            aim(&mut app, window, pos, 8);
            frame(&mut app);
            let preview = app
                .world()
                .resource::<ViewState>()
                .track_editor
                .preview
                .unwrap();
            assert!(preview.valid);
            assert_eq!(preview.spec, kind.spec(pos, 8, Direction::South));
            assert_eq!(app.world().resource::<Simulation>().0, original);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            persistence_key_frame(&mut app, &[KeyCode::F5]);
            let saved = &app.world().resource::<Simulation>().0;
            assert_eq!(saved.track_draft(), &[preview.spec]);
            assert_eq!(directory.store().load().unwrap(), *saved);
            assert_eq!(saved.circuit(), original.circuit());
            assert_eq!(saved.train(), original.train());
        }
    }
}
