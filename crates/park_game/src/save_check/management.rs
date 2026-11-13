//! Research, stocks, policies and reports survive a fresh-process continuation.
use super::*;
const FUTURE: usize = 9000;
pub(crate) fn fixture() -> Result<Park, String> {
    let mut park = super::park_life::fixture()?;
    park.enable_management();
    park.set_research(
        park_sim::ResearchPriority::Facilities,
        park_sim::ResearchFunding::High,
    )?;
    park.start_promotion(park_sim::Promotion::Family)?;
    Ok(park)
}
fn check(park: &Park) -> Result<(), String> {
    super::summary::check(park)?;
    let status = park.management_status();
    if !status.enabled
        || status.research_spent[5] != 12_000
        || status.promotion_candidates == 0
        || status.promotion_admissions == 0
        || park.management_reports().len() < 10
        || park.facility_stocks().iter().all(|s| s.purchased == 0)
    {
        return Err(
            "Management proof did not exercise research, promotion, stock and history".into(),
        );
    }
    Ok(())
}
pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid save directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = fixture()?;
    for _ in 0..27000 {
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
            .join("expected-management-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE {
        park.advance(Speed::Normal);
    }
    check(&park)?;
    write_new(
        &store
            .directory()
            .join("expected-management-after.park.json"),
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
                .join("expected-management-primary.park.json"),
        )?
    {
        return Err("Management changed on loading".into());
    }
    for _ in 0..FUTURE {
        park.advance(Speed::Normal);
    }
    check(&park)?;
    if park
        != read_expected(
            &store
                .directory()
                .join("expected-management-after.park.json"),
        )?
    {
        return Err("Management continuation diverged".into());
    }
    if bytes != fs::read(store.save_path()).map_err(|e| e.to_string())? {
        return Err("Verification changed the player's save".into());
    }
    write_new(&store.directory().join("verification-management.txt"),b"PASS\n14 rides, 8 facilities, 5 staff roles, groups, shelters and weather.\n36000 ticks including exact 9000-tick fresh-process continuation.\n")?;
    Ok(())
}
