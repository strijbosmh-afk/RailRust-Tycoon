//! Read-only lifetime ride cashflow, including retained demolished rides.
use crate::{Park, RideId, RideKind};

/// Recorded cash, not a valuation or a forecast. Legacy free construction has
/// zero recorded basis. Running expenses remain attributed after demolition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideCashflow {
    pub ride: RideId,
    pub kind: RideKind,
    pub installed: bool,
    pub open: bool,
    pub ticket_revenue_cents: u64,
    pub construction_cents: u64,
    pub refunds_cents: u64,
    pub upkeep_cents: u64,
    pub closed_for_funds: bool,
    pub net_cash_cents: i128,
}

impl Park {
    pub fn ride_cashflow(&self, id: RideId) -> Option<RideCashflow> {
        let ride = self.ride_by_id(id)?;
        let kind = self.ride_kind(id)?;
        let (construction_cents, refunds_cents) = match kind {
            RideKind::Coaster => (
                self.track_economy.construction_cents,
                self.track_economy.refund_cents,
            ),
            RideKind::Carousel => {
                let carousel = self.carousel(id)?;
                (
                    carousel.construction_cost_cents,
                    carousel.demolition_refund_cents,
                )
            }
        };
        Some(RideCashflow {
            upkeep_cents: self.ride_running_costs(id)?.upkeep_cents,
            closed_for_funds: self.ride_running_costs(id)?.closed_for_funds,
            ride: id,
            kind,
            installed: self.ride_available(id),
            open: self.ride_is_open(id),
            ticket_revenue_cents: ride.revenue_cents,
            construction_cents,
            refunds_cents,
            net_cash_cents: i128::from(ride.revenue_cents) + i128::from(refunds_cents)
                - i128::from(construction_cents)
                - i128::from(self.ride_running_costs(id)?.upkeep_cents),
        })
    }

    /// Stable identity order, including retained construction/demolition history.
    pub fn ride_cashflows(&self) -> Vec<RideCashflow> {
        let mut values = self
            .all_rides()
            .filter_map(|ride| self.ride_cashflow(ride.id))
            .collect::<Vec<_>>();
        values.sort_by_key(|value| value.ride);
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FinanceKind, RideFinanceTotals, Speed, TilePos};

    fn ledger_total(park: &Park, id: RideId) -> (u64, u64, u64, u64) {
        let mut total = (0, 0, 0, 0);
        let ledger = park.finance_ledger();
        for row in ledger
            .opening_rides
            .iter()
            .chain(&ledger.archived_rides)
            .filter(|row| row.ride == id)
        {
            let RideFinanceTotals {
                revenue_cents,
                construction_cents,
                demolition_refunds_cents,
                upkeep_cents,
                ..
            } = *row;
            total.0 += revenue_cents;
            total.1 += construction_cents;
            total.2 += demolition_refunds_cents;
            total.3 += upkeep_cents;
        }
        for row in ledger.entries.iter().filter(|row| row.ride == Some(id)) {
            match row.kind {
                FinanceKind::RideUpkeep => total.3 += row.amount_cents,
                FinanceKind::RideRevenue => total.0 += row.amount_cents,
                FinanceKind::RideConstruction | FinanceKind::TrackConstruction => {
                    total.1 += row.amount_cents
                }
                FinanceKind::RideDemolitionRefund | FinanceKind::TrackRefund => {
                    total.2 += row.amount_cents
                }
                _ => panic!("unexpected service entry with ride owner"),
            }
        }
        total
    }

    #[test]
    fn cashflows_include_paid_removed_rides_free_legacy_track_and_immutable_queries() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let coaster = park.ride().unwrap().id;
        park.set_train_open(false);
        let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        let before = park.clone();
        let reports = park.ride_cashflows();
        assert_eq!(park, before);
        assert_eq!(
            reports.iter().map(|r| r.ride).collect::<Vec<_>>(),
            vec![coaster, carousel]
        );
        assert_eq!(park.ride_cashflow(coaster).unwrap().construction_cents, 0);
        assert_eq!(
            park.ride_cashflow(carousel).unwrap().construction_cents,
            crate::CAROUSEL_BUILD_COST_CENTS
        );
        park.remove_carousel(carousel).unwrap();
        let row = park.ride_cashflow(carousel).unwrap();
        assert!(!row.installed && !row.open);
        assert_eq!(row.refunds_cents, crate::CAROUSEL_BUILD_COST_CENTS / 2);
        assert_eq!(
            row.net_cash_cents,
            -i128::from(crate::CAROUSEL_BUILD_COST_CENTS / 2)
        );
        assert_eq!(
            ledger_total(&park, carousel),
            (
                row.ticket_revenue_cents,
                row.construction_cents,
                row.refunds_cents,
                row.upkeep_cents
            )
        );
        assert!(park.ride_cashflow(RideId(999)).is_none());
        let bytes = park.to_save_bytes().unwrap();
        assert_eq!(
            Park::from_save_bytes(&bytes).unwrap().ride_cashflows(),
            park.ride_cashflows()
        );
    }

    #[test]
    fn two_paid_rides_reconcile_with_opening_archived_and_recent_ledger_after_restart() {
        let (mut park, _) = crate::carousel::tests::fixture(true);
        // A real operating facility rolls the bounded ledger while sharing a
        // numeric ID with a ride. Its costs must never enter a ride's totals.
        park.place_service(
            TilePos { x: 4, y: 5 },
            0,
            crate::PathDirection::North,
            crate::ServiceKind::Toilet,
        )
        .unwrap();
        for _ in 0..24_000 {
            park.advance(Speed::Normal);
        }
        let before = park.clone();
        let rows = park.ride_cashflows();
        assert_eq!(park, before);
        assert!(park.finance_ledger().archived_entries > 0);
        for row in &rows {
            assert!(row.ticket_revenue_cents > 0);
            assert_eq!(
                ledger_total(&park, row.ride),
                (
                    row.ticket_revenue_cents,
                    row.construction_cents,
                    row.refunds_cents,
                    row.upkeep_cents
                )
            );
            assert_eq!(
                row.net_cash_cents,
                i128::from(row.ticket_revenue_cents) + i128::from(row.refunds_cents)
                    - i128::from(row.construction_cents)
                    - i128::from(row.upkeep_cents)
            );
        }
        assert_eq!(
            rows.iter().map(|r| r.ticket_revenue_cents).sum::<u64>(),
            park.finance_stats().ride_revenue_cents
        );
        let loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(loaded.ride_cashflows(), rows);
    }
}
