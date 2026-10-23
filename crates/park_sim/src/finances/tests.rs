use super::*;
use crate::{PathDirection, ServiceKind, ServiceVisitPhase, Speed, TilePos};
use serde_json::{Value, json};

fn advance(park: &mut Park, ticks: u64) {
    for _ in 0..ticks {
        park.advance(Speed::Normal);
    }
}
fn roundtrip(park: &Park) -> Park {
    validate_finances(park).unwrap();
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(bytes, loaded.to_save_bytes().unwrap());
    loaded
}
fn site(park: &mut Park, x: u32, kind: ServiceKind) -> ServiceId {
    park.place_service(TilePos { x, y: 2 }, 0, PathDirection::North, kind)
        .unwrap()
}
fn using_fixture() -> (Park, usize, ServiceId) {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    let id = park
        .place_service(
            TilePos { x: 18, y: 21 },
            8,
            PathDirection::North,
            ServiceKind::Toilet,
        )
        .unwrap();
    for _ in 0..12_000 {
        if let Some(index) = park.guests.iter().position(|guest| {
            guest.service_visit.as_ref().is_some_and(|visit| {
                visit.phase == ServiceVisitPhase::Using && visit.progress == 89
            })
        }) {
            return (park, index, id);
        }
        park.advance(Speed::Normal);
    }
    panic!("natural toilet use did not begin");
}

/// Test-only projection of a current park into the preceding strict DTO.
pub(crate) fn strip_finances(mut value: Value) -> Value {
    let net_capital = value["park"]["services"]
        .as_array()
        .unwrap()
        .iter()
        .map(|service| {
            service["construction_cost_cents"].as_u64().unwrap()
                - service["demolition_refund_cents"].as_u64().unwrap()
        })
        .sum::<u64>();
    value["park"]["cash_cents"] =
        json!(value["park"]["cash_cents"].as_u64().unwrap() + net_capital);
    strip_carousel_fields(&mut value);
    let costs = value["park"]["services"]
        .as_array()
        .unwrap()
        .iter()
        .map(|service| {
            service["upkeep_cents"].as_u64().unwrap() + service["supplies_cents"].as_u64().unwrap()
        })
        .sum::<u64>();
    let cash = value["park"]["cash_cents"].as_u64().unwrap();
    value["park"]["cash_cents"] = json!(cash + costs);
    value["park"].as_object_mut().unwrap().remove("finances");
    for service in value["park"]["services"].as_array_mut().unwrap() {
        for key in [
            "upkeep_progress",
            "upkeep_cents",
            "supplies_cents",
            "closed_for_funds",
        ] {
            service.as_object_mut().unwrap().remove(key);
        }
    }
    value["schema_version"] = json!(8);
    value["content_version"] = json!(8);
    value
}
pub(crate) fn strip_carousel_fields(value: &mut Value) {
    strip_construction_fields(value);
    value["park"].as_object_mut().unwrap().remove("carousels");
    for guest in value["park"]["guests"].as_array_mut().unwrap() {
        guest.as_object_mut().unwrap().remove("ride_history");
    }
    if let Some(ledger) = value["park"]["finances"].as_object_mut() {
        ledger.remove("opening_rides");
        ledger.remove("archived_rides");
        for entry in ledger["entries"].as_array_mut().unwrap() {
            entry.as_object_mut().unwrap().remove("ride");
        }
    }
}
pub(crate) fn strip_construction_fields(value: &mut Value) {
    strip_service_construction_fields(value);
    for carousel in value["park"]["carousels"].as_array_mut().unwrap() {
        carousel
            .as_object_mut()
            .unwrap()
            .remove("construction_cost_cents");
        carousel
            .as_object_mut()
            .unwrap()
            .remove("demolition_refund_cents");
    }
    if let Some(ledger) = value["park"]["finances"].as_object_mut() {
        for key in ["construction_cents", "demolition_refunds_cents"] {
            ledger["archived"].as_object_mut().unwrap().remove(key);
            for group in ["opening_rides", "archived_rides"] {
                for ride in ledger[group].as_array_mut().unwrap() {
                    ride.as_object_mut().unwrap().remove(key);
                }
            }
        }
    }
}
pub(crate) fn strip_service_construction_fields(value: &mut Value) {
    for service in value["park"]["services"].as_array_mut().unwrap() {
        service
            .as_object_mut()
            .unwrap()
            .remove("construction_cost_cents");
        service
            .as_object_mut()
            .unwrap()
            .remove("demolition_refund_cents");
    }
    if let Some(ledger) = value["park"]["finances"].as_object_mut() {
        for entry in ledger["archived_services"].as_array_mut().unwrap() {
            entry.as_object_mut().unwrap().remove("construction_cents");
            entry
                .as_object_mut()
                .unwrap()
                .remove("demolition_refunds_cents");
        }
    }
}
/// Explicit zero-basis fixture for legacy operating-cost and lifetime-cap tests.
/// Restore the original ledger as well as cash: no synthetic capital entries.
pub(crate) fn place_legacy_service(
    park: &mut Park,
    pos: TilePos,
    height: u8,
    facing: PathDirection,
    kind: ServiceKind,
) -> ServiceId {
    let cash = park.cash_cents;
    let ledger = park.finances.clone();
    park.cash_cents = cash.max(kind.build_cost_cents());
    let id = park.place_service(pos, height, facing, kind).unwrap();
    park.services.last_mut().unwrap().construction_cost_cents = 0;
    park.cash_cents = cash;
    park.finances = ledger;
    id
}
pub(crate) fn reset_finances(park: &mut Park) {
    park.cash_cents = INITIAL_PARK_CASH_CENTS
        + park
            .guests
            .iter()
            .map(|guest| guest.spent_cents)
            .sum::<u64>();
    for service in &mut park.services {
        service.upkeep_progress = 0;
        service.upkeep_cents = 0;
        service.supplies_cents = 0;
        service.closed_for_funds = false;
        service.construction_cost_cents = 0;
        service.demolition_refund_cents = 0;
    }
    park.finances = FinanceLedger::opening(park);
    if let Some(ride) = &park.ride {
        for guest in &mut park.guests {
            guest.ride_history = vec![crate::GuestRideHistory {
                ride: ride.id,
                spent_cents: guest.spent_cents - guest.service_spent_cents,
                completed: guest.rides_completed,
                rating_count: guest.ratings_given,
                rating_total: guest.rating_total,
            }];
        }
    }
}

