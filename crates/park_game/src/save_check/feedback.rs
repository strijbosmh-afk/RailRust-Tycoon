//! Natural guest feedback, independently checked reviews and exact continuation.
use super::*;
use park_sim::{Guest, GuestId, GuestThought, RideVisitPhase};

const REPLAY_TICKS: u64 = 18_000;
pub(crate) const SEEK_LIMIT: usize = 18_000;

pub(crate) fn check(park: &Park) -> Result<(), String> {
    super::economy::require_accounting(park)?;
    let ride = park.ride().ok_or("feedback proof requires a ride")?;
    let stats = park.ride_experience_stats(ride.id);
    let mut count = 0u128;
    let mut total = 0u128;
    let mut satisfaction = 0u64;
    for guest in park.guests() {
        if guest.satisfaction > 100
            || guest.thought_tick > park.ticks()
            || guest.ratings_given > guest.rides_completed
            || guest.rating_total < guest.ratings_given
            || u128::from(guest.rating_total) > u128::from(guest.ratings_given) * 5
            || (guest.ratings_given == 0) != guest.last_rating.is_none()
            || guest
                .last_rating
                .is_some_and(|rating| !(1..=5).contains(&rating))
            || ![400, 600, 800, 1000].contains(&guest.preferences.max_ticket_price_cents)
            || ![450, 750, 1050, 1350].contains(&guest.preferences.comfortable_queue_ticks)
        {
            return Err("guest feedback bounds, review history or trait domain disagree".into());
        }
        count += u128::from(guest.ratings_given);
        total += u128::from(guest.rating_total);
        satisfaction += u64::from(guest.satisfaction);
    }
    let willing = park
        .guests()
        .iter()
        .filter(|g| g.willing_to_pay(ride.ticket_price_cents))
        .count();
    let interested = park
        .guests()
        .iter()
        .filter(|g| {
            g.willing_to_pay(ride.ticket_price_cents) && g.wallet_cents >= ride.ticket_price_cents
        })
        .count();
    if u128::from(stats.rating_count) != count
        || u128::from(stats.rating_total) != total
        || stats.average_rating_tenths != (count > 0).then(|| (total * 10 / count) as u8)
        || stats.willing_guests != willing
        || stats.interested_guests != interested
        || stats.guest_count != park.guests().len()
    {
        return Err("ride review/interest summaries differ from individual guests".into());
    }
    let summary = park.guest_experience_summary();
    if summary.guest_count != park.guests().len()
        || summary.unhappy_guests
            != park
                .guests()
                .iter()
                .filter(|guest| guest.satisfaction < 40)
                .count()
        || summary.average_satisfaction
            != (!park.guests().is_empty())
                .then(|| (satisfaction / park.guests().len() as u64) as u8)
    {
        return Err("guest satisfaction summary differs from individual guests".into());
    }
    Ok(())
}

/// Use the previous transfer and locked fare, not the new rating counters, as
/// the independent evidence that a review was earned on this exact tick.
pub(crate) fn advance_checked(park: &mut Park) -> Result<(), String> {
    let before = park.guests().to_vec();
    super::economy::advance_checked(park)?;
    for (old, new) in before.iter().zip(park.guests()) {
        if old.id != new.id || old.preferences != new.preferences {
            return Err("guest identity or preferences changed during feedback tick".into());
        }
        if new.thought_tick < old.thought_tick
            || (old.thought != new.thought && new.thought_tick != park.ticks())
        {
            return Err("thought changes are not stamped at their simulation event".into());
        }
        if old.rides_completed.checked_add(1) == Some(new.rides_completed) {
            let visit = old
                .ride_visit
                .as_ref()
                .ok_or("review has no previous visit")?;
            let passenger = visit
                .passenger
                .as_ref()
                .ok_or("review has no previous passenger")?;
            let public = new
                .current
                .and_then(|id| park.object(id))
                .is_some_and(|object| {
                    object.kind == ObjectKind::Path && object.queue_owner.is_none()
                });
            if visit.phase != RideVisitPhase::Exiting
                || !passenger.completed_lap
                || !passenger.departed
                || !passenger.fare_paid
                || new.ride_visit.is_some()
                || !public
            {
                return Err("review did not follow a completed public return".into());
            }
            let limit = old.preferences.max_ticket_price_cents;
            let price_penalty = if passenger.fare_cents <= limit / 2 {
                0
            } else if passenger.fare_cents <= limit {
                1
            } else {
                2
            };
            let wait_penalty =
                (visit.waited_ticks / old.preferences.comfortable_queue_ticks).min(2);
            let rating = (5u32 - price_penalty - wait_penalty).clamp(1, 5) as u8;
            let thought = if rating >= 3 {
                GuestThought::EnjoyedRide
            } else {
                GuestThought::DisappointedByRide
            };
            let (_, satisfaction_after_needs) =
                super::services::needs_after_tick(old, park.ticks());
            if new.ratings_given != old.ratings_given + 1
                || new.rating_total != old.rating_total + u64::from(rating)
                || new.last_rating != Some(rating)
                || new.satisfaction
                    != ((3 * u16::from(satisfaction_after_needs) + 20 * u16::from(rating)) / 4)
                        as u8
                || new.thought != thought
                || new.thought_tick != park.ticks()
            {
                return Err("review score/count/satisfaction differs from the completed trip's frozen experience".into());
            }
        } else if new.rides_completed != old.rides_completed
            || new.ratings_given != old.ratings_given
            || new.rating_total != old.rating_total
            || new.last_rating != old.last_rating
        {
            return Err("rating changed without exactly one completed public return".into());
        }
    }
    check(park)
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
        "feedback did not reach {label} within {SEEK_LIMIT} ticks; ticks={}, journey={:?}, queue={:?}, experience={:?}",
        park.ticks(),
        park.ride_journey_stats(park.ride().unwrap().id),
        park.ride_queue_stats(park.ride().unwrap().id),
        park.ride_experience_stats(park.ride().unwrap().id)
    ))
}

