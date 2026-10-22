//! Migration preserves old state at load; future decisions use the current rules.
use super::*;
use park_sim::{GuestPreferences, GuestThought, MAX_SAVE_BYTES};
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
    let seed = old["park"]["crowd_seed"].as_u64().unwrap_or(0);
    for guest in park.guests() {
        if guest.preferences != GuestPreferences::for_guest(seed, guest.id)
            || guest.satisfaction != 70
            || guest.thought != GuestThought::Exploring
            || guest.thought_tick != 0
            || guest.ratings_given != 0
            || guest.rating_total != 0
            || guest.last_rating.is_some()
        {
            return Err(
                "legacy feedback defaults changed history or drew inconsistent traits".into(),
            );
        }
    }
    if let Some(ride) = park.ride() {
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
            if park.ride().is_some() {
                super::feedback::advance_checked(park)?;
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
        if !(1..=6).contains(&schema)
            || old["content_version"].as_u64() != Some(schema)
            || old["format"] != "parkbound-save"
        {
            return Err(
                "verify-legacy requires an authentic strict version1..6 source slot".into(),
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
        if modern["schema_version"] != 7 || modern["content_version"] != 7 {
            return Err("migration did not produce strict7 state".into());
        }
        require_old_fields(&old["park"], &modern["park"], "/park")?;
        require_defaults(&park, &old)?;
        if Park::from_save_bytes(&bytes).map_err(|error| error.to_string())? != park {
            return Err("repeated legacy loading changed deterministic defaults".into());
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
    write_new(&store.directory().join("verification-legacy.txt"), format!("Parkbound {}\nOriginal primary/backup schemas: {versions:?}\nEvery original park field preserved at load (including RNG, IDs, money and active journeys): PASS\nNew traits/default feedback without retroactive ratings: PASS\nProduction loads paused/Inspect with guest panel reset: PASS\nDeterministic current-version continuation: PASS ({CONTINUATION_TICKS} ticks per slot, commands and repeated v7 roundtrips)\nOriginal primary and backup bytes unchanged: PASS\nLegacy future equivalence: NOT CLAIMED; v7 guest decisions intentionally use new rules\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
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
        value["schema_version"] = json!(6);
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
}
