use super::*;
use crate::Speed;
use serde_json::{Value, json};

fn fixture() -> Park {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    for (x, kind) in [
        (10, ServiceKind::Food),
        (15, ServiceKind::Drink),
        (18, ServiceKind::Toilet),
    ] {
        park.place_service(TilePos { x, y: 21 }, 8, PathDirection::North, kind)
            .unwrap();
    }
    park
}
fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
    for _ in 0..30_000 {
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!(
        "service state not reached: {:?}",
        park.guest_needs_summary()
    );
}
fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(&loaded, park);
    assert_eq!(bytes, loaded.to_save_bytes().unwrap());
    loaded
}
fn using(park: &Park) -> Option<usize> {
    park.guests.iter().position(|guest| {
        guest
            .service_visit
            .as_ref()
            .is_some_and(|visit| visit.phase == ServiceVisitPhase::Using)
    })
}
fn ledger(park: &Park) {
    validate_services(park).unwrap();
    crate::economy::validate_economy(park).unwrap();
    for service in &park.services {
        assert!(park.service_stats(service.id).using <= 1);
    }
    assert_eq!(
        park.guests
            .iter()
            .map(|guest| guest.services_used)
            .sum::<u64>(),
        park.services
            .iter()
            .map(|service| service.served)
            .sum::<u64>()
    );
    assert_eq!(
        park.guests
            .iter()
            .map(|guest| guest.service_spent_cents)
            .sum::<u64>(),
        park.services
            .iter()
            .map(|service| service.revenue_cents)
            .sum::<u64>()
    );
}

pub(crate) fn strip_services(value: Value) -> Value {
    let mut value = crate::finances::tests::strip_finances(value);
    value["schema_version"] = json!(7);
    value["content_version"] = json!(7);
    assert!(value["park"]["services"].as_array().unwrap().is_empty());
    let park = value["park"].as_object_mut().unwrap();
    park.remove("services");
    park.remove("next_service_id");
    for guest in value["park"]["guests"].as_array_mut().unwrap() {
        assert_eq!(guest["service_spent_cents"], json!(0));
        assert_eq!(guest["services_used"], json!(0));
        for field in [
            "needs",
            "service_visit",
            "service_cooldown",
            "service_spent_cents",
            "services_used",
        ] {
            guest.as_object_mut().unwrap().remove(field);
        }
    }
    value
}
pub(crate) fn reset_needs(park: &mut Park) {
    assert!(park.services.is_empty());
    crate::finances::tests::reset_finances(park);
    for guest in &mut park.guests {
        guest.needs = GuestNeeds::for_guest(park.crowd_seed.unwrap_or(0), guest.id);
        guest.service_visit = None;
        guest.service_cooldown = 0;
        guest.service_spent_cents = 0;
        guest.services_used = 0;
    }
}

