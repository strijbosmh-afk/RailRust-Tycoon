use crate::*;

pub(crate) fn without_upkeep(park: &Park) -> Park {
    let mut value: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    value["schema_version"] = 19.into();
    crate::legacy_cost_test_support::project_old_fixture(&mut value);
    value["schema_version"] = 20.into();
    value["content_version"] = 20.into();
    value["park"]["ride_costs"] = serde_json::json!([]);
    for key in ["opening_rides", "archived_rides"] {
        for row in value["park"]["finances"][key].as_array_mut().unwrap() {
            row["upkeep_cents"] = 0.into();
        }
    }
    Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).unwrap()
}

fn ticks(park: &mut Park, count: usize) {
    for _ in 0..count {
        park.advance(Speed::Normal);
    }
}

#[test]
fn simultaneous_bills_use_stable_ride_order_before_any_new_fares() {
    let (mut park, carousel) = crate::carousel::tests::fixture(true);
    let coaster = park.ride().unwrap().id;
    for id in [coaster, carousel] {
        park.set_ride_ticket_price(id, MAX_TICKET_PRICE_CENTS)
            .unwrap();
    }
    ticks(&mut park, 899);
    // Isolate the scheduling budget; full finance validity is exercised by the
    // real-purchase closure/recovery proof and strict ledger roundtrips.
    park.cash_cents = 100;
    ticks(&mut park, 1);
    assert_eq!(park.cash_cents(), 0);
    assert_eq!(park.ride_running_costs(coaster).unwrap().upkeep_cents, 100);
    assert!(park.ride_is_open(coaster));
    assert!(park.ride_running_costs(carousel).unwrap().closed_for_funds);
    assert_eq!(park.ride_running_costs(carousel).unwrap().upkeep_cents, 0);
}

#[test]
fn billing_is_exact_paused_closed_and_speed_grouping_preserves_pending_costs() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    let id = park.ride().unwrap().id;
    ticks(&mut park, 899);
    assert_eq!(park.ride_running_costs(id).unwrap().upkeep_progress, 899);
    let before = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, before);
    ticks(&mut park, 1);
    assert_eq!(park.cash_cents(), 99_900);
    assert_eq!(park.ride_running_costs(id).unwrap().upkeep_cents, 100);
    ticks(&mut park, 13);
    park.set_ride_open(id, false).unwrap();
    let cost = park.ride_running_costs(id).unwrap();
    ticks(&mut park, 100);
    assert_eq!(park.ride_running_costs(id).unwrap(), cost);
    let bytes = park.to_save_bytes().unwrap();
    let mut loaded = Park::from_save_bytes(&bytes).unwrap();
    park.set_ride_open(id, true).unwrap();
    loaded.set_ride_open(id, true).unwrap();
    ticks(&mut park, 1200);
    for _ in 0..300 {
        loaded.advance(Speed::Fastest);
    }
    assert_eq!(park, loaded);
    assert_eq!(
        park.finance_stats().ride_upkeep_cents,
        park.ride_running_costs(id).unwrap().upkeep_cents
    );
}

#[test]
fn zero_cash_closes_in_stable_order_and_overdue_payment_cannot_be_reset() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    let id = park.ride().unwrap().id;
    park.set_train_open(false);
    for _ in 0..4 {
        let c = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        park.remove_carousel(c).unwrap();
    }
    let reserve = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    park.copy_installed_track_to_draft().unwrap();
    let pieces = park
        .track_draft()
        .iter()
        .filter(|p| p.kind == TrackKind::Straight)
        .take(5)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), 5);
    for mut p in pieces {
        p.kind = TrackKind::Brake;
        park.set_track_draft_piece(p).unwrap();
    }
    park.apply_track_draft().unwrap();
    assert_eq!(park.cash_cents(), 0);
    park.set_train_open(true);
    park.set_ride_open(reserve, true).unwrap();
    ticks(&mut park, 900);
    for owner in [id, reserve] {
        let c = park.ride_running_costs(owner).unwrap();
        assert!(c.closed_for_funds);
        assert_eq!(c.upkeep_progress, 899);
        assert_eq!(c.upkeep_cents, 0);
        assert!(!park.ride_is_open(owner));
    }
    let costs = park.ride_running_costs(id);
    ticks(&mut park, 90);
    assert_eq!(park.ride_running_costs(id), costs);
    Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    park.remove_carousel(reserve).unwrap();
    assert_eq!(park.cash_cents(), 15_000);
    park.set_train_open(true);
    assert_eq!(park.cash_cents(), 15_000);
    ticks(&mut park, 1);
    assert_eq!(park.cash_cents(), 14_900);
    assert_eq!(park.ride_running_costs(id).unwrap().upkeep_progress, 0);
    assert!(!park.ride_running_costs(id).unwrap().closed_for_funds);
    Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
}

#[test]
fn real_v19_paid_save_preserves_all_old_fields_and_starts_costs_prospectively() {
    let bytes = include_bytes!("../testdata/v19-paid-opening.park.json");
    let old: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let mut park = Park::from_save_bytes(bytes).unwrap();
    let current: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    fn fields(old: &serde_json::Value, new: &serde_json::Value) {
        match (old, new) {
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                for (k, v) in a {
                    fields(v, &b[k]);
                }
            }
            (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(b) {
                    fields(a, b);
                }
            }
            _ => assert_eq!(old, new),
        }
    }
    fields(&old["park"], &current["park"]);
    for ride in park.all_rides() {
        let c = park.ride_running_costs(ride.id).unwrap();
        assert_eq!(c.upkeep_cents, 0);
        assert_eq!(c.upkeep_progress, 0);
    }
    let mut resumed = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    for _ in 0..3600 {
        park.advance(Speed::Normal);
        resumed.advance(Speed::Normal);
    }
    assert_eq!(park, resumed);
    for ride in park.all_rides() {
        assert!(park.ride_running_costs(ride.id).unwrap().upkeep_cents > 0);
    }
    let mut future = old.clone();
    future["park"]["ride_costs"] = serde_json::json!([]);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&future).unwrap()).is_err());
}

#[test]
fn current_save_requires_cost_records_and_rejects_forged_attribution_and_closure() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    ticks(&mut park, 1801);
    let value: serde_json::Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    for (path, bad) in [
        ("/park/ride_costs/0/upkeep_progress", serde_json::json!(900)),
        ("/park/ride_costs/0/upkeep_cents", serde_json::json!(300)),
        (
            "/park/ride_costs/0/closed_for_funds",
            serde_json::json!(true),
        ),
        ("/park/ride_costs/0/ride", serde_json::json!(999)),
        (
            "/park/finances/entries/0/amount_cents",
            serde_json::json!(50),
        ),
    ] {
        let mut bad_value = value.clone();
        *bad_value.pointer_mut(path).unwrap() = bad;
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&bad_value).unwrap()).is_err(),
            "{path}"
        );
    }
    let mut missing = value.clone();
    missing["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_costs");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&missing).unwrap()).is_err());
    let mut duplicate = value.clone();
    let row = duplicate["park"]["ride_costs"][0].clone();
    duplicate["park"]["ride_costs"]
        .as_array_mut()
        .unwrap()
        .push(row);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&duplicate).unwrap()).is_err());
}
