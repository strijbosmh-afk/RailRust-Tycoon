//! Authored Meadow Challenge rules, objective completion and exact continuation.
use super::*;
use crate::ui::visitor_flow::VisitorCommand;
use park_sim::{ChargingRule, MEADOW_ARRIVAL_GOAL, MEADOW_CASH_GOAL_CENTS, MEADOW_RIDE_GOAL};

const REPLAY_TICKS: usize = 3_600;

pub(crate) fn fixture() -> Result<Park, String> {
    let mut park = super::visitors::install()?;
    let notice = super::visitors::command(&mut park, VisitorCommand::StartChallenge, None, None)?;
    if !notice.contains("Meadow Challenge started")
        || park.visitor_flow_stats().charging_rule != ChargingRule::FreeEntry
    {
        return Err("challenge button did not lock free entry".into());
    }
    let before = park.clone();
    for (command, expected) in [
        (VisitorCommand::CycleRule, "requires free park entry"),
        (VisitorCommand::RunRadio, "does not allow radio ads"),
        (VisitorCommand::StartChallenge, "already active"),
    ] {
        let notice = super::visitors::command(&mut park, command, None, None)?;
        if !notice.contains(expected) || park != before {
            return Err("challenge restriction changed park state".into());
        }
    }
    park.borrow_loan(100_000).map_err(|e| e.to_string())?;
    park.start_flyer_campaign().map_err(|e| e.to_string())?;
    for _ in 0..8_100 {
        park.advance(Speed::Normal);
    }
    let status = park.scenario_status().ok_or("challenge disappeared")?;
    if status.arrivals != 18
        || status.rides < MEADOW_RIDE_GOAL
        || status.completed_tick.is_some()
        || status.cash_cents < MEADOW_CASH_GOAL_CENTS
        || park.visitor_flow_stats().flyer_ticks_remaining != 900
    {
        return Err(format!(
            "challenge progress differs from schedule: {status:?}"
        ));
    }
    super::visitors::check(&park)?;
    Ok(park)
}

fn replay(park: &mut Park) -> Result<(), String> {
    for _ in 0..REPLAY_TICKS {
        park.advance(Speed::Normal);
    }
    let status = park.scenario_status().ok_or("challenge disappeared")?;
    if status.arrivals < MEADOW_ARRIVAL_GOAL
        || status.rides < MEADOW_RIDE_GOAL
        || status.cash_cents < MEADOW_CASH_GOAL_CENTS
        || status.completed_tick.is_none()
        || park.visitor_flow_stats().flyer_ticks_remaining != 0
    {
        return Err(format!(
            "challenge completion or flyer expiry diverged: {status:?}"
        ));
    }
    super::visitors::check(park)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = fixture()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store
            .directory()
            .join("expected-challenge-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-challenge-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Meadow Challenge continuation proof written.");
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
                    .join("expected-challenge-primary.park.json"),
            )?
        || park
            .scenario_status()
            .is_none_or(|status| status.completed_tick.is_some() || status.arrivals != 18)
    {
        return Err("loaded challenge progress changed".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-challenge-after.park.json"))?
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != original
    {
        return Err("challenge replay diverged or rewrote primary slot".into());
    }
    write_new(&store.directory().join("verification-challenge.txt"), b"Meadow Challenge locked rules, paid flyer expiry and objective completion resumed after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n")?;
    println!("Meadow Challenge continuation verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn challenge_button_restrictions_and_replay() {
        let mut park = fixture().unwrap();
        replay(&mut park).unwrap();
        let saved = park.to_save_bytes().unwrap();
        assert_eq!(Park::from_save_bytes(&saved).unwrap(), park);
    }
}
