use super::*;
use crate::{Park, RideDispatchPolicy, RideVisitPhase, Speed};
use std::collections::BTreeSet;

fn fixture() -> (Park, RideId, RideId) {
    let (mut park, carousel) = crate::carousel::tests::fixture(true);
    let coaster = park.ride().unwrap().id;
    park.set_ride_open(coaster, false).unwrap();
    park.set_ride_open(carousel, false).unwrap();
    (park, coaster, carousel)
}

#[test]
fn cars_are_bounded_per_family_and_downsizing_preserves_load_policy() {
    let (mut park, coaster, carousel) = fixture();
    let before = park.clone();
    for cars in [0, 4, 255] {
        assert_eq!(
            park.set_coaster_car_count(coaster, cars),
            Err(TrainConfigurationError::InvalidCars)
        );
        assert_eq!(park, before);
    }
    assert!(park.set_coaster_car_count(carousel, 1).is_err());
    assert!(park.set_coaster_car_count(RideId(u64::MAX), 1).is_err());
    park.set_ride_dispatch_policy(
        coaster,
        RideDispatchPolicy {
            minimum_passengers: 5,
            ..Default::default()
        },
    )
    .unwrap();
    let before = park.clone();
    assert_eq!(
        park.set_coaster_car_count(coaster, 2),
        Err(TrainConfigurationError::MinimumLoad {
            minimum: 5,
            capacity: 4
        })
    );
    assert_eq!(park, before);
    park.set_ride_dispatch_policy(coaster, RideDispatchPolicy::default())
        .unwrap();
    park.set_coaster_car_count(coaster, 1).unwrap();
    assert_eq!(park.ride_capacity(coaster), Some(2));
    assert_eq!(park.ride_capacity(carousel), Some(6));
    assert!(
        park.set_ride_dispatch_policy(
            coaster,
            RideDispatchPolicy {
                minimum_passengers: 3,
                ..Default::default()
            }
        )
        .is_err()
    );
    park.set_ride_open(coaster, true).unwrap();
    let before = park.clone();
    assert!(park.set_coaster_car_count(coaster, 2).is_err());
    assert_eq!(park.set_coaster_car_count(coaster, 1), Ok(false));
    assert_eq!(park, before);
}

#[test]
fn fault_repair_retains_car_count_and_configuration_invalidates_mechanics_cache() {
    let (mut park, coaster, _) = fixture();
    let report = park.test_coaster().unwrap();
    park.set_coaster_car_count(coaster, 1).unwrap();
    assert!(!report.input.matches(&park));
    let shorter = park.test_coaster().unwrap();
    assert_eq!(report.moving_ticks, shorter.moving_ticks);
    assert_eq!(
        report.max_speed_units_per_tick,
        shorter.max_speed_units_per_tick
    );
    let rail = park.circuit().unwrap().pieces[4].object;
    park.remove_object(rail).unwrap();
    assert!(park.train_cars().is_empty());
    let bytes = park.to_save_bytes().unwrap();
    let mut loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(loaded.coaster_car_count(coaster), Some(1));
    loaded.install_train_fixture().unwrap();
    assert_eq!(loaded.train_cars().len(), 1);
    assert_eq!(loaded.ride_capacity(coaster), Some(2));
    assert_eq!(
        loaded,
        Park::from_save_bytes(&loaded.to_save_bytes().unwrap()).unwrap()
    );
}

