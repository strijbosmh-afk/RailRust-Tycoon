//! Independent transaction, rollover and fresh-process operating-cost proof.
use super::*;
use park_sim::{
    FINANCE_LEDGER_CAPACITY, FinanceEntry, FinanceKind, FinanceLedger, Guest, PathDirection,
    SERVICE_UPKEEP_TICKS, SERVICE_USE_TICKS, Service, ServiceId, ServiceKind, ServiceVisitPhase,
};
use std::collections::BTreeMap;

const REPLAY_TICKS: u64 = 24_000;
const SEEK_LIMIT: usize = 24_000;

// Contract amounts, intentionally independent of the simulation cost methods.
fn upkeep_cost(kind: ServiceKind) -> u64 {
    match kind {
        ServiceKind::Food => 30,
        ServiceKind::Drink => 20,
        ServiceKind::Toilet => 10,
    }
}
fn supply_cost(kind: ServiceKind) -> u64 {
    match kind {
        ServiceKind::Food => 60,
        ServiceKind::Drink => 25,
        ServiceKind::Toilet => 10,
    }
}
fn income(kind: FinanceKind) -> bool {
    matches!(
        kind,
        FinanceKind::LoanBorrow
            | FinanceKind::RideRevenue
            | FinanceKind::ServiceRevenue
            | FinanceKind::RideDemolitionRefund
            | FinanceKind::ServiceDemolitionRefund
            | FinanceKind::TrackRefund
    )
}
fn total(entries: &[FinanceEntry], kind: FinanceKind, service: Option<ServiceId>) -> u64 {
    entries
        .iter()
        .filter(|e| e.kind == kind && (service.is_none() || e.service == service))
        .map(|e| e.amount_cents)
        .sum()
}