#[test]
fn placement_preview_commit_caps_and_facing_share_one_atomic_contract() {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    park.advance(Speed::Normal);
    let before = park.clone();
    let cache = park.path_cache_stats();
    let pos = TilePos { x: 10, y: 21 };
    assert_eq!(
        park.validate_service(pos, 8, PathDirection::North, ServiceKind::Food),
        Ok(())
    );
    assert_eq!(park, before);
    assert_eq!(park.path_cache_stats(), cache);
    let id = park
        .place_service(pos, 8, PathDirection::North, ServiceKind::Food)
        .unwrap();
    let object = park.service(id).unwrap().object.unwrap();
    let path = park.service_access(id).public_path.unwrap();
    assert_eq!(park.service_access(id).status, ServiceAccessStatus::Ready);
    assert!(park.geometric_path_neighbors(object).contains(&path));
    assert!(park.path_neighbors(object).is_empty());
    assert!(!park.path_neighbors(path).contains(&object));
    let priced = park.clone();
    assert_eq!(
        park.set_service_price(id, MAX_SERVICE_PRICE_CENTS + 1),
        Err(ServiceError::PriceTooHigh {
            price: MAX_SERVICE_PRICE_CENTS + 1,
            max: MAX_SERVICE_PRICE_CENTS
        })
    );
    assert_eq!(
        park.set_service_price(ServiceId(999), 0),
        Err(ServiceError::UnknownService(ServiceId(999)))
    );
    assert_eq!(park, priced);
    let occupied = park.clone();
    let expected = park
        .validate_service(pos, 8, PathDirection::North, ServiceKind::Food)
        .unwrap_err();
    assert_eq!(
        park.place_service(pos, 8, PathDirection::North, ServiceKind::Food),
        Err(expected)
    );
    assert_eq!(park, occupied);
    assert_eq!(park.path_cache_stats(), cache);
    assert!(
        park.place_object(
            TilePos { x: 1, y: 1 },
            ObjectKind::Service,
            crate::Axis::X,
            0,
            3
        )
        .is_err()
    );
    assert!(
        park.validate_service(
            TilePos { x: 1, y: 1 },
            30,
            PathDirection::East,
            ServiceKind::Toilet
        )
        .is_err()
    );
    park.remove_object(object).unwrap();
    assert_eq!(park.service(id).unwrap().object, None);
    assert_eq!(park.service_access(id).status, ServiceAccessStatus::Removed);
    let replacement = park
        .place_service(pos, 8, PathDirection::South, ServiceKind::Drink)
        .unwrap();
    assert!(replacement > id);
    assert_ne!(park.service(replacement).unwrap().object, Some(object));
    assert_eq!(
        park.service_access(replacement).status,
        ServiceAccessStatus::Disconnected
    );
    roundtrip(&park);
}

#[test]
fn service_front_connects_only_to_matching_public_slope_ends() {
    for direction in PathDirection::ALL {
        let mut park = Park::new(32, 32);
        let pos = TilePos { x: 5, y: 5 };
        let id = park
            .place_service(pos, 9, direction, ServiceKind::Food)
            .unwrap();
        let path_pos = direction.neighbor(pos).unwrap();
        let rise = match direction.opposite() {
            PathDirection::North => PathSlope::North,
            PathDirection::East => PathSlope::East,
            PathDirection::South => PathSlope::South,
            PathDirection::West => PathSlope::West,
        };
        let slope = park.place_sloped_path(path_pos, 8, rise).unwrap();
        assert_eq!(park.service_access(id).public_path, Some(slope));
        park.remove_object(slope).unwrap();
        park.place_path(path_pos, 8).unwrap();
        assert_eq!(
            park.service_access(id).status,
            ServiceAccessStatus::Disconnected
        );
        roundtrip(&park);
    }
}

#[test]
fn needs_are_seeded_and_tick_order_applies_each_critical_crossing_once() {
    let mut park = fixture();
    let rng = park.crowd_rng;
    for guest in &park.guests {
        assert_eq!(guest.needs, GuestNeeds::for_guest(42, guest.id));
        assert!(
            ServiceKind::ALL
                .iter()
                .all(|kind| (10..=30).contains(&guest.needs.value(*kind)))
        );
    }
    let before = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, before);
    assert_eq!(park.crowd_rng, rng);
    // The test targets the pure needs stage at a tick shared by all cadences.
    park.guests[0].needs = GuestNeeds {
        hunger: 84,
        thirst: 84,
        toilet: 84,
    };
    park.ticks = 360;
    park.step_needs();
    assert_eq!(
        park.guests[0].needs,
        GuestNeeds {
            hunger: 85,
            thirst: 85,
            toilet: 85
        }
    );
    assert_eq!(park.guests[0].satisfaction, 55);
    assert_eq!(park.guests[0].thought, GuestThought::NeedsToilet);
    park.ticks = 720;
    park.step_needs();
    assert_eq!(park.guests[0].satisfaction, 55);
    roundtrip(&park);
}

