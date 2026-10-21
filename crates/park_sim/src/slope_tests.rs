//! Behavioral checks for sloped geometry, navigation, edits and persistence.

use crate::*;

const SLOPES: [PathSlope; 4] = [
    PathSlope::North,
    PathSlope::East,
    PathSlope::South,
    PathSlope::West,
];

fn ramp(slope: PathSlope) -> (Park, [ObjectId; 3]) {
    let mut park = Park::new(8, 8);
    let center = TilePos { x: 3, y: 3 };
    let high = slope.direction().unwrap();
    let low = park
        .place_path(high.opposite().neighbor(center).unwrap(), 4)
        .unwrap();
    let ramp = park.place_sloped_path(center, 4, slope).unwrap();
    let high = park.place_path(high.neighbor(center).unwrap(), 5).unwrap();
    (park, [low, ramp, high])
}

fn start_walk(park: &mut Park, sequence: &[ObjectId]) {
    let current = *park.object(sequence[0]).unwrap();
    park.crowd_seed = Some(31337);
    park.crowd_rng = 31337;
    park.next_guest_id = 2;
    park.guests = vec![Guest {
        id: GuestId(1),
        current: Some(current.id),
        next: Some(sequence[1]),
        destination: sequence.last().copied(),
        anchor: current.pos,
        height: current.height,
        anchor_slope: current.path_slope,
        progress: 0,
        state: GuestState::Walking,
        arrivals: 0,
        route: sequence[2..].iter().copied().collect(),
        wait_ticks: 0,
    }];
}

fn ticks(park: &mut Park, count: u32) {
    for _ in 0..count {
        park.advance(Speed::Normal);
    }
}

#[test]
fn four_rise_directions_have_exact_planes_and_only_matching_opposite_ends() {
    for slope in SLOPES {
        let (mut park, [low, ramp, high]) = ramp(slope);
        let object = *park.object(ramp).unwrap();
        let direction = slope.direction().unwrap();
        assert_eq!(object.axis, slope.axis());
        assert_eq!(object.clearance, 3);
        assert_eq!(
            object.path_center().unwrap().height,
            4 * PATH_UNITS + PATH_UNITS / 2
        );
        for edge in PathDirection::ALL {
            let (dx, dy) = edge.offset();
            let expected = if edge == direction {
                Some(5)
            } else if edge == direction.opposite() {
                Some(4)
            } else {
                None
            };
            assert_eq!(object.path_edge_height(edge), expected);
            if let Some(height) = expected {
                assert_eq!(
                    object.path_height_at(dx * 30, dy * 30),
                    Some(i32::from(height) * PATH_UNITS)
                );
            } else {
                let side = park
                    .place_path(edge.neighbor(object.pos).unwrap(), 4)
                    .unwrap();
                assert!(!paths_connect(&object, park.object(side).unwrap()));
            }
        }
        assert_eq!(object.path_height_at(31, 0), None);
        assert_eq!(park.path_neighbors(ramp), vec![low, high]);
        let stacked = park.place_path(park.object(low).unwrap().pos, 7).unwrap();
        assert!(!park.path_neighbors(stacked).contains(&ramp));
        let track = park
            .place_object(TilePos { x: 7, y: 7 }, ObjectKind::Track, Axis::X, 4, 2)
            .unwrap();
        assert_eq!(park.object(track).unwrap().path_center(), None);
        assert!(park.path_neighbors(track).is_empty());
        start_walk(&mut park, &[low, ramp, high]);
        park.advance(Speed::Normal); // Warm the indexed graph and compare the same topology.
        assert_eq!(park.path_neighbors(ramp), vec![low, high]);
        assert_eq!(park.max_surface_height(), 7);
    }
}

#[test]
fn matching_slope_chains_crests_and_troughs_connect_but_sides_and_wrong_heights_do_not() {
    for (left, left_height, right, right_height, connected) in [
        (PathSlope::East, 4, PathSlope::East, 5, true),
        (PathSlope::East, 4, PathSlope::West, 4, true),
        (PathSlope::West, 4, PathSlope::East, 4, true),
        (PathSlope::East, 4, PathSlope::East, 4, false),
        (PathSlope::East, 4, PathSlope::North, 4, false),
        (PathSlope::Flat, 4, PathSlope::East, 5, false),
    ] {
        let mut park = Park::new(3, 3);
        let a = park
            .place_sloped_path(TilePos { x: 0, y: 1 }, left_height, left)
            .unwrap();
        let b = park
            .place_sloped_path(TilePos { x: 1, y: 1 }, right_height, right)
            .unwrap();
        assert_eq!(
            paths_connect(park.object(a).unwrap(), park.object(b).unwrap()),
            connected
        );
        assert_eq!(park.path_neighbors(a).contains(&b), connected);
        assert_eq!(park.path_neighbors(b).contains(&a), connected);
    }
}

