//! Fresh-process boarding, occupied laps, exit recovery and completed journeys.
use super::*;
use park_sim::{
    GuestState, PathDirection, PathSlope, RIDE_TRANSFER_TICKS, RideId, RideVisitPhase, TRAIN_SEATS,
    TrainPhase,
};
use std::collections::BTreeSet;

const EXIT: TilePos = TilePos { x: 8, y: 9 };
const REPLAY_TICKS: u64 = 7_200;
const SEEK_LIMIT: usize = 18_000;

fn ride_id(park: &Park) -> Result<RideId, String> {
    park.ride()
        .map(|ride| ride.id)
        .ok_or("journey proof needs a ride".into())
}

pub(super) fn install(park: &mut Park) -> Result<RideId, String> {
    let ride = ride_id(park)?;
    for (y, height, slope) in [
        (12, 8, PathSlope::Flat),
        (13, 8, PathSlope::South),
        (14, 9, PathSlope::Flat),
        (15, 8, PathSlope::North),
    ] {
        park.place_queue_path(TilePos { x: 8, y }, height, slope, ride)
            .map_err(|error| error.to_string())?;
    }
    park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, ride)
        .map_err(|error| error.to_string())?;
    install_exit(park, ride)?;
    Ok(ride)
}

pub(super) fn install_exit(park: &mut Park, ride: RideId) -> Result<(), String> {
    for x in 8..=13 {
        park.place_path(TilePos { x, y: 8 }, 8)
            .map_err(|error| error.to_string())?;
    }
    for y in 9..=11 {
        park.place_path(TilePos { x: 13, y }, 8)
            .map_err(|error| error.to_string())?;
    }
    park.place_ride_exit(EXIT, 8, PathDirection::South, ride)
        .map_err(|error| error.to_string())?;
    if park.ride_exit_access(ride).status != park_sim::RideExitStatus::Ready {
        return Err("journey exit must reach the queue's public component".into());
    }
    Ok(())
}

fn passenger_count(park: &Park) -> usize {
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

fn boarding(park: &Park) -> bool {
    park.guests().iter().any(|guest| {
        guest.ride_visit.as_ref().is_some_and(|visit| {
            visit.phase == RideVisitPhase::Boarding
                && visit
                    .passenger
                    .as_ref()
                    .and_then(|passenger| passenger.transfer.as_ref())
                    .is_some_and(|transfer| {
                        transfer.progress > 0 && transfer.progress < RIDE_TRANSFER_TICKS
                    })
        })
    })
}

fn riding(park: &Park) -> bool {
    park.train().is_some_and(|train| train.speed > 0)
        && park.guests().iter().any(|guest| {
            guest.ride_visit.as_ref().is_some_and(|visit| {
                visit.phase == RideVisitPhase::Riding
                    && visit
                        .passenger
                        .as_ref()
                        .is_some_and(|passenger| passenger.departed && !passenger.completed_lap)
            })
        })
}

pub(super) fn require_membership(park: &Park) -> Result<(), String> {
    super::admission::require_roles(park)?;
    let ride = ride_id(park)?;
    let mut seats = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut counts = [0usize; 3];
    for guest in park.guests() {
        let Some(visit) = &guest.ride_visit else {
            continue;
        };
        let index = match visit.phase {
            RideVisitPhase::Boarding => 0,
            RideVisitPhase::Riding => 1,
            RideVisitPhase::Exiting => 2,
            _ => {
                if visit.passenger.is_some() {
                    return Err("ordinary visitor retained a seat".into());
                }
                continue;
            }
        };
        let passenger = visit
            .passenger
            .as_ref()
            .ok_or("passenger phase has no seat membership")?;
        if visit.ride != ride
            || usize::from(passenger.seat) >= TRAIN_SEATS
            || !seats.insert(passenger.seat)
            || !ids.insert(guest.id)
            || guest.current.is_some()
            || guest.next.is_some()
            || guest.destination.is_some()
            || guest.progress != 0
        {
            return Err("passenger identity/seat/walking state is inconsistent".into());
        }
        counts[index] += 1;
        if index == 1 {
            if passenger.transfer.is_some() || guest.state != GuestState::Waiting {
                return Err("riding passenger has a transfer or walking state".into());
            }
        } else {
            let transfer = passenger
                .transfer
                .as_ref()
                .ok_or("boarding/exit phase has no transfer")?;
            let path = park
                .object(transfer.path)
                .ok_or("transfer path is absent")?;
            let portal = park
                .object(transfer.portal)
                .ok_or("transfer portal is absent")?;
            if transfer.progress >= RIDE_TRANSFER_TICKS
                || guest.state != GuestState::Walking
                || path.kind != ObjectKind::Path
                || (index == 0
                    && (path.queue_owner != Some(ride) || portal.kind != ObjectKind::RideEntrance))
                || (index == 2
                    && (path.queue_owner.is_some() || portal.kind != ObjectKind::RideExit))
            {
                return Err("transfer progress, role or portal is inconsistent".into());
            }
        }
    }
    let stats = park.ride_journey_stats(ride);
    let positions = park.ride_guest_positions();
    let positioned: BTreeSet<_> = positions.iter().map(|position| position.guest).collect();
    if [stats.boarding, stats.riding, stats.exiting] != counts
        || stats.capacity != TRAIN_SEATS
        || counts.iter().sum::<usize>() > TRAIN_SEATS
        || positioned != ids
        || positioned.len() != positions.len()
        || stats.completed
            != park
                .guests()
                .iter()
                .map(|guest| guest.rides_completed)
                .sum::<u64>()
    {
        return Err("journey counts, positions or completed trips disagree".into());
    }
    Ok(())
}

fn roundtrip(park: &Park) -> Result<Vec<u8>, String> {
    require_membership(park)?;
    let bytes = park.to_save_bytes().map_err(|error| error.to_string())?;
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if json["schema_version"] != 9 || json["content_version"] != 9 {
        return Err("journey proof requires strict schema/content 9/9".into());
    }
    if Park::from_save_bytes(&bytes).map_err(|error| error.to_string())? != *park {
        return Err("journey save roundtrip changed authoritative state".into());
    }
    Ok(bytes)
}

fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool, label: &str) -> Result<(), String> {
    for _ in 0..SEEK_LIMIT {
        if predicate(park) {
            return Ok(());
        }
        park.advance(Speed::Normal);
        require_membership(park)?;
    }
    Err(format!(
        "journey proof did not reach {label} within {SEEK_LIMIT} ticks"
    ))
}