#[test]
fn natural_service_use_locks_price_pays_once_and_relieves_only_on_completion() {
    let mut park = fixture();
    seek(&mut park, |park| using(park).is_some());
    let index = using(&park).unwrap();
    let old = park.guests[index].clone();
    let visit = old.service_visit.as_ref().unwrap().clone();
    let service = park.service(visit.service).unwrap().clone();
    assert_eq!(visit.progress, 0);
    assert!(visit.price_cents > 0);
    park.set_service_price(visit.service, 950).unwrap();
    let before = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, before);
    let mut loaded = roundtrip(&park);
    for step in 1..=SERVICE_USE_TICKS {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
        assert_eq!(park, loaded);
        ledger(&park);
        if step < SERVICE_USE_TICKS {
            assert_eq!(park.guests[index].wallet_cents, old.wallet_cents);
            assert_eq!(park.guests[index].services_used, old.services_used);
        }
    }
    let guest = &park.guests[index];
    assert_eq!(guest.wallet_cents, old.wallet_cents - visit.price_cents);
    assert_eq!(
        guest.service_spent_cents,
        old.service_spent_cents + u64::from(visit.price_cents)
    );
    assert_eq!(guest.services_used, old.services_used + 1);
    assert_eq!(guest.needs.value(service.kind), 20);
    assert!(guest.service_visit.is_none() && guest.ride_visit.is_none());
    assert_eq!(guest.thought, service.kind.relief_thought());
    let spent = guest.service_spent_cents;
    for _ in 0..100 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.guests[index].service_spent_cents, spent);
    roundtrip(&park);
}

#[test]
fn closing_removing_or_losing_access_cancels_without_payment_or_teleport() {
    let mut source = fixture();
    seek(&mut source, |park| using(park).is_some());
    for action in 0..3 {
        let mut park = source.clone();
        let index = using(&park).unwrap();
        let old = park.guests[index].clone();
        let visit = old.service_visit.as_ref().unwrap().clone();
        let position = park.guest_position(old.id).unwrap().point;
        match action {
            0 => park.set_service_open(visit.service, false).unwrap(),
            1 => {
                park.remove_object(park.service(visit.service).unwrap().object.unwrap())
                    .unwrap();
            }
            _ => {
                park.remove_object(visit.access).unwrap();
            }
        }
        assert_eq!(park.guest_position(old.id).unwrap().point, position);
        let guest = &park.guests[index];
        assert!(guest.service_visit.is_none());
        assert_eq!(guest.wallet_cents, old.wallet_cents);
        assert_eq!(guest.service_spent_cents, old.service_spent_cents);
        assert_eq!(guest.services_used, old.services_used);
        assert_eq!(guest.needs, old.needs);
        assert_eq!(guest.thought, GuestThought::ServiceCancelled);
        ledger(&park);
        roundtrip(&park);
    }
    seek(&mut source, |park| {
        park.guests.iter().any(|guest| {
            guest
                .service_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == ServiceVisitPhase::Approaching)
                && guest.progress == 7
        })
    });
    let index = source
        .guests
        .iter()
        .position(|guest| {
            guest
                .service_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == ServiceVisitPhase::Approaching)
                && guest.progress == 7
        })
        .unwrap();
    let old = source.guests[index].clone();
    source
        .set_service_open(old.service_visit.as_ref().unwrap().service, false)
        .unwrap();
    assert_eq!(source.guests[index].current, old.current);
    assert_eq!(source.guests[index].next, old.next);
    assert_eq!(source.guests[index].progress, old.progress);
    assert_eq!(source.guests[index].destination, old.next);
    roundtrip(&source);
}

#[test]
fn unmet_and_unaffordable_needs_retry_then_use_repaired_service() {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    seek(&mut park, |park| {
        park.guests
            .iter()
            .any(|guest| guest.thought == GuestThought::ServiceUnavailable)
    });
    assert!(
        park.guests
            .iter()
            .all(|guest| guest.service_spent_cents == 0)
    );
    let id = park
        .place_service(
            TilePos { x: 15, y: 21 },
            8,
            PathDirection::North,
            ServiceKind::Drink,
        )
        .unwrap();
    park.set_service_price(id, 3000).unwrap();
    seek(&mut park, |park| {
        park.service(id).unwrap().unaffordable_rejections > 0
    });
    assert_eq!(park.service(id).unwrap().served, 0);
    park.set_service_price(id, 100).unwrap();
    seek(&mut park, |park| park.service(id).unwrap().served > 0);
    let earned = park.service(id).unwrap().revenue_cents;
    let served = park.service(id).unwrap().served;
    park.remove_object(park.service(id).unwrap().object.unwrap())
        .unwrap();
    assert_eq!(park.service(id).unwrap().revenue_cents, earned);
    assert_eq!(park.service(id).unwrap().served, served);
    assert_eq!(
        park.set_service_open(id, true),
        Err(ServiceError::RemovedService(id))
    );
    ledger(&park);
    roundtrip(&park);
}

