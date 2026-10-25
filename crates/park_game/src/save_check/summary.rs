//! Actual mixed sales, ledger rollover, removal and read-only summary navigation.
use super::*;
use crate::{Simulation, ui};
use bevy::prelude::*;
use park_sim::{FinanceKind, PathDirection, ServiceId, ServiceKind};

pub(crate) fn install() -> Result<Park, String> {
    let (mut park, coaster, carousel) = super::cashflow::install()?;
    park.place_service(
        TilePos { x: 15, y: 21 },
        8,
        PathDirection::North,
        ServiceKind::Drink,
    )
    .map_err(|e| e.to_string())?;
    park.place_service(
        TilePos { x: 4, y: 5 },
        0,
        PathDirection::North,
        ServiceKind::Toilet,
    )
    .map_err(|e| e.to_string())?;
    for id in [coaster, carousel] {
        super::opening::toggle(&mut park, id)?;
    }
    check(&park)?;
    Ok(park)
}

/// Expected buckets come from opening + archived + recent receipts, independently
/// of the lifetime-counter implementation used by the summary.
pub(crate) fn check(park: &Park) -> Result<(), String> {
    super::finance::check(park)?;
    let before = park.to_save_bytes().map_err(|e| e.to_string())?;
    let ledger = park.finance_ledger();
    let mut ride_revenue = ledger.opening_ride_revenue_cents + ledger.archived.ride_revenue_cents;
    let mut service_revenue = ledger
        .opening_services
        .iter()
        .map(|s| s.revenue_cents)
        .sum::<u64>()
        + ledger.archived.service_revenue_cents;
    let mut ride_cost = ledger
        .archived_rides
        .iter()
        .map(|r| r.upkeep_cents)
        .sum::<u64>();
    let mut service_cost = ledger.archived.upkeep_cents - ride_cost;
    let mut supplies = ledger.archived.supplies_cents;
    let mut construction = ledger.archived.construction_cents;
    let mut refunds = ledger.archived.demolition_refunds_cents;
    let mut borrowed = ledger.archived.loan_borrowed_cents;
    let mut repaid = ledger.archived.loan_repaid_cents;
    let mut interest = ledger.archived.loan_interest_cents;
    for entry in &ledger.entries {
        let amount = entry.amount_cents;
        match entry.kind {
            FinanceKind::LoanBorrow => borrowed += amount,
            FinanceKind::LoanRepayment => repaid += amount,
            FinanceKind::LoanInterest => interest += amount,
            FinanceKind::RideRevenue => ride_revenue += amount,
            FinanceKind::ServiceRevenue => service_revenue += amount,
            FinanceKind::RideUpkeep => ride_cost += amount,
            FinanceKind::ServiceUpkeep => service_cost += amount,
            FinanceKind::ServiceSupplies => supplies += amount,
            FinanceKind::RideConstruction
            | FinanceKind::TrackConstruction
            | FinanceKind::ServiceConstruction => construction += amount,
            FinanceKind::RideDemolitionRefund
            | FinanceKind::TrackRefund
            | FinanceKind::ServiceDemolitionRefund => refunds += amount,
        }
    }
    let report = park.finance_summary();
    let ride_profit = i128::from(ride_revenue) - i128::from(ride_cost);
    let service_profit =
        i128::from(service_revenue) - i128::from(service_cost) - i128::from(supplies);
    if report.loan_borrowed_cents != borrowed
        || report.loan_repaid_cents != repaid
        || report.loan_interest_cents != interest
        || report.rides.revenue_cents != ride_revenue
        || report.rides.upkeep_cents != ride_cost
        || report.rides.supplies_cents != 0
        || report.rides.profit_cents != ride_profit
        || report.services.revenue_cents != service_revenue
        || report.services.upkeep_cents != service_cost
        || report.services.supplies_cents != supplies
        || report.services.profit_cents != service_profit
        || report.operating_profit_cents != ride_profit + service_profit
        || report.construction_cents != construction
        || report.refunds_cents != refunds
        || report.net_cash_cents
            != ride_profit + service_profit + i128::from(refunds) - i128::from(construction)
                + i128::from(borrowed)
                - i128::from(repaid)
                - i128::from(interest)
        || i128::from(report.starting_cash_cents) + report.net_cash_cents
            != i128::from(park.cash_cents())
        || report.cash_cents != park.cash_cents()
    {
        return Err(
            "park summary differs from independent ledger buckets or cash conservation".into(),
        );
    }
    if before != park.to_save_bytes().map_err(|e| e.to_string())? {
        return Err("summary query changed save bytes".into());
    }
    Ok(())
}