pub(crate) fn check(park: &Park) -> Result<(), String> {
    let ledger = park.finance_ledger();
    let stats = park.finance_stats();
    let loan = park.loan_stats();
    let ride = park.all_rides().map(|r| r.revenue_cents).sum::<u64>();
    let revenue: u64 = park.services().iter().map(|s| s.revenue_cents).sum();
    let upkeep: u64 = park.services().iter().map(|s| s.upkeep_cents).sum::<u64>()
        + park
            .all_rides()
            .map(|r| park.ride_running_costs(r.id).unwrap().upkeep_cents)
            .sum::<u64>();
    let supplies: u64 = park.services().iter().map(|s| s.supplies_cents).sum();
    let construction: u64 = park
        .carousels()
        .iter()
        .map(|r| r.construction_cost_cents)
        .sum::<u64>()
        + park
            .services()
            .iter()
            .map(|s| s.construction_cost_cents)
            .sum::<u64>()
        + park.track_economy().construction_cents;
    let refunds: u64 = park
        .carousels()
        .iter()
        .map(|r| r.demolition_refund_cents)
        .sum::<u64>()
        + park
            .services()
            .iter()
            .map(|s| s.demolition_refund_cents)
            .sum::<u64>()
        + park.track_economy().refund_cents;
    if stats.cash_cents != park.cash_cents()
        || stats.ride_revenue_cents != ride
        || stats.service_revenue_cents != revenue
        || stats.upkeep_cents != upkeep
        || stats.supplies_cents != supplies
        || stats.construction_cents != construction
        || stats.demolition_refunds_cents != refunds
        || stats.total_costs_cents != upkeep + supplies + construction + loan.interest_paid_cents
        || i128::from(stats.net_profit_cents)
            != i128::from(ride) + i128::from(revenue) + i128::from(refunds)
                - i128::from(upkeep)
                - i128::from(supplies)
                - i128::from(construction)
                - i128::from(loan.interest_paid_cents)
        || stats.retained_entries != ledger.entries.len()
        || stats.archived_entries != ledger.archived_entries
        || ledger.entries.len() > FINANCE_LEDGER_CAPACITY
        || ledger.next_entry_id != ledger.archived_entries + ledger.entries.len() as u64 + 1
    {
        return Err("finance summary or bounded sequence differs from authoritative totals".into());
    }
    let per_minute: u64 = park
        .services()
        .iter()
        .filter(|s| s.object.is_some() && s.open)
        .map(|s| upkeep_cost(s.kind) * 1800 / u64::from(SERVICE_UPKEEP_TICKS))
        .sum::<u64>()
        + park
            .all_rides()
            .filter(|r| park.ride_is_open(r.id))
            .map(|r| {
                u64::from(
                    park.ride_running_costs(r.id)
                        .unwrap()
                        .cost_per_interval_cents,
                ) * 2
            })
            .sum::<u64>();
    if stats.open_upkeep_per_minute_cents != per_minute {
        return Err("open upkeep forecast differs from live services".into());
    }
    let archived = &ledger.archived;
    let mut balance = i128::from(ledger.opening_cash_cents)
        + i128::from(archived.ride_revenue_cents)
        + i128::from(archived.service_revenue_cents)
        + i128::from(archived.demolition_refunds_cents)
        - i128::from(archived.construction_cents)
        - i128::from(archived.upkeep_cents)
        - i128::from(archived.supplies_cents)
        + i128::from(archived.loan_borrowed_cents)
        - i128::from(archived.loan_repaid_cents)
        - i128::from(archived.loan_interest_cents);
    let mut tick = ledger.opening_tick.max(ledger.archived_through_tick);
    for (index, entry) in ledger.entries.iter().enumerate() {
        if entry.id != ledger.archived_entries + index as u64 + 1
            || entry.amount_cents == 0
            || entry.tick < tick
            || entry.tick > park.ticks()
            || (matches!(
                entry.kind,
                FinanceKind::LoanBorrow | FinanceKind::LoanRepayment | FinanceKind::LoanInterest
            ) && (entry.ride.is_some() || entry.service.is_some()))
            || (!matches!(
                entry.kind,
                FinanceKind::LoanBorrow | FinanceKind::LoanRepayment | FinanceKind::LoanInterest
            ) && (matches!(
                entry.kind,
                FinanceKind::RideRevenue
                    | FinanceKind::RideUpkeep
                    | FinanceKind::RideConstruction
                    | FinanceKind::RideDemolitionRefund
                    | FinanceKind::TrackConstruction
                    | FinanceKind::TrackRefund
            ) != entry.ride.is_some()
                || entry.service.is_none() != entry.ride.is_some()))
            || entry.ride.is_some_and(|id| park.ride_by_id(id).is_none())
            || entry.service.is_some_and(|id| park.service(id).is_none())
        {
            return Err(
                "finance retained entry identity, chronology or reference is invalid".into(),
            );
        }
        balance += if income(entry.kind) {
            i128::from(entry.amount_cents)
        } else {
            -i128::from(entry.amount_cents)
        };
        if balance != i128::from(entry.balance_cents) {
            return Err(
                "finance entry running balance disagrees with opening plus archived history".into(),
            );
        }
        tick = entry.tick;
    }
    if archived.loan_borrowed_cents + total(&ledger.entries, FinanceKind::LoanBorrow, None)
        != loan.borrowed_cents
        || archived.loan_repaid_cents + total(&ledger.entries, FinanceKind::LoanRepayment, None)
            != loan.repaid_cents
        || archived.loan_interest_cents + total(&ledger.entries, FinanceKind::LoanInterest, None)
            != loan.interest_paid_cents
        || balance != i128::from(park.cash_cents())
        || ledger.opening_ride_revenue_cents
            + archived.ride_revenue_cents
            + total(&ledger.entries, FinanceKind::RideRevenue, None)
            != ride
        || archived.upkeep_cents
            + total(&ledger.entries, FinanceKind::ServiceUpkeep, None)
            + total(&ledger.entries, FinanceKind::RideUpkeep, None)
            != upkeep
        || archived.supplies_cents + total(&ledger.entries, FinanceKind::ServiceSupplies, None)
            != supplies
        || archived.construction_cents
            + total(&ledger.entries, FinanceKind::RideConstruction, None)
            + total(&ledger.entries, FinanceKind::ServiceConstruction, None)
            + total(&ledger.entries, FinanceKind::TrackConstruction, None)
            != construction
        || archived.demolition_refunds_cents
            + total(&ledger.entries, FinanceKind::RideDemolitionRefund, None)
            + total(&ledger.entries, FinanceKind::ServiceDemolitionRefund, None)
            + total(&ledger.entries, FinanceKind::TrackRefund, None)
            != refunds
    {
        return Err(
            "ledger opening/archive/retained amounts do not reconstruct current cash and totals"
                .into(),
        );
    }
    if let Some(coaster) = park.ride() {
        let archived = ledger.archived_rides.iter().find(|r| r.ride == coaster.id);
        for (kind, archived_amount, expected) in [
            (
                FinanceKind::TrackConstruction,
                archived.map_or(0, |r| r.construction_cents),
                park.track_economy().construction_cents,
            ),
            (
                FinanceKind::TrackRefund,
                archived.map_or(0, |r| r.demolition_refunds_cents),
                park.track_economy().refund_cents,
            ),
        ] {
            let recent: u64 = ledger
                .entries
                .iter()
                .filter(|e| e.kind == kind && e.ride == Some(coaster.id))
                .map(|e| e.amount_cents)
                .sum();
            if archived_amount + recent != expected {
                return Err("track construction/refund amounts differ from coaster-attributed ledger history".into());
            }
        }
    }
    for carousel in park.carousels() {
        let archived = ledger
            .archived_rides
            .iter()
            .find(|r| r.ride == carousel.ride.id);
        for (kind, archived_amount, expected) in [
            (
                FinanceKind::RideConstruction,
                archived.map_or(0, |r| r.construction_cents),
                carousel.construction_cost_cents,
            ),
            (
                FinanceKind::RideDemolitionRefund,
                archived.map_or(0, |r| r.demolition_refunds_cents),
                carousel.demolition_refund_cents,
            ),
        ] {
            let recent: u64 = ledger
                .entries
                .iter()
                .filter(|e| e.kind == kind && e.ride == Some(carousel.ride.id))
                .map(|e| e.amount_cents)
                .sum();
            if archived_amount + recent != expected {
                return Err(
                    "ride construction/refund totals disagree with owner-attributed ledger history"
                        .into(),
                );
            }
        }
    }
    for service in park.services() {
        let opening = ledger
            .opening_services
            .iter()
            .find(|s| s.service == service.id)
            .map_or(0, |s| s.revenue_cents);
        let archived = ledger
            .archived_services
            .iter()
            .find(|s| s.service == service.id);
        let old_revenue = archived.map_or(0, |s| s.revenue_cents);
        let old_upkeep = archived.map_or(0, |s| s.upkeep_cents);
        let old_supplies = archived.map_or(0, |s| s.supplies_cents);
        let old_construction = archived.map_or(0, |s| s.construction_cents);
        let old_refunds = archived.map_or(0, |s| s.demolition_refunds_cents);
        if opening
            + old_revenue
            + total(
                &ledger.entries,
                FinanceKind::ServiceRevenue,
                Some(service.id),
            )
            != service.revenue_cents
            || old_upkeep
                + total(
                    &ledger.entries,
                    FinanceKind::ServiceUpkeep,
                    Some(service.id),
                )
                != service.upkeep_cents
            || old_supplies
                + total(
                    &ledger.entries,
                    FinanceKind::ServiceSupplies,
                    Some(service.id),
                )
                != service.supplies_cents
            || old_construction
                + total(
                    &ledger.entries,
                    FinanceKind::ServiceConstruction,
                    Some(service.id),
                )
                != service.construction_cost_cents
            || old_refunds
                + total(
                    &ledger.entries,
                    FinanceKind::ServiceDemolitionRefund,
                    Some(service.id),
                )
                != service.demolition_refund_cents
        {
            return Err("retained facility income/costs differ from its ledger history".into());
        }
        let derived = park.service_stats(service.id);
        if i128::from(derived.profit_cents)
            != i128::from(service.revenue_cents)
                - i128::from(service.upkeep_cents)
                - i128::from(service.supplies_cents)
                - i128::from(service.construction_cost_cents)
                + i128::from(service.demolition_refund_cents)
        {
            return Err("facility net profit differs from income minus both costs".into());
        }
    }
    Ok(())
}

