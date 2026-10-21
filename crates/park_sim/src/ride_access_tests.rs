use crate::*;

const GATE: TilePos = TilePos { x: 8, y: 11 };
const ROOT: TilePos = TilePos { x: 8, y: 12 };
const TAIL: TilePos = TilePos { x: 8, y: 13 };
const PUBLIC: TilePos = TilePos { x: 8, y: 14 };

fn fixture() -> (Park, RideId) {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    let ride = park.ride().unwrap().id;
    (park, ride)
}

fn gate(park: &mut Park, ride: RideId) -> ObjectId {
    park.place_ride_entrance(GATE, 8, PathDirection::North, ride)
        .unwrap()
}

fn ready() -> (Park, RideId, [ObjectId; 4]) {
    let (mut park, ride) = fixture();
    let entrance = gate(&mut park, ride);
    let first = park
        .place_queue_path(ROOT, 8, PathSlope::Flat, ride)
        .unwrap();
    let last = park
        .place_queue_path(TAIL, 8, PathSlope::Flat, ride)
        .unwrap();
    let public = park.place_path(PUBLIC, 8).unwrap();
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
    (park, ride, [entrance, first, last, public])
}

#[test]
fn entrance_validation_is_pure_complete_and_generic_orphans_are_rejected() {
    let (mut park, ride) = fixture();
    for (pos, height, facing, owner) in [
        (GATE, 9, PathDirection::North, ride),
        (GATE, 8, PathDirection::South, ride),
        (TilePos { x: 7, y: 10 }, 8, PathDirection::East, ride),
        (TilePos { x: 32, y: 0 }, 8, PathDirection::North, ride),
        (GATE, 8, PathDirection::North, RideId(999)),
    ] {
        let before = park.clone();
        let cache = park.path_cache_stats();
        let preview = park.validate_ride_entrance(pos, height, facing, owner);
        assert!(preview.is_err());
        assert_eq!(
            park.place_ride_entrance(pos, height, facing, owner)
                .map(|_| ()),
            preview
        );
        assert_eq!(park, before);
        assert_eq!(park.path_cache_stats(), cache);
    }
    assert_eq!(
        park.place_object(GATE, ObjectKind::RideEntrance, Axis::Y, 8, 2),
        Err(PlacementError::EntranceRequiresRide)
    );
    let before = park.clone();
    assert_eq!(
        park.validate_ride_entrance(GATE, 8, PathDirection::North, ride),
        Ok(())
    );
    assert_eq!(park, before);
    let object = gate(&mut park, ride);
    assert_eq!(park.object(object).unwrap().kind, ObjectKind::RideEntrance);
    assert_eq!(park.ride().unwrap().entrance.unwrap().object, object);
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::NoQueue);
    let before = park.clone();
    assert_eq!(
        park.place_ride_entrance(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride),
        Err(RideAccessError::EntranceAlreadyExists)
    );
    assert_eq!(park, before);
    assert!(park.set_terrain_height(GATE, 9).is_err());
    assert!(park.to_save_bytes().is_ok());
}

