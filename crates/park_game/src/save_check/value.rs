//! Book-value queries checked against independent archived/recent cash receipts.
use super::*;
use crate::{Simulation, ui};
use bevy::prelude::*;
use park_sim::{FinanceKind, RideId, ServiceId};

fn ride_basis(p: &Park, id: RideId) -> Result<u64, String> {
    let l = p.finance_ledger();
    let mut build = l
        .archived_rides
        .iter()
        .filter(|r| r.ride == id)
        .map(|r| r.construction_cents)
        .sum::<u64>();
    let mut refund = l
        .archived_rides
        .iter()
        .filter(|r| r.ride == id)
        .map(|r| r.demolition_refunds_cents)
        .sum::<u64>();
    for e in l.entries.iter().filter(|e| e.ride == Some(id)) {
        match e.kind {
            FinanceKind::RideConstruction | FinanceKind::TrackConstruction => {
                build += e.amount_cents
            }
            FinanceKind::RideDemolitionRefund | FinanceKind::TrackRefund => {
                refund += e.amount_cents
            }
            _ => {}
        }
    }
    build
        .checked_sub(2 * refund)
        .ok_or("ride basis is not backed by settled receipts".into())
}
fn service_basis(p: &Park, id: ServiceId) -> Result<u64, String> {
    let l = p.finance_ledger();
    let mut build = l
        .archived_services
        .iter()
        .filter(|r| r.service == id)
        .map(|r| r.construction_cents)
        .sum::<u64>();
    let mut refund = l
        .archived_services
        .iter()
        .filter(|r| r.service == id)
        .map(|r| r.demolition_refunds_cents)
        .sum::<u64>();
    for e in l.entries.iter().filter(|e| e.service == Some(id)) {
        match e.kind {
            FinanceKind::ServiceConstruction => build += e.amount_cents,
            FinanceKind::ServiceDemolitionRefund => refund += e.amount_cents,
            _ => {}
        }
    }
    build
        .checked_sub(2 * refund)
        .ok_or("service basis is not backed by settled receipts".into())
}
pub(crate) fn check(p: &Park) -> Result<(), String> {
    super::summary::check(p)?;
    let before = p.to_save_bytes().map_err(|e| e.to_string())?;
    let v = p.park_value();
    let track = p
        .ride()
        .map(|r| ride_basis(p, r.id))
        .transpose()?
        .unwrap_or(0);
    let carousel = p
        .carousels()
        .iter()
        .map(|r| ride_basis(p, r.ride.id))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .sum::<u64>();
    let services = p
        .services()
        .iter()
        .map(|s| service_basis(p, s.id))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .sum::<u64>();
    let debt = p.loan_stats();
    if v.track_cents != track
        || v.carousel_cents != carousel
        || v.service_cents != services
        || v.park_cents != track + carousel + services
        || v.cash_cents != p.cash_cents()
        || v.principal_cents != debt.principal_cents
        || v.interest_due_cents != debt.interest_due_cents
        || v.gross_assets_cents
            != i128::from(p.cash_cents()) + i128::from(track + carousel + services)
        || v.company_cents
            != v.gross_assets_cents
                - i128::from(debt.principal_cents)
                - i128::from(debt.interest_due_cents)
        || before != p.to_save_bytes().map_err(|e| e.to_string())?
    {
        return Err("book values differ from settled owner receipts, cash or liabilities".into());
    }
    Ok(())
}

pub(crate) fn inspect(p: &Park) -> Result<String, String> {
    check(p)?;
    let before = p.to_save_bytes().map_err(|e| e.to_string())?;
    let mut app = App::new();
    app.insert_resource(Simulation(p.clone()))
        .insert_resource(ViewState {
            finance_open: true,
            finance_view: ui::finance::FinanceView::Loans,
            finance_view_pending: true,
            finance_page: usize::MAX,
            finance_page_pending: i32::MAX,
            speed: Speed::Paused,
            ..default()
        })
        .add_systems(Startup, ui::setup)
        .add_systems(Update, ui::finance::refresh);
    app.update();
    let view = app.world().resource::<ViewState>();
    if view.finance_view != ui::finance::FinanceView::Value || view.finance_page != 0 {
        return Err("value view did not reset paging".into());
    }
    let rows = ui::finance::value_rows(p);
    let world = app.world_mut();
    let mut query = world.query::<(&ui::finance::FinanceLabel, &Text)>();
    for (i, row) in rows.iter().enumerate() {
        if !query.iter(world).any(|(label, text)| {
            matches!(label,ui::finance::FinanceLabel::Row(n) if *n==i) && text.0 == *row
        }) {
            return Err("value labels differ from queried amounts".into());
        }
    }
    for _ in 0..5 {
        app.world_mut()
            .resource_mut::<ViewState>()
            .finance_view_pending = true;
        app.update();
    }
    if app
        .world()
        .resource::<Simulation>()
        .0
        .to_save_bytes()
        .map_err(|e| e.to_string())?
        != before
    {
        return Err("valuation navigation changed the saved park".into());
    }
    Ok(rows.join("\n\n"))
}

