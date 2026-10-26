use super::*;
use crate::{Speed, TilePos};

fn simple() -> (Park, ObjectId) {
    let mut p = Park::new(32, 32);
    let entry = p.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
    p.place_path(TilePos { x: 6, y: 5 }, 0).unwrap();
    p.set_visitor_entry(entry).unwrap();
    (p, entry)
}
#[test]
fn arrivals_are_opt_in_timed_and_do_not_mint_park_cash() {
    let (mut p, _) = simple();
    for _ in 0..1800 {
        p.advance(Speed::Normal);
    }
    assert!(p.guests().is_empty());
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..899 {
        p.advance(Speed::Normal);
    }
    assert!(p.guests().is_empty());
    p.advance(Speed::Normal);
    assert_eq!(p.guests().len(), 1);
    assert_eq!(p.cash_cents(), 100_000);
    assert_eq!(p.guests()[0].wallet_cents, 2000);
    assert_eq!(p.finance_summary().operating_profit_cents, 0);
    assert_eq!(
        Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap(),
        p
    );
}
#[test]
fn departures_walk_to_entry_and_keep_wallets_outside_park_income() {
    let (mut p, _) = simple();
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    p.set_visitor_arrivals(false).unwrap();
    let id = p.guests()[0].id;
    p.request_guest_departure(id).unwrap();
    for _ in 0..300 {
        p.advance(Speed::Normal);
        p.to_save_bytes().unwrap();
    }
    assert!(p.guests().is_empty());
    assert_eq!(p.departed_visitors().count, 1);
    assert_eq!(p.departed_visitors().wallet_cents, 2000);
    assert_eq!(p.cash_cents(), 100_000);
    let old = id;
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    assert!(p.guests()[0].id > old);
}
#[test]
fn entry_removal_stops_arrivals_and_departures_until_a_new_entry_is_set() {
    let (mut p, entry) = simple();
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    let id = p.guests()[0].id;
    p.request_guest_departure(id).unwrap();
    p.remove_object(entry).unwrap();
    for _ in 0..100 {
        p.advance(Speed::Normal);
        p.to_save_bytes().unwrap();
    }
    assert_eq!(p.guests().len(), 1);
    assert!(!p.visitor_flow_stats().entry_ready);
    let next = p.objects().iter().find(|o| o.is_public_path()).unwrap().id;
    p.set_visitor_entry(next).unwrap();
    for _ in 0..100 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.departed_visitors().count, 1);
}

#[test]
fn paid_passengers_finish_before_departing_and_keep_all_receipts_and_reviews() {
    let (mut p, carousel) = crate::carousel::tests::fixture(true);
    let mut selected = None;
    for _ in 0..20_000 {
        p.advance(Speed::Normal);
        if let Some(g) = p.guests().iter().find(|g| {
            g.ride_visit
                .as_ref()
                .and_then(|v| v.passenger.as_ref())
                .is_some_and(|s| s.fare_paid)
        }) {
            selected = Some(g.id);
            break;
        }
    }
    let id = selected.unwrap();
    let entry = p.objects().iter().find(|p| p.is_public_path()).unwrap().id;
    p.set_visitor_entry(entry).unwrap();
    p.request_guest_departure(id).unwrap();
    p.set_train_open(false);
    p.set_ride_open(carousel, false).unwrap();
    let before = p.cash_cents();
    p.advance(Speed::Normal);
    assert!(
        p.guests()
            .iter()
            .any(|g| g.id == id && g.ride_visit.is_some())
    );
    assert_eq!(p.departed_visitors().count, 0);
    for _ in 0..24_000 {
        p.advance(Speed::Normal);
        if p.departed_visitors().count > 0 {
            break;
        }
    }
    assert_eq!(p.departed_visitors().count, 1);
    assert!(!p.guests().iter().any(|g| g.id == id));
    assert!(p.departed_visitors().spent_cents > 0);
    assert!(
        p.departed_visitors()
            .rides
            .iter()
            .any(|r| r.completed > 0 && r.rating_count > 0)
    );
    assert_eq!(p.cash_cents(), before);
    assert_eq!(
        Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap(),
        p
    );
}

