//! Player-facing research, supply ordering and operating reports.
use super::*;
use park_sim::{Promotion, ResearchPriority};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Command {
    Close,
    Enable,
    Tab(u8),
    Priority(ResearchPriority),
    Funding,
    AutoStock,
    Previous,
    Next,
    Order,
    ServicePrice(bool),
    ServiceOpen,
    Budget(bool),
    Promotion(Promotion),
    Flyers,
    Radio,
    Finances,
    Visitors,
    Staff,
    Catalogue,
    RidePrices(u32),
}
#[derive(Component)]
pub(crate) struct Panel;
#[derive(Component)]
pub(crate) struct Content;
#[derive(Component)]
pub(crate) struct Controls(u8);

pub(crate) fn open(view: &mut ViewState) {
    view.set_modal(false, false, false, false);
    view.management_open = true;
    view.ui_pointer_consumed = true;
}
pub(crate) fn command(park: &mut park_sim::Park, view: &mut ViewState, action: Command) {
    if !view.management_open
        || view.front_modal
        || view.capture_active
        || view.diagnostic_case.is_some()
    {
        return;
    }
    view.ui_pointer_consumed = true;
    let status = park.management_status();
    let id = park
        .services()
        .iter()
        .filter(|s| s.object.is_some())
        .nth(view.management_page)
        .map(|s| s.id);
    let result = match action {
        Command::Close => {
            view.set_modal(false, false, false, false);
            return;
        }
        Command::Enable => {
            park.enable_management();
            Ok("Management enabled. Choose research funding and review supplies.".into())
        }
        Command::Tab(tab) => {
            view.management_tab = tab;
            view.management_page = 0;
            return;
        }
        Command::Priority(p) => park
            .set_research(p, status.funding)
            .map(|_| format!("Research priority set to {}.", p.label())),
        Command::Funding => park
            .set_research(
                status.priority,
                if park
                    .scenario_status()
                    .is_some_and(|s| s.kind == park_sim::ScenarioKind::MeadowChallenge)
                    && status.funding == park_sim::ResearchFunding::Medium
                {
                    park_sim::ResearchFunding::Paused
                } else {
                    status.funding.next()
                },
            )
            .map(|_| "Research funding updated.".into()),
        Command::AutoStock => {
            park.set_automatic_stock(!status.automatic_stock);
            Ok("Automatic stock ordering updated.".into())
        }
        Command::Previous => {
            view.management_page = view.management_page.saturating_sub(1);
            return;
        }
        Command::Next => {
            view.management_page = (view.management_page + 1).min(255);
            return;
        }
        Command::Order => id
            .ok_or("Build a facility first".into())
            .and_then(|id| park.order_facility_stock(id, 20))
            .map(|_| "Purchased 20 units. The supplies ledger records their cost.".into()),
        Command::ServicePrice(up) => id
            .ok_or("Build a facility first".into())
            .and_then(|id| {
                let price = park.service(id).unwrap().price_cents;
                park.set_service_price(
                    id,
                    if up {
                        price.saturating_add(50)
                    } else {
                        price.saturating_sub(50)
                    },
                )
                .map_err(|e| e.to_string())
            })
            .map(|_| "Facility price updated for its next customer.".into()),
        Command::ServiceOpen => id
            .ok_or("Build a facility first".into())
            .and_then(|id| {
                park.set_service_open(id, !park.service(id).unwrap().open)
                    .map_err(|e| e.to_string())
            })
            .map(|_| "Facility operating status updated.".into()),
        Command::Budget(up) => park
            .set_staff_budget(if up {
                (status.staff_budget_cents + 100).min(2000)
            } else {
                status.staff_budget_cents.saturating_sub(100)
            })
            .map(|_| {
                "Staff payroll limit updated. Suspended staff retry when the budget allows.".into()
            }),
        Command::Promotion(p) => park
            .start_promotion(p)
            .map(|_| format!("{} started for five simulation minutes.", p.label())),
        Command::Flyers => park
            .start_flyer_campaign()
            .map_err(|e| e.to_string())
            .map(|_| "Local flyer campaign started.".into()),
        Command::Radio => park
            .start_radio_campaign()
            .map_err(|e| e.to_string())
            .map(|_| "Radio campaign started.".into()),
        Command::Finances => {
            view.set_modal(false, false, false, true);
            return;
        }
        Command::Visitors => {
            view.set_modal(false, true, false, false);
            view.guest_flow_view = true;
            return;
        }
        Command::Staff => {
            super::park_life::open(view);
            return;
        }
        Command::Catalogue => {
            super::catalogue::command(view, super::catalogue::Command::Open);
            return;
        }
        Command::RidePrices(price) => {
            let mut candidate = park.clone();
            let ids: Vec<_> = candidate.all_rides().map(|r| r.id).collect();
            let result = ids.into_iter().try_for_each(|id| {
                candidate
                    .set_ride_ticket_price(id, price)
                    .map_err(|e| e.to_string())
            });
            result.map(|_| { *park = candidate; format!("All installed ride prices set to {}. Reserved fares retain their quoted price.", money(u64::from(price))) })
        }
    };
    view.management_notice = result.unwrap_or_else(|e| e);
}
fn controls(p: &mut ChildSpawnerCommands, tab: u8, actions: &[(&str, Command)]) {
    p.spawn((
        Controls(tab),
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(6),
            row_gap: px(5),
            ..row()
        },
    ))
    .with_children(|p| {
        for (text, action) in actions {
            button(p, text, Action::Management(*action), 158.0);
        }
    });
}
pub(crate) fn setup(commands: &mut Commands) {
    commands
        .spawn((
            Panel,
            GlobalZIndex(12),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.09, 0.72)),
        ))
        .with_children(|p| {
            p.spawn((
                layout::ScrollLayer::Management,
                ScrollPosition::default(),
                layout::dialog(94.0, 1050.0, Some(610.0)),
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                p.spawn(row()).with_children(|p| {
                    label(p, "Management and economy", 25.0, INK);
                    button(p, "Close [Esc]", Action::Management(Command::Close), 130.0);
                });
                p.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    ..row()
                })
                .with_children(|p| {
                    for (i, text) in [
                        "Overview",
                        "Research",
                        "Supplies",
                        "Marketing",
                        "Policies",
                        "History",
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        button(p, text, Action::Management(Command::Tab(i as u8)), 135.0);
                    }
                });
                p.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(INK),
                    Content,
                ));
                controls(
                    p,
                    0,
                    &[
                        ("Enable management", Command::Enable),
                        ("Finance accounts", Command::Finances),
                        ("Visitor flow", Command::Visitors),
                        ("Staff and park life", Command::Staff),
                    ],
                );
                controls(
                    p,
                    1,
                    &[
                        ("Funding level", Command::Funding),
                        ("Build catalogue", Command::Catalogue),
                    ],
                );
                let priorities: Vec<_> = ResearchPriority::ALL
                    .into_iter()
                    .map(|p| (p.label(), Command::Priority(p)))
                    .collect();
                controls(p, 1, &priorities);
                controls(
                    p,
                    2,
                    &[
                        ("Previous facility", Command::Previous),
                        ("Next facility", Command::Next),
                        ("Order 20 units", Command::Order),
                        ("Automatic ordering", Command::AutoStock),
                        ("Price - $0.50", Command::ServicePrice(false)),
                        ("Price + $0.50", Command::ServicePrice(true)),
                        ("Open / close", Command::ServiceOpen),
                    ],
                );
                controls(
                    p,
                    3,
                    &[
                        ("Flyers / $25", Command::Flyers),
                        ("Radio / $40", Command::Radio),
                        ("Family week / $50", Command::Promotion(Promotion::Family)),
                        (
                            "Enthusiasts / $50",
                            Command::Promotion(Promotion::Enthusiast),
                        ),
                    ],
                );
                controls(
                    p,
                    4,
                    &[
                        ("Staff budget - $1", Command::Budget(false)),
                        ("Staff budget + $1", Command::Budget(true)),
                        ("Ride prices $2", Command::RidePrices(200)),
                        ("Ride prices $3", Command::RidePrices(300)),
                        ("Ride prices $5", Command::RidePrices(500)),
                        ("Entry pricing", Command::Visitors),
                        ("Staff team", Command::Staff),
                    ],
                );
                controls(
                    p,
                    5,
                    &[
                        ("Previous page", Command::Previous),
                        ("Next page", Command::Next),
                        ("Receipt ledger", Command::Finances),
                    ],
                );
            });
        });
}
pub(crate) fn refresh(
    park: Res<Simulation>,
    view: Res<ViewState>,
    mut panel: Query<&mut Node, (With<Panel>, Without<Controls>)>,
    mut controls: Query<(&Controls, &mut Node), Without<Panel>>,
    mut content: Query<&mut Text, With<Content>>,
) {
    for mut node in &mut panel {
        node.display = if view.management_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !view.management_open {
        return;
    }
    for (tab, mut node) in &mut controls {
        node.display = if tab.0 == view.management_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
    let p = &park.0;
    let s = p.management_status();
    let r = p.management_report();
    let intro = format!(
        "{}   Cash {}   Operating result {}\n{}\n\n",
        if s.enabled {
            "Management active"
        } else {
            "Legacy park · enable management to begin"
        },
        money(r.cash_cents),
        signed_money(r.operating_profit_cents()),
        view.management_notice
    );
    let body = match view.management_tab {
        1 => {
            let rows = ResearchPriority::ALL
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    format!(
                        "{}{} — {} invested",
                        if p == s.priority { "> " } else { "" },
                        p.label(),
                        money(s.research_spent[i])
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let locked = park_sim::RideModel::ALL
                .into_iter()
                .filter(|m| !p.ride_researched(*m))
                .map(|m| m.name())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "Research funding {}\nNext project {}\nEach project costs $40. Progress is saved when funding pauses or priorities change.\n\n{}\n\nAwaiting research\n{}",
                s.funding.label(),
                p.research_next_name(),
                rows,
                if locked.is_empty() {
                    "All ride models available"
                } else {
                    &locked
                }
            )
        }
        2 => {
            let service = p
                .services()
                .iter()
                .filter(|s| s.object.is_some())
                .nth(view.management_page);
            if let Some(service) = service {
                let stock = p.facility_stocks().iter().find(|s| s.service == service.id);
                let profit = i128::from(service.revenue_cents)
                    - i128::from(service.upkeep_cents)
                    - i128::from(service.supplies_cents);
                format!(
                    "Automatic ordering {} · Reorders 20 units below 5 · Capacity 100\n\n{} #{} · {}\nPrice {} · Supply cost {} per unit\nStock {} · Purchased {} · Served {}\nSales {} · Supplies {} · Upkeep {}\nLifetime operating result {}\n\nOrders are paid immediately. Guests pay only when served. Empty stock prevents a sale.\nClosing a facility stops automatic orders and upkeep.",
                    if s.automatic_stock { "on" } else { "off" },
                    service.kind,
                    service.id.0,
                    if service.open { "open" } else { "closed" },
                    money(u64::from(service.price_cents)),
                    money(u64::from(service.kind.supply_cost_cents())),
                    stock.map_or(0, |s| s.remaining),
                    stock.map_or(0, |s| s.purchased),
                    service.served,
                    money(service.revenue_cents),
                    money(service.supplies_cents),
                    money(service.upkeep_cents),
                    signed_money(profit)
                )
            } else {
                "Build a facility, or select the previous facility. Enable management to order stock.".into()
            }
        }
        3 => format!(
            "{} · {} seconds remaining\nPromotion purchases {} · Total cost {}\nAttributed candidates {} · Admitted {} · Entry receipts {}\n\nPromotions increase arrival frequency to one party every 10 seconds.\nFamily week requires a gentle ride. Enthusiast events require a coaster.\nAttribution counts candidates and paid entries while the promotion is active.\n\nFlyers and radio retain their separate results in Visitor flow.\nScenario rules and the guest's admission budget remain in effect.",
            s.promotion.label(),
            s.promotion_remaining.div_ceil(30),
            s.promotions_bought,
            money(s.promotions_bought * park_sim::PROMOTION_COST_CENTS),
            s.promotion_candidates,
            s.promotion_admissions,
            money(s.promotion_revenue_cents)
        ),
        4 => format!(
            "Park staff payroll cap {} per 30 seconds\n{} staff · {} waiting for wages\nResearch {} · Automatic orders {}\n\nThe payroll cap covers the five-role staff team. Legacy mechanic and grounds controls retain their own wages.\nRide price presets affect all installed rides. Existing reservations retain their price.\nEntry pricing, free-entry/free-ride rules and arrivals are in Visitor flow.\nUse Finance accounts for borrowing, repayments and interest.",
            money(s.staff_budget_cents),
            p.park_staff().len(),
            p.park_staff().iter().filter(|s| !s.working).count(),
            s.funding.label(),
            if s.automatic_stock { "on" } else { "off" }
        ),
        5 => {
            let reports = p.management_reports();
            let rows = reports
                .windows(2)
                .rev()
                .skip(view.management_page * 6)
                .take(6)
                .map(|pair| {
                    let a = &pair[0];
                    let b = &pair[1];
                    format!(
                        "Minute {:.1}–{:.1}   Sales {}   Costs {}   Result {}   Cash {}",
                        a.tick as f64 / 1800.0,
                        b.tick as f64 / 1800.0,
                        money(b.revenue_cents - a.revenue_cents),
                        money(b.expenses_cents - a.expenses_cents),
                        signed_money(b.operating_profit_cents() - a.operating_profit_cents()),
                        money(b.cash_cents)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n");
            format!(
                "Last 60 account snapshots · One per simulation minute · Page {}\nOperating costs include stock purchases, staff, research and marketing.\nConstruction, demolition and loan movements remain separate in Finance accounts.\n\n{}",
                view.management_page + 1,
                if rows.is_empty() {
                    "No completed periods on this page. Run the park for a minute."
                } else {
                    &rows
                }
            )
        }
        _ => format!(
            "Guest satisfaction {} / 100 · Admission conversion {}% · Ride reliability {}%\nPark assets {} · Debt including interest {}\nSales {} · Operating costs {}\nResearch {} · Promotions {}\n\n{}\n\nRating is active guests' average satisfaction. Popularity is admissions divided by candidates.\nReliability is the share of rides below the maintenance shutdown threshold.\nOpen Finance accounts for per-ride profit, asset values, loans and receipts.",
            r.rating,
            r.popularity,
            r.reliability,
            money(r.park_value_cents),
            money(r.debt_cents),
            money(r.revenue_cents),
            money(r.expenses_cents),
            money(p.finance_summary().research_cents),
            money(p.finance_summary().promotion_cents),
            p.management_notices()
                .into_iter()
                .take(5)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    };
    for mut text in &mut content {
        text.0 = format!("{intro}{body}");
    }
}
fn signed_money(value: i128) -> String {
    format!(
        "{}{}",
        if value < 0 { "-" } else { "+" },
        money(value.unsigned_abs().min(u128::from(u64::MAX)) as u64)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup_system(mut commands: Commands) {
        setup(&mut commands);
    }
    #[test]
    fn every_management_tab_refreshes_through_the_live_ui_system() {
        let mut park = park_sim::Park::new(32, 32);
        park.enable_management();
        let mut view = ViewState::default();
        open(&mut view);
        let mut app = App::new();
        app.insert_resource(Simulation(park))
            .insert_resource(view)
            .add_systems(Startup, setup_system)
            .add_systems(Update, refresh);
        for tab in 0..6 {
            app.world_mut().resource_mut::<ViewState>().management_tab = tab;
            app.update();
            let world = app.world_mut();
            let mut texts = world.query_filtered::<&Text, With<Content>>();
            assert!(texts.iter(world).all(|t| !t.0.is_empty()));
        }
    }
    #[test]
    fn modal_and_research_commands_work_through_player_actions() {
        let mut p = park_sim::Park::new(32, 32);
        let mut v = ViewState::default();
        open(&mut v);
        assert!(v.edits_blocked());
        command(&mut p, &mut v, Command::Enable);
        command(&mut p, &mut v, Command::Priority(ResearchPriority::Water));
        command(&mut p, &mut v, Command::Funding);
        assert_eq!(p.management_status().priority, ResearchPriority::Water);
        assert_eq!(
            p.management_status().funding,
            park_sim::ResearchFunding::Low
        );
        command(&mut p, &mut v, Command::Close);
        assert!(!v.management_open);
    }
    #[test]
    fn background_and_capture_cannot_change_policies() {
        let mut p = park_sim::Park::new(32, 32);
        let mut v = ViewState::default();
        command(&mut p, &mut v, Command::Enable);
        assert!(!p.management_status().enabled);
        open(&mut v);
        v.capture_active = true;
        command(&mut p, &mut v, Command::Enable);
        assert!(!p.management_status().enabled);
    }
}
