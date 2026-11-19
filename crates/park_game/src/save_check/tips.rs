//! Read-only verification of the three-use allowance across a real restart.
use crate::{new_game_park, park_library::ParkLibrary, persistence::SaveSession};

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let root = session
        .store
        .as_ref()
        .ok_or("save store missing")?
        .directory();
    std::fs::create_dir(root).map_err(|e| e.to_string())?;
    let library = ParkLibrary::new(root);
    let mut park = new_game_park();
    let entry = library.create("Three uses", &park)?;
    let independent = library.create("New park", &new_game_park())?;
    for expected in 1..=park_sim::MAX_TIP_USES {
        let used = library.record_tip_use(entry.id, park.tips_used())?;
        if used != expected {
            return Err("Tip allowance did not increment exactly once".into());
        }
        park.retain_tip_uses(used).map_err(str::to_owned)?;
        library.save(entry.id, &park)?;
    }
    let copy = library.create("Copy of used park", &park)?;
    let ids =
        serde_json::to_vec(&[entry.id, independent.id, copy.id]).map_err(|e| e.to_string())?;
    super::write_new(&root.join("tip-ids.json"), &ids)?;
    println!("Three tip uses recorded and saved.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let root = session
        .store
        .as_ref()
        .ok_or("save store missing")?
        .directory();
    let ids: [u64; 3] = serde_json::from_slice(
        &std::fs::read(root.join("tip-ids.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let library = ParkLibrary::new(root);
    let snapshots = snapshot(root)?;
    if library.load(ids[0], false)?.tips_used() != 3
        || library.load(ids[0], true)?.tips_used() != 3
        || library.load(ids[1], false)?.tips_used() != 0
        || library.load(ids[2], false)?.tips_used() != 3
    {
        return Err("Tip allowance differs after restart, backup load or copy".into());
    }
    if library.record_tip_use(ids[0], 0).is_ok() || library.record_tip_use(ids[2], 3).is_ok() {
        return Err("A fourth use was accepted".into());
    }
    if snapshots != snapshot(root)? {
        return Err("Rejected requests changed stored park files".into());
    }
    println!("Restart, backup and copy retain the allowance; fourth use rejected.");
    Ok(())
}

fn snapshot(root: &std::path::Path) -> Result<Vec<(std::path::PathBuf, Vec<u8>)>, String> {
    let mut result = Vec::new();
    for id in 1..=3 {
        for name in ["slot.json", "park.park.json", "park.park.json.bak"] {
            let path = root.join(format!("parks/park-{id:016x}")).join(name);
            if path.exists() {
                result.push((
                    path.clone(),
                    std::fs::read(path).map_err(|e| e.to_string())?,
                ));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allowance_is_durable_and_independent_for_new_parks() {
        let root = std::env::temp_dir().join(format!(
            "parkbound-tips-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let session =
            SaveSession::from_arguments(["--save-dir".into(), root.clone().into_os_string()]);
        write(&session).unwrap();
        verify(&session).unwrap();
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        assert!(
            root.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("parkbound-tips-")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
