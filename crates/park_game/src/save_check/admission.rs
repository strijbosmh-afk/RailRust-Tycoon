//! Production continuation with seeking, occupied slopes and queue cancellation.
use super::*;
use park_sim::{
    GuestState, PathDirection, PathSlope, RideAccessStatus, RideId, RideVisitPhase, WALK_TICKS,
};
use std::collections::BTreeSet;

const GATE: TilePos = TilePos { x: 8, y: 11 };
const PUBLIC_ENTRY: TilePos = TilePos { x: 8, y: 16 };
const SEEK_LIMIT: usize = 12_000;

fn owner(park: &Park) -> Result<RideId, String> {
    park.ride()
        .map(|ride| ride.id)
        .ok_or("admission proof needs a ride".into())
}

fn install(park: &mut Park) -> Result<RideId, String> {
    let ride = owner(park)?;
    for (y, height, slope) in [
        (12, 8, PathSlope::Flat),
        (13, 8, PathSlope::South),
        (14, 9, PathSlope::Flat),
        (15, 8, PathSlope::North),
    ] {
        park.place_queue_path(TilePos { x: 8, y }, height, slope, ride)
            .map_err(|error| error.to_string())?;
    }
    park.place_ride_entrance(GATE, 8, PathDirection::North, ride)
        .map_err(|error| error.to_string())?;
    if park.ride_access(ride).status != RideAccessStatus::Ready {
        return Err("admission fixture must connect its four queue tiles".into());
    }
    Ok(ride)
}

/// Ordinary and seeking guests remain public; admitted/returning guests may
/// additionally use their own queue, never a gate or another owner's queue.
pub(super) fn require_roles(park: &Park) -> Result<(), String> {
    let mut identities = BTreeSet::new();
    let mut tickets = BTreeSet::new();
    let mut members = Vec::new();
    let mut reserved = BTreeSet::new();
    for guest in park.guests() {
        if !identities.insert(guest.id) {
            return Err("duplicate guest identity in admission proof".into());
        }
        let admitted = guest.ride_visit.as_ref().filter(|visit| {
            matches!(
                visit.phase,
                RideVisitPhase::Queueing | RideVisitPhase::Leaving
            )
        });
        if let Some(visit) = &guest.ride_visit {
            if park.ride().is_none_or(|ride| ride.id != visit.ride) {
                return Err("guest visit refers to an absent ride".into());
            }
            if visit.phase != RideVisitPhase::Approaching {
                let ticket = visit.ticket.ok_or("admitted guest has no ticket")?;
                if !tickets.insert(ticket) {
                    return Err("duplicate queue ticket".into());
                }
                if admitted.is_some() {
                    members.push((ticket, guest.id));
                }
            } else if visit.ticket.is_some() {
                return Err("approaching guest reserved a queue ticket".into());
            }
            if matches!(
                visit.phase,
                RideVisitPhase::Boarding | RideVisitPhase::Riding | RideVisitPhase::Exiting
            ) && [guest.current, guest.next, guest.destination]
                .iter()
                .any(Option::is_some)
            {
                return Err("passenger retained ordinary walking references".into());
            }
            if let Some(transfer) = visit
                .passenger
                .as_ref()
                .and_then(|passenger| passenger.transfer.as_ref())
                && park
                    .object(transfer.path)
                    .is_some_and(|path| path.queue_owner.is_some())
                && !reserved.insert(transfer.path)
            {
                return Err("passenger transfer duplicated a queue reservation".into());
            }
        }
        for id in [guest.current, guest.next, guest.destination]
            .into_iter()
            .flatten()
        {
            let path = park
                .object(id)
                .ok_or("guest retained a deleted path reference")?;
            if path.kind != ObjectKind::Path
                || path
                    .queue_owner
                    .is_some_and(|ride| admitted.is_none_or(|visit| visit.ride != ride))
            {
                return Err("guest crossed its public/owned-queue role boundary".into());
            }
        }
        // Current and next are reservations, whereas destination may name a
        // farther cell already occupied by a preceding member.
        for id in [guest.current, guest.next].into_iter().flatten() {
            if park
                .object(id)
                .is_some_and(|path| path.queue_owner.is_some())
                && !reserved.insert(id)
            {
                return Err("two guests reserved the same queue tile".into());
            }
        }
    }
    if let Some(ride) = park.ride() {
        members.sort_unstable();
        let expected: Vec<_> = members.into_iter().map(|(_, id)| id).collect();
        let stats = park.ride_queue_stats(ride.id);
        if park.ride_queue_order(ride.id) != expected
            || stats.queueing + stats.leaving != expected.len()
            || stats.queueing > stats.capacity
        {
            return Err("queue membership/order/capacity counters disagree".into());
        }
    }
    Ok(())
}

