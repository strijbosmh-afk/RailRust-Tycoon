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
fn paid_entry_moves_wallet_money_to_separate_park_receipts() {
    let (mut p, _) = simple();
    p.set_park_entry_price(300).unwrap();
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.guests().len(), 1);
    assert_eq!(p.guests()[0].wallet_cents, 1700);
    assert_eq!(p.guests()[0].spent_cents, 300);
    assert_eq!(p.cash_cents(), 100_300);
    assert_eq!(p.visitor_flow_stats().admission_revenue_cents, 300);
    assert_eq!(p.finance_summary().admission_revenue_cents, 300);
    assert_eq!(
        p.finance_ledger().entries.last().unwrap().kind,
        crate::FinanceKind::ParkAdmission
    );
    assert_eq!(
        Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap(),
        p
    );
}

#[test]
fn high_entry_fee_declines_candidates_without_minting_money_or_guests() {
    let (mut p, _) = simple();
    p.set_park_entry_price(2_000).unwrap();
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    assert!(p.guests().is_empty());
    assert_eq!(p.visitor_flow_stats().turned_away, 1);
    assert_eq!(p.visitor_flow_stats().spawned, 0);
    assert_eq!(p.cash_cents(), 100_000);
    let before = p.to_save_bytes().unwrap();
    let mut resumed = Park::from_save_bytes(&before).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
        resumed.advance(Speed::Normal);
    }
    assert_eq!(p, resumed);
    assert_eq!(p.visitor_flow_stats().turned_away, 2);
}

#[test]
fn charging_rules_reset_forbidden_prices_and_guard_future_changes() {
    let (mut p, _) = simple();
    let ride = p.place_carousel(TilePos { x: 12, y: 12 }, 0).unwrap();
    p.set_ride_ticket_price(ride, 100).unwrap();
    p.set_park_entry_price(500).unwrap();
    p.set_charging_rule(ChargingRule::FreeEntry).unwrap();
    assert_eq!(p.visitor_flow_stats().entry_price_cents, 0);
    assert_eq!(
        p.set_park_entry_price(50),
        Err(VisitorError::ChargingRuleRestricted)
    );
    p.set_charging_rule(ChargingRule::FreeRides).unwrap();
    assert_eq!(p.ride_by_id(ride).unwrap().ticket_price_cents, 0);
    assert_eq!(
        p.set_ride_ticket_price(ride, 50),
        Err(crate::EconomyError::ChargingRuleRestricted)
    );
    p.set_park_entry_price(50).unwrap();
    p.set_charging_rule(ChargingRule::Flexible).unwrap();
    p.set_ride_ticket_price(ride, 50).unwrap();
    assert_eq!(
        Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap(),
        p
    );
}
#[test]
fn local_flyers_charge_once_double_candidate_pace_and_resume_exactly() {
    let (mut park, _) = simple();
    park.set_visitor_arrivals(true).unwrap();
    park.set_park_entry_price(300).unwrap();
    park.start_flyer_campaign().unwrap();
    assert_eq!(park.cash_cents(), 97_500);
    assert_eq!(
        park.finance_stats().advertising_cents,
        FLYER_CAMPAIGN_COST_CENTS
    );
    assert_eq!(park.finance_summary().operating_profit_cents, -2_500);
    assert_eq!(
        park.start_flyer_campaign(),
        Err(VisitorError::CampaignAlreadyActive)
    );
    for _ in 0..449 {
        park.advance(Speed::Normal);
    }
    assert!(park.guests().is_empty());
    park.advance(Speed::Normal);
    assert_eq!(park.visitor_flow_stats().flyer_candidates, 1);
    assert_eq!(park.visitor_flow_stats().admission_revenue_cents, 300);
    let mut resumed = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    for _ in 0..2_137 {
        park.advance(Speed::Normal);
        resumed.advance(Speed::Normal);
    }
    assert_eq!(park, resumed);
    for _ in 0..6_413 {
        park.advance(Speed::Normal);
        resumed.advance(Speed::Normal);
    }
    assert_eq!(park, resumed);
    assert_eq!(park.visitor_flow_stats().flyer_ticks_remaining, 0);
    assert_eq!(park.visitor_flow_stats().flyer_candidates, 20);
    assert_eq!(park.visitor_flow_stats().spawned, 20);
    assert_eq!(park.cash_cents(), 103_500);
    for _ in 0..899 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.visitor_flow_stats().spawned, 20);
    park.advance(Speed::Normal);
    assert_eq!(park.visitor_flow_stats().spawned, 21);
    assert_eq!(
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
        park
    );
}

#[test]
fn local_flyers_require_ready_arrivals_and_sufficient_cash() {
    let (mut park, _) = simple();
    let original = park.clone();
    assert_eq!(
        park.start_flyer_campaign(),
        Err(VisitorError::CampaignNeedsArrivals)
    );
    assert_eq!(park, original);
    park.set_visitor_arrivals(true).unwrap();
    park.set_park_open(false);
    assert_eq!(
        park.start_flyer_campaign(),
        Err(VisitorError::CampaignNeedsArrivals)
    );
    park.set_park_open(true);
    park.cash_cents = FLYER_CAMPAIGN_COST_CENTS - 1;
    assert_eq!(
        park.start_flyer_campaign(),
        Err(VisitorError::CampaignUnaffordable)
    );
    assert_eq!(park.visitor_flow_stats().flyer_campaigns, 0);
}

