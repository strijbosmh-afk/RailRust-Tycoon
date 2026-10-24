//! Production car controls, occupied seats and exact fresh-process continuation.
use super::*;
use crate::ride_operations::{self, OperatingField as Field, OperationCommand as Command};
use park_sim::RideId;

pub(crate) fn install(park: &mut Park, cars: u8) -> Result<RideId, String> {
    let (id, _) = super::operations::install(park)?;
    if cars != 3
        && !ride_operations::perform_command(
            park,
            &mut ViewState::default(),
            id,
            Command::Adjust(Field::Cars, cars as i8 - 3),
        )
    {
        return Err("car-count control failed".into());
    }
    require(park, id, cars)?;
    Ok(id)
}

fn require(park: &Park, id: RideId, cars: u8) -> Result<(), String> {
    if park.coaster_car_count(id) != Some(cars)
        || park.ride_capacity(id) != Some(usize::from(cars * 2))
        || park.train_cars().len() != usize::from(cars)
        || park
            .carousels()
            .iter()
            .any(|c| park.ride_capacity(c.ride.id) != Some(6))
    {
        return Err("car count, visible poses or per-ride capacity disagree".into());
    }
    super::ride_choice::check(park)?;
    for guest in park.guests() {
        if let Some(visit) = guest.ride_visit.as_ref().filter(|v| v.ride == id)
            && visit.passenger.as_ref().is_some_and(|p| p.seat >= cars * 2)
        {
            return Err("passenger owns a seat outside this train".into());
        }
    }
    Ok(())
}

fn replay(park: &mut Park) -> Result<(), String> {
    let id = park.ride().ok_or("missing coaster")?.id;
    let before = park.clone();
    if ride_operations::perform_command(
        park,
        &mut ViewState::default(),
        id,
        Command::Adjust(Field::Cars, 1),
    ) || *park != before
    {
        return Err("occupied train allowed car editing".into());
    }
    let completed = park.ride_by_id(id).unwrap().completed;
    for tick in 0..7_200 {
        super::ride_choice::advance_checked(park)?;
        require(park, id, 1)?;
        if tick % 300 == 0 {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    if park.ride_by_id(id).unwrap().completed <= completed {
        return Err("short train completed no further trips".into());
    }
    park.set_ride_open(id, false).map_err(|e| e.to_string())?;
    super::ride_choice::seek(
        park,
        |p| p.ride_operating_settings_status(id).is_ok(),
        "short train drains before changing cars",
    )?;
    let history = park.ride_by_id(id).unwrap().clone();
    let money = park.finance_ledger().clone();
    if !ride_operations::perform_command(
        park,
        &mut ViewState::default(),
        id,
        Command::Adjust(Field::Cars, 1),
    ) {
        return Err("empty train failed to grow to two cars".into());
    }
    require(park, id, 2)?;
    if park.ride_by_id(id) != Some(&history) || park.finance_ledger() != &money {
        return Err("car adjustment rewrote history or money".into());
    }
    if !ride_operations::perform_command(
        park,
        &mut ViewState::default(),
        id,
        Command::ResetDefaults,
    ) {
        return Err("reset did not restore three cars".into());
    }
    require(park, id, 3)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let id = install(&mut park, 1)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    for owner in park.all_rides().map(|r| r.id).collect::<Vec<_>>() {
        park.set_ride_open(owner, true).map_err(|e| e.to_string())?;
    }
    super::ride_choice::seek(
        &mut park,
        |p| super::ride_choice::paid_running(p, id),
        "one-car train carries paid passengers",
    )?;
    require(&park, id, 1)?;
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
        return Err("occupied short train changed at load".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("short train continuation differs".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))? {
        return Err("parked train backup changed".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("train verification changed source slots".into());
    }
    write_new(&store.directory().join("verification-trains.txt"), b"Car controls and poses: PASS\nOne-car occupied paid primary, parked backup and 7200-tick continuation: PASS\nBoth ride capacities, seats, fares, configuration guards, growth and reset: PASS\nOriginal primary and backup unchanged: PASS\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn train_proof_replays_controls_and_occupied_seats() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound train proof {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-trains", "verify-trains"] {
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