#[test]
fn queue_topology_reports_all_fragments_branches_loops_and_public_boundary_edges() {
    let (base, ride, ids) = ready();
    let report = base.ride_access(ride);
    assert_eq!(report.ordered_queue, vec![ids[1], ids[2]]);
    assert_eq!(report.public_entry, Some(ids[3]));
    assert_eq!(report.queue_tiles, 2);
    assert!(report.station.is_some());
    for (case, expected) in [
        (0, RideAccessStatus::NoPublicEntry),
        (1, RideAccessStatus::MultiplePublicEntries),
        (2, RideAccessStatus::PublicEntryNotAtEnd),
        (3, RideAccessStatus::Branch),
        (4, RideAccessStatus::Disconnected),
        (5, RideAccessStatus::Loop),
    ] {
        let mut park = base.clone();
        match case {
            0 => {
                park.remove_object(ids[3]).unwrap();
            }
            1 => {
                park.place_path(TilePos { x: 7, y: 13 }, 8).unwrap();
            }
            2 => {
                park.remove_object(ids[3]).unwrap();
                park.place_path(TilePos { x: 7, y: 12 }, 8).unwrap();
            }
            3 => {
                park.place_queue_path(TilePos { x: 9, y: 12 }, 8, PathSlope::Flat, ride)
                    .unwrap();
            }
            4 => {
                park.place_queue_path(TilePos { x: 20, y: 20 }, 8, PathSlope::Flat, ride)
                    .unwrap();
            }
            _ => {
                for (x, y) in [(20, 20), (20, 21), (21, 21), (21, 20)] {
                    park.place_queue_path(TilePos { x, y }, 8, PathSlope::Flat, ride)
                        .unwrap();
                }
            }
        }
        assert_eq!(park.ride_access(ride).status, expected, "case {case}");
        assert!(park.ride_access(ride).ordered_queue.is_empty());
        assert!(
            park.to_save_bytes().is_ok(),
            "incomplete construction remains saveable"
        );
    }
    let (mut park, ride) = fixture();
    gate(&mut park, ride);
    for (x, y) in [(8, 12), (9, 12), (10, 12), (10, 13), (10, 14), (9, 14)] {
        park.place_queue_path(TilePos { x, y }, 8, PathSlope::Flat, ride)
            .unwrap();
    }
    park.place_path(TilePos { x: 9, y: 13 }, 8).unwrap();
    assert_eq!(
        park.ride_access(ride).status,
        RideAccessStatus::MultiplePublicEntries,
        "count boundary edges even when they touch the same public object"
    );
}

#[test]
fn queue_geometry_joins_ramp_ends_and_gate_outer_edge_without_public_navigation() {
    let (mut park, ride) = fixture();
    let entrance = gate(&mut park, ride);
    let mut queue = Vec::new();
    for (y, height, slope) in [
        (12, 8, PathSlope::Flat),
        (13, 8, PathSlope::South),
        (14, 9, PathSlope::Flat),
        (15, 8, PathSlope::North),
    ] {
        queue.push(
            park.place_queue_path(TilePos { x: 8, y }, height, slope, ride)
                .unwrap(),
        );
    }
    let public = park.place_path(TilePos { x: 8, y: 16 }, 8).unwrap();
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
    assert_eq!(park.ride_access(ride).ordered_queue, queue);
    assert_eq!(park.geometric_path_neighbors(entrance), vec![queue[0]]);
    assert!(park.geometric_path_neighbors(public).contains(&queue[3]));
    for id in [entrance, public].into_iter().chain(queue.iter().copied()) {
        assert!(park.path_neighbors(id).is_empty());
    }
    let mut ghost = *park.object(queue[0]).unwrap();
    ghost.id = ObjectId(0);
    assert!(park.geometric_paths_connect(&ghost, park.object(entrance).unwrap()));
    ghost.queue_owner = Some(RideId(999));
    assert!(!park.geometric_paths_connect(&ghost, park.object(entrance).unwrap()));
    park.remove_object(queue[1]).unwrap();
    assert_eq!(
        park.ride_access(ride).status,
        RideAccessStatus::Disconnected
    );
    park.place_queue_path(TilePos { x: 8, y: 13 }, 8, PathSlope::North, ride)
        .unwrap();
    assert_eq!(
        park.ride_access(ride).status,
        RideAccessStatus::Disconnected,
        "wrong-facing ramp cannot connect by base height alone"
    );
}

