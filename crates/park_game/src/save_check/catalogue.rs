//! Fresh-process verification for the complete 14-ride, 8-facility park.
use super::*;
const FUTURE: usize = 2400;
fn check(park: &Park) -> Result<(), String> {
    if park.all_rides().count() != 14 || park.services().len() != 8 {
        return Err("catalogue entries are missing".into());
    }
    for model in park_sim::RideModel::ALL {
        if !park
            .all_rides()
            .any(|r| park.ride_model(r.id) == Some(model))
        {
            return Err(format!("missing {model}"));
        }
    }
    for kind in park_sim::ServiceKind::ALL {
        if !park.services().iter().any(|s| s.kind == kind) {
            return Err(format!("missing {kind}"));
        }
    }
    park.to_save_bytes().map_err(|e| e.to_string())?;
    Ok(())
}
pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid save directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = park_sim::diagnostics::build_catalogue_showcase()?;
    for _ in 0..3600 {
        park.advance(Speed::Normal);
    }
    check(&park)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store
            .directory()
            .join("expected-catalogue-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE {
        park.advance(Speed::Normal);
    }
    check(&park)?;
    write_new(
        &store.directory().join("expected-catalogue-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Complete catalogue continuation written.");
    Ok(())
}
pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid save directory")?;
    let bytes = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let mut park = Park::new(32, 32);
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    check(&park)?;
    if park
        != read_expected(
            &store
                .directory()
                .join("expected-catalogue-primary.park.json"),
        )?
        || view.speed != Speed::Paused
    {
        return Err("catalogue load changed state".into());
    }
    for _ in 0..FUTURE {
        park.advance(Speed::Normal);
    }
    check(&park)?;
    if park != read_expected(&store.directory().join("expected-catalogue-after.park.json"))? {
        return Err("catalogue future diverged".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != bytes {
        return Err("verification changed the saved park".into());
    }
    write_new(&store.directory().join("verification-catalogue.txt"),b"14 rides and 8 facilities loaded in a fresh process.\nExact 2400-tick continuation and unchanged original slot verified.\nPASS\n")?;
    println!("Complete catalogue continuation verified.");
    Ok(())
}
