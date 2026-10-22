//! Guest inspection uses real input/action systems with no native event backend.
use super::*;
use park_sim::GuestId;

fn guest_app(directory: &InputSaveDirectory, park: Park, view: ViewState) -> (App, Entity) {
    let (mut app, window) = persistence_input_app(directory, park, view);
    app.add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                apply_terrain_profile,
                apply_path_actions,
                apply_train_actions,
                apply_price_actions,
            )
                .chain()
                .after(keyboard_input)
                .before(pointer_input),
        )
        .add_systems(
            Update,
            (ui::refresh, ui::guests::refresh)
                .chain()
                .after(persistence::apply_save_action),
        );
    (app, window)
}

fn text_for(app: &mut App, predicate: impl Fn(&ui::guests::GuestLabel) -> bool) -> String {
    let world = app.world_mut();
    let mut labels = world.query::<(&ui::guests::GuestLabel, &Text)>();
    labels
        .iter(world)
        .find(|(kind, _)| predicate(kind))
        .unwrap()
        .1
        .0
        .clone()
}

fn click_action(app: &mut App, action: ui::Action) {
    let entity = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, action))
        .id();
    frame(app);
    app.world_mut().despawn(entity);
}

#[test]
fn modal_keys_and_buttons_consume_open_close_frames_before_edits_and_world_clicks() {
    for keyboard in [true, false] {
        for reverse_buttons in [false, true] {
            let directory = InputSaveDirectory::new();
            let original = train_park();
            let (mut app, window) = guest_app(&directory, original.clone(), ViewState::default());
            frame(&mut app);
            let point = world::path_build_center(
                TilePos { x: 15, y: 16 },
                &original,
                0,
                8,
                park_sim::PathSlope::Flat,
            )
            .unwrap();
            let cursor = cursor_for_world(&app, window, point);
            set_cursor(&mut app, window, Some(cursor));
            for opening in [true, false] {
                let mut actions = vec![
                    ui::Action::TicketPrice(1),
                    ui::Action::Save,
                    ui::Action::ToggleTrain,
                    ui::Action::Terrain,
                    ui::Action::RotateRight,
                ];
                if !keyboard {
                    actions.push(if opening {
                        ui::Action::Guests
                    } else {
                        ui::Action::CloseGuests
                    });
                }
                if reverse_buttons {
                    actions.reverse();
                }
                let entities: Vec<_> = actions
                    .into_iter()
                    .map(|action| {
                        app.world_mut()
                            .spawn((Button, Interaction::Pressed, action))
                            .id()
                    })
                    .collect();
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .press(MouseButton::Left);
                let mut keys = vec![
                    KeyCode::F7,
                    KeyCode::F5,
                    KeyCode::KeyO,
                    KeyCode::KeyT,
                    KeyCode::KeyE,
                    KeyCode::KeyF,
                    KeyCode::Delete,
                ];
                if keyboard {
                    keys.push(if opening {
                        KeyCode::KeyL
                    } else {
                        KeyCode::Escape
                    });
                }
                persistence_key_frame(&mut app, &keys);
                assert_eq!(app.world().resource::<Simulation>().0, original);
                let view = app.world().resource::<ViewState>();
                assert_eq!(view.guests_open, opening);
                assert_eq!(view.rotation, 0);
                assert_eq!(view.pan, Vec2::ZERO);
                assert!(view.selected.is_none() && view.hovered.is_none());
                assert!(view.save_pending.is_none());
                assert!(!directory.store().save_path().exists());
                for entity in entities {
                    app.world_mut().despawn(entity);
                }
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .reset_all();
                frame(&mut app);
                assert_eq!(
                    app.world().resource::<Simulation>().0,
                    original,
                    "no deferred edit"
                );
            }
        }
    }
}

