//! Fresh-process fare locking, departure charging and cancellation accounting.
use super::*;
use park_sim::{
    GuestId, INITIAL_GUEST_WALLET_CENTS, INITIAL_PARK_CASH_CENTS, MAX_TICKET_PRICE_CENTS,
    RIDE_TRANSFER_TICKS, RideId, RideVisitPhase, SERVICE_USE_TICKS, ServiceVisitPhase,
};

const REPLAY_TICKS: u64 = 10_800;
const SEEK_LIMIT: usize = 18_000;

fn ride_id(park: &Park) -> Result<RideId, String> {
    park.ride()
        .map(|ride| ride.id)
        .ok_or("economy proof needs a ride".into())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Balances {
    guests: Vec<(GuestId, u32, u64)>,
    revenue: u64,
    cash: u64,
}

fn balances(park: &Park) -> Balances {
    Balances {
        guests: park
            .guests()
            .iter()
            .map(|guest| (guest.id, guest.wallet_cents, guest.spent_cents))
            .collect(),
        revenue: park.ride().unwrap().revenue_cents,
        cash: park.cash_cents(),
    }
}

/// Independent accounting checks shared by opt-in proof and capture fixtures.
pub(crate) fn require_accounting(park: &Park) -> Result<(), String> {
    super::journey::require_membership(park)?;
    let ride = ride_id(park)?;
    let stats = park.ride_economy_stats(ride);
    let mut spent = 0u64;
    let mut service_spent = 0u64;
    let mut services_used = 0u128;
    let mut reserved = 0u64;
    for guest in park.guests() {
        if u64::from(guest.wallet_cents) + guest.spent_cents
            != u64::from(INITIAL_GUEST_WALLET_CENTS)
        {
            return Err("guest wallet and lifetime spending do not conserve initial funds".into());
        }
        spent += guest.spent_cents;
        if guest.service_spent_cents > guest.spent_cents {
            return Err("service spending exceeds total guest spending".into());
        }
        service_spent += guest.service_spent_cents;
        services_used += u128::from(guest.services_used);
        if let Some(passenger) = guest
            .ride_visit
            .as_ref()
            .and_then(|visit| visit.passenger.as_ref())
        {
            if passenger.fare_cents > MAX_TICKET_PRICE_CENTS
                || passenger.fare_paid != passenger.departed
            {
                return Err(
                    "passenger locked fare or departure payment flag is inconsistent".into(),
                );
            }
            if !passenger.fare_paid {
                if passenger.fare_cents > guest.wallet_cents {
                    return Err("unpaid reserved fare exceeds that guest's wallet".into());
                }
                reserved += u64::from(passenger.fare_cents);
            }
        }
    }
    let actual = park.ride().unwrap();
    let affordable = park
        .guests()
        .iter()
        .filter(|guest| guest.wallet_cents >= actual.ticket_price_cents)
        .count();
    let service_revenue: u64 = park
        .services()
        .iter()
        .map(|service| service.revenue_cents)
        .sum();
    let service_count: u128 = park
        .services()
        .iter()
        .map(|service| u128::from(service.served))
        .sum();
    let costs: u64 = park
        .services()
        .iter()
        .map(|service| {
            service.upkeep_cents + service.supplies_cents + service.construction_cost_cents
        })
        .sum();
    let refunds = park
        .services()
        .iter()
        .map(|s| s.demolition_refund_cents)
        .sum::<u64>();
    if spent - service_spent != actual.revenue_cents
        || service_spent != service_revenue
        || services_used != service_count
        || u128::from(park.cash_cents()) + u128::from(costs)
            != u128::from(INITIAL_PARK_CASH_CENTS) + u128::from(spent) + u128::from(refunds)
        || stats.revenue_cents != spent - service_spent
        || stats.ticket_price_cents != actual.ticket_price_cents
        || stats.unaffordable_rejections != actual.unaffordable_rejections
        || stats.reserved_fares_cents != reserved
        || stats.guest_count != park.guests().len()
        || stats.affordable_guests != affordable
    {
        return Err(
            "ride revenue, park cash, reserved fares or affordability statistics disagree".into(),
        );
    }
    super::finance::check(park)
}

/// Conservation alone cannot detect an equal repeated debit/credit. Derive each
/// permitted payment from the same reserved ticket and an actual train departure.
pub(crate) fn advance_checked(park: &mut Park) -> Result<(), String> {
    let before = park.guests().to_vec();
    let services = park.services().to_vec();
    let ledger = park.finance_ledger().clone();
    let revenue = park.ride().unwrap().revenue_cents;
    let cash = park.cash_cents();
    let was_stopped = park.train().is_some_and(|train| train.speed == 0);
    park.advance(Speed::Normal);
    let started_moving = was_stopped && park.train().is_some_and(|train| train.speed > 0);
    if before.len() != park.guests().len() {
        return Err("economy tick changed the guest population".into());
    }
    let mut earned = 0u64;
    let mut service_earned = std::collections::BTreeMap::new();
    for (old, guest) in before.iter().zip(park.guests()) {
        if old.id != guest.id {
            return Err("economy tick changed guest identity/order".into());
        }
        let old_visit = old.ride_visit.as_ref();
        let new_visit = guest.ride_visit.as_ref();
        let same_ticket = old_visit.zip(new_visit).is_some_and(|(old, new)| {
            old.ride == new.ride && old.ticket.is_some() && old.ticket == new.ticket
        });
        let same_riding_ticket = same_ticket
            && old_visit.zip(new_visit).is_some_and(|(old, new)| {
                old.phase == RideVisitPhase::Riding && new.phase == RideVisitPhase::Riding
            });
        let old_passenger = old_visit.and_then(|visit| visit.passenger.as_ref());
        let new_passenger = new_visit.and_then(|visit| visit.passenger.as_ref());
        if same_ticket
            && let (Some(old), Some(new)) = (old_passenger, new_passenger)
            && (old.fare_cents != new.fare_cents || old.fare_paid && !new.fare_paid)
        {
            return Err(
                "existing passenger ticket changed its locked fare or reverted payment".into(),
            );
        }
        let charge = match (old_passenger, new_passenger) {
            (Some(old), Some(new)) if !old.fare_paid && new.fare_paid => {
                if !started_moving
                    || !same_riding_ticket
                    || old.departed
                    || !new.departed
                    || old.fare_cents != new.fare_cents
                {
                    return Err(
                        "fare settlement did not match a first departure at its locked price"
                            .into(),
                    );
                }
                old.fare_cents
            }
            (None, Some(new)) if new.fare_paid => {
                return Err("new passenger became paid without a prior reservation".into());
            }
            _ => 0,
        };
        let completed_service = old.service_visit.as_ref().filter(|visit| {
            visit.phase == ServiceVisitPhase::Using
                && visit.progress + 1 == SERVICE_USE_TICKS
                && guest.service_visit.is_none()
                && old.services_used.checked_add(1) == Some(guest.services_used)
        });
        let service_charge = if let Some(visit) = completed_service {
            if old.current != Some(visit.access)
                || guest.current != Some(visit.access)
                || old.next.is_some()
                || old.ride_visit.is_some()
                || guest.ride_visit.is_some()
                || !services.iter().any(|service| {
                    service.id == visit.service && service.object.is_some() && service.open
                })
            {
                return Err(
                    "service payment did not follow stationary use at its actual access path"
                        .into(),
                );
            }
            let earned = service_earned.entry(visit.service).or_insert((0u64, 0u64));
            earned.0 += 1;
            earned.1 += u64::from(visit.price_cents);
            visit.price_cents
        } else {
            if guest.services_used != old.services_used {
                return Err("service use counter changed without a full completed use".into());
            }
            0
        };
        if let (Some(old), Some(new)) = (&old.service_visit, &guest.service_visit)
            && old.service == new.service
            && old.phase == ServiceVisitPhase::Using
            && new.phase == ServiceVisitPhase::Using
            && old.price_cents != new.price_cents
        {
            return Err("active service changed its locked quote".into());
        }
        let total_charge = charge + service_charge;
        if old.wallet_cents.checked_sub(total_charge) != Some(guest.wallet_cents)
            || guest.spent_cents != old.spent_cents + u64::from(total_charge)
            || guest.service_spent_cents != old.service_spent_cents + u64::from(service_charge)
        {
            return Err(
                "guest payment delta differs from actual ride departure/service completion".into(),
            );
        }
        earned += u64::from(charge);
    }
    if services.len() != park.services().len() {
        return Err("simulation tick changed service records".into());
    }
    for (old, new) in services.iter().zip(park.services()) {
        let (count, money) = service_earned.get(&old.id).copied().unwrap_or_default();
        if old.id != new.id
            || new.served != old.served + count
            || new.revenue_cents != old.revenue_cents + money
        {
            return Err("service revenue/count delta differs from actual completed uses".into());
        }
    }
    if park.ride().unwrap().revenue_cents != revenue + earned {
        return Err("cash/revenue delta differs from actual per-guest departure payments".into());
    }
    super::finance::check_tick(&services, &before, &ledger, cash, park, earned)?;
    require_accounting(park)
}

pub(crate) fn set_price_checked(park: &mut Park, cents: u32) -> Result<(), String> {
    let ride = ride_id(park)?;
    let before = park.guests().to_vec();
    let money = balances(park);
    let ticks = park.ticks();
    let cache = park.path_cache_stats();
    park.set_ride_ticket_price(ride, cents)
        .map_err(|error| error.to_string())?;
    if park.guests() != before
        || balances(park) != money
        || park.ticks() != ticks
        || park.path_cache_stats() != cache
    {
        return Err("price edit changed a locked fare, guest, balance, clock or path cache".into());
    }
    require_accounting(park)
}

fn roundtrip(park: &Park) -> Result<Vec<u8>, String> {
    require_accounting(park)?;
    let bytes = park.to_save_bytes().map_err(|error| error.to_string())?;
    let envelope: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if envelope["schema_version"] != 12 || envelope["content_version"] != 12 {
        return Err("economy proof requires strict schema/content 12/12".into());
    }
    if Park::from_save_bytes(&bytes).map_err(|error| error.to_string())? != *park {
        return Err("economy roundtrip changed authoritative state".into());
    }
    Ok(bytes)
}

fn reserved_boarding(park: &Park) -> bool {
    park.guests().iter().any(|guest| {
        guest.ride_visit.as_ref().is_some_and(|visit| {
            visit.phase == RideVisitPhase::Boarding
                && visit.passenger.as_ref().is_some_and(|passenger| {
                    passenger.fare_cents > 0
                        && !passenger.fare_paid
                        && !passenger.departed
                        && passenger.transfer.as_ref().is_some_and(|transfer| {
                            transfer.progress > 0 && transfer.progress < RIDE_TRANSFER_TICKS
                        })
                })
        })
    })
}

fn paid_moving(park: &Park) -> bool {
    park.train().is_some_and(|train| train.speed > 0)
        && park.guests().iter().any(|guest| {
            guest.ride_visit.as_ref().is_some_and(|visit| {
                visit.phase == RideVisitPhase::Riding
                    && visit.passenger.as_ref().is_some_and(|passenger| {
                        passenger.fare_paid
                            && passenger.fare_cents > 0
                            && passenger.departed
                            && !passenger.completed_lap
                    })
            })
        })
}

fn passengers(park: &Park) -> usize {
    park.guests()
        .iter()
        .filter(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.passenger.is_some())
        })
        .count()
}

fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool, label: &str) -> Result<(), String> {
    for _ in 0..SEEK_LIMIT {
        if predicate(park) {
            return Ok(());
        }
        advance_checked(park)?;
    }
    Err(format!(
        "economy proof did not reach {label} within {SEEK_LIMIT} ticks"
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    FinishPaid,
    ReserveCancellation,
    CancelReturn,
    RejectUnaffordable,
    RecoverPaid,
    Done,
}

fn continue_economy(park: &mut Park) -> Result<(), String> {
    if !paid_moving(park) {
        return Err("economy primary needs an actually paid moving passenger".into());
    }
    let ride = ride_id(park)?;
    let ticks = park.ticks();
    let ids: Vec<_> = park.guests().iter().map(|guest| guest.id).collect();
    let initial_completed = park.ride_journey_stats(ride).completed;
    set_price_checked(park, 700)?;
    roundtrip(park)?;
    let mut stage = Stage::FinishPaid;
    let mut transitions = vec![(stage, 0)];
    let mut cancel_money = None;
    let mut cancelled_guest = None;
    let mut cancelled_completed = 0;
    let mut rejections = 0;
    let mut recovery_revenue = 0;
    for tick in 0..REPLAY_TICKS {
        let previous_stage = stage;
        let mut changed = false;
        if tick == 30 {
            if !paid_moving(park) {
                return Err("paid primary returned before close command".into());
            }
            park.set_train_open(false);
            changed = true;
        }
        let stats = park.ride_journey_stats(ride);
        match stage {
            Stage::FinishPaid
                if tick > 30 && passengers(park) == 0 && stats.completed > initial_completed =>
            {
                if park.train().unwrap().open {
                    return Err("occupied close silently reopened train".into());
                }
                set_price_checked(park, 400)?;
                cancel_money = Some(balances(park));
                cancelled_completed = stats.completed;
                park.set_train_open(true);
                stage = Stage::ReserveCancellation;
                changed = true;
            }
            Stage::ReserveCancellation if reserved_boarding(park) => {
                cancelled_guest = park
                    .guests()
                    .iter()
                    .find(|guest| {
                        guest
                            .ride_visit
                            .as_ref()
                            .is_some_and(|visit| visit.phase == RideVisitPhase::Boarding)
                    })
                    .map(|guest| guest.id);
                // The active fare remains 400, even though the displayed price changes.
                set_price_checked(park, 600)?;
                park.set_train_open(false);
                stage = Stage::CancelReturn;
                changed = true;
            }
            Stage::CancelReturn if passengers(park) == 0 => {
                let guest = park
                    .guests()
                    .iter()
                    .find(|guest| Some(guest.id) == cancelled_guest)
                    .ok_or("cancelled guest identity vanished")?;
                if guest.ride_visit.is_some() || stats.completed != cancelled_completed {
                    return Err(
                        "predeparture cancellation did not return publicly without completion"
                            .into(),
                    );
                }
                if cancel_money.as_ref() != Some(&balances(park)) {
                    return Err("predeparture cancellation charged or refunded money".into());
                }
                set_price_checked(park, MAX_TICKET_PRICE_CENTS)?;
                rejections = park.ride_economy_stats(ride).unaffordable_rejections;
                park.set_train_open(true);
                stage = Stage::RejectUnaffordable;
                changed = true;
            }
            Stage::RejectUnaffordable
                if park.ride_economy_stats(ride).unaffordable_rejections > rejections =>
            {
                if passengers(park) != 0 || cancel_money.as_ref() != Some(&balances(park)) {
                    return Err("unaffordable attempt reserved a seat or moved money".into());
                }
                recovery_revenue = park.ride_economy_stats(ride).revenue_cents;
                set_price_checked(park, 200)?;
                stage = Stage::RecoverPaid;
                changed = true;
            }
            Stage::RecoverPaid
                if stats.completed > cancelled_completed
                    && park.ride_economy_stats(ride).revenue_cents > recovery_revenue =>
            {
                park.set_train_open(false);
                stage = Stage::Done;
                changed = true;
            }
            _ => {}
        }
        if matches!(
            stage,
            Stage::ReserveCancellation | Stage::CancelReturn | Stage::RejectUnaffordable
        ) && cancel_money.as_ref() != Some(&balances(park))
        {
            return Err("unearned/cancelled/unaffordable visit moved money".into());
        }
        if changed {
            roundtrip(park)?;
        }
        if stage != previous_stage {
            transitions.push((stage, tick));
        }
        advance_checked(park)?;
        if park
            .guests()
            .iter()
            .map(|guest| guest.id)
            .collect::<Vec<_>>()
            != ids
            || ride_id(park)? != ride
        {
            return Err("economy replay changed guest or ride identities".into());
        }
    }
    if stage != Stage::Done || park.ticks() != ticks + REPLAY_TICKS || cancelled_guest.is_none() {
        return Err(format!(
            "economy replay incomplete after {} ticks: stage={stage:?}, transitions={transitions:?}, cancelled_guest={cancelled_guest:?}, initial_completed={initial_completed}, cancelled_completed={cancelled_completed}, rejections_before={rejections}, recovery_revenue={recovery_revenue}, train={:?}, journey={:?}, queue={:?}, economy={:?}",
            park.ticks() - ticks,
            park.train(),
            park.ride_journey_stats(ride),
            park.ride_queue_stats(ride),
            park.ride_economy_stats(ride),
        ));
    }
    roundtrip(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    for _ in 0..731 {
        park.advance(Speed::Normal);
    }
    super::journey::install(&mut park)?;
    set_price_checked(&mut park, 300)?;
    seek(
        &mut park,
        reserved_boarding,
        "uncharged boarding reservation",
    )?;
    if park.ride().unwrap().revenue_cents != 0 || park.cash_cents() != INITIAL_PARK_CASH_CENTS {
        return Err("reservation must not debit wallets or earn cash".into());
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &roundtrip(&park)?,
    )?;
    set_price_checked(&mut park, 500)?;
    seek(&mut park, paid_moving, "departure charged at locked fares")?;
    if !park.guests().iter().any(|guest| {
        guest
            .ride_visit
            .as_ref()
            .and_then(|visit| visit.passenger.as_ref())
            .is_some_and(|passenger| passenger.fare_cents == 300 && passenger.fare_paid)
    }) {
        return Err("price change repriced the original reserved fare".into());
    }
    park.set_terrain_height(TilePos { x: 3, y: 24 }, 1)
        .map_err(|error| error.to_string())?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &roundtrip(&park)?,
    )?;
    continue_economy(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &roundtrip(&park)?,
    )?;
    println!("Ride economy continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|error| error.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    view.path_tool.set_mode(crate::path_tools::PathTool::Exit);
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || !view.terrain_custom
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
        || !paid_moving(&park)
    {
        return Err("economy primary must restore exact paid motion paused/Custom/Inspect".into());
    }
    continue_economy(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("reloaded economy continuation differs from uninterrupted state".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
        || !reserved_boarding(&park)
        || park.ride().unwrap().revenue_cents != 0
    {
        return Err("economy backup must restore exact unpaid boarding paused/Inspect".into());
    }
    roundtrip(&park)?;
    // Loading a reservation must not turn it into a charge; cancelling that loaded
    // reservation exercises the distinct backup continuation rather than only equality.
    let money = balances(&park);
    let ride = ride_id(&park)?;
    let completed = park.ride_journey_stats(ride).completed;
    park.set_train_open(false);
    seek(
        &mut park,
        |park| passengers(park) == 0,
        "cancelled loaded reservation returns to public path",
    )?;
    if balances(&park) != money || park.ride_journey_stats(ride).completed != completed {
        return Err("loaded reservation cancellation changed money or completed trips".into());
    }
    roundtrip(&park)?;
    if fs::read(store.save_path()).map_err(|error| error.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|error| error.to_string())? != backup
    {
        return Err("economy verification changed primary or backup bytes".into());
    }
    write_new(&store.directory().join("verification-economy.txt"), format!(
        "Parkbound {}\nFresh-process paid moving primary equality: PASS\nContinuation: PASS ({REPLAY_TICKS} complete ticks + price changes, paid completion, predeparture cancellation, unaffordable rejection and affordable recovery)\nLocked fares survive price changes and charge only at actual departure: PASS\nEvery-tick wallet plus spent, revenue and park cash conservation: PASS\nPredeparture cancellation and rejected attempts move no money: PASS\nExplicit reserved-transfer backup, paused/Inspect loading and uncharged cancellation: PASS\nPrimary and backup files unchanged: PASS\nStrict schema/content 12/12: PASS\n", env!("CARGO_PKG_VERSION")
    ).as_bytes())?;
    println!("Fresh-process economy and reserved-fare backup verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn economy_modes_conserve_paid_and_reserved_fares_through_replay_and_both_slots() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!(
            "Parkbound economy proof é {} {stamp}",
            std::process::id()
        )));
        let arguments = |mode: &str| {
            vec![
                OsString::from("--save-check"),
                OsString::from(mode),
                OsString::from("--save-dir"),
                scratch.0.clone().into_os_string(),
            ]
        };
        assert_eq!(run(arguments("write-economy")), Some(AppExit::Success));
        let primary = fs::read(scratch.0.join("park.park.json")).unwrap();
        let backup = fs::read(scratch.0.join("park.park.json.bak")).unwrap();
        assert_eq!(run(arguments("write-economy")), Some(AppExit::error()));
        assert_eq!(run(arguments("verify-economy")), Some(AppExit::Success));
        assert_eq!(fs::read(scratch.0.join("park.park.json")).unwrap(), primary);
        assert_eq!(
            fs::read(scratch.0.join("park.park.json.bak")).unwrap(),
            backup
        );
        assert!(scratch.0.join("verification-economy.txt").is_file());
    }
}