fn roundtrip(park: &Park) -> Result<Vec<u8>, String> {
    require_roles(park)?;
    let bytes = park.to_save_bytes().map_err(|error| error.to_string())?;
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if json["schema_version"] != 6 || json["content_version"] != 6 {
        return Err("admission proof requires strict schema/content 6/6".into());
    }
    if Park::from_save_bytes(&bytes).map_err(|error| error.to_string())? != *park {
        return Err("admission state changed during save roundtrip".into());
    }
    Ok(bytes)
}

fn seek(park: &mut Park, ready: impl Fn(&Park) -> bool, description: &str) -> Result<(), String> {
    for _ in 0..SEEK_LIMIT {
        if ready(park) {
            return Ok(());
        }
        park.advance(Speed::Normal);
        require_roles(park)?;
    }
    Err(format!(
        "admission proof did not reach {description} within {SEEK_LIMIT} ticks"
    ))
}

fn occupied_ramp(park: &Park) -> bool {
    park.guests().iter().any(|guest| {
        guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
            && guest.state == GuestState::Walking
            && guest.progress > 0
            && guest.progress < WALK_TICKS
            && [guest.current, guest.next].into_iter().flatten().any(|id| {
                park.object(id).is_some_and(|path| {
                    path.queue_owner.is_some() && path.path_slope != PathSlope::Flat
                })
            })
    })
}