#[test]
fn use_slots_wait_timeout_and_stable_id_ties_are_deterministic() {
    let mut park = fixture();
    let service = park.services[0].id;
    let path = *park
        .object(park.service_access(service).public_path.unwrap())
        .unwrap();
    for guest in &mut park.guests {
        guest.current = Some(path.id);
        guest.next = None;
        guest.destination = None;
        guest.route.clear();
        guest.progress = 0;
        guest.anchor = path.pos;
        guest.height = path.height;
        guest.anchor_slope = path.path_slope;
        guest.state = GuestState::Waiting;
        guest.wait_ticks = 0;
        guest.needs.hunger = 60;
        guest.service_visit = Some(ServiceVisit {
            service,
            access: path.id,
            phase: ServiceVisitPhase::Waiting,
            price_cents: 0,
            progress: 0,
            waited_ticks: 899,
        });
    }
    for guest in &mut park.guests[..2] {
        guest.service_visit.as_mut().unwrap().waited_ticks = 0;
    }
    let first = park.guests[0].id;
    let second = park.guests[1].id;
    assert!(first < second);
    let mut ordered = park.clone();
    park.guests.reverse();
    park.advance(Speed::Normal);
    ordered.advance(Speed::Normal);
    assert_eq!(park.service_stats(service).using, 1);
    assert_eq!(
        park.guests
            .iter()
            .find(|guest| guest.id == first)
            .unwrap()
            .service_visit
            .as_ref()
            .unwrap()
            .phase,
        ServiceVisitPhase::Using
    );
    let still_waiting = park
        .guests
        .iter()
        .find(|guest| guest.id == second)
        .unwrap()
        .service_visit
        .as_ref()
        .unwrap();
    assert_eq!(still_waiting.phase, ServiceVisitPhase::Waiting);
    assert_eq!(still_waiting.waited_ticks, 1);
    assert!(
        park.guests
            .iter()
            .filter(|guest| guest.id != first && guest.id != second)
            .all(|guest| guest.service_visit.is_none()
                && guest.services_used == 0
                && guest.service_cooldown == SERVICE_COOLDOWN_TICKS)
    );
    park.guests.sort_by_key(|guest| guest.id);
    assert_eq!(park, ordered);
    roundtrip(&park);
}

#[test]
fn each_service_history_bounds_its_own_revenue_even_when_all_ledgers_balance() {
    let mut park = fixture();
    for index in 0..2 {
        park.guests[index].wallet_cents = 0;
        park.guests[index].spent_cents = u64::from(crate::INITIAL_GUEST_WALLET_CENTS);
        park.guests[index].service_spent_cents = u64::from(crate::INITIAL_GUEST_WALLET_CENTS);
        park.guests[index].services_used = 1;
        park.services[index].served = 1;
        park.services[index].revenue_cents = u64::from(crate::INITIAL_GUEST_WALLET_CENTS);
    }
    park.cash_cents += 2 * u64::from(crate::INITIAL_GUEST_WALLET_CENTS);
    crate::finances::tests::reset_finances(&mut park);
    ledger(&park);
    roundtrip(&park);

    // Redistributing earned money preserves every park/guest total but gives
    // one facility more revenue than its one customer could ever have paid.
    let mut forged = park.clone();
    let redistributed = forged.services[1].revenue_cents;
    forged.services[0].revenue_cents += redistributed;
    forged.services[1].revenue_cents = 0;
    crate::economy::validate_economy(&forged).unwrap();
    assert_eq!(
        validate_services(&forged),
        Err("service identity, price or history is invalid".into())
    );
    assert!(forged.to_save_bytes().is_err());

    let mut json: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    json["park"]["services"][0]["revenue_cents"] =
        json!(2 * u64::from(crate::INITIAL_GUEST_WALLET_CENTS));
    json["park"]["services"][1]["revenue_cents"] = json!(0);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
}

