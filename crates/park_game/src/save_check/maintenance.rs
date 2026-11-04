//! Fresh-process proof of the visible service action and exact future replay.
use super::*;

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let id = park.ride().ok_or("coaster fixture missing")?.id;
    for _ in 0..720 {
        park.advance(Speed::Normal);
    }
    park.set_ride_open(id, false).map_err(|e| e.to_string())?;
    for _ in 0..1_200 {
        if park.can_service_ride(id) {
            break;
        }
        park.advance(Speed::Normal);
    }
    if !park.can_service_ride(id) {
        return Err("ride did not finish its trip".into());
    }
    let cash = park.cash_cents();
    let ledger = park.finance_ledger().clone();
    park.service_ride(id).map_err(|e| e.to_string())?;
    if park
        .ride_maintenance_status(id)
        .is_none_or(|s| s.service_count != 1 || s.visits_since_service != 0)
        || park.cash_cents() != cash
        || *park.finance_ledger() != ledger
    {
        return Err("service changed finance or failed to reset wear".into());
    }
    park.set_mechanic_hired(true);
    for _ in 0..park_sim::MECHANIC_WAGE_TICKS {
        park.advance(Speed::Normal);
    }
    if !park.mechanic_status().hired
        || park.mechanic_status().wages_paid_cents != park_sim::MECHANIC_WAGE_CENTS
        || park.mechanic_status().wage_progress != 0
    {
        return Err("mechanic wage was not recorded at the fixed interval".into());
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store
            .directory()
            .join("expected-maintenance-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..3_600 {
        park.advance(Speed::Normal);
    }
    write_new(
        &store
            .directory()
            .join("expected-maintenance-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Ride maintenance continuation proof written.");
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
    let expected = read_expected(
        &store
            .directory()
            .join("expected-maintenance-primary.park.json"),
    )?;
    if park != expected || view.speed != Speed::Paused {
        return Err("maintenance state changed on load".into());
    }
    let id = park.ride().ok_or("loaded coaster missing")?.id;
    if park
        .ride_maintenance_status(id)
        .is_none_or(|s| s.service_count != 1 || s.visits_since_service != 0)
        || !park.mechanic_status().hired
        || park.mechanic_status().wages_paid_cents != park_sim::MECHANIC_WAGE_CENTS
    {
        return Err("service count or trip baseline was not saved".into());
    }
    for _ in 0..3_600 {
        park.advance(Speed::Normal);
    }
    let expected = read_expected(
        &store
            .directory()
            .join("expected-maintenance-after.park.json"),
    )?;
    if park != expected || fs::read(store.save_path()).map_err(|e| e.to_string())? != original {
        return Err("maintenance restart diverged or changed the primary slot".into());
    }
    write_new(&store.directory().join("verification-maintenance.txt"), b"Saved ride service, paid mechanic and exact 3600-tick restart: PASS\nOriginal primary slot unchanged: PASS\n")?;
    println!("Ride maintenance continuation verified.");
    Ok(())
}
