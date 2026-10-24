use super::*;
use crate::{Axis, PathDirection, ServiceKind, Speed, TerrainProfile};
use serde_json::{Value, json};

pub(super) fn loop_specs(x: u32, y: u32, height: u8) -> Vec<TrackPieceSpec> {
    use TrackBuildKind::{CurveRight, Station, Straight};
    use TrackDirection::{East, North, South, West};
    [
        (1, 0, Station, East),
        (2, 0, CurveRight, East),
        (2, 1, Straight, South),
        (2, 2, CurveRight, South),
        (1, 2, Straight, West),
        (0, 2, CurveRight, West),
        (0, 1, Straight, North),
        (0, 0, CurveRight, North),
    ]
    .into_iter()
    .map(|(dx, dy, kind, direction)| {
        kind.spec(
            TilePos {
                x: x + dx,
                y: y + dy,
            },
            height,
            direction,
        )
    })
    .collect()
}

pub(super) fn graded_loop_specs() -> Vec<TrackPieceSpec> {
    use TrackBuildKind::{Brake, CurveRight, Drop, Lift, Station, SteepDrop, Straight};
    use TrackDirection::{East, North, South, West};
    [
        (5, 4, Station, East, 8),
        (6, 4, Lift, East, 8),
        (7, 4, Lift, East, 9),
        (8, 4, Lift, East, 10),
        (9, 4, Straight, East, 11),
        (10, 4, CurveRight, East, 11),
        (10, 5, Straight, South, 11),
        (10, 6, Drop, South, 11),
        (10, 7, Straight, South, 10),
        (10, 8, CurveRight, South, 10),
        (9, 8, SteepDrop, West, 10),
        (8, 8, Brake, West, 8),
        (7, 8, Straight, West, 8),
        (6, 8, Straight, West, 8),
        (5, 8, Straight, West, 8),
        (4, 8, CurveRight, West, 8),
        (4, 7, Straight, North, 8),
        (4, 6, Straight, North, 8),
        (4, 5, Straight, North, 8),
        (4, 4, CurveRight, North, 8),
    ]
    .into_iter()
    .map(|(x, y, kind, heading, height)| kind.spec(TilePos { x, y }, height, heading))
    .collect()
}

pub(super) fn draft(park: &mut Park, specs: &[TrackPieceSpec]) {
    for spec in specs.iter().rev() {
        park.set_track_draft_piece(*spec).unwrap();
    }
}
pub(super) fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(bytes, loaded.to_save_bytes().unwrap());
    loaded
}
pub(super) fn json_value(park: &Park) -> Value {
    serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap()
}
pub(super) fn read(value: &Value) -> Result<Park, crate::SaveError> {
    Park::from_save_bytes(&serde_json::to_vec(value).unwrap())
}

#[test]
fn rotation_and_curve_handedness_match_exact_sample_endpoints() {
    for heading in [
        TrackDirection::North,
        TrackDirection::East,
        TrackDirection::South,
        TrackDirection::West,
    ] {
        let pos = TilePos { x: 8, y: 8 };
        let straight = TrackBuildKind::Straight.spec(pos, 8, heading);
        assert_eq!(straight.entry.edge, heading.opposite());
        assert_eq!(straight.exit.edge, heading);
        let left = TrackBuildKind::CurveLeft.spec(pos, 8, heading);
        let right = TrackBuildKind::CurveRight.spec(pos, 8, heading);
        assert_eq!(left.entry, right.entry);
        assert_eq!(left.exit.edge, right.exit.edge.opposite());
        assert_eq!(left.samples().first(), straight.samples().first());
        assert_ne!(left.samples().last(), right.samples().last());
        assert!(
            left.samples()
                .iter()
                .all(|point| point.height == 8 * crate::TRACK_UNITS)
        );
    }
}