#[test]
fn guest_modal_browses_without_panning_or_building_and_allows_pause() {
    let directory = InputSaveDirectory::new();
    let original = train_park();
    let (mut app, _) = guest_app(&directory, original.clone(), ViewState::default());
    persistence_key_frame(&mut app, &[KeyCode::KeyL]);
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(1))
    );
    persistence_key_frame(
        &mut app,
        &[
            KeyCode::ArrowDown,
            KeyCode::PageDown,
            KeyCode::Space,
            KeyCode::KeyP,
            KeyCode::F7,
            KeyCode::F5,
            KeyCode::F9,
            KeyCode::KeyO,
            KeyCode::KeyF,
            KeyCode::KeyE,
            KeyCode::Digit3,
            KeyCode::Delete,
        ],
    );
    let view = app.world().resource::<ViewState>();
    assert_eq!(view.selected_guest, Some(GuestId(10)));
    assert_eq!(view.guest_page, 1);
    assert_eq!(view.speed, Speed::Paused);
    assert_eq!(view.pan, Vec2::ZERO);
    assert_eq!(view.zoom, 1.0);
    assert_eq!(view.path_tool.height, 8);
    assert_eq!(view.path_tool.mode, path_tools::PathTool::Inspect);
    assert_eq!(app.world().resource::<Simulation>().0, original);
    assert!(!directory.store().save_path().exists());
    persistence_key_frame(&mut app, &[KeyCode::KeyH]);
    assert!(app.world().resource::<ViewState>().help);
    assert!(!app.world().resource::<ViewState>().guests_open);
    persistence_key_frame(&mut app, &[KeyCode::KeyL]);
    assert!(!app.world().resource::<ViewState>().help);
    assert!(app.world().resource::<ViewState>().guests_open);
    persistence_key_frame(&mut app, &[KeyCode::KeyL]);
    assert!(!app.world().resource::<ViewState>().guests_open);
    persistence_key_frame(&mut app, &[KeyCode::F7]);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .ride()
            .unwrap()
            .ticket_price_cents,
        50,
        "ordinary controls resume the following frame"
    );
}

#[test]
fn list_is_sorted_by_identity_wraps_and_ignores_blank_rows() {
    let directory = InputSaveDirectory::new();
    let original = train_park();
    let mut json: serde_json::Value =
        serde_json::from_slice(&original.to_save_bytes().unwrap()).unwrap();
    json["park"]["guests"].as_array_mut().unwrap().reverse();
    let reordered = Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).unwrap();
    assert_eq!(reordered.guests()[0].id, GuestId(12));
    let (mut app, _) = guest_app(
        &directory,
        reordered,
        ViewState {
            guests_open: true,
            selected_guest: Some(GuestId(12)),
            ..default()
        },
    );
    frame(&mut app);
    assert_eq!(app.world().resource::<ViewState>().guest_page, 1);
    assert_eq!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Row(0)
        )),
        "Guest #9\nMood 70"
    );
    persistence_key_frame(&mut app, &[KeyCode::ArrowDown]);
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(1))
    );
    persistence_key_frame(&mut app, &[KeyCode::ArrowUp]);
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(12))
    );
    click_action(&mut app, ui::Action::SelectGuest(None));
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(12))
    );
    click_action(&mut app, ui::Action::GuestPage(1));
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(4))
    );
    click_action(&mut app, ui::Action::SelectGuest(Some(GuestId(7))));
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(7))
    );
    assert!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Details
        ))
        .contains("Comfortable ticket:")
    );
    // Replacing the same-tick simulation with a different storage order must
    // preserve the selected identity and refresh its live detail text.
    app.world_mut().resource_mut::<Simulation>().0 = original;
    frame(&mut app);
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(7))
    );
    assert_eq!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Title
        )),
        "Guest #7"
    );
}

#[test]
fn capture_blocks_guest_input_but_programmatic_inspection_and_diagnostics_are_read_only() {
    let directory = InputSaveDirectory::new();
    let original = train_park();
    let (mut app, _) = guest_app(
        &directory,
        original.clone(),
        ViewState {
            capture_active: true,
            guests_open: true,
            selected_guest: Some(GuestId(5)),
            ..default()
        },
    );
    click_action(&mut app, ui::Action::CloseGuests);
    persistence_key_frame(
        &mut app,
        &[
            KeyCode::KeyL,
            KeyCode::Escape,
            KeyCode::ArrowDown,
            KeyCode::Space,
        ],
    );
    assert!(app.world().resource::<ViewState>().guests_open);
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(5))
    );
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Normal);
    assert_eq!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Title
        )),
        "Guest #5"
    );
    {
        let mut view = app.world_mut().resource_mut::<ViewState>();
        view.capture_active = false;
        view.diagnostic_case = Some(park_sim::diagnostics::DiagnosticCase::SaveScale);
    }
    persistence_key_frame(&mut app, &[KeyCode::ArrowDown]);
    assert_eq!(
        app.world().resource::<ViewState>().selected_guest,
        Some(GuestId(6))
    );
    persistence_key_frame(&mut app, &[KeyCode::Escape]);
    persistence_key_frame(&mut app, &[KeyCode::F7, KeyCode::F5]);
    assert_eq!(app.world().resource::<Simulation>().0, original);
    assert!(!directory.store().save_path().exists());
}

