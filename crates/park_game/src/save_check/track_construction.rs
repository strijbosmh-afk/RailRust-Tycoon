//! Player-built graded circuits, persisted drafts and paid passenger continuation.
use super::*;
use park_sim::{
    PathDirection, PathSlope, RideId, TrackBuildKind, TrackDirection, TrackKind, TrackPieceSpec,
    TrainPhase,
};

const REPLAY_TICKS: u64 = 7_200;

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
    for piece in replacement_specs() {
        park.set_track_draft_piece(piece)
            .map_err(|e| e.to_string())?;
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
    write_new(&store.directory().join("verification-track-construction.txt"), format!("Parkbound {}\nArbitrary-order station, curves, chain lifts, one/two-step drops and brakes form an exact directed height-matched loop: PASS\nIncomplete persisted draft changes no live objects, IDs, money, RNG or path cache: PASS\nIncomplete, disconnected, mismatched-height, reversed-corner and occupied-path rejection preserve full state: PASS\nWhole lift envelope protects the public bridge where a level rail still fits: PASS\nClosed Apply retains coaster identity, portals, guests, finances, history and laps: PASS\nChanged drop/brake profile preserves earned revenue/reviews/laps, allocates fresh track IDs and completes another paid journey: PASS\nRepeated Apply preserves existing piece IDs: PASS\nPaid mid-lift primary, safe occupied refusal, complete lap and public guest return: PASS\nBoth built profiles show fixed-speed lifting, downhill acceleration and brake speed: PASS\nExact mid-lift/drop/brake roundtrips and fresh-process continuation: PASS ({REPLAY_TICKS} ticks)\nStrict14 roundtrips, paused incomplete-draft backup and resumed construction: PASS\nOriginal primary and backup bytes unchanged: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
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
