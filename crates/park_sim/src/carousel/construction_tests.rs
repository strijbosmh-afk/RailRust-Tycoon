use super::*;
use crate::{FinanceKind, FinanceLedger, PathDirection, ServiceKind, Speed};
use serde_json::{Value, json};

const ORIGIN: TilePos = TilePos { x: 19, y: 22 };

fn saved(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let restored = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &restored);
    assert_eq!(bytes, restored.to_save_bytes().unwrap());
    restored
}
fn value(park: &Park) -> Value {
    serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap()
}
fn read(value: &Value) -> Result<Park, crate::SaveError> {
    Park::from_save_bytes(&serde_json::to_vec(value).unwrap())
}
/// Establish valid elapsed, fixed-cost service history without running unrelated
/// guest decisions hundreds of thousands of times in a construction boundary test.
fn spend_to(park: &mut Park, target: u64) {
    let service = park
        .place_service(
            TilePos { x: 2, y: 2 },
            0,
            PathDirection::North,
            ServiceKind::Toilet,
        )
        .unwrap();
    while park.cash_cents > target {
        park.ticks += u64::from(crate::SERVICE_UPKEEP_TICKS);
        park.services
            .iter_mut()
            .find(|s| s.id == service)
            .unwrap()
            .upkeep_cents += 10;
        park.record_cash(FinanceKind::ServiceUpkeep, Some(service), 10);
    }
    assert_eq!(park.cash_cents, target);
    saved(park);
}

#[test]
fn paused_quotes_are_read_only_and_commands_record_exact_body_cash_once() {
    let mut park = Park::new(32, 32);
    let before = park.clone();
    let cache = park.path_cache_stats();
    assert_eq!(park.carousel_build_quote(ORIGIN, 0), Ok(30_000));
    assert_eq!(park, before);
    assert_eq!(park.path_cache_stats(), cache);
    let id = park.place_carousel(ORIGIN, 0).unwrap();
    assert_eq!(park.cash_cents, 70_000);
    assert_eq!(park.ticks, 0);
    assert_eq!(park.carousel(id).unwrap().construction_cost_cents, 30_000);
    let entry = &park.finances.entries[0];
    assert_eq!(
        (
            entry.kind,
            entry.ride,
            entry.service,
            entry.tick,
            entry.amount_cents,
            entry.balance_cents
        ),
        (
            FinanceKind::RideConstruction,
            Some(id),
            None,
            0,
            30_000,
            70_000
        )
    );
    saved(&park);
    let built = park.clone();
    assert_eq!(park.carousel_demolition_quote(id), Ok(15_000));
    assert_eq!(park, built);
    park.remove_carousel(id).unwrap();
    assert_eq!(park.cash_cents, 85_000);
    assert_eq!(
        park.finances.entries[1].kind,
        FinanceKind::RideDemolitionRefund
    );
    assert_eq!(park.finances.entries[1].ride, Some(id));
    assert_eq!(park.carousel(id).unwrap().demolition_refund_cents, 15_000);
    let removed = saved(&park);
    assert_eq!(
        park.carousel_demolition_quote(id),
        Err(CarouselError::Removed)
    );
    assert_eq!(park.remove_carousel(id), Err(CarouselError::Removed));
    assert_eq!(park, removed);
    assert_eq!(park.guests, before.guests);
    assert_eq!(park.crowd_rng, before.crowd_rng);
    let stats = park.finance_stats();
    assert_eq!(
        (
            stats.ride_revenue_cents,
            stats.service_revenue_cents,
            stats.construction_cents,
            stats.demolition_refunds_cents,
            stats.total_costs_cents,
            stats.net_profit_cents
        ),
        (0, 0, 30_000, 15_000, 30_000, -15_000)
    );
}

