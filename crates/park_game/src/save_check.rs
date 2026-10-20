//! Opt-in cross-process continuation proof using the production save commands.
//! `write` requires a new absolute directory. `verify` never replaces its saves.

use std::{ffi::OsString, fs, io::Write, path::Path};

use bevy::app::AppExit;
use park_sim::{Park, Speed};

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
            .filter(|value| matches!(*value, "write" | "verify"))
            .ok_or("--save-check requires write or verify")?;
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