#[test]
fn every_train_length_conserves_seats_fares_and_two_independent_rides() {
    for cars in 1..=3 {
        let (mut park, coaster, carousel) = fixture();
        park.set_coaster_car_count(coaster, cars).unwrap();
        assert_eq!(park.train_cars().len(), usize::from(cars));
        let input = park.coaster_test_input().unwrap();
        assert_eq!(input.cars, cars);
        park.set_ride_ticket_price(coaster, 100).unwrap();
        park.set_ride_open(coaster, true).unwrap();
        park.set_ride_open(carousel, true).unwrap();
        let mut occupied_seen = false;
        for tick in 0..7_200 {
            park.advance(Speed::Normal);
            let mut seats = BTreeSet::new();
            for guest in park.guests() {
                if let Some(visit) = guest.ride_visit.as_ref().filter(|v| v.ride == coaster)
                    && let Some(passenger) = &visit.passenger
                {
                    occupied_seen = true;
                    assert!(passenger.seat < cars * 2);
                    assert!(seats.insert(passenger.seat));
                    assert!(matches!(
                        visit.phase,
                        RideVisitPhase::Boarding | RideVisitPhase::Riding | RideVisitPhase::Exiting
                    ));
                }
            }
            assert!(seats.len() <= usize::from(cars * 2));
            assert_eq!(
                park.ride_journey_stats(coaster).capacity,
                usize::from(cars * 2)
            );
            if tick % 300 == 0 {
                let bytes = park.to_save_bytes().unwrap();
                assert_eq!(Park::from_save_bytes(&bytes).unwrap(), park);
            }
        }
        assert!(occupied_seen);
        assert!(park.ride_by_id(coaster).unwrap().completed > 0);
        assert!(park.ride_by_id(coaster).unwrap().revenue_cents > 0);
        assert!(park.ride_by_id(carousel).unwrap().completed > 0);
    }
}

#[test]
fn schema_eighteen_migrates_exact_body_with_three_cars_and_keeps_passengers() {
    let (mut park, coaster, _) = fixture();
    park.set_ride_open(coaster, true).unwrap();
    for _ in 0..12_000 {
        park.advance(Speed::Normal);
        if park.ride_journey_stats(coaster).riding > 0 {
            break;
        }
    }
    assert!(park.ride_journey_stats(coaster).riding > 0);
    let mut park = crate::ride_costs_tests::without_upkeep(&park);
    let mut old: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    old["schema_version"] = 18.into();
    crate::legacy_cost_test_support::project_old_fixture(&mut old);
    old["content_version"] = 18.into();
    old["park"].as_object_mut().unwrap().remove("coaster_cars");
    let bytes = serde_json::to_vec(&old).unwrap();
    let mut migrated = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(migrated, park);
    for _ in 0..1200 {
        migrated.advance(Speed::Normal);
        park.advance(Speed::Normal);
    }
    assert_eq!(migrated, park);
    assert_eq!(serde_json::to_vec(&old).unwrap(), bytes);
    let mut forged = old.clone();
    forged["park"]["coaster_cars"] = 1.into();
    assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
}

#[test]
fn strict_saves_reject_missing_configuration_over_capacity_seats_and_minimums() {
    let (mut park, coaster, _) = fixture();
    park.set_coaster_car_count(coaster, 1).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    for bad in [serde_json::Value::Null, 0.into(), 4.into(), "1".into()] {
        let mut bad_save = value.clone();
        bad_save["park"]["coaster_cars"] = bad;
        assert!(Park::from_save_bytes(&serde_json::to_vec(&bad_save).unwrap()).is_err());
    }
    let mut missing = value.clone();
    missing["park"]
        .as_object_mut()
        .unwrap()
        .remove("coaster_cars");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&missing).unwrap()).is_err());
    let mut too_many = value.clone();
    too_many["park"]["ride_dispatch"] = serde_json::json!([{"ride": coaster.0,
        "policy": {"minimum_passengers": 3, "maximum_extra_wait_ticks": 900}, "extra_waited_ticks": 0}]);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&too_many).unwrap()).is_err());
    park.set_ride_open(coaster, true).unwrap();
    for _ in 0..12_000 {
        park.advance(Speed::Normal);
        if park.ride_journey_stats(coaster).riding > 0 {
            break;
        }
    }
    let mut bad: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    let guest = bad["park"]["guests"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|g| g["ride_visit"]["ride"] == coaster.0 && !g["ride_visit"]["passenger"].is_null())
        .unwrap();
    guest["ride_visit"]["passenger"]["seat"] = 2.into();
    assert!(Park::from_save_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
}
