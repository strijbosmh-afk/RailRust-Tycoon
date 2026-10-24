//! Purchased graded circuits, refunds, persisted drafts and paid passenger continuation.
use super::*;
use crate::track_tools::{self, TrackCommand, TrackEditor};
use park_sim::{
    FinanceKind, ObjectId, PathDirection, PathSlope, RideId, TrackBuildKind, TrackDirection,
    TrackKind, TrackPieceSpec, TrainPhase,
};

const REPLAY_TICKS: u64 = 7_200;

/// Use the production picker and edit path in captures and continuation proofs.
pub(crate) fn edit_draft_piece(
    park: &mut Park,
    view: &mut ViewState,
    kind: TrackBuildKind,
    pos: TilePos,
    height: u8,
    direction: TrackDirection,
) -> Result<(), String> {
    track_tools::apply_command(park, view, TrackCommand::Piece(kind));
    view.track_editor.direction = direction;
    view.track_editor.height = height;
    let spec = kind.spec(pos, height, direction);
    let shift = (f32::from(spec.exit.height) - f32::from(spec.entry.height))
        * 0.5
        * crate::world::HEIGHT_STEP;
    let point = track_tools::build_center(park, pos, height, view.rotation)
        + bevy::prelude::Vec2::Y * shift;
    track_tools::point_at(park, view, point, true);
    if !park.track_draft().contains(&spec) {
        return Err("production track picker did not apply the requested draft edit".into());
    }
    Ok(())
}

fn history_counts(view: &ViewState) -> (usize, usize) {
    (view.track_editor.undo_len(), view.track_editor.redo_len())
}

fn editing_session(directory: &Path) -> SaveSession {
    SaveSession::from_arguments([
        "--save-dir".into(),
        directory.join("editing-session").into_os_string(),
    ])
}

fn write_editing_session(directory: &Path) -> Result<(), String> {
    let mut park = demonstration_park();
    install_access(&mut park)?;
    draft(&mut park, true)?;
    apply(&mut park)?;
    park.clear_track_draft();
    let original = park.clone();
    let mut view = ViewState {
        track_editor: TrackEditor {
            active: true,
            ..Default::default()
        },
        ..Default::default()
    };
    track_tools::apply_command(&mut park, &mut view, TrackCommand::CopyInstalled);
    if park.track_draft() != specs() || history_counts(&view) != (1, 0) {
        return Err("copy installed did not create one exact graded draft history entry".into());
    }
    let copied = park.clone();
    let session = editing_session(directory);
    if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &directory.join("expected-editor-copy.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    edit_draft_piece(
        &mut park,
        &mut view,
        TrackBuildKind::Brake,
        TilePos { x: 14, y: 5 },
        11,
        TrackDirection::North,
    )?;
    let edited = park.clone();
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Undo);
    if park != copied || history_counts(&view) != (1, 1) {
        return Err("undo did not restore the copied graded draft exactly".into());
    }
    // An identical copy is a no-op, so it cannot destroy this redo.
    track_tools::apply_command(&mut park, &mut view, TrackCommand::CopyInstalled);
    if park != copied || history_counts(&view) != (1, 1) {
        return Err("identical copy destroyed redo or changed the live park".into());
    }
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Redo);
    if park != edited || history_counts(&view) != (2, 0) {
        return Err("redo did not recover the exact paid replacement draft".into());
    }
    unchanged_except_draft(&original, &park)?;
    let quote = park.track_construction_quote().map_err(|e| e.to_string())?;
    if (quote.construction_cents, quote.refund_cents) != (2_000, 500) {
        return Err("redone draft quote no longer matches the actual paid straight".into());
    }
    if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &directory.join("expected-editor-edited.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    Ok(())
}

fn verify_editing_session(directory: &Path) -> Result<(), String> {
    let session = editing_session(directory);
    let store = session
        .store
        .as_ref()
        .ok_or("invalid editor proof save directory")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let edited = read_expected(&directory.join("expected-editor-edited.park.json"))?;
    let copied = read_expected(&directory.join("expected-editor-copy.park.json"))?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    // Populate an unrelated session so a successful load must discard it.
    track_tools::apply_command(&mut park, &mut view, TrackCommand::CopyInstalled);
    if view.track_editor.undo_len() != 1 {
        return Err("editor load fixture did not create old history".into());
    }
    if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != edited || view.speed != Speed::Paused || history_counts(&view) != (0, 0) {
        return Err(
            "edited primary failed to load exactly, paused and without session history".into(),
        );
    }
    for command in [TrackCommand::Undo, TrackCommand::Redo] {
        track_tools::apply_command(&mut park, &mut view, command);
        if park != edited || history_counts(&view) != (0, 0) {
            return Err("loaded draft invented an undo/redo snapshot".into());
        }
    }
    track_tools::apply_command(&mut park, &mut view, TrackCommand::CopyInstalled);
    if park != copied || history_counts(&view) != (1, 0) {
        return Err("copy after load did not start a new draft-only editing session".into());
    }
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Undo);
    if park != edited || history_counts(&view) != (0, 1) {
        return Err("copy after load cannot be undone to the loaded replacement draft".into());
    }
    // Apply is the purchase boundary. No later history action can refund or
    // reconstruct the park from an earlier editing snapshot.
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Apply);
    if park.cash_cents() + 1_500 != edited.cash_cents() || history_counts(&view) != (0, 0) {
        return Err("paid Apply failed to settle the edited quote and clear history".into());
    }
    let applied = park.clone();
    for command in [TrackCommand::Undo, TrackCommand::Redo] {
        track_tools::apply_command(&mut park, &mut view, command);
        if park != applied {
            return Err("session history reversed a settled purchase".into());
        }
    }
    if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != copied || history_counts(&view) != (0, 0) {
        return Err("copied-plan backup did not restore exactly without session history".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("editing verification changed an original primary or backup slot".into());
    }
    Ok(())
}