/// Reconstruct allowed transactions independently in their specified tick order.
pub(crate) fn check_tick(
    old_ride_costs: &[(park_sim::RideId, bool, park_sim::RideRunningCosts)],
    old_services: &[Service],
    old_guests: &[Guest],
    old_ledger: &FinanceLedger,
    old_cash: u64,
    park: &Park,
    ride_income: u64,
) -> Result<(), String> {
    let mut services: Vec<_> = old_services.iter().collect();
    services.sort_by_key(|s| s.id);
    let mut cash = old_cash;
    let mut expected = Vec::new();
    for (id, open, costs) in old_ride_costs {
        let amount = u64::from(costs.cost_per_interval_cents);
        if *open && costs.upkeep_progress + 1 == park_sim::RIDE_UPKEEP_TICKS && cash >= amount {
            cash -= amount;
            expected.push(FinanceEntry {
                id: old_ledger.next_entry_id + expected.len() as u64,
                tick: park.ticks(),
                kind: FinanceKind::RideUpkeep,
                service: None,
                ride: Some(*id),
                amount_cents: amount,
                balance_cents: cash,
            });
        }
    }
    let mut costs = BTreeMap::new();
    let mut closed = BTreeMap::new();
    let mut progress = BTreeMap::new();
    let mut record = |kind, service, amount: u64, cash: &mut u64| -> Result<(), String> {
        if amount == 0 {
            return Ok(());
        }
        *cash = if income(kind) {
            cash.checked_add(amount)
        } else {
            cash.checked_sub(amount)
        }
        .ok_or("finance oracle balance overflow/overdraft")?;
        expected.push(FinanceEntry {
            id: old_ledger.next_entry_id + expected.len() as u64,
            tick: park.ticks(),
            kind,
            service,
            ride: if kind == FinanceKind::RideRevenue {
                park.ride().map(|r| r.id)
            } else {
                None
            },
            amount_cents: amount,
            balance_cents: *cash,
        });
        Ok(())
    };
    for service in &services {
        let mut p = service.upkeep_progress;
        let mut shut = service.closed_for_funds;
        let mut upkeep = 0;
        if service.object.is_some() && service.open {
            if p + 1 == SERVICE_UPKEEP_TICKS {
                let due = upkeep_cost(service.kind);
                if cash >= due {
                    record(FinanceKind::ServiceUpkeep, Some(service.id), due, &mut cash)?;
                    upkeep = due;
                    p = 0;
                } else {
                    shut = true;
                }
            } else {
                p += 1;
            }
        }
        costs.insert(service.id, (upkeep, 0));
        closed.insert(service.id, shut);
        progress.insert(service.id, p);
    }
    for service in &services {
        let completed = old_guests.iter().find_map(|g| {
            g.service_visit
                .as_ref()
                .filter(|v| {
                    v.service == service.id
                        && v.phase == ServiceVisitPhase::Using
                        && v.progress + 1 == SERVICE_USE_TICKS
                })
                .map(|v| (g, v))
        });
        if let Some((old, visit)) = completed
            && service.open
            && !closed[&service.id]
        {
            let due = supply_cost(service.kind);
            let new = park
                .guests()
                .iter()
                .find(|g| g.id == old.id)
                .ok_or("finance tick lost a guest")?;
            if cash + u64::from(visit.price_cents) >= due {
                if new.services_used != old.services_used + 1 {
                    return Err("affordable completed use did not transact".into());
                }
                record(
                    FinanceKind::ServiceRevenue,
                    Some(service.id),
                    u64::from(visit.price_cents),
                    &mut cash,
                )?;
                record(
                    FinanceKind::ServiceSupplies,
                    Some(service.id),
                    due,
                    &mut cash,
                )?;
                costs.get_mut(&service.id).unwrap().1 = due;
            } else {
                closed.insert(service.id, true);
                if new.services_used != old.services_used {
                    return Err("unaffordable supplies manufactured a completed use".into());
                }
            }
        }
    }
    record(FinanceKind::RideRevenue, None, ride_income, &mut cash)?;
    for old in old_services {
        let new = park
            .service(old.id)
            .ok_or("finance tick erased a service")?;
        let (upkeep, supplies) = costs[&old.id];
        if new.upkeep_cents != old.upkeep_cents + upkeep
            || new.supplies_cents != old.supplies_cents + supplies
            || new.upkeep_progress != progress[&old.id]
            || new.closed_for_funds != closed[&old.id]
            || new.open != (old.open && !closed[&old.id])
        {
            return Err("service cost, partial period or automatic closure differs from independent tick oracle".into());
        }
    }
    let mut retained = old_ledger.entries.clone();
    retained.extend(expected);
    let evicted = retained.len().saturating_sub(FINANCE_LEDGER_CAPACITY);
    let removed = &retained[..evicted];
    let ledger = park.finance_ledger();
    if ledger.entries != retained[evicted..]
        || ledger.next_entry_id
            != old_ledger.next_entry_id + (retained.len() - old_ledger.entries.len()) as u64
        || ledger.archived_entries != old_ledger.archived_entries + evicted as u64
        || ledger.archived_through_tick
            != removed
                .last()
                .map_or(old_ledger.archived_through_tick, |e| e.tick)
        || ledger.archived.ride_revenue_cents
            != old_ledger.archived.ride_revenue_cents
                + total(removed, FinanceKind::RideRevenue, None)
        || ledger.archived.service_revenue_cents
            != old_ledger.archived.service_revenue_cents
                + total(removed, FinanceKind::ServiceRevenue, None)
        || ledger.archived.upkeep_cents
            != old_ledger.archived.upkeep_cents
                + total(removed, FinanceKind::ServiceUpkeep, None)
                + total(removed, FinanceKind::RideUpkeep, None)
        || ledger.archived.supplies_cents
            != old_ledger.archived.supplies_cents
                + total(removed, FinanceKind::ServiceSupplies, None)
        || ledger.archived.construction_cents
            != old_ledger.archived.construction_cents
                + total(removed, FinanceKind::RideConstruction, None)
                + total(removed, FinanceKind::ServiceConstruction, None)
        || ledger.archived.demolition_refunds_cents
            != old_ledger.archived.demolition_refunds_cents
                + total(removed, FinanceKind::RideDemolitionRefund, None)
                + total(removed, FinanceKind::ServiceDemolitionRefund, None)
        || ledger.opening_tick != old_ledger.opening_tick
        || ledger.opening_cash_cents != old_ledger.opening_cash_cents
        || ledger.opening_ride_revenue_cents != old_ledger.opening_ride_revenue_cents
        || ledger.opening_services != old_ledger.opening_services
        || cash != park.cash_cents()
    {
        return Err("per-tick ledger entries, rollover, opening baseline or cash differ from expected transactions".into());
    }
    check(park)
}