#[test]
fn sloped_preview_commit_clearance_limits_and_rejections_are_atomic() {
    let (mut park, ids) = ramp(PathSlope::East);
    start_walk(&mut park, &ids);
    park.advance(Speed::Normal);
    let raised = TilePos { x: 0, y: 0 };
    park.set_terrain_height(raised, 5).unwrap();
    let occupied = park.object(ids[1]).unwrap().pos;
    for (pos, height) in [
        (TilePos { x: 8, y: 0 }, 4),
        (raised, 4),
        (occupied, 6),
        (raised, 30),
        (raised, u8::MAX),
    ] {
        let before = park.clone();
        let stats = park.path_cache_stats();
        let preview = park.validate_sloped_path(pos, height, PathSlope::North);
        assert!(preview.is_err());
        assert_eq!(
            park.place_sloped_path(pos, height, PathSlope::North)
                .map(|_| ()),
            preview
        );
        assert_eq!(park, before);
        assert_eq!(park.path_cache_stats(), stats);
    }
    let expected_id = park.next_object_id;
    let stats = park.path_cache_stats();
    assert_eq!(
        park.validate_sloped_path(raised, MAX_SLOPED_PATH_HEIGHT, PathSlope::West),
        Ok(())
    );
    assert_eq!(park.path_cache_stats(), stats);
    let id = park
        .place_sloped_path(raised, MAX_SLOPED_PATH_HEIGHT, PathSlope::West)
        .unwrap();
    assert_eq!(id, ObjectId(expected_id));
    assert_eq!(park.object(id).unwrap().height, 29);
    assert_eq!(park.max_surface_height(), 30);
    assert_eq!(
        park.path_cache_stats().invalidations,
        stats.invalidations + 1
    );
    assert!(park.validate_terrain_height(occupied, 5).is_err());
    assert_eq!(park.validate_terrain_height(occupied, 4), Ok(()));
    // Full-column occupancy forbids a track inside the high-end clearance.
    assert!(
        park.place_object(occupied, ObjectKind::Track, Axis::X, 6, 2)
            .is_err()
    );
    assert!(
        park.place_object(occupied, ObjectKind::Track, Axis::X, 7, 2)
            .is_ok()
    );
    let mut old = Park::new(2, 2);
    let mut new = old.clone();
    assert_eq!(
        old.place_path(TilePos::default(), MAX_PATH_HEIGHT),
        new.place_sloped_path(TilePos::default(), MAX_PATH_HEIGHT, PathSlope::Flat)
    );
    assert_eq!(old, new);
}

#[test]
fn player_slope_capacity_rejection_preserves_ids_and_warm_cache() {
    let mut park = Park::new(32, 32);
    for index in 0..crate::saves::MAX_PATHS {
        park.place_path(
            TilePos {
                x: index as u32 % 32,
                y: index as u32 / 32,
            },
            4,
        )
        .unwrap();
    }
    let first = park.objects[0].id;
    let second = park.objects[1].id;
    start_walk(&mut park, &[first, second]);
    park.advance(Speed::Normal);
    let before = park.clone();
    let stats = park.path_cache_stats();
    assert!(matches!(
        park.validate_sloped_path(TilePos::default(), 8, PathSlope::East),
        Err(PlacementError::CapacityExceeded { .. })
    ));
    assert!(
        park.place_sloped_path(TilePos::default(), 8, PathSlope::East)
            .is_err()
    );
    assert_eq!(park, before);
    assert_eq!(park.path_cache_stats(), stats);
    assert!(park.to_save_bytes().is_ok());
}

#[test]
fn walking_both_directions_follows_each_half_tile_and_arrives_at_correct_base() {
    for slope in SLOPES {
        for backwards in [false, true] {
            let (mut park, mut ids) = ramp(slope);
            if backwards {
                ids.reverse();
            }
            start_walk(&mut park, &ids);
            let flat_height = if backwards {
                5 * PATH_UNITS
            } else {
                4 * PATH_UNITS
            };
            for progress in 0..WALK_TICKS {
                let point = park.guest_position(GuestId(1)).unwrap().point;
                let rise = if backwards { -2 } else { 2 };
                let expected = flat_height + rise * i32::from(progress.saturating_sub(15));
                assert_eq!(
                    point.height, expected,
                    "{slope:?}, backwards={backwards}, progress={progress}"
                );
                park.advance(Speed::Normal);
            }
            assert_eq!(park.guests[0].current, Some(ids[1]));
            assert_eq!(park.guests[0].height, 4);
            assert_eq!(park.guests[0].anchor_slope, slope);
            assert_eq!(park.guest_position(GuestId(1)).unwrap().point.height, 270);
            for progress in 0..WALK_TICKS {
                let change = if backwards { -2 } else { 2 };
                assert_eq!(
                    park.guest_position(GuestId(1)).unwrap().point.height,
                    270 + change * i32::from(progress.min(15))
                );
                park.advance(Speed::Normal);
            }
            assert_eq!(park.guests[0].current, Some(ids[2]));
            assert_eq!(park.guests[0].height, if backwards { 4 } else { 5 });
            assert_eq!(park.guests[0].anchor_slope, PathSlope::Flat);
            assert_eq!(park.guests[0].arrivals, 1);
        }
    }
}

