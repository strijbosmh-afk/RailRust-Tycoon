//! Independent-process continuation of a paid handyman under a saved patrol policy.
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
        return Err("visitors did not naturally fill the patrol bin".into());
    }
    park.set_visitor_arrivals(false)
        .map_err(|e| e.to_string())?;
    park.toggle_handyman_patrol(bin_path)
        .map_err(|e| e.to_string())?;
    if park.toggle_handyman_cleaning() {
        return Err("cleaning duty did not turn off".into());
    }
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
        || status.patrol_paths != 1
        || status.clean_enabled
    {
        return Err(format!(
            "patrol did not retain a mid-edge eligible bin target: {status:?}"
        ));
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-patrol-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    if park.grounds_status().bins_emptied == 0
        || park.grounds_status().wages_paid_cents < 4 * park_sim::HANDYMAN_WAGE_CENTS
    {
        return Err("patrol did not empty the bin and pay wages".into());
    }
    write_new(
        &store.directory().join("expected-patrol-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Handyman patrol continuation proof written.");
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
    let expected = read_expected(&store.directory().join("expected-patrol-primary.park.json"))?;
    if park != expected
        || view.speed != Speed::Paused
        || park.grounds_status().target_bin.is_none()
        || park.grounds_status().clean_enabled
    {
        return Err("saved mid-edge patrol policy changed on load".into());
    }
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    let expected = read_expected(&store.directory().join("expected-patrol-after.park.json"))?;
    if park != expected || fs::read(store.save_path()).map_err(|e| e.to_string())? != original {
        return Err("patrol replay diverged or rewrote primary slot".into());
    }
    write_new(&store.directory().join("verification-patrol.txt"), b"Natural used bin and mid-edge paid handyman patrol resumed after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n")?;
    println!("Handyman patrol continuation verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_runs_with_natural_bin_fill() {
        let (mut park, bin) = fixture().unwrap();
        for _ in 0..24_000 {
            if park
                .ground_bins()
                .iter()
                .any(|item| item.path == bin && item.fill > 0)
            {
                park.set_visitor_arrivals(false).unwrap();
                park.toggle_handyman_patrol(bin).unwrap();
                park.toggle_handyman_cleaning();
                park.set_handyman_assigned(true);
                for _ in 0..600 {
                    let status = park.grounds_status();
                    if status.target_bin == Some(bin)
                        && status.next_path == Some(bin)
                        && status.walk_progress >= 7
                    {
                        let bytes = park.to_save_bytes().unwrap();
                        assert_eq!(Park::from_save_bytes(&bytes).unwrap(), park);
                        return;
                    }
                    park.advance(Speed::Normal);
                }
                panic!("handyman never reached used bin under patrol");
            }
            park.advance(Speed::Normal);
        }
        panic!("natural bin fill");
    }
}
