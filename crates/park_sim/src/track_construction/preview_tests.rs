use super::tests::{graded_loop_specs, loop_specs};
use super::*;
use crate::{Axis, Speed};

fn warm_paths(park: &mut Park) {
    park.place_path(TilePos { x: 1, y: 1 }, 8).unwrap();
    park.path_cache.shared_graph(&park.objects);
    assert!(park.path_cache_stats().cached);
}

fn assert_unchanged(park: &Park, before: &Park, cache: crate::PathCacheStats) {
    assert_eq!(park, before);
    assert_eq!(park.path_cache_stats(), cache);
}

#[test]
fn candidate_quote_preserves_unrelated_draft_and_matches_paid_apply_and_retained_ids() {
    let mut park = Park::new(32, 32);
    let installed = graded_loop_specs();
    park.replace_track_draft(&installed).unwrap();
    park.apply_track_draft().unwrap();
    warm_paths(&mut park);
    let old_ids: Vec<_> = park
        .circuit()
        .unwrap()
        .pieces
        .iter()
        .map(|piece| piece.object)
        .collect();
    let mut candidate = installed.clone();
    candidate[11].kind = TrackKind::Straight;
    candidate.reverse();
    park.replace_track_draft(&loop_specs(24, 2, 8)[..3])
        .unwrap();
    assert!(park.track_construction_quote().is_err());
    let before = park.clone();
    let cache = park.path_cache_stats();
    let quote = park.track_construction_quote_for(&candidate).unwrap();
    assert_unchanged(&park, &before, cache);
    assert_eq!(
        (quote.construction_cents, quote.refund_cents),
        (1_000, 1_000)
    );
    assert_eq!(
        (
            quote.retained_pieces,
            quote.new_pieces,
            quote.removed_pieces
        ),
        (19, 1, 1)
    );
    assert_eq!(park.track_construction_operating_error(), None);
    park.replace_track_draft(&candidate).unwrap();
    assert_eq!(park.track_construction_quote().unwrap(), quote);
    park.apply_track_draft().unwrap();
    assert_eq!(park.cash_cents(), quote.balance_after().unwrap());
    assert_eq!(
        park.track_economy.construction_cents,
        before.track_economy.construction_cents + quote.construction_cents
    );
    assert_eq!(
        park.track_economy.refund_cents,
        before.track_economy.refund_cents + quote.refund_cents
    );
    for (index, piece) in park.circuit().unwrap().pieces.iter().enumerate() {
        assert_eq!(piece.object == old_ids[index], index != 11);
    }
    assert_eq!(park.path_cache_stats(), cache);
}

#[test]
fn invalid_candidates_share_apply_errors_and_leave_draft_ids_finances_and_cache_unchanged() {
    let mut park = Park::new(32, 32);
    warm_paths(&mut park);
    park.replace_track_draft(&loop_specs(3, 3, 8)).unwrap();
    let candidate = loop_specs(24, 2, 8);
    let mut duplicate = candidate.clone();
    duplicate.push(candidate[0]);
    let mut unsupported = candidate.clone();
    unsupported[2].kind = TrackKind::Lift;
    let mut outside = candidate.clone();
    outside[2].pos.x = 32;
    let mut high = candidate.clone();
    high[2].entry.height = 31;
    high[2].exit.height = 31;
    let mut disconnected = candidate.clone();
    disconnected.push(TrackBuildKind::Straight.spec(
        TilePos { x: 20, y: 20 },
        8,
        TrackDirection::East,
    ));
    for invalid in [
        Vec::new(),
        duplicate,
        unsupported,
        outside,
        high,
        candidate[..3].to_vec(),
        disconnected,
    ] {
        let before = park.clone();
        let cache = park.path_cache_stats();
        let error = park.track_construction_quote_for(&invalid).unwrap_err();
        assert_unchanged(&park, &before, cache);
        let mut apply = park.clone();
        // Exercise the same validator even for a malformed candidate that the
        // editor's ordinary draft replacement would already refuse.
        apply.track_draft = invalid;
        assert_eq!(apply.validate_track_draft(), Err(error.clone()));
        assert_eq!(apply.apply_track_draft(), Err(error));
    }
    park.place_object(candidate[2].pos, ObjectKind::Track, Axis::Y, 8, 2)
        .unwrap();
    let before = park.clone();
    let cache = park.path_cache_stats();
    let error = park.track_construction_quote_for(&candidate).unwrap_err();
    assert!(matches!(
        error,
        TrackConstructionError::Track(TrainError::Placement(PlacementError::Occupied { .. }))
    ));
    assert_unchanged(&park, &before, cache);
    park.remove_object(park.objects.last().unwrap().id).unwrap();
    park.next_object_id = u64::MAX;
    let before = park.clone();
    let cache = park.path_cache_stats();
    assert_eq!(
        park.track_construction_quote_for(&candidate),
        Err(PlacementError::IdExhausted.into())
    );
    assert_unchanged(&park, &before, cache);
}

