use super::*;
use crate::{RideVisitPhase, Speed};
use serde_json::{Value, json};

fn fixture() -> Park {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
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
    assert_eq!(park.ride_exit_access(ride).status, RideExitStatus::Ready);
    park
}
fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
    for _ in 0..12_000 {
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!(
        "journey state not reached: {:?} {:?}",
        park.ride_journey_stats(park.ride().unwrap().id),
        park.train()
    );
}
fn has_phase(park: &Park, phase: RideVisitPhase) -> bool {
    park.guests().iter().any(|guest| {
        guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| visit.phase == phase)
    })
}
fn passengers(park: &Park) -> usize {
    park.guests()
        .iter()
        .filter(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.passenger.is_some())
        })
        .count()
}
fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(loaded.to_save_bytes().unwrap(), bytes);
    loaded
}

#[test]
fn exit_validation_connectivity_and_public_graph_are_distinct() {
    let mut park = fixture();
    let ride = park.ride().unwrap().id;
    let exit = park.ride().unwrap().exit.unwrap();
    let before = park.clone();
    let cache = park.path_cache_stats();
    assert_eq!(
        park.validate_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride),
        Err(RideAccessError::ExitAlreadyExists)
    );
    assert_eq!(park, before);
    assert_eq!(park.path_cache_stats(), cache);
    let landing = park.ride_exit_access(ride).public_exit.unwrap();
    assert!(
        park.geometric_path_neighbors(exit.object)
            .contains(&landing)
    );
    assert!(park.path_neighbors(exit.object).is_empty());
    assert!(!park.path_neighbors(landing).contains(&exit.object));
    let bridge = park
        .objects()
        .iter()
        .find(|object| object.is_public_path() && object.pos == TilePos { x: 13, y: 11 })
        .unwrap()
        .id;
    park.remove_object(bridge).unwrap();
    let access = park.ride_exit_access(ride);
    assert_eq!(access.status, RideExitStatus::PublicDisconnected);
    assert_eq!(access.public_exit, Some(landing));
    roundtrip(&park);
    park.remove_object(exit.object).unwrap();
    let before = park.clone();
    for (pos, height, facing) in [
        (TilePos { x: 8, y: 9 }, 9, PathDirection::South),
        (TilePos { x: 7, y: 10 }, 8, PathDirection::East),
    ] {
        assert!(park.validate_ride_exit(pos, height, facing, ride).is_err());
        assert!(park.place_ride_exit(pos, height, facing, ride).is_err());
        assert_eq!(park, before);
    }
    let fresh = park
        .place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride)
        .unwrap();
    assert_ne!(fresh, exit.object);
}

#[test]
fn physical_boarding_reserves_front_and_visits_exact_gate_center() {
    let mut park = fixture();
    seek(&mut park, |park| has_phase(park, RideVisitPhase::Boarding));
    let guest = park
        .guests()
        .iter()
        .find(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == RideVisitPhase::Boarding)
        })
        .unwrap()
        .id;
    let transfer = park
        .guests()
        .iter()
        .find(|g| g.id == guest)
        .unwrap()
        .ride_visit
        .as_ref()
        .unwrap()
        .passenger
        .as_ref()
        .unwrap()
        .transfer
        .clone()
        .unwrap();
    assert_eq!(transfer.progress, 0);
    assert_eq!(
        park.guest_position(guest).unwrap().point,
        park.object(transfer.path).unwrap().path_center().unwrap()
    );
    let distance = park.train().unwrap().distance;
    let track = park.circuit().unwrap().pieces[10].object;
    for id in [transfer.path, transfer.portal, track] {
        let before = park.clone();
        let cache = park.path_cache_stats();
        assert!(park.object_removal_block_reason(id).is_some());
        assert_eq!(park.remove_object(id), None);
        assert_eq!(park, before);
        assert_eq!(park.path_cache_stats(), cache);
    }
    for step in 1..=RIDE_TRANSFER_TICKS {
        park.advance(Speed::Normal);
        assert_eq!(park.train().unwrap().distance, distance);
        if step < RIDE_TRANSFER_TICKS {
            assert!(park.guests().iter().all(|g| g.id == guest
                || (g.current != Some(transfer.path) && g.next != Some(transfer.path))));
        }
        if step == RIDE_TRANSFER_TICKS / 2 {
            let gate = park.object(transfer.portal).unwrap();
            assert_eq!(
                park.guest_position(guest).unwrap().point,
                PathPoint {
                    x: gate.pos.x as i32 * PATH_UNITS,
                    y: gate.pos.y as i32 * PATH_UNITS,
                    height: i32::from(gate.height) * PATH_UNITS
                }
            );
            roundtrip(&park);
        }
    }
    let member = park.guests().iter().find(|g| g.id == guest).unwrap();
    assert_eq!(
        member.ride_visit.as_ref().unwrap().phase,
        RideVisitPhase::Riding
    );
    assert_eq!(park.ride().unwrap().boarded, 1);
    roundtrip(&park);
}

