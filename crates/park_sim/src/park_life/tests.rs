use super::*;
use crate::{PathDirection, Speed, TilePos};
fn park() -> Park {
    let mut p = Park::new(32, 32);
    for x in 1..12 {
        p.place_path(TilePos { x, y: 5 }, 0).unwrap();
    }
    let entry = p
        .objects()
        .iter()
        .find(|p| p.pos == TilePos { x: 1, y: 5 })
        .unwrap()
        .id;
    p.set_visitor_entry(entry).unwrap();
    p.set_visitor_arrivals(true).unwrap();
    p.enable_park_life();
    p
}
fn advance(p: &mut Park, ticks: usize) {
    for _ in 0..ticks {
        p.advance(Speed::Normal);
    }
}
#[test]
fn parties_weather_workers_and_finances_continue_after_loading() {
    let mut p = park();
    let entry = p.visitor_flow_stats().entry.unwrap();
    p.toggle_path_shelter(entry).unwrap();
    for role in StaffRole::ALL {
        p.hire_park_staff(role).unwrap();
    }
    p.place_service(
        TilePos { x: 4, y: 4 },
        0,
        PathDirection::South,
        crate::ServiceKind::FirstAid,
    )
    .unwrap();
    advance(&mut p, 2400);
    assert!(!p.park_staff().is_empty());
    assert!(p.park_life_status().parties > 0);
    let bytes = p.to_save_bytes().unwrap();
    let mut loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(p, loaded);
    for _ in 0..6000 {
        p.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
    }
    assert_eq!(p, loaded);
    p.to_save_bytes().unwrap();
    assert!(p.finance_stats().wages_cents > 0);
}
#[test]
fn shelter_weather_and_security_have_real_effects_and_recover_after_path_removal() {
    let mut p = park();
    advance(&mut p, 900);
    let path = p.guests()[0].current.unwrap();
    p.toggle_path_shelter(path).unwrap();
    assert!(p.path_is_sheltered(path));
    p.park_life.as_mut().unwrap().incidents.push(ParkIncident {
        id: 1,
        path,
        kind: IncidentKind::Vandalism,
        tick: p.ticks,
    });
    p.park_life.as_mut().unwrap().next_incident_id = 2;
    assert!(!p.path_is_sheltered(path));
    p.hire_park_staff(StaffRole::Security).unwrap();
    advance(&mut p, 1800);
    assert_eq!(p.park_life_status().resolved, 1);
    p.set_weather_policy(WeatherPolicy::Rain);
    assert_eq!(p.park_life_status().weather, Weather::Rain);
    p.to_save_bytes().unwrap();
    p.remove_object(path).unwrap();
    p.to_save_bytes().unwrap();
    advance(&mut p, 60);
    p.to_save_bytes().unwrap();
}
#[test]
fn old_saves_keep_legacy_mode_and_old_headers_reject_park_life() {
    let old = Park::new(8, 8);
    let mut value: serde_json::Value =
        serde_json::from_slice(&old.to_save_bytes().unwrap()).unwrap();
    value["schema_version"] = 48.into();
    value["content_version"] = 48.into();
    let loaded = Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(old, loaded);
    let p = park();
    let mut value: serde_json::Value = serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    value["schema_version"] = 48.into();
    value["content_version"] = 48.into();
    assert!(Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn all_roles_have_reachable_jobs_and_bounded_hiring() {
    let mut p = park();
    for _ in 0..4 {
        for role in StaffRole::ALL {
            p.hire_park_staff(role).unwrap();
        }
    }
    assert!(p.hire_park_staff(StaffRole::Security).is_err());
    advance(&mut p, 1000);
    assert_eq!(p.park_staff().len(), 20);
    p.to_save_bytes().unwrap();
}

#[test]
fn staff_carry_out_each_guest_and_grounds_job() {
    let mut p = park();
    advance(&mut p, 900);
    let path = p.visitor_flow_stats().entry.unwrap();
    // Keep the test guest at the real entry while staff arrive and work.
    p.guests[0].current = Some(path);
    p.guests[0].next = None;
    p.guests[0].destination = None;
    p.guests[0].route.clear();
    p.guests[0].progress = 0;
    p.guests[0].wait_ticks = 1000;
    p.guests[0].state = crate::GuestState::Waiting;
    p.guests[0].satisfaction = 40;
    p.guests[0].needs.nausea = 80;
    p.guests[0].needs.energy = 10;
    p.grounds.messes.push(crate::GroundMess {
        id: 1,
        path,
        kind: crate::GroundMessKind::Litter,
    });
    p.grounds.next_mess_id = 2;
    for role in [
        StaffRole::Handyman,
        StaffRole::Entertainer,
        StaffRole::Attendant,
    ] {
        p.hire_park_staff(role).unwrap();
    }
    advance(&mut p, 125);
    assert!(p.grounds.cleaned > 0);
    assert!(p.guests[0].satisfaction > 40);
    assert!(p.guests[0].needs.nausea < 80);
    assert!(p.park_staff().iter().all(|s| s.completed > 0));
    p.guests[0].wait_ticks = 0;
    p.to_save_bytes().unwrap();
}

#[test]
fn mechanic_team_reaches_and_services_a_stopped_ride() {
    let mut p = Park::new(32, 32);
    p.install_train_fixture().unwrap();
    let id = p.ride().unwrap().id;
    p.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, id)
        .unwrap();
    for x in 8..=13 {
        p.place_path(TilePos { x, y: 8 }, 8).unwrap();
    }
    let mut entry = None;
    for y in 9..=11 {
        entry = Some(p.place_path(TilePos { x: 13, y }, 8).unwrap());
    }
    p.set_visitor_entry(entry.unwrap()).unwrap();
    p.enable_park_life();
    p.set_train_open(false);
    p.ride.as_mut().unwrap().completed = crate::MAINTENANCE_DUE_VISITS;
    p.maintenance
        .push(crate::maintenance::RideMaintenanceRecord {
            ride: id,
            serviced_completed: 0,
            service_count: 0,
            last_service_tick: 0,
        });
    p.hire_park_staff(StaffRole::Mechanic).unwrap();
    advance(&mut p, 900);
    assert_eq!(p.ride_maintenance_status(id).unwrap().service_count, 1);
    assert!(p.park_staff()[0].completed > 0);
}

#[test]
fn satisfied_groups_can_return_after_their_departure() {
    let mut p = park();
    advance(&mut p, 900);
    p.set_visitor_arrivals(false).unwrap();
    let guests = p.guests.clone();
    for g in &guests {
        p.request_guest_departure(g.id).unwrap();
    }
    advance(&mut p, 1200);
    assert!(p.guests.is_empty());
    assert_eq!(p.park_life.as_ref().unwrap().returns.len(), 1);
    advance(&mut p, 6000);
    p.set_visitor_arrivals(true).unwrap();
    advance(&mut p, 900);
    assert!(p.park_life_status().returning_arrivals > 0);
    p.to_save_bytes().unwrap();
}

#[test]
fn wage_shortfall_stops_staff_then_funding_resumes_without_free_work() {
    let mut p = park();
    let id = p.hire_park_staff(StaffRole::Security).unwrap();
    p.cash_cents = 0;
    advance(&mut p, 900);
    assert!(!p.park_staff()[0].working);
    assert_eq!(p.park_staff()[0].wages_paid_cents, 0);
    p.cash_cents = 100;
    advance(&mut p, 1);
    assert!(p.park_staff()[0].working);
    assert_eq!(p.park_staff()[0].wages_paid_cents, 100);
    assert_eq!(p.cash_cents, 0);
    p.dismiss_park_staff(id);
    assert!(p.park_staff().is_empty());
}

#[test]
fn attendant_helps_a_normal_paid_service_visit_without_holding_the_guest() {
    let mut p = park();
    let service = p
        .place_service(
            TilePos { x: 1, y: 4 },
            0,
            PathDirection::South,
            crate::ServiceKind::Food,
        )
        .unwrap();
    p.hire_park_staff(StaffRole::Attendant).unwrap();
    advance(&mut p, 900);
    p.guests[0].needs.hunger = 80;
    advance(&mut p, 1200);
    assert!(p.park_staff()[0].completed > 0);
    assert!(p.service(service).unwrap().served > 0);
    p.to_save_bytes().unwrap();
}

#[test]
fn weather_controls_do_not_advance_visitors_and_storms_reopen_only_weather_closures() {
    let mut p = crate::diagnostics::build_catalogue_showcase().unwrap();
    p.enable_park_life();
    let id = p
        .all_rides()
        .find(|r| p.ride_weather_exposed(r.id) && p.ride_is_open(r.id))
        .unwrap()
        .id;
    let ticks = p.ticks();
    let needs = p.guests.iter().map(|g| g.needs).collect::<Vec<_>>();
    p.set_weather_policy(WeatherPolicy::Storm);
    assert_eq!(p.ticks(), ticks);
    assert!(!p.ride_is_open(id));
    assert!(p.set_ride_open_checked(id, true).is_err());
    assert_eq!(needs, p.guests.iter().map(|g| g.needs).collect::<Vec<_>>());
    p.set_weather_policy(WeatherPolicy::Sunny);
    assert!(p.ride_is_open(id));
    p.to_save_bytes().unwrap();
}

#[test]
fn deleting_a_covered_bench_removes_its_shelter_and_forged_state_is_refused() {
    let mut p = park();
    let path = p.visitor_flow_stats().entry.unwrap();
    p.toggle_path_shelter(path).unwrap();
    p.toggle_path_bench(path).unwrap();
    assert!(!p.path_is_sheltered(path));
    p.to_save_bytes().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    value["park"]["park_life"]["shelters"] = serde_json::json!([999999]);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
}