/// One station, both curve directions, three chain lifts, both descending
/// grades and brakes. The station keeps its original entrance and exit portals.
pub(crate) fn specs() -> Vec<TrackPieceSpec> {
    use TrackBuildKind::{Brake, CurveLeft, CurveRight, Drop, Lift, Station, SteepDrop, Straight};
    use TrackDirection::{East, North, South, West};
    let mut pieces = Vec::new();
    let mut add = |x, y, kind: TrackBuildKind, height, direction| {
        pieces.push(kind.spec(TilePos { x, y }, height, direction));
    };
    add(8, 10, Station, 8, East);
    for x in 9..=14 {
        add(x, 10, Straight, 8, East);
    }
    add(15, 10, CurveLeft, 8, East);
    add(15, 9, Lift, 8, North);
    add(15, 8, Lift, 9, North);
    add(15, 7, CurveLeft, 10, North);
    add(14, 7, CurveRight, 10, West);
    add(14, 6, Lift, 10, North);
    add(14, 5, Straight, 11, North);
    add(14, 4, CurveLeft, 11, North);
    add(13, 4, Drop, 11, West);
    add(12, 4, SteepDrop, 10, West);
    add(11, 4, Brake, 8, West);
    for x in (8..=10).rev() {
        add(x, 4, Straight, 8, West);
    }
    add(7, 4, CurveLeft, 8, West);
    for y in 5..=9 {
        add(7, y, Straight, 8, South);
    }
    add(7, 10, CurveLeft, 8, South);
    pieces
}

/// Change the drop profile and move its brake without moving the station,
/// public crossing or height at either end of the rebuilt stretch.
fn replacement_specs() -> Vec<TrackPieceSpec> {
    specs()
        .into_iter()
        .map(|piece| {
            let (kind, height) = match (piece.pos.x, piece.pos.y) {
                (13, 4) => (TrackBuildKind::SteepDrop, 11),
                (12, 4) => (TrackBuildKind::Drop, 9),
                (11, 4) => (TrackBuildKind::Straight, 8),
                (10, 4) => (TrackBuildKind::Brake, 8),
                _ => return piece,
            };
            kind.spec(piece.pos, height, TrackDirection::West)
        })
        .collect()
}

pub(crate) fn install_access(park: &mut Park) -> Result<RideId, String> {
    park.set_train_open(false);
    let id = park
        .ride()
        .ok_or("track proof needs its original coaster")?
        .id;
    for (y, height, slope) in [
        (12, 8, PathSlope::Flat),
        (13, 8, PathSlope::South),
        (14, 9, PathSlope::Flat),
        (15, 8, PathSlope::North),
    ] {
        park.place_queue_path(TilePos { x: 8, y }, height, slope, id)
            .map_err(|e| e.to_string())?;
    }
    park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, id)
        .map_err(|e| e.to_string())?;
    // A public bridge crosses above the level southern track without overlap.
    // This also fits below the original graded circuit before its replacement.
    for pos in [TilePos { x: 8, y: 8 }, TilePos { x: 8, y: 7 }]
        .into_iter()
        .chain((9..=13).map(|x| TilePos { x, y: 7 }))
    {
        park.place_path(pos, 8).map_err(|e| e.to_string())?;
    }
    for (pos, height, slope) in [
        (TilePos { x: 13, y: 8 }, 8, PathSlope::South),
        (TilePos { x: 13, y: 9 }, 9, PathSlope::South),
        (TilePos { x: 13, y: 10 }, 10, PathSlope::Flat),
        (TilePos { x: 14, y: 10 }, 10, PathSlope::Flat),
        (TilePos { x: 14, y: 11 }, 9, PathSlope::North),
        (TilePos { x: 14, y: 12 }, 8, PathSlope::North),
        (TilePos { x: 14, y: 13 }, 8, PathSlope::Flat),
    ] {
        park.place_sloped_path(pos, height, slope)
            .map_err(|e| e.to_string())?;
    }
    park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, id)
        .map_err(|e| e.to_string())?;
    park.set_ride_ticket_price(id, 100)
        .map_err(|e| e.to_string())?;
    if park.ride_access(id).status != park_sim::RideAccessStatus::Ready
        || park.ride_exit_access(id).status != park_sim::RideExitStatus::Ready
    {
        return Err(
            "constructed circuit needs connected independent entrance and public exit".into(),
        );
    }
    Ok(id)
}

