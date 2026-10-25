//! Natural concurrent rides, owner-specific money and exact continuation.
use super::*;
use park_sim::{FinanceKind, PathDirection, PathSlope, RideId, RideVisitPhase};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const ORIGIN: TilePos = TilePos { x: 19, y: 22 };
const REPLAY_TICKS: u64 = 24_000;
const SEEK_LIMIT: usize = 48_000;

pub(crate) fn install_carousel(park: &mut Park) -> Result<RideId, String> {
    let id = park.place_carousel(ORIGIN, 8).map_err(|e| e.to_string())?;
    // Separate from the coaster queue. The exit meets a public extension of
    // the existing loop; a ride-owned queue reaches its western entrance.
    if !park
        .objects()
        .iter()
        .any(|o| o.is_public_path() && o.pos == (TilePos { x: 20, y: 20 }) && o.height == 8)
    {
        park.place_path(TilePos { x: 20, y: 20 }, 8)
            .map_err(|e| e.to_string())?;
    }
    for y in 21..=23 {
        park.place_queue_path(TilePos { x: 17, y }, 8, PathSlope::Flat, id)
            .map_err(|e| e.to_string())?;
    }
    park.place_ride_entrance(TilePos { x: 18, y: 23 }, 8, PathDirection::East, id)
        .map_err(|e| e.to_string())?;
    park.place_ride_exit(TilePos { x: 20, y: 21 }, 8, PathDirection::South, id)
        .map_err(|e| e.to_string())?;
    if park.ride_access(id).status != park_sim::RideAccessStatus::Ready
        || park.ride_exit_access(id).status != park_sim::RideExitStatus::Ready
    {
        return Err("carousel fixture needs its own connected queue and public exit".into());
    }
    park.set_ride_ticket_price(id, 100)
        .map_err(|e| e.to_string())?;
    park.set_ride_open(id, true).map_err(|e| e.to_string())?;
    Ok(id)
}

pub(crate) fn install(park: &mut Park) -> Result<(RideId, RideId), String> {
    let coaster = super::journey::install(park)?;
    park.set_ride_ticket_price(coaster, 100)
        .map_err(|e| e.to_string())?;
    let carousel = install_carousel(park)?;
    Ok((coaster, carousel))
}

pub(crate) fn paid_running(park: &Park, ride: RideId) -> bool {
    park.ride_is_running(ride)
        && park.guests().iter().any(|g| {
            g.ride_visit.as_ref().is_some_and(|v| {
                v.ride == ride
                    && v.phase == RideVisitPhase::Riding
                    && v.passenger
                        .as_ref()
                        .is_some_and(|p| p.departed && p.fare_paid && !p.completed_lap)
            })
        })
}

pub(crate) fn check(park: &Park) -> Result<(), String> {
    super::finance::check(park)?;
    let mut seats = BTreeSet::new();
    let mut receipts = BTreeMap::<RideId, (u64, u64, u64, u64)>::new();
    for guest in park.guests() {
        let mut spent = 0;
        let mut completed = 0;
        let mut ratings = 0;
        let mut rating_total = 0;
        let mut owners = BTreeSet::new();
        for history in &guest.ride_history {
            if park.ride_by_id(history.ride).is_none() || !owners.insert(history.ride) {
                return Err("guest history has unknown or duplicate ride ownership".into());
            }
            let totals = receipts.entry(history.ride).or_default();
            totals.0 += history.spent_cents;
            totals.1 += history.completed;
            totals.2 += history.rating_count;
            totals.3 += history.rating_total;
            spent += history.spent_cents;
            completed += history.completed;
            ratings += history.rating_count;
            rating_total += history.rating_total;
        }
        if spent != guest.spent_cents - guest.service_spent_cents
            || completed != guest.rides_completed
            || ratings != guest.ratings_given
            || rating_total != guest.rating_total
        {
            return Err(
                "guest aggregate history does not equal its per-ride receipts and reviews".into(),
            );
        }
        if let Some(visit) = &guest.ride_visit {
            if park.ride_by_id(visit.ride).is_none() || guest.service_visit.is_some() {
                return Err("guest has unknown ride or concurrent service ownership".into());
            }
            if let Some(passenger) = &visit.passenger
                && (usize::from(passenger.seat) >= park.ride_capacity(visit.ride).unwrap_or(0)
                    || !seats.insert((visit.ride, passenger.seat)))
            {
                return Err("duplicate seat within the same ride".into());
            }
        }
    }
    let ledger = park.finance_ledger();
    for ride in park.all_rides() {
        let total = receipts.get(&ride.id).copied().unwrap_or_default();
        if total
            != (
                ride.revenue_cents,
                ride.completed,
                ride.rating_count,
                ride.rating_total,
            )
        {
            return Err(format!(
                "ride {} counters differ from guest-owned history",
                ride.id.0
            ));
        }
        let opening = ledger
            .opening_rides
            .iter()
            .filter(|r| r.ride == ride.id)
            .map(|r| r.revenue_cents)
            .sum::<u64>();
        let archived = ledger
            .archived_rides
            .iter()
            .filter(|r| r.ride == ride.id)
            .map(|r| r.revenue_cents)
            .sum::<u64>();
        let recent = ledger
            .entries
            .iter()
            .filter(|e| e.ride == Some(ride.id) && e.kind == FinanceKind::RideRevenue)
            .map(|e| e.amount_cents)
            .sum::<u64>();
        if opening + archived + recent != ride.revenue_cents {
            return Err("ride ledger attribution differs from earned revenue".into());
        }
    }
    for entry in &ledger.entries {
        if matches!(
            entry.kind,
            FinanceKind::RideRevenue
                | FinanceKind::RideUpkeep
                | FinanceKind::RideConstruction
                | FinanceKind::RideDemolitionRefund
                | FinanceKind::TrackConstruction
                | FinanceKind::TrackRefund
        ) != entry.ride.is_some()
            || entry.ride.is_some_and(|id| park.ride_by_id(id).is_none())
        {
            return Err("ledger transaction has incorrect ride ownership".into());
        }
    }
    Ok(())
}

