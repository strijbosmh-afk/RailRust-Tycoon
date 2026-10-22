//! Price commands share the ordinary input and persistence order, including pause.
use super::*;
use park_sim::{MAX_TICKET_PRICE_CENTS, PRICE_STEP_CENTS};

fn economy_app(directory: &InputSaveDirectory, park: Park, view: ViewState) -> App {
    let (mut app, _) = persistence_input_app(directory, park, view);
    app.add_systems(
        Update,
        apply_price_actions
            .after(keyboard_input)
            .before(pointer_input),
    );
    app
}

fn price(app: &App) -> u32 {
    app.world()
        .resource::<Simulation>()
        .0
        .ride()
        .unwrap()
        .ticket_price_cents
}

#[test]
fn price_keys_use_exact_cents_while_paused_and_respect_both_limits() {
    let directory = InputSaveDirectory::new();
    let park = train_park();
    let ticks = park.ticks();
    let wallets: Vec<_> = park
        .guests()
        .iter()
        .map(|guest| guest.wallet_cents)
        .collect();
    let cash = park.cash_cents();
    let mut app = economy_app(
        &directory,
        park,
        ViewState {
            speed: Speed::Paused,
            ..default()
        },
    );
    persistence_key_frame(&mut app, &[KeyCode::F7]);
    assert_eq!(price(&app), PRICE_STEP_CENTS);
    frame(&mut app);
    assert_eq!(price(&app), PRICE_STEP_CENTS, "no held-key repeat");
    persistence_key_frame(&mut app, &[KeyCode::F6]);
    assert_eq!(price(&app), 0);
    persistence_key_frame(&mut app, &[KeyCode::F6]);
    assert_eq!(price(&app), 0);
    assert_eq!(
        app.world().resource::<ViewState>().price_notice.as_deref(),
        Some("Tickets are already free.")
    );
    app.world_mut()
        .resource_mut::<ViewState>()
        .price_delta_pending = i32::MAX;
    frame(&mut app);
    assert_eq!(price(&app), MAX_TICKET_PRICE_CENTS);
    persistence_key_frame(&mut app, &[KeyCode::F7]);
    assert_eq!(price(&app), MAX_TICKET_PRICE_CENTS);
    assert_eq!(
        app.world().resource::<ViewState>().price_notice.as_deref(),
        Some("Ticket limit: $100.00.")
    );
    app.world_mut()
        .resource_mut::<ViewState>()
        .price_delta_pending = i32::MIN;
    frame(&mut app);
    assert_eq!(price(&app), 0);
    {
        let mut sim = app.world_mut().resource_mut::<Simulation>();
        let id = sim.0.ride().unwrap().id;
        sim.0.set_ride_ticket_price(id, 1234).unwrap();
    }
    persistence_key_frame(&mut app, &[KeyCode::F7]);
    assert_eq!(
        price(&app),
        1284,
        "a loaded non-step price keeps its exact cents"
    );
    persistence_key_frame(&mut app, &[KeyCode::F6, KeyCode::F7]);
    assert_eq!(price(&app), 1284, "opposite keys cancel before clamping");
    let sim = &app.world().resource::<Simulation>().0;
    assert_eq!(sim.ticks(), ticks);
    assert_eq!(sim.cash_cents(), cash);
    assert_eq!(
        sim.guests()
            .iter()
            .map(|guest| guest.wallet_cents)
            .collect::<Vec<_>>(),
        wallets
    );
    assert_eq!(app.world().resource::<ViewState>().price_delta_pending, 0);
}

#[test]
fn button_and_keyboard_price_requests_combine_once_without_overflow() {
    let directory = InputSaveDirectory::new();
    let mut app = economy_app(&directory, train_park(), ViewState::default());
    let button = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::TicketPrice(1)))
        .id();
    persistence_key_frame(&mut app, &[KeyCode::F7]);
    assert_eq!(price(&app), 2 * PRICE_STEP_CENTS);
    frame(&mut app);
    assert_eq!(price(&app), 2 * PRICE_STEP_CENTS);
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
    frame(&mut app);
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    persistence_key_frame(&mut app, &[KeyCode::F6]);
    assert_eq!(
        price(&app),
        2 * PRICE_STEP_CENTS,
        "opposite input sources cancel"
    );
    {
        let mut view = app.world_mut().resource_mut::<ViewState>();
        view.price_delta_pending = i32::MAX;
        view.request_price_change(1);
        assert_eq!(view.price_delta_pending, i32::MAX);
    }
    frame(&mut app);
    assert_eq!(price(&app), MAX_TICKET_PRICE_CENTS);
}

#[test]
fn help_capture_and_diagnostics_consume_pending_price_edits_without_mutation() {
    for case in 0..4 {
        let directory = InputSaveDirectory::new();
        let original = train_park();
        let view = ViewState {
            help: case == 0,
            capture_active: case == 1,
            diagnostic_case: (case == 2)
                .then_some(park_sim::diagnostics::DiagnosticCase::SaveScale),
            price_delta_pending: 1,
            ..default()
        };
        let mut app = economy_app(&directory, original.clone(), view);
        app.world_mut()
            .spawn((Button, Interaction::Pressed, ui::Action::TicketPrice(1)));
        let keys = if case == 3 {
            vec![KeyCode::F7, KeyCode::KeyH]
        } else {
            vec![KeyCode::F7]
        };
        persistence_key_frame(&mut app, &keys);
        assert_eq!(app.world().resource::<Simulation>().0, original);
        assert_eq!(app.world().resource::<ViewState>().price_delta_pending, 0);
        {
            let mut view = app.world_mut().resource_mut::<ViewState>();
            view.help = false;
            view.capture_active = false;
            view.diagnostic_case = None;
        }
        frame(&mut app);
        assert_eq!(
            app.world().resource::<Simulation>().0,
            original,
            "blocked edits must not leak into the next frame"
        );
    }
}