#[test]
fn open_disconnected_services_charge_at_the_exact_boundary_and_preserve_partial_close() {
    let mut park = Park::new(32, 32);
    let food = site(&mut park, 2, ServiceKind::Food);
    let drink = site(&mut park, 3, ServiceKind::Drink);
    let toilet = site(&mut park, 4, ServiceKind::Toilet);
    assert_eq!(park.finance_stats().open_upkeep_per_minute_cents, 1200);
    advance(&mut park, 89);
    assert_eq!(park.cash_cents, INITIAL_PARK_CASH_CENTS - 30_000);
    assert!(
        park.services
            .iter()
            .all(|service| service.upkeep_progress == 89)
    );
    let frozen = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, frozen);
    park.set_service_open(drink, false).unwrap();
    let mut loaded = roundtrip(&park);
    park.advance(Speed::Normal);
    loaded.advance(Speed::Normal);
    assert_eq!(park, loaded);
    assert_eq!(park.cash_cents, INITIAL_PARK_CASH_CENTS - 30_000 - 40);
    assert_eq!(
        park.finance_ledger()
            .entries
            .iter()
            .map(|entry| entry.service.unwrap())
            .collect::<Vec<_>>(),
        vec![food, drink, toilet, food, toilet]
    );
    assert_eq!(park.service(drink).unwrap().upkeep_progress, 89);
    park.set_service_open(drink, true).unwrap();
    park.advance(Speed::Normal);
    assert_eq!(park.service(drink).unwrap().upkeep_cents, 20);
    assert_eq!(park.service(drink).unwrap().upkeep_progress, 0);
    let object = park.service(food).unwrap().object.unwrap();
    park.remove_object(object).unwrap();
    let history = park.service(food).unwrap().clone();
    advance(&mut park, 180);
    assert_eq!(park.service(food).unwrap(), &history);
    roundtrip(&park);
}

