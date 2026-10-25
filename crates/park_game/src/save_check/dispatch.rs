//! Persisted loading waits and real paid dispatch through production controls.
use super::*;
use crate::ride_operations::{self, OperatingField as Field, OperationCommand as Command};
use park_sim::{RideDispatchPolicy, RideId, RideOperatingSettings};

const PAID_REPLAY_TICKS: u64 = 7_200;
const SAVED_EXTRA_WAIT: u16 = 17;

fn require(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn command(park: &mut Park, id: RideId, value: Command) -> Result<(), String> {
    let before = park.clone();
    let mut view = ViewState::default();
    if !ride_operations::perform_command(park, &mut view, id, value) {
        return Err(view
            .ride_operating_notice
            .unwrap_or("dispatch command failed".into()));
    }
    if value != Command::ToggleOpen {
        require(
            park.cash_cents() == before.cash_cents()
                && park.finance_ledger() == before.finance_ledger()
                && park.guests() == before.guests()
                && park.track_draft() == before.track_draft()
                && park.ticks() == before.ticks()
                && park
                    .all_rides()
                    .all(|ride| before.ride_by_id(ride.id) == Some(ride)),
            "dispatch adjustment changed money, guests, draft, clock or ride history",
        )?;
    }
    super::ride_choice::bytes(park)?;
    Ok(())
}

fn ids(park: &Park) -> Result<[RideId; 2], String> {
    Ok([
        park.ride().ok_or("missing dispatch coaster")?.id,
        park.carousels()
            .first()
            .ok_or("missing dispatch carousel")?
            .ride
            .id,
    ])
}

fn roundtrip(park: &mut Park) -> Result<(), String> {
    *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn install_empty(park: &mut Park) -> Result<[RideId; 2], String> {
    // An empty loading test must still meet player opening rules. Connect both
    // rides and use an unaffordable fare to keep guests out of this wait fixture.
    let (coaster, carousel) = super::ride_choice::install(park)?;
    for id in [coaster, carousel] {
        park.set_ride_open_checked(id, false)
            .map_err(|e| e.to_string())?;
        park.set_ride_ticket_price(id, park_sim::MAX_TICKET_PRICE_CENTS)
            .map_err(|e| e.to_string())?;
    }
    super::ride_choice::seek(
        park,
        |p| {
            [coaster, carousel]
                .iter()
                .all(|id| p.ride_operating_settings_status(*id).is_ok())
        },
        "both connected rides closed and empty",
    )?;
    for (id, minimum, extra_adjustment) in [(coaster, 6, -28), (carousel, 3, -27)] {
        command(park, id, Command::Adjust(Field::StationWait, -2))?;
        command(park, id, Command::Adjust(Field::MinimumPassengers, minimum))?;
        command(
            park,
            id,
            Command::Adjust(Field::MaxExtraWait, extra_adjustment),
        )?;
        require(
            park.ride_opening_status(id).is_ok(),
            "empty dispatch fixture must retain valid access",
        )?;
    }
    require(
        park.ride_dispatch_policy(coaster)
            == Some(RideDispatchPolicy {
                minimum_passengers: 6,
                maximum_extra_wait_ticks: 60,
            })
            && park.ride_dispatch_policy(carousel)
                == Some(RideDispatchPolicy {
                    minimum_passengers: 3,
                    maximum_extra_wait_ticks: 90,
                }),
        "independent dispatch controls produced unexpected policies",
    )?;
    Ok([coaster, carousel])
}

pub(crate) fn open_empty_wait(park: &mut Park, owners: [RideId; 2]) -> Result<(), String> {
    for id in owners {
        command(park, id, Command::ToggleOpen)?;
    }
    super::ride_choice::seek(
        park,
        |p| {
            p.ride_dispatch_progress(owners[0])
                .is_some_and(|v| v.extra_waited_ticks == SAVED_EXTRA_WAIT)
                && owners.iter().all(|id| {
                    p.ride_dispatch_progress(*id).is_some_and(|v| {
                        v.waiting_for_load
                            && v.extra_waited_ticks > 0
                            && v.extra_waited_ticks < v.maximum_extra_wait_ticks
                    })
                })
        },
        "both empty rides partway through their extra loading waits",
    )?;
    require(
        owners.iter().all(|id| {
            park.ride_dispatch_progress(*id)
                .is_some_and(|v| v.seated_passengers == 0)
        }),
        "empty fixture admitted a passenger",
    )
}

fn replay_empty(park: &mut Park) -> Result<(), String> {
    let owners = ids(park)?;
    let remaining: Vec<_> = owners
        .iter()
        .map(|id| {
            let progress = park
                .ride_dispatch_progress(*id)
                .ok_or("missing loading progress")?;
            require(
                progress.waiting_for_load
                    && progress.extra_waited_ticks > 0
                    && progress.extra_waited_ticks < progress.maximum_extra_wait_ticks,
                "primary must resume partway through both extra waits",
            )?;
            Ok(u32::from(progress.maximum_extra_wait_ticks - progress.extra_waited_ticks) + 1)
        })
        .collect::<Result<_, String>>()?;
    // Closing while still waiting clears only those loading intervals.
    let mut cancelled = park.clone();
    for id in owners {
        command(&mut cancelled, id, Command::ToggleOpen)?;
        require(
            !cancelled.ride_is_open(id)
                && !cancelled.ride_is_running(id)
                && cancelled
                    .ride_dispatch_progress(id)
                    .is_some_and(|p| p.extra_waited_ticks == 0),
            "closing a waiting ride retained its extra wait",
        )?;
    }
    roundtrip(&mut cancelled)?;
    let revenue = owners.map(|id| park.ride_by_id(id).unwrap().revenue_cents);
    let mut departures = [None; 2];
    for elapsed in 1..=remaining.iter().copied().max().unwrap() {
        super::ride_choice::advance_checked(park)?;
        for (index, id) in owners.into_iter().enumerate() {
            if park.ride_is_running(id) && departures[index].is_none() {
                departures[index] = Some(elapsed);
                require(
                    park.ride_dispatch_progress(id).unwrap().extra_waited_ticks == 0,
                    "departure did not reset its extra wait",
                )?;
            }
        }
        if elapsed % 17 == 0 {
            roundtrip(park)?;
        }
    }
    require(
        departures == [Some(remaining[0]), Some(remaining[1])],
        "resumed empty rides departed at the wrong timeout ticks",
    )?;
    require(
        owners
            .into_iter()
            .enumerate()
            .all(|(index, id)| park.ride_by_id(id).unwrap().revenue_cents == revenue[index]),
        "empty timeout dispatch earned a fare",
    )?;
    for id in owners {
        command(park, id, Command::ToggleOpen)?;
    }
    super::ride_choice::seek(
        park,
        |p| {
            owners
                .iter()
                .all(|id| p.ride_operating_settings_status(*id).is_ok())
        },
        "timed-out rides finish closed and empty",
    )?;
    let other_policy = park.ride_dispatch_policy(owners[1]);
    let other_settings = park.ride_operating_settings(owners[1]);
    command(park, owners[0], Command::ResetDefaults)?;
    require(
        park.ride_dispatch_policy(owners[0]) == Some(RideDispatchPolicy::default())
            && park.ride_operating_settings(owners[0]) == Some(RideOperatingSettings::default())
            && park.ride_dispatch_policy(owners[1]) == other_policy
            && park.ride_operating_settings(owners[1]) == other_settings,
        "reset changed another ride or failed to reset both operating domains",
    )?;
    roundtrip(park)
}

pub(crate) fn install_paid(park: &mut Park) -> Result<[RideId; 2], String> {
    let (coaster, carousel) = super::operations::install(park)?;
    for (id, extra_adjustment) in [(coaster, -24), (carousel, -20)] {
        command(park, id, Command::Adjust(Field::MinimumPassengers, 1))?;
        command(
            park,
            id,
            Command::Adjust(Field::MaxExtraWait, extra_adjustment),
        )?;
    }
    Ok([coaster, carousel])
}

pub(crate) fn open_paid(park: &mut Park, owners: [RideId; 2]) -> Result<(), String> {
    for id in owners {
        command(park, id, Command::ToggleOpen)?;
    }
    super::ride_choice::seek(
        park,
        |p| {
            owners
                .iter()
                .all(|id| super::ride_choice::paid_running(p, *id))
        },
        "both minimum-load rides carry paid passengers",
    )
}

fn replay_paid(park: &mut Park) -> Result<(), String> {
    let owners = ids(park)?;
    for id in owners {
        require(
            super::ride_choice::paid_running(park, id),
            "paid primary needs occupied running rides",
        )?;
        for value in [
            Command::Adjust(Field::MinimumPassengers, 1),
            Command::Adjust(Field::MaxExtraWait, 1),
            Command::ResetDefaults,
        ] {
            let before = park.clone();
            let bytes = super::ride_choice::bytes(park)?;
            let mut view = ViewState::default();
            require(
                !ride_operations::perform_command(park, &mut view, id, value),
                "occupied loading-policy command was accepted",
            )?;
            require(
                *park == before && super::ride_choice::bytes(park)? == bytes,
                "occupied policy refusal changed authoritative state",
            )?;
        }
    }
    let counters = owners.map(|id| {
        let ride = park.ride_by_id(id).unwrap();
        (ride.completed, ride.revenue_cents)
    });
    let start = park.ticks();
    for tick in 0..PAID_REPLAY_TICKS {
        super::ride_choice::advance_checked(park)?;
        if tick % 257 == 0 {
            roundtrip(park)?;
        }
    }
    require(
        park.ticks() == start + PAID_REPLAY_TICKS
            && owners.into_iter().enumerate().all(|(index, id)| {
                let ride = park.ride_by_id(id).unwrap();
                ride.completed > counters[index].0 && ride.revenue_cents > counters[index].1
            }),
        "minimum-load rides did not complete further paid journeys",
    )?;
    super::ride_choice::check(park)
}

fn action(
    session: &SaveSession,
    park: &mut Park,
    view: &mut ViewState,
    value: SaveAction,
) -> Result<(), String> {
    if !persistence::perform_action(session, park, view, value) {
        return Err(view.save_notice.clone());
    }
    if value != SaveAction::Save {
        require(
            view.speed == Speed::Paused && view.ride_operating_commands.is_empty(),
            "dispatch load was not paused with cleared commands",
        )?;
    }
    Ok(())
}

fn paid_session(directory: &Path) -> SaveSession {
    SaveSession::from_arguments([
        OsString::from("--save-dir"),
        directory.join("paid").into_os_string(),
    ])
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let owners = install_empty(&mut park)?;
    let mut view = ViewState::default();
    action(session, &mut park, &mut view, SaveAction::Save)?;
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    open_empty_wait(&mut park, owners)?;
    action(session, &mut park, &mut view, SaveAction::Save)?;
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    replay_empty(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;

    let paid_session = paid_session(store.directory());
    let paid_store = paid_session.store.as_ref().ok_or("invalid paid profile")?;
    fs::create_dir(paid_store.directory()).map_err(|e| e.to_string())?;
    let mut paid = demonstration_park();
    let owners = install_paid(&mut paid)?;
    action(&paid_session, &mut paid, &mut view, SaveAction::Save)?;
    write_new(
        &paid_store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&paid)?,
    )?;
    open_paid(&mut paid, owners)?;
    action(&paid_session, &mut paid, &mut view, SaveAction::Save)?;
    write_new(
        &paid_store.directory().join("expected-primary.park.json"),
        &super::ride_choice::bytes(&paid)?,
    )?;
    replay_paid(&mut paid)?;
    write_new(
        &paid_store.directory().join("expected-after.park.json"),
        &super::ride_choice::bytes(&paid)?,
    )?;
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let paid_session = paid_session(store.directory());
    let paid_store = paid_session.store.as_ref().ok_or("invalid paid profile")?;
    let paths = [
        store.save_path(),
        store.backup_path(),
        paid_store.save_path(),
        paid_store.backup_path(),
    ];
    let originals = paths
        .iter()
        .map(fs::read)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    for (active, paid) in [(session, false), (&paid_session, true)] {
        let active_store = active.store.as_ref().unwrap();
        action(active, &mut park, &mut view, SaveAction::Load)?;
        require(
            park == read_expected(&active_store.directory().join("expected-primary.park.json"))?,
            "dispatch primary differs after fresh-process load",
        )?;
        if paid {
            replay_paid(&mut park)?;
        } else {
            replay_empty(&mut park)?;
        }
        require(
            park == read_expected(&active_store.directory().join("expected-after.park.json"))?,
            "dispatch continuation differs after fresh-process load",
        )?;
        action(active, &mut park, &mut view, SaveAction::Backup)?;
        require(
            park == read_expected(&active_store.directory().join("expected-backup.park.json"))?,
            "independent closed-policy backup differs",
        )?;
        let owners = ids(&park)?;
        if paid {
            open_paid(&mut park, owners)?;
        } else {
            open_empty_wait(&mut park, owners)?;
        }
        require(
            park == read_expected(&active_store.directory().join("expected-primary.park.json"))?,
            "closed backup did not reproduce its waiting/paid primary",
        )?;
    }
    for (path, original) in paths.iter().zip(originals) {
        require(
            fs::read(path).map_err(|e| e.to_string())? == original,
            "dispatch verification changed a source slot",
        )?;
    }
    write_new(&store.directory().join("verification-dispatch.txt"),format!(
        "Parkbound {}\nIndependent closed loading policies and positive mid-wait timers survive restart: PASS\nExact remaining timeout ticks, empty departures, close/reset and strict roundtrips: PASS\nReset preserves the other ride's policy and equipment settings: PASS\nOccupied minimum/maximum/reset policy changes refuse atomically: PASS\nBoth minimum-load rides complete further paid journeys over {PAID_REPLAY_TICKS} checked ticks: PASS\nOwner, seat, fare, review and finance invariants checked on every replay tick: PASS\nBoth closed backups recreate their exact waiting/paid primaries: PASS\nAll four original primary/backup slots unchanged: PASS\nNative pointer gestures: NOT CLAIMED\n",env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_proof_resumes_waits_and_paid_journeys_without_rewriting_any_slot() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound dispatch proof é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-dispatch", "verify-dispatch"] {
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
        let receipt = fs::read_to_string(directory.join("verification-dispatch.txt")).unwrap();
        assert!(receipt.contains("Exact remaining timeout ticks"));
        assert!(receipt.contains("All four original primary/backup slots unchanged: PASS"));
        fs::remove_dir_all(directory).unwrap();
    }
}
