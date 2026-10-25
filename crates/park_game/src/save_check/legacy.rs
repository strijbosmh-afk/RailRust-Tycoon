//! Migration preserves old state at load; future decisions use the current rules.
use super::*;
use park_sim::{GuestNeeds, GuestPreferences, GuestThought, MAX_SAVE_BYTES};
use serde_json::Value;
use std::io::Read;

const CONTINUATION_TICKS: u64 = 3_600;

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|error| error.to_string())?
        .take(MAX_SAVE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_SAVE_BYTES {
        return Err("legacy source exceeds save byte cap".into());
    }
    Ok(bytes)
}

/// Compare original data directly, rather than constructing the expected state
/// with the same migration code being checked. New fields may be added only;
/// every existing value, array member and array order must survive unchanged.
fn require_old_fields(old: &Value, current: &Value, location: &str) -> Result<(), String> {
    match (old, current) {
        (Value::Object(old), Value::Object(current)) => {
            for (key, value) in old {
                let target = current
                    .get(key)
                    .ok_or_else(|| format!("migration dropped {location}/{key}"))?;
                require_old_fields(value, target, &format!("{location}/{key}"))?;
            }
        }
        (Value::Array(old), Value::Array(current)) if old.len() == current.len() => {
            for (index, (a, b)) in old.iter().zip(current).enumerate() {
                require_old_fields(a, b, &format!("{location}/{index}"))?;
            }
        }
        _ if old == current => {}
        _ => return Err(format!("migration changed original field {location}")),
    }
    Ok(())
}