#[test]
fn chronological_rollover_retains_all_category_and_source_totals() {
    let mut park = Park::new(32, 32);
    site(&mut park, 2, ServiceKind::Drink);
    advance(&mut park, 90 * 200);
    let ledger = park.finance_ledger();
    assert_eq!(ledger.entries.len(), 128);
    assert_eq!(ledger.archived_entries, 73);
    assert_eq!(ledger.entries[0].id, 74);
    assert_eq!(ledger.entries[0].tick, 90 * 73);
    assert_eq!(ledger.next_entry_id, 202);
    assert_eq!(ledger.archived.upkeep_cents, 72 * 20);
    assert_eq!(ledger.archived_services[0].upkeep_cents, 72 * 20);
    assert_eq!(ledger.archived.construction_cents, 8_000);
    assert_eq!(park.finance_stats().net_profit_cents, -12_000);
    // Keep the allocator and all retained IDs mutually consistent, so only
    // fixed-cost archive-count validation can reject this forged history.
    for count in [1_u64, 74] {
        let mut forged = park.clone();
        forged.finances.archived_entries = count;
        for (index, entry) in forged.finances.entries.iter_mut().enumerate() {
            entry.id = count + index as u64 + 1;
        }
        forged.finances.next_entry_id = count + forged.finances.entries.len() as u64 + 1;
        assert_eq!(
            validate_finances(&forged),
            Err("archived finance totals or count are invalid".into())
        );
        assert!(forged.to_save_bytes().is_err());
        let mut data: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        data["park"]["finances"] = serde_json::to_value(&forged.finances).unwrap();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&data).unwrap()).is_err());
    }
    let mut loaded = roundtrip(&park);
    for _ in 0..1000 {
        park.advance(Speed::Fastest);
        loaded.advance(Speed::Fastest);
        assert_eq!(park, loaded);
    }
    roundtrip(&park);
}

#[test]
fn no_debt_auto_closure_is_stable_by_service_id_and_reopen_retries_due_cost() {
    let mut park = Park::new(32, 32);
    for x in 1..=20 {
        place_legacy_service(
            &mut park,
            TilePos { x, y: 2 },
            0,
            PathDirection::North,
            ServiceKind::Food,
        );
    }
    let mut reversed = park.clone();
    reversed.services.reverse();
    for _ in 0..18_000 {
        park.advance(Speed::Normal);
        reversed.advance(Speed::Normal);
        if park.services.iter().all(|service| !service.open) {
            break;
        }
    }
    assert!(
        park.services
            .iter()
            .all(|service| service.closed_for_funds && service.upkeep_progress == 89)
    );
    assert_eq!(park.cash_cents, 10);
    reversed.services.sort_by_key(|service| service.id);
    assert_eq!(park, reversed);
    assert_eq!(park.service(ServiceId(1)).unwrap().upkeep_cents, 5010);
    assert_eq!(park.service(ServiceId(20)).unwrap().upkeep_cents, 4980);
    let id = park.services[0].id;
    park.set_service_open(id, true).unwrap();
    assert!(!park.service(id).unwrap().closed_for_funds);
    let before = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, before);
    park.advance(Speed::Normal);
    assert!(!park.service(id).unwrap().open && park.service(id).unwrap().closed_for_funds);
    assert_eq!(park.cash_cents, 10);
    roundtrip(&park);
}

/// Construct a valid exhausted cash boundary with real fixed-cost entries,
/// without spending an existing customer's reserved wallet.
fn exhaust_cash_at_counter(park: &mut Park) {
    let drain = site(park, 2, ServiceKind::Toilet);
    while park.cash_cents > 0 {
        assert_eq!(park.cash_cents % 10, 0);
        park.ticks += 90;
        park.services
            .iter_mut()
            .find(|service| service.id == drain)
            .unwrap()
            .upkeep_cents += 10;
        park.record_cash(FinanceKind::ServiceUpkeep, Some(drain), 10);
    }
    park.set_service_open(drain, false).unwrap();
    for service in &mut park.services {
        service.upkeep_progress = 0;
    }
    roundtrip(park);
}