/// The retained single-coaster proof helpers assert their fixture's one owner.
/// This proof validates all ride owners before using the production codec.
pub(crate) fn bytes(park: &Park) -> Result<Vec<u8>, String> {
    check(park)?;
    let bytes = park.to_save_bytes().map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if value["schema_version"] != park_sim::SAVE_SCHEMA_VERSION
        || value["content_version"] != park_sim::SAVE_CONTENT_VERSION
        || Park::from_save_bytes(&bytes).map_err(|e| e.to_string())? != *park
    {
        return Err("dual-ride current-schema roundtrip changed authoritative state".into());
    }
    Ok(bytes)
}

/// Derive a review from the previous final transfer step, independently of the
/// new counters. Other rides' history must remain unchanged on this guest.
fn check_history_delta(
    park: &Park,
    old: &park_sim::Guest,
    new: &park_sim::Guest,
    payment: Option<(RideId, u32)>,
) -> Result<(), String> {
    let mut expected: BTreeMap<_, _> = old
        .ride_history
        .iter()
        .map(|h| {
            (
                h.ride,
                (h.spent_cents, h.completed, h.rating_count, h.rating_total),
            )
        })
        .collect();
    if let Some((owner, fare)) = payment {
        expected.entry(owner).or_default().0 += u64::from(fare);
    }
    let review = old.ride_visit.as_ref().and_then(|visit| {
        let passenger = visit.passenger.as_ref()?;
        let transfer = passenger.transfer.as_ref()?;
        (visit.phase == RideVisitPhase::Exiting
            && passenger.completed_lap
            && transfer.progress.checked_add(1) == Some(park_sim::RIDE_TRANSFER_TICKS))
        .then_some((visit, passenger, transfer))
    });
    let mut completed = 0;
    let mut score = 0;
    let mut last_rating = old.last_rating;
    if let Some((visit, passenger, transfer)) = review {
        if !passenger.departed
            || !passenger.fare_paid
            || new.ride_visit.is_some()
            || new.current != Some(transfer.path)
            || !park
                .object(transfer.path)
                .is_some_and(|o| o.is_public_path())
        {
            return Err(
                "completed transfer did not return its passenger to the public path".into(),
            );
        }
        let limit = old.preferences.max_ticket_price_cents;
        let price_penalty = if passenger.fare_cents <= limit / 2 {
            0
        } else if passenger.fare_cents <= limit {
            1
        } else {
            2
        };
        let wait_penalty = (visit.waited_ticks / old.preferences.comfortable_queue_ticks).min(2);
        let rating = (5u32 - price_penalty - wait_penalty).clamp(1, 5) as u8;
        completed = 1;
        score = u64::from(rating);
        last_rating = Some(rating);
        let history = expected.entry(visit.ride).or_default();
        history.1 += 1;
        history.2 += 1;
        history.3 += score;
    }
    let actual: BTreeMap<_, _> = new
        .ride_history
        .iter()
        .map(|h| {
            (
                h.ride,
                (h.spent_cents, h.completed, h.rating_count, h.rating_total),
            )
        })
        .collect();
    if old.id != new.id
        || old.preferences != new.preferences
        || new.rides_completed != old.rides_completed + completed
        || new.ratings_given != old.ratings_given + completed
        || new.rating_total != old.rating_total + score
        || new.last_rating != last_rating
        || actual != expected
    {
        return Err(format!(
            "guest {} history differs from its actual owner departure or completed public return",
            old.id.0
        ));
    }
    Ok(())
}

