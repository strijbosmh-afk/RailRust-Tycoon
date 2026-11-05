//! Fresh-process proof of naturally dirty paths and a travelling handyman.
use super::*;

const FUTURE_TICKS: usize = 3_600;

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let entry = path_at(&park, TilePos { x: 8, y: 16 }).ok_or("public park entry missing")?;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.set_handyman_assigned(true);
    park.set_handyman_assigned(false);
    for _ in 0..24_000 {
        if park.ground_messes().iter().any(|mess| mess.path != entry) {
            break;
        }
        park.advance(Speed::Normal);
    }
    if !park.ground_messes().iter().any(|mess| mess.path != entry) {
        return Err("visitors did not create an off-entry mess".into());
    }
    park.set_handyman_assigned(true);
    for _ in 0..300 {
        let status = park.grounds_status();
        if status.next_path.is_some() && status.walk_progress >= 7 {
            break;
        }
        park.advance(Speed::Normal);
    }
    let status = park.grounds_status();
    if status.next_path.is_none() || status.walk_progress < 7 {
        return Err(format!("handyman did not begin a public route: {status:?}"));
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-grounds-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    if park.grounds_status().cleaned == 0 {
        return Err("handyman did not clean a path".into());
    }
    write_new(
        &store.directory().join("expected-grounds-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Grounds continuation proof written.");
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
    let expected = read_expected(&store.directory().join("expected-grounds-primary.park.json"))?;
    if park != expected
        || view.speed != Speed::Paused
        || park.grounds_status().next_path.is_none()
        || park.grounds_status().walk_progress < 7
    {
        return Err("saved mid-edge handyman route changed on load".into());
    }
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    let expected = read_expected(&store.directory().join("expected-grounds-after.park.json"))?;
    if park != expected || fs::read(store.save_path()).map_err(|e| e.to_string())? != original {
        return Err("grounds replay diverged or rewrote primary slot".into());
    }
    write_new(&store.directory().join("verification-grounds.txt"),
        b"Saved mid-edge handyman cleaned naturally dirty paths after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n")?;
    println!("Grounds continuation verified.");
    Ok(())
}
