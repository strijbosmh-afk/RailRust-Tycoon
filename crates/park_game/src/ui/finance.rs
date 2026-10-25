//! Read-only financial history, shown newest first without changing ledger order.
use park_sim::{FinanceEntry, FinanceKind, Park, RideCashflow, RideKind};

use super::*;

pub(crate) const PANEL_WIDTH: f32 = 760.0;
pub(crate) const PANEL_HEIGHT: f32 = 520.0;
pub(crate) const PAGE_SIZE: usize = 8;
pub(crate) const RIDE_PAGE_SIZE: usize = 4;
pub(crate) const SUMMARY_PAGE_SIZE: usize = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoanCommand {
    Borrow,
    Repay,
    Interest,
    Conflict,
}

pub(crate) fn queue_loan(view: &mut ViewState, command: LoanCommand) {
    view.finance_loan_pending = Some(if view.finance_loan_pending.is_some() {
        LoanCommand::Conflict
    } else {
        command
    });
}

pub(crate) fn apply_commands(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    let Some(command) = view.finance_loan_pending.take() else {
        return;
    };
    if !view.finance_open
        || view.finance_view != FinanceView::Loans
        || view.finance_view_pending
        || view.modal_input_blocked
        || view.capture_active
        || view.diagnostic_case.is_some()
    {
        return;
    }
    let result = match command {
        LoanCommand::Borrow => sim
            .0
            .borrow_loan(park_sim::LOAN_STEP_CENTS)
            .map(|_| "Borrowed $100; recorded as funding.".into()),
        LoanCommand::Repay => sim
            .0
            .repay_loan(park_sim::LOAN_STEP_CENTS)
            .map(|_| "Repaid $100 and settled accrued whole-cent interest.".into()),
        LoanCommand::Interest => sim
            .0
            .pay_loan_interest()
            .map(|paid| format!("Paid {} interest.", money(paid))),
        LoanCommand::Conflict => {
            view.finance_loan_notice = Some("Choose one loan command at a time.".into());
            return;
        }
    };
    view.finance_loan_notice =
        Some(result.unwrap_or_else(|error| format!("Loan unchanged: {error}")));
}