#[test]
fn full_speed_schedules_and_save_continuation_preserve_services_and_ledgers() {
    let mut normal = fixture();
    let mut fast = normal.clone();
    let mut fastest = normal.clone();
    let drink = normal.services[1].id;
    for group in 0..3000 {
        if [750, 850, 1300].contains(&group) {
            for park in [&mut normal, &mut fast, &mut fastest] {
                if group == 1300 {
                    park.set_service_price(drink, 50).unwrap();
                } else {
                    park.set_service_open(drink, group == 850).unwrap();
                }
            }
        }
        for _ in 0..4 {
            normal.advance(Speed::Normal);
        }
        fast.advance(Speed::Fast);
        fast.advance(Speed::Fast);
        fastest.advance(Speed::Fastest);
        assert_eq!(normal, fast);
        assert_eq!(normal, fastest);
        if group % 300 == 0 {
            ledger(&normal);
            fastest = roundtrip(&fastest);
        }
    }
    assert!(normal.services.iter().all(|service| service.served > 0));
    assert!(
        normal
            .guests
            .iter()
            .any(|guest| guest.service_spent_cents > 0)
    );
    ledger(&normal);
    roundtrip(&normal);
}

#[test]
fn v7_migration_preserves_old_feedback_money_rng_and_strict_shape() {
    let mut source = Park::new(32, 32);
    source.install_crowd_fixture(42).unwrap();
    source.install_train_fixture().unwrap();
    source.guests[0].satisfaction = 55;
    source.guests[0].thought = GuestThought::DisappointedByRide;
    for _ in 0..90 {
        source.advance(Speed::Normal);
    }
    let old = strip_services(serde_json::from_slice(&source.to_save_bytes().unwrap()).unwrap());
    let bytes = serde_json::to_vec(&old).unwrap();
    let mut loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(
        strip_services(serde_json::from_slice(&loaded.to_save_bytes().unwrap()).unwrap()),
        old
    );
    assert_eq!(loaded.crowd_rng, source.crowd_rng);
    for guest in &loaded.guests {
        assert_eq!(guest.needs, GuestNeeds::for_guest(42, guest.id));
    }
    let mut repeated = Park::from_save_bytes(&bytes).unwrap();
    for _ in 0..6000 {
        loaded.advance(Speed::Normal);
        repeated.advance(Speed::Normal);
        assert_eq!(loaded, repeated);
    }
    assert!(loaded.guests.iter().any(|guest| guest.needs.thirst == 100));
    for (path, field, added) in [
        ("/park", "services", json!([])),
        (
            "/park/guests/0",
            "needs",
            json!({"hunger":20,"thirst":20,"toilet":20}),
        ),
        ("/park/guests/0", "thought", json!("Hungry")),
    ] {
        let mut forged = old.clone();
        forged.pointer_mut(path).unwrap()[field] = added;
        assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
    }
    roundtrip(&loaded);
}