#[test]
fn exact_cash_boundary_and_rejected_placement_preserve_every_authoritative_field() {
    let mut exact = Park::new(32, 32);
    spend_to(&mut exact, 30_000);
    let mut short = exact.clone();
    short.ticks += u64::from(crate::SERVICE_UPKEEP_TICKS);
    short.services[0].upkeep_cents += 10;
    let service = short.services[0].id;
    short.record_cash(FinanceKind::ServiceUpkeep, Some(service), 10);
    let before = saved(&short);
    let cache = short.path_cache_stats();
    let expected = Err(CarouselError::InsufficientFunds {
        required_cents: 30_000,
        available_cents: 29_990,
    });
    assert_eq!(short.carousel_build_quote(ORIGIN, 0), expected);
    assert_eq!(short.place_carousel(ORIGIN, 0), expected.map(|_| RideId(0)));
    assert_eq!(short, before);
    assert_eq!(short.path_cache_stats(), cache);
    let id = exact.place_carousel(ORIGIN, 0).unwrap();
    assert_eq!(exact.cash_cents, 0);
    let mut loaded = saved(&exact);
    loaded.remove_carousel(id).unwrap();
    assert_eq!(loaded.cash_cents, 15_000);
    saved(&loaded);
}

#[test]
fn rebuilding_cannot_generate_cash_and_insufficient_quote_rejects_with_no_allocation() {
    let mut park = Park::new(32, 32);
    let mut previous_id = None;
    for index in 1..=5 {
        let id = park.place_carousel(ORIGIN, 0).unwrap();
        assert!(previous_id.is_none_or(|previous| id > previous));
        park.remove_carousel(id).unwrap();
        assert_eq!(park.cash_cents, 100_000 - index * 15_000);
        previous_id = Some(id);
        saved(&park);
    }
    let before = park.clone();
    assert!(matches!(
        park.place_carousel(ORIGIN, 0),
        Err(CarouselError::InsufficientFunds { .. })
    ));
    assert_eq!(park, before);
    assert_eq!(park.finances.entries.len(), 10);
    assert_eq!(park.finance_stats().net_profit_cents, -75_000);
}