#[test]
fn successful_primary_and_backup_load_reset_guest_modal_state_and_pending_browse() {
    let directory = InputSaveDirectory::new();
    let initial = train_park();
    directory.store().save(&initial).unwrap();
    let mut advanced = initial.clone();
    advanced.advance(Speed::Fastest);
    directory.store().save(&advanced).unwrap();
    for action in [SaveAction::Load, SaveAction::Backup] {
        let mut park = advanced.clone();
        let mut view = ViewState {
            guests_open: true,
            selected_guest: Some(GuestId(12)),
            guest_page: 1,
            guest_browse_pending: 3,
            guest_page_pending: -1,
            modal_transition_pending: true,
            modal_input_blocked: true,
            ..default()
        };
        assert!(persistence::perform_action(
            &directory.session(),
            &mut park,
            &mut view,
            action
        ));
        assert_eq!(
            park,
            if action == SaveAction::Load {
                advanced.clone()
            } else {
                initial.clone()
            }
        );
        assert!(
            !view.guests_open
                && !view.help
                && !view.modal_input_blocked
                && !view.modal_transition_pending
        );
        assert!(view.selected_guest.is_none());
        assert_eq!(
            (
                view.guest_page,
                view.guest_browse_pending,
                view.guest_page_pending
            ),
            (0, 0, 0)
        );
        assert_eq!(view.speed, Speed::Paused);
    }
}

#[test]
fn live_detail_distinguishes_preference_from_wallet_and_unrated_from_zero_stars() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    let id = park.guests()[0].id;
    let preference = park.guests()[0].preferences.max_ticket_price_cents;
    park.set_ride_ticket_price(park.ride().unwrap().id, preference + 50)
        .unwrap();
    let ticks = park.ticks();
    let (mut app, _) = guest_app(
        &directory,
        park,
        ViewState {
            guests_open: true,
            selected_guest: Some(id),
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    assert!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Demand
        ))
        .contains("Can pay: yes / Willing to pay: no\nInterested now: no")
    );
    assert!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Summary
        ))
        .contains("Unrated from 0 opinions")
    );
    assert!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Thought
        ))
        .contains("Exploring the park")
    );
    {
        let mut sim = app.world_mut().resource_mut::<Simulation>();
        let ride = sim.0.ride().unwrap().id;
        sim.0.set_ride_ticket_price(ride, preference).unwrap();
    }
    frame(&mut app);
    assert!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Demand
        ))
        .contains("Can pay: yes / Willing to pay: yes\nInterested now: yes")
    );
    assert_eq!(app.world().resource::<Simulation>().0.ticks(), ticks);
}

#[test]
fn empty_guest_panel_has_no_stale_selection_and_fixed_bounds_fit_minimum_window() {
    let directory = InputSaveDirectory::new();
    let (mut app, window) = guest_app(
        &directory,
        Park::new(32, 32),
        ViewState {
            guests_open: true,
            selected_guest: Some(GuestId(u64::MAX)),
            guest_page: usize::MAX,
            guest_browse_pending: i32::MAX,
            guest_page_pending: i32::MIN,
            ..default()
        },
    );
    *app.world_mut().get_mut::<Window>(window).unwrap() = test_window(1024, 680, 2.0);
    frame(&mut app);
    let view = app.world().resource::<ViewState>();
    assert!(view.selected_guest.is_none());
    assert_eq!(view.guest_page, 0);
    assert_eq!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Title
        )),
        "No guests in this park"
    );
    assert_eq!(
        text_for(&mut app, |kind| matches!(
            kind,
            ui::guests::GuestLabel::Details
        )),
        ""
    );
    let window = app.world().get::<Window>(window).unwrap();
    assert!(ui::guests::PANEL_WIDTH <= window.width());
    assert!(ui::guests::PANEL_HEIGHT <= window.height() - HEADER - FOOTER);
    let world = app.world_mut();
    let mut nodes = world.query::<&Node>();
    assert!(
        nodes
            .iter(world)
            .any(|node| node.width == px(760) && node.height == px(520))
    );
    let mut actions = world.query::<&ui::Action>();
    assert_eq!(
        actions
            .iter(world)
            .filter(|action| matches!(action, ui::Action::SelectGuest(_)))
            .count(),
        8
    );
}