#[test]
fn a_completed_sale_can_fund_supplies_but_failed_free_use_cancels_without_charge_or_relief() {
    let (mut source, index, id) = using_fixture();
    exhaust_cash_at_counter(&mut source);
    let guest = source.guests[index].clone();
    let point = source.guest_position(guest.id).unwrap().point;
    let mut failed = source.clone();
    failed.advance(Speed::Normal);
    assert_eq!(failed.guests[index].wallet_cents, guest.wallet_cents);
    assert_eq!(failed.guests[index].services_used, guest.services_used);
    assert!(failed.guests[index].service_visit.is_none());
    assert_eq!(failed.guest_position(guest.id).unwrap().point, point);
    assert!(failed.guests[index].needs.toilet >= guest.needs.toilet);
    assert_eq!(
        failed.service(id).unwrap().served,
        source.service(id).unwrap().served
    );
    assert!(failed.service(id).unwrap().closed_for_funds && !failed.service(id).unwrap().open);
    assert_eq!(failed.cash_cents, 0);
    roundtrip(&failed);

    let visit = source.guests[index].service_visit.as_mut().unwrap();
    visit.price_cents = 10;
    let before = source.clone();
    roundtrip(&source);
    source.advance(Speed::Normal);
    assert_eq!(source.guests[index].wallet_cents, guest.wallet_cents - 10);
    assert_eq!(source.guests[index].services_used, guest.services_used + 1);
    assert_eq!(source.guests[index].needs.toilet, 20);
    assert_eq!(source.cash_cents, 0);
    let tail: Vec<_> = source
        .finances
        .entries
        .iter()
        .rev()
        .take(2)
        .map(|entry| (entry.kind, entry.amount_cents, entry.balance_cents))
        .collect();
    assert_eq!(
        tail,
        vec![
            (FinanceKind::ServiceSupplies, 10, 0),
            (FinanceKind::ServiceRevenue, 10, 10)
        ]
    );
    assert_eq!(
        source.service(id).unwrap().supplies_cents,
        before.service(id).unwrap().supplies_cents + 10
    );
    let object = source.service(id).unwrap().object.unwrap();
    source.remove_object(object).unwrap();
    roundtrip(&source);
}

#[test]
fn free_service_relief_still_consumes_supplies_and_upkeep_precedes_same_tick_use() {
    let (mut park, index, id) = using_fixture();
    park.ticks += u64::from(89 - park.service(id).unwrap().upkeep_progress);
    park.services
        .iter_mut()
        .find(|service| service.id == id)
        .unwrap()
        .upkeep_progress = 89;
    // The elapsed lifetime is already long enough for this feasible due phase.
    roundtrip(&park);
    let old_cash = park.cash_cents;
    let old_spent = park.guests[index].spent_cents;
    park.advance(Speed::Normal);
    assert_eq!(park.guests[index].spent_cents, old_spent);
    assert_eq!(park.guests[index].needs.toilet, 20);
    assert_eq!(park.cash_cents, old_cash - 20);
    let entries = &park.finances.entries;
    assert_eq!(entries[entries.len() - 2].kind, FinanceKind::ServiceUpkeep);
    assert_eq!(
        entries[entries.len() - 1].kind,
        FinanceKind::ServiceSupplies
    );
    assert_eq!(
        entries[entries.len() - 2].tick,
        entries[entries.len() - 1].tick
    );
    roundtrip(&park);
}

#[test]
fn finance_continuation_is_identical_at_all_complete_tick_speeds() {
    let mut normal = Park::new(32, 32);
    normal.install_crowd_fixture(42).unwrap();
    normal
        .place_service(
            TilePos { x: 10, y: 21 },
            8,
            PathDirection::North,
            ServiceKind::Food,
        )
        .unwrap();
    normal
        .place_service(
            TilePos { x: 18, y: 21 },
            8,
            PathDirection::North,
            ServiceKind::Toilet,
        )
        .unwrap();
    let mut fast = normal.clone();
    let mut fastest = normal.clone();
    for group in 0..4000 {
        for _ in 0..4 {
            normal.advance(Speed::Normal);
        }
        fast.advance(Speed::Fast);
        fast.advance(Speed::Fast);
        fastest.advance(Speed::Fastest);
        assert_eq!(normal, fast);
        assert_eq!(normal, fastest);
        if group % 500 == 0 {
            fastest = roundtrip(&fastest);
        }
    }
    assert!(normal.finances.archived_entries > 0);
    assert!(
        normal
            .services
            .iter()
            .all(|service| service.served > 0 && service.supplies_cents > 0)
    );
    roundtrip(&normal);
}