#[test]
fn completed_lap_returns_same_guest_identity_and_credits_only_public_arrival() {
    let mut park = fixture();
    let ids: Vec<_> = park.guests().iter().map(|g| g.id).collect();
    seek(&mut park, |park| has_phase(park, RideVisitPhase::Exiting));
    let member = park
        .guests()
        .iter()
        .find(|g| {
            g.ride_visit
                .as_ref()
                .is_some_and(|v| v.phase == RideVisitPhase::Exiting)
        })
        .unwrap()
        .clone();
    assert!(
        member
            .ride_visit
            .as_ref()
            .unwrap()
            .passenger
            .as_ref()
            .unwrap()
            .completed_lap
    );
    assert_eq!(park.ride().unwrap().completed, 0);
    let target = member
        .ride_visit
        .as_ref()
        .unwrap()
        .passenger
        .as_ref()
        .unwrap()
        .transfer
        .as_ref()
        .unwrap()
        .path;
    for _ in 0..RIDE_TRANSFER_TICKS {
        park.advance(Speed::Normal);
    }
    let arrived = park.guests().iter().find(|g| g.id == member.id).unwrap();
    assert_eq!(arrived.current, Some(target));
    assert!(arrived.ride_visit.is_none());
    assert_eq!(arrived.rides_completed, 1);
    assert_eq!(park.ride().unwrap().completed, 1);
    assert_eq!(park.ride().unwrap().exited, 1);
    assert_eq!(park.guests().iter().map(|g| g.id).collect::<Vec<_>>(), ids);
    roundtrip(&park);
}

#[test]
fn close_in_flight_missing_exit_holds_then_unloads_without_entrance() {
    let mut park = fixture();
    seek(&mut park, |park| {
        has_phase(park, RideVisitPhase::Riding) && park.train().unwrap().speed > 0
    });
    let points = park.ride_guest_positions();
    park.set_train_open(false);
    assert_eq!(park.ride_guest_positions(), points);
    let old = park.ride().unwrap().exit.unwrap();
    park.remove_object(old.object).unwrap();
    seek(&mut park, |park| park.train().unwrap().speed == 0);
    let stationary = park.ride_guest_positions();
    assert!(park.ride_holds_train());
    assert_eq!(
        park.ride_queue_stats(park.ride().unwrap().id)
            .oldest_wait_ticks,
        0
    );
    for _ in 0..120 {
        park.advance(Speed::Normal);
        assert_eq!(park.ride_guest_positions(), stationary);
    }
    roundtrip(&park);
    let entrance = park.ride().unwrap().entrance.unwrap().object;
    park.remove_object(entrance).unwrap();
    let ride = park.ride().unwrap().id;
    let fresh = park
        .place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride)
        .unwrap();
    assert_ne!(fresh, old.object);
    assert_eq!(
        park.ride_exit_access(ride).status,
        RideExitStatus::PublicDisconnected
    );
    seek(&mut park, |park| passengers(park) == 0);
    assert!(park.ride().unwrap().completed > 0);
    assert_eq!(park.ride().unwrap().boarded, park.ride().unwrap().exited);
    assert!(!park.train().unwrap().open);
    roundtrip(&park);
    let track = park.circuit().unwrap().pieces[5].object;
    park.remove_object(track).unwrap();
    assert_eq!(park.train().unwrap().phase, crate::TrainPhase::Fault);
    park.install_train_fixture().unwrap();
    assert_eq!(park.train().unwrap().phase, crate::TrainPhase::Closed);
    roundtrip(&park);
}

#[test]
fn closing_during_boarding_walks_to_seat_then_exit_without_ride_credit() {
    let mut park = fixture();
    seek(&mut park, |park| {
        park.guests().iter().any(|g| {
            g.ride_visit.as_ref().is_some_and(|v| {
                v.phase == RideVisitPhase::Boarding
                    && v.passenger
                        .as_ref()
                        .unwrap()
                        .transfer
                        .as_ref()
                        .unwrap()
                        .progress
                        == 7
            })
        })
    });
    let before = park.ride_guest_positions();
    park.set_train_open(false);
    assert_eq!(before, park.ride_guest_positions());
    let snapshot = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, snapshot);
    roundtrip(&park);
    seek(&mut park, |park| passengers(park) == 0);
    assert!(park.ride().unwrap().boarded > 0);
    assert_eq!(park.ride().unwrap().boarded, park.ride().unwrap().exited);
    assert_eq!(park.ride().unwrap().completed, 0);
    roundtrip(&park);
}

