//! Player opening commands, atomic refusals and actual paid restart journeys.
use super::*;
use crate::ride_operations::{self, OperationCommand};
use park_sim::RideId;

pub(crate) fn toggle(park: &mut Park, id: RideId) -> Result<(), String> {
    let mut view = ViewState::default();
    if ride_operations::perform_command(park, &mut view, id, OperationCommand::ToggleOpen) {
        Ok(())
    } else {
        Err(view
            .ride_operating_notice
            .unwrap_or("opening command refused".into()))
    }
}

fn refusals(source: &Park) -> Result<(), String> {
    for id in source.all_rides().map(|r| r.id) {
        for exit in [false, true] {
            let mut park = source.clone();
            park.set_ride_open_checked(id, false)
                .map_err(|e| e.to_string())?;
            let ride = park.ride_by_id(id).ok_or("missing proof ride")?;
            let object = if exit { ride.exit } else { ride.entrance }
                .ok_or("missing proof gate")?
                .object;
            park.remove_object(object);
            let before = park.clone();
            let notice = toggle(&mut park, id)
                .err()
                .ok_or("opened with missing gate")?;
            if park != before || !notice.contains(if exit { "exit" } else { "entrance" }) {
                return Err("opening refusal mutated state or hid its reason".into());
            }
            super::ride_choice::bytes(&park)?;
        }
    }
    Ok(())
}

fn replay(park: &mut Park) -> Result<(), String> {
    let ids = park.all_rides().map(|r| r.id).collect::<Vec<_>>();
    for id in &ids {
        toggle(park, *id)?;
    }
    super::ride_choice::seek(
        park,
        |p| {
            ids.iter()
                .all(|id| p.ride_operating_settings_status(*id).is_ok())
        },
        "both opening-proof rides parked and empty",
    )?;
    refusals(park)?;
    let completed = ids
        .iter()
        .map(|id| park.ride_by_id(*id).unwrap().completed)
        .collect::<Vec<_>>();
    for id in &ids {
        toggle(park, *id)?;
    }
    for tick in 0..7200 {
        super::ride_choice::advance_checked(park)?;
        if tick % 600 == 0 {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    if ids
        .iter()
        .zip(completed)
        .any(|(id, count)| park.ride_by_id(*id).unwrap().completed <= count)
    {
        return Err("reopened ride completed no further paid journey".into());
    }
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let (coaster, carousel) = super::operations::install(&mut park)?;
    refusals(&park)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    for id in [coaster, carousel] {
        toggle(&mut park, id)?;
    }
    super::ride_choice::seek(
        &mut park,
        |p| {
            super::ride_choice::paid_running(p, coaster)
                && super::ride_choice::paid_running(p, carousel)
        },
        "both checked-open rides carry paid passengers",
    )?;
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
    )
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
    if park != read_expected(&store.directory().join("expected-primary.park.json"))? {
        return Err("checked-open paid state changed at load".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("opening journey continuation differs".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))? {
        return Err("closed ready backup changed".into());
    }
    refusals(&park)?;
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("opening proof changed source save slots".into());
    }
    write_new(&store.directory().join("verification-opening.txt"),b"Checked opening for both ride families: PASS\nMissing entrance/exit refusals preserve complete state: PASS\nPaid occupied primary / parked ready backup: PASS\nClose, reopen and 7200-tick exact fresh-process continuation: PASS\nOriginal slots unchanged: PASS\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opening_production_modes_preserve_slots_and_replay_both_paid_rides() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound opening proof {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-opening", "verify-opening"] {
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