#[test]
fn graded_piece_directions_heights_and_samples_match_the_selected_piece() {
    let park = Park::new(32, 32);
    let pos = TilePos { x: 8, y: 8 };
    for heading in [
        TrackDirection::North,
        TrackDirection::East,
        TrackDirection::South,
        TrackDirection::West,
    ] {
        for (kind, track_kind, exit_height) in [
            (TrackBuildKind::Lift, TrackKind::Lift, 9),
            (TrackBuildKind::Drop, TrackKind::Drop, 7),
            (TrackBuildKind::SteepDrop, TrackKind::Drop, 6),
            (TrackBuildKind::Brake, TrackKind::Brake, 8),
        ] {
            let spec = kind.spec(pos, 8, heading);
            assert_eq!(park.validate_track_draft_piece(spec), Ok(()));
            assert_eq!(spec.kind, track_kind);
            assert_eq!(
                spec.entry,
                TrackPort {
                    edge: heading.opposite(),
                    height: 8
                }
            );
            assert_eq!(
                spec.exit,
                TrackPort {
                    edge: heading,
                    height: exit_height
                }
            );
            let samples = spec.samples();
            assert_eq!(
                samples.first(),
                TrackBuildKind::Straight
                    .spec(pos, 8, heading)
                    .samples()
                    .first()
            );
            assert_eq!(
                samples.last(),
                TrackBuildKind::Straight
                    .spec(pos, exit_height, heading)
                    .samples()
                    .last()
            );
            assert!(samples.windows(2).all(|pair| {
                match kind {
                    TrackBuildKind::Lift => pair[0].height < pair[1].height,
                    TrackBuildKind::Drop | TrackBuildKind::SteepDrop => {
                        pair[0].height > pair[1].height
                    }
                    _ => pair[0].height == pair[1].height,
                }
            }));
        }
    }
}

#[test]
fn every_entry_height_respects_kind_bounds_without_saturating_grades() {
    let mut park = Park::new(32, 32);
    let pos = TilePos { x: 5, y: 5 };
    for kind in [
        TrackBuildKind::Station,
        TrackBuildKind::Straight,
        TrackBuildKind::CurveLeft,
        TrackBuildKind::CurveRight,
        TrackBuildKind::Lift,
        TrackBuildKind::Drop,
        TrackBuildKind::SteepDrop,
        TrackBuildKind::Brake,
    ] {
        for height in 0..=u8::MAX {
            let spec = kind.spec(pos, height, TrackDirection::East);
            let expected = (kind.min_height()..=kind.max_height()).contains(&height);
            let before = park.clone();
            assert_eq!(
                park.set_track_draft_piece(spec).is_ok(),
                expected,
                "{kind:?} at {height}"
            );
            if expected {
                assert_eq!(park.track_draft(), [spec]);
                park.clear_track_draft();
            } else {
                assert_eq!(park, before);
            }
        }
    }
}

#[test]
fn graded_clearance_covers_low_end_and_space_above_high_end() {
    let pos = TilePos { x: 5, y: 5 };
    for (kind, entry, lowest, highest) in [
        (TrackBuildKind::Lift, 8, 8, 9),
        (TrackBuildKind::Drop, 9, 8, 9),
        (TrackBuildKind::SteepDrop, 10, 8, 10),
    ] {
        let spec = kind.spec(pos, entry, TrackDirection::East);
        for obstacle_height in lowest..highest + 2 {
            let mut park = Park::new(32, 32);
            park.place_object(pos, ObjectKind::Track, Axis::X, obstacle_height, 1)
                .unwrap();
            let before = park.clone();
            assert!(matches!(
                park.set_track_draft_piece(spec),
                Err(TrackConstructionError::Track(TrainError::Placement(
                    PlacementError::Occupied { .. }
                )))
            ));
            assert_eq!(park, before);
        }
        for terrain_height in lowest + 1..highest + 2 {
            let mut park = Park::new(32, 32);
            park.set_terrain_height(pos, terrain_height).unwrap();
            let before = park.clone();
            assert!(matches!(
                park.set_track_draft_piece(spec),
                Err(TrackConstructionError::Track(TrainError::Placement(
                    PlacementError::TerrainIntersection { .. }
                )))
            ));
            assert_eq!(park, before);
        }
        let mut boundaries = Park::new(32, 32);
        boundaries.set_terrain_height(pos, lowest).unwrap();
        boundaries
            .place_object(pos, ObjectKind::Track, Axis::X, highest + 2, 1)
            .unwrap();
        assert_eq!(boundaries.validate_track_draft_piece(spec), Ok(()));
        let mut underground = Park::new(32, 32);
        underground.set_terrain_height(pos, highest + 2).unwrap();
        assert_eq!(underground.validate_track_draft_piece(spec), Ok(()));
    }
}