#[test]
fn malformed_service_references_quotes_histories_and_missing_fields_are_rejected() {
    let mut park = fixture();
    seek(&mut park, |park| using(park).is_some());
    let index = using(&park).unwrap();
    let stem = format!("/park/guests/{index}");
    let base: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    for (path, value) in [
        (format!("{stem}/needs/thirst"), json!(101)),
        (format!("{stem}/service_visit/service"), json!(99999)),
        (format!("{stem}/service_visit/access"), json!(99999)),
        (format!("{stem}/service_visit/progress"), json!(90)),
        (format!("{stem}/service_visit/price_cents"), json!(2001)),
        (format!("{stem}/service_visit/waited_ticks"), json!(900)),
        (format!("{stem}/service_cooldown"), json!(1)),
        (format!("{stem}/service_spent_cents"), json!(1)),
        (format!("{stem}/services_used"), json!(u64::MAX)),
        ("/park/services/0/open".into(), json!(false)),
        ("/park/services/0/revenue_cents".into(), json!(1)),
        ("/park/next_service_id".into(), json!(1)),
    ] {
        let mut forged = base.clone();
        *forged.pointer_mut(&path).unwrap() = value;
        // Closing a different, unoccupied service is legal. Target the active one.
        if path == "/park/services/0/open" {
            let active = park.guests[index].service_visit.as_ref().unwrap().service;
            let s = park
                .services
                .iter()
                .position(|service| service.id == active)
                .unwrap();
            forged["park"]["services"][s]["open"] = json!(false);
        }
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err(),
            "accepted {path}"
        );
    }
    for (path, fields) in [
        ("/park".to_string(), vec!["services", "next_service_id"]),
        (
            stem.clone(),
            vec![
                "needs",
                "service_visit",
                "service_cooldown",
                "service_spent_cents",
                "services_used",
            ],
        ),
    ] {
        for field in fields {
            let mut forged = base.clone();
            forged
                .pointer_mut(&path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
        }
    }
    for (schema, content) in [(10, 10), (8, 7), (7, 8)] {
        let mut forged = base.clone();
        forged["schema_version"] = json!(schema);
        forged["content_version"] = json!(content);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
    }
    let mut missing_object = base.clone();
    missing_object["park"]["services"][0]
        .as_object_mut()
        .unwrap()
        .remove("object");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&missing_object).unwrap()).is_err());
    let mut duplicate = park.clone();
    let other = (index + 1) % duplicate.guests.len();
    let other_id = duplicate.guests[other].id;
    duplicate.guests[other] = duplicate.guests[index].clone();
    duplicate.guests[other].id = other_id;
    assert!(duplicate.to_save_bytes().is_err());
    roundtrip(&park);
}

#[test]
fn active_service_visits_require_the_matching_need_to_remain_urgent() {
    for phase in [ServiceVisitPhase::Approaching, ServiceVisitPhase::Using] {
        let mut park = fixture();
        seek(&mut park, |park| {
            park.guests.iter().any(|guest| {
                guest
                    .service_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == phase)
            })
        });
        let index = park
            .guests
            .iter()
            .position(|guest| {
                guest
                    .service_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == phase)
            })
            .unwrap();
        let id = park.guests[index].service_visit.as_ref().unwrap().service;
        let kind = park.service(id).unwrap().kind;
        *park.guests[index].needs.value_mut(kind) = NEED_SEEK_THRESHOLD;
        ledger(&park);
        roundtrip(&park);

        let mut forged = park.clone();
        *forged.guests[index].needs.value_mut(kind) = NEED_SEEK_THRESHOLD - 1;
        assert_eq!(
            validate_services(&forged),
            Err("guest service visit conflicts with access or navigation".into())
        );
        assert!(forged.to_save_bytes().is_err());

        let mut json: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        let field = match kind {
            ServiceKind::Food => "hunger",
            ServiceKind::Drink => "thirst",
            ServiceKind::Toilet => "toilet",
        };
        json["park"]["guests"][index]["needs"][field] = json!(NEED_SEEK_THRESHOLD - 1);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
    }
}

#[test]
fn service_history_capacity_and_counter_exhaustion_reject_atomically() {
    let mut park = Park::new(32, 32);
    let pos = TilePos { x: 2, y: 2 };
    for _ in 0..MAX_SERVICES {
        let id = park
            .place_service(pos, 0, PathDirection::North, ServiceKind::Toilet)
            .unwrap();
        park.remove_object(park.service(id).unwrap().object.unwrap())
            .unwrap();
    }
    let before = park.clone();
    assert!(
        park.place_service(pos, 0, PathDirection::North, ServiceKind::Toilet)
            .is_err()
    );
    assert_eq!(park, before);
    roundtrip(&park);
    let mut park = fixture();
    park.services[0].served = MAX_SERVICE_USES;
    park.guests[0].services_used = MAX_SERVICE_USES;
    crate::finances::tests::reset_finances(&mut park);
    roundtrip(&park);
    for _ in 0..6000 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.services[0].served, MAX_SERVICE_USES);
    ledger(&park);
    roundtrip(&park);
}