fn require_defaults(park: &Park, old: &Value) -> Result<(), String> {
    if old["schema_version"].as_u64().unwrap() <= 18
        && park
            .ride()
            .is_some_and(|ride| park.coaster_car_count(ride.id) != Some(3))
    {
        return Err("legacy train did not retain three cars and six seats".into());
    }
    let version = old["schema_version"].as_u64().unwrap();
    let seed = old["park"]["crowd_seed"].as_u64().unwrap_or(0);
    for guest in park.guests() {
        if old["schema_version"].as_u64().unwrap() <= 6
            && (guest.preferences != GuestPreferences::for_guest(seed, guest.id)
                || guest.satisfaction != 70
                || guest.thought != GuestThought::Exploring
                || guest.thought_tick != 0
                || guest.ratings_given != 0
                || guest.rating_total != 0
                || guest.last_rating.is_some())
        {
            return Err(
                "legacy feedback defaults changed history or drew inconsistent traits".into(),
            );
        }
        if old["schema_version"].as_u64().unwrap() <= 7
            && (guest.needs != GuestNeeds::for_guest(seed, guest.id)
                || guest.service_visit.is_some()
                || guest.service_cooldown != 0
                || guest.service_spent_cents != 0
                || guest.services_used != 0)
        {
            return Err(
                "legacy service defaults invented usage, money or inconsistent needs".into(),
            );
        }
    }
    if old["schema_version"].as_u64().unwrap() <= 7 && !park.services().is_empty() {
        return Err("migration invented service facilities/history".into());
    }
    if version <= 9 && !park.carousels().is_empty() {
        return Err("migration invented a second ride".into());
    }
    for guest in park.guests().iter().filter(|_| version <= 9) {
        let ride_spent = guest.spent_cents - guest.service_spent_cents;
        if park.ride().is_some() {
            let history = guest
                .ride_history
                .first()
                .ok_or("missing legacy ride history")?;
            if guest.ride_history.len() != 1
                || Some(history.ride) != park.ride().map(|r| r.id)
                || history.spent_cents != ride_spent
                || history.completed != guest.rides_completed
                || history.rating_count != guest.ratings_given
                || history.rating_total != guest.rating_total
            {
                return Err("legacy guest history changed owner, receipts or reviews".into());
            }
        } else if !guest.ride_history.is_empty() {
            return Err("legacy migration invented guest ride history".into());
        }
    }
    let ledger = park.finance_ledger();
    if old["schema_version"].as_u64().unwrap() <= 8 {
        if ledger.opening_tick != park.ticks()
            || ledger.opening_cash_cents != park.cash_cents()
            || ledger.opening_ride_revenue_cents != park.ride().map_or(0, |r| r.revenue_cents)
            || !ledger.entries.is_empty()
            || ledger.next_entry_id != 1
            || ledger.archived_entries != 0
            || ledger.archived_through_tick != park.ticks()
            || !ledger.archived_services.is_empty()
            || ledger.archived != park_sim::FinanceTotals::default()
            || ledger.opening_services.len() != park.services().len()
        {
            return Err(
                "legacy migration invented ledger transactions or changed opening balances".into(),
            );
        }
        for service in park.services() {
            let opening = ledger
                .opening_services
                .iter()
                .find(|s| s.service == service.id)
                .ok_or("legacy service lacks opening baseline")?;
            if opening.revenue_cents != service.revenue_cents
                || opening.served != service.served
                || service.upkeep_progress != 0
                || service.upkeep_cents != 0
                || service.supplies_cents != 0
                || service.closed_for_funds
            {
                return Err(
                    "legacy service finance defaults changed history or invented expenses".into(),
                );
            }
        }
    }
    for entry in ledger.entries.iter().filter(|_| version <= 9) {
        let owner = if entry.kind == park_sim::FinanceKind::RideRevenue {
            park.ride().map(|r| r.id)
        } else {
            None
        };
        if entry.ride != owner {
            return Err("legacy transaction attributed to the wrong ride".into());
        }
    }
    if version <= 10
        && (park
            .carousels()
            .iter()
            .any(|ride| ride.construction_cost_cents != 0 || ride.demolition_refund_cents != 0)
            || ledger.archived.construction_cents != 0
            || ledger.archived.demolition_refunds_cents != 0
            || ledger
                .opening_rides
                .iter()
                .chain(&ledger.archived_rides)
                .any(|r| r.construction_cents != 0 || r.demolition_refunds_cents != 0)
            || ledger.entries.iter().any(|e| {
                matches!(
                    e.kind,
                    park_sim::FinanceKind::RideConstruction
                        | park_sim::FinanceKind::RideDemolitionRefund
                )
            }))
    {
        return Err(
            "migration invented construction charges or a refund basis for free legacy rides"
                .into(),
        );
    }
    if version <= 11
        && (park
            .services()
            .iter()
            .any(|s| s.construction_cost_cents != 0 || s.demolition_refund_cents != 0)
            || ledger
                .archived_services
                .iter()
                .any(|s| s.construction_cents != 0 || s.demolition_refunds_cents != 0)
            || ledger.entries.iter().any(|e| {
                matches!(
                    e.kind,
                    park_sim::FinanceKind::ServiceConstruction
                        | park_sim::FinanceKind::ServiceDemolitionRefund
                )
            }))
    {
        return Err("legacy service import invented paid construction or refunds".into());
    }
    if version <= 12 && !park.track_draft().is_empty() {
        return Err("legacy import invented a construction draft".into());
    }
    if version <= 14
        && (!park.track_economy().paid_pieces.is_empty()
            || park.track_economy().construction_cents != 0
            || park.track_economy().refund_cents != 0
            || ledger.entries.iter().any(|entry| {
                matches!(
                    entry.kind,
                    park_sim::FinanceKind::TrackConstruction | park_sim::FinanceKind::TrackRefund
                )
            }))
    {
        return Err("legacy import invented paid track, charges or refund history".into());
    }
    if version <= 14
        && park
            .objects()
            .iter()
            .filter(|object| object.kind == park_sim::ObjectKind::Track)
            .any(|object| park.track_demolition_refund_cents(object.id) != 0)
    {
        return Err("free imported track has a positive demolition quote".into());
    }
    if version <= 15
        && park.all_rides().any(|ride| {
            park.ride_operating_settings(ride.id)
                .is_some_and(|settings| settings != park_sim::RideOperatingSettings::default())
        })
    {
        return Err("legacy import invented non-default ride settings".into());
    }
    if version <= 16
        && park.all_rides().any(|ride| {
            park.ride_dispatch_policy(ride.id) != Some(park_sim::RideDispatchPolicy::default())
                || park
                    .ride_dispatch_progress(ride.id)
                    .is_none_or(|progress| progress.extra_waited_ticks != 0)
        })
    {
        return Err("legacy import invented loading policies or elapsed loading wait".into());
    }
    super::finance::check(park)?;
    if old["schema_version"].as_u64().unwrap() <= 6
        && let Some(ride) = park.ride()
    {
        let stats = park.ride_experience_stats(ride.id);
        if stats.rating_count != 0
            || stats.rating_total != 0
            || stats.average_rating_tenths.is_some()
            || stats.value_rejections != 0
        {
            return Err("migration invented ride feedback history".into());
        }
    }
    Ok(())
}

