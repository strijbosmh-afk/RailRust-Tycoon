//! Read-only financial history, shown newest first without changing ledger order.
use park_sim::{FinanceEntry, FinanceKind, Park, RideCashflow, RideKind};

use super::*;

pub(crate) const PANEL_WIDTH: f32 = 760.0;
pub(crate) const PANEL_HEIGHT: f32 = 520.0;
pub(crate) const PAGE_SIZE: usize = 8;
pub(crate) const RIDE_PAGE_SIZE: usize = 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum FinanceView {
    #[default]
    Transactions,
    Rides,
    ParkSummary,
}

#[derive(Component)]
pub(crate) struct FinanceRow(usize);

#[derive(Component)]
pub(crate) struct FinancePanel;

#[derive(Component)]
pub(crate) enum FinanceLabel {
    Summary,
    Categories,
    History,
    Row(usize),
    Page,
    View,
    Footer,
    ListHeading,
}

fn text(parent: &mut ChildSpawnerCommands, value: &str, size: f32, kind: FinanceLabel) {
    parent.spawn((
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(INK),
        kind,
    ));
}

pub(super) fn setup(commands: &mut Commands) {
    commands
        .spawn((
            FinancePanel,
            GlobalZIndex(9),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.09, 0.65)),
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    width: px(PANEL_WIDTH),
                    height: px(PANEL_HEIGHT),
                    padding: UiRect::all(px(20)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(5),
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..row()
                })
                .with_children(|p| {
                    label(p, "Park finances", 23.0, INK);
                    p.spawn((
                        Button,
                        Action::FinanceView,
                        Node {
                            width: px(170),
                            height: px(30),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(BUTTON),
                        BorderColor::all(BORDER),
                        BorderRadius::all(px(5)),
                    ))
                    .with_children(|p| text(p, "Ride cashflows", 12.0, FinanceLabel::View));
                    button(p, "Close [N/Esc]", Action::CloseFinance, 140.0);
                });
                text(
                    p,
                    "Cash / Income / Costs / Net",
                    14.0,
                    FinanceLabel::Summary,
                );
                text(
                    p,
                    "Ride tickets / Service sales / Upkeep / Supplies / Construction / Refunds",
                    11.0,
                    FinanceLabel::Categories,
                );
                p.spawn(Node {
                    height: px(52),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|p| {
                    text(p, "", 11.0, FinanceLabel::History);
                });
                p.spawn(Node {
                    height: px(208),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                })
                .with_children(|p| {
                    text(p, "", 11.0, FinanceLabel::ListHeading);
                    for index in 0..PAGE_SIZE {
                        p.spawn((
                            FinanceRow(index),
                            Node {
                                height: px(22),
                                flex_shrink: 0.0,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            text(p, "", 11.0, FinanceLabel::Row(index));
                        });
                    }
                });
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..row()
                })
                .with_children(|p| {
                    button(p, "Previous [PgUp]", Action::FinancePage(-1), 140.0);
                    text(p, "Page 1 / 1", 12.0, FinanceLabel::Page);
                    button(p, "Next [PgDn]", Action::FinancePage(1), 140.0);
                });
                text(p, "", 11.0, FinanceLabel::Footer);
            });
        });
}

pub(super) fn signed_money(cents: i64) -> String {
    format!(
        "{}{}",
        if cents < 0 { "-" } else { "+" },
        money(cents.unsigned_abs())
    )
}

pub(crate) fn signed_cash(cents: i128) -> String {
    let amount = cents.unsigned_abs();
    format!(
        "{}${}.{:02}",
        if cents < 0 { "-" } else { "+" },
        amount / 100,
        amount % 100
    )
}

