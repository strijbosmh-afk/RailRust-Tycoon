use super::*;
use crate::{PathDirection, RideAccessStatus, RideExitStatus, Speed};
use std::collections::BTreeMap;

pub(crate) fn fixture(two: bool) -> (Park, RideId) {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    if two {
        park.install_train_fixture().unwrap();
        let ride = park.ride().unwrap().id;
        park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, ride)
            .unwrap();
        for (y, height, slope) in [
            (12, 8, PathSlope::Flat),
            (13, 8, PathSlope::South),
            (14, 9, PathSlope::Flat),
            (15, 8, PathSlope::North),
        ] {
            park.place_queue_path(TilePos { x: 8, y }, height, slope, ride)
                .unwrap();
        }
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride)
            .unwrap();
        for x in 8..=13 {
            park.place_path(TilePos { x, y: 8 }, 8).unwrap();
        }
        for y in 9..=11 {
            park.place_path(TilePos { x: 13, y }, 8).unwrap();
        }
        park.set_ride_ticket_price(ride, 100).unwrap();
    }
    let ride = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    park.place_ride_entrance(TilePos { x: 18, y: 23 }, 8, PathDirection::East, ride)
        .unwrap();
    for y in 21..=23 {
        park.place_queue_path(TilePos { x: 17, y }, 8, PathSlope::Flat, ride)
            .unwrap();
    }
    park.place_ride_exit(TilePos { x: 20, y: 21 }, 8, PathDirection::South, ride)
        .unwrap();
    park.place_path(TilePos { x: 20, y: 20 }, 8).unwrap();
    park.set_ride_ticket_price(ride, 100).unwrap();
    park.set_ride_open(ride, true).unwrap();
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
    assert_eq!(park.ride_exit_access(ride).status, RideExitStatus::Ready);
    (park, ride)
}

fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
    for _ in 0..24_000 {
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!(
        "carousel predicate not reached: {:?}; {:?}",
        park.carousels(),
        park.guests()
    );
}
fn occupied(park: &Park, id: RideId) -> bool {
    park.guests().iter().any(|g| {
        g.ride_visit
            .as_ref()
            .is_some_and(|v| v.ride == id && v.passenger.is_some())
    })
}
fn snapshot(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(loaded.to_save_bytes().unwrap(), bytes);
    loaded
}

#[test]
fn footprint_validation_is_atomic_and_portals_use_outer_edges() {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(17).unwrap();
    park.advance(Speed::Normal);
    let before = park.clone();
    let cache = park.path_cache_stats();
    for (origin, height) in [
        (TilePos { x: 31, y: 30 }, 8),
        (TilePos { x: 10, y: 18 }, 8),
        (TilePos { x: 19, y: 22 }, 30),
    ] {
        assert!(park.validate_carousel(origin, height).is_err());
        assert!(park.place_carousel(origin, height).is_err());
        assert_eq!(park, before);
        assert_eq!(park.path_cache_stats(), cache);
    }
    let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    let parts = park.carousel(id).unwrap().objects.clone();
    assert_eq!(parts.len(), 9);
    for part in &parts {
        assert_eq!(park.object_ride(*part), Some(id));
        assert!(park.path_neighbors(*part).is_empty());
    }
    for (pos, facing) in [
        (TilePos { x: 20, y: 21 }, PathDirection::South),
        (TilePos { x: 22, y: 23 }, PathDirection::West),
        (TilePos { x: 20, y: 25 }, PathDirection::North),
        (TilePos { x: 18, y: 23 }, PathDirection::East),
    ] {
        assert!(park.validate_ride_entrance(pos, 8, facing, id).is_ok());
        assert!(park.validate_ride_entrance(pos, 9, facing, id).is_err());
        assert!(
            park.validate_ride_entrance(pos, 8, facing.opposite(), id)
                .is_err()
        );
    }
    let installed = park.clone();
    assert_eq!(
        park.place_carousel(TilePos { x: 1, y: 24 }, 8),
        Err(CarouselError::ActiveCarouselExists)
    );
    assert_eq!(park, installed);
    park.remove_object(parts[5]).unwrap();
    assert!(parts.iter().all(|id| park.object(*id).is_none()));
    assert!(park.carousel(id).unwrap().objects.is_empty());
    snapshot(&park);
}