/// Derive allowed fare changes from actual per-owner runtime departures, not
/// from the cash totals or fare-paid flags alone, and reviews from the previous
/// terminal exit transfer. This fixture has no services.
pub(crate) fn advance_checked(park: &mut Park) -> Result<Vec<RideId>, String> {
    if !park.services().is_empty() {
        return Err("dual-ride oracle expects no service facilities".into());
    }
    let guests = park.guests().to_vec();
    let running: BTreeMap<_, _> = park
        .all_rides()
        .map(|r| (r.id, park.ride_is_running(r.id)))
        .collect();
    let revenues: BTreeMap<_, _> = park.all_rides().map(|r| (r.id, r.revenue_cents)).collect();
    let old_cash = park.cash_cents();
    let old_next = park.finance_ledger().next_entry_id;
    let mut balance = old_cash;
    let mut expected = Vec::new();
    let mut owners = park.all_rides().map(|r| r.id).collect::<Vec<_>>();
    owners.sort_unstable();
    for id in owners {
        let costs = park.ride_running_costs(id).unwrap();
        if park.ride_is_open(id)
            && costs.upkeep_progress + 1 == park_sim::RIDE_UPKEEP_TICKS
            && balance >= u64::from(costs.cost_per_interval_cents)
        {
            let amount = u64::from(costs.cost_per_interval_cents);
            balance -= amount;
            expected.push((FinanceKind::RideUpkeep, id, amount, balance));
        }
    }
    park.advance(Speed::Normal);
    let mut earned = BTreeMap::<RideId, u64>::new();
    let mut choices = Vec::new();
    for old in &guests {
        let new = park
            .guests()
            .iter()
            .find(|g| g.id == old.id)
            .ok_or("guest disappeared")?;
        let old_visit = old.ride_visit.as_ref();
        let new_visit = new.ride_visit.as_ref();
        if old_visit.is_none()
            && let Some(v) = new_visit
        {
            choices.push(v.ride);
        }
        let old_p = old_visit.and_then(|v| v.passenger.as_ref());
        let new_p = new_visit.and_then(|v| v.passenger.as_ref());
        let mut charge = 0;
        let mut payment = None;
        if let (Some(a), Some(b)) = (old_p, new_p) {
            let before = old_visit.unwrap();
            let after = new_visit.unwrap();
            if before.ride != after.ride
                || before.ticket != after.ticket
                || a.fare_cents != b.fare_cents
                || (a.fare_paid && !b.fare_paid)
            {
                return Err("active passenger changed owner, ticket or locked fare".into());
            }
            if !a.fare_paid && b.fare_paid {
                if running[&after.ride]
                    || !park.ride_is_running(after.ride)
                    || a.departed
                    || !b.departed
                    || before.phase != RideVisitPhase::Riding
                    || after.phase != RideVisitPhase::Riding
                {
                    return Err(
                        "payment did not coincide with its owner's actual first departure".into(),
                    );
                }
                charge = a.fare_cents;
                payment = Some((after.ride, charge));
                *earned.entry(after.ride).or_default() += u64::from(charge);
            }
        } else if old_p.is_none() && new_p.is_some_and(|p| p.fare_paid) {
            return Err("new rider was charged without a prior reservation".into());
        }
        if new.wallet_cents.checked_add(charge) != Some(old.wallet_cents)
            || new.spent_cents != old.spent_cents + u64::from(charge)
            || new.service_spent_cents != old.service_spent_cents
        {
            return Err("guest payment differs from the owner's departure fare".into());
        }
        check_history_delta(park, old, new, payment)?;
    }
    for ride in park.all_rides() {
        if ride.revenue_cents
            != revenues[&ride.id] + earned.get(&ride.id).copied().unwrap_or_default()
        {
            return Err("fare credited to another ride".into());
        }
    }
    for (owner, amount) in earned.into_iter().filter(|(_, amount)| *amount != 0) {
        balance += amount;
        expected.push((FinanceKind::RideRevenue, owner, amount, balance));
    }
    let actual: Vec<_> = park
        .finance_ledger()
        .entries
        .iter()
        .filter(|e| e.id >= old_next)
        .collect();
    if actual.len() != expected.len()
        || park.cash_cents() != balance
        || park.finance_ledger().next_entry_id != old_next + expected.len() as u64
    {
        return Err(
            "new ledger count or cash differs from independently derived departures".into(),
        );
    }
    for (index, (entry, (kind, owner, amount, cash))) in actual.iter().zip(expected).enumerate() {
        if entry.id != old_next + index as u64
            || entry.tick != park.ticks()
            || entry.kind != kind
            || entry.ride != Some(owner)
            || entry.service.is_some()
            || entry.amount_cents != amount
            || entry.balance_cents != cash
        {
            return Err("ordered ride receipt or resulting ledger balance is incorrect".into());
        }
    }
    check(park)?;
    Ok(choices)
}