#[test]
fn local_flyers_keep_time_and_cost_when_park_closes() {
    let (mut park, _) = simple();
    park.set_visitor_arrivals(true).unwrap();
    park.start_flyer_campaign().unwrap();
    park.set_park_open(false);
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.visitor_flow_stats().flyer_ticks_remaining, 8_100);
    assert_eq!(park.visitor_flow_stats().flyer_candidates, 0);
    assert_eq!(park.cash_cents(), 97_500);
    let stopped = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, stopped);
    park.set_park_open(true);
    for _ in 0..450 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.visitor_flow_stats().flyer_candidates, 1);
    assert_eq!(park.visitor_flow_stats().flyer_campaigns, 1);
    assert_eq!(
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
        park
    );
}
#[test]
fn radio_ads_change_gate_decisions_without_minting_guest_money_and_overlap_flyers() {
    let (mut park, _) = simple();
    park.set_visitor_arrivals(true).unwrap();
    park.set_park_entry_price(500).unwrap();
    park.start_radio_campaign().unwrap();
    assert_eq!(park.cash_cents(), 96_000);
    assert_eq!(
        park.start_radio_campaign(),
        Err(VisitorError::RadioAlreadyActive)
    );
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.visitor_flow_stats().radio_candidates, 1);
    assert_eq!(park.visitor_flow_stats().radio_admissions, 1);
    assert_eq!(park.guests()[0].wallet_cents, 1_500);
    assert_eq!(park.guests()[0].spent_cents, 500);
    park.start_flyer_campaign().unwrap();
    assert_eq!(park.finance_stats().advertising_cents, 6_500);
    let mut resumed = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    for _ in 0..9_000 {
        park.advance(Speed::Normal);
        resumed.advance(Speed::Normal);
    }
    assert_eq!(park, resumed);
    assert_eq!(park.visitor_flow_stats().radio_ticks_remaining, 0);
    assert_eq!(park.visitor_flow_stats().flyer_ticks_remaining, 0);
    assert!(park.visitor_flow_stats().radio_admissions > 1);
    assert!(
        park.visitor_flow_stats().radio_candidates < park.visitor_flow_stats().flyer_candidates
    );
    assert_eq!(park.finance_summary().advertising_cents, 6_500);
    park.to_save_bytes().unwrap();
}

#[test]
fn radio_ads_require_arrivals_cash_and_keep_time_when_closed() {
    let (mut park, _) = simple();
    assert_eq!(
        park.start_radio_campaign(),
        Err(VisitorError::CampaignNeedsArrivals)
    );
    park.set_visitor_arrivals(true).unwrap();
    park.cash_cents = RADIO_CAMPAIGN_COST_CENTS - 1;
    assert_eq!(
        park.start_radio_campaign(),
        Err(VisitorError::RadioUnaffordable)
    );
    park.cash_cents = 100_000;
    park.start_radio_campaign().unwrap();
    park.set_park_open(false);
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.visitor_flow_stats().radio_ticks_remaining, 8_100);
    assert_eq!(park.visitor_flow_stats().radio_candidates, 0);
    let paused = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, paused);
}

#[test]
fn schema_36_migrates_idle_radio_and_schema_37_requires_radio_state() {
    let (park, _) = simple();
    let mut json: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    json["schema_version"] = serde_json::json!(36);
    json["content_version"] = serde_json::json!(36);
    crate::legacy_cost_test_support::strip_radio_fields(&mut json);
    assert_eq!(
        Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).unwrap(),
        park
    );
    let mut current = json.clone();
    current["schema_version"] = serde_json::json!(37);
    current["content_version"] = serde_json::json!(37);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&current).unwrap()).is_err());
    json["park"]["visitors"]["radio_campaigns"] = serde_json::json!(0);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
}

#[test]
fn schema_36_preserves_an_active_paid_flyer_campaign() {
    let (mut park, _) = simple();
    park.set_visitor_arrivals(true).unwrap();
    park.start_flyer_campaign().unwrap();
    for _ in 0..450 {
        park.advance(Speed::Normal);
    }
    let mut json: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    json["schema_version"] = serde_json::json!(36);
    json["content_version"] = serde_json::json!(36);
    crate::legacy_cost_test_support::strip_radio_fields(&mut json);
    let loaded = Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).unwrap();
    assert_eq!(loaded, park);
    assert_eq!(loaded.visitor_flow_stats().flyer_ticks_remaining, 8_550);
    assert_eq!(
        loaded.finance_stats().advertising_cents,
        FLYER_CAMPAIGN_COST_CENTS
    );
}
#[test]
fn closing_park_stops_arrivals_and_departing_guests_reopen_without_instant_arrival() {
    let (mut p, _) = simple();
    p.set_visitor_arrivals(true).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.guests().len(), 1);
    let id = p.guests()[0].id;
    p.set_park_open(false);
    assert!(!p.visitor_flow_stats().park_open);
    assert!(p.guest_is_leaving(id));
    assert_eq!(p.visitor_flow_stats().arrival_progress, 0);
    let saved = p.to_save_bytes().unwrap();
    assert_eq!(Park::from_save_bytes(&saved).unwrap(), p);
    for _ in 0..1200 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.visitor_flow_stats().spawned, 1);
    assert_eq!(p.visitor_flow_stats().departed, 1);
    assert!(p.guests().is_empty());
    p.set_park_open(true);
    for _ in 0..899 {
        p.advance(Speed::Normal);
    }
    assert!(p.guests().is_empty());
    p.advance(Speed::Normal);
    assert_eq!(p.guests().len(), 1);
    assert!(p.guests()[0].id > id);
}

