//! Natural needs, physical service use and independently checked relief/payment.
use super::*;
use park_sim::{
    Guest, GuestId, GuestNeeds, GuestThought, PathDirection, SERVICE_USE_TICKS, ServiceId,
    ServiceKind, ServiceVisitPhase,
};

const REPLAY_TICKS: u64 = 24_000;
const SEEK_LIMIT: usize = 24_000;
pub(crate) const SITES: [(TilePos, ServiceKind); 3] = [
    (TilePos { x: 10, y: 21 }, ServiceKind::Food),
    (TilePos { x: 15, y: 21 }, ServiceKind::Drink),
    (TilePos { x: 18, y: 21 }, ServiceKind::Toilet),
];

pub(crate) fn needs_after_tick(old: &Guest, tick: u64) -> (GuestNeeds, u8) {
    let mut needs = old.needs;
    let mut satisfaction = old.satisfaction;
    for (value, period) in [
        (&mut needs.hunger, 90),
        (&mut needs.thirst, 60),
        (&mut needs.toilet, 120),
    ] {
        let previous = *value;
        if tick.is_multiple_of(period) {
            *value = value.saturating_add(1).min(100);
        }
        if previous < 85 && *value >= 85 {
            satisfaction = satisfaction.saturating_sub(5);
        }
    }
    (needs, satisfaction)
}

fn relieved(needs: &mut GuestNeeds, kind: ServiceKind) {
    match kind {
        ServiceKind::Food => needs.hunger = 20,
        ServiceKind::Drink => needs.thirst = 20,
        ServiceKind::Toilet => needs.toilet = 20,
    }
}

pub(crate) fn check(park: &Park) -> Result<(), String> {
    super::feedback::check(park)?;
    let summary = park.guest_needs_summary();
    let guests = park.guests();
    if guests.iter().any(|g| {
        g.needs.hunger > 100
            || g.needs.thirst > 100
            || g.needs.toilet > 100
            || (g.service_visit.is_some() && g.ride_visit.is_some())
    }) {
        return Err("needs bounds or exclusive visit roles are invalid".into());
    }
    let phase_count = |phase| {
        guests
            .iter()
            .filter(|g| g.service_visit.as_ref().is_some_and(|v| v.phase == phase))
            .count()
    };
    if summary.guest_count != guests.len()
        || summary.hungry_guests != guests.iter().filter(|g| g.needs.hunger >= 60).count()
        || summary.thirsty_guests != guests.iter().filter(|g| g.needs.thirst >= 60).count()
        || summary.toilet_guests != guests.iter().filter(|g| g.needs.toilet >= 60).count()
        || summary.critical_guests
            != guests
                .iter()
                .filter(|g| g.needs.hunger >= 85 || g.needs.thirst >= 85 || g.needs.toilet >= 85)
                .count()
        || summary.seeking != phase_count(ServiceVisitPhase::Approaching)
        || summary.waiting != phase_count(ServiceVisitPhase::Waiting)
        || summary.using != phase_count(ServiceVisitPhase::Using)
        || u128::from(summary.services_used)
            != guests
                .iter()
                .map(|g| u128::from(g.services_used))
                .sum::<u128>()
        || summary.service_revenue_cents
            != park.services().iter().map(|s| s.revenue_cents).sum::<u64>()
    {
        return Err("needs/service summary differs from individual authoritative states".into());
    }
    for service in park.services() {
        let stats = park.service_stats(service.id);
        let count = |phase| {
            guests
                .iter()
                .filter(|g| {
                    g.service_visit
                        .as_ref()
                        .is_some_and(|v| v.service == service.id && v.phase == phase)
                })
                .count()
        };
        if stats.approaching != count(ServiceVisitPhase::Approaching)
            || stats.waiting != count(ServiceVisitPhase::Waiting)
            || stats.using != count(ServiceVisitPhase::Using)
            || stats.using > 1
            || stats.price_cents != service.price_cents
            || stats.served != service.served
            || stats.revenue_cents != service.revenue_cents
            || stats.unaffordable_rejections != service.unaffordable_rejections
        {
            return Err("facility statistics/reservation differ from individual visitors".into());
        }
    }
    Ok(())
}