pub(crate) fn cashflow_text(row: &RideCashflow) -> String {
    format!(
        "Ride #{} / {} / {} / Net cash {}\nTickets +{} / Build -{} / Refunds +{} / Upkeep -{}",
        row.ride.0,
        if row.kind == RideKind::Coaster {
            "Coaster"
        } else {
            "Carousel"
        },
        if !row.installed {
            "Not installed"
        } else if row.open {
            "Open"
        } else if row.closed_for_funds {
            "Closed: funds"
        } else {
            "Closed"
        },
        signed_cash(row.net_cash_cents),
        money(row.ticket_revenue_cents),
        money(row.construction_cents),
        money(row.refunds_cents),
        money(row.upkeep_cents)
    )
}

pub(crate) fn summary_rows(park: &Park) -> [String; 4] {
    let report = park.finance_summary();
    [
        format!(
            "Rides / Operating profit {}\nTickets +{} / Upkeep -{}",
            signed_cash(report.rides.profit_cents),
            money(report.rides.revenue_cents),
            money(report.rides.upkeep_cents)
        ),
        format!(
            "Services / Operating profit {}\nSales +{} / Upkeep -{} / Supplies -{}",
            signed_cash(report.services.profit_cents),
            money(report.services.revenue_cents),
            money(report.services.upkeep_cents),
            money(report.services.supplies_cents)
        ),
        format!(
            "Combined operating profit {}\nConstruction -{} / Refunds +{} / excluded from operating profit",
            signed_cash(report.operating_profit_cents),
            money(report.construction_cents),
            money(report.refunds_cents)
        ),
        format!(
            "Net cash {} = operating profit + refunds - construction\nStarting cash {} + net cash = current cash {}",
            signed_cash(report.net_cash_cents),
            money(report.starting_cash_cents),
            money(report.cash_cents)
        ),
    ]
}

