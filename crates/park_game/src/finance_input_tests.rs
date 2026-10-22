//! Exercise the read-only finance window through production input ordering.
use super::*;
use park_sim::{FINANCE_LEDGER_CAPACITY, PathDirection, SERVICE_UPKEEP_TICKS, ServiceKind};

fn finance_park(entries: usize) -> Park {
    let mut park = Park::new(32, 32);
    park.place_service(
        TilePos { x: 4, y: 5 },
        0,
        PathDirection::North,
        ServiceKind::Toilet,
    )
    .unwrap();
    for _ in 0..entries * usize::from(SERVICE_UPKEEP_TICKS) {
        park.advance(Speed::Normal);
    }
    park
}

fn finance_app(directory: &InputSaveDirectory, park: Park, view: ViewState) -> (App, Entity) {
    let (mut app, window) = persistence_input_app(directory, park, view);
    app.add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                apply_terrain_profile,
                apply_path_actions,
                apply_train_actions,
                apply_price_actions,
                apply_service_actions,
            )
                .chain()
                .after(keyboard_input)
                .before(pointer_input),
        )
        .add_systems(
            Update,
            (
                ui::refresh,
                ui::guests::refresh,
                ui::services::refresh,
                ui::finance::refresh,
            )
                .chain()
                .after(persistence::apply_save_action),
        );
    (app, window)
}

fn label(app: &mut App, matches: impl Fn(&ui::finance::FinanceLabel) -> bool) -> String {
    let world = app.world_mut();
    let mut labels = world.query::<(&ui::finance::FinanceLabel, &Text)>();
    labels
        .iter(world)
        .find(|(kind, _)| matches(kind))
        .unwrap()
        .1
        .0
        .clone()
}

fn click(app: &mut App, action: ui::Action) {
    let entity = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, action))
        .id();
    frame(app);
    app.world_mut().despawn(entity);
}

#[test]
fn finance_open_and_close_consume_conflicting_keys_buttons_and_world_clicks() {
    for keyboard in [true, false] {
        for reverse in [true, false] {
            let directory = InputSaveDirectory::new();
            let park = train_park();
            let (mut app, window) = finance_app(&directory, park.clone(), ViewState::default());
            frame(&mut app);
            let point = world::path_build_center(
                TilePos { x: 15, y: 16 },
                &park,
                0,
                8,
                park_sim::PathSlope::Flat,
            )
            .unwrap();
            let cursor = cursor_for_world(&app, window, point);
            set_cursor(&mut app, window, Some(cursor));
            for opening in [true, false] {
                let mut actions = vec![
                    ui::Action::Save,
                    ui::Action::TicketPrice(1),
                    ui::Action::ToggleTrain,
                    ui::Action::RemovePath,
                    ui::Action::RotateRight,
                    ui::Action::Terrain,
                ];
                if !keyboard {
                    actions.push(if opening {
                        ui::Action::Finance
                    } else {
                        ui::Action::CloseFinance
                    });
                }
                if reverse {
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
                    KeyCode::F9,
                    KeyCode::KeyO,
                    KeyCode::KeyT,
                    KeyCode::KeyE,
                    KeyCode::KeyF,
                    KeyCode::Delete,
                    KeyCode::PageUp,
                    KeyCode::KeyP,
                ];
                if keyboard {
                    keys.push(if opening {
                        KeyCode::KeyN
                    } else {
                        KeyCode::Escape
                    });
                }
                persistence_key_frame(&mut app, &keys);
                let view = app.world().resource::<ViewState>();
                assert_eq!(view.finance_open, opening);
                assert!(view.modal_input_blocked && view.ui_pointer_consumed);
                assert_eq!(view.rotation, 0);
                assert_eq!(view.pan, Vec2::ZERO);
                assert_eq!(view.path_tool.height, 8);
                assert!(
                    view.selected.is_none()
                        && view.hovered.is_none()
                        && view.save_pending.is_none()
                );
                assert_eq!(app.world().resource::<Simulation>().0, park);
                for entity in entities {
                    app.world_mut().despawn(entity);
                }
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .reset_all();
                frame(&mut app);
                assert_eq!(
                    app.world().resource::<Simulation>().0,
                    park,
                    "no deferred world command"
                );
                assert!(!directory.store().save_path().exists());
            }
        }
    }
}

