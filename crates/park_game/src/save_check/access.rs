//! Access assistance is reproducible after restart and never edits the park.
use super::*;
use crate::access_planning::{self, AccessCommand};
use crate::path_tools::PathTool;
use park_sim::{
    RideAccessBuildKind, RideAccessStatus, RideAccessSuggestion, RideExitStatus, RideId,
};

fn inspect(park: &mut Park) -> Result<String, String> {
    let before = park.clone();
    let ids: Vec<_> = park.all_rides().map(|ride| ride.id).collect();
    let mut report = String::new();
    for id in ids {
        let plan = park.ride_access_plan(id).map_err(|e| e.to_string())?;
        report.push_str(&format!("{plan:#?}\n"));
        for gate in &plan.entrance_candidates {
            park.validate_ride_entrance(gate.pos, gate.height, gate.facing, id)
                .map_err(|e| e.to_string())?;
        }
        for gate in &plan.exit_candidates {
            park.validate_ride_exit(gate.pos, gate.height, gate.facing, id)
                .map_err(|e| e.to_string())?;
        }
        for suggestion in &plan.suggestions {
            match suggestion.kind {
                RideAccessBuildKind::Entrance => park
                    .validate_ride_entrance(
                        suggestion.pos,
                        suggestion.height,
                        suggestion.facing,
                        id,
                    )
                    .map_err(|e| e.to_string())?,
                RideAccessBuildKind::Exit => park
                    .validate_ride_exit(suggestion.pos, suggestion.height, suggestion.facing, id)
                    .map_err(|e| e.to_string())?,
                RideAccessBuildKind::Queue => park
                    .validate_queue_path(suggestion.pos, suggestion.height, suggestion.slope, id)
                    .map_err(|e| e.to_string())?,
                RideAccessBuildKind::PublicPath => park
                    .validate_sloped_path(suggestion.pos, suggestion.height, suggestion.slope)
                    .map_err(|e| e.to_string())?,
            };
        }
        // The controller deliberately selects the first ranked tile of a kind.
        // Validate every alternative above, but exercise each visible action once.
        for (kind, mode) in [
            (RideAccessBuildKind::Entrance, PathTool::Entrance),
            (RideAccessBuildKind::Exit, PathTool::Exit),
            (RideAccessBuildKind::Queue, PathTool::BuildQueue),
            (RideAccessBuildKind::PublicPath, PathTool::Build),
        ] {
            let Some(chosen) = plan.suggestions.iter().find(|value| value.kind == kind) else {
                continue;
            };
            let mut view = ViewState {
                rides_open: true,
                selected_ride: Some(id),
                ..ViewState::default()
            };
            if !access_planning::perform_command(park, &mut view, id, AccessCommand::ShowAccess)
                || !access_planning::perform_command(
                    park,
                    &mut view,
                    id,
                    AccessCommand::Prepare(kind),
                )
            {
                return Err("valid access suggestion could not prepare its build tool".into());
            }
            let preview = view
                .path_tool
                .preview
                .as_ref()
                .ok_or("prepared suggestion has no preview")?;
            let gate = matches!(
                kind,
                RideAccessBuildKind::Entrance | RideAccessBuildKind::Exit
            );
            let owner = (kind != RideAccessBuildKind::PublicPath).then_some(id);
            if view.path_tool.mode != mode
                || view.selected_ride != Some(id)
                || view.path_tool.height != chosen.height
                || view.path_tool.slope != chosen.slope
                || view.path_tool.entrance_facing != chosen.facing
                || view.path_tool.exit_facing != chosen.facing
                || preview.pos != chosen.pos
                || preview.height != chosen.height
                || preview.slope != chosen.slope
                || preview.queue_owner != owner
                || preview.entrance_facing != gate.then_some(chosen.facing)
                || preview.exit_gate != (kind == RideAccessBuildKind::Exit)
                || preview.demolish
                || !preview.valid
            {
                return Err("prepared tool differs from the valid access suggestion".into());
            }
        }
    }
    if *park != before
        || park.to_save_bytes().map_err(|e| e.to_string())?
            != before.to_save_bytes().map_err(|e| e.to_string())?
    {
        return Err("access reports or prepared tools changed the live park".into());
    }
    Ok(report)
}

fn first_suggestion(
    park: &Park,
    id: RideId,
    kind: RideAccessBuildKind,
) -> Result<RideAccessSuggestion, String> {
    park.ride_access_plan(id)
        .map_err(|e| e.to_string())?
        .suggestions
        .into_iter()
        .find(|suggestion| suggestion.kind == kind)
        .ok_or_else(|| format!("intermediate access fixture must exercise {kind:?} suggestions"))
}

