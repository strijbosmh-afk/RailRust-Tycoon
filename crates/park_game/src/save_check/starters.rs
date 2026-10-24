//! Real file imports/exports, paid starter construction and occupied continuation.
use super::*;
use crate::{
    design_library::DesignLibrary,
    design_store::DesignStore,
    design_transfer::{self, TransferRequest},
    starter_designs,
    track_tools::{self, TrackCommand},
};
use park_sim::{
    PathDirection, PathSlope, RideAccessBuildKind, RideAccessStatus, RideExitStatus, RideId,
};

pub(crate) fn install(index: usize) -> Result<Park, String> {
    let design = starter_designs::designs()
        .map_err(|e| e.to_string())?
        .remove(index);
    let mut park = Park::new(32, 32);
    park.apply_terrain_profile(park_sim::TerrainProfile::Terraced);
    park.install_crowd_fixture(42).map_err(|e| e.to_string())?;
    let mut view = ViewState::default();
    let cash = park.cash_cents();
    track_tools::load_design_draft(&mut park, &mut view, design.pieces())
        .map_err(|e| e.to_string())?;
    if park.cash_cents() != cash {
        return Err("loading a starter charged money".into());
    }
    let quote = park.track_construction_quote().map_err(|e| e.to_string())?;
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Apply);
    if park.circuit().is_none() || park.cash_cents() != cash - quote.construction_cents {
        return Err("starter Apply did not settle its actual quote".into());
    }
    let id = park.ride().ok_or("starter has no ride")?.id;
    park.set_coaster_car_count(id, index as u8 + 1)
        .map_err(|e| e.to_string())?;
    park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, id)
        .map_err(|e| e.to_string())?;
    for y in 12..=15 {
        park.place_queue_path(TilePos { x: 8, y }, 8, PathSlope::Flat, id)
            .map_err(|e| e.to_string())?;
    }
    park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, id)
        .map_err(|e| e.to_string())?;
    for _ in 0..32 {
        let plan = park.ride_access_plan(id).map_err(|e| e.to_string())?;
        if plan.exit.status == RideExitStatus::Ready {
            break;
        }
        let next = plan
            .suggestions
            .iter()
            .find(|s| s.kind == RideAccessBuildKind::PublicPath)
            .ok_or("starter has no suggested return path")?;
        park.place_sloped_path(next.pos, next.height, next.slope)
            .map_err(|e| e.to_string())?;
    }
    if park.ride_exit_access(id).status != RideExitStatus::Ready
        || park.ride_access(id).status != RideAccessStatus::Ready
    {
        return Err("starter gates/queue/return route are incomplete".into());
    }
    park.set_ride_ticket_price(id, 100)
        .map_err(|e| e.to_string())?;
    super::ride_choice::bytes(&park)?;
    Ok(park)
}

pub(crate) fn open_paid(park: &mut Park) -> Result<RideId, String> {
    let id = park.ride().ok_or("missing starter")?.id;
    park.set_ride_open(id, true).map_err(|e| e.to_string())?;
    super::ride_choice::seek(
        park,
        |p| super::ride_choice::paid_running(p, id),
        "starter has paid passengers",
    )?;
    Ok(id)
}

