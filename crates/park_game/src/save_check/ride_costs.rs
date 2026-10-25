//! Real purchases drain cash; a paid trip survives closure and manual reopening.
use super::*;
use crate::ride_operations::{self, OperationCommand};
use park_sim::{RideId, RideOperatingSettings, TrackKind};

fn require(ok: bool, message: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(message.into()) }
}
pub(crate) fn install() -> Result<(Park, RideId, RideId), String> {
    let mut park = demonstration_park();
    park.set_train_open(false);
    let id = super::journey::install(&mut park)?;
    for _ in 0..4 {
        let c = park
            .place_carousel(super::ride_choice::ORIGIN, 8)
            .map_err(|e| e.to_string())?;
        park.remove_carousel(c).map_err(|e| e.to_string())?;
    }
    let reserve = park
        .place_carousel(super::ride_choice::ORIGIN, 8)
        .map_err(|e| e.to_string())?;
    park.copy_installed_track_to_draft()
        .map_err(|e| e.to_string())?;
    let pieces = park
        .track_draft()
        .iter()
        .filter(|p| p.kind == TrackKind::Straight)
        .take(4)
        .copied()
        .collect::<Vec<_>>();
    require(pieces.len() == 4, "need four replaceable straights")?;
    let cycled = pieces[0];
    for mut p in pieces {
        p.kind = TrackKind::Brake;
        park.set_track_draft_piece(p).map_err(|e| e.to_string())?;
    }
    let quote = park.track_construction_quote().map_err(|e| e.to_string())?;
    require(
        quote.construction_cents == 8_000,
        "paid brake replacement quote changed",
    )?;
    park.apply_track_draft().map_err(|e| e.to_string())?;
    // An actual loss-making replacement cycle leaves $5 of operating float.
    for kind in [TrackKind::Straight, TrackKind::Brake] {
        let mut p = cycled;
        p.kind = kind;
        park.set_track_draft_piece(p).map_err(|e| e.to_string())?;
        park.apply_track_draft().map_err(|e| e.to_string())?;
    }
    park.set_coaster_car_count(id, 1)
        .map_err(|e| e.to_string())?;
    park.set_ride_operating_settings(
        id,
        RideOperatingSettings {
            station_wait_ticks: 900,
            lift_speed: 12,
            brake_speed: 8,
            ..RideOperatingSettings::default()
        },
    )
    .map_err(|e| e.to_string())?;
    park.set_ride_ticket_price(id, 50)
        .map_err(|e| e.to_string())?;
    require(
        park.test_coaster().map_err(|e| e.to_string())?.moving_ticks > 1800,
        "paid lap must extend beyond the second billing boundary",
    )?;
    require(
        park.cash_cents() == 500,
        "real purchases did not leave the planned operating float",
    )?;
    park.set_ride_ticket_price(id, park_sim::MAX_TICKET_PRICE_CENTS)
        .map_err(|e| e.to_string())?;
    park.set_ride_open_checked(id, true)
        .map_err(|e| e.to_string())?;
    for _ in 0..4 * usize::from(park_sim::RIDE_UPKEEP_TICKS) {
        super::ride_choice::advance_checked(&mut park)?;
    }
    park.set_ride_open_checked(id, false)
        .map_err(|e| e.to_string())?;
    super::ride_choice::seek(
        &mut park,
        |p| p.ride_operating_settings_status(id).is_ok(),
        "empty warm-up train finishes closed",
    )?;
    require(
        park.cash_cents() == 100 && park.ride_running_costs(id).unwrap().upkeep_cents == 400,
        "four genuine warm-up bills did not leave $1",
    )?;
    park.set_ride_ticket_price(id, 50)
        .map_err(|e| e.to_string())?;
    // Prepare a natural public-path crowd near the queue while the ride is
    // closed. The first paid departure can then fund the first idle-cost bill.
    let entry = park
        .ride_access(id)
        .public_entry
        .ok_or("missing public entry")?;
    let mut nearby = std::collections::BTreeSet::from([entry]);
    for _ in 0..4 {
        for path in nearby.clone() {
            nearby.extend(park.path_neighbors(path));
        }
    }
    super::ride_choice::seek(
        &mut park,
        |p| {
            p.guests()
                .iter()
                .filter(|g| {
                    g.ride_cooldown == 0 && g.current.is_some_and(|path| nearby.contains(&path))
                })
                .count()
                >= 3
        },
        "three walking guests near the closed queue entry",
    )?;
    super::ride_choice::bytes(&park)?;
    Ok((park, id, reserve))
}
pub(crate) fn open(park: &mut Park, id: RideId) -> Result<(), String> {
    let mut view = ViewState::default();
    if ride_operations::perform_command(park, &mut view, id, OperationCommand::ToggleOpen) {
        Ok(())
    } else {
        Err(view
            .ride_operating_notice
            .unwrap_or("cost opening command refused".into()))
    }
}
pub(crate) fn paid(park: &mut Park, id: RideId) -> Result<(), String> {
    open(park, id)?;
    super::ride_choice::seek(
        park,
        |p| super::ride_choice::paid_running(p, id),
        "two paid passengers with operating float",
    )?;
    require(
        park.ride_journey_stats(id).riding == 2 && park.cash_cents() == 100,
        "two half-dollar fares were not charged exactly once",
    )?;
    // Future boarders cannot replenish cash before the funds-closure proof.
    park.set_ride_ticket_price(id, park_sim::MAX_TICKET_PRICE_CENTS)
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub(crate) fn closed(park: &mut Park, id: RideId) -> Result<(), String> {
    let before = park.ride_by_id(id).unwrap().revenue_cents;
    super::ride_choice::seek(
        park,
        |p| p.ride_running_costs(id).unwrap().closed_for_funds,
        "funds closure during the already-paid trip",
    )?;
    let costs = park.ride_running_costs(id).unwrap();
    require(
        park.cash_cents() == 0
            && costs.upkeep_cents == 600
            && costs.upkeep_progress == 899
            && !park.ride_is_open(id),
        "missed payment charged money, lost its period or left admission open",
    )?;
    require(
        super::ride_choice::paid_running(park, id),
        "paid trip did not survive automatic closure",
    )?;
    require(
        park.ride_by_id(id).unwrap().revenue_cents == before,
        "closure changed existing fares",
    )?;
    let original = park.clone();
    require(
        open(park, id).is_err() && *park == original,
        "unfunded reopening was not refused atomically",
    )?;
    Ok(())
}
pub(crate) fn reopen(park: &mut Park, id: RideId, reserve: RideId) -> Result<(), String> {
    let completed = park.ride_by_id(id).unwrap().completed;
    super::ride_choice::seek(
        park,
        |p| p.ride_operating_settings_status(id).is_ok(),
        "closed paid passengers finish and exit",
    )?;
    require(
        park.ride_by_id(id).unwrap().completed > completed,
        "paid guests did not complete their trip after closure",
    )?;
    require(
        park.cash_cents() == 0 && park.ride_running_costs(id).unwrap().upkeep_cents == 600,
        "closed trip charged further upkeep",
    )?;
    park.remove_carousel(reserve).map_err(|e| e.to_string())?;
    require(
        park.cash_cents() == 15_000,
        "reserve demolition did not provide its exact paid refund",
    )?;
    require(
        !park.ride_is_open(id),
        "adding cash reopened a ride automatically",
    )?;
    open(park, id)?;
    super::ride_choice::advance_checked(park)?;
    require(
        park.cash_cents() == 14_900
            && park.ride_running_costs(id).unwrap().upkeep_cents == 700
            && !park.ride_running_costs(id).unwrap().closed_for_funds,
        "manual reopening did not pay the single overdue bill next tick",
    )
}
fn replay(park: &mut Park) -> Result<(), String> {
    let id = park.ride().ok_or("missing cost coaster")?.id;
    let reserve = park
        .carousels()
        .iter()
        .find(|r| !r.objects.is_empty())
        .ok_or("missing refund reserve")?
        .ride
        .id;
    closed(park, id)?;
    reopen(park, id, reserve)?;
    for tick in 0..3600 {
        super::ride_choice::advance_checked(park)?;
        if tick % 300 == 0 {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let (mut park, id, _) = install()?;
    paid(&mut park, id)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    super::ride_choice::seek(
        &mut park,
        |p| p.ride_running_costs(id).unwrap().upkeep_progress == 899,
        "paid primary one tick before next bill",
    )?;
    require(
        park.cash_cents() == 100,
        "primary must still hold the first fare income",
    )?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &super::ride_choice::bytes(&park)?,
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
    require(
        park == read_expected(&store.directory().join("expected-primary.park.json"))?,
        "mid-period primary changed at load",
    )?;
    replay(&mut park)?;
    require(
        park == read_expected(&store.directory().join("expected-after.park.json"))?,
        "funds closure/reopening continuation differs after restart",
    )?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    require(
        park == read_expected(&store.directory().join("expected-backup.park.json"))?,
        "paid backup changed at load",
    )?;
    require(
        fs::read(store.save_path()).map_err(|e| e.to_string())? == primary
            && fs::read(store.backup_path()).map_err(|e| e.to_string())? == backup,
        "verification changed source slots",
    )?;
    write_new(&store.directory().join("verification-ride-costs.txt"),b"Actual purchases, fares and owner-attributed upkeep: PASS\nFunds closure preserves paid moving passengers: PASS\nUnfunded opening refused; closed trips finish without extra upkeep: PASS\nCash recovery does not auto-open; manual opening retains and pays overdue bill: PASS\nExact restart continuation / source slots unchanged: PASS\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operating_cost_modes_verify_paid_closure_recovery_and_pending_periods() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound ride costs {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-ride-costs", "verify-ride-costs"] {
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
