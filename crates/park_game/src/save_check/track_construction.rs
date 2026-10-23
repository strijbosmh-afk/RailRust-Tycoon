//! Player-built level circuits, persisted drafts and paid passenger continuation.
use super::*;
use park_sim::{
    PathDirection, PathSlope, RideId, TrackBuildKind, TrackDirection, TrackKind, TrackPieceSpec,
    TrainPhase,
};

const REPLAY_TICKS: u64 = 7_200;

/// One station, both curve directions and a small inward notch. The original
/// station position remains compatible with its entrance and exit portals.
pub(crate) fn specs() -> Vec<TrackPieceSpec> {
    use TrackBuildKind::{CurveLeft, CurveRight, Station, Straight};
    use TrackDirection::{East, North, South, West};
    let mut pieces = Vec::new();
    let mut add = |x, y, kind: TrackBuildKind, direction| {
        pieces.push(kind.spec(TilePos { x, y }, 8, direction));
    };
    add(8, 10, Station, East);
    for x in 9..=14 {
        add(x, 10, Straight, East);
    }
    add(15, 10, CurveLeft, East);
    for y in (8..=9).rev() {
        add(15, y, Straight, North);
    }
    add(15, 7, CurveLeft, North);
    add(14, 7, CurveRight, West);
    for y in (5..=6).rev() {
        add(14, y, Straight, North);
    }
    add(14, 4, CurveLeft, North);
    for x in (8..=13).rev() {
        add(x, 4, Straight, West);
    }
    add(7, 4, CurveLeft, West);
    for y in 5..=9 {
        add(7, y, Straight, South);
    }
    add(7, 10, CurveLeft, South);
    pieces
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
    // A public bridge crosses above the new level track without overlapping it.
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
    draft(park, true)?;
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
    park.set_track_draft_piece(TrackBuildKind::CurveLeft.spec(corner.pos, 8, TrackDirection::West))
        .map_err(|e| e.to_string())?;
    reject_apply(park)?;
    park.set_track_draft_piece(corner)
        .map_err(|e| e.to_string())?;
    if park.validate_track_draft().map_err(|e| e.to_string())? != specs() {
        return Err("correcting a reversed corner did not restore a closed loop".into());
    }
    Ok(())
}

pub(crate) fn apply(park: &mut Park) -> Result<(), String> {
    let before = park.clone();
    let owned: Vec<_> = before
        .circuit()
        .unwrap()
        .pieces
        .iter()
        .map(|p| p.object)
        .collect();
    park.apply_track_draft().map_err(|e| e.to_string())?;
    let circuit = park.circuit().ok_or("applied draft has no circuit")?;
    let unrelated = |source: &Park| -> Result<serde_json::Value, String> {
        let mut value: serde_json::Value =
            serde_json::from_slice(&source.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        for field in ["objects", "next_object_id", "circuit", "train"] {
            value["park"].as_object_mut().unwrap().remove(field);
        }
        Ok(value)
    };
    if unrelated(&before)? != unrelated(park)? {
        return Err(
            "track Apply changed authoritative fields outside track objects and train placement"
                .into(),
        );
    }
    if circuit.pieces != before.circuit().unwrap().pieces {
        let largest_old_id = before.objects().iter().map(|o| o.id.0).max().unwrap_or(0);
        if owned.iter().any(|id| park.object(*id).is_some())
            || circuit
                .pieces
                .iter()
                .any(|piece| piece.object.0 <= largest_old_id)
        {
            return Err("replacement retained stale track or reused an old object identity".into());
        }
    }
    if park.ride() != before.ride()
        || park.guests() != before.guests()
        || park.cash_cents() != before.cash_cents()
        || park.finance_ledger() != before.finance_ledger()
        || park.ticks() != before.ticks()
        || park.services() != before.services()
        || park.carousels() != before.carousels()
        || park.track_draft() != before.track_draft()
        || park.path_cache_stats() != before.path_cache_stats()
        || circuit.pieces.len() != 28
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
                || t.laps != before.train().unwrap().laps
        })
        || before
            .objects()
            .iter()
            .filter(|o| !owned.contains(&o.id))
            .any(|o| park.object(o.id) != Some(o))
    {
        return Err("track Apply changed unrelated ownership/history or did not park the replacement closed".into());
    }
    super::ride_choice::bytes(park)?;
    let applied = park.clone();
    park.apply_track_draft().map_err(|e| e.to_string())?;
    if *park != applied {
        return Err("reapplying an unchanged circuit allocated new IDs or changed history".into());
    }
    Ok(())
}

