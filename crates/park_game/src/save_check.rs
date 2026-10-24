//! Opt-in cross-process continuation proof using the production save commands.
//! All write modes require a new absolute directory. Verification modes
//! never replace the writer's saves. Current-version write/verify checks exact future
//! equality; verify-legacy preserves old fields at load and checks new-rule replay.

use std::{ffi::OsString, fs, io::Write, path::Path};

use bevy::app::AppExit;
use park_sim::{ObjectId, ObjectKind, Park, Speed, Terrain, TerrainHeightError, TilePos};

use crate::{
    ViewState, demonstration_park,
    persistence::{self, SaveAction, SaveSession},
};

mod access;
mod admission;
pub(super) mod construction;
mod design_transfer;
mod designs;
pub(super) mod economy;
pub(super) mod facility_construction;
pub(super) mod feedback;
pub(super) mod finance;
mod journey;
mod legacy;
pub(super) mod operations;
mod queues;
pub(super) mod ride_choice;
pub(super) mod services;
mod slopes;
pub(super) mod track_construction;

pub(super) fn run(arguments: Vec<OsString>) -> Option<AppExit> {
    let index = arguments.iter().position(|value| value == "--save-check")?;
    let result = (|| -> Result<(), String> {
        if arguments
            .iter()
            .filter(|value| *value == "--save-check")
            .count()
            != 1
        {
            return Err("--save-check may be supplied only once".into());
        }
        let mode = arguments
            .get(index + 1)
            .and_then(|value| value.to_str())
            .filter(|value| matches!(*value,
                "write" | "verify" | "write-paths" | "verify-paths" | "write-terrain" | "verify-terrain" | "write-slopes" | "verify-slopes" | "write-queues" | "verify-queues" | "write-admission" | "verify-admission" | "write-journey" | "verify-journey" | "write-economy" | "verify-economy" | "write-feedback" | "verify-feedback" | "verify-legacy" | "write-services" | "verify-services" | "write-finance" | "verify-finance" | "write-rides" | "verify-rides" | "write-construction" | "verify-construction" | "write-facility-construction" | "verify-facility-construction" | "write-track-construction" | "verify-track-construction" | "write-designs" | "verify-designs" | "write-design-transfer" | "verify-design-transfer" | "write-operations" | "verify-operations" | "write-access" | "verify-access"
            ))
            .ok_or("--save-check requires a documented write/verify proof mode or verify-legacy; see RESUME_DEVELOPMENT.md")?;
        if !arguments.iter().any(|value| value == "--save-dir") {
            return Err("--save-check requires an explicit absolute --save-dir".into());
        }
        let session = SaveSession::from_arguments(arguments.clone());
        let store = session.store.as_ref().ok_or("invalid --save-dir")?;
        match mode {
            "write" => {
                // Never reuse a player's save directory for a diagnostic.
                fs::create_dir(store.directory()).map_err(|error| error.to_string())?;
                let mut park = demonstration_park();
                for _ in 0..731 {
                    park.advance(Speed::Normal);
                }
                let mut view = ViewState::default();
                if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Save) {
                    return Err(view.save_notice);
                }
                write_new(
                    &store.directory().join("expected-backup.park.json"),
                    &park.to_save_bytes().map_err(|e| e.to_string())?,
                )?;
                park.set_train_open(false);
                let occupied = park.guests()[0]
                    .current
                    .ok_or("proof guest must occupy a path")?;
                for neighbor in park.path_neighbors(occupied) {
                    park.remove_object(neighbor);
                }
                park.remove_object(occupied);
                if park.crowd_stats().lost == 0 {
                    return Err("proof needs a lost guest".into());
                }
                if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Save) {
                    return Err(view.save_notice);
                }
                continue_park(&mut park)?;
                write_new(
                    &store.directory().join("expected-after.park.json"),
                    &park.to_save_bytes().map_err(|e| e.to_string())?,
                )?;
                println!("Save continuation proof written.");
            }
            "verify" => {
                let mut park = demonstration_park();
                let mut view = ViewState::default();
                if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Load) {
                    return Err(view.save_notice);
                }
                if view.speed != Speed::Paused
                    || park.crowd_stats().lost == 0
                    || park
                        .train()
                        .is_none_or(|train| train.open || train.speed == 0)
                {
                    return Err(
                        "loaded proof must be paused with lost guests and a moving closing train"
                            .into(),
                    );
                }
                continue_park(&mut park)?;
                let expected = read_expected(&store.directory().join("expected-after.park.json"))?;
                if park != expected {
                    return Err("reloaded continuation differs from uninterrupted state".into());
                }
                if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Backup)
                {
                    return Err(view.save_notice);
                }
                let expected_backup =
                    read_expected(&store.directory().join("expected-backup.park.json"))?;
                if park != expected_backup || view.speed != Speed::Paused {
                    return Err("backup differs from the previous valid save".into());
                }
                write_new(
                    &store.directory().join("verification.txt"),
                    format!("Parkbound {}\nFresh-process primary continuation: PASS (3600 ticks + commands)\nExplicit backup restoration: PASS\nLoad starts paused: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes(),
                )?;
                println!("Fresh-process save/load continuation and backup verified.");
            }
            "write-paths" => write_paths(&session)?,
            "verify-paths" => verify_paths(&session)?,
            "write-terrain" => write_terrain(&session)?,
            "verify-terrain" => verify_terrain(&session)?,
            "write-slopes" => slopes::write(&session)?,
            "verify-slopes" => slopes::verify(&session)?,
            "write-queues" => queues::write(&session)?,
            "verify-queues" => queues::verify(&session)?,
            "write-admission" => admission::write(&session)?,
            "verify-admission" => admission::verify(&session)?,
            "write-journey" => journey::write(&session)?,
            "verify-journey" => journey::verify(&session)?,
            "write-economy" => economy::write(&session)?,
            "verify-economy" => economy::verify(&session)?,
            "write-feedback" => feedback::write(&session)?,
            "verify-feedback" => feedback::verify(&session)?,
            "verify-legacy" => legacy::verify(&session)?,
            "write-services" => services::write(&session)?,
            "verify-services" => services::verify(&session)?,
            "write-finance" => finance::write(&session)?,
            "verify-finance" => finance::verify(&session)?,
            "write-rides" => ride_choice::write(&session)?,
            "verify-rides" => ride_choice::verify(&session)?,
            "write-construction" => construction::write(&session)?,
            "verify-construction" => construction::verify(&session)?,
            "write-facility-construction" => facility_construction::write(&session)?,
            "verify-facility-construction" => facility_construction::verify(&session)?,
            "write-track-construction" => track_construction::write(&session)?,
            "verify-track-construction" => track_construction::verify(&session)?,
            "write-designs" => designs::write(&session)?,
            "verify-designs" => designs::verify(&session)?,
            "write-design-transfer" => design_transfer::write(&session)?,
            "verify-design-transfer" => design_transfer::verify(&session)?,
            "write-operations" => operations::write(&session)?,
            "verify-operations" => operations::verify(&session)?,
            "write-access" => access::write(&session)?,
            "verify-access" => access::verify(&session)?,
            _ => unreachable!(),
        }
        Ok(())
    })();
    Some(match result {
        Ok(()) => AppExit::Success,
        Err(error) => {
            eprintln!("Save continuation proof failed: {error}");
            AppExit::error()
        }
    })
}