pub(crate) fn advance_checked(park: &mut Park) -> Result<(), String> {
    super::services::advance_checked(park)
}
pub(crate) fn seek(
    park: &mut Park,
    predicate: impl Fn(&Park) -> bool,
    label: &str,
) -> Result<(), String> {
    for _ in 0..SEEK_LIMIT {
        if predicate(park) {
            return Ok(());
        }
        advance_checked(park)?;
    }
    Err(format!(
        "finance did not reach {label}: tick={}, stats={:?}, services={:?}",
        park.ticks(),
        park.finance_stats(),
        park.services()
    ))
}
pub(crate) fn bytes(park: &Park) -> Result<Vec<u8>, String> {
    check(park)?;
    super::services::bytes(park)
}

pub(crate) fn install_auxiliaries(park: &mut Park) -> Result<Vec<ServiceId>, String> {
    let mut result = Vec::new();
    // Keep these genuine paid facilities within the fixture's available budget.
    let count = ((park.cash_cents().saturating_sub(12_000)) / 12_000).min(4) as u32;
    if count == 0 {
        return Err("finance fixture cannot afford an auxiliary facility".into());
    }
    for x in 1..=count {
        let pos = TilePos { x, y: 24 };
        let height = park.tile(pos).unwrap().height;
        result.push(
            park.place_service(pos, height, PathDirection::North, ServiceKind::Food)
                .map_err(|e| e.to_string())?,
        );
    }
    Ok(result)
}
pub(crate) fn close_all(park: &mut Park) -> Result<(), String> {
    let ids: Vec<_> = park
        .services()
        .iter()
        .filter(|s| s.object.is_some())
        .map(|s| s.id)
        .collect();
    for id in ids {
        park.set_service_open(id, false)
            .map_err(|e| e.to_string())?;
    }
    check(park)
}
pub(crate) fn install_income_ride(park: &mut Park) -> Result<(), String> {
    // Operating a ride now requires float before its first departure. Recover
    // that float from a real paid auxiliary refund, never from invented cash.
    if park.cash_cents() < 500 {
        let object = park
            .services()
            .iter()
            .filter(|s| {
                s.object.is_some()
                    && !s.open
                    && s.kind == ServiceKind::Food
                    && s.construction_cost_cents > 0
            })
            .find_map(|s| {
                s.object.filter(|object| {
                    park.object(*object)
                        .is_some_and(|o| o.pos.y == 24 && o.pos.x <= 5)
                })
            })
            .ok_or("no paid auxiliary available to fund ride operation")?;
        park.remove_object(object)
            .ok_or("auxiliary refund failed")?;
    }
    super::journey::install(park)?;
    super::economy::set_price_checked(park, 200)?;
    park.set_train_open(true);
    Ok(())
}

