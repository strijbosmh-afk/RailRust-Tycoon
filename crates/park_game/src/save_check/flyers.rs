//! Paid local flyers, accelerated demand and exact save continuation through expiry.
use super::*;
use crate::ui::visitor_flow::VisitorCommand;
use park_sim::{FLYER_CAMPAIGN_COST_CENTS, FinanceKind};

const FUTURE_TICKS: usize = 3_600;

fn fixture() -> Result<Park, String> {
    let mut park = Park::new(32, 32);
    let entry = park
        .place_path(TilePos { x: 5, y: 5 }, 0)
        .map_err(|e| e.to_string())?;
    park.place_path(TilePos { x: 6, y: 5 }, 0)
        .map_err(|e| e.to_string())?;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.set_visitor_arrivals(true).map_err(|e| e.to_string())?;
    park.set_park_entry_price(300).map_err(|e| e.to_string())?;
    let notice = super::visitors::command(&mut park, VisitorCommand::RunFlyers, None, None)?;
    if !notice.contains("Local flyers started")
        || park.cash_cents() != park_sim::INITIAL_PARK_CASH_CENTS - FLYER_CAMPAIGN_COST_CENTS
        || park.finance_ledger().entries.last().map(|entry| entry.kind)
            != Some(FinanceKind::Advertising)
    {
        return Err("flyer button did not buy one recorded campaign".into());
    }
    for _ in 0..8_100 {
        park.advance(Speed::Normal);
    }
    let flow = park.visitor_flow_stats();
    if flow.flyer_ticks_remaining != 900
        || flow.flyer_candidates != 18
        || flow.spawned != 18
        || flow.admission_revenue_cents != 5_400
        || park.finance_stats().advertising_cents != FLYER_CAMPAIGN_COST_CENTS
    {
        return Err("active flyer campaign, admission or cost differs from schedule".into());
    }
    super::visitors::check(&park)?;
    Ok(park)
}

fn replay(park: &mut Park) -> Result<(), String> {
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    let flow = park.visitor_flow_stats();
    if flow.flyer_ticks_remaining != 0
        || flow.flyer_campaigns != 1
        || flow.flyer_candidates != 20
        || flow.spawned != 23
        || flow.admission_revenue_cents != 6_900
        || park.finance_summary().advertising_cents != FLYER_CAMPAIGN_COST_CENTS
    {
        return Err("flyer expiry, normal arrivals or income diverged".into());
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
        &store.directory().join("expected-flyers-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-flyers-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Local flyer continuation proof written.");
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
        || park != read_expected(&store.directory().join("expected-flyers-primary.park.json"))?
        || park.visitor_flow_stats().flyer_ticks_remaining != 900
    {
        return Err("loaded flyer campaign changed".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-flyers-after.park.json"))?
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != original
    {
        return Err("flyer replay diverged or rewrote primary slot".into());
    }
    write_new(
        &store.directory().join("verification-flyers.txt"),
        b"Paid local flyers, accelerated candidates and return to normal pace resumed after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n",
    )?;
    println!("Local flyer continuation verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flyer_button_buys_once_and_saved_campaign_expires_on_schedule() {
        let mut park = fixture().unwrap();
        let before = park.clone();
        let notice =
            super::super::visitors::command(&mut park, VisitorCommand::RunFlyers, None, None)
                .unwrap();
        assert!(notice.contains("already running"));
        assert_eq!(park, before);
        replay(&mut park).unwrap();
        park.start_flyer_campaign().unwrap();
        assert_eq!(park.visitor_flow_stats().flyer_campaigns, 2);
    }
}