fn replay(park: &mut Park) -> Result<(), String> {
    let id = park.ride().ok_or("missing starter")?.id;
    let before = park.ride_by_id(id).unwrap().completed;
    for tick in 0..3600 {
        super::ride_choice::advance_checked(park)?;
        if tick % 300 == 0 {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    if park.ride_by_id(id).unwrap().completed <= before {
        return Err("starter completed no further journeys".into());
    }
    Ok(())
}

fn profile(root: &Path, index: usize) -> std::path::PathBuf {
    if index == 0 {
        root.to_path_buf()
    } else {
        root.join(if index == 1 { "winder" } else { "climb" })
    }
}
fn session_for(path: &Path) -> SaveSession {
    SaveSession::from_arguments(["--save-dir".into(), path.as_os_str().to_owned()])
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let root = session
        .store
        .as_ref()
        .ok_or("invalid --save-dir")?
        .directory();
    fs::create_dir(root).map_err(|e| e.to_string())?;
    let inputs = root.join("starter-inputs");
    fs::create_dir(&inputs).map_err(|e| e.to_string())?;
    let outputs = root.join("starter-exports");
    fs::create_dir(&outputs).map_err(|e| e.to_string())?;
    let catalog = DesignStore::new(root.to_path_buf());
    let mut library = DesignLibrary::default();
    for (file, json) in starter_designs::FILES {
        let path = inputs.join(file);
        write_new(&path, json.as_bytes())?;
        if !design_transfer::complete_transfer(
            &catalog,
            &mut library,
            TransferRequest::Import,
            Ok(Some(path)),
        ) {
            return Err(library.notice);
        }
        let original = park_sim::TrackDesign::from_json(json).map_err(|e| e.to_string())?;
        let design = catalog.load(original.name()).map_err(|e| e.to_string())?;
        let output = outputs.join(file);
        if !design_transfer::complete_transfer(
            &catalog,
            &mut library,
            TransferRequest::Export(design),
            Ok(Some(output.clone())),
        ) {
            return Err(library.notice);
        }
        if fs::read(output).map_err(|e| e.to_string())? != json.as_bytes() {
            return Err("starter export changed canonical content".into());
        }
    }
    for index in 0..3 {
        let dir = profile(root, index);
        if index != 0 {
            fs::create_dir(&dir).map_err(|e| e.to_string())?;
        }
        let session = session_for(&dir);
        let mut park = install(index)?;
        let mut view = ViewState::default();
        if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Save) {
            return Err(view.save_notice);
        }
        write_new(
            &dir.join("expected-backup.park.json"),
            &super::ride_choice::bytes(&park)?,
        )?;
        open_paid(&mut park)?;
        if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Save) {
            return Err(view.save_notice);
        }
        replay(&mut park)?;
        write_new(
            &dir.join("expected-after.park.json"),
            &super::ride_choice::bytes(&park)?,
        )?;
    }
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let root = session
        .store
        .as_ref()
        .ok_or("invalid --save-dir")?
        .directory();
    let catalog = DesignStore::new(root.to_path_buf());
    let catalog_before = fs::read(catalog.catalog_path()).map_err(|e| e.to_string())?;
    if catalog.list().map_err(|e| e.to_string())?.len() != 3 {
        return Err("starter catalog did not survive restart".into());
    }
    for (file, json) in starter_designs::FILES {
        let original = park_sim::TrackDesign::from_json(json).map_err(|e| e.to_string())?;
        if catalog.load(original.name()).map_err(|e| e.to_string())? != original
            || fs::read(root.join("starter-exports").join(file)).map_err(|e| e.to_string())?
                != json.as_bytes()
        {
            return Err("starter import/export changed at restart".into());
        }
    }
    for index in 0..3 {
        let dir = profile(root, index);
        let session = session_for(&dir);
        let store = session.store.as_ref().unwrap();
        let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
        let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
        let mut park = Park::new(32, 32);
        let mut view = ViewState::default();
        if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Load) {
            return Err(view.save_notice);
        }
        let id = park.ride().ok_or("missing imported starter ride")?.id;
        if !super::ride_choice::paid_running(&park, id)
            || park.coaster_car_count(id) != Some(index as u8 + 1)
        {
            return Err("starter primary lost paid passenger/configuration state".into());
        }
        replay(&mut park)?;
        if park != read_expected(&dir.join("expected-after.park.json"))? {
            return Err("starter future differs after restart".into());
        }
        if !persistence::perform_action(&session, &mut park, &mut view, SaveAction::Backup) {
            return Err(view.save_notice);
        }
        if park != read_expected(&dir.join("expected-backup.park.json"))? {
            return Err("starter parked backup changed".into());
        }
        if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
            || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
        {
            return Err("starter verification changed source slots".into());
        }
    }
    if fs::read(catalog.catalog_path()).map_err(|e| e.to_string())? != catalog_before {
        return Err("starter verification changed catalog".into());
    }
    write_new(&root.join("verification-starters.txt"),b"Three canonical starter files imported/exported through production transfer: PASS\nAll three paid layouts, derived exit bridges, car counts and occupied primary/parked backup continue across restart: PASS\n3600 ticks per layout with strict roundtrips and completed journeys: PASS\nOriginal slots, exports and catalog unchanged: PASS\nNative picker gestures: NOT CLAIMED\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starter_content_transfers_builds_and_continues_from_real_slots() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound starter proof {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-starters", "verify-starters"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    dir.clone().into_os_string()
                ]),
                Some(AppExit::Success)
            );
        }
        assert!(dir.starts_with(std::env::temp_dir()));
        let _ = fs::remove_dir_all(dir);
    }
}