pub(crate) fn has_thought(park: &Park, thought: GuestThought) -> Option<GuestId> {
    park.guests()
        .iter()
        .find(|guest| guest.thought == thought)
        .map(|guest| guest.id)
}

pub(crate) fn close_checked(park: &mut Park) -> Result<GuestId, String> {
    let queued = park
        .guests()
        .iter()
        .find(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
        })
        .cloned()
        .ok_or("cancellation needs an admitted queue guest")?;
    let before = park.clone();
    park.set_train_open(false);
    let after = park
        .guests()
        .iter()
        .find(|guest| guest.id == queued.id)
        .unwrap();
    if after.thought != GuestThought::RideCancelled
        || after.satisfaction != queued.satisfaction.saturating_sub(8)
        || after.thought_tick != park.ticks()
        || park.ticks() != before.ticks()
        || after.ratings_given != queued.ratings_given
        || after.rating_total != queued.rating_total
        || after.wallet_cents != queued.wallet_cents
        || after.spent_cents != queued.spent_cents
    {
        return Err(
            "joined cancellation did not give exactly one penalty without a review/payment".into(),
        );
    }
    let once = park.clone();
    park.set_train_open(false);
    if *park != once {
        return Err("repeating Close duplicated a feedback event".into());
    }
    check(park)?;
    Ok(queued.id)
}

fn bytes(park: &Park) -> Result<Vec<u8>, String> {
    check(park)?;
    let bytes = park.to_save_bytes().map_err(|error| error.to_string())?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if value["schema_version"] != 12
        || value["content_version"] != 12
        || Park::from_save_bytes(&bytes).map_err(|error| error.to_string())? != *park
    {
        return Err("feedback strict v12 roundtrip changed authoritative state".into());
    }
    Ok(bytes)
}

fn reviewed_public(guest: &Guest) -> bool {
    guest.ratings_given > 0
        && guest.ride_visit.is_none()
        && matches!(
            guest.thought,
            GuestThought::EnjoyedRide | GuestThought::DisappointedByRide
        )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    CancelQueue,
    Drain,
    ValueRefusal,
    Recover,
    Done,
}

