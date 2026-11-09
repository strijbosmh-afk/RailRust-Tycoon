//! Paid park entry, deterministic demand and charging-rule continuation.
use super::*;
use park_sim::{ChargingRule, RideId};

const FUTURE_TICKS: usize = 3_600;

fn fixture() -> Result<(Park, RideId), String> {
    let (mut park, carousel, tower) = super::drop_tower::fixture()?;
    park.set_park_entry_price(2_000)
        .map_err(|e| e.to_string())?;
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    if park.visitor_flow_stats().turned_away != 1 || !park.guests().is_empty() {
        return Err("high entry price did not decline the first visitor".into());
    }
    park.set_park_entry_price(300).map_err(|e| e.to_string())?;
    let mut found = false;
    for _ in 0..60_000 {
        if park.ride_by_id(carousel).is_some_and(|r| r.completed > 0)
            && park.visitor_flow_stats().admission_revenue_cents > 0
            && super::drop_tower::paid_drop_tower_running(&park, tower)
        {
            found = true;
            break;
        }
        park.advance(Speed::Normal);
    }
    if !found {
        return Err("paid entry and both ride visits did not mature".into());
    }
    park.set_charging_rule(ChargingRule::FreeRides)
        .map_err(|e| e.to_string())?;
    if park.all_rides().any(|r| r.ticket_price_cents != 0)
        || park.set_ride_ticket_price(tower, 50).is_ok()
        || !super::drop_tower::paid_drop_tower_running(&park, tower)
    {
        return Err("free-ride rule changed an active paid visit or allowed a new fare".into());
    }
    super::visitors::check(&park)?;
    Ok((park, tower))
}

fn replay(park: &mut Park) -> Result<(), String> {
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    if park.visitor_flow_stats().charging_rule != ChargingRule::FreeRides
        || park.visitor_flow_stats().entry_price_cents != 300
        || park.all_rides().any(|r| r.ticket_price_cents != 0)
    {
        return Err("saved charging rule or entry price changed during replay".into());
    }
    super::visitors::check(park)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let (mut park, _) = fixture()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store
            .directory()
            .join("expected-admission-rules-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    replay(&mut park)?;
    write_new(
        &store
            .directory()
            .join("expected-admission-rules-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Park admission and charging-rule continuation proof written.");
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
    let tower = park
        .carousels()
        .iter()
        .find(|c| c.kind == park_sim::RideKind::DropTower)
        .ok_or("missing drop tower")?
        .ride
        .id;
    if view.speed != Speed::Paused
        || park
            != read_expected(
                &store
                    .directory()
                    .join("expected-admission-rules-primary.park.json"),
            )?
        || park.visitor_flow_stats().turned_away == 0
        || park.visitor_flow_stats().admission_revenue_cents == 0
        || !super::drop_tower::paid_drop_tower_running(&park, tower)
    {
        return Err("loaded admission state or paid passenger changed".into());
    }
    replay(&mut park)?;
    if park
        != read_expected(
            &store
                .directory()
                .join("expected-admission-rules-after.park.json"),
        )?
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != original
    {
        return Err("admission replay diverged or rewrote primary slot".into());
    }
    write_new(
        &store.directory().join("verification-admission-rules.txt"),
        b"Paid entry, declined demand, free-ride restrictions and an existing paid passenger resumed after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n",
    )?;
    println!("Park admission and charging-rule continuation verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::visitor_flow::VisitorCommand;

    #[test]
    fn visitor_panel_sets_entry_price_and_cycles_charging_rules() {
        let mut park = super::super::visitors::install().unwrap();
        super::super::visitors::command(&mut park, VisitorCommand::EntryUp, None, None).unwrap();
        assert_eq!(park.visitor_flow_stats().entry_price_cents, 50);
        super::super::visitors::command(&mut park, VisitorCommand::CycleRule, None, None).unwrap();
        assert_eq!(
            park.visitor_flow_stats().charging_rule,
            ChargingRule::FreeEntry
        );
        assert_eq!(park.visitor_flow_stats().entry_price_cents, 0);
        super::super::visitors::command(&mut park, VisitorCommand::CycleRule, None, None).unwrap();
        assert_eq!(
            park.visitor_flow_stats().charging_rule,
            ChargingRule::FreeRides
        );
        assert!(park.all_rides().all(|r| r.ticket_price_cents == 0));
    }
}
