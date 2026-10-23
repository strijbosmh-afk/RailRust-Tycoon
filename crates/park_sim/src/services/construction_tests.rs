use super::*;
use crate::{FinanceKind, FinanceLedger, Speed};
use serde_json::{Value, json};

const POS: TilePos = TilePos { x: 4, y: 2 };

fn saved(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let restored = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &restored);
    assert_eq!(bytes, restored.to_save_bytes().unwrap());
    restored
}
fn read(value: &Value) -> Result<Park, crate::SaveError> {
    Park::from_save_bytes(&serde_json::to_vec(value).unwrap())
}
fn value(park: &Park) -> Value {
    serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap()
}
fn advance(park: &mut Park, ticks: u64) {
    for _ in 0..ticks {
        park.advance(Speed::Normal);
    }
}
fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
    for _ in 0..30_000 {
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!("service construction fixture did not reach its state");
}
fn customer_fixture() -> (Park, ServiceId) {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    let id = park
        .place_service(
            TilePos { x: 10, y: 21 },
            8,
            PathDirection::North,
            ServiceKind::Food,
        )
        .unwrap();
    (park, id)
}

#[test]
fn every_kind_has_an_immutable_quote_and_exact_paused_purchase_and_once_only_refund() {
    for (kind, cost) in [
        (ServiceKind::Food, 12_000),
        (ServiceKind::Drink, 8_000),
        (ServiceKind::Toilet, 10_000),
    ] {
        let mut park = Park::new(32, 32);
        let before = park.clone();
        let cache = park.path_cache_stats();
        assert_eq!(kind.build_cost_cents(), cost);
        assert_eq!(
            park.service_build_quote(POS, 0, PathDirection::North, kind),
            Ok(cost)
        );
        assert_eq!(park, before);
        assert_eq!(park.path_cache_stats(), cache);
        let id = park
            .place_service(POS, 0, PathDirection::North, kind)
            .unwrap();
        let service = park.service(id).unwrap();
        let object = service.object.unwrap();
        assert_eq!(
            (
                service.construction_cost_cents,
                service.demolition_refund_cents
            ),
            (cost, 0)
        );
        assert_eq!(park.cash_cents(), 100_000 - cost);
        let entry = &park.finance_ledger().entries[0];
        assert_eq!(
            (
                entry.kind,
                entry.service,
                entry.ride,
                entry.tick,
                entry.amount_cents
            ),
            (FinanceKind::ServiceConstruction, Some(id), None, 0, cost)
        );
        let built = saved(&park);
        assert_eq!(park.service_demolition_quote(id), Ok(cost / 2));
        assert_eq!(park, built);
        park.remove_object(object).unwrap();
        assert_eq!(park.cash_cents(), 100_000 - cost / 2);
        assert_eq!(
            park.finance_ledger().entries[1].kind,
            FinanceKind::ServiceDemolitionRefund
        );
        assert_eq!(park.service_stats(id).profit_cents, -(cost as i64 / 2));
        assert_eq!(park.finance_stats().net_profit_cents, -(cost as i64 / 2));
        let removed = saved(&park);
        assert_eq!(
            park.service_demolition_quote(id),
            Err(ServiceError::RemovedService(id))
        );
        assert!(park.remove_object(object).is_none());
        assert_eq!(park, removed);
        assert_eq!(park.guests, before.guests);
        assert_eq!(park.crowd_rng, before.crowd_rng);
    }
}

#[test]
fn exact_budget_build_succeeds_and_a_stale_quote_cannot_spend_missing_cash() {
    for kind in ServiceKind::ALL {
        let mut park = Park::new(32, 32);
        let drain = crate::finances::tests::place_legacy_service(
            &mut park,
            TilePos { x: 2, y: 2 },
            0,
            PathDirection::North,
            ServiceKind::Toilet,
        );
        let cost = kind.build_cost_cents();
        // Valid fixed-cost history isolates the exact cash boundary without
        // hundreds of thousands of unrelated guest decision updates.
        while park.cash_cents > cost {
            park.ticks += 90;
            park.services[0].upkeep_cents += 10;
            park.record_cash(FinanceKind::ServiceUpkeep, Some(drain), 10);
        }
        assert_eq!(
            park.service_build_quote(POS, 0, PathDirection::North, kind),
            Ok(cost)
        );
        let exact = saved(&park);
        park.ticks += 90;
        park.services[0].upkeep_cents += 10;
        park.record_cash(FinanceKind::ServiceUpkeep, Some(drain), 10);
        let before = saved(&park);
        let cache = park.path_cache_stats();
        let error = PlacementError::InsufficientFunds {
            required_cents: cost,
            available_cents: cost - 10,
        };
        assert_eq!(
            park.service_build_quote(POS, 0, PathDirection::North, kind),
            Err(error)
        );
        assert_eq!(
            park.place_service(POS, 0, PathDirection::North, kind),
            Err(error)
        );
        assert_eq!(park, before);
        assert_eq!(park.path_cache_stats(), cache);
        let mut exact = exact;
        let id = exact
            .place_service(POS, 0, PathDirection::North, kind)
            .unwrap();
        assert_eq!(exact.cash_cents(), 0);
        let mut restored = saved(&exact);
        restored
            .remove_object(restored.service(id).unwrap().object.unwrap())
            .unwrap();
        assert_eq!(restored.cash_cents(), cost / 2);
        saved(&restored);
    }
}

