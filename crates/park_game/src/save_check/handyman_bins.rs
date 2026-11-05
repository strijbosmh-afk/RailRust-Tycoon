//! Fresh-process proof of a naturally used bin and paid handyman route.
use super::*;

const FUTURE_TICKS: usize = 3_600;

fn fixture() -> Result<(Park, ObjectId), String> {
    let mut park = Park::new(32, 32);
    let entry = park
        .place_path(TilePos { x: 5, y: 5 }, 0)
        .map_err(|e| e.to_string())?;
    let bin_path = park
        .place_path(TilePos { x: 6, y: 5 }, 0)
        .map_err(|e| e.to_string())?;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.toggle_ground_bin(bin_path)
        .map_err(|e| e.to_string())?;
    park.set_visitor_arrivals(true).map_err(|e| e.to_string())?;
    Ok((park, bin_path))
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let (mut park, bin_path) = fixture()?;
    for _ in 0..24_000 {
        if park
            .ground_bins()
            .iter()
            .any(|bin| bin.path == bin_path && bin.fill > 0)
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    if !park
        .ground_bins()
        .iter()
        .any(|bin| bin.path == bin_path && bin.fill > 0)
    {
        return Err("visitors did not naturally use the bin".into());
    }
    park.set_visitor_arrivals(false)
        .map_err(|e| e.to_string())?;
    park.set_handyman_assigned(true);
    for _ in 0..600 {
        let status = park.grounds_status();
        if status.target_bin == Some(bin_path)
            && status.next_path == Some(bin_path)
            && status.walk_progress >= 7
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    let status = park.grounds_status();
    if status.target_bin != Some(bin_path)
        || status.next_path != Some(bin_path)
        || status.walk_progress < 7
    {
        return Err(format!("handyman did not walk to the used bin: {status:?}"));
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-bin-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    if park.grounds_status().bins_emptied == 0
        || park.grounds_status().wages_paid_cents < 4 * park_sim::HANDYMAN_WAGE_CENTS
    {
        return Err("bin was not emptied or handyman wages were not paid".into());
    }
    write_new(
        &store.directory().join("expected-bin-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Handyman bin and wage continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let original = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let mut park = Park::new(32, 32);
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    let expected = read_expected(&store.directory().join("expected-bin-primary.park.json"))?;
    if park != expected || view.speed != Speed::Paused || park.grounds_status().target_bin.is_none()
    {
        return Err("saved mid-edge bin route changed on load".into());
    }
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    let expected = read_expected(&store.directory().join("expected-bin-after.park.json"))?;
    if park != expected || fs::read(store.save_path()).map_err(|e| e.to_string())? != original {
        return Err("bin and wage replay diverged or rewrote primary slot".into());
    }
    write_new(&store.directory().join("verification-handyman-bins.txt"),
        b"Natural bin use, mid-edge emptying and paid handyman resumed after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n")?;
    println!("Handyman bin and wage continuation verified.");
    Ok(())
}
