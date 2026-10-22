//! Compact displays use integer game cents. The ticket itself is always exact.
use park_sim::{Park, RideEconomyStats, RideExperienceStats};

pub(crate) fn money(cents: u64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

fn compact_money(cents: u64) -> String {
    if cents > 99_999_999 {
        // A lower bound rather than rounding an enormous saved balance into
        // an apparently exact number; keeps the minimum-window header bounded.
        "$999999+".into()
    } else {
        money(cents)
    }
}

pub(super) fn header(park: &Park, stats: Option<&RideEconomyStats>) -> String {
    let wallets = park.guests().iter().map(|guest| guest.wallet_cents).fold(
        None::<(u32, u32)>,
        |range, wallet| {
            Some(range.map_or((wallet, wallet), |(low, high)| {
                (low.min(wallet), high.max(wallet))
            }))
        },
    );
    let wallets = match wallets {
        None => "none".into(),
        Some((low, high)) if low == high => compact_money(u64::from(low)),
        Some((low, high)) => format!(
            "{}-{}",
            compact_money(u64::from(low)),
            compact_money(u64::from(high))
        ),
    };
    format!(
        "Cash {} / Revenue {} / Wallets {wallets} / Can pay {}/{}",
        compact_money(park.cash_cents()),
        compact_money(stats.map_or(0, |stats| stats.revenue_cents)),
        super::compact_count(stats.map_or(0, |stats| stats.affordable_guests) as u64),
        super::compact_count(park.guests().len() as u64),
    )
}

pub(super) fn ticket(
    stats: Option<&RideEconomyStats>,
    experience: Option<&RideExperienceStats>,
) -> String {
    stats.map_or_else(
        || "No ride\nTicket unavailable".into(),
        |stats| {
            format!(
                "Ticket {}\n{} interested",
                money(u64::from(stats.ticket_price_cents)),
                super::compact_count(experience.map_or(0, |stats| stats.interested_guests) as u64),
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_currency_is_exact_and_large_header_balances_are_bounded() {
        assert_eq!(money(0), "$0.00");
        assert_eq!(money(1), "$0.01");
        assert_eq!(money(50), "$0.50");
        assert_eq!(money(10_000), "$100.00");
        assert_eq!(money(u64::MAX), "$184467440737095516.15");
        assert_eq!(compact_money(99_999_999), "$999999.99");
        assert_eq!(compact_money(u64::MAX), "$999999+");
    }
}