/// Build only in a disposable copy of the missing-access backup. The source
/// slots remain the original missing and fully connected fixtures.
fn intermediate_reports(source: &Park) -> Result<String, String> {
    let before = source.clone();
    let before_bytes = super::ride_choice::bytes(source)?;
    let mut park = source.clone();
    let id = park.ride().ok_or("missing intermediate coaster")?.id;
    let entrance = first_suggestion(&park, id, RideAccessBuildKind::Entrance)?;
    park.place_ride_entrance(entrance.pos, entrance.height, entrance.facing, id)
        .map_err(|e| e.to_string())?;
    let exit = first_suggestion(&park, id, RideAccessBuildKind::Exit)?;
    park.place_ride_exit(exit.pos, exit.height, exit.facing, id)
        .map_err(|e| e.to_string())?;
    if park.ride_access(id).status != RideAccessStatus::NoQueue {
        return Err("intermediate gates must expose a missing queue".into());
    }
    let queue = first_suggestion(&park, id, RideAccessBuildKind::Queue)?;
    let mut report = format!("Gates placed; queue required:\n{}", inspect(&mut park)?);
    park.place_queue_path(queue.pos, queue.height, queue.slope, id)
        .map_err(|e| e.to_string())?;
    first_suggestion(&park, id, RideAccessBuildKind::PublicPath)?;
    if park.ride_access(id).status != RideAccessStatus::NoPublicEntry
        && park.ride_exit_access(id).status != RideExitStatus::NoPublicExit
    {
        return Err("intermediate queue must expose a missing public path".into());
    }
    report.push_str(&format!(
        "Queue placed; public path required:\n{}",
        inspect(&mut park)?
    ));
    if *source != before || super::ride_choice::bytes(source)? != before_bytes {
        return Err("intermediate access proof changed its source park".into());
    }
    Ok(report)
}

fn replay(park: &mut Park) -> Result<(), String> {
    let ids: Vec<_> = park.all_rides().map(|ride| ride.id).collect();
    for id in &ids {
        let plan = park.ride_access_plan(*id).map_err(|e| e.to_string())?;
        if plan.entrance.status != RideAccessStatus::Ready
            || plan.exit.status != RideExitStatus::Ready
        {
            return Err("connected access fixture is not ready".into());
        }
        park.set_ride_open(*id, true).map_err(|e| e.to_string())?;
    }
    for tick in 0..3_600 {
        super::ride_choice::advance_checked(park)?;
        if tick % 300 == 0 {
            inspect(park)?;
            *park = Park::from_save_bytes(&super::ride_choice::bytes(park)?)
                .map_err(|e| e.to_string())?;
        }
    }
    if ids
        .iter()
        .any(|id| park.ride_by_id(*id).is_none_or(|ride| ride.completed == 0))
    {
        return Err("ready rides did not complete passenger journeys".into());
    }
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let coaster = park.ride().ok_or("missing coaster")?.id;
    if park.ride_access(coaster).status != RideAccessStatus::NoEntrance {
        return Err("initial fixture should need an entrance".into());
    }
    let missing = inspect(&mut park)?;
    let intermediate = intermediate_reports(&park)?;
    if park.ride_access_plan(RideId(u64::MAX)).is_ok() {
        return Err("unknown ride unexpectedly produced access assistance".into());
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-missing.txt"),
        missing.as_bytes(),
    )?;
    write_new(
        &store.directory().join("expected-intermediate.txt"),
        intermediate.as_bytes(),
    )?;
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &super::ride_choice::bytes(&park)?,
    )?;
    super::operations::install(&mut park)?;
    let ready = inspect(&mut park)?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-ready.txt"),
        ready.as_bytes(),
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
    let original_primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let original_backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if view.speed != Speed::Paused
        || inspect(&mut park)?.as_bytes()
            != fs::read(store.directory().join("expected-ready.txt")).map_err(|e| e.to_string())?
    {
        return Err("ready access report differs after process restart".into());
    }
    replay(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("access-assisted continuation differs after restart".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || inspect(&mut park)?.as_bytes()
            != fs::read(store.directory().join("expected-missing.txt"))
                .map_err(|e| e.to_string())?
    {
        return Err("missing-access backup or suggestions differ after restart".into());
    }
    if intermediate_reports(&park)?.as_bytes()
        != fs::read(store.directory().join("expected-intermediate.txt"))
            .map_err(|e| e.to_string())?
    {
        return Err("intermediate queue/public-path assistance differs after restart".into());
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != original_primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != original_backup
    {
        return Err("access verification changed source saves".into());
    }
    write_new(&store.directory().join("verification-access.txt"), format!(
        "Parkbound {}\nMissing and ready access reports survive fresh-process loads: PASS\nIntermediate missing-queue/public-path reports replay from a disposable backup copy: PASS\nAll gate candidates and suggestions pass existing placement validation: PASS\nEach available Prepare action selects its first ranked tile with matching mode, height, slope, owner and gate facing: PASS\nPrepared tools and intermediate planning preserve the source park and save bytes: PASS\nConnected rides complete paid journeys over 3600 ticks with strict roundtrips: PASS\nOriginal primary and backup unchanged: PASS\nNative pointer gestures: NOT CLAIMED\n", env!("CARGO_PKG_VERSION")
    ).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intermediate_proof_requires_both_path_tools_and_preserves_source() {
        let park = demonstration_park();
        let before = park.clone();
        let bytes = super::super::ride_choice::bytes(&park).unwrap();
        let report = intermediate_reports(&park).unwrap();
        assert!(report.contains("Gates placed; queue required:"));
        assert!(report.contains("Queue placed; public path required:"));
        assert!(report.contains("kind: Queue"));
        assert!(report.contains("kind: PublicPath"));
        assert_eq!(park, before);
        assert_eq!(super::super::ride_choice::bytes(&park).unwrap(), bytes);
        let mut ready = park;
        super::super::operations::install(&mut ready).unwrap();
        assert!(intermediate_reports(&ready).is_err());
    }

    #[test]
    fn access_proof_preserves_planning_and_real_connected_journeys() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound access proof é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-access", "verify-access"] {
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
        let receipt = fs::read_to_string(directory.join("verification-access.txt")).unwrap();
        assert!(receipt.contains("Intermediate missing-queue/public-path reports"));
        assert!(receipt.contains("Each available Prepare action"));
        fs::remove_dir_all(directory).unwrap();
    }
}
