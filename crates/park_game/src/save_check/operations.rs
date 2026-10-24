//! Production ride controls, independent owners and exact process continuation.
use super::*;
use crate::ride_operations::{self, OperatingField as Field, OperationCommand as Command};
use park_sim::{RideId, RideOperatingSettings};

const REPLAY_TICKS: u64 = 7_200;

fn command(park: &mut Park, id: RideId, value: Command) -> Result<(), String> {
    let before = park.clone();
    let mut view = ViewState::default();
    if !ride_operations::perform_command(park, &mut view, id, value) {
        return Err(view
            .ride_operating_notice
            .unwrap_or("operating command failed".into()));
    }
    if value != Command::ToggleOpen
        && (park.cash_cents() != before.cash_cents()
            || park.finance_ledger() != before.finance_ledger()
            || park.guests() != before.guests()
            || park.track_draft() != before.track_draft()
            || park
                .all_rides()
                .any(|ride| before.ride_by_id(ride.id) != Some(ride)))
    {
        return Err("operating adjustment changed money, passengers, draft or ride history".into());
    }
    super::ride_choice::bytes(park)?;
    Ok(())
}

fn close_empty(park: &mut Park, ids: [RideId; 2]) -> Result<(), String> {
    for id in ids {
        if park.ride_is_open(id) {
            command(park, id, Command::ToggleOpen)?;
        }
    }
    super::ride_choice::seek(
        park,
        |p| {
            ids.iter()
                .all(|id| p.ride_operating_settings_status(*id).is_ok())
        },
        "both rides closed, stationary and empty",
    )
}

pub(crate) fn install(park: &mut Park) -> Result<(RideId, RideId), String> {
    let (coaster, carousel) = super::ride_choice::install(park)?;
    close_empty(park, [coaster, carousel])?;
    for (id, field, amount) in [
        (coaster, Field::StationWait, 2),
        (coaster, Field::LiftSpeed, 2),
        (coaster, Field::BrakeSpeed, -1),
        (carousel, Field::StationWait, 4),
        (carousel, Field::CycleDuration, 2),
    ] {
        command(park, id, Command::Adjust(field, amount))?;
    }
    if park.ride_operating_settings(coaster)
        != Some(RideOperatingSettings {
            station_wait_ticks: 150,
            lift_speed: 32,
            brake_speed: 16,
            ..RideOperatingSettings::default()
        })
        || park.ride_operating_settings(carousel)
            != Some(RideOperatingSettings {
                station_wait_ticks: 210,
                cycle_ticks: 1080,
                ..RideOperatingSettings::default()
            })
    {
        return Err("visible adjustment increments differ from requested settings".into());
    }
    Ok((coaster, carousel))
}

fn replay(park: &mut Park) -> Result<(), String> {
    let coaster = park.ride().ok_or("missing coaster")?.id;
    let carousel = park.carousels().first().ok_or("missing carousel")?.ride.id;
    for id in [coaster, carousel] {
        let before = park.clone();
        let mut view = ViewState::default();
        if ride_operations::perform_command(
            park,
            &mut view,
            id,
            Command::Adjust(Field::StationWait, 1),
        ) || *park != before
        {
            return Err("occupied settings refusal changed authoritative state".into());
        }
    }
    close_empty(park, [coaster, carousel])?;
    let carousel_settings = park.ride_operating_settings(carousel);
    command(park, coaster, Command::ResetDefaults)?;
    if park.ride_operating_settings(coaster) != Some(RideOperatingSettings::default())
        || park.ride_operating_settings(carousel) != carousel_settings
    {
        return Err("reset did not preserve the other ride's settings".into());
    }
    command(park, coaster, Command::Adjust(Field::StationWait, 3))?;
    command(park, coaster, Command::Adjust(Field::LiftSpeed, -2))?;
    command(park, coaster, Command::Adjust(Field::BrakeSpeed, 2))?;
    command(park, carousel, Command::Adjust(Field::CycleDuration, -3))?;
    for id in [coaster, carousel] {
        command(park, id, Command::ToggleOpen)?;
    }
    let started = park.ticks();
    let completed = [coaster, carousel].map(|id| park.ride_by_id(id).unwrap().completed);
    for tick in 0..REPLAY_TICKS {
        super::ride_choice::advance_checked(park)?;
        if tick % 257 == 0 {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    if park.ticks() != started + REPLAY_TICKS
        || [coaster, carousel]
            .into_iter()
            .zip(completed)
            .any(|(id, before)| park.ride_by_id(id).unwrap().completed <= before)
    {
        return Err("retuned rides did not complete paid passenger journeys".into());
    }
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let (coaster, carousel) = install(&mut park)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    command(&mut park, carousel, Command::Adjust(Field::StationWait, -2))?;
    for id in [coaster, carousel] {
        command(&mut park, id, Command::ToggleOpen)?;
    }
    super::ride_choice::seek(
        &mut park,
        |p| {
            super::ride_choice::paid_running(p, coaster)
                && super::ride_choice::paid_running(p, carousel)
        },
        "both tuned rides carry paid passengers",
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
    )?;
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState {
        rides_open: true,
        selected_ride: Some(RideId(999)),
        ride_operating_commands: vec![Command::ResetDefaults],
        ride_operating_notice: Some("stale".into()),
        ..ViewState::default()
    };
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || view.rides_open
        || view.selected_ride.is_some()
        || !view.ride_operating_commands.is_empty()
        || view.ride_operating_notice.is_some()
    {
        return Err("operating primary or paused/reset load differs".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("tuned continuation differs after process restart".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))? {
        return Err("backup lost its independent operating settings".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("verification changed source slots".into());
    }
    write_new(&store.directory().join("verification-operations.txt"), format!("Parkbound {}\nProduction controls, exact increments and independent settings: PASS\nSettings leave money, guests, draft and ride history unchanged: PASS\nOccupied adjustment refused atomically: PASS\nClose, empty, reset one ride, retune and reopen: PASS\nFresh-process continuation: PASS (bounded close/empty plus {REPLAY_TICKS} ticks, repeated strict16 roundtrips, paid journeys on both rides)\nIndependent exact backup and paused load clearing stale controls: PASS\nOriginal primary and backup unchanged: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operating_proof_roundtrips_real_controls_and_both_occupied_owners() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound operations é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-operations", "verify-operations"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    directory.clone().into_os_string()
                ]),
                Some(AppExit::Success)
            );
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