fn require_protected(park: &mut Park, object: ObjectId) -> Result<(), String> {
    let before = park.clone();
    let cache = park.path_cache_stats();
    let bytes = roundtrip(park)?;
    if park.object_removal_block_reason(object).is_none()
        || park.remove_object(object).is_some()
        || *park != before
        || park.path_cache_stats() != cache
        || roundtrip(park)? != bytes
    {
        return Err("occupied infrastructure removal was not rejected atomically".into());
    }
    Ok(())
}

fn protect_transfers(park: &mut Park) -> Result<bool, String> {
    let transfer = park.guests().iter().find_map(|guest| {
        guest
            .ride_visit
            .as_ref()
            .and_then(|visit| visit.passenger.as_ref())
            .and_then(|passenger| passenger.transfer.as_ref())
            .map(|transfer| (transfer.path, transfer.portal))
    });
    if let Some((path, portal)) = transfer {
        require_protected(park, path)?;
        require_protected(park, portal)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn continue_journey(park: &mut Park) -> Result<(), String> {
    if !riding(park) {
        return Err("journey primary must contain a moving occupied train".into());
    }
    let ride = ride_id(park)?;
    let ticks = park.ticks();
    let ids: Vec<_> = park.guests().iter().map(|guest| guest.id).collect();
    let initial = park.ride_journey_stats(ride);
    let protected_track = park.circuit().unwrap().pieces[0].object;
    require_protected(park, protected_track)?;
    let mut removed_exit = None;
    let mut held_since = None;
    let mut repaired_exit = false;
    let mut saw_exit_transfer = false;
    let mut faulted_at = None;
    let mut removed_track = None;
    let mut repaired_track = false;
    let mut completed_before_reopen = 0;
    let mut saw_second_completion = false;
    for tick in 0..REPLAY_TICKS {
        let mut changed = false;
        if tick == 30 {
            if !riding(park) {
                return Err("train returned before the missing-exit command".into());
            }
            park.set_train_open(false);
            let exit = park
                .ride()
                .unwrap()
                .exit
                .as_ref()
                .ok_or("exit absent before demolition")?
                .object;
            park.remove_object(exit)
                .ok_or("unused exit must be removable during the lap")?;
            removed_exit = Some(exit);
            changed = true;
        }
        let stats = park.ride_journey_stats(ride);
        if removed_exit.is_some()
            && !repaired_exit
            && stats.held
            && stats.riding > 0
            && park.train().unwrap().speed == 0
        {
            let first = *held_since.get_or_insert(tick);
            if tick == first {
                roundtrip(park)?;
            }
            if tick >= first + 30 {
                let fresh = park
                    .place_ride_exit(EXIT, 8, PathDirection::South, ride)
                    .map_err(|error| error.to_string())?;
                if Some(fresh) <= removed_exit {
                    return Err("exit repair reused its object identity".into());
                }
                repaired_exit = true;
                changed = true;
            }
        }
        if repaired_exit && !saw_exit_transfer && stats.exiting > 0 {
            saw_exit_transfer = protect_transfers(park)?;
            roundtrip(park)?;
        }
        if repaired_exit
            && saw_exit_transfer
            && faulted_at.is_none()
            && passenger_count(park) == 0
            && stats.completed > initial.completed
        {
            if park.train().unwrap().open {
                return Err("closing occupied lap silently reopened".into());
            }
            completed_before_reopen = stats.completed;
            let track = park.circuit().unwrap().pieces[0].object;
            park.remove_object(track)
                .ok_or("empty-ride track removal must succeed")?;
            if park.train().unwrap().phase != TrainPhase::Fault {
                return Err("empty track demolition did not fault the train".into());
            }
            removed_track = Some(track);
            faulted_at = Some(tick);
            changed = true;
        }
        if let Some(fault_tick) = faulted_at
            && !repaired_track
            && tick >= fault_tick + 30
        {
            let before_ticks = park.ticks();
            park.install_train_fixture()
                .map_err(|error| error.to_string())?;
            if park.ticks() != before_ticks
                || park.train().unwrap().phase != TrainPhase::Closed
                || park.train().unwrap().open
                || ride_id(park)? != ride
                || park.circuit().unwrap().pieces[0].object <= removed_track.unwrap()
            {
                return Err("empty fault repair did not preserve identity/time and park closed with fresh track".into());
            }
            roundtrip(park)?;
            park.set_train_open(true);
            repaired_track = true;
            changed = true;
        }
        if changed {
            roundtrip(park)?;
        }
        park.advance(Speed::Normal);
        require_membership(park)?;
        saw_second_completion |=
            repaired_track && park.ride_journey_stats(ride).completed > completed_before_reopen;
        if park
            .guests()
            .iter()
            .map(|guest| guest.id)
            .collect::<Vec<_>>()
            != ids
            || ride_id(park)? != ride
        {
            return Err(
                "journey replay lost, duplicated or reordered guest/ride identities".into(),
            );
        }
    }
    if park.ticks() != ticks + REPLAY_TICKS
        || held_since.is_none()
        || !repaired_exit
        || !saw_exit_transfer
        || !repaired_track
        || !saw_second_completion
    {
        return Err("journey replay did not witness held passengers, exit recovery, empty fault repair and another completed trip".into());
    }
    roundtrip(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    for _ in 0..731 {
        park.advance(Speed::Normal);
    }
    install(&mut park)?;
    seek(&mut park, boarding, "actual boarding transfer")?;
    if !protect_transfers(&mut park)? {
        return Err("boarding proof has no protected transfer".into());
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &roundtrip(&park)?,
    )?;
    seek(&mut park, riding, "a departed occupied train")?;
    park.set_terrain_height(TilePos { x: 3, y: 24 }, 1)
        .map_err(|error| error.to_string())?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &roundtrip(&park)?,
    )?;
    continue_journey(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &roundtrip(&park)?,
    )?;
    println!("Complete ride journey continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|error| error.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    view.path_tool.set_mode(crate::path_tools::PathTool::Exit);
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || !view.terrain_custom
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
        || !riding(&park)
    {
        return Err(
            "journey primary must restore exact occupied motion paused/Custom/Inspect".into(),
        );
    }
    continue_journey(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("reloaded journey continuation differs from uninterrupted state".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
        || !boarding(&park)
    {
        return Err("journey backup must restore exact boarding transfer paused/Inspect".into());
    }
    roundtrip(&park)?;
    if fs::read(store.save_path()).map_err(|error| error.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|error| error.to_string())? != backup
    {
        return Err("journey verification changed primary or backup bytes".into());
    }
    write_new(&store.directory().join("verification-journey.txt"), format!(
        "Parkbound {}\nFresh-process occupied moving primary equality: PASS\nContinuation: PASS (7200 complete ticks + close, missing exit, fresh-ID exit repair, empty fault/closed repair/reopen)\nNatural boarding, occupied lap, exit transfer and completed public returns: PASS\nSeat uniqueness, role/transfer reservations and guest/ride identity conservation: PASS\nOccupied circuit and active transfer demolition rejected atomically: PASS\nMissing-exit station hold, unloading recovery and another completed trip: PASS\nExplicit boarding backup and paused/Inspect loading: PASS\nPrimary and backup files unchanged: PASS\nStrict schema/content 9/9: PASS\n", env!("CARGO_PKG_VERSION")
    ).as_bytes())?;
    println!("Fresh-process complete journey and boarding backup verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn journey_modes_replay_occupied_train_exit_recovery_and_preserve_both_slots() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!(
            "Parkbound journey proof é {} {stamp}",
            std::process::id()
        )));
        let arguments = |mode: &str| {
            vec![
                OsString::from("--save-check"),
                OsString::from(mode),
                OsString::from("--save-dir"),
                scratch.0.as_os_str().to_owned(),
            ]
        };
        assert_eq!(run(arguments("write-journey")), Some(AppExit::Success));
        let store = SaveSession::from_arguments(arguments("verify-journey"))
            .store
            .unwrap();
        let primary = fs::read(store.save_path()).unwrap();
        let backup = fs::read(store.backup_path()).unwrap();
        assert_ne!(primary, backup);
        assert_eq!(run(arguments("write-journey")), Some(AppExit::error()));
        assert_eq!(run(arguments("verify-journey")), Some(AppExit::Success));
        assert_eq!(fs::read(store.save_path()).unwrap(), primary);
        assert_eq!(fs::read(store.backup_path()).unwrap(), backup);
        assert!(scratch.0.join("verification-journey.txt").is_file());
    }
}