#[test]
fn graded_draft_builds_matching_envelopes_and_runs_after_save_reload() {
    let mut park = Park::new(32, 32);
    let specs = graded_loop_specs();
    draft(&mut park, &specs);
    assert_eq!(park.validate_track_draft().unwrap(), specs);
    roundtrip(&park);
    park.apply_track_draft().unwrap();
    for (spec, piece) in specs.iter().zip(&park.circuit().unwrap().pieces) {
        let object = park.object(piece.object).unwrap();
        assert_eq!(object.height, spec.entry.height.min(spec.exit.height));
        assert_eq!(
            object.clearance,
            spec.entry.height.abs_diff(spec.exit.height) + 2
        );
        assert_eq!(piece.samples, spec.samples());
    }
    let built = park.clone();
    park.apply_track_draft().unwrap();
    assert_eq!(park, built);
    park.set_train_open(true);
    let mut loaded = roundtrip(&park);
    let mut seen = [false; 4];
    for _ in 0..6000 {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
        assert_eq!(park, loaded);
        let train = park.train().unwrap();
        assert_ne!(train.phase, TrainPhase::Fault);
        let spec = specs[train.piece_index];
        let index = match train.phase {
            TrainPhase::Lift => Some(0),
            TrainPhase::Drop if spec.entry.height - spec.exit.height == 1 => Some(1),
            TrainPhase::Drop => Some(2),
            TrainPhase::Brake => Some(3),
            _ => None,
        };
        if let Some(index) = index.filter(|index| !seen[*index]) {
            loaded = roundtrip(&park);
            seen[index] = true;
        }
        if train.laps >= 2 {
            break;
        }
    }
    assert_eq!(seen, [true; 4]);
    assert!(park.train().unwrap().laps >= 2);
    roundtrip(&park);
}

#[test]
fn blocked_or_disconnected_graded_replacement_is_atomic() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    park.set_train_open(false);
    draft(&mut park, &graded_loop_specs());
    let steep = park
        .track_draft
        .iter()
        .find(|spec| spec.entry.height.abs_diff(spec.exit.height) == 2)
        .copied()
        .unwrap();
    let mut blocked = park.clone();
    blocked
        .place_object(steep.pos, ObjectKind::Track, Axis::X, steep.exit.height, 1)
        .unwrap();
    let mut disconnected = park.clone();
    disconnected
        .set_track_draft_piece(TrackBuildKind::Drop.spec(
            steep.pos,
            steep.entry.height,
            TrackDirection::West,
        ))
        .unwrap();
    for mut invalid in [blocked, disconnected] {
        let before = invalid.clone();
        assert!(invalid.apply_track_draft().is_err());
        assert_eq!(invalid, before);
        roundtrip(&invalid);
    }
}

#[test]
fn unordered_plan_builds_closed_coaster_and_continues_deterministically() {
    let mut park = Park::new(32, 32);
    let specs = loop_specs(3, 3, 8);
    draft(&mut park, &specs);
    assert_eq!(park.validate_track_draft().unwrap(), specs);
    assert!(park.circuit().is_none());
    roundtrip(&park);
    park.apply_track_draft().unwrap();
    assert_eq!(park.train().unwrap().phase, TrainPhase::Closed);
    assert_eq!(park.circuit().unwrap().pieces.len(), 8);
    assert_eq!(park.cash_cents, crate::INITIAL_PARK_CASH_CENTS - 19_000);
    assert_eq!(park.finances.entries.len(), 1);
    let before = park.clone();
    park.apply_track_draft().unwrap();
    assert_eq!(park, before);
    let next = park.next_object_id;
    park.next_object_id = u64::MAX;
    let exhausted = park.clone();
    park.apply_track_draft().unwrap();
    assert_eq!(park, exhausted);
    park.next_object_id = next;
    park.set_train_open(true);
    let mut loaded = roundtrip(&park);
    for _ in 0..2000 {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
        assert_eq!(park, loaded);
    }
    assert!(park.train().unwrap().laps >= 2);
    roundtrip(&park);
}