#[test]
fn rejected_footprint_identity_and_capacity_edits_do_not_charge_or_mutate_allocators() {
    let mut park = Park::new(32, 32);
    let id = park
        .place_service(POS, 0, PathDirection::North, ServiceKind::Food)
        .unwrap();
    let before = park.clone();
    for (pos, height) in [
        (POS, 0),
        (TilePos { x: 32, y: 2 }, 0),
        (TilePos { x: 8, y: 2 }, 30),
    ] {
        assert!(
            park.place_service(pos, height, PathDirection::East, ServiceKind::Drink)
                .is_err()
        );
        assert_eq!(park, before);
    }
    assert_eq!(
        park.service_demolition_quote(ServiceId(id.0 + 1)),
        Err(ServiceError::UnknownService(ServiceId(id.0 + 1)))
    );
    let mut exhausted = park.clone();
    exhausted.next_service_id = u64::MAX;
    let before = exhausted.clone();
    assert_eq!(
        exhausted.place_service(
            TilePos { x: 8, y: 2 },
            0,
            PathDirection::East,
            ServiceKind::Drink
        ),
        Err(PlacementError::IdExhausted)
    );
    assert_eq!(exhausted, before);
}

#[test]
fn demolition_cancels_an_unfinished_purchase_free_and_preserves_public_position() {
    let (mut park, id) = customer_fixture();
    seek(&mut park, |p| {
        p.guests.iter().any(|g| {
            g.service_visit.as_ref().is_some_and(|v| {
                v.service == id && v.phase == ServiceVisitPhase::Using && v.progress == 89
            })
        })
    });
    let index = park
        .guests
        .iter()
        .position(|g| {
            g.service_visit
                .as_ref()
                .is_some_and(|v| v.service == id && v.phase == ServiceVisitPhase::Using)
        })
        .unwrap();
    let guest = park.guests[index].clone();
    let point = park.guest_position(guest.id).unwrap();
    let before = saved(&park);
    let object = park.service(id).unwrap().object.unwrap();
    let refund = park.service_demolition_quote(id).unwrap();
    assert_eq!(refund, 6_000);
    assert_eq!(park, before);
    park.remove_object(object).unwrap();
    let after = &park.guests[index];
    assert!(after.service_visit.is_none());
    assert_eq!(
        (
            after.wallet_cents,
            after.spent_cents,
            after.services_used,
            after.needs
        ),
        (
            guest.wallet_cents,
            guest.spent_cents,
            guest.services_used,
            guest.needs
        )
    );
    assert_eq!(park.guest_position(guest.id).unwrap().point, point.point);
    assert_eq!(
        park.service(id).unwrap().served,
        before.service(id).unwrap().served
    );
    assert_eq!(
        park.service(id).unwrap().revenue_cents,
        before.service(id).unwrap().revenue_cents
    );
    assert_eq!(park.cash_cents(), before.cash_cents() + refund);
    assert_eq!(park.ticks, before.ticks);
    let removed = park.service(id).unwrap().clone();
    let mut restored = saved(&park);
    advance(&mut restored, 1000);
    assert_eq!(restored.service(id).unwrap(), &removed);
    saved(&restored);
}

#[test]
fn replacement_history_and_global_totals_include_rides_without_cash_generation() {
    let mut park = Park::new(32, 32);
    let ride = park.place_carousel(TilePos { x: 19, y: 22 }, 0).unwrap();
    let mut previous = None;
    for index in 1..=15 {
        if park.cash_cents() < 8_000 {
            break;
        }
        let id = park
            .place_service(POS, 0, PathDirection::North, ServiceKind::Drink)
            .unwrap();
        assert!(previous.is_none_or(|old| id > old));
        park.remove_object(park.service(id).unwrap().object.unwrap())
            .unwrap();
        assert_eq!(park.cash_cents(), 70_000 - index * 4_000);
        previous = Some(id);
    }
    park.remove_carousel(ride).unwrap();
    let stats = park.finance_stats();
    assert_eq!(
        stats.construction_cents,
        30_000 + park.services.len() as u64 * 8_000
    );
    assert_eq!(
        stats.demolition_refunds_cents,
        15_000 + park.services.len() as u64 * 4_000
    );
    assert_eq!(stats.ride_revenue_cents + stats.service_revenue_cents, 0);
    assert_eq!(stats.net_profit_cents, park.cash_cents() as i64 - 100_000);
    saved(&park);
}

