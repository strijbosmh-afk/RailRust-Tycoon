//! Paid facilities, cancellation refunds and mixed legacy/current continuation.
use super::*;
use park_sim::{
    FinanceKind, PathDirection, PlacementError, ServiceId, ServiceKind, ServiceVisitPhase,
};

const REPLAY_TICKS: u64 = 12_000;
pub(crate) const DRAIN_SITE: TilePos = TilePos { x: 5, y: 27 };
const LEGACY_SITE: TilePos = TilePos { x: 1, y: 26 };

pub(crate) fn cost(kind: ServiceKind) -> u64 {
    match kind {
        ServiceKind::Food => 12_000,
        ServiceKind::Drink => 8_000,
        ServiceKind::Toilet => 10_000,
    }
}

fn transaction(
    park: &Park,
    before: &Park,
    id: ServiceId,
    kind: FinanceKind,
    amount: u64,
) -> Result<(), String> {
    if amount == 0 {
        if park.cash_cents() != before.cash_cents()
            || park.finance_ledger() != before.finance_ledger()
        {
            return Err("free legacy removal created cash or a ledger transaction".into());
        }
    } else {
        let cash = if kind == FinanceKind::ServiceConstruction {
            before.cash_cents().checked_sub(amount)
        } else {
            before.cash_cents().checked_add(amount)
        };
        let entry = park
            .finance_ledger()
            .entries
            .last()
            .ok_or("facility command has no transaction")?;
        if cash != Some(park.cash_cents())
            || entry.id != before.finance_ledger().next_entry_id
            || entry.tick != before.ticks()
            || entry.kind != kind
            || entry.service != Some(id)
            || entry.ride.is_some()
            || entry.amount_cents != amount
            || entry.balance_cents != park.cash_cents()
            || park.finance_ledger().next_entry_id != before.finance_ledger().next_entry_id + 1
        {
            return Err(
                "facility command recorded an incorrect owner, amount, sign or resulting balance"
                    .into(),
            );
        }
    }
    if park.ticks() != before.ticks()
        || park.ride() != before.ride()
        || park.train() != before.train()
        || park.carousels() != before.carousels()
    {
        return Err("facility command advanced time or changed a ride".into());
    }
    super::ride_choice::check(park)
}

pub(crate) fn build(
    park: &mut Park,
    pos: TilePos,
    height: u8,
    kind: ServiceKind,
) -> Result<ServiceId, String> {
    let before = park.clone();
    let price = cost(kind);
    if park
        .service_build_quote(pos, height, PathDirection::North, kind)
        .map_err(|e| e.to_string())?
        != price
        || *park != before
    {
        return Err(
            "facility quote changed the park or differs from its fixed construction price".into(),
        );
    }
    let id = park
        .place_service(pos, height, PathDirection::North, kind)
        .map_err(|e| e.to_string())?;
    let service = park.service(id).ok_or("built facility lacks metadata")?;
    if service.construction_cost_cents != price
        || service.demolition_refund_cents != 0
        || service.object.is_none()
        || park.guests() != before.guests()
    {
        return Err("facility placement did not retain paid basis or changed guests".into());
    }
    transaction(park, &before, id, FinanceKind::ServiceConstruction, price)?;
    super::ride_choice::bytes(park)?;
    Ok(id)
}