#[test]
fn editing_draft_preserves_live_world_until_apply_and_replaces_only_owned_track() {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    park.install_train_fixture().unwrap();
    park.set_train_open(false);
    let foreign = park
        .place_object(TilePos { x: 1, y: 1 }, ObjectKind::Track, Axis::X, 8, 2)
        .unwrap();
    let ride = park.ride().unwrap().id;
    park.set_ride_ticket_price(ride, 200).unwrap();
    let before = park.clone();
    let specs = loop_specs(24, 2, 8);
    draft(&mut park, &specs);
    let mut original = before.clone();
    original.track_draft = park.track_draft.clone();
    assert_eq!(park, original);
    park.apply_track_draft().unwrap();
    assert_eq!(park.ride, before.ride);
    assert_eq!(park.guests, before.guests);
    assert_eq!(
        park.finances.entries.len(),
        before.finances.entries.len() + 1
    );
    assert_eq!(park.cash_cents, before.cash_cents - 19_000);
    assert_eq!(park.crowd_rng, before.crowd_rng);
    assert_eq!(park.path_cache_stats(), before.path_cache_stats());
    assert_eq!(park.object(foreign), before.object(foreign));
    assert!(
        before
            .circuit()
            .unwrap()
            .pieces
            .iter()
            .all(|piece| park.object(piece.object).is_none())
    );
    assert!(
        park.circuit()
            .unwrap()
            .pieces
            .iter()
            .all(|piece| piece.object.0 >= before.next_object_id)
    );
    assert_eq!(park.train().unwrap().laps, before.train().unwrap().laps);
    roundtrip(&park);
}

#[test]
fn gaps_extra_islands_multiple_stations_and_wrong_ports_reject_without_changes() {
    let mut park = Park::new(32, 32);
    let specs = loop_specs(3, 3, 8);
    draft(&mut park, &specs);
    for variant in 0..4 {
        let mut invalid = park.clone();
        match variant {
            0 => {
                invalid.remove_track_draft_piece(specs[3].pos);
            }
            1 => {
                invalid
                    .set_track_draft_piece(TrackBuildKind::Straight.spec(
                        TilePos { x: 20, y: 20 },
                        8,
                        TrackDirection::East,
                    ))
                    .unwrap();
            }
            2 => {
                invalid
                    .set_track_draft_piece(TrackBuildKind::Station.spec(
                        specs[4].pos,
                        8,
                        TrackDirection::West,
                    ))
                    .unwrap();
            }
            _ => {
                invalid
                    .set_track_draft_piece(TrackBuildKind::Straight.spec(
                        specs[2].pos,
                        8,
                        TrackDirection::North,
                    ))
                    .unwrap();
            }
        }
        let before = invalid.clone();
        assert!(invalid.apply_track_draft().is_err());
        assert_eq!(invalid, before);
        roundtrip(&invalid);
    }
    let mut wrong_height = park.clone();
    wrong_height
        .set_track_draft_piece(TrackBuildKind::Straight.spec(
            specs[2].pos,
            9,
            TrackDirection::South,
        ))
        .unwrap();
    assert!(wrong_height.validate_track_draft().is_err());
    roundtrip(&wrong_height);
}

#[test]
fn blocked_drafts_survive_later_path_service_and_land_edits() {
    for obstacle in 0..3 {
        let mut park = Park::new(32, 32);
        let specs = loop_specs(3, 3, 8);
        draft(&mut park, &specs);
        let pos = specs[2].pos;
        match obstacle {
            0 => {
                park.place_path(pos, 8).unwrap();
            }
            1 => {
                park.place_service(pos, 8, PathDirection::North, ServiceKind::Drink)
                    .unwrap();
            }
            _ => {
                park.set_terrain_height(pos, 9).unwrap();
            }
        }
        let mut loaded = roundtrip(&park);
        let before = loaded.clone();
        assert!(loaded.apply_track_draft().is_err());
        assert_eq!(loaded, before);
        assert_eq!(
            loaded.track_draft(),
            specs.iter().rev().copied().collect::<Vec<_>>()
        );
    }
}