#[test]
fn reopening_mid_transfer_preserves_positions_and_completes_the_existing_journey() {
    for phase in [RideVisitPhase::Boarding, RideVisitPhase::Exiting] {
        let mut park = fixture();
        seek(&mut park, |park| {
            park.guests().iter().any(|guest| {
                guest.ride_visit.as_ref().is_some_and(|visit| {
                    visit.phase == phase
                        && visit.passenger.as_ref().is_some_and(|passenger| {
                            passenger
                                .transfer
                                .as_ref()
                                .is_some_and(|transfer| transfer.progress == 7)
                        })
                })
            })
        });
        let guest = park
            .guests()
            .iter()
            .find(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == phase)
            })
            .unwrap()
            .id;
        let positions = park.ride_guest_positions();
        let completed = park.ride().unwrap().completed;
        park.set_train_open(false);
        assert_eq!(park.ride_guest_positions(), positions);
        park.set_train_open(true);
        assert_eq!(park.ride_guest_positions(), positions);
        let paused = park.clone();
        park.advance(Speed::Paused);
        assert_eq!(park, paused);
        let mut loaded = roundtrip(&park);
        for step in 8..=RIDE_TRANSFER_TICKS {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
            assert_eq!(park, loaded);
            if step < RIDE_TRANSFER_TICKS {
                assert_eq!(park.ride().unwrap().completed, completed);
            }
        }
        let member = park
            .guests()
            .iter()
            .find(|member| member.id == guest)
            .unwrap();
        if phase == RideVisitPhase::Boarding {
            assert_eq!(
                member.ride_visit.as_ref().unwrap().phase,
                RideVisitPhase::Riding
            );
            assert_eq!(member.rides_completed, 0);
            assert_eq!(park.ride().unwrap().completed, completed);
            assert!(
                !member
                    .ride_visit
                    .as_ref()
                    .unwrap()
                    .passenger
                    .as_ref()
                    .unwrap()
                    .departed
            );
        } else {
            assert!(member.ride_visit.is_none());
            assert!(
                park.object(member.current.unwrap())
                    .unwrap()
                    .is_public_path()
            );
            assert_eq!(member.rides_completed, 1);
            assert_eq!(park.ride().unwrap().completed, completed + 1);
        }
        roundtrip(&park);
    }
}

#[test]
fn journey_speeds_saves_and_capacity_preserve_complete_ticks() {
    let mut normal = fixture();
    let mut fast = normal.clone();
    let mut fastest = normal.clone();
    for _ in 0..1800 {
        for _ in 0..4 {
            normal.advance(Speed::Normal);
        }
        for _ in 0..2 {
            fast.advance(Speed::Fast);
        }
        fastest.advance(Speed::Fastest);
        assert_eq!(normal, fast);
        assert_eq!(normal, fastest);
        let poses = normal.ride_guest_positions();
        let seats: BTreeSet<_> = poses.iter().map(|pose| pose.seat).collect();
        assert_eq!(seats.len(), poses.len());
        assert!(seats.len() <= TRAIN_SEATS);
    }
    assert!(normal.ride().unwrap().completed > 0);
    roundtrip(&normal);
}

#[test]
fn exact_save_continuation_covers_transfers_movement_and_held_station() {
    let mut park = fixture();
    for phase in [
        RideVisitPhase::Boarding,
        RideVisitPhase::Riding,
        RideVisitPhase::Exiting,
    ] {
        seek(&mut park, |park| has_phase(park, phase));
        let mut loaded = roundtrip(&park);
        for tick in 0..120 {
            if tick == 10 {
                park.set_train_open(false);
                loaded.set_train_open(false);
            }
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
            assert_eq!(park, loaded);
        }
        park.set_train_open(true);
    }
}

#[test]
fn strict_passenger_fields_seats_refs_and_counters_reject_forgery() {
    let mut park = fixture();
    seek(&mut park, |park| has_phase(park, RideVisitPhase::Boarding));
    let base: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    let index = park
        .guests()
        .iter()
        .position(|g| g.ride_visit.as_ref().is_some_and(|v| v.passenger.is_some()))
        .unwrap();
    let stem = format!("/park/guests/{index}/ride_visit/passenger");
    for (path, value) in [
        (format!("{stem}/seat"), json!(6)),
        (format!("{stem}/completed_lap"), json!(true)),
        (format!("{stem}/transfer/progress"), json!(60)),
        (format!("{stem}/transfer/path"), json!(99999)),
        (format!("{stem}/transfer/portal"), json!(99999)),
        ("/park/ride/boarded".into(), json!(999)),
        ("/park/ride/exited".into(), json!(999)),
        ("/park/ride/completed".into(), json!(1)),
    ] {
        let mut data = base.clone();
        *data.pointer_mut(&path).unwrap() = value;
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&data).unwrap()).is_err(),
            "accepted {path}"
        );
    }
    for field in ["transfer", "seat", "departed", "completed_lap"] {
        let mut data = base.clone();
        data.pointer_mut(&stem)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&data).unwrap()).is_err());
    }
}
