//! Read-only financial history, shown newest first without changing ledger order.
use park_sim::{FinanceEntry, FinanceKind, Park};

use super::*;

pub(crate) const PANEL_WIDTH: f32 = 760.0;
pub(crate) const PANEL_HEIGHT: f32 = 520.0;
pub(crate) const PAGE_SIZE: usize = 8;

#[derive(Component)]
pub(crate) struct FinancePanel;

#[derive(Component)]
pub(crate) enum FinanceLabel {
    Summary,
    Categories,
    History,
    Row(usize),
    Page,
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
    commands.spawn((FinancePanel, GlobalZIndex(9), Node {
        display: Display::None, position_type: PositionType::Absolute,
        width: percent(100), height: percent(100), align_items: AlignItems::Center,
        justify_content: JustifyContent::Center, ..default()
    }, BackgroundColor(Color::srgba(0.04, 0.12, 0.09, 0.65))))
    .with_children(|p| {
        p.spawn((Node {
            width: px(PANEL_WIDTH), height: px(PANEL_HEIGHT), padding: UiRect::all(px(20)),
            flex_direction: FlexDirection::Column, row_gap: px(5), ..default()
        }, BackgroundColor(PAPER), BorderRadius::all(px(10))))
        .with_children(|p| {
            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..row() }).with_children(|p| {
                label(p, "Park finances", 23.0, INK);
                button(p, "Close [N/Esc]", Action::CloseFinance, 140.0);
            });
            text(p, "Cash / Income / Costs / Net", 14.0, FinanceLabel::Summary);
            text(p, "Ride tickets / Service sales / Upkeep / Supplies / Construction / Refunds", 11.0, FinanceLabel::Categories);
            p.spawn(Node { height: px(52), flex_shrink: 0.0, ..default() }).with_children(|p| {
                text(p, "", 11.0, FinanceLabel::History);
            });
            p.spawn(Node { height: px(208), flex_shrink: 0.0, flex_direction: FlexDirection::Column,
                row_gap: px(2), ..default() }).with_children(|p| {
                label(p, "RECENT TRANSACTIONS / newest first / resulting cash at right", 11.0, MUTED);
                for index in 0..PAGE_SIZE {
                    p.spawn(Node { height: px(22), flex_shrink: 0.0, ..default() }).with_children(|p| {
                        text(p, "", 11.0, FinanceLabel::Row(index));
                    });
                }
            });
            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..row() }).with_children(|p| {
                button(p, "Newer [PgUp]", Action::FinancePage(-1), 140.0);
                text(p, "Page 1 / 1", 12.0, FinanceLabel::Page);
                button(p, "Older [PgDn]", Action::FinancePage(1), 140.0);
            });
            label(p, "Open facilities cost money even without customers. Space pauses all costs and activity.", 11.0, MUTED);
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

fn clock(tick: u64) -> String {
    let seconds = tick / TICKS_PER_SECOND;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn row_text(entry: &FinanceEntry) -> String {
    let (label, sign) = match entry.kind {
        FinanceKind::RideRevenue => ("Ride tickets", "+"),
        FinanceKind::ServiceRevenue => ("Service sales", "+"),
        FinanceKind::ServiceUpkeep => ("Service upkeep", "-"),
        FinanceKind::ServiceSupplies => ("Service supplies", "-"),
        FinanceKind::RideConstruction => ("Ride construction", "-"),
        FinanceKind::RideDemolitionRefund => ("Demolition refund", "+"),
    };
    let service = entry.ride.map_or_else(
        || {
            entry
                .service
                .map_or(String::new(), |id| format!(" #{}", id.0))
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
    let pages = park
        .finance_ledger()
        .entries
        .len()
        .div_ceil(PAGE_SIZE)
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
        return;
    }
    let (page, pages) = page(&sim.0, &mut view);
    let stats = sim.0.finance_stats();
    let ledger = sim.0.finance_ledger();
    for (label, mut text) in &mut labels {
        let value = match label {
            FinanceLabel::Summary => format!(
                "Cash {} / Sales +{} / Costs -{} / Net cash {}\nRefunds +{} / Open facilities: {} per simulation minute",
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
                "Ride tickets +{} / Service sales +{} / Upkeep -{} / Supplies -{}\nRide construction -{} / Demolition refunds +{}",
                money(stats.ride_revenue_cents),
                money(stats.service_revenue_cents),
                money(stats.upkeep_cents),
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
        for (kind, sign) in [
            (FinanceKind::RideRevenue, "+"),
            (FinanceKind::ServiceRevenue, "+"),
            (FinanceKind::ServiceUpkeep, "-"),
            (FinanceKind::ServiceSupplies, "-"),
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
            assert!(text.contains(&format!(" #7 / {sign}$0.60 / Cash $12.34")));
            assert_eq!(entry.amount_cents, 60);
        }
        assert_eq!(signed_money(-123), "-$1.23");
        assert_eq!(signed_money(0), "+$0.00");
        assert_eq!(signed_money(i64::MIN), "-$92233720368547758.08");
        for (kind, label, sign) in [
            (FinanceKind::RideRevenue, "Ride tickets", "+"),
            (FinanceKind::RideConstruction, "Ride construction", "-"),
            (FinanceKind::RideDemolitionRefund, "Demolition refund", "+"),
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
