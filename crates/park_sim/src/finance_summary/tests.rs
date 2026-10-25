use super::*;
use crate::{FinanceKind, PathDirection, ServiceKind, Speed, TilePos};

#[test]
fn capital_purchases_and_refunds_never_become_operating_profit() {
    let mut park = Park::new(32, 32);
    let empty = park.finance_summary();
    assert_eq!(empty.operating_profit_cents, 0);
    assert_eq!(empty.net_cash_cents, 0);
    let ride = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    let service = park
        .place_service(
            TilePos { x: 4, y: 5 },
            0,
            PathDirection::North,
            ServiceKind::Drink,
        )
        .unwrap();
    assert_eq!(ride.0, service.0); // Distinct owner namespaces.
    park.remove_carousel(ride).unwrap();
    park.remove_object(park.service(service).unwrap().object.unwrap())
        .unwrap();
    let report = park.finance_summary();
    assert_eq!(report.rides, OperatingProfit::default());
    assert_eq!(report.services, OperatingProfit::default());
    assert_eq!(report.operating_profit_cents, 0);
    assert_eq!(report.construction_cents, 38_000);
    assert_eq!(report.refunds_cents, 19_000);
    assert_eq!(report.net_cash_cents, -19_000);
    assert_eq!(report.cash_cents, 81_000);
}

#[test]
fn archived_operating_loss_survives_closure_removal_and_restart_without_query_mutation() {
    let mut park = Park::new(32, 32);
    let id = park
        .place_service(
            TilePos { x: 4, y: 5 },
            0,
            PathDirection::North,
            ServiceKind::Toilet,
        )
        .unwrap();
    for _ in 0..12_600 {
        park.advance(Speed::Normal);
    }
    assert!(park.finance_ledger().archived_entries > 0);
    assert_eq!(park.finance_summary().services.profit_cents, -1400);
    park.set_service_open(id, false).unwrap();
    let operating = park.finance_summary().services;
    park.remove_object(park.service(id).unwrap().object.unwrap())
        .unwrap();
    let before = park.to_save_bytes().unwrap();
    let report = park.finance_summary();
    assert_eq!(report.services, operating);
    assert_eq!(report.operating_profit_cents, -1400);
    assert_eq!(report.net_cash_cents, -6400);
    assert_eq!(park.to_save_bytes().unwrap(), before);
    let mut loaded = Park::from_save_bytes(&before).unwrap();
    for _ in 0..900 {
        loaded.advance(Speed::Normal);
    }
    assert_eq!(loaded.finance_summary(), report);
}

#[test]
fn paid_ride_and_service_summary_matches_independent_ledger_buckets_after_rollover() {
    let (mut park, _) = crate::carousel::tests::fixture(true);
    park.place_service(
        TilePos { x: 15, y: 21 },
        8,
        PathDirection::North,
        ServiceKind::Drink,
    )
    .unwrap();
    for _ in 0..24_000 {
        park.advance(Speed::Normal);
    }
    let ledger = park.finance_ledger();
    assert!(ledger.archived_entries > 0);
    let mut ride_revenue = ledger.opening_ride_revenue_cents + ledger.archived.ride_revenue_cents;
    let mut service_revenue = ledger
        .opening_services
        .iter()
        .map(|s| s.revenue_cents)
        .sum::<u64>()
        + ledger.archived.service_revenue_cents;
    let mut ride_cost = ledger
        .archived_rides
        .iter()
        .map(|r| r.upkeep_cents)
        .sum::<u64>();
    let mut service_cost = ledger.archived.upkeep_cents - ride_cost;
    let mut supplies = ledger.archived.supplies_cents;
    for row in &ledger.entries {
        match row.kind {
            FinanceKind::RideRevenue => ride_revenue += row.amount_cents,
            FinanceKind::RideUpkeep => ride_cost += row.amount_cents,
            FinanceKind::ServiceRevenue => service_revenue += row.amount_cents,
            FinanceKind::ServiceUpkeep => service_cost += row.amount_cents,
            FinanceKind::ServiceSupplies => supplies += row.amount_cents,
            _ => {}
        }
    }
    assert!(ride_revenue > 0 && ride_cost > 0 && service_revenue > 0 && supplies > 0);
    let report = park.finance_summary();
    assert_eq!(
        report.rides.profit_cents,
        i128::from(ride_revenue) - i128::from(ride_cost)
    );
    assert_eq!(
        report.services.profit_cents,
        i128::from(service_revenue) - i128::from(service_cost) - i128::from(supplies)
    );
    assert_eq!(
        i128::from(report.starting_cash_cents) + report.net_cash_cents,
        i128::from(report.cash_cents)
    );
    assert_eq!(
        Park::from_save_bytes(&park.to_save_bytes().unwrap())
            .unwrap()
            .finance_summary(),
        report
    );
}

#[test]
fn authentic_schema_19_opening_preserves_income_without_inventing_operating_expenses() {
    let bytes = include_bytes!("../../testdata/v19-paid-opening.park.json");
    let mut park = Park::from_save_bytes(bytes).unwrap();
    let before = park.finance_summary();
    assert!(before.rides.revenue_cents > 0);
    assert_eq!(before.rides.upkeep_cents, 0);
    assert_eq!(
        before.rides.profit_cents,
        i128::from(before.rides.revenue_cents)
    );
    let current = park.to_save_bytes().unwrap();
    assert_eq!(
        Park::from_save_bytes(&current).unwrap().finance_summary(),
        before
    );
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    // This authentic opening park has both a coaster and a carousel open.
    assert_eq!(park.finance_summary().rides.upkeep_cents, 150);
}
