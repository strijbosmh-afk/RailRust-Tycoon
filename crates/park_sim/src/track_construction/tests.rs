use super::*;
use crate::{Axis, PathDirection, ServiceKind, Speed, TerrainProfile};
use serde_json::{Value, json};

fn loop_specs(x: u32, y: u32, height: u8) -> Vec<TrackPieceSpec> {
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
fn draft(park: &mut Park, specs: &[TrackPieceSpec]) {
    for spec in specs.iter().rev() {
        park.set_track_draft_piece(*spec).unwrap();
    }
}
fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(bytes, loaded.to_save_bytes().unwrap());
    loaded
}
fn json_value(park: &Park) -> Value {
    serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap()
}
fn read(value: &Value) -> Result<Park, crate::SaveError> {
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
    assert_eq!(park.cash_cents, crate::INITIAL_PARK_CASH_CENTS);
    assert!(park.finances.entries.is_empty());
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
    assert_eq!(park.finances, before.finances);
    assert_eq!(park.cash_cents, before.cash_cents);
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
fn schema_thirteen_requires_strict_bounded_drafts_and_migrates_twelve_without_rewriting_state() {
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
    assert_eq!(base["schema_version"], json!(13));
    let mut old = base.clone();
    old["schema_version"] = json!(12);
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
