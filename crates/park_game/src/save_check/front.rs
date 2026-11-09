//! Fresh-process proof for independent park slots and explicit old-slot import.
use std::fs;

use park_sim::Speed;

use crate::{new_game_park, park_library::ParkLibrary, persistence::SaveSession};

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("save store missing")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut original = new_game_park();
    store.save(&original).map_err(|e| e.to_string())?;
    let original_bytes = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    for _ in 0..100 {
        original.advance(Speed::Normal);
    }
    store.save(&original).map_err(|e| e.to_string())?;
    let backup_bytes = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    if backup_bytes != original_bytes {
        return Err("legacy backup did not rotate".into());
    }
    let primary_bytes = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let library = ParkLibrary::new(store.directory());
    let imported = library.import_legacy(store, false)?;
    library.rename(imported.id, "Imported été")?;
    let backup = library.import_legacy(store, true)?;
    let mut new = new_game_park();
    for _ in 0..300 {
        new.advance(Speed::Normal);
    }
    let created = library.create("Meadow Two", &new)?;
    for _ in 0..90 {
        new.advance(Speed::Normal);
    }
    library.save(created.id, &new)?;
    library.archive(backup.id)?;
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary_bytes
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup_bytes
    {
        return Err("import modified the legacy slot".into());
    }
    super::write_new(
        &store.directory().join("front-expected-primary.json"),
        &primary_bytes,
    )?;
    super::write_new(
        &store.directory().join("front-expected-backup.json"),
        &backup_bytes,
    )?;
    super::write_new(
        &store.directory().join("front-expected-current.json"),
        &new.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Park library proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("save store missing")?;
    let root = store.directory();
    let before_primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let before_backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    if before_primary
        != fs::read(root.join("front-expected-primary.json")).map_err(|e| e.to_string())?
        || before_backup
            != fs::read(root.join("front-expected-backup.json")).map_err(|e| e.to_string())?
    {
        return Err("legacy primary or backup changed".into());
    }
    let library = ParkLibrary::new(root);
    let entries = library.entries()?;
    if entries.len() != 2 || entries[0].name != "Meadow Two" || entries[1].name != "Imported été"
    {
        return Err("fresh-process park list or rename differs".into());
    }
    let current = library.load(entries[0].id, false)?;
    if current.to_save_bytes().map_err(|e| e.to_string())?
        != fs::read(root.join("front-expected-current.json")).map_err(|e| e.to_string())?
    {
        return Err("fresh-process continuation differs".into());
    }
    let old = library.load(entries[0].id, true)?;
    if old.ticks() + 90 != current.ticks() {
        return Err("park backup did not retain prior state".into());
    }
    if library
        .load(entries[1].id, false)?
        .to_save_bytes()
        .map_err(|e| e.to_string())?
        != before_primary
    {
        return Err("legacy import differs from original".into());
    }
    if !root
        .join("parks/deleted-parks/park-0000000000000002/park.park.json")
        .exists()
    {
        return Err("archived park is missing".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != before_primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != before_backup
    {
        return Err("verification changed legacy save".into());
    }
    super::write_new(&root.join("front-verification.txt"),
        b"Fresh-process park list, continuation, backup, rename, archive, and unchanged legacy primary/backup: PASS\n")?;
    println!("Fresh-process park library verified.");
    Ok(())
}
