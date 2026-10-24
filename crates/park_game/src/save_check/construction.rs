//! Command-priced ride construction, retained refund basis and cash continuation.
use super::*;
use park_sim::{CarouselError, FinanceKind, RideId};

// Independent contract values: the proof must detect an accidental price change.
const BUILD: u64 = 30_000;
const REFUND: u64 = 15_000;
const REPLAY_TICKS: u64 = 7_200;

fn receipt(
    park: &Park,
    before: &Park,
    id: RideId,
    kind: FinanceKind,
    amount: u64,
) -> Result<(), String> {
    let entry = park
        .finance_ledger()
        .entries
        .last()
        .ok_or("missing construction transaction")?;
    let cash = if kind == FinanceKind::RideConstruction {
        before.cash_cents().checked_sub(amount)
    } else {
        before.cash_cents().checked_add(amount)
    };
    if cash != Some(park.cash_cents())
        || entry.id != before.finance_ledger().next_entry_id
        || entry.tick != before.ticks()
        || entry.kind != kind
        || entry.ride != Some(id)
        || entry.service.is_some()
        || entry.amount_cents != amount
        || entry.balance_cents != park.cash_cents()
        || park.finance_ledger().next_entry_id != before.finance_ledger().next_entry_id + 1
        || park.guests() != before.guests()
        || park.ticks() != before.ticks()
        || park.ride() != before.ride()
        || park.train() != before.train()
    {
        return Err("construction command changed unrelated activity or recorded the wrong owner, amount or balance".into());
    }
    super::ride_choice::check(park)
}

pub(crate) fn build(park: &mut Park) -> Result<RideId, String> {
    let before = park.clone();
    if park
        .carousel_build_quote(super::ride_choice::ORIGIN, 8)
        .map_err(|e| e.to_string())?
        != BUILD
        || *park != before
    {
        return Err("construction quote changed the park or differs from $300".into());
    }
    let id = park
        .place_carousel(super::ride_choice::ORIGIN, 8)
        .map_err(|e| e.to_string())?;
    let body = park
        .carousel(id)
        .ok_or("built ride lacks retained definition")?;
    if body.construction_cost_cents != BUILD
        || body.demolition_refund_cents != 0
        || body.objects.len() != 9
    {
        return Err("built body did not retain its exact paid basis".into());
    }
    receipt(park, &before, id, FinanceKind::RideConstruction, BUILD)?;
    super::ride_choice::bytes(park)?;
    Ok(id)
}

pub(crate) fn demolish(park: &mut Park, id: RideId) -> Result<(), String> {
    let before = park.clone();
    let body = before.carousel(id).ok_or("missing demolition owner")?;
    if park
        .carousel_demolition_quote(id)
        .map_err(|e| e.to_string())?
        != REFUND
        || *park != before
    {
        return Err("demolition quote changed state or did not return half the paid basis".into());
    }
    // Exercise the same object-removal route used by the mouse tool, on a corner.
    park.remove_object(*body.objects.last().ok_or("missing body corner")?)
        .ok_or("body demolition rejected")?;
    let removed = park.carousel(id).unwrap();
    if !removed.objects.is_empty()
        || removed.construction_cost_cents != BUILD
        || removed.demolition_refund_cents != REFUND
        || body
            .objects
            .iter()
            .any(|object| park.object(*object).is_some())
        || removed.ride.entrance.is_some()
        || removed.ride.exit.is_some()
        || removed.ride.revenue_cents != body.ride.revenue_cents
        || removed.ride.rating_total != body.ride.rating_total
    {
        return Err(
            "demolition did not remove the entire body/gates while retaining finances".into(),
        );
    }
    for queue in before
        .objects()
        .iter()
        .filter(|o| o.queue_owner == Some(id))
    {
        if park.object(queue.id) != Some(queue) {
            return Err("body demolition changed a retained queue path".into());
        }
    }
    receipt(park, &before, id, FinanceKind::RideDemolitionRefund, REFUND)?;
    let after = park.clone();
    if park.remove_carousel(id).is_ok()
        || park.carousel_demolition_quote(id).is_ok()
        || park.remove_object(body.objects[0]).is_some()
        || *park != after
    {
        return Err("repeated demolition refunded twice or changed history".into());
    }
    super::ride_choice::bytes(park)?;
    Ok(())
}

pub(crate) fn exhaust_build_funds(park: &mut Park) -> Result<(), String> {
    let mut previous = park.carousels().last().map(|r| r.ride.id);
    while park.cash_cents() >= BUILD {
        let cash = park.cash_cents();
        let id = build(park)?;
        if previous.is_some_and(|old| id <= old) {
            return Err("rebuilding reused a ride identity".into());
        }
        demolish(park, id)?;
        if park.cash_cents() != cash - (BUILD - REFUND) {
            return Err("build/remove cycle generated money".into());
        }
        previous = Some(id);
    }
    let frozen = park.clone();
    for result in [
        park.carousel_build_quote(super::ride_choice::ORIGIN, 8),
        park.place_carousel(super::ride_choice::ORIGIN, 8)
            .map(|id| id.0),
    ] {
        if !matches!(result, Err(CarouselError::InsufficientFunds { required_cents: BUILD, available_cents }) if available_cents == frozen.cash_cents())
        {
            return Err(
                "unaffordable placement did not reject with the actual available cash".into(),
            );
        }
    }
    if *park != frozen {
        return Err("rejected placement changed cash, objects, IDs, guests, RNG or ledger".into());
    }
    super::ride_choice::bytes(park)?;
    Ok(())
}

