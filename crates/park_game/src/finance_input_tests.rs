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
                ui::finance::apply_commands,
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
fn ride_cashflow_view_pages_retained_rides_and_is_read_only_through_actual_buttons() {
    let directory = InputSaveDirectory::new();
    let mut park = train_park();
    for _ in 0..3 {
        let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        park.remove_carousel(id).unwrap();
    }
    let live = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    let before = park.clone();
    let (mut app, _) = finance_app(
        &directory,
        park,
        ViewState {
            finance_open: true,
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    click(&mut app, ui::Action::FinanceView);
    assert_eq!(
        app.world().resource::<ViewState>().finance_view,
        ui::finance::FinanceView::Rides
    );
    assert!(
        label(&mut app, |l| matches!(
            l,
            ui::finance::FinanceLabel::History
        ))
        .contains("5 retained rides / 2 currently installed")
    );
    assert!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::Row(1)))
            .contains("Not installed / Net cash -$150.00")
    );
    click(&mut app, ui::Action::FinancePage(1));
    assert_eq!(app.world().resource::<ViewState>().finance_page, 1);
    assert!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::Row(0))).contains(&format!(
            "Ride #{} / Carousel / Closed / Net cash -$300.00",
            live.0
        ))
    );
    assert!(label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::Row(4))).is_empty());
    // A view switch wins over simultaneous page requests, independent of order.
    let page_button = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, ui::Action::FinancePage(1)))
        .id();
    click(&mut app, ui::Action::FinanceView);
    app.world_mut().despawn(page_button);
    assert_eq!(
        app.world().resource::<ViewState>().finance_view,
        ui::finance::FinanceView::ParkSummary
    );
    assert_eq!(app.world().resource::<ViewState>().finance_page, 0);
    assert!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::Row(0)))
            .contains("Rides / Operating profit +$0.00")
    );
    assert!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::Row(2)))
            .contains("excluded from operating profit")
    );
    click(&mut app, ui::Action::FinancePage(1));
    assert_eq!(app.world().resource::<ViewState>().finance_page, 0);
    click(&mut app, ui::Action::FinanceView);
    assert_eq!(
        app.world().resource::<ViewState>().finance_view,
        ui::finance::FinanceView::Loans
    );
    click(&mut app, ui::Action::FinanceView);
    assert_eq!(
        app.world().resource::<ViewState>().finance_view,
        ui::finance::FinanceView::Value
    );
    click(&mut app, ui::Action::FinanceView);
    assert_eq!(
        app.world().resource::<ViewState>().finance_view,
        ui::finance::FinanceView::Transactions
    );
    assert_eq!(app.world().resource::<Simulation>().0, before);
    assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
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
    assert_eq!(park.finance_ledger().archived_entries, 6);
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
        first.starts_with("06:39 / ") && first.contains("-$0.10 / Cash $886.70"),
        "{first}"
    );
    assert!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::History
        ))
        .contains("Older 6 transactions: income +$0.00 / costs -$100.50")
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
        18
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

#[test]
fn finance_shows_construction_and_refunds_separately_from_guest_sales_and_keeps_archives() {
    let directory = InputSaveDirectory::new();
    let mut park = Park::new(32, 32);
    let id = park.place_carousel(TilePos { x: 12, y: 12 }, 0).unwrap();
    park.remove_carousel(id).unwrap();
    let (mut app, _) = finance_app(
        &directory,
        park.clone(),
        ViewState {
            finance_open: true,
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    assert_eq!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::Row(0)
        )),
        "00:00 / Demolition refund / ride #1 / +$150.00 / Cash $850.00"
    );
    assert_eq!(
        label(&mut app, |kind| matches!(
            kind,
            ui::finance::FinanceLabel::Row(1)
        )),
        "00:00 / Ride construction / ride #1 / -$300.00 / Cash $700.00"
    );
    let summary = label(&mut app, |kind| {
        matches!(kind, ui::finance::FinanceLabel::Summary)
    });
    assert!(
        summary.contains("Sales +$0.00 / Costs -$300.00 / Net cash -$150.00"),
        "{summary}"
    );
    assert!(summary.contains("Refunds +$150.00"));
    let categories = label(&mut app, |kind| {
        matches!(kind, ui::finance::FinanceLabel::Categories)
    });
    assert!(categories.contains("Construction -$300.00 / Demolition refunds +$150.00"));
    assert_eq!(app.world().resource::<Simulation>().0, park);

    park.place_service(
        TilePos { x: 4, y: 5 },
        0,
        PathDirection::North,
        ServiceKind::Toilet,
    )
    .unwrap();
    for _ in 0..FINANCE_LEDGER_CAPACITY * usize::from(SERVICE_UPKEEP_TICKS) {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.finance_ledger().archived_entries, 3);
    app.world_mut().resource_mut::<Simulation>().0 = park.clone();
    frame(&mut app);
    let history = label(&mut app, |kind| {
        matches!(kind, ui::finance::FinanceLabel::History)
    });
    assert!(
        history.contains("Older 3 transactions: income +$0.00 / costs -$400.00 / refunds +$150.00"),
        "{history}"
    );
    let summary = label(&mut app, |kind| {
        matches!(kind, ui::finance::FinanceLabel::Summary)
    });
    assert!(
        summary.contains("Cash $737.20 / Sales +$0.00 / Costs -$412.80 / Net cash -$262.80"),
        "{summary}"
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
}