#[test]
fn archives_preserve_service_capital_at_tick_zero_and_after_demolition() {
    let mut park = Park::new(32, 32);
    let id = park
        .place_service(POS, 0, PathDirection::North, ServiceKind::Drink)
        .unwrap();
    advance(&mut park, 128 * 90);
    assert_eq!(park.finances.archived_entries, 1);
    assert_eq!(park.finances.archived_through_tick, 0);
    assert_eq!(park.finances.archived.construction_cents, 8_000);
    assert_eq!(park.finances.archived_services[0].construction_cents, 8_000);
    let mut park = saved(&park);
    park.remove_object(park.service(id).unwrap().object.unwrap())
        .unwrap();
    let fresh = park
        .place_service(POS, 0, PathDirection::North, ServiceKind::Drink)
        .unwrap();
    advance(&mut park, 128 * 90);
    assert_eq!(
        park.finances.archived_services[0].demolition_refunds_cents,
        4_000
    );
    assert_eq!(park.finances.archived.construction_cents, 16_000);
    assert_eq!(park.service(fresh).unwrap().demolition_refund_cents, 0);
    saved(&park);
    let valid = value(&park);
    for (path, invalid) in [
        (
            "/park/finances/archived_services/0/construction_cents",
            json!(0),
        ),
        (
            "/park/finances/archived_services/0/demolition_refunds_cents",
            json!(8_000),
        ),
        ("/park/finances/archived/construction_cents", json!(8_000)),
        ("/park/finances/archived/demolition_refunds_cents", json!(0)),
    ] {
        let mut bad = valid.clone();
        *bad.pointer_mut(path).unwrap() = invalid;
        assert!(read(&bad).is_err(), "{path}");
    }
}

#[test]
fn paid_service_cannot_launder_archived_income_or_use_counts_through_opening_history() {
    let (mut park, id) = customer_fixture();
    seek(&mut park, |p| {
        p.finances
            .archived_services
            .iter()
            .any(|r| r.service == id && r.revenue_cents > 1)
    });
    saved(&park);
    let source = park
        .finances
        .archived_services
        .iter()
        .find(|r| r.service == id)
        .unwrap();
    assert_eq!(source.construction_cents, 12_000);
    assert!(park.service(id).unwrap().served > 0);
    let mut forged = park.clone();
    forged
        .finances
        .opening_services
        .push(crate::ServiceFinanceOpening {
            service: id,
            revenue_cents: 1,
            served: 1,
        });
    forged.finances.opening_cash_cents += 1;
    forged.finances.archived.service_revenue_cents -= 1;
    forged.finances.archived_services[0].revenue_cents -= 1;
    assert_eq!(
        crate::finances::validate_finances(&forged),
        Err("opening service history is inconsistent".into())
    );
    assert_eq!(forged.guests, park.guests);
    assert_eq!(forged.cash_cents, park.cash_cents);
    assert_eq!(forged.finances.entries, park.finances.entries);
    assert!(forged.to_save_bytes().is_err());
}