fn unchanged_except_draft(before: &Park, park: &Park) -> Result<(), String> {
    let strip = |park: &Park| -> Result<serde_json::Value, String> {
        let mut value: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        value["park"].as_object_mut().unwrap().remove("track_draft");
        Ok(value)
    };
    if strip(before)? != strip(park)? || before.path_cache_stats() != park.path_cache_stats() {
        return Err(
            "planning track changed live objects, IDs, RNG, guests, finances or paths".into(),
        );
    }
    Ok(())
}

/// Deliberately place every other piece backwards before the remaining half.
pub(crate) fn draft(park: &mut Park, complete: bool) -> Result<(), String> {
    let before = park.clone();
    let pieces = specs();
    for parity in 0..=usize::from(complete) {
        for (_, piece) in pieces
            .iter()
            .enumerate()
            .rev()
            .filter(|(i, _)| i % 2 == parity)
        {
            park.set_track_draft_piece(*piece)
                .map_err(|e| e.to_string())?;
        }
    }
    unchanged_except_draft(&before, park)?;
    super::ride_choice::bytes(park)?;
    if complete && park.validate_track_draft().map_err(|e| e.to_string())? != pieces {
        return Err("draft traversal depends on the order pieces were placed".into());
    }
    Ok(())
}

pub(crate) fn reject_apply(park: &mut Park) -> Result<(), String> {
    let before = park.clone();
    let cache = park.path_cache_stats();
    let bytes = super::ride_choice::bytes(park)?;
    if park.apply_track_draft().is_ok()
        || *park != before
        || park.path_cache_stats() != cache
        || super::ride_choice::bytes(park)? != bytes
    {
        return Err("refused track replacement changed complete state or path caches".into());
    }
    Ok(())
}

fn invalid_plans(park: &mut Park) -> Result<(), String> {
    // A draft with gaps can be saved, but cannot replace a working circuit.
    if park.validate_track_draft().is_ok() {
        return Err("incomplete draft was accepted".into());
    }
    reject_apply(park)?;
    let before = park.clone();
    let blocked = TrackBuildKind::Straight.spec(TilePos { x: 13, y: 16 }, 8, TrackDirection::East);
    if park.validate_track_draft_piece(blocked).is_ok()
        || park.set_track_draft_piece(blocked).is_ok()
        || *park != before
    {
        return Err("overlapping public-path track changed the draft or live world".into());
    }
    // A level rail fits under this bridge; a one-step lift at the same entry
    // height occupies its upper span and must be rejected before mutation.
    let under_bridge = TilePos { x: 13, y: 10 };
    park.validate_track_draft_piece(TrackBuildKind::Straight.spec(
        under_bridge,
        8,
        TrackDirection::East,
    ))
    .map_err(|e| format!("level rail no longer fits under the proof bridge: {e}"))?;
    let blocked_lift = TrackBuildKind::Lift.spec(under_bridge, 8, TrackDirection::East);
    if park.validate_track_draft_piece(blocked_lift).is_ok()
        || park.set_track_draft_piece(blocked_lift).is_ok()
        || *park != before
    {
        return Err("upper lift clearance failed to protect the public bridge atomically".into());
    }
    draft(park, true)?;
    // Every piece is geometrically legal, but one high endpoint cannot join
    // its lower neighbor. Such an incomplete height transition is saveable.
    let slope = specs()
        .into_iter()
        .find(|piece| piece.pos == (TilePos { x: 15, y: 8 }))
        .unwrap();
    park.set_track_draft_piece(TrackBuildKind::Lift.spec(slope.pos, 10, TrackDirection::North))
        .map_err(|e| e.to_string())?;
    if park.validate_track_draft().is_ok() {
        return Err("mismatched slope endpoint was accepted as a complete circuit".into());
    }
    super::ride_choice::bytes(park)?;
    reject_apply(park)?;
    park.set_track_draft_piece(slope)
        .map_err(|e| e.to_string())?;
    // A valid independent spare piece must not disappear silently at Apply.
    let spare = TrackBuildKind::Straight.spec(TilePos { x: 2, y: 2 }, 8, TrackDirection::East);
    park.set_track_draft_piece(spare)
        .map_err(|e| e.to_string())?;
    reject_apply(park)?;
    if !park.remove_track_draft_piece(spare.pos) {
        return Err("draft erase lost its target".into());
    }
    let corner = specs()
        .into_iter()
        .find(|piece| piece.pos == (TilePos { x: 14, y: 7 }))
        .unwrap();
    park.set_track_draft_piece(TrackBuildKind::CurveLeft.spec(
        corner.pos,
        corner.entry.height,
        TrackDirection::West,
    ))
    .map_err(|e| e.to_string())?;
    reject_apply(park)?;
    park.set_track_draft_piece(corner)
        .map_err(|e| e.to_string())?;
    if park.validate_track_draft().map_err(|e| e.to_string())? != specs() {
        return Err("correcting a reversed corner did not restore a closed loop".into());
    }
    Ok(())
}