pub(crate) fn install() -> Result<Park, String> {
    let mut p = super::summary::install()?;
    let equity = p.park_value().company_cents;
    let operating = p.finance_summary().operating_profit_cents;
    super::loans::command(&mut p, ui::finance::LoanCommand::Borrow)?;
    if p.park_value().company_cents != equity
        || p.finance_summary().operating_profit_cents != operating
    {
        return Err("borrowing created equity or operating profit".into());
    }
    inspect(&p)?;
    Ok(p)
}
pub(crate) fn negative() -> Result<Park, String> {
    let mut p = Park::new(32, 32);
    p.borrow_loan(20_000).map_err(|e| e.to_string())?;
    for _ in 0..7 {
        let id = p
            .place_carousel(TilePos { x: 19, y: 22 }, 0)
            .map_err(|e| e.to_string())?;
        p.remove_carousel(id).map_err(|e| e.to_string())?;
    }
    if p.park_value().company_cents != -5000 || p.cash_cents() != 15000 {
        return Err("actual demolition losses did not produce negative equity".into());
    }
    inspect(&p)?;
    Ok(p)
}
pub(crate) fn faulted() -> Result<Park, String> {
    let mut p = super::starters::install(0)?;
    let piece = p
        .track_economy()
        .paid_pieces
        .iter()
        .find(|r| r.spec.kind == park_sim::TrackKind::Straight)
        .ok_or("no paid straight")?
        .object;
    p.remove_object(piece).ok_or("paid track removal failed")?;
    if p.circuit().is_some() || p.park_value().track_cents != 26000 {
        return Err("faulted remnants lost their basis".into());
    }
    inspect(&p)?;
    Ok(p)
}
fn replay(p: &mut Park) -> Result<(), String> {
    for tick in 0..3600 {
        p.advance(Speed::Normal);
        if tick % 300 == 0 {
            check(p)?;
            *p = Park::from_save_bytes(&p.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
    }
    inspect(p)?;
    Ok(())
}
pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut p = install()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut p, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &p.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    write_new(
        &store.directory().join("value-backup.txt"),
        inspect(&p)?.as_bytes(),
    )?;
    super::summary::mature(&mut p, true)?;
    if !persistence::perform_action(session, &mut p, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &p.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    write_new(
        &store.directory().join("value-primary.txt"),
        inspect(&p)?.as_bytes(),
    )?;
    replay(&mut p)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &p.to_save_bytes().map_err(|e| e.to_string())?,
    )
}
pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut p = demonstration_park();
    let mut view = ViewState::default();
    for (action, name) in [
        (SaveAction::Load, "primary"),
        (SaveAction::Backup, "backup"),
    ] {
        if !persistence::perform_action(session, &mut p, &mut view, action) {
            return Err(view.save_notice);
        }
        if p != read_expected(&store.directory().join(format!("expected-{name}.park.json")))?
            || inspect(&p)?.as_bytes()
                != fs::read(store.directory().join(format!("value-{name}.txt")))
                    .map_err(|e| e.to_string())?
        {
            return Err("saved valuation changed after restart".into());
        }
        if name == "primary" {
            replay(&mut p)?;
            if p != read_expected(&store.directory().join("expected-after.park.json"))? {
                return Err("valuation replay changed future state".into());
            }
        }
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("valuation verification altered source slots".into());
    }
    write_new(&store.directory().join("verification-value.txt"),b"Paid basis reconciles against archived/recent owner receipts: PASS\nBorrowing does not create equity or operating profit: PASS\nMixed receipts, interest and removed paid service: PASS\nRead-only actual UI refresh/navigation: PASS\n3600-tick exact restart continuation: PASS\nOriginal save slots unchanged: PASS\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valuation_modes_reconcile_debt_assets_income_and_restart() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound value {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-value", "verify-value"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    dir.clone().into_os_string()
                ]),
                Some(AppExit::Success)
            );
        }
        assert!(dir.starts_with(std::env::temp_dir()));
        let _ = fs::remove_dir_all(dir);
    }
}
