//! Fresh-process continuation across incomplete queue and entrance topology.
use super::*;
use park_sim::{PathDirection, PathSlope, RideAccessStatus, RideId};

const GATE: TilePos = TilePos { x: 8, y: 11 };
const BRIDGE: TilePos = TilePos { x: 8, y: 13 };
const PUBLIC_ENTRY: TilePos = TilePos { x: 8, y: 16 };
const QUEUE: [(TilePos, u8, PathSlope); 4] = [
    (TilePos { x: 8, y: 12 }, 8, PathSlope::Flat),
    (BRIDGE, 8, PathSlope::South),
    (TilePos { x: 8, y: 14 }, 9, PathSlope::Flat),
    (TilePos { x: 8, y: 15 }, 8, PathSlope::North),
];

fn ride_id(park: &Park) -> Result<RideId, String> {
    park.ride()
        .map(|ride| ride.id)
        .ok_or_else(|| "queue proof requires the train ride".into())
}

fn require_status(park: &Park, ride: RideId, expected: RideAccessStatus) -> Result<(), String> {
    if ride_id(park)? != ride {
        return Err("queue editing changed the ride identity".into());
    }
    let actual = park.ride_access(ride).status;
    if actual != expected {
        return Err(format!("queue access must be {expected:?}, got {actual:?}"));
    }
    Ok(())
}

fn require_guest_roles(park: &Park) -> Result<(), String> {
    super::admission::require_roles(park)
}

fn queue_at(park: &Park, pos: TilePos, ride: RideId) -> Result<ObjectId, String> {
    park.objects()
        .iter()
        .find(|object| {
            object.pos == pos && object.kind == ObjectKind::Path && object.queue_owner == Some(ride)
        })
        .map(|object| object.id)
        .ok_or_else(|| "queue proof segment is missing".into())
}

fn save_bytes(park: &Park) -> Result<Vec<u8>, String> {
    let bytes = park.to_save_bytes().map_err(|error| error.to_string())?;
    let envelope: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if envelope["schema_version"] != 16 || envelope["content_version"] != 16 {
        return Err("queue proof requires strict schema/content 15/15".into());
    }
    Ok(bytes)
}

pub(super) fn install(park: &mut Park) -> Result<RideId, String> {
    let ride = ride_id(park)?;
    for (pos, height, slope) in QUEUE {
        park.place_queue_path(pos, height, slope, ride)
            .map_err(|error| error.to_string())?;
    }
    park.place_ride_entrance(GATE, 8, PathDirection::North, ride)
        .map_err(|error| error.to_string())?;
    require_status(park, ride, RideAccessStatus::Ready)?;
    require_guest_roles(park)?;
    Ok(ride)
}

fn require_rejections_unchanged(park: &mut Park, ride: RideId) -> Result<(), String> {
    let before = park.clone();
    let cache = park.path_cache_stats();
    if park
        .place_queue_path(BRIDGE, 8, PathSlope::South, ride)
        .is_ok()
        || park
            .place_queue_path(
                TilePos { x: 3, y: 24 },
                8,
                PathSlope::Flat,
                RideId(u64::MAX),
            )
            .is_ok()
        || park
            .place_ride_entrance(GATE, 8, PathDirection::East, ride)
            .is_ok()
        || *park != before
        || park.path_cache_stats() != cache
    {
        return Err("rejected queue/entrance commands changed authoritative state or cache".into());
    }
    Ok(())
}

