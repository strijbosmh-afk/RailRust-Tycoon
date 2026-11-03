//! Natural rest use, preserved guest wellness, and exact fresh-process replay.
use super::*;
use park_sim::{GuestId, PathDirection, ServiceId, ServiceKind, ServiceVisitPhase};

const REST_SITE: TilePos = TilePos { x: 12, y: 21 };
const REPLAY_TICKS: usize = 3_600;

fn rest_user(park: &Park, rest: ServiceId) -> Option<GuestId> {
    park.guests().iter().find_map(|guest| {
        guest.service_visit.as_ref().and_then(|visit| {
            (visit.service == rest && visit.phase == ServiceVisitPhase::Using && visit.progress > 0)
                .then_some(guest.id)
        })
    })
}

fn continue_from_use(park: &mut Park, rest: ServiceId, id: GuestId) -> Result<(), String> {
    let served_before = park.service(rest).ok_or("Rest facility vanished")?.served;
    let mut relieved = false;
    for _ in 0..REPLAY_TICKS {
        park.advance(Speed::Normal);
        if !relieved && park.service(rest).ok_or("Rest facility vanished")?.served > served_before {
            let guest = park
                .guests()
                .iter()
                .find(|guest| guest.id == id)
                .ok_or("Rest visitor vanished")?;
            if guest.needs.energy < 90
                || guest.needs.nausea > 10
                || guest.thought != park_sim::GuestThought::Rested
            {
                return Err("Rest completion failed to relieve both wellness needs".into());
            }
            relieved = true;
        }
    }
    if !relieved {
        return Err("No real Rest completion during replay".into());
    }
    let bytes = park.to_save_bytes().map_err(|e| e.to_string())?;
    if Park::from_save_bytes(&bytes).map_err(|e| e.to_string())? != *park {
        return Err("Wellness state changed during current-schema roundtrip".into());
    }
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Invalid save directory")?;
    let mut park = demonstration_park();
    let rest = park
        .place_service(REST_SITE, 8, PathDirection::North, ServiceKind::Rest)
        .map_err(|e| e.to_string())?;
    park.set_service_price(rest, 50)
        .map_err(|e| e.to_string())?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    let mut visitor = None;
    for _ in 0..36_000 {
        park.advance(Speed::Normal);
        if let Some(id) = rest_user(&park, rest) {
            visitor = Some(id);
            break;
        }
    }
    let id = visitor.ok_or("No visitor naturally sought the Rest facility")?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    continue_from_use(&mut park, rest, id)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Natural Rest use and wellness continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Invalid save directory")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState {
        guests_open: true,
        services_open: true,
        ..Default::default()
    };
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || view.guests_open
        || view.services_open
    {
        return Err("Rest primary or transient view differed after restart".into());
    }
    let rest = park
        .services()
        .iter()
        .find(|service| service.kind == ServiceKind::Rest)
        .ok_or("Loaded Rest facility missing")?
        .id;
    let id = rest_user(&park, rest).ok_or("Loaded Rest visitor missing")?;
    continue_from_use(&mut park, rest, id)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("Rest fresh-process continuation diverged".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup)
        || park != read_expected(&store.directory().join("expected-backup.park.json"))?
    {
        return Err("Rest backup restoration differed".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("Rest verifier changed the source slots".into());
    }
    write_new(
        &store.directory().join("verification-wellness.txt"),
        b"PASS: natural Rest use, energy and nausea relief, exact 3600-tick replay, primary/backup restore, unchanged source slots.\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wellness_survives_primary_backup_and_exact_restart_replay() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound wellness {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-wellness", "verify-wellness"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    directory.clone().into_os_string(),
                ]),
                Some(AppExit::Success)
            );
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
