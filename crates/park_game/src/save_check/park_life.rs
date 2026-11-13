//! Mixed-park staff, weather and visitor continuation through normal saves.
use super::*;
const FUTURE: usize = 9000;
pub(crate) fn fixture() -> Result<Park, String> {
    let mut park = park_sim::diagnostics::build_catalogue_showcase()?;
    park.enable_park_life();
    for role in park_sim::StaffRole::ALL {
        park.hire_park_staff(role)?;
    }
    let paths: Vec<_> = park
        .objects()
        .iter()
        .filter(|p| p.is_public_path())
        .step_by(12)
        .take(12)
        .map(|p| p.id)
        .collect();
    for path in paths {
        park.toggle_path_shelter(path)?;
    }
    Ok(park)
}
fn check(park: &Park) -> Result<(), String> {
    if !park.park_life_status().enabled
        || park.park_staff().len() != 5
        || park.path_shelters().is_empty()
        || park.park_life_status().parties == 0
    {
        return Err("Park life systems missing from mixed park".into());
    }
    if park.park_staff().iter().all(|s| s.completed == 0) {
        return Err("Staff did not complete any work".into());
    }
    if !park
        .park_staff()
        .iter()
        .any(|s| s.role == park_sim::StaffRole::Attendant && s.completed > 0)
    {
        return Err("Service attendant did not assist normal visitors".into());
    }
    park.to_save_bytes().map_err(|e| e.to_string())?;
    Ok(())
}
pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid save directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = fixture()?;
    for _ in 0..18000 {
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
            .join("expected-park-life-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE {
        park.advance(Speed::Normal);
    }
    check(&park)?;
    write_new(
        &store.directory().join("expected-park-life-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
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
    if park
        != read_expected(
            &store
                .directory()
                .join("expected-park-life-primary.park.json"),
        )?
    {
        return Err("Park life changed on loading".into());
    }
    for _ in 0..FUTURE {
        park.advance(Speed::Normal);
    }
    check(&park)?;
    if park != read_expected(&store.directory().join("expected-park-life-after.park.json"))? {
        return Err("Park life continuation diverged".into());
    }
    if bytes != fs::read(store.save_path()).map_err(|e| e.to_string())? {
        return Err("Verification changed the player's save".into());
    }
    write_new(&store.directory().join("verification-park-life.txt"),b"PASS\n14 rides, 8 facilities, 5 staff roles, groups, shelters and weather.\n27000 ticks including exact 9000-tick fresh-process continuation.\n")?;
    Ok(())
}