fn continue_queues(park: &mut Park) -> Result<(), String> {
    let ride = ride_id(park)?;
    let initial_ticks = park.ticks();
    let mut newest_id = park
        .objects()
        .iter()
        .map(|object| object.id)
        .max()
        .ok_or("queue proof has no objects")?;
    require_status(park, ride, RideAccessStatus::Disconnected)?;
    for tick in 0..3_600 {
        if tick == 120 {
            let replacement = park
                .place_queue_path(BRIDGE, 8, PathSlope::South, ride)
                .map_err(|error| error.to_string())?;
            if replacement <= newest_id {
                return Err("queue replacement reused an object identity".into());
            }
            newest_id = replacement;
            require_status(park, ride, RideAccessStatus::Ready)?;
        }
        if tick == 300 {
            let gate = park
                .ride()
                .and_then(|ride| ride.entrance.as_ref())
                .ok_or("queue proof gate missing")?
                .object;
            park.remove_object(gate).ok_or("gate removal failed")?;
            require_status(park, ride, RideAccessStatus::NoEntrance)?;
        }
        if tick == 450 {
            let replacement = park
                .place_ride_entrance(GATE, 8, PathDirection::North, ride)
                .map_err(|error| error.to_string())?;
            if replacement <= newest_id {
                return Err("entrance replacement reused an object identity".into());
            }
            newest_id = replacement;
            require_status(park, ride, RideAccessStatus::Ready)?;
        }
        if tick == 600 {
            let public = park
                .objects()
                .iter()
                .find(|object| {
                    object.pos == PUBLIC_ENTRY
                        && object.kind == ObjectKind::Path
                        && object.queue_owner.is_none()
                })
                .ok_or("queue proof public connection missing")?
                .id;
            park.remove_object(public)
                .ok_or("public connection removal failed")?;
            require_status(park, ride, RideAccessStatus::NoPublicEntry)?;
        }
        if tick == 900 {
            let replacement = park
                .place_path(PUBLIC_ENTRY, 8)
                .map_err(|error| error.to_string())?;
            if replacement <= newest_id {
                return Err("public connection replacement reused an object identity".into());
            }
            require_status(park, ride, RideAccessStatus::Ready)?;
        }
        if tick == 1_800 {
            park.set_train_open(false);
        }
        if tick == 2_400 {
            park.set_train_open(true);
        }
        park.advance(Speed::Normal);
        require_guest_roles(park)?;
    }
    require_status(park, ride, RideAccessStatus::Ready)?;
    if park.ticks() != initial_ticks + 3_600 {
        return Err("queue continuation did not execute exactly 3600 complete ticks".into());
    }
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    // Refuse any existing folder before touching a player's or previous QA save.
    fs::create_dir(store.directory()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    for _ in 0..731 {
        park.advance(Speed::Normal);
    }
    let ride = install(&mut park)?;
    require_rejections_unchanged(&mut park, ride)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &save_bytes(&park)?,
    )?;
    let bridge = queue_at(&park, BRIDGE, ride)?;
    park.remove_object(bridge)
        .ok_or("queue bridge removal failed")?;
    require_status(&park, ride, RideAccessStatus::Disconnected)?;
    park.set_terrain_height(TilePos { x: 3, y: 24 }, 1)
        .map_err(|error| error.to_string())?;
    park.set_train_open(false);
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &save_bytes(&park)?,
    )?;
    continue_queues(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &save_bytes(&park)?,
    )?;
    println!("Queue and entrance continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary_bytes = fs::read(store.save_path()).map_err(|error| error.to_string())?;
    let backup_bytes = fs::read(store.backup_path()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || !view.terrain_custom
    {
        return Err("queue primary must restore exact incomplete state paused/Custom".into());
    }
    let ride = ride_id(&park)?;
    require_status(&park, ride, RideAccessStatus::Disconnected)?;
    require_guest_roles(&park)?;
    continue_queues(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("queue continuation differs from uninterrupted state".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
    {
        return Err("queue backup did not restore its exact prior state paused".into());
    }
    require_status(&park, ride, RideAccessStatus::Ready)?;
    require_guest_roles(&park)?;
    if fs::read(store.save_path()).map_err(|error| error.to_string())? != primary_bytes
        || fs::read(store.backup_path()).map_err(|error| error.to_string())? != backup_bytes
    {
        return Err("queue verification changed a primary or backup file".into());
    }
    write_new(
        &store.directory().join("verification-queues.txt"),
        format!(
            "Parkbound {}\nFresh-process incomplete queue primary equality: PASS\nQueue/gate/public-connection demolition and fresh-ID repair: PASS\nContinuation: PASS (3600 complete ticks + topology and train commands)\nStable ride identity and role-aware public/owned-queue guest references: PASS\nExplicit connected backup and paused loading: PASS\nPrimary and backup files unchanged by verification: PASS\nStrict schema/content 15/15: PASS\n",
            env!("CARGO_PKG_VERSION")
        )
        .as_bytes(),
    )?;
    println!("Fresh-process queue/entrance continuation and backup verified.");
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
    fn queue_proof_uses_real_slots_preserves_files_and_replays_incomplete_topology() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!(
            "Parkbound queue proof test é {} {stamp}",
            std::process::id()
        )));
        let session = SaveSession::from_arguments([
            OsString::from("--save-dir"),
            scratch.0.as_os_str().to_owned(),
        ]);
        write(&session).unwrap();
        let store = session.store.as_ref().unwrap();
        let primary = fs::read(store.save_path()).unwrap();
        let backup = fs::read(store.backup_path()).unwrap();
        assert_ne!(primary, backup);
        assert!(write(&session).is_err());
        assert_eq!(fs::read(store.save_path()).unwrap(), primary);
        assert_eq!(fs::read(store.backup_path()).unwrap(), backup);
        verify(&session).unwrap();
        assert_eq!(fs::read(store.save_path()).unwrap(), primary);
        assert_eq!(fs::read(store.backup_path()).unwrap(), backup);
        assert!(scratch.0.join("verification-queues.txt").is_file());
    }
}