#[test]
fn finance_paging_is_newest_first_bounded_live_and_does_not_change_the_park() {
    let directory = InputSaveDirectory::new();
    let park = finance_park(FINANCE_LEDGER_CAPACITY + 5);
    assert_eq!(park.finance_ledger().entries.len(), 128);
    assert_eq!(park.finance_ledger().archived_entries, 5);
    let (mut app, window) = finance_app(
        &directory,
        park.clone(),
        ViewState {
            finance_open: true,
            ..default()
        },
    );
    *app.world_mut().get_mut::<Window>(window).unwrap() = test_window(1024, 680, 2.0);
    frame(&mut app);
    assert_eq!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::Page
        )),
        "Page 1 / 16"
    );
    let first = label(&mut app, |kind| {
        matches!(kind, ui::finance::FinanceLabel::Row(0))
    });
    assert!(
        first.starts_with("06:39 / ") && first.contains("-$0.10 / Cash $986.70"),
        "{first}"
    );
    assert!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::History
        ))
        .contains("Older 5 transactions: income +$0.00 / costs -$0.50")
    );
    persistence_key_frame(
        &mut app,
        &[
            KeyCode::PageDown,
            KeyCode::ArrowRight,
            KeyCode::KeyC,
            KeyCode::F6,
            KeyCode::KeyO,
        ],
    );
    assert_eq!(app.world().resource::<ViewState>().finance_page, 1);
    assert!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::Row(0)
        ))
        .starts_with("06:15 / ")
    );
    let entity = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::FinancePage(1)))
        .id();
    frame(&mut app);
    frame(&mut app);
    assert_eq!(
        app.world().resource::<ViewState>().finance_page,
        2,
        "held button is consumed once"
    );
    app.world_mut().despawn(entity);
    {
        let mut view = app.world_mut().resource_mut::<ViewState>();
        view.finance_page = usize::MAX;
        view.finance_page_pending = i32::MAX;
    }
    frame(&mut app);
    assert_eq!(app.world().resource::<ViewState>().finance_page, 15);
    assert!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::Row(7)
        ))
        .starts_with("00:18 / ")
    );
    app.world_mut()
        .resource_mut::<ViewState>()
        .finance_page_pending = i32::MIN;
    frame(&mut app);
    assert_eq!(app.world().resource::<ViewState>().finance_page, 0);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    let window = app.world().get::<Window>(window).unwrap();
    assert!(ui::finance::PANEL_WIDTH <= window.width());
    assert!(ui::finance::PANEL_HEIGHT <= window.height() - HEADER - FOOTER);
    let world = app.world_mut();
    let mut rows = world.query::<&ui::finance::FinanceLabel>();
    assert_eq!(
        rows.iter(world)
            .filter(|kind| matches!(kind, ui::finance::FinanceLabel::Row(_)))
            .count(),
        8
    );
    let mut actions = world.query::<&ui::Action>();
    assert!(
        actions
            .iter(world)
            .any(|action| matches!(action, ui::Action::Finance))
    );
}

#[test]
fn finance_is_mutually_exclusive_allows_pause_and_blocks_capture_input() {
    let directory = InputSaveDirectory::new();
    let park = finance_park(1);
    let (mut app, _) = finance_app(&directory, park.clone(), ViewState::default());
    for key in [KeyCode::KeyH, KeyCode::KeyL, KeyCode::KeyM] {
        persistence_key_frame(&mut app, &[key]);
        persistence_key_frame(&mut app, &[KeyCode::KeyN]);
        let view = app.world().resource::<ViewState>();
        assert!(view.finance_open && !view.help && !view.guests_open && !view.services_open);
        persistence_key_frame(&mut app, &[KeyCode::Space]);
        assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
        click(&mut app, ui::Action::Pause);
        assert_eq!(app.world().resource::<ViewState>().speed, Speed::Normal);
    }
    app.world_mut().resource_mut::<ViewState>().capture_active = true;
    click(&mut app, ui::Action::CloseFinance);
    persistence_key_frame(&mut app, &[KeyCode::KeyN, KeyCode::Escape, KeyCode::Space]);
    assert!(app.world().resource::<ViewState>().finance_open);
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Normal);
    assert_eq!(app.world().resource::<Simulation>().0, park);
}

#[test]
fn finance_empty_and_loaded_history_reset_without_saving_transient_state() {
    let directory = InputSaveDirectory::new();
    let empty = Park::new(32, 32);
    directory.store().save(&empty).unwrap();
    let (mut app, _) = finance_app(
        &directory,
        finance_park(17),
        ViewState {
            finance_open: true,
            finance_page: 2,
            ..default()
        },
    );
    frame(&mut app);
    persistence_key_frame(&mut app, &[KeyCode::F9, KeyCode::F5]);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .finance_ledger()
            .entries
            .len(),
        17
    );
    assert_eq!(directory.store().load().unwrap(), empty);
    persistence_key_frame(&mut app, &[KeyCode::KeyN]);
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    let view = app.world().resource::<ViewState>();
    assert!(!view.finance_open && view.finance_page == 0 && view.finance_page_pending == 0);
    assert_eq!(view.speed, Speed::Paused);
    persistence_key_frame(&mut app, &[KeyCode::KeyN]);
    assert_eq!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::Row(0)
        )),
        "No transactions yet."
    );
    assert_eq!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::Row(1)
        )),
        ""
    );
    assert_eq!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::Page
        )),
        "Page 1 / 1"
    );
    assert_eq!(app.world().resource::<Simulation>().0, empty);
}