/// Independent price oracle: do not call the simulation's pricing helper here.
fn piece_cost(spec: TrackPieceSpec) -> u64 {
    match spec.kind {
        TrackKind::Station => 10_000,
        TrackKind::Straight => 1_000,
        TrackKind::Curve => 1_500,
        TrackKind::Lift => 2_500,
        TrackKind::Drop if spec.entry.height.abs_diff(spec.exit.height) == 2 => 2_000,
        TrackKind::Drop => 1_500,
        TrackKind::Brake => 2_000,
    }
}

fn paid_refund(park: &Park, object: ObjectId) -> Result<u64, String> {
    let refund = match park
        .track_economy()
        .paid_pieces
        .iter()
        .find(|piece| piece.object == object)
    {
        Some(piece) => {
            if piece.paid_cents != piece_cost(piece.spec) {
                return Err("saved track basis disagrees with the independent piece prices".into());
            }
            piece.paid_cents / 2
        }
        None => 0,
    };
    if park.track_demolition_refund_cents(object) != refund {
        return Err("track demolition quote differs from half of its actual paid basis".into());
    }
    Ok(refund)
}

fn check_paid_history(park: &Park) -> Result<(), String> {
    let economy = park.track_economy();
    let mut active = 0;
    for piece in &economy.paid_pieces {
        active += 2 * paid_refund(park, piece.object)?;
        if park.object(piece.object).is_none() {
            return Err("a refunded or missing rail retained an active paid basis".into());
        }
    }
    if active + 2 * economy.refund_cents != economy.construction_cents {
        return Err(
            "track purchases, remaining paid rails and refunds do not conserve basis".into(),
        );
    }
    let restored =
        Park::from_save_bytes(&super::ride_choice::bytes(park)?).map_err(|e| e.to_string())?;
    if restored != *park {
        return Err("track paid bases or ledger changed across a strict save roundtrip".into());
    }
    Ok(())
}