#[test]
fn finance_distinguishes_facility_and_ride_capital_even_when_their_numeric_ids_match() {
    let directory = InputSaveDirectory::new();
    let mut park = Park::new(32, 32);
    let ride = park.place_carousel(TilePos { x: 12, y: 12 }, 0).unwrap();
    park.remove_carousel(ride).unwrap();
    let facility = park
        .place_service(
            TilePos { x: 4, y: 5 },
            0,
            PathDirection::North,
            ServiceKind::Drink,
        )
        .unwrap();
    assert_eq!(facility.0, ride.0);
    park.remove_object(park.service(facility).unwrap().object.unwrap())
        .unwrap();
    let (mut app, _) = finance_app(
        &directory,
        park.clone(),
        ViewState {
            finance_open: true,
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    for (index, expected) in [
        "00:00 / Demolition refund / facility #1 / +$40.00 / Cash $810.00",
        "00:00 / Facility construction / facility #1 / -$80.00 / Cash $770.00",
        "00:00 / Demolition refund / ride #1 / +$150.00 / Cash $850.00",
        "00:00 / Ride construction / ride #1 / -$300.00 / Cash $700.00",
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(
            label(
                &mut app,
                |kind| matches!(kind,ui::finance::FinanceLabel::Row(row) if *row==index)
            ),
            *expected
        );
    }
    let categories = label(&mut app, |kind| {
        matches!(kind, ui::finance::FinanceLabel::Categories)
    });
    assert!(categories.contains("Construction -$380.00 / Demolition refunds +$190.00"));
    let summary = label(&mut app, |kind| {
        matches!(kind, ui::finance::FinanceLabel::Summary)
    });
    assert!(summary.contains("Sales +$0.00 / Costs -$380.00 / Net cash -$190.00"));
    assert_eq!(app.world().resource::<Simulation>().0, park);
}

#[test]
fn park_summary_buttons_show_archived_removed_service_loss_separate_from_capital() {
    let directory = InputSaveDirectory::new();
    let mut park = finance_park(140);
    let object = park.services()[0].object.unwrap();
    park.remove_object(object).unwrap();
    assert!(park.finance_ledger().archived_entries > 0);
    let bytes = park.to_save_bytes().unwrap();
    let (mut app, _) = finance_app(
        &directory,
        park.clone(),
        ViewState {
            finance_open: true,
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    click(&mut app, ui::Action::FinanceView);
    assert_eq!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::View)),
        "Park summary"
    );
    click(&mut app, ui::Action::FinanceView);
    for (index, expected) in [
        "Rides / Operating profit +$0.00\nTickets +$0.00 / Upkeep -$0.00",
        "Services / Operating profit -$14.00\nSales +$0.00 / Upkeep -$14.00 / Supplies -$0.00",
        "Combined operating profit -$14.00\nConstruction -$100.00 / Refunds +$50.00 / excluded from operating profit",
        "Financing / Borrowed +$0.00 / Repaid -$0.00 / Interest paid -$0.00\nLoan balance $0.00 / Interest due $0.00",
        "Net cash -$64.00 / includes construction, refunds and financing\nStarting cash $1000.00 + net cash = current cash $936.00",
    ].iter().enumerate() {
        assert_eq!(label(&mut app, |l|matches!(l,ui::finance::FinanceLabel::Row(i) if *i==index)),*expected);
    }
    click(&mut app, ui::Action::FinancePage(1));
    click(&mut app, ui::Action::FinancePage(-1));
    assert_eq!(app.world().resource::<ViewState>().finance_page, 0);
    click(&mut app, ui::Action::CloseFinance);
    assert!(!app.world().resource::<ViewState>().finance_open);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .unwrap(),
        bytes
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
}

#[test]
fn loan_buttons_borrow_once_and_reject_limits_without_touching_operating_profit() {
    use ui::finance::{FinanceView, LoanCommand};
    let directory = InputSaveDirectory::new();
    let park = Park::new(32, 32);
    let (mut app, _) = finance_app(
        &directory,
        park,
        ViewState {
            finance_open: true,
            finance_view: FinanceView::Loans,
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    click(&mut app, ui::Action::Loan(LoanCommand::Borrow));
    let p = &app.world().resource::<Simulation>().0;
    assert_eq!(p.cash_cents(), 110_000);
    assert_eq!(p.loan_stats().principal_cents, 10_000);
    assert_eq!(p.finance_summary().operating_profit_cents, 0);
    click(&mut app, ui::Action::Loan(LoanCommand::Repay));
    assert_eq!(app.world().resource::<Simulation>().0.cash_cents(), 100_000);
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .borrow_loan(park_sim::MAX_LOAN_CENTS)
        .unwrap();
    let before = app
        .world()
        .resource::<Simulation>()
        .0
        .to_save_bytes()
        .unwrap();
    click(&mut app, ui::Action::Loan(LoanCommand::Borrow));
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
            .finance_loan_notice
            .as_ref()
            .unwrap()
            .contains("limit")
    );
}

#[test]
fn loan_conflicts_capture_and_close_transitions_never_change_cash() {
    use ui::finance::{FinanceView, LoanCommand};
    for reverse in [false, true] {
        let directory = InputSaveDirectory::new();
        let park = Park::new(32, 32);
        let (mut app, _) = finance_app(
            &directory,
            park.clone(),
            ViewState {
                finance_open: true,
                finance_view: FinanceView::Loans,
                speed: Speed::Paused,
                ..default()
            },
        );
        frame(&mut app);
        let mut cmds = vec![LoanCommand::Borrow, LoanCommand::Repay];
        if reverse {
            cmds.reverse();
        }
        let ids: Vec<_> = cmds
            .into_iter()
            .map(|c| {
                app.world_mut()
                    .spawn((Button, Interaction::Pressed, ui::Action::Loan(c)))
                    .id()
            })
            .collect();
        frame(&mut app);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        assert!(
            app.world()
                .resource::<ViewState>()
                .finance_loan_notice
                .as_ref()
                .unwrap()
                .contains("one loan")
        );
        for id in ids {
            app.world_mut().despawn(id);
        }
        app.world_mut().resource_mut::<ViewState>().capture_active = true;
        click(&mut app, ui::Action::Loan(LoanCommand::Borrow));
        assert_eq!(app.world().resource::<Simulation>().0, park);
        app.world_mut().resource_mut::<ViewState>().capture_active = false;
        let id = app
            .world_mut()
            .spawn((
                Button,
                Interaction::Pressed,
                ui::Action::Loan(LoanCommand::Borrow),
            ))
            .id();
        click(&mut app, ui::Action::CloseFinance);
        app.world_mut().despawn(id);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        assert!(!app.world().resource::<ViewState>().finance_open);
    }
}

#[test]
fn park_value_view_shows_signed_equity_and_blocks_loan_commands_through_actual_buttons() {
    use ui::finance::{FinanceView, LoanCommand};
    let directory = InputSaveDirectory::new();
    let park = save_check::value::negative().unwrap();
    let bytes = park.to_save_bytes().unwrap();
    let (mut app, _) = finance_app(
        &directory,
        park.clone(),
        ViewState {
            finance_open: true,
            finance_view: FinanceView::Loans,
            speed: Speed::Paused,
            ..default()
        },
    );
    frame(&mut app);
    assert_eq!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::View)),
        "Park value"
    );
    click(&mut app, ui::Action::FinanceView);
    assert_eq!(
        app.world().resource::<ViewState>().finance_view,
        FinanceView::Value
    );
    assert_eq!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::Row(0))),
        "Recorded park value $0.00\nTrack $0.00 / Carousels $0.00 / Facilities $0.00"
    );
    assert!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::Row(2)))
            .contains("Loan principal -$200.00 / Unpaid interest -$0.00")
    );
    assert!(
        label(&mut app, |l| matches!(l, ui::finance::FinanceLabel::Row(3)))
            .contains("Company book value -$50.00")
    );
    click(&mut app, ui::Action::FinancePage(1));
    click(&mut app, ui::Action::FinancePage(-1));
    click(&mut app, ui::Action::Loan(LoanCommand::Borrow));
    assert_eq!(app.world().resource::<ViewState>().finance_page, 0);
    click(&mut app, ui::Action::CloseFinance);
    assert_eq!(
        app.world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .unwrap(),
        bytes
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
}