#[test]
fn required_basis_refund_and_transaction_chronology_reject_corrupt_saves() {
    let mut park = Park::new(32, 32);
    let id = park
        .place_service(POS, 0, PathDirection::North, ServiceKind::Food)
        .unwrap();
    let built = value(&park);
    for (path, invalid) in [
        ("/park/services/0/construction_cost_cents", json!(0)),
        ("/park/services/0/construction_cost_cents", json!(8_000)),
        ("/park/services/0/demolition_refund_cents", json!(6_000)),
        ("/park/finances/entries/0/amount_cents", json!(6_000)),
        ("/park/finances/entries/0/kind", json!("ServiceRevenue")),
        ("/park/finances/entries/0/service", json!(99)),
        ("/park/finances/entries/0/ride", json!(1)),
    ] {
        let mut bad = built.clone();
        *bad.pointer_mut(path).unwrap() = invalid;
        assert!(read(&bad).is_err(), "{path}");
    }
    for key in ["construction_cost_cents", "demolition_refund_cents"] {
        let mut bad = built.clone();
        bad["park"]["services"][0]
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(read(&bad).is_err(), "{key}");
    }
    park.remove_object(park.service(id).unwrap().object.unwrap())
        .unwrap();
    let mut bad = value(&park);
    bad["park"]["finances"]["entries"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    bad["park"]["finances"]["entries"][0]["id"] = json!(1);
    bad["park"]["finances"]["entries"][0]["balance_cents"] = json!(106_000);
    bad["park"]["finances"]["entries"][1]["id"] = json!(2);
    bad["park"]["finances"]["entries"][1]["balance_cents"] = json!(94_000);
    assert!(
        read(&bad).is_err(),
        "balance-correct refund must not precede construction"
    );
}

#[test]
fn schema_eleven_keeps_paid_rides_and_free_services_without_retroactive_refunds() {
    let mut legacy = Park::new(32, 32);
    let ride = legacy.place_carousel(TilePos { x: 19, y: 22 }, 0).unwrap();
    let food = crate::finances::tests::place_legacy_service(
        &mut legacy,
        POS,
        0,
        PathDirection::North,
        ServiceKind::Food,
    );
    let drink = crate::finances::tests::place_legacy_service(
        &mut legacy,
        TilePos { x: 5, y: 2 },
        0,
        PathDirection::North,
        ServiceKind::Drink,
    );
    legacy
        .remove_object(legacy.service(drink).unwrap().object.unwrap())
        .unwrap();
    advance(&mut legacy, 140 * 90);
    legacy.remove_carousel(ride).unwrap();
    let mut old = value(&legacy);
    crate::finances::tests::strip_service_construction_fields(&mut old);
    old["schema_version"] = json!(11);
    old["content_version"] = json!(11);
    let mut migrated = read(&old).unwrap();
    assert_eq!(migrated, legacy);
    assert_eq!(
        migrated.carousel(ride).unwrap().demolition_refund_cents,
        15_000
    );
    assert_eq!(migrated.service_demolition_quote(food), Ok(0));
    let cash = migrated.cash_cents;
    let ledger = migrated.finances.clone();
    migrated
        .remove_object(migrated.service(food).unwrap().object.unwrap())
        .unwrap();
    assert_eq!(migrated.cash_cents, cash);
    assert_eq!(migrated.finances, ledger);
    let paid = migrated
        .place_service(POS, 0, PathDirection::North, ServiceKind::Food)
        .unwrap();
    assert!(paid > drink);
    assert_eq!(migrated.service_demolition_quote(paid), Ok(6_000));
    saved(&migrated);
    for field in ["construction_cost_cents", "demolition_refund_cents"] {
        let mut bad = old.clone();
        bad["park"]["services"][0][field] = json!(0);
        assert!(read(&bad).is_err(), "v11 accepted {field}");
    }
    for field in ["construction_cents", "demolition_refunds_cents"] {
        let mut bad = old.clone();
        bad["park"]["finances"]["archived_services"][0][field] = json!(0);
        assert!(read(&bad).is_err(), "v11 accepted {field}");
    }
    for kind in ["ServiceConstruction", "ServiceDemolitionRefund"] {
        let mut bad = old.clone();
        bad["park"]["finances"]["entries"][0]["kind"] = json!(kind);
        assert!(read(&bad).is_err(), "v11 accepted {kind}");
    }
}

#[test]
fn using_legacy_service_continues_with_zero_basis_and_a_new_paid_service_after_migration() {
    let (mut legacy, id) = customer_fixture();
    legacy.services[0].construction_cost_cents = 0;
    legacy.cash_cents += ServiceKind::Food.build_cost_cents();
    legacy.finances = FinanceLedger::default();
    seek(&mut legacy, |p| {
        p.guests.iter().any(|g| {
            g.service_visit
                .as_ref()
                .is_some_and(|v| v.phase == ServiceVisitPhase::Using && v.progress == 45)
        })
    });
    let mut old = value(&legacy);
    crate::finances::tests::strip_service_construction_fields(&mut old);
    old["schema_version"] = json!(11);
    old["content_version"] = json!(11);
    let mut migrated = read(&old).unwrap();
    assert_eq!(migrated, legacy);
    assert_eq!(migrated.service_demolition_quote(id), Ok(0));
    for _ in 0..3600 {
        legacy.advance(Speed::Normal);
        migrated.advance(Speed::Normal);
        assert_eq!(migrated, legacy);
    }
    let fresh = migrated
        .place_service(POS, 0, PathDirection::North, ServiceKind::Drink)
        .unwrap();
    assert_eq!(
        migrated.service(fresh).unwrap().construction_cost_cents,
        8_000
    );
    saved(&migrated);
}