#[test]
fn removed_slope_anchor_preserves_half_height_and_rejects_wrong_side_or_level_recovery() {
    for slope in SLOPES {
        let (mut park, [low, ramp, high]) = ramp(slope);
        start_walk(&mut park, &[ramp, high]);
        ticks(&mut park, 7);
        let center = park.object(ramp).unwrap().pos;
        let side = PathDirection::ALL
            .into_iter()
            .find(|edge| {
                *edge != slope.direction().unwrap()
                    && *edge != slope.direction().unwrap().opposite()
            })
            .unwrap();
        park.place_path(side.neighbor(center).unwrap(), 4).unwrap();
        park.remove_object(low).unwrap();
        park.remove_object(high).unwrap();
        park.remove_object(ramp).unwrap();
        park.place_path(center, 8).unwrap();
        ticks(&mut park, 90);
        assert_eq!(park.guests[0].current, None);
        assert_eq!(park.guests[0].anchor_slope, slope);
        assert_eq!(park.guest_position(GuestId(1)).unwrap().point.height, 270);
        let bytes = park.to_save_bytes().unwrap();
        let mut loaded = Park::from_save_bytes(&bytes).unwrap();
        assert_eq!(loaded, park);
        let restored = park.place_sloped_path(center, 4, slope).unwrap();
        assert!(restored.0 > ramp.0);
        assert_eq!(
            loaded.place_sloped_path(center, 4, slope).unwrap(),
            restored
        );
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
        assert_eq!(loaded, park);
        assert_eq!(park.guests[0].current, Some(restored));
        // With both enabled ends absent, the restored slope remains isolated.
        assert_eq!(park.guests[0].state, GuestState::Lost);
        let end = slope.direction().unwrap().neighbor(center).unwrap();
        park.place_path(end, 5).unwrap();
        park.advance(Speed::Normal);
        assert_eq!(park.guests[0].state, GuestState::Walking);
    }
}

#[test]
fn deleting_current_next_or_destination_clears_all_references_and_recovers_locally() {
    for removed_index in 0..3 {
        let (mut park, ids) = ramp(PathSlope::East);
        start_walk(&mut park, &ids);
        ticks(&mut park, 9);
        let removed = ids[removed_index];
        park.remove_object(removed).unwrap();
        let guest = &park.guests[0];
        assert_ne!(guest.current, Some(removed));
        assert_eq!(guest.next, None);
        assert_eq!(guest.destination, None);
        assert!(guest.route.is_empty());
        assert_eq!(guest.progress, 0);
        assert_eq!(guest.state, GuestState::Lost);
        assert!(park.to_save_bytes().is_ok());
        park.advance(Speed::Normal);
        // If the low current was deleted, its only valid incident slope is allowed.
        if removed_index == 0 {
            assert_eq!(park.guests[0].current, Some(ids[1]));
        }
        assert!(park.to_save_bytes().is_ok());
    }
}

#[test]
fn slope_saves_cache_rebuilds_and_speed_grouping_preserve_exact_continuation() {
    let (mut park, ids) = ramp(PathSlope::East);
    start_walk(&mut park, &ids);
    for elapsed in [7, 8, 8] {
        ticks(&mut park, elapsed);
        let bytes = park.to_save_bytes().unwrap();
        let loaded = Park::from_save_bytes(&bytes).unwrap();
        assert_eq!(loaded, park);
        assert_eq!(loaded.to_save_bytes().unwrap(), bytes);
        assert_eq!(
            loaded.guest_position(GuestId(1)),
            park.guest_position(GuestId(1))
        );
    }
    let mut fresh = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    let mut fast = park.clone();
    for block in 0..120 {
        if block == 30 {
            for candidate in [&mut park, &mut fresh, &mut fast] {
                candidate.remove_object(ids[1]).unwrap();
            }
        }
        if block == 40 {
            for candidate in [&mut park, &mut fresh, &mut fast] {
                candidate
                    .place_sloped_path(TilePos { x: 3, y: 3 }, 4, PathSlope::East)
                    .unwrap();
            }
        }
        let paused = park.clone();
        let cache = park.path_cache_stats();
        park.advance(Speed::Paused);
        assert_eq!(park, paused);
        assert_eq!(park.path_cache_stats(), cache);
        for _ in 0..4 {
            park.advance(Speed::Normal);
            fresh.path_cache.invalidate();
            fresh.advance(Speed::Normal);
        }
        if block % 2 == 0 {
            fast.advance(Speed::Fast);
            fast.advance(Speed::Fast);
        } else {
            fast.advance(Speed::Fastest);
        }
        assert_eq!(park, fresh);
        assert_eq!(park, fast);
    }
    assert!(park.path_cache_stats().builds < fresh.path_cache_stats().builds);
    assert_eq!(
        park.to_save_bytes().unwrap(),
        fresh.to_save_bytes().unwrap()
    );
}