#[test]
fn both_rides_use_independent_seats_payments_and_completion_histories() {
    let (mut park, carousel) = fixture(true);
    let coaster = park.ride().unwrap().id;
    let mut paid = BTreeMap::<RideId, u64>::new();
    let mut seen_transfer = BTreeMap::<RideId, bool>::new();
    for tick in 0..24_000 {
        let old = park.clone();
        park.advance(Speed::Normal);
        let mut seats = std::collections::BTreeSet::new();
        for guest in park.guests() {
            let Some(visit) = &guest.ride_visit else {
                continue;
            };
            let Some(p) = &visit.passenger else {
                continue;
            };
            assert!(seats.insert((visit.ride, p.seat)));
            if let Some(t) = &p.transfer
                && t.progress == 30
            {
                seen_transfer.insert(visit.ride, true);
                let gate = park.object(t.portal).unwrap();
                let point = park.ride_guest_point(guest).unwrap();
                assert_eq!(
                    (point.x, point.y),
                    (
                        gate.pos.x as i32 * PATH_UNITS,
                        gate.pos.y as i32 * PATH_UNITS
                    )
                );
            }
            let oldguest = old.guests().iter().find(|g| g.id == guest.id).unwrap();
            if p.fare_paid
                && oldguest
                    .ride_visit
                    .as_ref()
                    .and_then(|v| v.passenger.as_ref())
                    .is_some_and(|p| !p.fare_paid)
            {
                assert!(!old.ride_is_running(visit.ride));
                assert!(park.ride_is_running(visit.ride));
                assert_eq!(oldguest.wallet_cents - guest.wallet_cents, p.fare_cents);
                *paid.entry(visit.ride).or_default() += u64::from(p.fare_cents);
            }
        }
        if tick % 500 == 0 {
            snapshot(&park);
        }
        if park.ride_by_id(coaster).unwrap().completed > 0
            && park.ride_by_id(carousel).unwrap().completed > 0
        {
            break;
        }
    }
    for id in [coaster, carousel] {
        let ride = park.ride_by_id(id).unwrap();
        assert!(ride.completed > 0);
        assert!(seen_transfer.contains_key(&id));
        assert_eq!(paid[&id], ride.revenue_cents);
        assert_eq!(
            ride.completed,
            park.guests()
                .iter()
                .flat_map(|g| &g.ride_history)
                .filter(|h| h.ride == id)
                .map(|h| h.completed)
                .sum::<u64>()
        );
    }
    assert!(park.guests().iter().all(|g| g.ride_history.len() <= 2));
    snapshot(&park);
}

#[test]
fn close_missing_exit_and_whole_ride_removal_preserve_owner_history() {
    let (mut park, id) = fixture(true);
    let coaster = park.ride().unwrap().id;
    seek(&mut park, |p| p.ride_is_running(id) && occupied(p, id));
    let part = park.carousel(id).unwrap().objects[4];
    let before = park.clone();
    assert_eq!(park.remove_carousel(id), Err(CarouselError::Occupied));
    assert!(park.remove_object(part).is_none());
    assert_eq!(park, before);
    let exit = park.ride_by_id(id).unwrap().exit.unwrap();
    park.remove_object(exit.object).unwrap();
    let poses = park.ride_guest_positions();
    park.set_ride_open(id, false).unwrap();
    assert_eq!(park.ride_guest_positions(), poses);
    assert!(park.ride_is_open(coaster));
    seek(&mut park, |p| !p.ride_is_running(id));
    let held = park.carousel(id).unwrap().clone();
    for _ in 0..100 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.carousel(id).unwrap(), &held);
    assert!(occupied(&park, id));
    snapshot(&park);
    park.place_ride_exit(TilePos { x: 20, y: 21 }, 8, PathDirection::South, id)
        .unwrap();
    seek(&mut park, |p| !occupied(p, id));
    let history = park.ride_by_id(id).unwrap().clone();
    let gates = [
        history.entrance.unwrap().object,
        history.exit.unwrap().object,
    ];
    let body = park.carousel(id).unwrap().objects.clone();
    park.remove_carousel(id).unwrap();
    assert!(
        body.iter()
            .chain(&gates)
            .all(|id| park.object(*id).is_none())
    );
    let tombstone = park.ride_by_id(id).unwrap();
    assert_eq!(tombstone.revenue_cents, history.revenue_cents);
    assert_eq!(tombstone.completed, history.completed);
    assert!(tombstone.entrance.is_none() && tombstone.exit.is_none());
    let fresh = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    assert_ne!(fresh, id);
    assert_eq!(park.ride_by_id(fresh).unwrap().revenue_cents, 0);
    assert_eq!(
        park.ride_by_id(id).unwrap().revenue_cents,
        history.revenue_cents
    );
    snapshot(&park);
}

#[test]
fn complete_tick_speeds_pause_and_active_load_agree_for_two_rides() {
    let (mut normal, id) = fixture(true);
    seek(&mut normal, |p| occupied(p, id));
    let mut fast = snapshot(&normal);
    let mut fastest = normal.clone();
    let before = normal.clone();
    normal.advance(Speed::Paused);
    assert_eq!(normal, before);
    for _ in 0..14_400 {
        normal.advance(Speed::Normal);
    }
    for _ in 0..7_200 {
        fast.advance(Speed::Fast);
    }
    for _ in 0..3_600 {
        fastest.advance(Speed::Fastest);
    }
    assert_eq!(normal, fast);
    assert_eq!(normal, fastest);
    assert!(normal.all_rides().all(|r| r.completed > 0));
    snapshot(&normal);
}