pub(crate) fn apply(park: &mut Park) -> Result<(), String> {
    let before = park.clone();
    let owned: Vec<_> = before
        .circuit()
        .into_iter()
        .flat_map(|circuit| &circuit.pieces)
        .map(|p| p.object)
        .collect();
    let planned = park.validate_track_draft().map_err(|e| e.to_string())?;
    let old: Vec<_> = before
        .circuit()
        .into_iter()
        .flat_map(|circuit| circuit.specs().iter().zip(&circuit.pieces))
        .map(|(spec, piece)| (*spec, piece.object))
        .collect();
    let retained: Vec<_> = old
        .iter()
        .filter(|(spec, _)| planned.contains(spec))
        .copied()
        .collect();
    let removed: Vec<_> = old
        .iter()
        .filter(|(spec, _)| !planned.contains(spec))
        .map(|(_, id)| *id)
        .collect();
    let cost: u64 = planned
        .iter()
        .filter(|spec| !old.iter().any(|(old, _)| old == *spec))
        .map(|spec| piece_cost(*spec))
        .sum();
    let refund = removed
        .iter()
        .try_fold(0, |sum, id| paid_refund(&before, *id).map(|n| sum + n))?;
    let quote = park.track_construction_quote().map_err(|e| e.to_string())?;
    if *park != before
        || park.path_cache_stats() != before.path_cache_stats()
        || quote.construction_cents != cost
        || quote.refund_cents != refund
        || quote.retained_pieces != retained.len()
        || quote.new_pieces != planned.len() - retained.len()
        || quote.removed_pieces != removed.len()
        || quote.available_cents != before.cash_cents()
        || !quote.affordable()
    {
        return Err(
            "immutable construction quote differs from independently priced changed rails".into(),
        );
    }
    park.apply_track_draft().map_err(|e| e.to_string())?;
    let circuit = park.circuit().ok_or("applied draft has no circuit")?;
    let unrelated = |source: &Park| -> Result<serde_json::Value, String> {
        let mut value: serde_json::Value =
            serde_json::from_slice(&source.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        for field in [
            "objects",
            "next_object_id",
            "circuit",
            "train",
            "ride",
            "next_ride_id",
            "cash_cents",
            "finances",
            "track_economy",
        ] {
            value["park"].as_object_mut().unwrap().remove(field);
        }
        Ok(value)
    };
    if unrelated(&before)? != unrelated(park)? {
        return Err(
            "track Apply changed authoritative fields outside track construction and payment"
                .into(),
        );
    }
    let largest_old_id = before.objects().iter().map(|o| o.id.0).max().unwrap_or(0);
    if removed.iter().any(|id| park.object(*id).is_some()) {
        return Err("replacement retained a removed rail object".into());
    }
    for (spec, piece) in circuit.specs().iter().zip(&circuit.pieces) {
        if let Some((_, old_id)) = retained.iter().find(|(old, _)| old == spec) {
            if piece.object != *old_id
                || paid_refund(park, *old_id)? != paid_refund(&before, *old_id)?
            {
                return Err("unchanged track lost its identity or original paid basis".into());
            }
        } else if piece.object.0 <= largest_old_id
            || paid_refund(park, piece.object)? != piece_cost(*spec) / 2
        {
            return Err("new track reused an identity or lacks its exact paid basis".into());
        }
    }
    if (before.ride().is_some() && park.ride() != before.ride())
        || park.guests() != before.guests()
        || park.cash_cents() != before.cash_cents() + refund - cost
        || park.track_economy().construction_cents
            != before.track_economy().construction_cents + cost
        || park.track_economy().refund_cents != before.track_economy().refund_cents + refund
        || park.ticks() != before.ticks()
        || park.services() != before.services()
        || park.carousels() != before.carousels()
        || park.track_draft() != before.track_draft()
        || park.path_cache_stats() != before.path_cache_stats()
        || circuit.specs() != planned
        || circuit
            .pieces
            .iter()
            .filter(|p| p.kind == TrackKind::Station)
            .count()
            != 1
        || park.train().is_none_or(|t| {
            t.open
                || t.speed != 0
                || t.phase != TrainPhase::Closed
                || t.laps != before.train().map_or(0, |t| t.laps)
        })
        || before
            .objects()
            .iter()
            .filter(|o| !owned.contains(&o.id))
            .any(|o| park.object(o.id) != Some(o))
    {
        return Err("track Apply changed unrelated ownership/history or did not park the replacement closed".into());
    }
    let expected_entries: Vec<_> = [
        (FinanceKind::TrackRefund, refund),
        (FinanceKind::TrackConstruction, cost),
    ]
    .into_iter()
    .filter(|(_, amount)| *amount != 0)
    .collect();
    let after_ledger = park.finance_ledger();
    if after_ledger.next_entry_id
        != before.finance_ledger().next_entry_id + expected_entries.len() as u64
    {
        return Err("track Apply recorded the wrong number of cash transactions".into());
    }
    let mut balance = before.cash_cents();
    for (entry, (kind, amount)) in after_ledger
        .entries
        .iter()
        .rev()
        .take(expected_entries.len())
        .rev()
        .zip(expected_entries)
    {
        balance = if kind == FinanceKind::TrackRefund {
            balance + amount
        } else {
            balance - amount
        };
        if entry.kind != kind
            || entry.amount_cents != amount
            || entry.balance_cents != balance
            || entry.ride != Some(park.ride().unwrap().id)
            || entry.service.is_some()
            || entry.tick != park.ticks()
        {
            return Err(
                "track Apply ledger did not attribute refund-before-purchase to its coaster".into(),
            );
        }
    }
    check_paid_history(park)?;
    let applied = park.clone();
    park.apply_track_draft().map_err(|e| e.to_string())?;
    if *park != applied {
        return Err(
            "reapplying an unchanged circuit allocated IDs, charged cash or changed history".into(),
        );
    }
    Ok(())
}

fn economics_side_cases() -> Result<(), String> {
    use TrackBuildKind::{CurveRight, Station, Straight};
    use TrackDirection::{East, North, South, West};
    let mut park = Park::new(32, 32);
    let pieces: Vec<_> = [
        (21, 20, Station, East),
        (22, 20, CurveRight, East),
        (22, 21, Straight, South),
        (22, 22, CurveRight, South),
        (21, 22, Straight, West),
        (20, 22, CurveRight, West),
        (20, 21, Straight, North),
        (20, 20, CurveRight, North),
    ]
    .into_iter()
    .map(|(x, y, kind, direction)| kind.spec(TilePos { x, y }, 8, direction))
    .collect();
    for piece in &pieces {
        park.set_track_draft_piece(*piece)
            .map_err(|e| e.to_string())?;
    }
    apply(&mut park)?;
    if park.cash_cents() != 81_000 || park.track_economy().construction_cents != 19_000 {
        return Err("independent eight-piece purchase did not cost exactly $190.00".into());
    }

    // Seventy one-piece replacements exceed the retained ledger window while
    // leaving enough money for each transaction. A brake costs $20, a straight
    // $10; retaining the other seven rails must never repurchase them.
    for replacement in 0..70 {
        let kind = if replacement % 2 == 0 {
            TrackBuildKind::Brake
        } else {
            Straight
        };
        park.set_track_draft_piece(kind.spec(TilePos { x: 22, y: 21 }, 8, South))
            .map_err(|e| e.to_string())?;
        apply(&mut park)?;
        if park.cash_cents() != 81_000 - (replacement / 2 + 1) * 1_500 {
            return Err(
                "alternating paid rail replacements created cash or charged retained pieces".into(),
            );
        }
    }
    if park.cash_cents() != 28_500
        || park.finance_ledger().archived_entries == 0
        || park.finance_ledger().entries.len() != park_sim::FINANCE_LEDGER_CAPACITY
    {
        return Err("track replacement proof did not exercise the bounded ledger archive".into());
    }
    check_paid_history(&park)?;

    // The complete 28-piece plan costs $470.00. The old $190.00 circuit
    // refunds $95.00; $285.00 cash plus that refund cannot fund the purchase.
    let mut poor = park.clone();
    poor.clear_track_draft();
    draft(&mut poor, true)?;
    let before_quote = poor.clone();
    let quote = poor.track_construction_quote().map_err(|e| e.to_string())?;
    if poor != before_quote
        || quote.construction_cents != 47_000
        || quote.refund_cents != 9_500
        || quote.available_cents != 28_500
        || quote.affordable()
    {
        return Err(
            "unaffordable replacement quote changed state or used the wrong net price".into(),
        );
    }
    reject_apply(&mut poor)?;

    // Paid records outlive the circuit reference set. Removing its first rail
    // faults the train; all seven remaining rails still refund exactly once,
    // including when the partially demolished park is loaded again.
    let installed: Vec<_> = park
        .circuit()
        .unwrap()
        .specs()
        .iter()
        .zip(&park.circuit().unwrap().pieces)
        .map(|(spec, piece)| (*spec, piece.object))
        .collect();
    let before_demolition = park.cash_cents();
    let mut total_refund = 0;
    for (spec, object) in installed {
        let refund = piece_cost(spec) / 2;
        if paid_refund(&park, object)? != refund {
            return Err("paid remnant lost its independent per-piece demolition quote".into());
        }
        let cash = park.cash_cents();
        park.remove_object(object)
            .ok_or("closed paid track demolition failed")?;
        total_refund += refund;
        if park.cash_cents() != cash + refund
            || park.circuit().is_some()
            || park
                .train()
                .is_none_or(|train| train.phase != TrainPhase::Fault)
            || park.track_demolition_refund_cents(object) != 0
        {
            return Err(
                "partial demolition did not fault safely and refund just its paid rail".into(),
            );
        }
        check_paid_history(&park)?;
        park =
            Park::from_save_bytes(&super::ride_choice::bytes(&park)?).map_err(|e| e.to_string())?;
        let removed = park.clone();
        if park.remove_object(object).is_some() || park != removed {
            return Err("a removed paid track object could be refunded twice".into());
        }
    }
    if total_refund != 9_500
        || park.cash_cents() != before_demolition + total_refund
        || !park.track_economy().paid_pieces.is_empty()
    {
        return Err("complete demolition failed to refund half the surviving paid bases".into());
    }

    // Fixture and migrated rails have no paid basis. Their removal, including
    // the faulted remnants, cannot manufacture construction refunds or entries.
    let mut free = Park::new(32, 32);
    free.install_train_fixture().map_err(|e| e.to_string())?;
    free.set_train_open(false);
    let ids: Vec<_> = free
        .circuit()
        .unwrap()
        .pieces
        .iter()
        .map(|piece| piece.object)
        .collect();
    let cash = free.cash_cents();
    let ledger = free.finance_ledger().clone();
    for object in ids {
        if paid_refund(&free, object)? != 0 {
            return Err("free fixture track unexpectedly acquired a paid basis".into());
        }
        free.remove_object(object)
            .ok_or("free fixture rail removal failed")?;
        if free.cash_cents() != cash || *free.finance_ledger() != ledger {
            return Err("removing free legacy track manufactured a refund or cash entry".into());
        }
    }
    check_paid_history(&free)?;
    Ok(())
}

pub(crate) fn occupied(park: &mut Park) -> Result<(), String> {
    let ride = park.ride().unwrap().id;
    park.set_train_open(true);
    super::ride_choice::seek(
        park,
        |p| {
            super::ride_choice::paid_running(p, ride)
                && p.train().is_some_and(|t| {
                    let height = p.circuit().unwrap().sample(t.distance).height;
                    t.phase == TrainPhase::Lift
                        && t.speed == 24
                        && height % park_sim::TRACK_UNITS > park_sim::TRACK_UNITS / 3
                        && height % park_sim::TRACK_UNITS < 2 * park_sim::TRACK_UNITS / 3
                })
        },
        "paid passengers climbing a constructed chain lift",
    )?;
    if park.circuit().unwrap().pieces.len() != 28 {
        return Err("proof is still on the original fixed circuit".into());
    }
    Ok(())
}

#[derive(Default)]
struct MotionEvidence {
    lift: bool,
    drop: bool,
    brake: bool,
}

impl MotionEvidence {
    fn observe(&mut self, park: &mut Park) -> Result<(), String> {
        let train = park.train().ok_or("constructed circuit lost its train")?;
        let kind = park.circuit().unwrap().pieces[train.piece_index].kind;
        let newly_seen = match kind {
            TrackKind::Lift if train.phase == TrainPhase::Lift && train.speed == 24 => {
                let first = !self.lift;
                self.lift = true;
                first
            }
            TrackKind::Drop if train.phase == TrainPhase::Drop && train.speed > 24 => {
                let first = !self.drop;
                self.drop = true;
                first
            }
            TrackKind::Brake if train.phase == TrainPhase::Brake && train.speed == 20 => {
                let first = !self.brake;
                self.brake = true;
                first
            }
            _ => false,
        };
        if newly_seen {
            let restored = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
            if restored != *park {
                return Err("mid-lift/drop/brake roundtrip changed the complete park".into());
            }
            *park = restored;
        }
        Ok(())
    }

    fn require_complete(&self) -> Result<(), String> {
        if !self.lift || !self.drop || !self.brake {
            return Err(format!(
                "constructed operation missing lift/drop/brake evidence: {}/{}/{}",
                self.lift, self.drop, self.brake
            ));
        }
        Ok(())
    }
}

fn replay(park: &mut Park) -> Result<(), String> {
    let start = park.ticks();
    let ride = park.ride().unwrap().id;
    if !super::ride_choice::paid_running(park, ride) {
        return Err("track primary lacks moving paid passengers".into());
    }
    let before_copy = park.clone();
    let mut editing = ViewState {
        track_editor: TrackEditor {
            active: true,
            ..Default::default()
        },
        ..Default::default()
    };
    track_tools::apply_command(park, &mut editing, TrackCommand::CopyInstalled);
    unchanged_except_draft(&before_copy, park)?;
    let original = park.clone();
    edit_draft_piece(
        park,
        &mut editing,
        TrackBuildKind::Brake,
        TilePos { x: 14, y: 5 },
        11,
        TrackDirection::North,
    )?;
    let edited = park.clone();
    let counts = history_counts(&editing);
    track_tools::apply_command(park, &mut editing, TrackCommand::Apply);
    if *park != edited || history_counts(&editing) != counts {
        return Err("occupied Apply changed draft history, passengers or paid track".into());
    }
    track_tools::apply_command(park, &mut editing, TrackCommand::Undo);
    if *park != original {
        return Err("undo while passengers travel changed more than the draft".into());
    }
    let original_laps = park.train().unwrap().laps;
    let completed = park.ride().unwrap().completed;
    let mut original_motion = MotionEvidence::default();
    original_motion.observe(park)?;
    reject_apply(park)?;
    park.set_train_open(false);
    reject_apply(park)?;
    while !(park.train().is_some_and(|t| t.phase == TrainPhase::Closed)
        && park
            .guests()
            .iter()
            .all(|g| g.ride_visit.as_ref().is_none_or(|v| v.ride != ride)))
    {
        if park.ticks() >= start + REPLAY_TICKS {
            return Err("constructed train failed to close after returning every guest".into());
        }
        super::ride_choice::advance_checked(park)?;
        original_motion.observe(park)?;
    }
    original_motion.require_complete()?;
    if park.train().unwrap().laps <= original_laps || park.ride().unwrap().completed <= completed {
        return Err("constructed circuit did not complete a paid lap and return guests".into());
    }
    // Perform a genuinely different replacement after earning money/reviews.
    // Change both descending grades and move the brake, retaining public access.
    let before_draft = park.clone();
    track_tools::apply_command(park, &mut editing, TrackCommand::Clear);
    track_tools::apply_command(park, &mut editing, TrackCommand::CopyInstalled);
    for (kind, x, height) in [
        (TrackBuildKind::SteepDrop, 13, 11),
        (TrackBuildKind::Drop, 12, 9),
        (TrackBuildKind::Straight, 11, 8),
        (TrackBuildKind::Brake, 10, 8),
    ] {
        edit_draft_piece(
            park,
            &mut editing,
            kind,
            TilePos { x, y: 4 },
            height,
            TrackDirection::West,
        )?;
    }
    let replacement = park.clone();
    for _ in 0..4 {
        track_tools::apply_command(park, &mut editing, TrackCommand::Undo);
    }
    if park.track_draft() != specs() {
        return Err(
            "undoing the four regrading edits did not restore the copied live profile".into(),
        );
    }
    for _ in 0..4 {
        track_tools::apply_command(park, &mut editing, TrackCommand::Redo);
    }
    if *park != replacement
        || park.validate_track_draft().map_err(|e| e.to_string())? != replacement_specs()
    {
        return Err("redoing the regraded replacement changed its exact plan or live state".into());
    }
    unchanged_except_draft(&before_draft, park)?;
    apply(park)?;
    let rebuilt_laps = park.train().unwrap().laps;
    let rebuilt_completed = park.ride().unwrap().completed;
    let rebuilt_revenue = park.ride().unwrap().revenue_cents;
    let mut rebuilt_motion = MotionEvidence::default();
    park.set_train_open(true);
    if park.ticks() >= start + REPLAY_TICKS {
        return Err("constructed journey exceeded its bounded continuation".into());
    }
    while park.ticks() < start + REPLAY_TICKS {
        super::ride_choice::advance_checked(park)?;
        rebuilt_motion.observe(park)?;
        if (park.ticks() - start).is_multiple_of(997) {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    rebuilt_motion.require_complete()?;
    if park.train().unwrap().laps <= rebuilt_laps
        || park.ride().unwrap().completed <= rebuilt_completed
        || park.ride().unwrap().revenue_cents <= rebuilt_revenue
    {
        return Err(
            "regraded replacement did not earn another departure and complete a guest journey"
                .into(),
        );
    }
    super::ride_choice::bytes(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    economics_side_cases()?;
    write_editing_session(store.directory())?;
    let mut park = demonstration_park();
    install_access(&mut park)?;
    draft(&mut park, false)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    invalid_plans(&mut park)?;
    write_new(
        &store.directory().join("expected-ready.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    let mut missing_station = park.clone();
    if !missing_station.remove_track_draft_piece(TilePos { x: 8, y: 10 }) {
        return Err("native handoff fixture has no station to remove".into());
    }
    write_new(
        &store.directory().join("native-missing-station.park.json"),
        &super::ride_choice::bytes(&missing_station)?,
    )?;
    apply(&mut park)?;
    occupied(&mut park)?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    economics_side_cases()?;
    verify_editing_session(store.directory())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
    {
        return Err("constructed occupied primary did not load exactly and paused".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("constructed track continuation diverged after fresh load".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
        || park.track_draft().len() != 14
        || park.validate_track_draft().is_ok()
    {
        return Err("incomplete editable draft backup was not restored exactly and paused".into());
    }
    let frozen = park.clone();
    park.advance(Speed::Paused);
    if park != frozen {
        return Err("paused draft backup changed time or live state".into());
    }
    invalid_plans(&mut park)?;
    apply(&mut park)?;
    occupied(&mut park)?;
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("track verification changed an original save slot".into());
    }
    write_new(&store.directory().join("verification-track-construction.txt"), format!("Parkbound {}\nArbitrary-order station, curves, chain lifts, one/two-step drops and brakes form an exact directed height-matched loop: PASS\nIncomplete persisted draft changes no live objects, IDs, money, RNG or path cache: PASS\nIncomplete, disconnected, mismatched-height, reversed-corner and occupied-path rejection preserve full state: PASS\nWhole lift envelope protects the public bridge where a level rail still fits: PASS\nImmutable quotes match independent fixed piece prices and actual paid-basis refunds: PASS\nClosed Apply retains coaster identity, portals, guests, earned history and laps: PASS\nChanged drop/brake profile preserves unchanged rail IDs and paid bases, charges only changed rails, refunds retired paid rails and completes another paid journey: PASS\nRefund-before-purchase entries attribute exact amounts and running balances to the coaster: PASS\nRepeated Apply preserves existing piece IDs, cash and ledger: PASS\nSeventy paid straight/brake replacements archive ledger entries without profitable rebuild cycles: PASS\nUnaffordable complete replacement rejects atomically including cash, paid bases and ledger: PASS\nPartial demolition faults safely; saved remnants retain half-price refunds exactly once: PASS\nFree fixture and legacy-basis rails yield no refund or ledger entry: PASS\nPaid mid-lift primary, safe occupied refusal, complete lap and public guest return: PASS\nBoth built profiles show fixed-speed lifting, downhill acceleration and brake speed: PASS\nExact mid-lift/drop/brake roundtrips and fresh-process continuation: PASS ({REPLAY_TICKS} ticks)\nInstalled graded circuit copied through production commands without changing live paid track: PASS\nProduction pointer regrading, four undo/redo steps and occupied Apply refusal preserve live state: PASS\nFresh-process edited primary and copied-plan backup load exactly, paused, with empty session history: PASS\nIdentical Copy preserves redo; new editing after load stays draft-only; successful Apply clears history: PASS\nStrict15 roundtrips preserve paid bases, archived history and paused incomplete-draft backup: PASS\nOriginal primary and backup bytes unchanged: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graded_track_preserves_drafts_and_continues_paid_complete_journeys() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound track construction é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-track-construction", "verify-track-construction"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    directory.clone().into_os_string()
                ]),
                Some(AppExit::Success)
            );
        }
        assert!(
            directory
                .join("verification-track-construction.txt")
                .is_file()
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