#[test]
fn schema_33_defaults_to_open_but_schema_34_requires_explicit_status() {
    let (p, _) = simple();
    let mut json: serde_json::Value = serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    json["park"]["visitors"]
        .as_object_mut()
        .unwrap()
        .remove("park_open");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
    json["schema_version"] = serde_json::json!(33);
    json["content_version"] = serde_json::json!(33);
    crate::legacy_cost_test_support::strip_admission_fields(&mut json);
    let old = Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).unwrap();
    assert!(old.visitor_flow_stats().park_open);
    json["park"]["visitors"]["park_open"] = serde_json::json!(false);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
}

#[test]
fn schema_34_defaults_to_free_flexible_entry_and_schema_35_requires_admission_state() {
    let (p, _) = simple();
    let mut json: serde_json::Value = serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    for field in [
        "entry_price_cents",
        "charging_rule",
        "admission_revenue_cents",
        "turned_away",
    ] {
        json["park"]["visitors"]
            .as_object_mut()
            .unwrap()
            .remove(field);
    }
    json["park"]["finances"]["archived"]
        .as_object_mut()
        .unwrap()
        .remove("admission_revenue_cents");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
    json["schema_version"] = serde_json::json!(34);
    json["content_version"] = serde_json::json!(34);
    crate::legacy_cost_test_support::strip_flyer_fields(&mut json);
    let old = Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).unwrap();
    assert_eq!(old, p);
    json["park"]["visitors"]["entry_price_cents"] = serde_json::json!(0);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
}
#[test]
fn schema_35_migrates_idle_flyers_and_schema_36_requires_their_state() {
    let (park, _) = simple();
    let mut json: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    json["schema_version"] = serde_json::json!(35);
    json["content_version"] = serde_json::json!(35);
    crate::legacy_cost_test_support::strip_flyer_fields(&mut json);
    let mut current = json.clone();
    current["schema_version"] = serde_json::json!(36);
    current["content_version"] = serde_json::json!(36);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&current).unwrap()).is_err());
    assert_eq!(
        Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).unwrap(),
        park
    );
    json["park"]["visitors"]["flyer_campaigns"] = serde_json::json!(0);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
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
    let mut false_closure = value.clone();
    false_closure["park"]["visitors"]["park_open"] = serde_json::json!(false);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&false_closure).unwrap()).is_err());
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
    current["park"]
        .as_object_mut()
        .unwrap()
        .remove("maintenance");
    current["park"]
        .as_object_mut()
        .unwrap()
        .remove("mechanic_route");
    current["park"].as_object_mut().unwrap().remove("grounds");
    for ride in current["park"]["carousels"].as_array_mut().unwrap() {
        ride.as_object_mut().unwrap().remove("kind");
    }
    current["park"]
        .as_object_mut()
        .unwrap()
        .remove("mechanic_hired");
    current["park"]
        .as_object_mut()
        .unwrap()
        .remove("mechanic_wage_progress");
    current["park"]["finances"]["archived"]
        .as_object_mut()
        .unwrap()
        .remove("wages_cents");
    current["park"]["finances"]["archived"]
        .as_object_mut()
        .unwrap()
        .remove("handyman_wages_cents");
    current["schema_version"] = serde_json::json!(21);
    crate::legacy_cost_test_support::strip_admission_fields(&mut current);
    let mut old_default = serde_json::to_value(super::VisitorFlow::default()).unwrap();
    for field in [
        "entry_price_cents",
        "charging_rule",
        "admission_revenue_cents",
        "turned_away",
        "flyer_ticks_remaining",
        "flyer_campaigns",
        "flyer_candidates",
        "radio_ticks_remaining",
        "radio_campaigns",
        "radio_candidates",
        "radio_admissions",
    ] {
        old_default.as_object_mut().unwrap().remove(field);
    }
    assert_eq!(
        current["park"]
            .as_object_mut()
            .unwrap()
            .remove("visitors")
            .unwrap(),
        old_default
    );
    assert_eq!(
        current["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_paints"),
        Some(serde_json::json!([]))
    );
    assert_eq!(
        current["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_names"),
        Some(serde_json::json!([]))
    );
    for guest in current["park"]["guests"].as_array_mut().unwrap() {
        guest["needs"].as_object_mut().unwrap().remove("energy");
        guest["needs"].as_object_mut().unwrap().remove("nausea");
    }
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
