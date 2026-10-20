//! Opt-in cross-process continuation proof using the production save commands.
//! `write`/`write-paths` require a new absolute directory. Verification modes
//! never replace their saves. The original write/verify scenario stays stable
//! so older releases can participate in the same cross-version proof.

use std::{ffi::OsString, fs, io::Write, path::Path};

use bevy::app::AppExit;
use park_sim::{ObjectId, ObjectKind, Park, Speed, TilePos};

use crate::{
    ViewState, demonstration_park,
    persistence::{self, SaveAction, SaveSession},
};

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
            .filter(|value| matches!(*value, "write" | "verify" | "write-paths" | "verify-paths"))
            .ok_or("--save-check requires write, verify, write-paths or verify-paths")?;
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
                    name.to_string_lossy()
                        .starts_with("Parkbound path proof test ")
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
}