#[test]
fn v8_migration_preserves_every_old_field_and_starts_a_new_opening_without_fake_entries() {
    let (mut source, _, _) = using_fixture();
    advance(&mut source, 1);
    let old = strip_finances(serde_json::from_slice(&source.to_save_bytes().unwrap()).unwrap());
    let old_bytes = serde_json::to_vec(&old).unwrap();
    let mut migrated = Park::from_save_bytes(&old_bytes).unwrap();
    assert_eq!(
        strip_finances(serde_json::from_slice(&migrated.to_save_bytes().unwrap()).unwrap()),
        old
    );
    assert_eq!(migrated.finances.opening_tick, source.ticks);
    assert_eq!(
        migrated.finances.opening_cash_cents,
        old["park"]["cash_cents"].as_u64().unwrap()
    );
    assert!(migrated.finances.entries.is_empty() && migrated.finances.archived_entries == 0);
    assert_eq!(
        migrated.finances.opening_services[0].served,
        source.services[0].served
    );
    assert!(
        migrated
            .services
            .iter()
            .all(|service| service.upkeep_cents == 0
                && service.supplies_cents == 0
                && service.upkeep_progress == 0)
    );
    let mut repeated = Park::from_save_bytes(&old_bytes).unwrap();
    for _ in 0..3600 {
        migrated.advance(Speed::Normal);
        repeated.advance(Speed::Normal);
        assert_eq!(migrated, repeated);
    }
    roundtrip(&migrated);
    for (path, field, value) in [
        ("/park", "finances", json!({})),
        ("/park/services/0", "upkeep_cents", json!(0)),
    ] {
        let mut bad = old.clone();
        bad.pointer_mut(path).unwrap()[field] = value;
        assert!(Park::from_save_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}

#[test]
fn strict_finance_metadata_rejects_forged_chronology_balances_sources_and_costs() {
    let mut park = Park::new(32, 32);
    site(&mut park, 2, ServiceKind::Food);
    advance(&mut park, 90 * 140 + 10);
    let base: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    for (path, value) in [
        ("/park/finances/opening_tick", json!(park.ticks + 1)),
        ("/park/finances/opening_cash_cents", json!(100001)),
        ("/park/finances/archived_entries", json!(14)),
        ("/park/finances/archived_through_tick", json!(0)),
        ("/park/finances/archived/upkeep_cents", json!(0)),
        ("/park/finances/archived_services/0/service", json!(999)),
        ("/park/finances/entries/0/id", json!(1)),
        ("/park/finances/entries/0/tick", json!(0)),
        ("/park/finances/entries/0/amount_cents", json!(0)),
        ("/park/finances/entries/0/balance_cents", json!(0)),
        ("/park/finances/entries/0/service", Value::Null),
        ("/park/finances/entries/0/kind", json!("Unknown")),
        ("/park/finances/next_entry_id", json!(u64::MAX)),
        ("/park/services/0/upkeep_progress", json!(90)),
        ("/park/services/0/upkeep_cents", json!(0)),
        ("/park/services/0/closed_for_funds", json!(true)),
    ] {
        let mut bad = base.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "accepted {path}"
        );
    }
    let mut too_early = park.clone();
    too_early.ticks = 90 * 140 - 1;
    too_early.finances.entries.last_mut().unwrap().tick = too_early.ticks;
    assert!(validate_finances(&too_early).is_err());
    let mut missing_ledger = base.clone();
    missing_ledger["park"]
        .as_object_mut()
        .unwrap()
        .remove("finances");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&missing_ledger).unwrap()).is_err());
    let mut missing_source = base.clone();
    missing_source["park"]["finances"]["entries"][0]
        .as_object_mut()
        .unwrap()
        .remove("service");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&missing_source).unwrap()).is_err());
    for (schema, content) in [(13, 13), (10, 9), (9, 10), (9, 8), (8, 9)] {
        let mut bad = base.clone();
        bad["schema_version"] = json!(schema);
        bad["content_version"] = json!(content);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}