fn continue_current_rules(mut uninterrupted: Park) -> Result<(), String> {
    let started = uninterrupted.ticks();
    let mut reloaded = Park::from_save_bytes(
        &uninterrupted
            .to_save_bytes()
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    for tick in 0..CONTINUATION_TICKS {
        for park in [&mut uninterrupted, &mut reloaded] {
            if tick == 100 {
                park.set_train_open(false);
            }
            if tick == 300
                && let Some(ride) = park.ride()
            {
                park.set_ride_ticket_price(ride.id, 0)
                    .map_err(|error| error.to_string())?;
            }
            if tick == 600 {
                park.set_train_open(true);
            }
            if !park.carousels().is_empty() {
                park.advance(Speed::Normal);
                super::ride_choice::check(park)?;
            } else if park.ride().is_some() {
                super::services::advance_checked(park)?;
            } else {
                park.advance(Speed::Normal);
            }
        }
        if tick % 257 == 0 {
            reloaded = Park::from_save_bytes(
                &reloaded
                    .to_save_bytes()
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        }
        if uninterrupted != reloaded {
            return Err(format!(
                "current-rule continuation diverged after migrated tick {tick}"
            ));
        }
    }
    if uninterrupted.ticks() != started + CONTINUATION_TICKS {
        return Err("migration continuation missed complete ticks".into());
    }
    Ok(())
}

fn check_free_legacy_demolition(source: &Park) -> Result<(), String> {
    let mut park = source.clone();
    for id in source
        .carousels()
        .iter()
        .filter(|c| !c.objects.is_empty() && c.construction_cost_cents == 0)
        .map(|c| c.ride.id)
    {
        park.set_ride_open(id, false).map_err(|e| e.to_string())?;
        for _ in 0..48_000 {
            if !park
                .guests()
                .iter()
                .any(|g| g.ride_visit.as_ref().is_some_and(|v| v.ride == id))
            {
                break;
            }
            park.advance(Speed::Normal);
        }
        let before = park.clone();
        if park
            .carousel_demolition_quote(id)
            .map_err(|e| e.to_string())?
            != 0
        {
            return Err("free legacy body acquired a positive refund".into());
        }
        park.remove_carousel(id).map_err(|e| e.to_string())?;
        if park.cash_cents() != before.cash_cents()
            || park.finance_ledger() != before.finance_ledger()
            || park.carousel(id).unwrap().demolition_refund_cents != 0
            || park.guests() != before.guests()
        {
            return Err(
                "legacy free-body removal created money, a transaction or changed guests".into(),
            );
        }
        super::ride_choice::bytes(&park)?;
    }
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let source = [
        (SaveAction::Load, store.save_path()),
        (SaveAction::Backup, store.backup_path()),
    ];
    let mut versions = Vec::new();
    let mut originals = Vec::new();
    for (action, path) in source {
        let bytes = read_bounded(&path)?;
        let old: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        let schema = old["schema_version"]
            .as_u64()
            .ok_or("legacy schema must be an integer")?;
        if !(1..=19).contains(&schema)
            || old["content_version"].as_u64() != Some(schema)
            || old["format"] != "parkbound-save"
        {
            return Err(
                "verify-legacy requires an authentic strict version 1..19 source slot".into(),
            );
        }
        let mut park = demonstration_park();
        let mut view = ViewState {
            guests_open: true,
            selected_guest: park.guests().last().map(|guest| guest.id),
            guest_page: 1,
            ..ViewState::default()
        };
        if !persistence::perform_action(session, &mut park, &mut view, action) {
            return Err(view.save_notice);
        }
        if view.speed != Speed::Paused
            || view.guests_open
            || view.selected_guest.is_some()
            || view.guest_page != 0
            || view.path_tool.mode != crate::path_tools::PathTool::Inspect
        {
            return Err("legacy load must pause and reset transient inspection".into());
        }
        let modern_bytes = park.to_save_bytes().map_err(|error| error.to_string())?;
        let modern: Value =
            serde_json::from_slice(&modern_bytes).map_err(|error| error.to_string())?;
        if modern["schema_version"] != park_sim::SAVE_SCHEMA_VERSION
            || modern["content_version"] != park_sim::SAVE_CONTENT_VERSION
            || (schema <= 7 && modern["park"]["next_service_id"] != 1)
        {
            return Err(
                "migration did not produce current-schema state with correct service allocation"
                    .into(),
            );
        }
        require_old_fields(&old["park"], &modern["park"], "/park")?;
        require_defaults(&park, &old)?;
        if Park::from_save_bytes(&bytes).map_err(|error| error.to_string())? != park {
            return Err("repeated legacy loading changed deterministic defaults".into());
        }
        check_free_legacy_demolition(&park)?;
        let mut free_services = park.clone();
        for id in park
            .services()
            .iter()
            .filter(|s| s.object.is_some() && s.construction_cost_cents == 0)
            .map(|s| s.id)
        {
            super::facility_construction::demolish(&mut free_services, id)?;
        }
        continue_current_rules(park)?;
        versions.push(schema);
        originals.push((path, bytes));
    }
    for (path, bytes) in originals {
        if read_bounded(&path)? != bytes {
            return Err("legacy verification changed an original slot".into());
        }
    }
    write_new(&store.directory().join("verification-legacy.txt"), format!("Parkbound {}\nOriginal primary/backup schemas: {versions:?}\nEvery original park field preserved at load (including RNG, IDs, money, needs/services and active journeys): PASS\nOnly newly added defaults initialized; original feedback/services preserved: PASS\nLegacy free carousel/facility basis and actual demolition refunds stay zero: PASS\nLegacy opening baselines preserved; v9-v12 transactions, archives, expenses, paid ride/facility costs and source attribution preserved: PASS\nProduction loads paused/Inspect with guest panel reset: PASS\nDeterministic current-version continuation: PASS ({CONTINUATION_TICKS} ticks per slot, commands and repeated current-schema roundtrips)\nOriginal primary and backup bytes unchanged: PASS\nLegacy future equivalence: NOT CLAIMED; imported parks continue under current needs/service/cost/ride-choice/track/dispatch/appeal rules; v1-v12 imports have an empty draft; v13 retains its level draft and v14 retains graded drafts; v1-v14 imported track basis and refunds stay zero; v15-v17 paid track basis and transactions are preserved; v1-v15 ride settings default to original timings; v16-v17 operating settings are preserved; v1-v16 loading policies default to no minimum and zero elapsed extra wait; v17 loading policies and elapsed waits are preserved; schema 17 body fields are unchanged and new appeal traits are derived without RNG draws\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_v17_waiting_and_paid_dispatch_states_preserve_every_field() {
        for paid in [false, true] {
            let directory = std::env::temp_dir().join(format!(
                "Parkbound legacy dispatch é {} {} {paid}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&directory).unwrap();
            let mut park = demonstration_park();
            let owners = if paid {
                let owners = super::super::dispatch::install_paid(&mut park).unwrap();
                super::super::dispatch::open_paid(&mut park, owners).unwrap();
                owners
            } else {
                let owners = super::super::dispatch::install_empty(&mut park).unwrap();
                super::super::dispatch::open_empty_wait(&mut park, owners).unwrap();
                owners
            };
            for id in owners {
                let progress = park.ride_dispatch_progress(id).unwrap();
                assert!(progress.minimum_passengers > 0);
                if paid {
                    assert!(super::super::ride_choice::paid_running(&park, id));
                } else {
                    assert!(progress.waiting_for_load && progress.extra_waited_ticks > 0);
                }
            }
            // Schema 17 predates the car-count field. Project the older
            // body and headers; external proofs also use genuine old files.
            let mut old: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
            old["schema_version"] = json!(17);
            crate::legacy_cost_test_support::project_old_fixture(&mut old);
            old["park"].as_object_mut().unwrap().remove("coaster_cars");
            old["content_version"] = json!(17);
            let bytes = serde_json::to_vec(&old).unwrap();
            let imported = Park::from_save_bytes(&bytes).unwrap();
            let current: Value =
                serde_json::from_slice(&imported.to_save_bytes().unwrap()).unwrap();
            require_old_fields(&old["park"], &current["park"], "park").unwrap();
            assert!(
                imported.all_rides().all(|r| imported
                    .ride_running_costs(r.id)
                    .unwrap()
                    .upkeep_cents
                    == 0)
            );
            for name in ["park.park.json", "park.park.json.bak"] {
                fs::write(directory.join(name), &bytes).unwrap();
            }
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    "verify-legacy".into(),
                    "--save-dir".into(),
                    directory.clone().into_os_string()
                ]),
                Some(AppExit::Success)
            );
            for name in ["park.park.json", "park.park.json.bak"] {
                assert_eq!(fs::read(directory.join(name)).unwrap(), bytes);
            }
            assert!(directory.is_absolute() && directory.starts_with(std::env::temp_dir()));
            fs::remove_dir_all(directory).unwrap();
        }
    }

    #[test]
    fn legacy_v16_variable_operating_settings_survive_dispatch_migration() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound legacy operations é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let mut park = demonstration_park();
        let (coaster, carousel) = super::super::operations::install(&mut park).unwrap();
        for id in [coaster, carousel] {
            park.set_ride_open(id, true).unwrap();
        }
        super::super::ride_choice::seek(
            &mut park,
            |p| {
                super::super::ride_choice::paid_running(p, coaster)
                    && super::super::ride_choice::paid_running(p, carousel)
            },
            "both legacy tuned rides carry paid passengers",
        )
        .unwrap();
        let mut old: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        old["schema_version"] = json!(16);
        crate::legacy_cost_test_support::project_old_fixture(&mut old);
        old["park"].as_object_mut().unwrap().remove("coaster_cars");
        old["content_version"] = json!(16);
        old["park"].as_object_mut().unwrap().remove("ride_dispatch");
        assert!(
            !old["park"]["ride_operating_settings"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let bytes = serde_json::to_vec(&old).unwrap();
        for name in ["park.park.json", "park.park.json.bak"] {
            fs::write(directory.join(name), &bytes).unwrap();
        }
        assert_eq!(
            run(vec![
                "--save-check".into(),
                "verify-legacy".into(),
                "--save-dir".into(),
                directory.clone().into_os_string()
            ]),
            Some(AppExit::Success)
        );
        for name in ["park.park.json", "park.park.json.bak"] {
            assert_eq!(fs::read(directory.join(name)).unwrap(), bytes);
        }
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn legacy_v15_paid_track_keeps_its_basis_and_passengers_during_continuation() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound legacy paid track é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let mut park = demonstration_park();
        super::super::track_construction::install_access(&mut park).unwrap();
        super::super::track_construction::draft(&mut park, true).unwrap();
        super::super::track_construction::apply(&mut park).unwrap();
        super::super::track_construction::occupied(&mut park).unwrap();
        assert!(park.track_economy().construction_cents > 0);
        assert!(!park.track_economy().paid_pieces.is_empty());
        let mut old: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        old["schema_version"] = json!(15);
        crate::legacy_cost_test_support::project_old_fixture(&mut old);
        old["park"].as_object_mut().unwrap().remove("coaster_cars");
        old["content_version"] = json!(15);
        old["park"].as_object_mut().unwrap().remove("ride_dispatch");
        old["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_operating_settings");
        let bytes = serde_json::to_vec(&old).unwrap();
        for name in ["park.park.json", "park.park.json.bak"] {
            fs::write(directory.join(name), &bytes).unwrap();
        }
        assert_eq!(
            run(vec![
                "--save-check".into(),
                "verify-legacy".into(),
                "--save-dir".into(),
                directory.clone().into_os_string()
            ]),
            Some(AppExit::Success)
        );
        for name in ["park.park.json", "park.park.json.bak"] {
            assert_eq!(fs::read(directory.join(name)).unwrap(), bytes);
        }
        fs::remove_dir_all(directory).unwrap();
    }

    fn strip_ride_choices(value: &mut Value) {
        value["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_dispatch");
        value["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_operating_settings");
        value["park"].as_object_mut().unwrap().remove("track_draft");
        value["park"]
            .as_object_mut()
            .unwrap()
            .remove("track_economy");
        value["park"].as_object_mut().unwrap().remove("carousels");
        for guest in value["park"]["guests"].as_array_mut().unwrap() {
            guest.as_object_mut().unwrap().remove("ride_history");
        }
        if let Some(ledger) = value["park"]["finances"].as_object_mut() {
            ledger.remove("opening_rides");
            ledger.remove("archived_rides");
            for entry in ledger["entries"].as_array_mut().unwrap() {
                entry.as_object_mut().unwrap().remove("ride");
            }
        }
    }
    fn strip_finances(value: &mut Value) {
        strip_ride_choices(value);
        value["park"].as_object_mut().unwrap().remove("finances");
        let mut costs = 0;
        for service in value["park"]["services"].as_array_mut().unwrap() {
            costs += service["upkeep_cents"].as_u64().unwrap()
                + service["supplies_cents"].as_u64().unwrap()
                + service["construction_cost_cents"].as_u64().unwrap()
                - service["demolition_refund_cents"].as_u64().unwrap();
            for field in [
                "upkeep_progress",
                "upkeep_cents",
                "supplies_cents",
                "closed_for_funds",
                "construction_cost_cents",
                "demolition_refund_cents",
            ] {
                service.as_object_mut().unwrap().remove(field);
            }
        }
        value["park"]["cash_cents"] = json!(value["park"]["cash_cents"].as_u64().unwrap() + costs);
    }
    fn strip_services(value: &mut Value) {
        strip_finances(value);
        value["park"].as_object_mut().unwrap().remove("services");
        value["park"]
            .as_object_mut()
            .unwrap()
            .remove("next_service_id");
        for guest in value["park"]["guests"].as_array_mut().unwrap() {
            for field in [
                "needs",
                "service_visit",
                "service_cooldown",
                "service_spent_cents",
                "services_used",
            ] {
                guest.as_object_mut().unwrap().remove(field);
            }
        }
    }
    #[test]
    fn legacy_projection_rejects_dropped_mutated_or_reordered_old_fields() {
        let old = json!({"rng":99,"money":7,"guests":[{"id":2},{"id":1}],"visit":null});
        let added = json!({"rng":99,"money":7,"guests":[{"id":2,"new":true},{"id":1}],"visit":null,"new":0});
        assert!(require_old_fields(&old, &added, "/park").is_ok());
        for changed in [
            json!({"rng":98,"money":7,"guests":[{"id":2},{"id":1}],"visit":null}),
            json!({"rng":99,"guests":[{"id":2},{"id":1}],"visit":null}),
            json!({"rng":99,"money":7,"guests":[{"id":1},{"id":2}],"visit":null}),
        ] {
            assert!(require_old_fields(&old, &changed, "/park").is_err());
        }
    }
    #[test]
    fn legacy_v8_preserves_needs_service_history_and_active_use_with_new_finance_baseline() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound legacy service é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let mut park = demonstration_park();
        super::super::services::install(&mut park).unwrap();
        super::super::services::seek(
            &mut park,
            |p| {
                p.guest_needs_summary().services_used > 0
                    && super::super::services::guest_in_phase(p, park_sim::ServiceVisitPhase::Using)
                        .is_some()
            },
            "earned v8 service history and occupied use",
        )
        .unwrap();
        let mut value: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        // Synthetic strict-v8 fixture: strip only v9 data and undo its costs.
        // Authentic old-executable files are checked separately by runtime QA.
        strip_finances(&mut value);
        value["schema_version"] = json!(8);
        crate::legacy_cost_test_support::project_old_fixture(&mut value);
        value["park"]
            .as_object_mut()
            .unwrap()
            .remove("coaster_cars");
        value["content_version"] = json!(8);
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(directory.join("park.park.json"), &bytes).unwrap();
        fs::write(directory.join("park.park.json.bak"), &bytes).unwrap();
        assert_eq!(
            run(vec![
                OsString::from("--save-check"),
                OsString::from("verify-legacy"),
                OsString::from("--save-dir"),
                directory.clone().into_os_string()
            ]),
            Some(AppExit::Success)
        );
        assert_eq!(fs::read(directory.join("park.park.json")).unwrap(), bytes);
        assert_eq!(
            fs::read(directory.join("park.park.json.bak")).unwrap(),
            bytes
        );
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn legacy_mode_checks_raw_v6_slots_then_current_rules_without_old_future_oracle() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound legacy é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let park = demonstration_park();
        let mut value: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        strip_services(&mut value);
        value["schema_version"] = json!(6);
        crate::legacy_cost_test_support::project_old_fixture(&mut value);
        value["park"]
            .as_object_mut()
            .unwrap()
            .remove("coaster_cars");
        value["content_version"] = json!(6);
        for guest in value["park"]["guests"].as_array_mut().unwrap() {
            for key in [
                "preferences",
                "satisfaction",
                "thought",
                "thought_tick",
                "ratings_given",
                "rating_total",
                "last_rating",
            ] {
                guest.as_object_mut().unwrap().remove(key);
            }
        }
        if let Some(ride) = value["park"]["ride"].as_object_mut() {
            for key in ["value_rejections", "rating_count", "rating_total"] {
                ride.remove(key);
            }
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(directory.join("park.park.json"), &bytes).unwrap();
        fs::write(directory.join("park.park.json.bak"), &bytes).unwrap();
        let args = vec![
            OsString::from("--save-check"),
            OsString::from("verify-legacy"),
            OsString::from("--save-dir"),
            directory.clone().into_os_string(),
        ];
        assert_eq!(run(args), Some(AppExit::Success));
        assert_eq!(fs::read(directory.join("park.park.json")).unwrap(), bytes);
        assert!(directory.join("verification-legacy.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn legacy_v7_preserves_earned_feedback_and_occupied_journey_before_current_needs_rules() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound legacy feedback é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let mut park = demonstration_park();
        super::super::journey::install(&mut park).unwrap();
        super::super::feedback::seek(
            &mut park,
            |p| p.ride().unwrap().rating_count > 0,
            "earned legacy feedback",
        )
        .unwrap();
        super::super::feedback::seek(
            &mut park,
            |p| {
                p.guests()
                    .iter()
                    .any(|g| g.ride_visit.as_ref().is_some_and(|v| v.passenger.is_some()))
            },
            "occupied legacy reviewed park",
        )
        .unwrap();
        // v7 has no services and supports every old thought, but never the new
        // service thoughts produced by needs while constructing this source.
        let mut value: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        strip_services(&mut value);
        value["schema_version"] = json!(7);
        crate::legacy_cost_test_support::project_old_fixture(&mut value);
        value["park"]
            .as_object_mut()
            .unwrap()
            .remove("coaster_cars");
        value["content_version"] = json!(7);
        for guest in value["park"]["guests"].as_array_mut().unwrap() {
            if [
                "Hungry",
                "Thirsty",
                "NeedsToilet",
                "SeekingService",
                "WaitingForService",
                "UsingService",
                "ServiceUnavailable",
                "CannotAffordService",
                "ServiceCancelled",
                "AteFood",
                "HadDrink",
                "UsedToilet",
            ]
            .contains(&guest["thought"].as_str().unwrap())
            {
                guest["thought"] = json!("Exploring");
            }
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(directory.join("park.park.json"), &bytes).unwrap();
        fs::write(directory.join("park.park.json.bak"), &bytes).unwrap();
        let args = vec![
            OsString::from("--save-check"),
            OsString::from("verify-legacy"),
            OsString::from("--save-dir"),
            directory.clone().into_os_string(),
        ];
        assert_eq!(run(args), Some(AppExit::Success));
        assert_eq!(fs::read(directory.join("park.park.json")).unwrap(), bytes);
        assert!(directory.join("verification-legacy.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }
}