const PATH_HEIGHT: u8 = 8;
const BRANCH_START: TilePos = TilePos { x: 13, y: 21 };
const BRANCH_BRIDGE: TilePos = TilePos { x: 13, y: 22 };
const BRANCH_END: TilePos = TilePos { x: 14, y: 22 };
const BRANCH_ALTERNATE: TilePos = TilePos { x: 14, y: 21 };

fn write_paths(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    // The new proof has the same no-existing-directory safety rule as `write`.
    fs::create_dir(store.directory()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    for _ in 0..731 {
        park.advance(Speed::Normal);
    }
    for pos in [BRANCH_START, BRANCH_BRIDGE, BRANCH_END] {
        park.place_path(pos, PATH_HEIGHT)
            .map_err(|e| e.to_string())?;
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    let removed_bridge = path_at(&park, BRANCH_BRIDGE).ok_or("missing constructed bridge")?;
    park.remove_object(removed_bridge)
        .ok_or("bridge demolition failed")?;
    let occupied = park.guests()[0]
        .current
        .ok_or("proof guest must occupy a path")?;
    park.remove_object(occupied)
        .ok_or("occupied path demolition failed")?;
    let alternate = park
        .place_path(BRANCH_ALTERNATE, PATH_HEIGHT)
        .map_err(|e| e.to_string())?;
    if alternate.0 <= removed_bridge.0 || alternate.0 <= occupied.0 {
        return Err("construction reused a demolished path identity".into());
    }
    park.set_train_open(false);
    validate_path_primary(&park)?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    continue_paths(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Path construction/demolition continuation proof written.");
    Ok(())
}

fn verify_paths(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    validate_path_primary(&park)?;
    if view.speed != Speed::Paused
        || park != read_expected(&store.directory().join("expected-primary.park.json"))?
    {
        return Err("constructed primary state did not load exactly and paused".into());
    }
    continue_paths(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("constructed path continuation differs from uninterrupted state".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if view.speed != Speed::Paused
        || park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || path_at(&park, BRANCH_BRIDGE).is_none()
        || path_at(&park, BRANCH_ALTERNATE).is_some()
    {
        return Err("path backup differs from the preceding construction state".into());
    }
    write_new(
        &store.directory().join("verification-paths.txt"),
        format!(
            "Parkbound {}\nFresh-process constructed primary equality: PASS\nConstruction/demolition continuation: PASS (3600 ticks + player path commands)\nFresh replacement object identities: PASS\nExplicit construction backup restoration: PASS\nLoad starts paused: PASS\n",
            env!("CARGO_PKG_VERSION")
        ).as_bytes(),
    )?;
    println!("Fresh-process path construction/demolition continuation and backup verified.");
    Ok(())
}

fn path_at(park: &Park, pos: TilePos) -> Option<ObjectId> {
    park.objects().iter().find_map(|object| {
        (object.kind == ObjectKind::Path && object.pos == pos && object.height == PATH_HEIGHT)
            .then_some(object.id)
    })
}

fn validate_path_primary(park: &Park) -> Result<(), String> {
    if [BRANCH_START, BRANCH_END, BRANCH_ALTERNATE]
        .into_iter()
        .any(|pos| path_at(park, pos).is_none())
        || path_at(park, BRANCH_BRIDGE).is_some()
        || park.crowd_stats().lost == 0
        || park
            .guests()
            .first()
            .is_none_or(|guest| guest.current.is_some())
        || park
            .train()
            .is_none_or(|train| train.open || train.speed == 0)
    {
        return Err(
            "path proof requires the built branch, demolished bridge, lost guest and closing train"
                .into(),
        );
    }
    Ok(())
}

fn continue_paths(park: &mut Park) -> Result<(), String> {
    let stranded = park.guests().first().ok_or("proof guest is missing")?;
    let repair = (stranded.anchor, stranded.height);
    for tick in 0..3_600 {
        if tick == 100 {
            park.place_path(repair.0, repair.1)
                .map_err(|e| e.to_string())?;
        }
        if tick == 240 {
            park.place_path(BRANCH_BRIDGE, PATH_HEIGHT)
                .map_err(|e| e.to_string())?;
        }
        if tick == 600 {
            let alternate = path_at(park, BRANCH_ALTERNATE).ok_or("alternate path is missing")?;
            park.remove_object(alternate)
                .ok_or("alternate demolition failed")?;
        }
        if tick == 900 {
            park.place_path(BRANCH_ALTERNATE, PATH_HEIGHT)
                .map_err(|e| e.to_string())?;
        }
        if tick == 1_800 {
            park.set_train_open(true);
        }
        park.advance(Speed::Normal);
    }
    Ok(())
}

const TERRAIN_GRASS: TilePos = TilePos { x: 3, y: 24 };
const TERRAIN_SAND: TilePos = TilePos { x: 24, y: 3 };
const TERRAIN_OVERLAP: TilePos = TilePos { x: 13, y: 16 };

fn write_terrain(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    if park.tile(TERRAIN_GRASS).map(|tile| tile.terrain) != Some(Terrain::Grass)
        || park.tile(TERRAIN_SAND).map(|tile| tile.terrain) != Some(Terrain::Sand)
    {
        return Err("terrain proof requires its original grass and sand materials".into());
    }
    for _ in 0..731 {
        park.advance(Speed::Normal);
    }
    for (pos, height) in [(TERRAIN_GRASS, 1), (TERRAIN_SAND, 1), (TERRAIN_OVERLAP, 7)] {
        terrain_step(&mut park, pos, height)?;
    }
    for pos in [BRANCH_START, BRANCH_BRIDGE] {
        park.place_path(pos, PATH_HEIGHT)
            .map_err(|error| error.to_string())?;
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &park.to_save_bytes().map_err(|error| error.to_string())?,
    )?;
    for (pos, height) in [(TERRAIN_GRASS, 2), (TERRAIN_SAND, 0), (TERRAIN_OVERLAP, 8)] {
        terrain_step(&mut park, pos, height)?;
    }
    reject_terrain_step(&mut park, TERRAIN_OVERLAP, 9)?;
    let bridge = path_at(&park, BRANCH_BRIDGE).ok_or("terrain proof bridge is missing")?;
    park.remove_object(bridge)
        .ok_or("terrain proof bridge demolition failed")?;
    let alternate = park
        .place_path(BRANCH_ALTERNATE, PATH_HEIGHT)
        .map_err(|error| error.to_string())?;
    if alternate.0 <= bridge.0 {
        return Err("terrain proof reused a demolished path identity".into());
    }
    park.set_train_open(false);
    validate_terrain_primary(&park)?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &park.to_save_bytes().map_err(|error| error.to_string())?,
    )?;
    continue_terrain(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &park.to_save_bytes().map_err(|error| error.to_string())?,
    )?;
    println!("Terrain editing continuation proof written.");
    Ok(())
}

fn verify_terrain(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    validate_terrain_primary(&park)?;
    if view.speed != Speed::Paused
        || !view.terrain_custom
        || park != read_expected(&store.directory().join("expected-primary.park.json"))?
    {
        return Err("edited terrain primary did not load exactly, custom and paused".into());
    }
    continue_terrain(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("terrain continuation differs from uninterrupted state".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if view.speed != Speed::Paused
        || !view.terrain_custom
        || park != read_expected(&store.directory().join("expected-backup.park.json"))?
    {
        return Err("terrain backup differs from the previous edited save".into());
    }
    write_new(
        &store.directory().join("verification-terrain.txt"),
        format!(
            "Parkbound {}\nFresh-process edited terrain primary equality: PASS\nTerrain/path continuation: PASS (3600 ticks + single-step terrain edits and path commands)\nPath and buried-track clearance rejection: PASS\nExplicit edited terrain backup restoration: PASS\nLoad starts paused with custom terrain: PASS\n",
            env!("CARGO_PKG_VERSION")
        ).as_bytes(),
    )?;
    println!("Fresh-process terrain editing continuation and backup verified.");
    Ok(())
}

fn validate_terrain_primary(park: &Park) -> Result<(), String> {
    let expected_tiles = [
        (TERRAIN_GRASS, Terrain::Grass, 2),
        (TERRAIN_SAND, Terrain::Sand, 0),
        (TERRAIN_OVERLAP, Terrain::Grass, 8),
    ];
    if expected_tiles.into_iter().any(|(pos, material, height)| {
        park.tile(pos)
            .is_none_or(|tile| tile.terrain != material || tile.height != height)
    }) || path_at(park, BRANCH_START).is_none()
        || path_at(park, BRANCH_BRIDGE).is_some()
        || path_at(park, BRANCH_ALTERNATE).is_none()
        || park
            .train()
            .is_none_or(|train| train.open || train.speed == 0)
    {
        return Err(
            "terrain proof requires edited grass/sand, a demolished bridge and a closing train"
                .into(),
        );
    }
    Ok(())
}

fn terrain_step(park: &mut Park, pos: TilePos, height: u8) -> Result<(), String> {
    let tile = park
        .tile(pos)
        .ok_or("terrain proof tile is outside the park")?;
    if tile.height.abs_diff(height) != 1 || tile.terrain == Terrain::Water {
        return Err("terrain proof edits must be single steps on grass or sand".into());
    }
    park.validate_terrain_height(pos, height)
        .map_err(|error| error.to_string())?;
    park.set_terrain_height(pos, height)
        .map_err(|error| error.to_string())
}

fn reject_terrain_step(park: &mut Park, pos: TilePos, height: u8) -> Result<(), String> {
    if park
        .tile(pos)
        .is_none_or(|tile| tile.height.abs_diff(height) != 1)
    {
        return Err("clearance proof must attempt one terrain step".into());
    }
    let before = park.clone();
    let cache = park.path_cache_stats();
    let preview = park.validate_terrain_height(pos, height);
    let committed = park.set_terrain_height(pos, height);
    if !matches!(preview, Err(TerrainHeightError::ObjectIntersection { .. }))
        || committed != preview
        || *park != before
        || park.path_cache_stats() != cache
    {
        return Err("terrain clearance rejection changed state or disagreed with preview".into());
    }
    Ok(())
}

fn continue_terrain(park: &mut Park) -> Result<(), String> {
    for tick in 0..3_600 {
        match tick {
            100 | 160 | 220 | 280 | 340 | 400 => {
                let height = 7 - ((tick - 100) / 60) as u8;
                terrain_step(park, TERRAIN_OVERLAP, height)?;
            }
            460 => reject_terrain_step(park, TERRAIN_OVERLAP, 1)?,
            520 | 580 | 640 | 700 | 760 | 820 => {
                let height = 3 + ((tick - 520) / 60) as u8;
                terrain_step(park, TERRAIN_OVERLAP, height)?;
            }
            880 => reject_terrain_step(park, TERRAIN_OVERLAP, 9)?,
            150 => terrain_step(park, TERRAIN_GRASS, 3)?,
            300 => terrain_step(park, TERRAIN_SAND, 1)?,
            1_200 => terrain_step(park, TERRAIN_GRASS, 2)?,
            1_500 => terrain_step(park, TERRAIN_SAND, 2)?,
            240 => {
                park.place_path(BRANCH_BRIDGE, PATH_HEIGHT)
                    .map_err(|error| error.to_string())?;
            }
            600 => {
                let alternate =
                    path_at(park, BRANCH_ALTERNATE).ok_or("terrain proof alternate is missing")?;
                park.remove_object(alternate)
                    .ok_or("terrain proof alternate demolition failed")?;
            }
            900 => {
                park.place_path(BRANCH_ALTERNATE, PATH_HEIGHT)
                    .map_err(|error| error.to_string())?;
            }
            1_800 => park.set_train_open(true),
            3_300 => {
                terrain_step(park, TERRAIN_GRASS, 1)?;
                terrain_step(park, TERRAIN_SAND, 1)?;
            }
            _ => {}
        }
        park.advance(Speed::Normal);
    }
    Ok(())
}

fn continue_park(park: &mut Park) -> Result<(), String> {
    for tick in 0..3_600 {
        if tick == 100 {
            park.restore_crowd_paths().map_err(|e| e.to_string())?;
        }
        if tick == 1_800 {
            park.set_train_open(true);
        }
        park.advance(Speed::Normal);
    }
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())
}

fn read_expected(path: &Path) -> Result<Park, String> {
    // Bound the reference snapshot too; never trust a diagnostic file's size.
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(park_sim::MAX_SAVE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Park::from_save_bytes(&bytes).map_err(|e| e.to_string())
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
            if let (Ok(path), Ok(root)) = (
                fs::canonicalize(&self.0),
                fs::canonicalize(std::env::temp_dir()),
            ) && path.parent() == Some(root.as_path())
                && path.file_name().is_some_and(|name| {
                    let name = name.to_string_lossy();
                    name.starts_with("Parkbound path proof test ")
                        || name.starts_with("Parkbound terrain proof test ")
                        || name.starts_with("Parkbound slope proof test ")
                })
            {
                let _ = fs::remove_dir_all(path);
            }
        }
    }

    #[test]
    fn path_modes_verify_real_store_continuation_and_reject_existing_write_directory() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!(
            "Parkbound path proof test é {} {timestamp}",
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
        assert_eq!(run(arguments("write-paths")), Some(AppExit::Success));
        let session = SaveSession::from_arguments(arguments("verify-paths"));
        let store = session.store.as_ref().unwrap();
        let before = store.load().unwrap();
        let backup = store.load_backup().unwrap();
        assert_ne!(before, backup);
        assert_eq!(run(arguments("write-paths")), Some(AppExit::error()));
        assert_eq!(store.load().unwrap(), before);
        assert_eq!(store.load_backup().unwrap(), backup);
        assert_eq!(run(arguments("verify-paths")), Some(AppExit::Success));
        assert_eq!(store.load().unwrap(), before);
        assert_eq!(store.load_backup().unwrap(), backup);
        assert!(scratch.0.join("verification-paths.txt").is_file());
    }

    #[test]
    fn terrain_modes_verify_real_store_custom_terrain_without_replacing_saved_files() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!(
            "Parkbound terrain proof test é {} {timestamp}",
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
        assert_eq!(
            run(vec![
                OsString::from("--save-check"),
                OsString::from("write-terrain")
            ]),
            Some(AppExit::error())
        );
        assert!(!scratch.0.exists());
        assert_eq!(run(arguments("write-terrain")), Some(AppExit::Success));
        let session = SaveSession::from_arguments(arguments("verify-terrain"));
        let store = session.store.as_ref().unwrap();
        let primary = fs::read(store.save_path()).unwrap();
        let backup = fs::read(store.backup_path()).unwrap();
        assert_ne!(primary, backup);
        validate_terrain_primary(&store.load().unwrap()).unwrap();
        assert_eq!(run(arguments("write-terrain")), Some(AppExit::error()));
        assert_eq!(fs::read(store.save_path()).unwrap(), primary);
        assert_eq!(fs::read(store.backup_path()).unwrap(), backup);
        assert_eq!(run(arguments("verify-terrain")), Some(AppExit::Success));
        assert_eq!(fs::read(store.save_path()).unwrap(), primary);
        assert_eq!(fs::read(store.backup_path()).unwrap(), backup);
        assert!(scratch.0.join("verification-terrain.txt").is_file());
    }

    #[test]
    fn slope_modes_preserve_primary_backup_and_continue_real_moving_guests() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!(
            "Parkbound slope proof test é {} {timestamp}",
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
        assert_eq!(run(arguments("write-slopes")), Some(AppExit::Success));
        let session = SaveSession::from_arguments(arguments("verify-slopes"));
        let store = session.store.as_ref().unwrap();
        let primary = fs::read(store.save_path()).unwrap();
        let backup = fs::read(store.backup_path()).unwrap();
        assert_ne!(primary, backup);
        assert_eq!(run(arguments("write-slopes")), Some(AppExit::error()));
        assert_eq!(run(arguments("verify-slopes")), Some(AppExit::Success));
        assert_eq!(fs::read(store.save_path()).unwrap(), primary);
        assert_eq!(fs::read(store.backup_path()).unwrap(), backup);
        assert!(scratch.0.join("verification-slopes.txt").is_file());
    }
}
