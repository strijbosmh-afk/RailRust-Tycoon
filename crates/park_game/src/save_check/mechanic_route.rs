//! Fresh-process proof of a mechanic travelling to a naturally due ride.
use super::*;
use park_sim::RideId;

const SEEK_DUE_TICKS: usize = 120_000;
const FUTURE_TICKS: usize = 3_600;

fn ride_id(park: &Park) -> Result<RideId, String> {
    park.ride()
        .map(|ride| ride.id)
        .ok_or("coaster fixture missing".into())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let id = journey::install(&mut park)?;
    let entry = path_at(&park, TilePos { x: 8, y: 16 }).ok_or("public park entry missing")?;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.set_visitor_arrivals(true).map_err(|e| e.to_string())?;
    park.set_ride_ticket_price(id, 100)
        .map_err(|e| e.to_string())?;
    for _ in 0..SEEK_DUE_TICKS {
        if park
            .ride_maintenance_status(id)
            .is_some_and(|s| s.inspection_due)
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    if !park
        .ride_maintenance_status(id)
        .is_some_and(|s| s.inspection_due)
    {
        return Err(format!(
            "coaster did not reach inspection due within {SEEK_DUE_TICKS} ticks"
        ));
    }
    park.set_ride_open(id, false).map_err(|e| e.to_string())?;
    for _ in 0..1_500 {
        if park.can_service_ride(id) {
            break;
        }
        park.advance(Speed::Normal);
    }
    if !park.can_service_ride(id) {
        return Err("coaster did not empty after closing".into());
    }
    park.set_mechanic_hired(true);
    for _ in 0..200 {
        let worker = park.mechanic_status();
        if worker.target_ride == Some(id) && worker.next_path.is_some() && worker.walk_progress >= 7
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    let worker = park.mechanic_status();
    if worker.target_ride != Some(id)
        || worker.next_path.is_none()
        || worker.walk_progress < 7
        || park
            .ride_maintenance_status(id)
            .is_none_or(|s| s.service_count != 0)
    {
        return Err(format!(
            "mechanic did not start the public route: {worker:?}"
        ));
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-route-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    if park
        .ride_maintenance_status(id)
        .is_none_or(|s| s.service_count != 1 || s.inspection_due)
    {
        return Err("mechanic did not reach and service the ride".into());
    }
    write_new(
        &store.directory().join("expected-route-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Mechanic public-route continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let original = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    let expected = read_expected(&store.directory().join("expected-route-primary.park.json"))?;
    let id = ride_id(&park)?;
    if park != expected
        || view.speed != Speed::Paused
        || park.mechanic_status().target_ride != Some(id)
        || park.mechanic_status().walk_progress < 7
    {
        return Err("saved mid-edge mechanic route changed on load".into());
    }
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    let expected = read_expected(&store.directory().join("expected-route-after.park.json"))?;
    if park != expected || fs::read(store.save_path()).map_err(|e| e.to_string())? != original {
        return Err("mechanic route replay diverged or rewrote primary slot".into());
    }
    write_new(&store.directory().join("verification-mechanic-route.txt"),
        b"Saved mid-edge mechanic reached a naturally due ride after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n")?;
    println!("Mechanic public-route continuation verified.");
    Ok(())
}