#[test]
fn each_body_tile_demolishes_once_and_occupied_quotes_refuse_without_money_changes() {
    let (mut park, id) = super::tests::fixture(true);
    park.set_ride_open(id, false).unwrap();
    let gates = [
        park.ride_by_id(id).unwrap().entrance.unwrap().object,
        park.ride_by_id(id).unwrap().exit.unwrap().object,
    ];
    let queue: Vec<_> = park
        .objects
        .iter()
        .filter(|o| o.queue_owner == Some(id))
        .map(|o| o.id)
        .collect();
    for part in park.carousel(id).unwrap().objects.clone() {
        let mut copy = park.clone();
        let before_cash = copy.cash_cents;
        copy.remove_object(part).unwrap();
        assert_eq!(copy.cash_cents, before_cash + 15_000);
        assert!(gates.iter().all(|gate| copy.object(*gate).is_none()));
        assert!(queue.iter().all(|tile| copy.object(*tile).is_some()));
        assert!(copy.remove_object(part).is_none());
        saved(&copy);
    }
    park.set_ride_open(id, true).unwrap();
    for _ in 0..24_000 {
        if park.guests.iter().any(|guest| {
            guest.ride_visit.as_ref().is_some_and(|visit| {
                visit.ride == id
                    && visit
                        .passenger
                        .as_ref()
                        .is_some_and(|p| p.transfer.is_some())
            })
        }) {
            let before = saved(&park);
            assert_eq!(
                park.carousel_demolition_quote(id),
                Err(CarouselError::Occupied)
            );
            assert_eq!(park.remove_carousel(id), Err(CarouselError::Occupied));
            assert_eq!(park, before);
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!("carousel did not reach occupied transfer");
}

#[test]
fn archived_tick_zero_charge_and_later_refund_retain_attributed_costs() {
    let mut park = Park::new(32, 32);
    let id = park.place_carousel(ORIGIN, 0).unwrap();
    let service = park
        .place_service(
            TilePos { x: 2, y: 2 },
            0,
            PathDirection::North,
            ServiceKind::Toilet,
        )
        .unwrap();
    for _ in 0..128 * u64::from(crate::SERVICE_UPKEEP_TICKS) {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.finances.archived_entries, 2);
    assert_eq!(park.finances.archived_through_tick, 0);
    assert_eq!(park.finances.archived.construction_cents, 40_000);
    assert_eq!(park.finances.archived_rides[0].ride, id);
    assert_eq!(park.finances.archived_rides[0].construction_cents, 30_000);
    let mut restored = saved(&park);
    restored.remove_carousel(id).unwrap();
    for _ in 0..128 * u64::from(crate::SERVICE_UPKEEP_TICKS) {
        restored.advance(Speed::Normal);
    }
    assert_eq!(
        restored.finances.archived_rides[0].demolition_refunds_cents,
        15_000
    );
    assert!(restored.service(service).unwrap().upkeep_cents > 0);
    saved(&restored);
    let valid = value(&restored);
    for (path, invalid) in [
        (
            "/park/finances/archived_rides/0/construction_cents",
            json!(0),
        ),
        (
            "/park/finances/archived_rides/0/demolition_refunds_cents",
            json!(30_000),
        ),
        ("/park/finances/archived/construction_cents", json!(0)),
        ("/park/finances/archived/demolition_refunds_cents", json!(0)),
        ("/park/finances/archived_through_tick", json!(0)),
    ] {
        let mut bad = valid.clone();
        *bad.pointer_mut(path).unwrap() = invalid;
        assert!(read(&bad).is_err(), "{path}");
    }
}

#[test]
fn paid_basis_and_financial_sources_are_required_strict_and_cross_checked() {
    let mut park = Park::new(32, 32);
    let id = park.place_carousel(ORIGIN, 0).unwrap();
    let built = value(&park);
    for (path, invalid) in [
        ("/park/carousels/0/construction_cost_cents", json!(0)),
        ("/park/carousels/0/construction_cost_cents", json!(30_001)),
        ("/park/carousels/0/demolition_refund_cents", json!(15_000)),
        ("/park/finances/entries/0/kind", json!("RideRevenue")),
        ("/park/finances/entries/0/amount_cents", json!(29_999)),
        ("/park/finances/entries/0/ride", json!(99)),
        ("/park/finances/entries/0/service", json!(1)),
        ("/park/finances/archived/construction_cents", json!(30_000)),
    ] {
        let mut bad = built.clone();
        *bad.pointer_mut(path).unwrap() = invalid;
        assert!(read(&bad).is_err(), "{path}");
    }
    for key in ["construction_cost_cents", "demolition_refund_cents"] {
        let mut bad = built.clone();
        bad["park"]["carousels"][0]
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(read(&bad).is_err(), "missing {key}");
    }
    park.remove_carousel(id).unwrap();
    let removed = value(&park);
    let mut reordered = removed.clone();
    reordered["park"]["finances"]["entries"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    reordered["park"]["finances"]["entries"][0]["id"] = json!(1);
    reordered["park"]["finances"]["entries"][0]["balance_cents"] = json!(115_000);
    reordered["park"]["finances"]["entries"][1]["id"] = json!(2);
    reordered["park"]["finances"]["entries"][1]["balance_cents"] = json!(85_000);
    assert!(
        read(&reordered).is_err(),
        "a running-balance-correct refund cannot precede its charge"
    );
    let mut hidden = removed.clone();
    hidden["park"]["finances"]["opening_rides"] = json!([{
        "ride": id, "revenue_cents": 0, "construction_cents": 30_000, "demolition_refunds_cents": 15_000
    }]);
    hidden["park"]["finances"]["opening_cash_cents"] = json!(85_000);
    hidden["park"]["finances"]["entries"] = json!([]);
    hidden["park"]["finances"]["next_entry_id"] = json!(1);
    assert!(
        read(&hidden).is_err(),
        "paid capital cannot be hidden in a legacy opening balance"
    );
    for (path, invalid) in [
        ("/park/carousels/0/demolition_refund_cents", json!(0)),
        ("/park/finances/entries/1/amount_cents", json!(30_000)),
        ("/park/finances/entries/1/kind", json!("RideConstruction")),
    ] {
        let mut bad = removed.clone();
        *bad.pointer_mut(path).unwrap() = invalid;
        assert!(read(&bad).is_err(), "{path}");
    }
}

#[test]
fn archived_paid_ride_cannot_relabel_genuine_income_as_legacy_opening_revenue() {
    let (mut park, id) = super::tests::fixture(true);
    for x in 0..20 {
        crate::finances::tests::place_legacy_service(
            &mut park,
            TilePos { x, y: 0 },
            0,
            PathDirection::North,
            ServiceKind::Food,
        );
    }
    for _ in 0..12_000 {
        if park
            .finances
            .archived_rides
            .iter()
            .any(|row| row.ride == id && row.revenue_cents > 1)
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    saved(&park);
    let archived = park
        .finances
        .archived_rides
        .iter()
        .find(|row| row.ride == id)
        .unwrap();
    assert_eq!(archived.construction_cents, CAROUSEL_BUILD_COST_CENTS);
    assert!(archived.revenue_cents > 1);
    // Preserve every guest receipt, total lifetime income, running balance and
    // current cash while moving one genuine archived cent before construction.
    let mut forged = park.clone();
    forged
        .finances
        .opening_rides
        .push(crate::RideFinanceTotals {
            ride: id,
            revenue_cents: 1,
            construction_cents: 0,
            demolition_refunds_cents: 0,
        });
    forged.finances.opening_ride_revenue_cents += 1;
    forged.finances.opening_cash_cents += 1;
    forged.finances.archived.ride_revenue_cents -= 1;
    forged
        .finances
        .archived_rides
        .iter_mut()
        .find(|row| row.ride == id)
        .unwrap()
        .revenue_cents -= 1;
    assert_eq!(
        crate::finances::validate_finances(&forged),
        Err("opening ride history is inconsistent".into())
    );
    assert!(forged.to_save_bytes().is_err());
    assert_eq!(forged.guests, park.guests);
    assert_eq!(forged.cash_cents, park.cash_cents);
    assert_eq!(forged.finances.entries, park.finances.entries);
}

#[test]
fn version_ten_migration_preserves_bodies_and_history_without_refund_windfalls() {
    let (mut legacy, id) = super::tests::fixture(true);
    // Model the exact v10 free-construction state before projecting its DTO.
    legacy.carousels[0].construction_cost_cents = 0;
    legacy.cash_cents += CAROUSEL_BUILD_COST_CENTS;
    legacy.finances = FinanceLedger::default();
    for _ in 0..6000 {
        legacy.advance(Speed::Normal);
    }
    legacy.set_ride_open(id, false).unwrap();
    for _ in 0..3000 {
        legacy.advance(Speed::Normal);
    }
    let mut old = value(&legacy);
    crate::finances::tests::strip_construction_fields(&mut old);
    old["schema_version"] = json!(10);
    old["park"].as_object_mut().unwrap().remove("coaster_cars");
    old["content_version"] = json!(10);
    let mut migrated = read(&old).unwrap();
    assert_eq!(migrated, legacy);
    assert_eq!(migrated.carousel_demolition_quote(id), Ok(0));
    let cash = migrated.cash_cents;
    let ledger = migrated.finances.clone();
    migrated.remove_carousel(id).unwrap();
    assert_eq!(migrated.cash_cents, cash);
    assert_eq!(migrated.finances, ledger);
    let fresh = migrated.place_carousel(ORIGIN, 8).unwrap();
    assert_ne!(fresh, id);
    assert_eq!(migrated.carousel_demolition_quote(fresh), Ok(15_000));
    saved(&migrated);
    for key in ["construction_cost_cents", "demolition_refund_cents"] {
        let mut bad = old.clone();
        bad["park"]["carousels"][0][key] = json!(0);
        assert!(read(&bad).is_err(), "v10 accepted {key}");
    }
    for kind in ["RideConstruction", "RideDemolitionRefund"] {
        let mut bad = old.clone();
        bad["park"]["finances"]["entries"][0]["kind"] = json!(kind);
        assert!(read(&bad).is_err(), "v10 accepted {kind}");
    }
}
