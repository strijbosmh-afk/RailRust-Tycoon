//! Cross-process proof that park closure drains guests and survives a paused load.
use super::*;

const FUTURE_TICKS: usize = 3_600;

fn closed_fixture() -> Result<Park, String> {
    let (mut park, carousel, tower) = super::drop_tower::fixture()?;
    for _ in 0..60_000 {
        if park.ride_by_id(carousel).is_some_and(|r| r.completed > 0)
            && super::drop_tower::paid_drop_tower_running(&park, tower)
        {
            break;
        }
        park.advance(Speed::Normal);
    }
    if !super::drop_tower::paid_drop_tower_running(&park, tower) {
        return Err("closure fixture needs a paid tower passenger in motion".into());
    }
    let active = park.guests().len();
    park.set_park_open(false);
    let stats = park.visitor_flow_stats();
    if stats.park_open || stats.leaving != active || stats.finishing_visits == 0 {
        return Err("closing did not mark every guest or preserve a live visit".into());
    }
    Ok(park)
}

fn replay(park: &mut Park) -> Result<(), String> {
    let spawned = park.visitor_flow_stats().spawned;
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    let stats = park.visitor_flow_stats();
    if stats.park_open || stats.spawned != spawned || stats.active != 0 || stats.departed == 0 {
        return Err("closed park failed to drain or admitted a new visitor".into());
    }
    super::visitors::check(park)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = closed_fixture()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store
            .directory()
            .join("expected-park-open-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-park-open-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Park closure continuation proof written.");
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
    if view.speed != Speed::Paused
        || park
            != read_expected(
                &store
                    .directory()
                    .join("expected-park-open-primary.park.json"),
            )?
        || park.visitor_flow_stats().finishing_visits == 0
    {
        return Err("closed park loaded with changed state or interrupted visit".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-park-open-after.park.json"))?
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != original
    {
        return Err("closed park replay diverged or rewrote primary slot".into());
    }
    write_new(
        &store.directory().join("verification-park-open.txt"),
        b"Closed park with paid mid-ride passenger drained without new arrivals after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n",
    )?;
    println!("Park closure continuation verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::visitor_flow::VisitorCommand;

    #[test]
    fn visitor_panel_closes_and_reopens_park_without_changing_arrivals() {
        let mut park = super::super::visitors::install().unwrap();
        for _ in 0..900 {
            park.advance(Speed::Normal);
        }
        assert!(!park.guests().is_empty());
        let arrivals = park.visitor_flow_stats().spawned;
        super::super::visitors::command(&mut park, VisitorCommand::TogglePark, None, None).unwrap();
        let closed = park.visitor_flow_stats();
        assert!(!closed.park_open && closed.enabled && closed.leaving == closed.active);
        for _ in 0..900 {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.visitor_flow_stats().spawned, arrivals);
        super::super::visitors::command(&mut park, VisitorCommand::TogglePark, None, None).unwrap();
        assert!(park.visitor_flow_stats().park_open);
        assert!(park.visitor_flow_stats().enabled);
    }
}
