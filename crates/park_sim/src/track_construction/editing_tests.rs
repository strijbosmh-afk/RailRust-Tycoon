use super::tests::{draft, graded_loop_specs, json_value, loop_specs, roundtrip};
use super::*;
use crate::{Axis, Speed};

#[test]
fn copying_paid_graded_track_preserves_the_complete_park_except_its_draft() {
    let mut park = Park::new(32, 32);
    let specs = graded_loop_specs();
    draft(&mut park, &specs);
    park.apply_track_draft().unwrap();
    park.clear_track_draft();
    let mut expected = park.clone();
    expected.track_draft = specs.clone();
    assert_eq!(park.copy_installed_track_to_draft(), Ok(true));
    assert_eq!(park, expected);
    assert_eq!(park.track_draft(), specs);
    let quote = park.track_construction_quote().unwrap();
    assert_eq!(quote.retained_pieces, specs.len());
    assert_eq!((quote.construction_cents, quote.refund_cents), (0, 0));

    let before = park.clone();
    assert_eq!(park.copy_installed_track_to_draft(), Ok(false));
    assert_eq!(park.replace_track_draft(&specs), Ok(false));
    park.apply_track_draft().unwrap();
    assert_eq!(park, before);
    roundtrip(&park);
}

#[test]
fn copying_open_legacy_fixture_preserves_exact_grades_and_free_basis() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    let specs = park.circuit().unwrap().specs().to_vec();
    assert_eq!(specs.len(), 46);
    assert!(specs.iter().any(|spec| spec.kind == TrackKind::Lift));
    assert!(
        specs.iter().any(|spec| {
            spec.kind == TrackKind::Drop && spec.entry.height - spec.exit.height == 2
        })
    );
    assert!(park.train().unwrap().open);
    let mut expected = park.clone();
    expected.track_draft = specs;
    assert_eq!(park.copy_installed_track_to_draft(), Ok(true));
    assert_eq!(park, expected);
    assert_eq!(*park.track_economy(), TrackEconomy::default());
    assert_eq!(park.track_construction_quote().unwrap().required_cents(), 0);
    park.set_train_open(false);
    let before = park.clone();
    park.apply_track_draft().unwrap();
    assert_eq!(park, before);
    roundtrip(&park);
}

#[test]
fn copying_a_moving_occupied_coaster_does_not_interrupt_passengers_or_train() {
    let mut park = crate::experience::tests::fixture(true, 200);
    for _ in 0..12_000 {
        if park.train().unwrap().departed
            && park.guests.iter().any(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.passenger.is_some())
            })
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    assert!(park.train().unwrap().departed);
    assert!(park.guests.iter().any(|guest| {
        guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| visit.passenger.is_some())
    }));
    let mut expected = park.clone();
    expected.track_draft = park.circuit().unwrap().specs().to_vec();
    assert_eq!(park.copy_installed_track_to_draft(), Ok(true));
    assert_eq!(park, expected);
    assert_eq!(
        park.apply_track_draft(),
        Err(TrackConstructionError::CloseRide)
    );
    assert_eq!(park, expected);
    roundtrip(&park);
}

#[test]
fn missing_faulted_and_invalid_installed_circuits_cannot_replace_a_plan() {
    let mut park = Park::new(32, 32);
    draft(&mut park, &loop_specs(3, 3, 8));
    let before = park.clone();
    assert_eq!(
        park.copy_installed_track_to_draft(),
        Err(TrackConstructionError::NoInstalledCircuit)
    );
    assert_eq!(park, before);

    park.install_train_fixture().unwrap();
    let missing = park.circuit().unwrap().pieces[2].object;
    park.remove_object(missing).unwrap();
    let before = park.clone();
    assert_eq!(
        park.copy_installed_track_to_draft(),
        Err(TrackConstructionError::NoInstalledCircuit)
    );
    assert_eq!(park, before);

    park.install_train_fixture().unwrap();
    // A corrupt source must fail atomically rather than silently flattening it.
    park.circuit.as_mut().unwrap().specs[0].kind = TrackKind::Lift;
    let before = park.clone();
    assert_eq!(
        park.copy_installed_track_to_draft(),
        Err(TrackConstructionError::UnsupportedPiece)
    );
    assert_eq!(park, before);
}

#[test]
fn draft_restore_rejects_invalid_geometry_bounds_duplicates_and_size_atomically() {
    let mut park = Park::new(32, 32);
    let existing = loop_specs(3, 3, 8);
    park.replace_track_draft(&existing).unwrap();
    let valid = TrackBuildKind::Straight.spec(TilePos { x: 8, y: 8 }, 8, TrackDirection::East);
    let mut malformed = valid;
    malformed.kind = TrackKind::Lift;
    let mut high = valid;
    high.entry.height = crate::MAX_TERRAIN_HEIGHT;
    high.exit.height = crate::MAX_TERRAIN_HEIGHT;
    let mut outside = valid;
    outside.pos.x = 32;
    let oversized: Vec<_> = (0..=MAX_TRACK_DRAFT_PIECES)
        .map(|index| TrackPieceSpec {
            pos: TilePos {
                x: index as u32 % 32,
                y: index as u32 / 32,
            },
            ..valid
        })
        .collect();
    for invalid in [
        vec![valid, valid],
        vec![valid, malformed],
        vec![valid, high],
        vec![valid, outside],
        oversized,
    ] {
        let before = park.clone();
        assert!(park.replace_track_draft(&invalid).is_err());
        assert_eq!(park, before);
    }
}

#[test]
fn restoring_a_blocked_or_incomplete_snapshot_is_allowed_but_apply_revalidates() {
    let mut park = Park::new(32, 32);
    let specs = loop_specs(3, 3, 8);
    park.replace_track_draft(&specs).unwrap();
    park.clear_track_draft();
    park.place_object(specs[2].pos, ObjectKind::Track, Axis::Y, 8, 2)
        .unwrap();
    assert_eq!(park.replace_track_draft(&specs), Ok(true));
    let before = park.clone();
    assert!(matches!(
        park.apply_track_draft(),
        Err(TrackConstructionError::Track(TrainError::Placement(
            PlacementError::Occupied { .. }
        )))
    ));
    assert_eq!(park, before);
    roundtrip(&park);

    park.replace_track_draft(&specs[..3]).unwrap();
    assert!(park.validate_track_draft().is_err());
    let json = json_value(&park);
    assert_eq!(json["schema_version"], crate::SAVE_SCHEMA_VERSION);
    assert_eq!(json["content_version"], crate::SAVE_CONTENT_VERSION);
    assert_eq!(json["park"]["track_draft"].as_array().unwrap().len(), 3);
    roundtrip(&park);
}

#[test]
fn restoring_maximum_size_preserves_order_and_clearing_is_a_real_edit_once() {
    let mut park = Park::new(32, 32);
    let specs: Vec<_> = (0..MAX_TRACK_DRAFT_PIECES)
        .rev()
        .map(|index| {
            TrackBuildKind::Straight.spec(
                TilePos {
                    x: index as u32 % 32,
                    y: index as u32 / 32,
                },
                8,
                TrackDirection::East,
            )
        })
        .collect();
    assert_eq!(park.replace_track_draft(&specs), Ok(true));
    assert_eq!(park.track_draft(), specs);
    roundtrip(&park);
    assert_eq!(park.replace_track_draft(&[]), Ok(true));
    let before = park.clone();
    assert_eq!(park.replace_track_draft(&[]), Ok(false));
    assert_eq!(park, before);
}
