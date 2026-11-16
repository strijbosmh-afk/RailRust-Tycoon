//! Exercise the playable template through production saves and real guest use.
use super::*;

const FUTURE: usize = 18_000;

fn check(park: &Park, require_visits: bool) -> Result<(), String> {
    if park.all_rides().count() != 7 || park.services().len() != 8 {
        return Err("The demo must contain seven rides and eight facilities.".into());
    }
    for ride in park.all_rides() {
        if !park.ride_is_open(ride.id) || park.ride_opening_status(ride.id).is_err() {
            return Err(format!("Demo ride {:?} is unavailable.", ride.id));
        }
        if require_visits && (ride.completed == 0 || ride.revenue_cents == 0) {
            return Err(format!(
                "Demo ride {:?} has no completed paid visit.",
                ride.id
            ));
        }
    }
    for service in park.services() {
        if !service.open
            || park.service_access(service.id).public_path.is_none()
            || park.service_access(service.id).status != park_sim::ServiceAccessStatus::Ready
        {
            return Err(format!(
                "Demo facility {:?} has no usable public entrance.",
                service.id
            ));
        }
    }
    if require_visits && park.services().iter().all(|service| service.served == 0) {
        return Err("No guest has used a demo facility.".into());
    }
    park.to_save_bytes().map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid save directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = park_sim::demo_park::build_demo_park()?;
    check(&park, false)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-demo-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE {
        park.advance(Speed::Normal);
    }
    check(&park, true)?;
    write_new(
        &store.directory().join("expected-demo-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Playable demo written. All seven rides completed paid visits.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid save directory")?;
    let original = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let mut park = Park::new(32, 32);
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    check(&park, false)?;
    if park != read_expected(&store.directory().join("expected-demo-primary.park.json"))?
        || view.speed != Speed::Paused
    {
        return Err("Demo load changed the park or did not pause.".into());
    }
    for _ in 0..FUTURE {
        park.advance(Speed::Normal);
    }
    check(&park, true)?;
    if park != read_expected(&store.directory().join("expected-demo-after.park.json"))? {
        return Err("Demo continuation diverged.".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != original {
        return Err("Demo verification changed the original slot.".into());
    }
    write_new(
        &store.directory().join("verification-demo-park.txt"),
        b"Seven rides completed paid visits. Facilities served guests.\nExact 18000-tick continuation, paused load and unchanged original slot verified.\nPASS\n",
    )?;
    println!("Playable demo fresh-process continuation verified.");
    Ok(())
}