fn loan_rows(park: &Park) -> [String; 4] {
    let loan = park.loan_stats();
    [
        format!("Outstanding principal {} / Available to borrow {}\nBorrowing adds cash and debt; it never counts as a sale.",money(loan.principal_cents),money(loan.available_cents)),
        format!("Lifetime borrowed +{} / Principal repaid -{}\nPrincipal repayment reduces cash and debt, not operating profit.",money(loan.borrowed_cents),money(loan.repaid_cents)),
        format!("Interest accrued {} / Paid -{} / Still due {}\nUnpaid interest is retained; no automatic borrowing or compounding.",money(loan.interest_accrued_cents),money(loan.interest_paid_cents),money(loan.interest_due_cents)),
        "Repayments first settle all accrued whole-cent interest.\nPartial automatic interest payments use available cash; the rest stays due.".into(),
    ]
}
#[derive(Component)]
pub(crate) struct FinanceLoanControls;
#[derive(Component)]
pub(crate) struct FinancePaging;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum FinanceView {
    #[default]
    Transactions,
    Rides,
    ParkSummary,
    Loans,
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
                p.spawn((
                    FinancePaging,
                    Node {
                        justify_content: JustifyContent::SpaceBetween,
                        ..row()
                    },
                ))
                .with_children(|p| {
                    button(p, "Previous [PgUp]", Action::FinancePage(-1), 140.0);
                    text(p, "Page 1 / 1", 12.0, FinanceLabel::Page);
                    button(p, "Next [PgDn]", Action::FinancePage(1), 140.0);
                });
                p.spawn((
                    FinanceLoanControls,
                    Node {
                        display: Display::None,
                        justify_content: JustifyContent::SpaceBetween,
                        ..row()
                    },
                ))
                .with_children(|p| {
                    button(p, "Borrow $100", Action::Loan(LoanCommand::Borrow), 200.0);
                    button(p, "Repay $100", Action::Loan(LoanCommand::Repay), 200.0);
                    button(
                        p,
                        "Pay interest",
                        Action::Loan(LoanCommand::Interest),
                        200.0,
                    );
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

pub(crate) fn summary_rows(park: &Park) -> [String; 5] {
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
            "Financing / Borrowed +{} / Repaid -{} / Interest paid -{}\nLoan balance {} / Interest due {}",
            money(report.loan_borrowed_cents),
            money(report.loan_repaid_cents),
            money(report.loan_interest_cents),
            money(park.loan_stats().principal_cents),
            money(park.loan_stats().interest_due_cents)
        ),
        format!(
            "Net cash {} / includes construction, refunds and financing\nStarting cash {} + net cash = current cash {}",
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
        FinanceKind::LoanBorrow => ("Loan borrowing", "+"),
        FinanceKind::LoanRepayment => ("Loan repayment", "-"),
        FinanceKind::LoanInterest => ("Loan interest", "-"),
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
        FinanceView::ParkSummary | FinanceView::Loans => 1,
    }
    .max(1);
    let current = view.finance_page.min(pages - 1);
    let delta = std::mem::take(&mut view.finance_page_pending);
    view.finance_page = (current as i64 + i64::from(delta)).clamp(0, (pages - 1) as i64) as usize;
    (view.finance_page, pages)
}

type FinancePanels<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<FinancePanel>,
        Without<FinanceLoanControls>,
        Without<FinancePaging>,
    ),
>;
type FinanceRows<'w, 's> = Query<
    'w,
    's,
    (&'static FinanceRow, &'static mut Node),
    (
        Without<FinancePanel>,
        Without<FinanceLoanControls>,
        Without<FinancePaging>,
    ),
>;
type FinanceControls<'w, 's> = Query<
    'w,
    's,
    (&'static mut Node, Has<FinanceLoanControls>),
    Or<(With<FinanceLoanControls>, With<FinancePaging>)>,
>;

pub(crate) fn refresh(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    mut panels: FinancePanels<'_, '_>,
    mut rows: FinanceRows<'_, '_>,
    mut labels: Query<(&FinanceLabel, &mut Text)>,
    mut controls: FinanceControls<'_, '_>,
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
            FinanceView::ParkSummary => FinanceView::Loans,
            FinanceView::Loans => FinanceView::Transactions,
        };
        view.finance_page = 0;
        view.finance_page_pending = 0;
    }
    let rides = view.finance_view == FinanceView::Rides;
    let summary = view.finance_view == FinanceView::ParkSummary;
    let loans = view.finance_view == FinanceView::Loans;
    for (mut node, is_loan) in &mut controls {
        node.display = if is_loan == loans {
            Display::Flex
        } else {
            Display::None
        };
    }
    let double_rows = rides || summary || loans;
    for (row, mut node) in &mut rows {
        node.display = if double_rows
            && row.0
                >= if summary {
                    SUMMARY_PAGE_SIZE
                } else {
                    RIDE_PAGE_SIZE
                } {
            Display::None
        } else {
            Display::Flex
        };
        node.height = px(if summary {
            36
        } else if double_rows {
            44
        } else {
            22
        });
    }
    let (page, pages) = page(&sim.0, &mut view);
    let stats = sim.0.finance_stats();
    let ledger = sim.0.finance_ledger();
    let cashflows = sim.0.ride_cashflows();
    let summary_rows = summary_rows(&sim.0);
    let loan = sim.0.loan_stats();
    let loan_rows = loan_rows(&sim.0);
    for (label, mut text) in &mut labels {
        let value = match label {
            FinanceLabel::View if loans=>"Transactions".into(),
            FinanceLabel::ListHeading if loans=>"LOAN ACCOUNT / funding is separate from operating income".into(),
            FinanceLabel::Categories if loans=>"Borrow/repay in $100 steps, up to $2000 outstanding.\nSimple interest: 0.5% per simulation minute, prorated each tick.".into(),
            FinanceLabel::History if loans=>format!("Interest is collected before ride/facility costs every 60 simulation seconds.\nNext collection in {} simulation seconds / current rate {} per simulation minute.\nUnpaid interest remains due without compounding; Space pauses accrual.",loan.next_collection_ticks.div_ceil(TICKS_PER_SECOND),money(loan.interest_per_minute_cents)),
            FinanceLabel::Row(i) if loans=>loan_rows.get(*i).cloned().unwrap_or_default(),
            FinanceLabel::Footer if loans=>view.finance_loan_notice.clone().unwrap_or_else(||"Repayment also settles accrued whole-cent interest; leave cash for running costs.".into()),
            FinanceLabel::ListHeading if summary => "PARK SUMMARY / lifetime operating results and cash reconciliation".into(),
            FinanceLabel::View if summary => "Loans".into(),
            FinanceLabel::Footer if summary => "Operating profit excludes borrowing, repayments and interest. Removed histories remain.".into(),
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
                signed_cash(stats.net_cash_cents),
                money(stats.demolition_refunds_cents),
                money(stats.open_upkeep_per_minute_cents)
            ),
            FinanceLabel::Categories => format!(
                "Ride tickets +{} / Service sales +{}\nRide upkeep -{} / Facility upkeep -{} / Supplies -{}\nConstruction -{} / Demolition refunds +{} / Interest -{}",
                money(stats.ride_revenue_cents),
                money(stats.service_revenue_cents),
                money(stats.ride_upkeep_cents),money(stats.service_upkeep_cents),
                money(stats.supplies_cents),
                money(stats.construction_cents),
                money(stats.demolition_refunds_cents),
                money(stats.loan_interest_cents)
            ),
            FinanceLabel::History if ledger.archived.loan_borrowed_cents!=0 || ledger.archived.loan_repaid_cents!=0 || ledger.archived.loan_interest_cents!=0 =>format!("Older {} transactions remain included in lifetime totals.\nOlder funding: borrowed +{} / repaid -{} / interest -{}.\nShowing latest {} transactions; financing never counts as customer sales.",compact_count(ledger.archived_entries),money(ledger.archived.loan_borrowed_cents),money(ledger.archived.loan_repaid_cents),money(ledger.archived.loan_interest_cents),ledger.entries.len()),
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