pub(crate) fn occupied(park: &mut Park) -> Result<(), String> {
    let ride = park.ride().unwrap().id;
    park.set_train_open(true);
    super::ride_choice::seek(
        park,
        |p| super::ride_choice::paid_running(p, ride),
        "paid departure on constructed track",
    )?;
    if park.circuit().unwrap().pieces.len() != 28 {
        return Err("proof is still on the original fixed circuit".into());
    }
    Ok(())
}

fn replay(park: &mut Park) -> Result<(), String> {
    let start = park.ticks();
    let ride = park.ride().unwrap().id;
    if !super::ride_choice::paid_running(park, ride) {
        return Err("track primary lacks moving paid passengers".into());
    }
    let original_laps = park.train().unwrap().laps;
    let completed = park.ride().unwrap().completed;
    reject_apply(park)?;
    park.set_train_open(false);
    reject_apply(park)?;
    super::ride_choice::seek(
        park,
        |p| {
            p.train().is_some_and(|t| t.phase == TrainPhase::Closed)
                && p.guests()
                    .iter()
                    .all(|g| g.ride_visit.as_ref().is_none_or(|v| v.ride != ride))
        },
        "closed constructed train with all guests safely returned",
    )?;
    if park.train().unwrap().laps <= original_laps || park.ride().unwrap().completed <= completed {
        return Err("constructed circuit did not complete a paid lap and return guests".into());
    }
    // Perform a genuinely different replacement after earning money/reviews.
    // Reversing every directed port retains station sides and all public access.
    let before_draft = park.clone();
    let reversed: Vec<_> = park
        .track_draft()
        .iter()
        .map(|piece| TrackPieceSpec {
            entry: piece.exit,
            exit: piece.entry,
            ..*piece
        })
        .collect();
    for piece in reversed {
        park.set_track_draft_piece(piece)
            .map_err(|e| e.to_string())?;
    }
    unchanged_except_draft(&before_draft, park)?;
    apply(park)?;
    let rebuilt_laps = park.train().unwrap().laps;
    let rebuilt_completed = park.ride().unwrap().completed;
    let rebuilt_revenue = park.ride().unwrap().revenue_cents;
    park.set_train_open(true);
    if park.ticks() >= start + REPLAY_TICKS {
        return Err("constructed journey exceeded its bounded continuation".into());
    }
    while park.ticks() < start + REPLAY_TICKS {
        super::ride_choice::advance_checked(park)?;
        if (park.ticks() - start).is_multiple_of(997) {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    if park.train().unwrap().laps <= rebuilt_laps
        || park.ride().unwrap().completed <= rebuilt_completed
        || park.ride().unwrap().revenue_cents <= rebuilt_revenue
    {
        return Err(
            "reversed replacement did not earn another departure and complete a guest journey"
                .into(),
        );
    }
    super::ride_choice::bytes(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
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
    write_new(&store.directory().join("verification-track-construction.txt"), format!("Parkbound {}\nArbitrary-order station, straights, left/right curves and exact directed loop: PASS\nIncomplete persisted draft changes no live objects, IDs, money, RNG or path cache: PASS\nIncomplete, disconnected, reversed-corner and occupied-path rejection preserve full state: PASS\nClosed Apply retains coaster identity, portals, guests, finances, history and laps: PASS\nChanged reverse-direction replacement preserves earned revenue/reviews/laps, allocates fresh track IDs and completes another paid journey: PASS\nRepeated Apply preserves existing piece IDs: PASS\nMoving paid primary, safe occupied refusal, complete lap and public guest return: PASS\nFresh-process exact continuation: PASS ({REPLAY_TICKS} ticks)\nStrict13 roundtrips, paused incomplete-draft backup and resumed construction: PASS\nOriginal primary and backup bytes unchanged: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constructed_track_preserves_drafts_and_continues_paid_complete_journeys() {
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