#[test]
fn choice_respects_price_reachability_seed_and_stable_ride_ids() {
    let (mut park, id) = fixture(true);
    let coaster = park.ride().unwrap().id;
    let graph = crate::crowds::PathGraph::new(&park.objects);
    for guest in &mut park.guests {
        guest.next = None;
        guest.destination = None;
        guest.route.clear();
        guest.wait_ticks = 0;
        guest.progress = 0;
        guest.state = crate::GuestState::Waiting;
    }
    let accesses = park
        .all_rides()
        .map(|r| (r.id, park.ride_access(r.id)))
        .collect();
    let rng = park.crowd_rng;
    let first = park.choose_guest_rides(&graph, &accesses);
    assert!(!first.is_empty());
    assert_eq!(first, park.clone().choose_guest_rides(&graph, &accesses));
    assert_eq!(rng, park.crowd_rng);
    let preferences: std::collections::BTreeSet<_> = park
        .guests
        .iter()
        .map(|g| format!("{:?}", park.guest_preferred_ride_kind(g.id).unwrap()))
        .collect();
    assert_eq!(preferences.len(), 2);
    park.set_ride_ticket_price(id, crate::MAX_TICKET_PRICE_CENTS)
        .unwrap();
    assert!(
        park.choose_guest_rides(&graph, &accesses)
            .values()
            .all(|ride| *ride == coaster)
    );
    park.set_ride_ticket_price(id, 100).unwrap();
    park.set_ride_ticket_price(coaster, crate::MAX_TICKET_PRICE_CENTS)
        .unwrap();
    assert!(
        park.choose_guest_rides(&graph, &accesses)
            .values()
            .all(|ride| *ride == id)
    );
    park.set_ride_open(id, false).unwrap();
    assert!(
        park.choose_guest_rides(&graph, &accesses)
            .values()
            .all(|ride| *ride == coaster)
    );
}

#[test]
fn lifetime_capacity_and_malformed_footprints_are_strict() {
    let mut park = Park::new(32, 32);
    for _ in 0..MAX_CAROUSEL_RECORDS {
        let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        park.remove_carousel(id).unwrap();
    }
    let before = park.clone();
    assert_eq!(
        park.place_carousel(TilePos { x: 19, y: 22 }, 8),
        Err(CarouselError::HistoryFull)
    );
    assert_eq!(park, before);
    snapshot(&park);
    let (mut park, id) = fixture(false);
    park.set_ride_open(id, false).unwrap();
    let valid: serde_json::Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    for field in ["objects", "phase", "cycles", "progress"] {
        let mut bad = valid.clone();
        match field {
            "objects" => {
                bad["park"]["carousels"][0][field][0] =
                    bad["park"]["carousels"][0][field][1].clone()
            }
            "phase" => bad["park"]["carousels"][0][field] = serde_json::json!("Running"),
            "cycles" => bad["park"]["carousels"][0][field] = serde_json::json!(100),
            _ => bad["park"]["carousels"][0][field] = serde_json::json!(720),
        }
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{field}"
        );
    }
}

#[test]
fn carousel_runtime_clock_and_removed_portals_have_positive_controls() {
    let mut park = Park::new(32, 32);
    let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    park.set_ride_open(id, true).unwrap();
    assert!(validate_carousels(&park).is_ok());
    let mut impossible_station = park.clone();
    impossible_station.carousels[0].dwell_ticks = 0;
    assert_eq!(
        validate_carousels(&impossible_station).unwrap_err(),
        "carousel operation counters or phase are invalid"
    );
    for _ in 0..100 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.carousel(id).unwrap().phase, CarouselPhase::Running);
    assert!(park.carousel(id).unwrap().progress > 0);
    assert!(validate_carousels(&park).is_ok());
    snapshot(&park);
    let mut impossible = park.clone();
    impossible.ticks = 0;
    assert_eq!(
        validate_carousels(&impossible).unwrap_err(),
        "carousel operation counters or phase are invalid"
    );
    park.set_ride_open(id, false).unwrap();
    seek(&mut park, |p| !p.ride_is_running(id));
    let gate = park
        .place_ride_entrance(TilePos { x: 18, y: 23 }, 8, PathDirection::East, id)
        .unwrap();
    let metadata = park.ride_by_id(id).unwrap().entrance;
    park.remove_carousel(id).unwrap();
    assert!(park.object(gate).is_none());
    assert!(validate_carousels(&park).is_ok());
    snapshot(&park);
    park.ride_by_id_mut(id).unwrap().entrance = metadata;
    assert_eq!(
        validate_carousels(&park).unwrap_err(),
        "removed carousel has active operation state"
    );
}