fn continue_feedback(park: &mut Park) -> Result<(), String> {
    let ride = park.ride().ok_or("feedback proof needs ride")?.id;
    let started = park.ticks();
    let mut recovery_ratings = park.ride_experience_stats(ride).rating_count;
    let ids: Vec<_> = park
        .guests()
        .iter()
        .map(|guest| (guest.id, guest.preferences))
        .collect();
    let mut stage = Stage::CancelQueue;
    let mut transitions = vec![(stage, 0)];
    let mut refusals = 0;
    let mut cancelled = None;
    super::economy::set_price_checked(park, 200)?;
    park.set_train_open(true);
    for tick in 0..REPLAY_TICKS {
        let old_stage = stage;
        match stage {
            Stage::CancelQueue
                if park.guests().iter().any(|guest| {
                    guest
                        .ride_visit
                        .as_ref()
                        .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
                }) =>
            {
                cancelled = Some(close_checked(park)?);
                stage = Stage::Drain;
            }
            Stage::Drain if park.guests().iter().all(|guest| guest.ride_visit.is_none()) => {
                super::economy::set_price_checked(park, 1100)?;
                refusals = park.ride_experience_stats(ride).value_rejections;
                park.set_train_open(true);
                stage = Stage::ValueRefusal;
            }
            Stage::ValueRefusal
                if park.ride_experience_stats(ride).value_rejections > refusals
                    && has_thought(park, GuestThought::TooExpensive).is_some() =>
            {
                super::economy::set_price_checked(park, 200)?;
                recovery_ratings = park.ride_experience_stats(ride).rating_count;
                stage = Stage::Recover;
            }
            Stage::Recover
                if park.ride_experience_stats(ride).rating_count > recovery_ratings
                    && park.guests().iter().any(reviewed_public) =>
            {
                park.set_train_open(false);
                stage = Stage::Done;
            }
            _ => {}
        }
        if old_stage != stage {
            transitions.push((stage, tick));
            bytes(park)?;
        }
        advance_checked(park)?;
        if tick % 997 == 0 {
            bytes(park)?;
        }
    }
    if stage != Stage::Done
        || cancelled.is_none()
        || park.ticks() != started + REPLAY_TICKS
        || park
            .guests()
            .iter()
            .map(|guest| (guest.id, guest.preferences))
            .collect::<Vec<_>>()
            != ids
    {
        return Err(format!(
            "feedback replay incomplete: {stage:?}, transitions={transitions:?}, stats={:?}, journey={:?}",
            park.ride_experience_stats(ride),
            park.ride_journey_stats(ride)
        ));
    }
    bytes(park)?;
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    for _ in 0..731 {
        advance_checked(&mut park)?;
    }
    let ride = super::queues::install(&mut park)?;
    super::economy::set_price_checked(&mut park, 1100)?;
    seek(
        &mut park,
        |p| has_thought(p, GuestThought::TooExpensive).is_some(),
        "affordable but poor-value refusal",
    )?;
    if park.ride_economy_stats(ride).unaffordable_rejections != 0
        || park.cash_cents() != park_sim::INITIAL_PARK_CASH_CENTS
    {
        return Err("value refusal was confused with wallet affordability/payment".into());
    }
    super::economy::set_price_checked(&mut park, 200)?;
    seek(
        &mut park,
        |p| has_thought(p, GuestThought::WaitingTooLong).is_some(),
        "actual queue discomfort without an exit",
    )?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &bytes(&park)?,
    )?;
    close_checked(&mut park)?;
    seek(
        &mut park,
        |p| p.guests().iter().all(|guest| guest.ride_visit.is_none()),
        "cancelled queue returns publicly",
    )?;
    super::journey::install_exit(&mut park, ride)?;
    park.set_train_open(true);
    seek(
        &mut park,
        |p| p.guests().iter().any(reviewed_public),
        "first genuine completed review",
    )?;
    park.set_terrain_height(TilePos { x: 3, y: 24 }, 1)
        .map_err(|error| error.to_string())?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &bytes(&park)?,
    )?;
    continue_feedback(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &bytes(&park)?,
    )?;
    println!("Guest feedback continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|error| error.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|error| error.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState {
        guests_open: true,
        selected_guest: park.guests().last().map(|guest| guest.id),
        guest_page: 1,
        ..ViewState::default()
    };
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || !view.terrain_custom
        || view.guests_open
        || view.selected_guest.is_some()
        || view.guest_page != 0
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
    {
        return Err(
            "feedback primary did not load exactly paused/Custom/Inspect with guest panel reset"
                .into(),
        );
    }
    continue_feedback(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("feedback replay differs after fresh-process loading".into());
    }
    view.guests_open = true;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
        || view.guests_open
        || has_thought(&park, GuestThought::WaitingTooLong).is_none()
    {
        return Err("feedback backup did not restore exact live queue experience paused".into());
    }
    let once = park.clone();
    park.advance(Speed::Paused);
    if park != once {
        return Err("paused feedback aged or changed authoritative state".into());
    }
    close_checked(&mut park)?;
    seek(
        &mut park,
        |p| p.guests().iter().all(|guest| guest.ride_visit.is_none()),
        "loaded queue cancellation",
    )?;
    if park
        .ride_experience_stats(park.ride().unwrap().id)
        .rating_count
        != 0
    {
        return Err("cancelled queue produced a review without a trip".into());
    }
    if fs::read(store.save_path()).map_err(|error| error.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|error| error.to_string())? != backup
    {
        return Err("feedback verification rewrote primary or backup".into());
    }
    write_new(&store.directory().join("verification-feedback.txt"), format!("Parkbound {}\nExact reviewed primary and active-wait backup: PASS\nCurrent-version continuation: PASS ({REPLAY_TICKS} ticks + cancellation, value refusal, price recovery and completed reviews)\nIndependent locked-fare, frozen-wait review and satisfaction deltas: PASS\nStable preferences, event thoughts, money conservation and strict12 roundtrips: PASS\nPaused load and guest-overlay reset: PASS\nCancelled queue produces no review: PASS\nBoth source slots unchanged: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feedback_modes_preserve_real_experiences_and_accounting_across_process_contract() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound feedback é {} {}",
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
        assert_eq!(run(args("write-feedback")), Some(AppExit::Success));
        assert_eq!(run(args("write-feedback")), Some(AppExit::error()));
        assert_eq!(run(args("verify-feedback")), Some(AppExit::Success));
        assert!(directory.join("verification-feedback.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }
}
