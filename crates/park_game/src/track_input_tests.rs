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
fn track_apply_refuses_open_ride_then_charges_exactly_once_for_the_closed_route() {
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
    assert_eq!(applied.cash_cents(), closed.cash_cents() - 19_000);
    let entry = applied.finance_ledger().entries.last().unwrap();
    assert_eq!(entry.kind, park_sim::FinanceKind::TrackConstruction);
    assert_eq!(entry.amount_cents, 19_000);
    assert_eq!(entry.ride, closed.ride().map(|ride| ride.id));
    assert_eq!(applied.ticks(), 0);
    assert!(app.world().resource::<ViewState>().scene_dirty);
    let applied = applied.clone();
    press_action(&mut app, TrackCommand::Apply);
    assert_eq!(app.world().resource::<Simulation>().0, applied);
}

#[test]
fn track_quote_refreshes_after_cash_changes_and_replacement_settles_only_changed_track() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    park.set_train_open(false);
    complete_draft(&mut park);
    let (mut app, _) = persistence_input_app(&directory, park.clone(), view(0));
    add_commands(&mut app);
    app.add_systems(
        Update,
        ui::track::refresh.after(track_tools::apply_commands),
    );
    let quote_label = app
        .world_mut()
        .spawn((ui::track::TrackLabel::Quote, Text::new("")))
        .id();
    frame(&mut app);
    assert_eq!(
        app.world().resource::<Simulation>().0,
        park,
        "showing a quote is read-only"
    );
    assert_eq!(
        app.world().get::<Text>(quote_label).unwrap().0,
        "Buy $190.00 / Refund $0.00\nPay $190.00 / Cash $1000.00\nAfter $810.00"
    );
    let revision = app.world().resource::<ViewState>().track_editor.revision;
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .place_service(
            TilePos { x: 2, y: 2 },
            8,
            park_sim::PathDirection::North,
            park_sim::ServiceKind::Food,
        )
        .unwrap();
    frame(&mut app);
    assert_eq!(
        app.world().resource::<ViewState>().track_editor.revision,
        revision
    );
    assert_eq!(
        app.world().get::<Text>(quote_label).unwrap().0,
        "Buy $190.00 / Refund $0.00\nPay $190.00 / Cash $880.00\nAfter $690.00"
    );
    press_action(&mut app, TrackCommand::Apply);
    {
        let mut sim = app.world_mut().resource_mut::<Simulation>();
        complete_draft(&mut sim.0);
        sim.0
            .set_track_draft_piece(Kind::Brake.spec(TilePos { x: 22, y: 21 }, 8, Direction::South))
            .unwrap();
    }
    let before = app.world().resource::<Simulation>().0.clone();
    frame(&mut app);
    assert_eq!(
        app.world().get::<Text>(quote_label).unwrap().0,
        "Buy $20.00 / Refund $5.00\nPay $15.00 / Cash $690.00\nAfter $675.00"
    );
    assert_eq!(app.world().resource::<Simulation>().0, before);
    press_action(&mut app, TrackCommand::Apply);
    let after = &app.world().resource::<Simulation>().0;
    assert_eq!(after.cash_cents(), 67_500);
    assert_eq!(after.track_economy().construction_cents, 21_000);
    assert_eq!(after.track_economy().refund_cents, 500);
    assert!(
        app.world()
            .resource::<ViewState>()
            .track_editor
            .notice
            .contains("paid $15.00")
    );
}

#[test]
fn unaffordable_track_apply_preserves_draft_cash_history_and_saved_state() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    park.set_train_open(false);
    crate::save_check::construction::exhaust_build_funds(&mut park).unwrap();
    for _ in 0..2 {
        let id = park
            .place_service(
                TilePos { x: 2, y: 2 },
                8,
                park_sim::PathDirection::North,
                park_sim::ServiceKind::Food,
            )
            .unwrap();
        park.remove_object(park.service(id).unwrap().object.unwrap())
            .unwrap();
    }
    assert_eq!(park.cash_cents(), 13_000);
    complete_draft(&mut park);
    assert_eq!(
        track_tools::quote_text(&park),
        "Buy $190.00 / Refund $0.00\nPay $190.00 / Cash $130.00\nShort $60.00 / cannot apply"
    );
    let (mut app, _) = persistence_input_app(&directory, park.clone(), view(0));
    add_commands(&mut app);
    press_action(&mut app, TrackCommand::Apply);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(
        app.world().resource::<ViewState>().track_editor.notice,
        "Apply refused: short $60.00\nCash $130.00. Draft kept."
    );
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    assert_eq!(directory.store().load().unwrap(), park);
}

fn visible_track(park: &Park, paid: bool) -> park_sim::ObjectId {
    park.objects()
        .iter()
        .filter(|object| {
            object.kind == park_sim::ObjectKind::Track
                && (!paid || park.track_demolition_refund_cents(object.id) > 0)
        })
        .find(|object| {
            world::surface_center(park, SurfaceId::Object(object.id), 0).is_some_and(|point| {
                world::pick_surface(point, park, 0, world::ViewMode::Surface)
                    == Some(SurfaceId::Object(object.id))
            })
        })
        .expect("fixture must have visible track")
        .id
}