fn continue_admission(park: &mut Park) -> Result<(), String> {
    let ride = owner(park)?;
    let before = park.ride_queue_stats(ride);
    let ticks = park.ticks();
    let identities: Vec<_> = park.guests().iter().map(|guest| guest.id).collect();
    let mut removed = None;
    let mut removed_gate = None;
    let mut removed_public = None;
    let mut saw_return = false;
    let mut saw_recovery = false;
    for tick in 0..3_600 {
        let mut changed = false;
        if tick == 120 || tick == 3_000 {
            park.set_train_open(false);
            changed = true;
        }
        if tick == 360 || tick == 3_150 {
            park.set_train_open(true);
            changed = true;
        }
        // Wait within a fixed command window for actual occupied-queue removal.
        if (900..1_200).contains(&tick) && removed.is_none() {
            let occupied = park.guests().iter().find_map(|guest| {
                guest
                    .current
                    .and_then(|id| park.object(id))
                    .filter(|path| path.queue_owner == Some(ride))
                    .copied()
            });
            if let Some(path) = occupied {
                park.remove_object(path.id)
                    .ok_or("occupied queue removal failed")?;
                removed = Some(path);
                changed = true;
            }
        }
        if tick == 1_200 {
            let path =
                removed.ok_or("no occupied queue path appeared before the repair deadline")?;
            let fresh = park
                .place_queue_path(path.pos, path.height, path.path_slope, ride)
                .map_err(|error| error.to_string())?;
            if fresh <= path.id {
                return Err("occupied queue repair reused its object identity".into());
            }
            changed = true;
        }
        if tick == 1_500 {
            let gate = park
                .ride()
                .and_then(|ride| ride.entrance.as_ref())
                .ok_or("gate is missing")?
                .object;
            park.remove_object(gate).ok_or("gate demolition failed")?;
            removed_gate = Some(gate);
            changed = true;
        }
        if tick == 1_800 {
            let fresh = park
                .place_ride_entrance(GATE, 8, PathDirection::North, ride)
                .map_err(|error| error.to_string())?;
            if Some(fresh) <= removed_gate {
                return Err("gate repair reused its object identity".into());
            }
            changed = true;
        }
        if tick == 2_100 {
            let path = park
                .objects()
                .iter()
                .find(|path| {
                    path.pos == PUBLIC_ENTRY
                        && path.kind == ObjectKind::Path
                        && path.queue_owner.is_none()
                })
                .ok_or("public connection missing")?
                .id;
            park.remove_object(path)
                .ok_or("public connection demolition failed")?;
            removed_public = Some(path);
            changed = true;
        }
        if tick == 2_400 {
            let fresh = park
                .place_path(PUBLIC_ENTRY, 8)
                .map_err(|error| error.to_string())?;
            if Some(fresh) <= removed_public {
                return Err("public connection repair reused its object identity".into());
            }
            changed = true;
        }
        if changed {
            roundtrip(park)?;
        }
        park.advance(Speed::Normal);
        require_roles(park)?;
        saw_return |= park.guests().iter().any(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == RideVisitPhase::Leaving)
        });
        saw_recovery |= tick > 2_400 && park.ride_queue_stats(ride).queueing > 0;
        if park
            .guests()
            .iter()
            .map(|guest| guest.id)
            .collect::<Vec<_>>()
            != identities
        {
            return Err("admission continuation lost, reordered or duplicated guests".into());
        }
    }
    let after = park.ride_queue_stats(ride);
    if park.ticks() != ticks + 3_600
        || owner(park)? != ride
        || !saw_return
        || !saw_recovery
        || after.joined <= before.joined
        || after.abandoned <= before.abandoned
        || park.ride_access(ride).status != RideAccessStatus::Ready
    {
        return Err(
            "admission continuation did not demonstrate cancellation and renewed admission".into(),
        );
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
    let ride = install(&mut park)?;
    seek(
        &mut park,
        |park| park.ride_queue_stats(ride).seeking > 0,
        "a public approach",
    )?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &roundtrip(&park)?,
    )?;
    seek(
        &mut park,
        |park| park.ride_queue_stats(ride).queueing >= 2 && occupied_ramp(park),
        "multiple members including mid-ramp motion",
    )?;
    park.set_terrain_height(TilePos { x: 3, y: 24 }, 1)
        .map_err(|error| error.to_string())?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &roundtrip(&park)?,
    )?;
    continue_admission(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &roundtrip(&park)?,
    )?;
    println!("Guest admission and queue continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|error| error.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    view.path_tool
        .set_mode(crate::path_tools::PathTool::BuildQueue);
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || !view.terrain_custom
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
        || !occupied_ramp(&park)
    {
        return Err(
            "admission primary must restore exact occupied-ramp state paused/Custom/Inspect".into(),
        );
    }
    continue_admission(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("admission continuation differs from uninterrupted state".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
        || park.ride_queue_stats(owner(&park)?).seeking == 0
    {
        return Err("admission backup must restore the exact public approach paused".into());
    }
    roundtrip(&park)?;
    if fs::read(store.save_path()).map_err(|error| error.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|error| error.to_string())? != backup
    {
        return Err("admission verification changed primary or backup bytes".into());
    }
    write_new(&store.directory().join("verification-admission.txt"), format!(
        "Parkbound {}\nFresh-process occupied-ramp primary equality: PASS\nContinuation: PASS (3600 complete ticks + close/open and occupied queue/gate/public-path commands)\nTicket order, capacity reservations and role-aware path references: PASS\nCancellation, fresh-ID repair and renewed admission: PASS\nGuest and ride identities preserved: PASS\nExplicit approaching backup, paused/Inspect loading: PASS\nPrimary and backup files unchanged: PASS\nStrict schema/content 6/6: PASS\n", env!("CARGO_PKG_VERSION")
    ).as_bytes())?;
    println!("Fresh-process guest admission continuation and backup verified.");
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
    fn admission_modes_replay_real_occupied_queues_and_preserve_both_slots() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!(
            "Parkbound admission proof é {} {stamp}",
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
        assert_eq!(run(arguments("write-admission")), Some(AppExit::Success));
        let store = SaveSession::from_arguments(arguments("verify-admission"))
            .store
            .unwrap();
        let primary = fs::read(store.save_path()).unwrap();
        let backup = fs::read(store.backup_path()).unwrap();
        assert_ne!(primary, backup);
        assert_eq!(run(arguments("write-admission")), Some(AppExit::error()));
        assert_eq!(run(arguments("verify-admission")), Some(AppExit::Success));
        assert_eq!(fs::read(store.save_path()).unwrap(), primary);
        assert_eq!(fs::read(store.backup_path()).unwrap(), backup);
        assert!(scratch.0.join("verification-admission.txt").is_file());
    }
}
