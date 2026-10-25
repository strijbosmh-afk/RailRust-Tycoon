//! Real construction, cashless arrears, actual loan buttons and archived funding.
use super::*;
use crate::{Simulation, ui};
use bevy::prelude::*;
use park_sim::{PathDirection, ServiceKind};
use ui::finance::{FinanceView, LoanCommand};

pub(crate) fn command(park: &mut Park, command: LoanCommand) -> Result<String, String> {
    let mut app = App::new();
    app.insert_resource(Simulation(park.clone()))
        .insert_resource(ViewState {
            finance_open: true,
            finance_view: FinanceView::Loans,
            speed: Speed::Paused,
            ..default()
        })
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                ui::button_actions,
                ui::finance::apply_commands,
                ui::finance::refresh,
            )
                .chain(),
        );
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &ui::Action)>();
    let button = query
        .iter(world)
        .find_map(|(e, a)| matches!(a,ui::Action::Loan(c) if *c==command).then_some(e))
        .ok_or("missing loan button")?;
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    app.update();
    let notice = app
        .world()
        .resource::<ViewState>()
        .finance_loan_notice
        .clone()
        .ok_or("loan command had no result")?;
    let result = app.world().resource::<Simulation>().0.clone();
    app.update();
    if app.world().resource::<Simulation>().0 != result {
        return Err("held loan button repeated".into());
    }
    *park = result;
    super::summary::check(park)?;
    Ok(notice)
}

pub(crate) fn install() -> Result<Park, String> {
    let mut p = Park::new(32, 32);
    command(&mut p, LoanCommand::Borrow)?;
    p.place_carousel(TilePos { x: 19, y: 22 }, 0)
        .map_err(|e| e.to_string())?;
    for x in (2..=16).step_by(2) {
        p.place_service(
            TilePos { x, y: 2 },
            0,
            PathDirection::North,
            ServiceKind::Toilet,
        )
        .map_err(|e| e.to_string())?;
    }
    if p.cash_cents() != 0 {
        return Err("actual purchases did not exhaust loan-funded cash".into());
    }
    for _ in 0..1800 {
        p.advance(Speed::Normal);
    }
    if p.loan_stats().interest_due_cents != 50
        || p.loan_stats().interest_paid_cents != 0
        || p.finance_summary().operating_profit_cents != 0
    {
        return Err("cashless interest arrears or operating profit is incorrect".into());
    }
    super::summary::inspect(&p)?;
    Ok(p)
}

pub(crate) fn recover(p: &mut Park) -> Result<(), String> {
    command(p, LoanCommand::Borrow)?;
    let before = p.clone();
    let notice = command(p, LoanCommand::Repay)?;
    if *p != before || !notice.contains("Need cash") {
        return Err("unaffordable principal/interest repayment was not atomic".into());
    }
    command(p, LoanCommand::Interest)?;
    command(p, LoanCommand::Borrow)?;
    command(p, LoanCommand::Repay)?;
    if p.cash_cents() != 9950
        || p.loan_stats().principal_cents != 20_000
        || p.loan_stats().interest_paid_cents != 50
        || p.services().iter().any(|s| s.open)
        || p.finance_summary().operating_profit_cents != 0
    {
        return Err("funding recovery invented operating income or reopened a facility".into());
    }
    for _ in 0..140 {
        p.borrow_loan(10_000).map_err(|e| e.to_string())?;
        p.repay_loan(10_000).map_err(|e| e.to_string())?;
    }
    if p.finance_ledger().archived_entries == 0 {
        return Err("financing fixture did not roll the ledger".into());
    }
    super::summary::inspect(p)?;
    Ok(())
}

pub(crate) fn partial() -> Result<Park, String> {
    let mut p = install()?;
    command(&mut p, LoanCommand::Borrow)?;
    command(&mut p, LoanCommand::Interest)?;
    for _ in 0..180_000 {
        p.advance(Speed::Normal);
    }
    if p.cash_cents() != 0 || p.loan_stats().interest_due_cents != 50 {
        return Err("partial collection failed to retain the unpaid balance".into());
    }
    super::summary::check(&p)?;
    Ok(p)
}

fn replay(p: &mut Park) -> Result<(), String> {
    for tick in 0..3600 {
        p.advance(Speed::Normal);
        if tick % 300 == 0 {
            super::summary::check(p)?;
            *p = Park::from_save_bytes(&p.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
    }
    super::summary::inspect(p)?;
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
    recover(&mut p)?;
    if !persistence::perform_action(session, &mut p, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &p.to_save_bytes().map_err(|e| e.to_string())?,
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
        if p != read_expected(&store.directory().join(format!("expected-{name}.park.json")))? {
            return Err("loaded loan account changed".into());
        }
        super::summary::inspect(&p)?;
        if name == "primary" {
            replay(&mut p)?;
            if p != read_expected(&store.directory().join("expected-after.park.json"))? {
                return Err("loan continuation differs after restart".into());
            }
        }
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("loan verification changed source save bytes".into());
    }
    write_new(&store.directory().join("verification-loans.txt"),b"Loan funds and repayments separate from operating profit: PASS\nReal purchases, arrears and atomic repayment refusal: PASS\nActual Borrow/Repay/Pay interest controls: PASS\nArchived financing and cash reconciliation: PASS\n3600-tick exact restart continuation: PASS\nOriginal primary/backup bytes unchanged: PASS\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loan_modes_verify_actual_buttons_arrears_recovery_and_restart() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound loans {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-loans", "verify-loans"] {
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