#[test]
fn unrelated_matching_track_is_never_claimed_and_preview_is_pure() {
    let mut park = Park::new(32, 32);
    let spec = TrackBuildKind::Station.spec(TilePos { x: 4, y: 3 }, 8, TrackDirection::East);
    let id = park
        .place_object(spec.pos, ObjectKind::Track, Axis::X, 8, 2)
        .unwrap();
    let before = park.clone();
    assert_eq!(
        park.validate_track_draft_piece(spec),
        Err(PlacementError::Occupied {
            pos: spec.pos,
            object: id
        }
        .into())
    );
    assert!(park.set_track_draft_piece(spec).is_err());
    assert_eq!(park, before);
    park.install_train_fixture().unwrap();
    let owned = park.circuit().unwrap().specs[0];
    assert_eq!(park.validate_track_draft_piece(owned), Ok(()));
}

#[test]
fn bounds_grade_terrain_and_capacity_validation_are_atomic() {
    let mut park = Park::new(32, 32);
    park.apply_terrain_profile(TerrainProfile::Terraced);
    let base = TrackBuildKind::Straight.spec(TilePos { x: 5, y: 5 }, 8, TrackDirection::East);
    for spec in [
        TrackPieceSpec {
            pos: TilePos { x: 32, y: 5 },
            ..base
        },
        TrackPieceSpec {
            entry: TrackPort {
                height: 31,
                ..base.entry
            },
            exit: TrackPort {
                height: 31,
                ..base.exit
            },
            ..base
        },
        TrackPieceSpec {
            kind: TrackKind::Lift,
            ..base
        },
        TrackPieceSpec {
            exit: TrackPort {
                height: 9,
                ..base.exit
            },
            ..base
        },
    ] {
        let before = park.clone();
        assert!(park.set_track_draft_piece(spec).is_err());
        assert_eq!(park, before);
    }
    park.clear_track_draft();
    for i in 0..MAX_TRACK_DRAFT_PIECES {
        park.set_track_draft_piece(TrackBuildKind::Straight.spec(
            TilePos {
                x: i as u32 % 32,
                y: i as u32 / 32,
            },
            20,
            TrackDirection::East,
        ))
        .unwrap();
    }
    let before = park.clone();
    assert_eq!(
        park.set_track_draft_piece(TrackBuildKind::Straight.spec(
            TilePos { x: 0, y: 9 },
            20,
            TrackDirection::East
        )),
        Err(TrackConstructionError::DraftFull)
    );
    assert_eq!(park, before);
    assert!(park.remove_track_draft_piece(TilePos { x: 0, y: 0 }));
    assert!(!park.remove_track_draft_piece(TilePos { x: 0, y: 0 }));
    park.clear_track_draft();
    assert!(park.track_draft().is_empty());
    roundtrip(&park);
}

#[test]
fn apply_requires_parked_empty_train_and_preserves_fault_recovery_remnants() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    draft(&mut park, &loop_specs(24, 2, 8));
    let before = park.clone();
    assert_eq!(
        park.apply_track_draft(),
        Err(TrackConstructionError::CloseRide)
    );
    assert_eq!(park, before);
    for _ in 0..150 {
        park.advance(Speed::Normal);
    }
    park.set_train_open(false);
    let before = park.clone();
    assert_eq!(
        park.apply_track_draft(),
        Err(TrackConstructionError::CloseRide)
    );
    assert_eq!(park, before);
    let missing = park.circuit().unwrap().pieces[2].object;
    park.remove_object(missing).unwrap();
    assert_eq!(park.train().unwrap().phase, TrainPhase::Fault);
    let remnants = park.objects.clone();
    park.apply_track_draft().unwrap();
    assert_eq!(&park.objects[..remnants.len()], &remnants);
    roundtrip(&park);
}