#[test]
fn the_last_reserved_service_use_completes_then_blocks_further_use() {
    let mut park = fixture();
    seek(&mut park, |park| using(park).is_some());
    let index = using(&park).unwrap();
    let id = park.guests[index].service_visit.as_ref().unwrap().service;
    let service_index = park
        .services
        .iter()
        .position(|service| service.id == id)
        .unwrap();
    park.guests[index].services_used = MAX_SERVICE_USES - 1;
    park.services[service_index].served = MAX_SERVICE_USES - 1;
    crate::finances::tests::reset_finances(&mut park);
    let mut loaded = roundtrip(&park);
    for _ in 0..SERVICE_USE_TICKS {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
        assert_eq!(park, loaded);
        ledger(&park);
    }
    assert_eq!(park.guests[index].services_used, MAX_SERVICE_USES);
    assert_eq!(park.service(id).unwrap().served, MAX_SERVICE_USES);
    roundtrip(&park);
    for _ in 0..6000 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.service(id).unwrap().served, MAX_SERVICE_USES);
    roundtrip(&park);
}

#[test]
fn service_selection_is_stable_when_service_storage_is_reversed() {
    let mut first = fixture();
    let mut reversed = first.clone();
    reversed.services.reverse();
    // Public roaming retains its established guest-vector RNG order. Only
    // service metadata order changes; counter admission separately uses GuestId.
    for _ in 0..7200 {
        first.advance(Speed::Normal);
        reversed.advance(Speed::Normal);
    }
    assert_eq!(first.guests, reversed.guests);
    assert_eq!(first.crowd_rng, reversed.crowd_rng);
    assert_eq!(first.cash_cents, reversed.cash_cents);
    reversed.services.sort_by_key(|service| service.id);
    assert_eq!(first, reversed);
    assert!(first.guest_needs_summary().services_used > 0);
    roundtrip(&first);
}

#[test]
fn loading_above_critical_does_not_repeat_the_crossing_penalty() {
    let mut park = fixture();
    park.guests[0].needs = GuestNeeds {
        hunger: 85,
        thirst: 90,
        toilet: 100,
    };
    park.guests[0].satisfaction = 45;
    let mut loaded = roundtrip(&park);
    park.ticks = 360;
    loaded.ticks = 360;
    park.step_needs();
    loaded.step_needs();
    assert_eq!(park, loaded);
    assert_eq!(park.guests[0].satisfaction, 45);
    assert_eq!(
        park.guests[0].needs,
        GuestNeeds {
            hunger: 86,
            thirst: 91,
            toilet: 100
        }
    );
}

#[test]
fn critical_needs_continue_during_a_reserved_passenger_journey() {
    let mut park = crate::experience::tests::fixture(true, 0);
    seek(&mut park, |park| {
        park.guests.iter().any(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == crate::RideVisitPhase::Riding)
        })
    });
    let index = park
        .guests
        .iter()
        .position(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == crate::RideVisitPhase::Riding)
        })
        .unwrap();
    park.guests[index].needs = GuestNeeds {
        hunger: 84,
        thirst: 84,
        toilet: 84,
    };
    park.ticks = (park.ticks / 360 + 1) * 360 - 1;
    let satisfaction = park.guests[index].satisfaction;
    let mut loaded = roundtrip(&park);
    park.advance(Speed::Normal);
    loaded.advance(Speed::Normal);
    assert_eq!(park, loaded);
    assert_eq!(
        park.guests[index].satisfaction,
        satisfaction.saturating_sub(15)
    );
    assert_eq!(
        park.guests[index].needs,
        GuestNeeds {
            hunger: 85,
            thirst: 85,
            toilet: 85
        }
    );
    assert!(park.guests[index].service_visit.is_none());
    assert!(park.guests[index].ride_visit.is_some());
    roundtrip(&park);
}