pub(crate) fn advance_checked(park: &mut Park) -> Result<(), String> {
    let before = park.guests().to_vec();
    super::feedback::advance_checked(park)?;
    for (old, new) in before.iter().zip(park.guests()) {
        // Needs values are checked every tick. Satisfaction also has queue and
        // cancellation events, so this oracle checks its complete value only
        // on service completion; the feedback oracle checks public reviews.
        let (mut needs, satisfaction) = needs_after_tick(old, park.ticks());
        if new.services_used != old.services_used {
            let visit = old
                .service_visit
                .as_ref()
                .ok_or("service completion lacks old visit")?;
            let service = park
                .service(visit.service)
                .ok_or("completed service record vanished")?;
            if visit.phase != ServiceVisitPhase::Using
                || visit.progress + 1 != SERVICE_USE_TICKS
                || new.services_used != old.services_used + 1
                || new.service_visit.is_some()
            {
                return Err("service relief occurred without exactly one full use".into());
            }
            relieved(&mut needs, service.kind);
            let thought = match service.kind {
                ServiceKind::Food => GuestThought::AteFood,
                ServiceKind::Drink => GuestThought::HadDrink,
                ServiceKind::Toilet => GuestThought::UsedToilet,
            };
            if new.satisfaction != satisfaction.saturating_add(5).min(100)
                || new.thought != thought
                || new.thought_tick != park.ticks()
                || new.ratings_given != old.ratings_given
                || new.rides_completed != old.rides_completed
            {
                return Err(
                    "service relief changed satisfaction/thought/reviews incorrectly".into(),
                );
            }
        }
        if new.needs != needs {
            return Err(format!(
                "guest {:?} needs changed outside growth or completed use",
                old.id
            ));
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
        "service proof did not reach {label} within {SEEK_LIMIT} ticks: needs={:?}, services={:?}, ride={:?}",
        park.guest_needs_summary(),
        park.services(),
        park.ride()
    ))
}

pub(crate) fn install(park: &mut Park) -> Result<Vec<ServiceId>, String> {
    SITES
        .iter()
        .map(|(pos, kind)| {
            park.place_service(*pos, 8, PathDirection::North, *kind)
                .map_err(|e| e.to_string())
        })
        .collect()
}

pub(crate) fn guest_in_phase(park: &Park, phase: ServiceVisitPhase) -> Option<GuestId> {
    park.guests()
        .iter()
        .find(|g| {
            g.service_visit.as_ref().is_some_and(|v| {
                v.phase == phase
                    && (phase != ServiceVisitPhase::Using
                        || (v.progress > 0 && v.progress < SERVICE_USE_TICKS - 1))
                    && (phase != ServiceVisitPhase::Approaching
                        || (g.next.is_some() && g.progress > 0))
            })
        })
        .map(|g| g.id)
}

pub(crate) fn guest(park: &Park, id: GuestId) -> &Guest {
    park.guests()
        .iter()
        .find(|g| g.id == id)
        .expect("stable proof guest")
}

pub(crate) fn bytes(park: &Park) -> Result<Vec<u8>, String> {
    check(park)?;
    let bytes = park.to_save_bytes().map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if json["schema_version"] != 8
        || json["content_version"] != 8
        || Park::from_save_bytes(&bytes).map_err(|e| e.to_string())? != *park
    {
        return Err("service strict8 roundtrip changed authoritative state".into());
    }
    Ok(bytes)
}

pub(crate) fn set_price(park: &mut Park, id: ServiceId, price: u32) -> Result<(), String> {
    let before = park.clone();
    park.set_service_price(id, price)
        .map_err(|e| e.to_string())?;
    if park.guests() != before.guests()
        || park.cash_cents() != before.cash_cents()
        || park.ticks() != before.ticks()
        || park.ride() != before.ride()
        || park.service(id).unwrap().revenue_cents != before.service(id).unwrap().revenue_cents
    {
        return Err("service price edit changed active quote, balances or unrelated state".into());
    }
    check(park)
}

pub(crate) fn cancel_access(
    park: &mut Park,
    id: GuestId,
) -> Result<(TilePos, u8, ObjectId), String> {
    let old = guest(park, id).clone();
    let visit = old
        .service_visit
        .as_ref()
        .ok_or("cancellation needs a real visit")?;
    let path = *park
        .object(visit.access)
        .ok_or("service access is missing")?;
    let before = park.clone();
    park.remove_object(path.id)
        .ok_or("service access removal unexpectedly rejected")?;
    let after = guest(park, id);
    if after.service_visit.is_some()
        || after.wallet_cents != old.wallet_cents
        || after.spent_cents != old.spent_cents
        || after.service_spent_cents != old.service_spent_cents
        || after.services_used != old.services_used
        || after.needs != old.needs
        || park.cash_cents() != before.cash_cents()
        || park.ticks() != before.ticks()
        || park.services() != before.services()
    {
        return Err("access-loss cancellation charged, relieved or advanced a visit".into());
    }
    bytes(park)?;
    Ok((path.pos, path.height, path.id))
}