pub(crate) fn inspect(park: &Park) -> Result<String, String> {
    check(park)?;
    let mut app = App::new();
    app.insert_resource(Simulation(park.clone()))
        .insert_resource(ViewState {
            finance_open: true,
            finance_view: ui::finance::FinanceView::Rides,
            finance_view_pending: true,
            finance_page: usize::MAX,
            finance_page_pending: i32::MAX,
            speed: Speed::Paused,
            ..default()
        })
        .add_systems(Startup, ui::setup)
        .add_systems(Update, ui::finance::refresh);
    app.update();
    if app.world().resource::<ViewState>().finance_view != ui::finance::FinanceView::ParkSummary
        || app.world().resource::<ViewState>().finance_page != 0
    {
        return Err("summary view/page did not reset".into());
    }
    let expected = ui::finance::summary_rows(park);
    let world = app.world_mut();
    let mut query = world.query::<(&ui::finance::FinanceLabel, &Text)>();
    for (index, row) in expected.iter().enumerate() {
        if !query.iter(world).any(|(label, text)| {
            matches!(label,ui::finance::FinanceLabel::Row(i) if *i==index) && text.0 == *row
        }) {
            return Err("displayed summary row differs from report".into());
        }
    }
    for _ in 0..5 {
        app.world_mut()
            .resource_mut::<ViewState>()
            .finance_view_pending = true;
        app.update();
    }
    let after = &app.world().resource::<Simulation>().0;
    if after != park
        || after.to_save_bytes().map_err(|e| e.to_string())?
            != park.to_save_bytes().map_err(|e| e.to_string())?
    {
        return Err("summary navigation mutated the park".into());
    }
    Ok(expected.join("\n\n"))
}

pub(crate) fn mature(park: &mut Park, remove: bool) -> Result<(), String> {
    for tick in 0..7200 {
        park.advance(Speed::Normal);
        if tick % 300 == 0 {
            check(park)?;
        }
    }
    let report = park.finance_summary();
    if report.rides.revenue_cents == 0
        || report.services.revenue_cents == 0
        || report.rides.upkeep_cents == 0
        || report.services.supplies_cents == 0
        || park.finance_ledger().archived_entries == 0
    {
        return Err("mixed summary lacks actual sales, costs or archived receipts".into());
    }
    if remove {
        let id: ServiceId = park
            .services()
            .iter()
            .find(|s| s.kind == ServiceKind::Drink && s.revenue_cents > 0)
            .ok_or("missing paid service")?
            .id;
        let before = park.finance_summary();
        let object = park
            .service(id)
            .unwrap()
            .object
            .ok_or("service already removed")?;
        park.remove_object(object)
            .ok_or("service removal rejected")?;
        if before.services != park.finance_summary().services {
            return Err("demolition changed historical service operating profit".into());
        }
    }
    inspect(park)?;
    Ok(())
}

fn replay(park: &mut Park) -> Result<(), String> {
    for tick in 0..3600 {
        park.advance(Speed::Normal);
        if tick % 300 == 0 {
            check(park)?;
            *park = Park::from_save_bytes(&park.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
    }
    inspect(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = install()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    write_new(
        &store.directory().join("summary-backup.txt"),
        inspect(&park)?.as_bytes(),
    )?;
    mature(&mut park, true)?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    write_new(
        &store.directory().join("summary-primary.txt"),
        inspect(&park)?.as_bytes(),
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    for (action, name) in [
        (SaveAction::Load, "primary"),
        (SaveAction::Backup, "backup"),
    ] {
        if !persistence::perform_action(session, &mut park, &mut view, action) {
            return Err(view.save_notice);
        }
        if park != read_expected(&store.directory().join(format!("expected-{name}.park.json")))?
            || inspect(&park)?.as_bytes()
                != fs::read(store.directory().join(format!("summary-{name}.txt")))
                    .map_err(|e| e.to_string())?
        {
            return Err("loaded park summary differs from writer".into());
        }
        if name == "primary" {
            replay(&mut park)?;
            if park != read_expected(&store.directory().join("expected-after.park.json"))? {
                return Err("summary restart continuation differs".into());
            }
        }
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("summary verification modified source saves".into());
    }
    write_new(&store.directory().join("verification-summary.txt"),b"Independent ledger buckets and exact cash reconciliation: PASS\nActual mixed ride/service sales and expenses: PASS\nRemoved owner history and archived receipts: PASS\nRead-only UI navigation: PASS\n3600-tick restart continuation: PASS\nPrimary/backup bytes unchanged: PASS\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn summary_modes_verify_mixed_operating_profit_and_capital_cash_separately() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound summary proof {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-summary", "verify-summary"] {
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