#[test]
fn active_coaster_transfer_blocks_apply_even_after_close() {
    let mut park = crate::experience::tests::fixture(true, 200);
    draft(&mut park, &loop_specs(24, 2, 8));
    for _ in 0..12000 {
        if park
            .guests
            .iter()
            .any(|g| g.ride_visit.as_ref().is_some_and(|v| v.passenger.is_some()))
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    assert!(
        park.guests
            .iter()
            .any(|g| g.ride_visit.as_ref().is_some_and(|v| v.passenger.is_some()))
    );
    park.set_train_open(false);
    let before = park.clone();
    assert!(park.apply_track_draft().is_err());
    assert_eq!(park, before);
    for _ in 0..24000 {
        if park.train().unwrap().phase == TrainPhase::Closed
            && park.guests.iter().all(|g| g.ride_visit.is_none())
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    park.apply_track_draft().unwrap();
    assert_eq!(
        park.ride_access(park.ride().unwrap().id).status,
        crate::RideAccessStatus::StationDisconnected
    );
    roundtrip(&park);
}

#[test]
fn allocator_and_object_capacity_failures_leave_complete_state_unchanged() {
    let mut park = Park::new(256, 256);
    draft(&mut park, &loop_specs(3, 3, 8));
    park.next_object_id = u64::MAX - 3;
    let before = park.clone();
    assert!(park.apply_track_draft().is_err());
    assert_eq!(park, before);
    park.next_object_id = 1;
    for i in 0..crate::saves::MAX_OBJECTS {
        park.place_object(
            TilePos {
                x: i as u32 % 256,
                y: i as u32 / 256 + 20,
            },
            ObjectKind::Track,
            Axis::X,
            8,
            2,
        )
        .unwrap();
    }
    let before = park.clone();
    assert!(park.apply_track_draft().is_err());
    assert_eq!(park, before);
}

#[test]
fn schema_fourteen_requires_strict_bounded_drafts_and_migrates_twelve_without_rewriting_state() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    park.place_service(
        TilePos { x: 2, y: 2 },
        8,
        PathDirection::North,
        ServiceKind::Food,
    )
    .unwrap();
    let base = json_value(&park);
    assert_eq!(base["schema_version"], json!(crate::SAVE_SCHEMA_VERSION));
    let mut old = base.clone();
    old["park"].as_object_mut().unwrap().remove("track_economy");
    old["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_operating_settings");
    old["park"].as_object_mut().unwrap().remove("ride_dispatch");
    old["schema_version"] = json!(12);
    old["park"].as_object_mut().unwrap().remove("coaster_cars");
    old["content_version"] = json!(12);
    old["park"].as_object_mut().unwrap().remove("track_draft");
    let loaded = read(&old).unwrap();
    assert_eq!(loaded, park);
    let mut hybrid = old.clone();
    hybrid["park"]["track_draft"] = json!([]);
    assert!(read(&hybrid).is_err());
    let mut missing = base.clone();
    missing["park"]
        .as_object_mut()
        .unwrap()
        .remove("track_draft");
    assert!(read(&missing).is_err());
    draft(&mut park, &loop_specs(24, 2, 8));
    let base = json_value(&park);
    let piece = base["park"]["track_draft"][0].clone();
    for bad in [
        json!(null),
        json!({}),
        json!(vec![piece.clone(); MAX_TRACK_DRAFT_PIECES + 1]),
        json!([piece.clone(), piece.clone()]),
    ] {
        let mut forged = base.clone();
        forged["park"]["track_draft"] = bad;
        assert!(read(&forged).is_err());
    }
    for (path, bad) in [
        ("/park/track_draft/0/pos", json!([24, 2])),
        ("/park/track_draft/0/kind", json!({"Curve":null})),
        ("/park/track_draft/0/entry", json!(["West", 8])),
        ("/park/track_draft/0/entry/height", json!(31)),
        ("/park/track_draft/0/exit/height", json!(9)),
        ("/park/track_draft/0/pos/x", json!(32)),
        ("/park/track_draft/0/kind", json!("Lift")),
    ] {
        let mut forged = base.clone();
        *forged.pointer_mut(path).unwrap() = bad;
        assert!(read(&forged).is_err(), "accepted {path}");
    }
    let mut unknown = base;
    unknown["park"]["track_draft"][0]["unknown"] = json!(0);
    assert!(read(&unknown).is_err());
    roundtrip(&park);
}

#[test]
fn schema_thirteen_migration_preserves_level_drafts_and_rejects_new_piece_kinds() {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    park.install_train_fixture().unwrap();
    draft(&mut park, &loop_specs(24, 2, 8));
    park.remove_track_draft_piece(TilePos { x: 24, y: 2 });
    for _ in 0..240 {
        park.advance(Speed::Normal);
    }
    let mut old = json_value(&park);
    old["park"].as_object_mut().unwrap().remove("track_economy");
    old["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_operating_settings");
    old["park"].as_object_mut().unwrap().remove("ride_dispatch");
    old["schema_version"] = json!(13);
    old["park"].as_object_mut().unwrap().remove("coaster_cars");
    old["content_version"] = json!(13);
    let mut migrated = read(&old).unwrap();
    assert_eq!(migrated, park);
    assert_eq!(
        json_value(&migrated)["schema_version"],
        json!(crate::SAVE_SCHEMA_VERSION)
    );
    assert_eq!(
        json_value(&migrated)["content_version"],
        json!(crate::SAVE_CONTENT_VERSION)
    );
    for _ in 0..300 {
        park.advance(Speed::Normal);
        migrated.advance(Speed::Normal);
        assert_eq!(migrated, park);
    }
    for kind in [
        TrackBuildKind::Lift,
        TrackBuildKind::Drop,
        TrackBuildKind::SteepDrop,
        TrackBuildKind::Brake,
    ] {
        let mut forged = old.clone();
        let piece = kind.spec(TilePos { x: 24, y: 2 }, 8, TrackDirection::East);
        forged["park"]["track_draft"] = json!([piece]);
        assert!(read(&forged).is_err(), "schema 13 accepted {kind:?}");
        forged["schema_version"] = json!(14);
        forged["park"]
            .as_object_mut()
            .unwrap()
            .remove("coaster_cars");
        forged["content_version"] = json!(14);
        assert_eq!(read(&forged).unwrap().track_draft(), [piece]);
    }
    let mut missing = old.clone();
    missing["park"]
        .as_object_mut()
        .unwrap()
        .remove("track_draft");
    assert!(read(&missing).is_err());
    let mut unknown = old;
    unknown["park"]["track_draft"][0]["future_field"] = json!(0);
    assert!(read(&unknown).is_err());
}

#[test]
fn schema_fourteen_rejects_bad_grades_and_ports_without_mutating_drafts() {
    let mut park = Park::new(32, 32);
    draft(&mut park, &graded_loop_specs());
    let base = json_value(&park);
    let pos = TilePos { x: 3, y: 3 };
    for (kind, exit_height, exit_edge) in [
        (TrackBuildKind::Lift, 10, TrackDirection::East),
        (TrackBuildKind::Drop, 9, TrackDirection::East),
        (TrackBuildKind::Drop, 5, TrackDirection::East),
        (TrackBuildKind::Brake, 7, TrackDirection::East),
        (TrackBuildKind::CurveRight, 7, TrackDirection::South),
        (TrackBuildKind::Lift, 9, TrackDirection::South),
        (TrackBuildKind::Drop, 7, TrackDirection::West),
    ] {
        let mut piece = kind.spec(pos, 8, TrackDirection::East);
        piece.exit = TrackPort {
            edge: exit_edge,
            height: exit_height,
        };
        let before = park.clone();
        assert_eq!(
            park.set_track_draft_piece(piece),
            Err(TrackConstructionError::UnsupportedPiece)
        );
        assert_eq!(park, before);
        let mut forged = base.clone();
        forged["park"]["track_draft"][0] = json!(piece);
        assert!(read(&forged).is_err());
    }
}