fn continue_finance(park: &mut Park) -> Result<(), String> {
    let start = park.ticks();
    let initial_revenue = park.ride().unwrap().revenue_cents;
    let drink = park
        .services()
        .iter()
        .find(|s| s.kind == ServiceKind::Drink && s.object.is_some())
        .unwrap()
        .id;
    let auxiliary = install_auxiliaries(park)?;
    super::facility_construction::spend_down(park)?;
    let removed_id = auxiliary[0];
    let mut stage = 0;
    let mut restored_count = 0;
    let mut transitions = Vec::new();
    for elapsed in 0..REPLAY_TICKS {
        if elapsed == 23 {
            park.set_service_open(drink, false)
                .map_err(|e| e.to_string())?;
        }
        if elapsed == 113 {
            park.set_service_open(drink, true)
                .map_err(|e| e.to_string())?;
        }
        if elapsed == 203 {
            let old = park.service(removed_id).unwrap().clone();
            park.remove_object(old.object.unwrap())
                .ok_or("auxiliary demolition rejected")?;
            let new = park.service(removed_id).unwrap();
            if new.upkeep_cents != old.upkeep_cents
                || new.supplies_cents != old.supplies_cents
                || new.revenue_cents != old.revenue_cents
            {
                return Err("demolition erased financial history".into());
            }
        }
        if stage == 0 && park.services().iter().any(|s| s.closed_for_funds) {
            close_all(park)?;
            install_income_ride(park)?;
            stage = 1;
            transitions.push((stage, elapsed));
        } else if stage == 1 && park.ride().unwrap().revenue_cents > initial_revenue {
            let before = park.clone();
            park.set_service_price(drink, 100)
                .map_err(|e| e.to_string())?;
            park.set_service_open(drink, true)
                .map_err(|e| e.to_string())?;
            if park.cash_cents() != before.cash_cents()
                || park.service(drink).unwrap().closed_for_funds
            {
                return Err("manual funded reopening changed cash or retained funds flag".into());
            }
            restored_count = park.service(drink).unwrap().served;
            stage = 2;
            transitions.push((stage, elapsed));
        } else if stage == 2 && park.service(drink).unwrap().served > restored_count {
            stage = 3;
            transitions.push((stage, elapsed));
        }
        advance_checked(park)?;
        if elapsed % 997 == 0 {
            bytes(park)?;
        }
    }
    if stage != 3
        || park.ticks() != start + REPLAY_TICKS
        || park.finance_ledger().archived_entries == 0
        || park.service(removed_id).unwrap().object.is_some()
        || park.finance_stats().upkeep_cents == 0
        || park.finance_stats().supplies_cents == 0
    {
        return Err(format!(
            "finance replay incomplete: stage={stage}, transitions={transitions:?}, stats={:?}",
            park.finance_stats()
        ));
    }
    bytes(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    park.set_train_open(false);
    super::services::install(&mut park)?;
    seek(
        &mut park,
        |p| {
            super::services::guest_in_phase(p, ServiceVisitPhase::Using).is_some()
                && p.services()
                    .iter()
                    .any(|s| s.upkeep_progress > 0 && s.upkeep_progress < SERVICE_UPKEEP_TICKS - 20)
        },
        "occupied midperiod backup",
    )?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &bytes(&park)?,
    )?;
    for _ in 0..17 {
        advance_checked(&mut park)?;
    }
    park.set_terrain_height(TilePos { x: 3, y: 24 }, 1)
        .map_err(|e| e.to_string())?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &bytes(&park)?,
    )?;
    continue_finance(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &bytes(&park)?,
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
        finance_page: 3,
        ..Default::default()
    };
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || !view.terrain_custom
        || view.finance_open
        || view.finance_page != 0
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
    {
        return Err(
            "finance primary did not load exactly paused/Custom/Inspect with panel reset".into(),
        );
    }
    continue_finance(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("finance fresh-process future differs".into());
    }
    view.finance_open = true;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
        || view.finance_open
        || !park.services().iter().any(|s| s.upkeep_progress > 0)
    {
        return Err("finance backup lost its partial period or paused state".into());
    }
    let paused = park.clone();
    park.advance(Speed::Paused);
    if park != paused {
        return Err("paused finance state changed".into());
    }
    for _ in 0..180 {
        advance_checked(&mut park)?;
    }
    bytes(&park)?;
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("finance verification changed original slots".into());
    }
    write_new(&store.directory().join("verification-finance.txt"), format!("Parkbound {}\nExact midperiod primary/backup and fresh-process continuation: PASS ({REPLAY_TICKS} ticks)\nIndependent ordered upkeep, sale, supplies and ride-entry amounts/balances: PASS\nBounded ledger rollover and first retained balance from opening plus archive: PASS\nClosure/reopen, retained demolition costs, automatic funds closure and earned-income recovery: PASS\nWallet/receipts/expenses/park cash identity: PASS\nPaused load, Finance reset, current-schema roundtrips and unchanged slots: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finance_modes_preserve_midperiod_costs_ledger_and_natural_cash_recovery() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound finance é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let args = |mode: &str| {
            vec![
                OsString::from("--save-check"),
                OsString::from(mode),
                OsString::from("--save-dir"),
                directory.clone().into_os_string(),
            ]
        };
        assert_eq!(run(args("write-finance")), Some(AppExit::Success));
        assert_eq!(run(args("write-finance")), Some(AppExit::error()));
        assert_eq!(run(args("verify-finance")), Some(AppExit::Success));
        assert!(directory.join("verification-finance.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }
}
