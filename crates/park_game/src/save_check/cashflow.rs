//! Production report rendering, retained demolished history and paid continuation.
use super::*;
use crate::{Simulation, ui};
use bevy::prelude::*;
use park_sim::RideId;

pub(crate) fn install() -> Result<(Park, RideId, RideId), String> {
    let mut park = super::starters::install(0)?;
    let coaster = park.ride().ok_or("missing paid coaster")?.id;
    let historical = park
        .place_carousel(super::ride_choice::ORIGIN, 8)
        .map_err(|e| e.to_string())?;
    park.remove_carousel(historical)
        .map_err(|e| e.to_string())?;
    let carousel = super::ride_choice::install_carousel(&mut park)?;
    park.set_ride_open_checked(carousel, false)
        .map_err(|e| e.to_string())?;
    inspect(&park)?;
    Ok((park, coaster, carousel))
}

/// The actual finance refresh builds report labels on a cloned resource. Its
/// navigation must not alter any authoritative field or encoded save byte.
pub(crate) fn inspect(park: &Park) -> Result<String, String> {
    let rows = park.ride_cashflows();
    let stats = park.finance_stats();
    if rows.len() != 3
        || rows.iter().filter(|r| r.installed).count() != 2
        || rows.iter().map(|r| r.construction_cents).sum::<u64>() != 87_000
        || rows.iter().map(|r| r.refunds_cents).sum::<u64>() != 15_000
        || rows.iter().map(|r| r.ticket_revenue_cents).sum::<u64>() != stats.ride_revenue_cents
        || rows.iter().map(|r| r.net_cash_cents).sum::<i128>()
            != i128::from(park.cash_cents()) - i128::from(park_sim::INITIAL_PARK_CASH_CENTS)
    {
        return Err(
            "ride cashflows do not reconcile with actual construction/refund/revenue/cash".into(),
        );
    }
    let before_bytes = super::ride_choice::bytes(park)?;
    let mut app = App::new();
    app.insert_resource(Simulation(park.clone()))
        .insert_resource(ViewState {
            finance_open: true,
            finance_view_pending: true,
            speed: Speed::Paused,
            ..default()
        })
        .add_systems(Startup, ui::setup)
        .add_systems(Update, ui::finance::refresh);
    app.update();
    if app.world().resource::<ViewState>().finance_view != ui::finance::FinanceView::Rides {
        return Err("report view did not switch".into());
    }
    let world = app.world_mut();
    let mut query = world.query::<(&ui::finance::FinanceLabel, &Text)>();
    let mut actual = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let expected = ui::finance::cashflow_text(row);
        let displayed = query
            .iter(world)
            .find_map(|(label, text)| {
                matches!(label,ui::finance::FinanceLabel::Row(row) if *row == index)
                    .then_some(text.0.clone())
            })
            .ok_or("missing report row")?;
        if displayed != expected {
            return Err("displayed report differs from authoritative figures".into());
        }
        actual.push(displayed);
    }
    for _ in 0..5 {
        app.world_mut()
            .resource_mut::<ViewState>()
            .finance_view_pending = true;
        app.update();
    }
    let after = &app.world().resource::<Simulation>().0;
    if after != park || after.to_save_bytes().map_err(|e| e.to_string())? != before_bytes {
        return Err("report navigation changed authoritative state".into());
    }
    Ok(actual.join("\n\n"))
}

fn replay(park: &mut Park) -> Result<(), String> {
    let before = park
        .all_rides()
        .map(|r| (r.id, r.completed))
        .collect::<Vec<_>>();
    for tick in 0..7200 {
        super::ride_choice::advance_checked(park)?;
        if tick % 600 == 0 {
            inspect(park)?;
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    for (id, completed) in before {
        if park.ride_cashflow(id).is_some_and(|r| r.installed)
            && park.ride_by_id(id).unwrap().completed <= completed
        {
            return Err("reported live ride completed no further journey".into());
        }
    }
    inspect(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let (mut park, coaster, carousel) = install()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    write_new(
        &store.directory().join("cashflow-backup.txt"),
        inspect(&park)?.as_bytes(),
    )?;
    for id in [coaster, carousel] {
        super::opening::toggle(&mut park, id)?;
    }
    super::ride_choice::seek(
        &mut park,
        |p| {
            super::ride_choice::paid_running(p, coaster)
                && super::ride_choice::paid_running(p, carousel)
        },
        "both reported rides carry paid passengers",
    )?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    write_new(
        &store.directory().join("cashflow-primary.txt"),
        inspect(&park)?.as_bytes(),
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &super::ride_choice::bytes(&park)?,
    )
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || inspect(&park)?.as_bytes()
            != fs::read(store.directory().join("cashflow-primary.txt"))
                .map_err(|e| e.to_string())?
    {
        return Err("paid report changed after load".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("cashflow continuation differs".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || inspect(&park)?.as_bytes()
            != fs::read(store.directory().join("cashflow-backup.txt")).map_err(|e| e.to_string())?
    {
        return Err("closed report backup changed".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("report verification changed source slots".into());
    }
    write_new(&store.directory().join("verification-cashflow.txt"),b"Paid coaster/live carousel/removed carousel figures reconcile: PASS\nActual UI refresh and view navigation are read-only: PASS\nOccupied paid primary and closed backup reports: PASS\n7200-tick exact restart continuation: PASS\nOriginal slots unchanged: PASS\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cashflow_modes_verify_actual_ui_reports_and_retained_paid_history() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound cashflow proof {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-cashflow", "verify-cashflow"] {
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