pub(crate) fn demolish(park: &mut Park, id: ServiceId) -> Result<(), String> {
    let before = park.clone();
    let old = before.service(id).ok_or("missing demolition facility")?;
    let object = old.object.ok_or("facility is already removed")?;
    let refund = old.construction_cost_cents / 2;
    if park
        .service_demolition_quote(id)
        .map_err(|e| e.to_string())?
        != refund
        || *park != before
    {
        return Err(
            "facility demolition quote changed state or did not use actual paid basis".into(),
        );
    }
    park.remove_object(object)
        .ok_or("facility demolition rejected")?;
    let removed = park.service(id).ok_or("facility history was erased")?;
    if removed.object.is_some()
        || removed.open
        || removed.construction_cost_cents != old.construction_cost_cents
        || removed.demolition_refund_cents != refund
        || removed.revenue_cents != old.revenue_cents
        || removed.served != old.served
        || removed.upkeep_cents != old.upkeep_cents
        || removed.supplies_cents != old.supplies_cents
        || before
            .objects()
            .iter()
            .filter(|o| o.id != object)
            .any(|o| park.object(o.id) != Some(o))
        || park.objects().len() + 1 != before.objects().len()
    {
        return Err(
            "facility demolition changed public paths, other objects or retained financial history"
                .into(),
        );
    }
    for (old_guest, guest) in before.guests().iter().zip(park.guests()) {
        if guest.id != old_guest.id
            || guest.wallet_cents != old_guest.wallet_cents
            || guest.spent_cents != old_guest.spent_cents
            || guest.service_spent_cents != old_guest.service_spent_cents
            || guest.services_used != old_guest.services_used
            || guest.needs != old_guest.needs
            || guest.satisfaction != old_guest.satisfaction
            || guest.ride_history != old_guest.ride_history
            || park.guest_position(guest.id).map(|position| position.point)
                != before
                    .guest_position(old_guest.id)
                    .map(|position| position.point)
            || guest
                .service_visit
                .as_ref()
                .is_some_and(|v| v.service == id)
            || (old_guest
                .service_visit
                .as_ref()
                .is_none_or(|v| v.service != id)
                && guest != old_guest)
        {
            return Err(
                "unfinished facility removal charged, relieved or changed an unrelated guest"
                    .into(),
            );
        }
    }
    transaction(
        park,
        &before,
        id,
        FinanceKind::ServiceDemolitionRefund,
        refund,
    )?;
    let removed_state = park.clone();
    if park.service_demolition_quote(id).is_ok()
        || park.remove_object(object).is_some()
        || *park != removed_state
    {
        return Err("repeated facility removal refunded or changed state twice".into());
    }
    super::ride_choice::bytes(park)?;
    Ok(())
}

pub(crate) fn spend_down(park: &mut Park) -> Result<(), String> {
    let height = park.tile(DRAIN_SITE).ok_or("missing drain site")?.height;
    while park.cash_cents() >= 8_000 {
        let cash = park.cash_cents();
        let old_id = park.services().last().map(|s| s.id);
        let id = build(park, DRAIN_SITE, height, ServiceKind::Drink)?;
        if old_id.is_some_and(|old| id <= old) {
            return Err("replacement reused a facility ID".into());
        }
        demolish(park, id)?;
        if park.cash_cents() != cash - 4_000 {
            return Err("facility build/demolish cycle created money".into());
        }
    }
    let before = park.clone();
    if !matches!(
        park.service_build_quote(DRAIN_SITE, height, PathDirection::North, ServiceKind::Drink),
        Err(PlacementError::InsufficientFunds { .. })
    ) || !matches!(
        park.place_service(DRAIN_SITE, height, PathDirection::North, ServiceKind::Drink),
        Err(PlacementError::InsufficientFunds { .. })
    ) || *park != before
    {
        return Err(
            "unaffordable facility quote/command changed cash, IDs, objects, RNG, guests or ledger"
                .into(),
        );
    }
    Ok(())
}

