//! Fresh-process persistence for independently constructed and moving coasters.
use super::*;

fn advance(park: &mut Park) {
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid save directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = Park::new(64, 64);
    let mut view = ViewState::default();
    crate::capture::coaster_scenes::prepare_completion(&mut park, &mut view, false, false);
    view.capture_active = false;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    advance(&mut park);
    write_new(
        &store.directory().join("expected-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid save directory")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let mut park = Park::new(32, 32);
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))? {
        return Err("independent coaster state changed at load".into());
    }
    advance(&mut park);
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("independent coaster continuation differs".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary {
        return Err("coaster verification changed its save slot".into());
    }
    write_new(&store.directory().join("verification-coasters.txt"), b"Two paid coasters, a three-train fleet, blocks, inversions, transitions, guests, fares, names and paints persisted.\n900-tick fresh-process continuation matched.\nSource save slot preserved.\n")
}