fn clock(tick: u64) -> String {
    let seconds = tick / TICKS_PER_SECOND;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn row_text(entry: &FinanceEntry) -> String {
    let (label, sign) = match entry.kind {
        FinanceKind::RideUpkeep => ("Ride upkeep", "-"),
        FinanceKind::RideRevenue => ("Ride tickets", "+"),
        FinanceKind::ServiceRevenue => ("Service sales", "+"),
        FinanceKind::ServiceUpkeep => ("Service upkeep", "-"),
        FinanceKind::ServiceSupplies => ("Service supplies", "-"),
        FinanceKind::RideConstruction => ("Ride construction", "-"),
        FinanceKind::RideDemolitionRefund => ("Demolition refund", "+"),
        FinanceKind::ServiceConstruction => ("Facility construction", "-"),
        FinanceKind::ServiceDemolitionRefund => ("Demolition refund", "+"),
        FinanceKind::TrackConstruction => ("Track construction", "-"),
        FinanceKind::TrackRefund => ("Track refund", "+"),
    };
    let service = entry.ride.map_or_else(
        || {
            entry
                .service
                .map_or(String::new(), |id| format!(" / facility #{}", id.0))
        },
        |id| format!(" / ride #{}", id.0),
    );
    format!(
        "{} / {label}{service} / {sign}{} / Cash {}",
        clock(entry.tick),
        money(entry.amount_cents),
        money(entry.balance_cents)
    )
}

fn page(park: &Park, view: &mut ViewState) -> (usize, usize) {
    let pages = match view.finance_view {
        FinanceView::Transactions => park.finance_ledger().entries.len().div_ceil(PAGE_SIZE),
        FinanceView::Rides => park.ride_cashflows().len().div_ceil(RIDE_PAGE_SIZE),
        FinanceView::ParkSummary => 1,
    }
    .max(1);
    let current = view.finance_page.min(pages - 1);
    let delta = std::mem::take(&mut view.finance_page_pending);
    view.finance_page = (current as i64 + i64::from(delta)).clamp(0, (pages - 1) as i64) as usize;
    (view.finance_page, pages)
}

pub(crate) fn refresh(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    mut panels: Query<&mut Node, With<FinancePanel>>,
    mut rows: Query<(&FinanceRow, &mut Node), Without<FinancePanel>>,
    mut labels: Query<(&FinanceLabel, &mut Text)>,
) {
    for mut panel in &mut panels {
        panel.display = if view.finance_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !view.finance_open {
        view.finance_page_pending = 0;
        view.finance_view_pending = false;
        return;
    }
    if std::mem::take(&mut view.finance_view_pending) {
        view.finance_view = match view.finance_view {
            FinanceView::Transactions => FinanceView::Rides,
            FinanceView::Rides => FinanceView::ParkSummary,
            FinanceView::ParkSummary => FinanceView::Transactions,
        };
        view.finance_page = 0;
        view.finance_page_pending = 0;
    }
    let rides = view.finance_view == FinanceView::Rides;
    let summary = view.finance_view == FinanceView::ParkSummary;
    let double_rows = rides || summary;
    for (row, mut node) in &mut rows {
        node.display = if double_rows && row.0 >= RIDE_PAGE_SIZE {
            Display::None
        } else {
            Display::Flex
        };
        node.height = px(if double_rows { 44 } else { 22 });
    }
    let (page, pages) = page(&sim.0, &mut view);
    let stats = sim.0.finance_stats();
    let ledger = sim.0.finance_ledger();
    let cashflows = sim.0.ride_cashflows();
    let summary_rows = summary_rows(&sim.0);
    for (label, mut text) in &mut labels {
        let value = match label {
            FinanceLabel::ListHeading if summary => "PARK SUMMARY / lifetime operating results and cash reconciliation".into(),
            FinanceLabel::View if summary => "Transactions".into(),
            FinanceLabel::Footer if summary => "Operating profit = sales - upkeep - supplies. Historical costs stay after demolition.".into(),
            FinanceLabel::Categories if summary => "Operating profit excludes construction and refunds.\nLifetime recorded amounts include removed rides/facilities and older receipts.".into(),
            FinanceLabel::History if summary => format!("{} retained rides / {} retained facilities\n{} older transactions are included, not discarded.\nThis is not a monthly report, asset valuation or forecast.",cashflows.len(),sim.0.services().len(),compact_count(ledger.archived_entries)),
            FinanceLabel::Row(index) if summary => summary_rows.get(*index).cloned().unwrap_or_default(),
            FinanceLabel::ListHeading => if rides {"RIDE CASHFLOWS / four rides per page / recorded amounts"} else {"RECENT TRANSACTIONS / newest first / resulting cash at right"}.into(),
            FinanceLabel::View => if rides {"Park summary"} else {"Ride cashflows"}.into(),
            FinanceLabel::Footer => if rides {"Net cash = tickets + refunds - build - upkeep. Ride upkeep accrues while open."} else {"Open facilities cost money even without customers. Space pauses all costs and activity."}.into(),
            FinanceLabel::Categories if rides => "RIDE CASHFLOWS / stable ride IDs / lifetime recorded amounts\nEach ride keeps its history after demolition; facility transactions are separate.".into(),
            FinanceLabel::History if rides => format!("{} retained rides / {} currently installed\nRecorded cash is not a ride valuation or a profit forecast.\nLegacy free construction has $0 recorded cost. Older receipts remain included.",cashflows.len(),cashflows.iter().filter(|row|row.installed).count()),
            FinanceLabel::Summary => format!(
                "Cash {} / Sales +{} / Costs -{} / Net cash {}\nRefunds +{} / Open rides/facilities: {} per simulation minute",
                money(stats.cash_cents),
                money(
                    stats
                        .ride_revenue_cents
                        .saturating_add(stats.service_revenue_cents)
                ),
                money(stats.total_costs_cents),
                signed_money(stats.net_profit_cents),
                money(stats.demolition_refunds_cents),
                money(stats.open_upkeep_per_minute_cents)
            ),
            FinanceLabel::Categories => format!(
                "Ride tickets +{} / Service sales +{}\nRide upkeep -{} / Facility upkeep -{} / Supplies -{}\nConstruction -{} / Demolition refunds +{}",
                money(stats.ride_revenue_cents),
                money(stats.service_revenue_cents),
                money(stats.ride_upkeep_cents),money(stats.service_upkeep_cents),
                money(stats.supplies_cents),
                money(stats.construction_cents),
                money(stats.demolition_refunds_cents)
            ),
            FinanceLabel::History => format!(
                "History starts at {} with cash {}. Lifetime totals above include earlier activity.\nOlder {} transactions: income +{} / costs -{} / refunds +{}.\nShowing the latest {} transactions; older amounts remain included in all totals.",
                clock(ledger.opening_tick),
                money(ledger.opening_cash_cents),
                compact_count(ledger.archived_entries),
                money(
                    ledger
                        .archived
                        .ride_revenue_cents
                        .saturating_add(ledger.archived.service_revenue_cents)
                ),
                money(
                    ledger
                        .archived
                        .upkeep_cents
                        .saturating_add(ledger.archived.supplies_cents)
                        .saturating_add(ledger.archived.construction_cents)
                ),
                money(ledger.archived.demolition_refunds_cents),
                ledger.entries.len()
            ),
            FinanceLabel::Row(index) if rides => cashflows.get(page*RIDE_PAGE_SIZE+index).filter(|_|*index < RIDE_PAGE_SIZE).map_or_else(||if *index == 0 && cashflows.is_empty() {"No ride history yet.".into()} else {String::new()},cashflow_text),
            FinanceLabel::Row(index) => ledger
                .entries
                .iter()
                .rev()
                .nth(page * PAGE_SIZE + index)
                .map_or_else(
                    || {
                        if *index == 0 && ledger.entries.is_empty() {
                            "No transactions yet.".into()
                        } else {
                            String::new()
                        }
                    },
                    row_text,
                ),
            FinanceLabel::Page => format!("Page {} / {pages}", page + 1),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_rows_use_transaction_kind_for_sign_and_preserve_positive_amounts() {
        assert_eq!(signed_cash(-123), "-$1.23");
        assert_eq!(signed_cash(0), "+$0.00");
        assert_eq!(
            signed_cash(i128::MIN),
            "-$1701411834604692317316873037158841057.28"
        );
        for (kind, sign) in [
            (FinanceKind::ServiceRevenue, "+"),
            (FinanceKind::ServiceUpkeep, "-"),
            (FinanceKind::ServiceSupplies, "-"),
            (FinanceKind::ServiceConstruction, "-"),
            (FinanceKind::ServiceDemolitionRefund, "+"),
        ] {
            let entry = FinanceEntry {
                ride: None,
                id: 3,
                tick: 90,
                kind,
                service: Some(park_sim::ServiceId(7)),
                amount_cents: 60,
                balance_cents: 1234,
            };
            let text = row_text(&entry);
            assert!(text.starts_with("00:03 / "));
            assert!(text.contains(&format!(" / facility #7 / {sign}$0.60 / Cash $12.34")));
            assert_eq!(entry.amount_cents, 60);
        }
        assert_eq!(signed_money(-123), "-$1.23");
        assert_eq!(signed_money(0), "+$0.00");
        assert_eq!(signed_money(i64::MIN), "-$92233720368547758.08");
        for (kind, label, sign) in [
            (FinanceKind::RideRevenue, "Ride tickets", "+"),
            (FinanceKind::RideConstruction, "Ride construction", "-"),
            (FinanceKind::RideDemolitionRefund, "Demolition refund", "+"),
            (FinanceKind::TrackConstruction, "Track construction", "-"),
            (FinanceKind::TrackRefund, "Track refund", "+"),
        ] {
            let entry = FinanceEntry {
                id: 1,
                tick: 30,
                kind,
                ride: Some(park_sim::RideId(2)),
                service: None,
                amount_cents: 150,
                balance_cents: 100150,
            };
            assert_eq!(
                row_text(&entry),
                format!("00:01 / {label} / ride #2 / {sign}$1.50 / Cash $1001.50")
            );
        }
    }
}