/// Synthetic schema-11 baseline, explicitly separate from authentic old-binary
/// migration tests. Its only facility was free under those older game rules.
fn mixed_legacy_baseline() -> Result<(Park, ServiceId), String> {
    let mut park = demonstration_park();
    park.set_train_open(false);
    let cash = park.cash_cents();
    let ledger = serde_json::to_value(park.finance_ledger()).map_err(|e| e.to_string())?;
    let height = park.tile(LEGACY_SITE).unwrap().height;
    let id = park
        .place_service(LEGACY_SITE, height, PathDirection::North, ServiceKind::Food)
        .map_err(|e| e.to_string())?;
    let mut old: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    old["schema_version"] = 11.into();
    old["park"].as_object_mut().unwrap().remove("ride_costs");
    old["park"].as_object_mut().unwrap().remove("coaster_cars");
    old["park"].as_object_mut().unwrap().remove("ride_dispatch");
    old["content_version"] = 11.into();
    old["park"].as_object_mut().unwrap().remove("track_draft");
    old["park"].as_object_mut().unwrap().remove("track_economy");
    old["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_operating_settings");
    old["park"]["cash_cents"] = cash.into();
    old["park"]["finances"] = ledger;
    // This synthetic pre-loan baseline has never borrowed. Remove only zero
    // financing defaults; authentic old-binary migration uses original bytes.
    if park.loan_stats().borrowed_cents != 0 || park.loan_stats().interest_accrued_cents != 0 {
        return Err("synthetic legacy baseline unexpectedly contains financing".into());
    }
    old["park"].as_object_mut().unwrap().remove("loans");
    for field in [
        "loan_borrowed_cents",
        "loan_repaid_cents",
        "loan_interest_cents",
    ] {
        if old["park"]["finances"]["archived"][field] != 0 {
            return Err("synthetic legacy baseline has nonzero archived financing".into());
        }
        old["park"]["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove(field);
    }
    for service in old["park"]["services"].as_array_mut().unwrap() {
        service
            .as_object_mut()
            .unwrap()
            .remove("construction_cost_cents");
        service
            .as_object_mut()
            .unwrap()
            .remove("demolition_refund_cents");
    }
    let park = Park::from_save_bytes(&serde_json::to_vec(&old).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if park.service(id).unwrap().construction_cost_cents != 0 || park.cash_cents() != cash {
        return Err("synthetic legacy free-facility baseline changed cash or invented cost".into());
    }
    Ok((park, id))
}

fn replay(park: &mut Park) -> Result<(), String> {
    let started = park.ticks();
    let guest = super::services::guest_in_phase(park, ServiceVisitPhase::Using)
        .ok_or("facility primary has no unfinished use")?;
    let id = super::services::guest(park, guest)
        .service_visit
        .as_ref()
        .unwrap()
        .service;
    let service = park.service(id).unwrap().clone();
    let object = *park.object(service.object.unwrap()).unwrap();
    demolish(park, id)?;
    let rebuilt = build(park, object.pos, object.height, service.kind)?;
    if rebuilt <= id {
        return Err("rebuilt paid facility reused its predecessor's ID".into());
    }
    let initial_served = park.service(rebuilt).unwrap().served;
    let legacy = park
        .services()
        .iter()
        .find(|s| s.object.is_some() && s.construction_cost_cents == 0)
        .ok_or("mixed fixture lost its free legacy service")?
        .id;
    // Keep the free legacy facility alive long enough to archive both owner types.
    let mut used = false;
    for elapsed in 0..4_320 {
        super::services::advance_checked(park)?;
        used |= park.service(rebuilt).unwrap().served > initial_served;
        if elapsed % 997 == 0 {
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    if !used
        || park
            .finance_ledger()
            .archived_services
            .iter()
            .find(|s| s.service == id)
            .is_none_or(|s| {
                s.construction_cents != cost(service.kind)
                    || s.demolition_refunds_cents != cost(service.kind) / 2
            })
    {
        return Err(
            "paid facility was not reused or archived construction/refund ownership was lost"
                .into(),
        );
    }
    demolish(park, legacy)?;
    super::finance::close_all(park)?;
    spend_down(park)?;
    while park.ticks() < started + REPLAY_TICKS {
        super::services::advance_checked(park)?;
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
    let (mut park, legacy) = mixed_legacy_baseline()?;
    for (pos, kind) in super::services::SITES {
        build(&mut park, pos, 8, kind)?;
    }
    if park.cash_cents() != 70_000
        || park.services().len() != 4
        || park.service(legacy).unwrap().construction_cost_cents != 0
    {
        return Err(
            "mixed legacy plus all three paid kinds charged the wrong starting total".into(),
        );
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    super::services::seek(
        &mut park,
        |p| super::services::guest_in_phase(p, ServiceVisitPhase::Using).is_some(),
        "unfinished paid use before demolition refund",
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
        services_open: true,
        finance_open: true,
        selected_service: Some(ServiceId(999)),
        ..Default::default()
    };
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || view.services_open
        || view.finance_open
        || view.selected_service.is_some()
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
    {
        return Err(
            "facility construction load did not restore exact state paused with panels reset"
                .into(),
        );
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("facility construction continuation diverged after fresh load".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))? {
        return Err("mixed free and paid facility backup lost cost basis".into());
    }
    let frozen = park.clone();
    park.advance(Speed::Paused);
    if park != frozen
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("paused facility verification changed state or original slots".into());
    }
    write_new(&store.directory().join("verification-facility-construction.txt"), format!("Parkbound {}\nExact occupied primary and mixed legacy/free plus paid-kind backup: PASS\nFresh-process continuation: PASS ({REPLAY_TICKS} ticks)\nFood $120, Drinks $80, Toilets $100; half of actually paid basis refunded once: PASS\nUnfinished-use demolition cancels without guest payment or need relief; public path retained: PASS\nFresh-ID rebuild and independent service-attributed amounts/balances: PASS\nPaid construction/refund archival and free legacy zero refund: PASS\nInsufficient-funds rejection preserves full state; repeat build/remove loses cash: PASS\nCurrent-schema reloads, paused/reset loading and unchanged original slots: PASS\nSynthetic11 free baseline is separate from authentic legacy-binary verification.\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn facility_construction_continues_mixed_paid_and_free_cost_basis_and_refunds() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound paid facilities é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in [
            "write-facility-construction",
            "verify-facility-construction",
        ] {
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
        assert!(
            directory
                .join("verification-facility-construction.txt")
                .is_file()
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