pub(crate) fn repair_access(
    park: &mut Park,
    removed: (TilePos, u8, ObjectId),
) -> Result<ObjectId, String> {
    let fresh = park
        .place_path(removed.0, removed.1)
        .map_err(|e| e.to_string())?;
    if fresh <= removed.2 {
        return Err("service access repair reused a removed object ID".into());
    }
    bytes(park)?;
    Ok(fresh)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Stage {
    LockedUse,
    AccessCancel,
    RepairUse,
    Demolish,
    Unaffordable,
    Recovery,
    RideReturn,
    Done,
}

fn continue_services(park: &mut Park) -> Result<(), String> {
    let started = park.ticks();
    let id = guest_in_phase(park, ServiceVisitPhase::Using)
        .ok_or("service primary needs occupied use")?;
    let initial = guest(park, id).clone();
    let initial_service = initial.service_visit.as_ref().unwrap().service;
    let quote = initial.service_visit.as_ref().unwrap().price_cents;
    set_price(park, initial_service, quote + 50)?;
    let mut stage = Stage::LockedUse;
    let mut transitions = vec![(stage, 0)];
    let mut repaired_service = initial_service;
    let mut repaired_count = 0;
    let mut refusals = 0;
    let mut recovery_customer = None;
    let mut completed = 0;
    for tick in 0..REPLAY_TICKS {
        let prior = stage;
        match stage {
            Stage::LockedUse if guest(park, id).services_used > initial.services_used => {
                if guest(park, id).service_spent_cents
                    != initial.service_spent_cents + u64::from(quote)
                {
                    return Err("price edit changed already-locked service payment".into());
                }
                stage = Stage::AccessCancel;
            }
            Stage::AccessCancel => {
                if let Some(id) = guest_in_phase(park, ServiceVisitPhase::Using) {
                    repaired_service = guest(park, id).service_visit.as_ref().unwrap().service;
                    repaired_count = park.service(repaired_service).unwrap().served;
                    let removed = cancel_access(park, id)?;
                    repair_access(park, removed)?;
                    stage = Stage::RepairUse;
                }
            }
            Stage::RepairUse if park.service(repaired_service).unwrap().served > repaired_count => {
                stage = Stage::Demolish;
            }
            Stage::Demolish => {
                let occupied_earned = park
                    .guests()
                    .iter()
                    .find(|guest| {
                        guest.service_visit.as_ref().is_some_and(|visit| {
                            visit.phase == ServiceVisitPhase::Using
                                && visit.progress > 0
                                && park.service(visit.service).is_some_and(|service| {
                                    service.served > 0 && service.revenue_cents > 0
                                })
                        })
                    })
                    .map(|guest| guest.id);
                if let Some(id) = occupied_earned {
                    let visit = guest(park, id).service_visit.as_ref().unwrap().clone();
                    let old = park.service(visit.service).unwrap().clone();
                    let object = *park.object(old.object.unwrap()).unwrap();
                    let before = park.clone();
                    park.remove_object(object.id)
                        .ok_or("occupied service demolition unexpectedly rejected")?;
                    let tombstone = park
                        .service(old.id)
                        .ok_or("demolition erased service history")?;
                    if tombstone.object.is_some()
                        || tombstone.served != old.served
                        || tombstone.revenue_cents != old.revenue_cents
                        || park.cash_cents() != before.cash_cents()
                        || guest(park, id).spent_cents != guest(&before, id).spent_cents
                        || guest(park, id).needs != guest(&before, id).needs
                        || guest(park, id).service_visit.is_some()
                    {
                        return Err("facility demolition lost history or charged/relieved a cancelled customer".into());
                    }
                    let new = park
                        .place_service(object.pos, object.height, old.facing, old.kind)
                        .map_err(|e| e.to_string())?;
                    if new <= old.id || park.service(new).unwrap().object.unwrap() <= object.id {
                        return Err("replacement facility reused an identity".into());
                    }
                    let ids: Vec<_> = park
                        .services()
                        .iter()
                        .filter(|s| s.object.is_some())
                        .map(|s| s.id)
                        .collect();
                    for id in ids {
                        set_price(park, id, 10_000)?;
                    }
                    refusals = park
                        .services()
                        .iter()
                        .map(|s| s.unaffordable_rejections)
                        .sum::<u64>();
                    stage = Stage::Unaffordable;
                }
            }
            Stage::Unaffordable
                if park
                    .services()
                    .iter()
                    .map(|s| s.unaffordable_rejections)
                    .sum::<u64>()
                    > refusals =>
            {
                let ids: Vec<_> = park
                    .services()
                    .iter()
                    .filter(|s| s.object.is_some())
                    .map(|s| s.id)
                    .collect();
                for id in ids {
                    set_price(park, id, 0)?;
                }
                stage = Stage::Recovery;
            }
            Stage::Recovery => {
                if let Some((id, used)) = recovery_customer {
                    if guest(park, id).services_used > used {
                        let ride = super::journey::install(park)?;
                        super::economy::set_price_checked(park, 0)?;
                        completed = park.ride_journey_stats(ride).completed;
                        park.set_train_open(true);
                        stage = Stage::RideReturn;
                    }
                } else {
                    recovery_customer = park
                        .guests()
                        .iter()
                        .find(|guest| {
                            guest.service_visit.as_ref().is_some_and(|visit| {
                                visit.phase == ServiceVisitPhase::Using && visit.price_cents == 0
                            })
                        })
                        .map(|guest| (guest.id, guest.services_used));
                }
            }
            Stage::RideReturn
                if park.ride_journey_stats(park.ride().unwrap().id).completed > completed =>
            {
                stage = Stage::Done;
            }
            _ => {}
        }
        if prior != stage {
            transitions.push((stage, tick));
            bytes(park)?;
        }
        advance_checked(park)?;
        if tick % 997 == 0 {
            bytes(park)?;
        }
    }
    if stage != Stage::Done || park.ticks() != started + REPLAY_TICKS {
        return Err(format!(
            "service replay incomplete: {stage:?}, transitions={transitions:?}, needs={:?}, services={:?}, ride={:?}",
            park.guest_needs_summary(),
            park.services(),
            park.ride()
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
    install(&mut park)?;
    seek(
        &mut park,
        |p| guest_in_phase(p, ServiceVisitPhase::Approaching).is_some(),
        "natural midapproach",
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
        |p| guest_in_phase(p, ServiceVisitPhase::Using).is_some(),
        "natural occupied service",
    )?;
    park.set_terrain_height(TilePos { x: 3, y: 24 }, 1)
        .map_err(|e| e.to_string())?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &bytes(&park)?,
    )?;
    continue_services(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &bytes(&park)?,
    )?;
    println!("Guest services continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState {
        guests_open: true,
        services_open: true,
        ..ViewState::default()
    };
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || !view.terrain_custom
        || view.guests_open
        || view.services_open
        || view.selected_service.is_some()
        || view.path_tool.mode != crate::path_tools::PathTool::Inspect
    {
        return Err(
            "service primary did not load exactly paused/Custom with transient panels reset".into(),
        );
    }
    continue_services(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("service fresh-process continuation diverged".into());
    }
    view.services_open = true;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
        || view.services_open
        || guest_in_phase(&park, ServiceVisitPhase::Approaching).is_none()
    {
        return Err("service backup did not preserve exact active approach paused".into());
    }
    let paused = park.clone();
    park.advance(Speed::Paused);
    if park != paused {
        return Err("paused needs/service visit changed".into());
    }
    seek(
        &mut park,
        |p| guest_in_phase(p, ServiceVisitPhase::Using).is_some(),
        "loaded approach reaches actual use",
    )?;
    let id = guest_in_phase(&park, ServiceVisitPhase::Using).unwrap();
    let prior = guest(&park, id).services_used;
    seek(
        &mut park,
        |p| guest(p, id).services_used > prior,
        "loaded use completes once",
    )?;
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("service verification changed source slots".into());
    }
    write_new(&store.directory().join("verification-services.txt"), format!("Parkbound {}\nExact miduse primary and midapproach backup: PASS\nContinuation: PASS ({REPLAY_TICKS} complete ticks with natural needs)\nPer-tick needs values and completed-use relief/locked-price payments: PASS\nReview and completed-use satisfaction include prior critical-crossing penalties: PASS\nAccess-loss cancellation, fresh path ID, demolition tombstones and fresh facility IDs: PASS\nUnaffordable refusal, free-price recovery and completed ride return: PASS\nCombined ride/service revenue, wallet and cash conservation: PASS\nPaused loading and modal reset: PASS\nStrict8 roundtrips and unchanged primary/backup bytes: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn services_modes_preserve_natural_use_relief_and_money_across_fresh_process_contract() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound services é {} {}",
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
        assert_eq!(run(args("write-services")), Some(AppExit::Success));
        assert_eq!(run(args("write-services")), Some(AppExit::error()));
        assert_eq!(run(args("verify-services")), Some(AppExit::Success));
        assert!(directory.join("verification-services.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }
}