fn replay(park: &mut Park) -> Result<(), String> {
    let started = park.ticks();
    let carousel = park
        .carousels()
        .iter()
        .find(|r| !r.objects.is_empty())
        .ok_or("primary lacks active paid body")?
        .ride
        .id;
    let before = park.clone();
    let corner = park.carousel(carousel).unwrap().objects[8];
    if park.carousel_demolition_quote(carousel) != Err(CarouselError::Occupied)
        || park.remove_carousel(carousel) != Err(CarouselError::Occupied)
        || park.remove_object(corner).is_some()
        || *park != before
    {
        return Err("occupied removal changed paid passengers, money or park state".into());
    }
    park.set_train_open(false);
    park.set_ride_open(carousel, false)
        .map_err(|e| e.to_string())?;
    super::ride_choice::seek(
        park,
        |p| !p.guests().iter().any(|g| g.ride_visit.is_some()),
        "all passengers return before refund",
    )?;
    demolish(park, carousel)?;
    for id in park
        .objects()
        .iter()
        .filter(|o| o.queue_owner == Some(carousel))
        .map(|o| o.id)
        .collect::<Vec<_>>()
    {
        park.remove_object(id)
            .ok_or("empty retained queue removal failed")?;
    }
    let rebuilt = build(park)?;
    if rebuilt <= carousel {
        return Err("rebuilt body did not allocate a fresh identity".into());
    }
    // Unreachable food facilities generate enough real expenses to archive the
    // construction and first refund; their existing independent tick oracle is used.
    super::finance::install_auxiliaries(park)?;
    for _ in 0..4_320 {
        let old = park.clone();
        park.advance(Speed::Normal);
        super::finance::check_tick(
            old.services(),
            old.guests(),
            old.finance_ledger(),
            old.cash_cents(),
            park,
            0,
        )?;
    }
    super::finance::close_all(park)?;
    if park
        .finance_ledger()
        .archived_rides
        .iter()
        .map(|r| r.construction_cents)
        .sum::<u64>()
        != 2 * BUILD
        || park
            .finance_ledger()
            .archived_rides
            .iter()
            .map(|r| r.demolition_refunds_cents)
            .sum::<u64>()
            != REFUND
    {
        return Err("construction and refund did not survive attributed ledger rollover".into());
    }
    let restored =
        Park::from_save_bytes(&super::ride_choice::bytes(park)?).map_err(|e| e.to_string())?;
    *park = restored;
    demolish(park, rebuilt)?;
    exhaust_build_funds(park)?;
    if park.ticks() >= started + REPLAY_TICKS {
        return Err("construction replay exceeded bounded continuation window".into());
    }
    while park.ticks() < started + REPLAY_TICKS {
        park.advance(Speed::Normal);
        super::ride_choice::check(park)?;
        if (park.ticks() - started).is_multiple_of(997) {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    super::ride_choice::bytes(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let (coaster, carousel) = super::ride_choice::install(&mut park)?;
    if park.cash_cents() != 70_000
        || park.carousel(carousel).unwrap().construction_cost_cents != BUILD
    {
        return Err("initial carousel did not charge exactly $300".into());
    }
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
        |p| {
            super::ride_choice::paid_running(p, coaster)
                && super::ride_choice::paid_running(p, carousel)
        },
        "paid occupied construction primary",
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
    )?;
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState {
        finance_open: true,
        rides_open: true,
        selected_ride: Some(RideId(999)),
        ..Default::default()
    };
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || view.finance_open
        || view.rides_open
        || view.selected_ride.is_some()
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
    {
        return Err("construction primary or paused/reset load differs".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("construction replay differs after fresh-process reload".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))? {
        return Err("paid construction backup lost its basis".into());
    }
    let frozen = park.clone();
    park.advance(Speed::Paused);
    if park != frozen
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("paused verification changed park or original save slots".into());
    }
    write_new(&store.directory().join("verification-construction.txt"), format!("Parkbound {}\nExact paid-body backup and occupied primary: PASS\nFresh-process continuation: PASS ({REPLAY_TICKS} ticks)\n$300 construction / $150 single refund / fresh-ID rebuilding: PASS\nPrecommit quotes, occupied protection and insufficient-funds atomic rejection: PASS\nBody/gate removal with retained queue, revenue and review history: PASS\nIndependent owner transaction amounts, wallet isolation and running balances: PASS\nConstruction/refund archive rollover and saved paid basis: PASS\nPaused/reset strict15 loading and unchanged source slots: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn construction_commands_continue_with_exact_cash_after_fresh_load() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound construction é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-construction", "verify-construction"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    directory.clone().into_os_string()
                ]),
                Some(AppExit::Success)
            );
        }
        assert!(directory.join("verification-construction.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }
}
