//! Real visitor turnover with retained paid receipts, reviews and saved replay.
use super::*;
use crate::{Simulation, SurfaceId, ui};
use bevy::prelude::*;
use park_sim::{GuestId, INITIAL_GUEST_WALLET_CENTS};
use ui::visitor_flow::VisitorCommand;

pub(crate) fn check(p: &Park) -> Result<(), String> {
    super::ride_choice::check(p)?;
    super::value::check(p)?;
    let a = p.departed_visitors();
    let spent = p.guests().iter().map(|g| g.spent_cents).sum::<u64>() + a.spent_cents;
    let wallets = p
        .guests()
        .iter()
        .map(|g| u64::from(g.wallet_cents))
        .sum::<u64>()
        + a.wallet_cents;
    let sales = p.all_rides().map(|r| r.revenue_cents).sum::<u64>()
        + p.services().iter().map(|s| s.revenue_cents).sum::<u64>();
    if spent != sales
        || wallets + spent
            != (p.guests().len() as u64 + a.count) * u64::from(INITIAL_GUEST_WALLET_CENTS)
    {
        return Err("active/retired visitor money does not reconcile with actual sales".into());
    }
    let bytes = p.to_save_bytes().map_err(|e| e.to_string())?;
    if Park::from_save_bytes(&bytes).map_err(|e| e.to_string())? != *p {
        return Err("visitor state changed after strict roundtrip".into());
    }
    Ok(())
}
pub(crate) fn command(
    p: &mut Park,
    command: VisitorCommand,
    selected: Option<GuestId>,
    entry: Option<ObjectId>,
) -> Result<String, String> {
    let mut app = App::new();
    app.insert_resource(Simulation(p.clone()))
        .insert_resource(ViewState {
            guests_open: true,
            guest_flow_view: true,
            selected_guest: selected,
            selected: entry.map(SurfaceId::Object),
            speed: Speed::Paused,
            ..default()
        })
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                ui::button_actions,
                ui::visitor_flow::apply_commands,
                ui::guests::refresh,
                ui::visitor_flow::refresh,
            )
                .chain(),
        );
    app.update();
    let world = app.world_mut();
    let mut buttons = world.query::<(Entity, &ui::Action)>();
    let button = buttons
        .iter(world)
        .find_map(|(e, a)| matches!(a,ui::Action::Visitor(c) if *c==command).then_some(e))
        .ok_or("missing visitor button")?;
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    app.update();
    let notice = app
        .world()
        .resource::<ViewState>()
        .visitor_notice
        .clone()
        .ok_or("visitor command gave no result")?;
    let result = app.world().resource::<Simulation>().0.clone();
    app.update();
    if app.world().resource::<Simulation>().0 != result {
        return Err("held visitor button repeated".into());
    }
    *p = result;
    check(p)?;
    Ok(notice)
}
pub(crate) fn install() -> Result<Park, String> {
    let mut p = super::summary::install()?;
    p.borrow_loan(50_000).map_err(|e| e.to_string())?;
    let entry = p
        .objects()
        .iter()
        .find(|o| o.is_public_path() && !p.path_neighbors(o.id).is_empty())
        .ok_or("no public entry")?
        .id;
    let cash = p.cash_cents();
    command(&mut p, VisitorCommand::SetEntry, None, Some(entry))?;
    command(&mut p, VisitorCommand::Toggle, None, None)?;
    if p.cash_cents() != cash || !p.visitor_flow_stats().enabled {
        return Err("visitor setup minted cash or did not enable arrivals".into());
    }
    Ok(p)
}
pub(crate) fn paid_departure(p: &mut Park) -> Result<GuestId, String> {
    for _ in 0..6000 {
        if let Some(id) = p
            .guests()
            .iter()
            .find(|g| {
                g.ride_visit
                    .as_ref()
                    .and_then(|v| v.passenger.as_ref())
                    .is_some_and(|v| v.fare_paid)
            })
            .map(|g| g.id)
        {
            let cash = p.cash_cents();
            command(p, VisitorCommand::SendHome, Some(id), None)?;
            if p.cash_cents() != cash
                || !p
                    .guests()
                    .iter()
                    .any(|g| g.id == id && g.ride_visit.is_some())
            {
                return Err("departure request interrupted a paid passenger".into());
            }
            return Ok(id);
        }
        p.advance(Speed::Normal);
    }
    Err("no actual paid passenger found".into())
}
pub(crate) fn mature(p: &mut Park) -> Result<(), String> {
    for tick in 0..24_000 {
        p.advance(Speed::Normal);
        if tick % 300 == 0 {
            check(p)?;
        }
    }
    let s = p.visitor_flow_stats();
    if s.spawned == 0
        || s.departed == 0
        || p.departed_visitors().spent_cents == 0
        || !p
            .departed_visitors()
            .rides
            .iter()
            .any(|r| r.rating_count > 0)
        || p.finance_ledger().archived_entries == 0
    {
        return Err(
            "visitor fixture lacks real arrivals, paid departures, reviews or archived receipts"
                .into(),
        );
    }
    check(p)
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
    check(p)
}
pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut p = install()?;
    paid_departure(&mut p)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut p, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &p.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    mature(&mut p)?;
    command(&mut p, VisitorCommand::Toggle, None, None)?;
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
            return Err("visitor save changed after restart".into());
        }
        check(&p)?;
        if name == "primary" {
            replay(&mut p)?;
            if p != read_expected(&store.directory().join("expected-after.park.json"))? {
                return Err("visitor replay diverged after restart".into());
            }
        }
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("visitor verification altered source slots".into());
    }
    write_new(&store.directory().join("verification-visitors.txt"),b"Actual visitor controls, arrivals and paid departure request: PASS\nRetired wallets, ride/service receipts and reviews retained: PASS\nIndependent money and ledger reconciliation: PASS\n3600-tick exact restart continuation: PASS\nOriginal primary/backup bytes unchanged: PASS\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visitor_modes_preserve_paid_departures_and_lifetime_history_across_restart() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound visitors {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-visitors", "verify-visitors"] {
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