pub(crate) fn seek(
    park: &mut Park,
    condition: impl Fn(&Park) -> bool,
    label: &str,
) -> Result<(), String> {
    for _ in 0..SEEK_LIMIT {
        if condition(park) {
            return Ok(());
        }
        advance_checked(park)?;
    }
    Err(format!("dual-ride state not reached: {label}"))
}

pub(crate) fn seek_new_choice(park: &mut Park, owner: RideId) -> Result<park_sim::GuestId, String> {
    for _ in 0..SEEK_LIMIT {
        let available: BTreeSet<_> = park
            .guests()
            .iter()
            .filter(|g| g.ride_visit.is_none())
            .map(|g| g.id)
            .collect();
        advance_checked(park)?;
        if let Some(guest) = park.guests().iter().find(|g| {
            available.contains(&g.id)
                && g.ride_visit
                    .as_ref()
                    .is_some_and(|v| v.ride == owner && v.phase == RideVisitPhase::Approaching)
        }) {
            return Ok(guest.id);
        }
    }
    Err(format!(
        "no new public guest approached ride {} after the command",
        owner.0
    ))
}

fn replay(park: &mut Park) -> Result<(), String> {
    let started = park.ticks();
    let coaster = park.ride().ok_or("missing coaster")?.id;
    let carousel = park
        .carousels()
        .iter()
        .find(|r| !r.objects.is_empty())
        .ok_or("missing carousel")?
        .ride
        .id;
    let protected = park.carousel(carousel).unwrap().objects[0];
    let before = park.clone();
    if park.remove_object(protected).is_some() || *park != before {
        return Err("occupied carousel demolition was not atomic".into());
    }
    let mut preferred_carousel = 0;
    let mut preferred_coaster = 0;
    let mut rebuilt = None;
    for elapsed in 0..REPLAY_TICKS {
        match elapsed {
            0 => park
                .set_ride_open(coaster, false)
                .map_err(|e| e.to_string())?,
            90 => park
                .set_ride_ticket_price(carousel, 150)
                .map_err(|e| e.to_string())?,
            300 => park
                .set_ride_open(coaster, true)
                .map_err(|e| e.to_string())?,
            750 => park
                .set_ride_open(carousel, false)
                .map_err(|e| e.to_string())?,
            1600 => park
                .set_ride_open(carousel, true)
                .map_err(|e| e.to_string())?,
            3600 => park
                .set_ride_ticket_price(coaster, 10_000)
                .map_err(|e| e.to_string())?,
            5400 => {
                park.set_ride_ticket_price(carousel, 10_000)
                    .map_err(|e| e.to_string())?;
                park.set_ride_ticket_price(coaster, 0)
                    .map_err(|e| e.to_string())?;
            }
            7200 => {
                park.set_ride_ticket_price(carousel, 100)
                    .map_err(|e| e.to_string())?;
                park.set_ride_ticket_price(coaster, 100)
                    .map_err(|e| e.to_string())?;
            }
            9000 => park
                .set_ride_open(carousel, false)
                .map_err(|e| e.to_string())?,
            _ => {}
        }
        if elapsed > 9000
            && rebuilt.is_none()
            && !park
                .guests()
                .iter()
                .any(|g| g.ride_visit.as_ref().is_some_and(|v| v.ride == carousel))
        {
            let old = park.ride_by_id(carousel).unwrap().clone();
            park.remove_carousel(carousel).map_err(|e| e.to_string())?;
            let paths: Vec<_> = park
                .objects()
                .iter()
                .filter(|o| o.queue_owner == Some(carousel))
                .map(|o| o.id)
                .collect();
            for id in paths {
                park.remove_object(id)
                    .ok_or("empty old queue removal failed")?;
            }
            if park.ride_by_id(carousel).unwrap().revenue_cents != old.revenue_cents
                || park.ride_by_id(carousel).unwrap().rating_total != old.rating_total
            {
                return Err("demolition erased ride finances or reviews".into());
            }
            let new = install_carousel(park)?;
            if new.0 <= carousel.0 {
                return Err("carousel rebuild reused a ride identity".into());
            }
            rebuilt = Some(new);
        }
        for selected in advance_checked(park)? {
            if (3600..5400).contains(&elapsed) && selected == carousel {
                preferred_carousel += 1;
            }
            if (5400..7200).contains(&elapsed) && selected == coaster {
                preferred_coaster += 1;
            }
        }
        if elapsed % 997 == 0 {
            bytes(park)?;
        }
    }
    let rebuilt = rebuilt.ok_or("closed carousel never emptied for rebuilding")?;
    if preferred_carousel == 0
        || preferred_coaster == 0
        || park.ticks() != started + REPLAY_TICKS
        || park.ride_by_id(coaster).unwrap().rating_count == 0
        || park.ride_by_id(carousel).unwrap().rating_count == 0
        || park.ride_by_id(rebuilt).unwrap().revenue_cents == 0
        || park.ride_by_id(rebuilt).unwrap().completed == 0
    {
        return Err(format!(
            "dual-ride replay incomplete: alternative choices={preferred_carousel}/{preferred_coaster}, rebuilt={rebuilt:?}"
        ));
    }
    bytes(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let (coaster, carousel) = install(&mut park)?;
    seek(
        &mut park,
        |p| {
            p.guests().iter().any(|g| {
                g.ride_visit.as_ref().is_some_and(|v| {
                    v.ride == carousel
                        && v.phase == RideVisitPhase::Boarding
                        && v.passenger
                            .as_ref()
                            .and_then(|s| s.transfer.as_ref())
                            .is_some_and(|t| t.progress > 0)
                })
            })
        },
        "carousel mid-boarding backup",
    )?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &bytes(&park)?,
    )?;
    seek(
        &mut park,
        |p| paid_running(p, coaster) && paid_running(p, carousel),
        "both rides carry paid passengers",
    )?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &bytes(&park)?,
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &bytes(&park)?,
    )?;
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let original = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState {
        rides_open: true,
        selected_ride: Some(RideId(999)),
        ride_browse_pending: 1,
        ..Default::default()
    };
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || view.rides_open
        || view.selected_ride.is_some()
        || view.ride_browse_pending != 0
    {
        return Err("dual-ride primary or transient ride selection did not restore exactly".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("dual-ride continuation diverged".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))? {
        return Err("carousel boarding backup changed".into());
    }
    let frozen = park.clone();
    park.advance(Speed::Paused);
    if park != frozen
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != original
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("paused verification changed park or save files".into());
    }
    write_new(&store.directory().join("verification-rides.txt"), format!("Parkbound {}\nExact dual occupied primary and carousel boarding backup: PASS\nContinuation: PASS ({REPLAY_TICKS} ticks, two independent runtimes)\nOwner-specific actual departure fares, locked quotes, reviews and ledger attribution: PASS\nReachable guest choice changes with price/availability: PASS\nClose/reopen and occupied demolition isolation: PASS\nIdle demolition retains history; rebuilt carousel uses fresh IDs and earns new income: PASS\nPaused load, ride selection reset, current-schema roundtrips and unchanged slots: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn natural_dual_ride_proof_survives_fresh_load_and_changed_choice() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound dual rides é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-rides", "verify-rides"] {
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
        assert!(directory.join("verification-rides.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn blocked_footprint_does_not_allocate_ride_or_objects() {
        let mut park = demonstration_park();
        park.place_path(
            TilePos {
                x: ORIGIN.x + 2,
                y: ORIGIN.y + 2,
            },
            8,
        )
        .unwrap();
        let before = park.clone();
        assert!(park.place_carousel(ORIGIN, 8).is_err());
        assert_eq!(park, before);
    }
}