#[test]
fn gate_side_wrong_height_and_foreign_owned_contacts_do_not_silently_connect() {
    for (pos, height, slope) in [
        (ROOT, 9, PathSlope::Flat),
        (TilePos { x: 9, y: 11 }, 8, PathSlope::Flat),
        (ROOT, 8, PathSlope::East),
    ] {
        let (mut park, ride) = fixture();
        let entrance = gate(&mut park, ride);
        park.place_queue_path(pos, height, slope, ride).unwrap();
        assert!(park.geometric_path_neighbors(entrance).is_empty());
        assert_eq!(
            park.ride_access(ride).status,
            RideAccessStatus::Disconnected
        );
    }
    let (mut park, ride, _) = ready();
    assert!(matches!(
        park.place_queue_path(TilePos { x: 9, y: 13 }, 8, PathSlope::Flat, RideId(999)),
        Err(RideAccessError::UnknownRide(_))
    ));
    // A deliberately corrupt/future multi-ride contact must be diagnosed, never chosen.
    let foreign = park
        .place_queue_path(TilePos { x: 9, y: 13 }, 8, PathSlope::Flat, ride)
        .unwrap();
    park.objects
        .iter_mut()
        .find(|o| o.id == foreign)
        .unwrap()
        .queue_owner = Some(RideId(999));
    assert_eq!(
        park.ride_access(ride).status,
        RideAccessStatus::ForeignQueue
    );
    assert!(park.to_save_bytes().is_err());
}

#[test]
fn queue_edits_preserve_public_cache_and_seeded_guests_never_enter_queues_or_gates() {
    let (mut park, ride, ids) = ready();
    park.install_crowd_fixture(771).unwrap();
    park.advance(Speed::Normal);
    let public_graph: Vec<_> = park
        .objects()
        .iter()
        .map(|o| (o.id, park.path_neighbors(o.id)))
        .collect();
    let cache = park.path_cache_stats();
    let extra = park
        .place_queue_path(TilePos { x: 20, y: 20 }, 8, PathSlope::East, ride)
        .unwrap();
    assert_eq!(park.path_cache_stats(), cache);
    park.remove_object(extra).unwrap();
    assert_eq!(park.path_cache_stats(), cache);
    for (id, neighbors) in public_graph {
        assert_eq!(park.path_neighbors(id), neighbors);
    }
    let mut cold = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    for object in park.objects() {
        assert_eq!(
            park.path_neighbors(object.id),
            cold.path_neighbors(object.id)
        );
    }
    for _ in 0..600 {
        park.advance(Speed::Fastest);
        cold.advance(Speed::Fastest);
        assert_eq!(park, cold);
        for guest in park.guests() {
            for id in guest
                .current
                .iter()
                .chain(guest.next.iter())
                .chain(guest.destination.iter())
                .chain(guest.route.iter())
            {
                assert!(park.object(*id).unwrap().is_public_path());
            }
        }
    }
    assert!(!park.guests().iter().any(|g| g.current == Some(ids[0])));
    let current = park.guests[0].current.unwrap();
    let removed = park.remove_object(current).unwrap();
    let queue = park
        .place_queue_path(removed.pos, removed.height, removed.path_slope, ride)
        .unwrap();
    for _ in 0..60 {
        park.advance(Speed::Normal);
        assert_ne!(park.guests[0].current, Some(queue));
    }
    let before = park.clone();
    let cache = park.path_cache_stats();
    assert!(park.restore_crowd_paths().is_err());
    assert_eq!(park, before);
    assert_eq!(park.path_cache_stats(), cache);
}

