//! Concurrent radio and flyer advertising, strict finance and cross-process continuation.
use super::*;
use crate::ui::visitor_flow::VisitorCommand;
use park_sim::{FinanceKind, RADIO_CAMPAIGN_COST_CENTS};

const REPLAY_TICKS: usize = 3_600;

fn fixture() -> Result<Park, String> {
    let mut park = Park::new(32, 32);
    let entry = park
        .place_path(TilePos { x: 5, y: 5 }, 0)
        .map_err(|e| e.to_string())?;
    park.place_path(TilePos { x: 6, y: 5 }, 0)
        .map_err(|e| e.to_string())?;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.set_visitor_arrivals(true).map_err(|e| e.to_string())?;
    park.set_park_entry_price(500).map_err(|e| e.to_string())?;
    let notice = super::visitors::command(&mut park, VisitorCommand::RunRadio, None, None)?;
    if !notice.contains("Radio ads started")
        || park.cash_cents() != park_sim::INITIAL_PARK_CASH_CENTS - RADIO_CAMPAIGN_COST_CENTS
        || park.finance_ledger().entries.last().map(|e| e.kind)
            != Some(FinanceKind::RadioAdvertising)
    {
        return Err("radio button did not purchase a recorded campaign".into());
    }
    park.start_flyer_campaign().map_err(|e| e.to_string())?;
    for _ in 0..8_100 {
        park.advance(Speed::Normal);
    }
    let flow = park.visitor_flow_stats();
    if flow.radio_ticks_remaining != 900
        || flow.flyer_ticks_remaining != 900
        || flow.radio_candidates != 18
        || flow.flyer_candidates != 18
        || flow.radio_admissions == 0
        || park.finance_stats().advertising_cents != 6_500
    {
        return Err("overlapping campaign state or finance differs from schedule".into());
    }
    super::visitors::check(&park)?;
    Ok(park)
}

fn replay(park: &mut Park) -> Result<(), String> {
    for _ in 0..REPLAY_TICKS {
        park.advance(Speed::Normal);
    }
    let flow = park.visitor_flow_stats();
    if flow.radio_ticks_remaining != 0
        || flow.flyer_ticks_remaining != 0
        || flow.radio_candidates != 20
        || flow.flyer_candidates != 20
        || flow.spawned + flow.turned_away != 23
        || park.finance_summary().advertising_cents != 6_500
    {
        return Err("campaign expiry, ordinary arrival pace or finance diverged".into());
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
        &store.directory().join("expected-radio-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-radio-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Radio advertising continuation proof written.");
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
        || park != read_expected(&store.directory().join("expected-radio-primary.park.json"))?
        || park.visitor_flow_stats().radio_ticks_remaining != 900
    {
        return Err("loaded radio campaign changed".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-radio-after.park.json"))?
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != original
    {
        return Err("radio replay diverged or rewrote primary slot".into());
    }
    write_new(&store.directory().join("verification-radio.txt"), b"Overlapping paid radio and flyer campaigns, gate decisions, expiry and normal arrivals resumed after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n")?;
    println!("Radio advertising continuation verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn radio_button_and_replay() {
        let mut park = fixture().unwrap();
        let before = park.clone();
        let notice =
            super::super::visitors::command(&mut park, VisitorCommand::RunRadio, None, None)
                .unwrap();
        assert!(notice.contains("already running"));
        assert_eq!(park, before);
        replay(&mut park).unwrap();
    }
}
