//! Fresh-process isolated mechanics test proof using production controls.
use super::*;
use crate::coaster_testing::{self, CoasterTesting};
use park_sim::{CoasterTestReport, RideOperatingSettings, TrackKind};

fn require(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

/// Exercise the same explicit Run boundary as the dashboard and prove that
/// repeat runs and cache refreshes leave every authoritative field untouched.
fn inspect(park: &Park, state: &mut CoasterTesting) -> Result<CoasterTestReport, String> {
    let before = park.clone();
    let before_bytes = super::ride_choice::bytes(park)?;
    let selected = Some(park.ride().ok_or("missing test coaster")?.id);
    state.run_pending = true;
    if !coaster_testing::run_selected(park, state, selected) {
        return Err(state.notice.clone().unwrap_or("coaster test failed".into()));
    }
    let report = state.report.clone().ok_or("completed test has no report")?;
    require(
        !state.run_pending
            && report.input.matches(park)
            && report.total_ticks == report.moving_ticks + u32::from(report.station_wait_ticks)
            && report.moving_ticks > 0
            && report.min_speed_units_per_tick > 0
            && report.min_speed_units_per_tick <= report.max_speed_units_per_tick,
        "test report has inconsistent motion metrics",
    )?;
    state.refresh(park, selected);
    require(
        state.report.as_ref() == Some(&report)
            && coaster_testing::run_selected(park, state, selected)
            && state.report.as_ref() == Some(&report),
        "identical inputs did not reproduce the exact test report",
    )?;
    require(
        *park == before && super::ride_choice::bytes(park)? == before_bytes,
        "isolated test changed the source park or save bytes",
    )?;
    Ok(report)
}

/// Intentional edits happen only in disposable copies. They establish the
/// difference between an unapplied draft and a changed installed test input.
fn invalidation_reports(source: &Park) -> Result<String, String> {
    let before = source.clone();
    let before_bytes = super::ride_choice::bytes(source)?;
    let id = source.ride().ok_or("missing invalidation coaster")?.id;
    let mut state = CoasterTesting::default();
    let original = inspect(source, &mut state)?;
    let mut settings_copy = source.clone();
    let mut settings = settings_copy
        .ride_operating_settings(id)
        .ok_or("missing test settings")?;
    settings.station_wait_ticks += 30;
    settings_copy
        .set_ride_operating_settings(id, settings)
        .map_err(|e| e.to_string())?;
    state.refresh(&settings_copy, Some(id));
    require(
        state.report.is_none() && !original.input.matches(&settings_copy),
        "changed station wait retained a stale test report",
    )?;
    let waited = inspect(&settings_copy, &mut state)?;
    require(
        waited.total_ticks == original.total_ticks + 30
            && waited.moving_ticks == original.moving_ticks
            && waited.ratings == original.ratings,
        "station wait changed isolated motion or layout estimates",
    )?;

    let mut geometry_copy = source.clone();
    state.report = Some(original.clone());
    geometry_copy
        .copy_installed_track_to_draft()
        .map_err(|e| e.to_string())?;
    let mut piece = *geometry_copy
        .track_draft()
        .iter()
        .find(|piece| piece.kind == TrackKind::Straight)
        .ok_or("test fixture needs a straight for a brake replacement")?;
    piece.kind = TrackKind::Brake;
    geometry_copy
        .set_track_draft_piece(piece)
        .map_err(|e| e.to_string())?;
    state.refresh(&geometry_copy, Some(id));
    require(
        state.report.as_ref() == Some(&original),
        "an unapplied draft invalidated installed-track telemetry",
    )?;
    geometry_copy
        .apply_track_draft()
        .map_err(|e| e.to_string())?;
    state.refresh(&geometry_copy, Some(id));
    require(
        state.report.is_none() && !original.input.matches(&geometry_copy),
        "changed installed geometry retained a stale test report",
    )?;
    let changed = inspect(&geometry_copy, &mut state)?;
    require(
        changed.layout.brakes == original.layout.brakes + 1,
        "fresh report did not reflect the installed brake replacement",
    )?;
    geometry_copy.set_train_open(true);
    let open_bytes = super::ride_choice::bytes(&geometry_copy)?;
    require(
        !coaster_testing::run_selected(&geometry_copy, &mut state, Some(id))
            && state.report.is_none()
            && super::ride_choice::bytes(&geometry_copy)? == open_bytes,
        "open-ride test did not refuse without changing the park",
    )?;
    require(
        *source == before && super::ride_choice::bytes(source)? == before_bytes,
        "invalidation proof changed its original park",
    )?;
    Ok(format!(
        "Changed wait:\n{waited:#?}\nChanged installed geometry:\n{changed:#?}\n"
    ))
}

fn save(session: &SaveSession, park: &mut Park, view: &mut ViewState) -> Result<(), String> {
    if persistence::perform_action(session, park, view, SaveAction::Save) {
        Ok(())
    } else {
        Err(view.save_notice.clone())
    }
}

fn load(
    session: &SaveSession,
    park: &mut Park,
    view: &mut ViewState,
    action: SaveAction,
) -> Result<(), String> {
    // Seed a real report even when loading the exact same inputs. A load must
    // clear presentation state independently of the comparable input key.
    inspect(park, &mut view.coaster_testing)?;
    view.coaster_testing.run_pending = true;
    view.coaster_testing.show_pending = true;
    view.rides_open = true;
    view.ride_tab = crate::ui::rides::RideTab::Test;
    if !persistence::perform_action(session, park, view, action) {
        return Err(view.save_notice.clone());
    }
    require(
        view.speed == Speed::Paused
            && !view.rides_open
            && view.coaster_testing.report.is_none()
            && !view.coaster_testing.run_pending
            && !view.coaster_testing.show_pending
            && view.coaster_testing.notice.is_none(),
        "successful load retained test results or pending commands",
    )
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    park.set_train_open(false);
    let id = park.ride().ok_or("missing coaster")?.id;
    let mut view = ViewState::default();
    let baseline = inspect(&park, &mut view.coaster_testing)?;
    let baseline_changes = invalidation_reports(&park)?;
    save(session, &mut park, &mut view)?;
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    write_new(
        &store.directory().join("expected-baseline.txt"),
        format!("{baseline:#?}\n{baseline_changes}").as_bytes(),
    )?;

    park.set_ride_operating_settings(
        id,
        RideOperatingSettings {
            station_wait_ticks: 150,
            lift_speed: 16,
            brake_speed: 12,
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())?;
    view.coaster_testing.refresh(&park, Some(id));
    require(
        view.coaster_testing.report.is_none(),
        "tuning retained the baseline report",
    )?;
    let tuned = inspect(&park, &mut view.coaster_testing)?;
    let tuned_changes = invalidation_reports(&park)?;
    require(
        tuned.input != baseline.input
            && tuned.station_wait_ticks == 150
            && tuned.moving_ticks > baseline.moving_ticks,
        "tuned proof must produce distinct measured motion",
    )?;
    save(session, &mut park, &mut view)?;
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    write_new(
        &store.directory().join("expected-tuned.txt"),
        format!("{tuned:#?}\n{tuned_changes}").as_bytes(),
    )?;
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    park.set_train_open(false);
    let mut view = ViewState::default();
    for (action, expected_park, expected_report) in [
        (
            SaveAction::Load,
            "expected-primary.park.json",
            "expected-tuned.txt",
        ),
        (
            SaveAction::Backup,
            "expected-backup.park.json",
            "expected-baseline.txt",
        ),
    ] {
        load(session, &mut park, &mut view, action)?;
        require(
            park == read_expected(&store.directory().join(expected_park))?,
            "loaded test fixture differs from its independent saved state",
        )?;
        let report = inspect(&park, &mut view.coaster_testing)?;
        let changes = invalidation_reports(&park)?;
        require(
            format!("{report:#?}\n{changes}").as_bytes()
                == fs::read(store.directory().join(expected_report)).map_err(|e| e.to_string())?,
            "fresh-process test or invalidation report differs from the writer",
        )?;
        // Repeat with identical geometry/settings: cache key equality must not
        // stop production Load/Backup from resetting all transient test state.
        load(session, &mut park, &mut view, action)?;
    }
    require(
        fs::read(store.save_path()).map_err(|e| e.to_string())? == primary
            && fs::read(store.backup_path()).map_err(|e| e.to_string())? == backup,
        "coaster testing proof changed source slots",
    )?;
    write_new(
        &store.directory().join("verification-coaster-testing.txt"),
        format!(
            "Parkbound {}\nIndependent closed baseline/tuned primary and backup: PASS\nExact fresh-process telemetry and layout-estimate reproduction: PASS\nRepeated tests preserve the entire source park and save bytes: PASS\nUnapplied draft retains result; applied geometry/settings invalidate it: PASS\nOpen-ride testing refuses without mutation: PASS\nLoad and Backup clear reports, pending tests and notices even for identical inputs: PASS\nOriginal primary and backup unchanged: PASS\nMechanics-only lap excludes passenger loading policy; no forces or demand simulation claimed.\nNative pointer gestures: NOT CLAIMED\n",
            env!("CARGO_PKG_VERSION")
        ).as_bytes(),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing_proof_recreates_reports_and_clears_loaded_cache_without_writing_slots() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound coaster testing proof é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-coaster-testing", "verify-coaster-testing"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    directory.clone().into_os_string(),
                ]),
                Some(AppExit::Success)
            );
        }
        let receipt =
            fs::read_to_string(directory.join("verification-coaster-testing.txt")).unwrap();
        assert!(receipt.contains("Original primary and backup unchanged: PASS"));
        assert!(receipt.contains("even for identical inputs: PASS"));
        fs::remove_dir_all(directory).unwrap();
    }
}