#[test]
fn selected_track_delete_refunds_paid_basis_once_and_saves_fault_remnants() {
    for paid in [false, true] {
        let directory = InputSaveDirectory::new();
        let mut park = train_park();
        park.set_train_open(false);
        if paid {
            complete_draft(&mut park);
            park.apply_track_draft().unwrap();
        }
        let id = visible_track(&park, paid);
        let refund = park.track_demolition_refund_cents(id);
        assert_eq!(refund > 0, paid);
        assert!(
            ui::track_removal_details(&park, id, true)
                .contains(&format!("Refund {}", ui::money(refund)))
        );
        let old_cash = park.cash_cents();
        let (mut app, _) = persistence_input_app(
            &directory,
            park,
            ViewState {
                selected: Some(SurfaceId::Object(id)),
                speed: Speed::Paused,
                ..default()
            },
        );
        app.add_systems(
            Update,
            apply_path_actions
                .after(keyboard_input)
                .before(pointer_input),
        );
        persistence_key_frame(&mut app, &[KeyCode::Delete, KeyCode::F5]);
        let removed = app.world().resource::<Simulation>().0.clone();
        assert!(removed.object(id).is_none());
        assert_eq!(removed.cash_cents(), old_cash + refund);
        assert_eq!(directory.store().load().unwrap(), removed);
        assert!(
            app.world()
                .resource::<ViewState>()
                .path_notice
                .as_ref()
                .unwrap()
                .contains(&format!("refund {}", ui::money(refund)))
        );
        app.world_mut().resource_mut::<ViewState>().selected = Some(SurfaceId::Object(id));
        persistence_key_frame(&mut app, &[KeyCode::Delete]);
        assert_eq!(
            app.world().resource::<Simulation>().0,
            removed,
            "a stale selection never refunds twice"
        );
        if paid {
            let next = visible_track(&removed, true);
            let remaining_refund = removed.track_demolition_refund_cents(next);
            app.world_mut().resource_mut::<ViewState>().selected = Some(SurfaceId::Object(next));
            persistence_key_frame(&mut app, &[KeyCode::Delete, KeyCode::F5]);
            let after = &app.world().resource::<Simulation>().0;
            assert!(after.object(next).is_none());
            assert_eq!(after.cash_cents(), removed.cash_cents() + remaining_refund);
            assert_eq!(directory.store().load().unwrap(), *after);
        }
    }
}

#[test]
fn selected_occupied_paid_track_cannot_refund_or_remove_passengers() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    crate::save_check::track_construction::install_access(&mut park).unwrap();
    crate::save_check::track_construction::draft(&mut park, true).unwrap();
    park.apply_track_draft().unwrap();
    crate::save_check::track_construction::occupied(&mut park).unwrap();
    let id = visible_track(&park, true);
    assert!(ui::track_removal_details(&park, id, true).contains("Removal blocked"));
    assert!(!ui::track_removal_details(&park, id, true).contains("Refund"));
    let (mut app, _) = persistence_input_app(
        &directory,
        park.clone(),
        ViewState {
            selected: Some(SurfaceId::Object(id)),
            speed: Speed::Paused,
            ..default()
        },
    );
    app.add_systems(
        Update,
        apply_path_actions
            .after(keyboard_input)
            .before(pointer_input),
    );
    persistence_key_frame(&mut app, &[KeyCode::Delete]);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(
        app.world().resource::<ViewState>().selected,
        Some(SurfaceId::Object(id))
    );
    assert!(
        app.world()
            .resource::<ViewState>()
            .path_notice
            .as_ref()
            .unwrap()
            .contains("not removed")
    );
}

#[test]
fn hiding_selected_track_keeps_buried_guidance_and_refund_without_offering_removal() {
    for paid in [false, true] {
        let directory = InputSaveDirectory::new();
        let mut park = train_park();
        park.set_train_open(false);
        let pos = TilePos { x: 22, y: 21 };
        let id = if paid {
            complete_draft(&mut park);
            park.apply_track_draft().unwrap();
            park.objects()
                .iter()
                .find(|object| object.pos == pos && object.kind == ObjectKind::Track)
                .unwrap()
                .id
        } else {
            park.place_object(pos, ObjectKind::Track, park_sim::Axis::Y, 8, 2)
                .unwrap()
        };
        park.set_terrain_height(pos, 16).unwrap();
        assert!(park.is_underground(id));
        let before = park.clone();
        let (mut app, _) = persistence_input_app(
            &directory,
            park,
            ViewState {
                selected: Some(SurfaceId::Object(id)),
                view_mode: world::ViewMode::Underground,
                speed: Speed::Paused,
                ..default()
            },
        );
        app.add_systems(
            Update,
            apply_path_actions
                .after(keyboard_input)
                .before(pointer_input),
        );
        app.add_systems(Update, ui::refresh.after(apply_path_actions));
        let label = app
            .world_mut()
            .spawn((ui::Label::SelectionDetails, Text::new("")))
            .id();
        frame(&mut app);
        let details = &app.world().get::<Text>(label).unwrap().0;
        assert!(details.contains("Underground"));
        assert!(details.contains("Refund"));
        assert!(details.contains("Delete to remove"));
        persistence_key_frame(&mut app, &[KeyCode::KeyU]);
        let details = &app.world().get::<Text>(label).unwrap().0;
        assert!(details.contains("Buried / U to reveal"));
        assert!(details.contains("Refund"));
        assert!(details.contains("reveal first"));
        assert!(!details.contains("Delete to remove"));
        persistence_key_frame(&mut app, &[KeyCode::Delete]);
        assert_eq!(app.world().resource::<Simulation>().0, before);
        assert_eq!(
            app.world().resource::<ViewState>().selected,
            Some(SurfaceId::Object(id))
        );
    }
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