#[test]
fn age_departures_replace_the_crowd_without_reusing_ids_or_creating_cash() {
    let (mut p, _) = simple();
    p.set_visitor_arrivals(true).unwrap();
    let mut fast = p.clone();
    for _ in 0..20_000 {
        p.advance(Speed::Normal);
    }
    for _ in 0..5_000 {
        fast.advance(Speed::Fastest);
    }
    assert_eq!(p, fast);
    assert!(p.departed_visitors().count > 0);
    assert_eq!(p.cash_cents(), 100_000);
    let before = p.clone();
    p.advance(Speed::Paused);
    assert_eq!(p, before);
    assert_eq!(
        p.departed_visitors().wallet_cents,
        2000 * p.departed_visitors().count
    );
    assert_eq!(
        Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap(),
        p
    );
}

#[test]
fn population_cap_holds_one_due_arrival_until_a_slot_is_free() {
    let (mut p, entry) = simple();
    p.crowd_seed = Some(0);
    let path = *p.object(entry).unwrap();
    for id in 1..=64 {
        p.guests.push(new_guest(GuestId(id), path, 0));
    }
    p.next_guest_id = 65;
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.guests().len(), 64);
    assert_eq!(p.visitor_flow_stats().spawned, 0);
    p.request_guest_departure(GuestId(1)).unwrap();
    for _ in 0..150 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.departed_visitors().count, 1);
    assert_eq!(p.guests().len(), 64);
    assert_eq!(p.visitor_flow_stats().spawned, 1);
    assert!(p.guests().iter().any(|g| g.id == GuestId(65)));
    p.to_save_bytes().unwrap();
}

#[test]
fn strict_saves_reject_forged_retirement_and_missing_visitor_state() {
    let (mut p, _) = simple();
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    let value: serde_json::Value = serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    for (path, bad) in [
        ("/park/visitors/arrival_progress", serde_json::json!(900)),
        ("/park/visitors/entry", serde_json::json!(999)),
        ("/park/visitors/stays/0/guest", serde_json::json!(999)),
        ("/park/visitors/departed/spent_cents", serde_json::json!(1)),
        ("/park/visitors/departed/count", serde_json::json!(u64::MAX)),
    ] {
        let mut v = value.clone();
        *v.pointer_mut(path).unwrap() = bad;
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{path}"
        );
    }
    let mut v = value;
    v["park"].as_object_mut().unwrap().remove("visitors");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
}

#[test]
fn authentic_v21_migration_preserves_every_field_and_has_no_automatic_arrivals() {
    let bytes = include_bytes!("../../testdata/v21-paid-value.park.json");
    let original: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let p = Park::from_save_bytes(bytes).unwrap();
    let mut current: serde_json::Value =
        serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    assert_eq!(
        current["park"]
            .as_object_mut()
            .unwrap()
            .remove("visitors")
            .unwrap(),
        serde_json::to_value(super::VisitorFlow::default()).unwrap()
    );
    assert_eq!(
        current["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_paints"),
        Some(serde_json::json!([]))
    );
    assert_eq!(current["park"], original["park"]);
    assert!(!p.visitor_flow_stats().enabled);
    let mut continuous = p.clone();
    let mut restarted = p;
    for i in 0..3600 {
        continuous.advance(Speed::Normal);
        restarted.advance(Speed::Normal);
        if i % 300 == 0 {
            restarted = Park::from_save_bytes(&restarted.to_save_bytes().unwrap()).unwrap();
        }
    }
    assert_eq!(continuous, restarted);
    assert_eq!(continuous.visitor_flow_stats().spawned, 0);
}

#[test]
fn departure_during_service_approach_finishes_purchase_before_walking_home() {
    let mut p = Park::new(32, 32);
    p.install_crowd_fixture(42).unwrap();
    p.place_service(
        TilePos { x: 15, y: 21 },
        8,
        crate::PathDirection::North,
        crate::ServiceKind::Drink,
    )
    .unwrap();
    let entry = p.objects().iter().find(|o| o.is_public_path()).unwrap().id;
    p.set_visitor_entry(entry).unwrap();
    let mut selected = None;
    for _ in 0..30000 {
        p.advance(Speed::Normal);
        if let Some(g) = p.guests().iter().find(|g| {
            g.service_visit
                .as_ref()
                .is_some_and(|v| v.phase == crate::ServiceVisitPhase::Approaching)
        }) {
            selected = Some(g.id);
            break;
        }
    }
    let id = selected.unwrap();
    p.request_guest_departure(id).unwrap();
    for _ in 0..12000 {
        p.advance(Speed::Normal);
        p = Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap();
        if p.departed_visitors().count > 0 {
            break;
        }
    }
    assert_eq!(p.departed_visitors().count, 1);
    assert!(p.departed_visitors().services_used > 0);
    assert!(p.departed_visitors().service_spent_cents > 0);
}