#[test]
fn gate_demolition_and_circuit_fault_repair_preserve_owner_and_fresh_object_ids() {
    let (mut park, ride, ids) = ready();
    let station = park.ride_access(ride).station.unwrap();
    let original_ids: Vec<_> = park
        .objects()
        .iter()
        .filter(|o| o.queue_owner == Some(ride))
        .map(|o| o.id)
        .collect();
    park.remove_object(station).unwrap();
    assert_eq!(
        park.ride_access(ride).status,
        RideAccessStatus::RideUnavailable
    );
    assert_eq!(park.ride().unwrap().id, ride);
    assert_eq!(park.ride().unwrap().entrance.unwrap().object, ids[0]);
    let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    park.install_train_fixture().unwrap();
    loaded.install_train_fixture().unwrap();
    assert_eq!(park, loaded);
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
    assert_ne!(park.ride_access(ride).station, Some(station));
    park.remove_object(ids[0]).unwrap();
    assert_eq!(park.ride().unwrap().entrance, None);
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::NoEntrance);
    let replacement = gate(&mut park, ride);
    assert!(replacement.0 > ids[0].0);
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
    assert_eq!(
        park.objects()
            .iter()
            .filter(|o| o.queue_owner == Some(ride))
            .map(|o| o.id)
            .collect::<Vec<_>>(),
        original_ids
    );
    let before = park.clone();
    park.install_train_fixture().unwrap();
    assert_eq!(park, before);
}

#[test]
fn queue_and_entrance_limits_reject_atomically_before_allocating() {
    let (mut park, ride) = fixture();
    for y in 0..32 {
        for x in 0..32 {
            park.place_path(TilePos { x, y }, 24).unwrap();
        }
    }
    let before = park.clone();
    let stats = park.path_cache_stats();
    assert!(matches!(
        park.place_queue_path(ROOT, 8, PathSlope::Flat, ride),
        Err(RideAccessError::Placement(
            PlacementError::CapacityExceeded { .. }
        ))
    ));
    assert_eq!(park, before);
    assert_eq!(park.path_cache_stats(), stats);
    assert!(park.to_save_bytes().is_ok());
    let (mut park, ride) = fixture();
    while park.objects.len() < crate::saves::MAX_OBJECTS {
        // Noncolliding upper track slots; leave the gate's lower envelope free.
        let index = park.objects.len();
        let pos = TilePos {
            x: (index % 32) as u32,
            y: ((index / 32) % 32) as u32,
        };
        let height = 20 + ((index / 1024) * 2) as u8;
        park.place_object(pos, ObjectKind::Track, Axis::X, height, 2)
            .unwrap();
    }
    let before = park.clone();
    assert!(matches!(
        park.place_ride_entrance(GATE, 8, PathDirection::North, ride),
        Err(RideAccessError::Placement(
            PlacementError::CapacityExceeded { .. }
        ))
    ));
    assert_eq!(park, before);
    let mut empty = Park::new(32, 32);
    empty.next_ride_id = u64::MAX;
    let before = empty.clone();
    assert!(empty.install_train_fixture().is_err());
    assert_eq!(empty, before);
}

#[test]
fn queue_save_continuation_survives_partial_build_edits_pause_and_speed_grouping() {
    let (mut park, ride, ids) = ready();
    park.install_crowd_fixture(918).unwrap();
    let bytes = park.to_save_bytes().unwrap();
    let mut loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(bytes, loaded.to_save_bytes().unwrap());
    for tick in 0..900 {
        for candidate in [&mut park, &mut loaded] {
            if tick == 100 {
                candidate.remove_object(ids[2]).unwrap();
            }
            if tick == 200 {
                candidate
                    .place_queue_path(TAIL, 8, PathSlope::Flat, ride)
                    .unwrap();
            }
            if tick == 300 {
                candidate.remove_object(ids[0]).unwrap();
            }
            if tick == 400 {
                gate(candidate, ride);
            }
            if tick == 500 {
                candidate.set_train_open(false);
            }
        }
        let speed = [Speed::Paused, Speed::Normal, Speed::Fast, Speed::Fastest][tick % 4];
        park.advance(speed);
        loaded.advance(speed);
        assert_eq!(park, loaded);
        assert_eq!(park.ride_access(ride), loaded.ride_access(ride));
        if tick % 100 == 0 {
            loaded = Park::from_save_bytes(&loaded.to_save_bytes().unwrap()).unwrap();
        }
    }
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
    assert_eq!(
        park.to_save_bytes().unwrap(),
        loaded.to_save_bytes().unwrap()
    );
}