#[test]
fn quote_reports_unaffordability_separately_from_operating_state_without_mutation() {
    let mut park = Park::new(32, 32);
    warm_paths(&mut park);
    park.cash_cents = 0;
    let candidate = loop_specs(24, 2, 8);
    let before = park.clone();
    let cache = park.path_cache_stats();
    let quote = park.track_construction_quote_for(&candidate).unwrap();
    assert_eq!(quote.construction_cents, 19_000);
    assert!(!quote.affordable());
    assert_eq!(park.track_construction_operating_error(), None);
    assert_unchanged(&park, &before, cache);
    park.replace_track_draft(&candidate).unwrap();
    let before = park.clone();
    assert_eq!(
        park.apply_track_draft(),
        Err(TrackConstructionError::InsufficientFunds {
            required_cents: quote.required_cents(),
            available_cents: 0
        })
    );
    assert_unchanged(&park, &before, cache);
}

#[test]
fn operating_preview_matches_all_train_guards_while_free_legacy_quote_remains_available() {
    let mut original = Park::new(32, 32);
    original.install_train_fixture().unwrap();
    original.set_train_open(false);
    let candidate = original.circuit().unwrap().specs().to_vec();
    original.replace_track_draft(&candidate).unwrap();
    warm_paths(&mut original);
    for (open, departed, speed, phase) in [
        (true, false, 0, TrainPhase::Closed),
        (false, true, 0, TrainPhase::Closed),
        (false, false, 1, TrainPhase::Closed),
        (false, false, 0, TrainPhase::Station),
        (false, false, 0, TrainPhase::Lift),
        (false, false, 0, TrainPhase::Closed),
        (false, false, 0, TrainPhase::Fault),
    ] {
        let mut park = original.clone();
        let train = park.train.as_mut().unwrap();
        train.open = open;
        train.departed = departed;
        train.speed = speed;
        train.phase = phase;
        let before = park.clone();
        let cache = park.path_cache_stats();
        let expected = if open
            || departed
            || speed != 0
            || !matches!(phase, TrainPhase::Closed | TrainPhase::Fault)
        {
            Some(TrackConstructionError::CloseRide)
        } else {
            None
        };
        assert_eq!(park.track_construction_operating_error(), expected);
        let quote = park.track_construction_quote_for(&candidate).unwrap();
        assert_eq!(
            (
                quote.construction_cents,
                quote.refund_cents,
                quote.retained_pieces
            ),
            (0, 0, 46)
        );
        assert_unchanged(&park, &before, cache);
        assert_eq!(park.apply_track_draft(), expected.map_or(Ok(()), Err));
        assert_unchanged(&park, &before, cache);
    }
}

#[test]
fn operating_preview_follows_occupied_train_then_guest_exit_until_apply_is_ready() {
    let mut park = crate::experience::tests::fixture(true, 200);
    let candidate = park.circuit().unwrap().specs().to_vec();
    park.replace_track_draft(&candidate).unwrap();
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
    for expected in [
        TrackConstructionError::CloseRide,
        TrackConstructionError::GuestsStillVisiting,
    ] {
        let before = park.clone();
        let cache = park.path_cache_stats();
        assert_eq!(
            park.track_construction_operating_error(),
            Some(expected.clone())
        );
        assert!(park.track_construction_quote_for(&candidate).is_ok());
        assert_eq!(park.apply_track_draft(), Err(expected.clone()));
        assert_unchanged(&park, &before, cache);
        if expected == TrackConstructionError::CloseRide {
            park.set_train_open(false);
            for _ in 0..24_000 {
                if park.track_construction_operating_error()
                    == Some(TrackConstructionError::GuestsStillVisiting)
                {
                    break;
                }
                park.advance(Speed::Normal);
            }
        }
    }
    for _ in 0..24_000 {
        if park.track_construction_operating_error().is_none() {
            break;
        }
        park.advance(Speed::Normal);
    }
    assert_eq!(park.track_construction_operating_error(), None);
    let before = park.clone();
    let cache = park.path_cache_stats();
    park.apply_track_draft().unwrap();
    assert_unchanged(&park, &before, cache);
}