#[test]
fn save_includes_price_edit_and_primary_or_backup_load_wins_for_keys_and_buttons() {
    for keyboard in [true, false] {
        let directory = InputSaveDirectory::new();
        let initial = train_park();
        directory.store().save(&initial).unwrap();
        let mut app = economy_app(
            &directory,
            initial.clone(),
            ViewState {
                speed: Speed::Paused,
                ..default()
            },
        );
        if keyboard {
            persistence_key_frame(&mut app, &[KeyCode::F7, KeyCode::F5]);
        } else {
            let price = app
                .world_mut()
                .spawn((Button, Interaction::Pressed, ui::Action::TicketPrice(1)))
                .id();
            let save = app
                .world_mut()
                .spawn((Button, Interaction::Pressed, ui::Action::Save))
                .id();
            frame(&mut app);
            app.world_mut().despawn(price);
            app.world_mut().despawn(save);
        }
        let saved = app.world().resource::<Simulation>().0.clone();
        assert_eq!(saved.ride().unwrap().ticket_price_cents, PRICE_STEP_CENTS);
        assert_eq!(saved.ticks(), initial.ticks());
        assert_eq!(directory.store().load().unwrap(), saved);
        assert_eq!(directory.store().load_backup().unwrap(), initial);
        let files = saved_file_pair(&directory.store());
        for backup in [false, true] {
            if keyboard {
                let mut keys = vec![KeyCode::F7, KeyCode::F5, KeyCode::F9];
                if backup {
                    keys.push(KeyCode::ShiftLeft);
                }
                persistence_key_frame(&mut app, &keys);
            } else {
                let entities: Vec<_> = [
                    ui::Action::TicketPrice(1),
                    ui::Action::Save,
                    if backup {
                        ui::Action::LoadBackup
                    } else {
                        ui::Action::Load
                    },
                ]
                .into_iter()
                .map(|action| {
                    app.world_mut()
                        .spawn((Button, Interaction::Pressed, action))
                        .id()
                })
                .collect();
                frame(&mut app);
                for entity in entities {
                    app.world_mut().despawn(entity);
                }
            }
            assert_eq!(
                app.world().resource::<Simulation>().0,
                if backup {
                    initial.clone()
                } else {
                    saved.clone()
                }
            );
            let view = app.world().resource::<ViewState>();
            assert_eq!(view.speed, Speed::Paused);
            assert_eq!(view.price_delta_pending, 0);
            assert!(view.price_notice.is_none());
            assert_eq!(saved_file_pair(&directory.store()), files);
        }
    }
}

#[test]
fn failed_load_and_failed_backup_still_discard_simultaneous_price_change() {
    for backup in [false, true] {
        let directory = InputSaveDirectory::new();
        let original = train_park();
        // A corrupt primary and absent backup must not be overwritten or replaced
        // by a price edit/save that happened to arrive with the load request.
        std::fs::write(directory.store().save_path(), b"corrupt fixture").unwrap();
        let mut app = economy_app(&directory, original.clone(), ViewState::default());
        let mut keys = vec![KeyCode::F7, KeyCode::F5, KeyCode::F9];
        if backup {
            keys.push(KeyCode::ShiftRight);
        }
        persistence_key_frame(&mut app, &keys);
        assert_eq!(app.world().resource::<Simulation>().0, original);
        assert_eq!(app.world().resource::<ViewState>().price_delta_pending, 0);
        assert_eq!(
            std::fs::read(directory.store().save_path()).unwrap(),
            b"corrupt fixture"
        );
        assert!(!directory.store().backup_path().exists());
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, original);
    }
}

#[test]
fn price_feedback_refreshes_without_ticks_and_minimum_chrome_stays_reserved() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    park.set_ride_ticket_price(park.ride().unwrap().id, 2000)
        .unwrap();
    let mut app = economy_app(
        &directory,
        park,
        ViewState {
            speed: Speed::Paused,
            ..default()
        },
    );
    let header = app
        .world_mut()
        .spawn((ui::Label::Economy, Text::new("stale")))
        .id();
    let ticket = app
        .world_mut()
        .spawn((ui::Label::TicketPrice, Text::new("stale")))
        .id();
    app.add_systems(Update, ui::refresh.after(persistence::apply_save_action));
    frame(&mut app);
    assert_eq!(
        app.world().get::<Text>(header).unwrap().0,
        "Cash $1000.00 / Revenue $0.00 / Wallets $20.00"
    );
    assert_eq!(
        app.world().get::<Text>(ticket).unwrap().0,
        "Ticket $20.00\n12/12 can pay"
    );
    persistence_key_frame(&mut app, &[KeyCode::F7]);
    assert_eq!(
        app.world().get::<Text>(ticket).unwrap().0,
        "Ticket $20.50\n0/12 can pay"
    );
    for dpi in [1.0, 1.5, 2.0] {
        let window = test_window(1024, 680, dpi);
        assert!(ui::PATH_PANEL_TOP + ui::PATH_PANEL_HEIGHT <= window.height() - FOOTER);
        let center_x = window.width() - RIDE_PANEL_MARGIN - RIDE_PANEL_WIDTH * 0.5;
        for y in [
            HEADER + RIDE_PANEL_MARGIN + RIDE_PANEL_HEIGHT - 1.0,
            ui::PATH_PANEL_TOP + ui::PATH_PANEL_HEIGHT - 1.0,
        ] {
            assert!(!is_map_position(Vec2::new(center_x, y), &window, false));
        }
    }
}
