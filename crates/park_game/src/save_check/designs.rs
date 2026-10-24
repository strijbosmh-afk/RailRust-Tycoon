//! Fresh-process named-design proof through the same commands as the modal.
use super::*;
use crate::{
    design_library::{self, DesignCommand, DesignLibrary},
    design_store::DesignStore,
    track_tools::{self, TrackCommand, TrackEditor},
};
use park_sim::{TrackBuildKind, TrackDesign, TrackDirection, TrainPhase};

const ORIGINAL: &str = "Woodland grade é";
const REVISED: &str = "Woodland grade revised é";

fn bytes(park: &Park) -> Result<Vec<u8>, String> {
    park.to_save_bytes().map_err(|error| error.to_string())
}

fn catalog(store: &DesignStore) -> Result<Vec<u8>, String> {
    fs::read(store.catalog_path()).map_err(|error| error.to_string())
}

fn require(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn history(view: &ViewState) -> (usize, usize) {
    (view.track_editor.undo_len(), view.track_editor.redo_len())
}

/// Derived images never become part of the catalog. Reopening the exact saved
/// geometry must recreate the same thumbnails and all four preview rotations.
fn image_proof(directory: &Path, designs: &[TrackDesign], verify: bool) -> Result<(), String> {
    use crate::design_preview::{PREVIEW_HEIGHT, PREVIEW_WIDTH, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH};
    for (index, design) in designs.iter().enumerate() {
        for (kind, rotation, width, height) in
            std::iter::once(("thumbnail", 0, THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT))
                .chain((0..4).map(|rotation| ("preview", rotation, PREVIEW_WIDTH, PREVIEW_HEIGHT)))
        {
            let image = crate::design_thumbnail::render(design, rotation, width, height);
            let pixels = image.data.ok_or("missing derived image pixels")?;
            let path = directory.join(format!("design-{index}-{kind}-{rotation}.rgba"));
            if verify {
                require(
                    fs::read(path).map_err(|e| e.to_string())? == pixels,
                    "reopened design preview differs from saved geometry's image",
                )?;
            } else {
                write_new(&path, &pixels)?;
            }
        }
    }
    Ok(())
}

fn live_bytes(park: &Park) -> Result<Vec<u8>, String> {
    let mut live = park.clone();
    live.clear_track_draft();
    bytes(&live)
}

/// Catalog commands must never alter a single serialized park field.
fn free_command(
    store: &DesignStore,
    park: &mut Park,
    view: &mut ViewState,
    library: &mut DesignLibrary,
    command: DesignCommand,
) -> Result<(), String> {
    let before = bytes(park)?;
    let counts = history(view);
    design_library::perform_command(store, park, view, library, command);
    require(
        bytes(park)? == before && history(view) == counts,
        "catalog management changed park state, money or draft history",
    )
}

fn select(
    store: &DesignStore,
    park: &mut Park,
    view: &mut ViewState,
    library: &mut DesignLibrary,
    name: &str,
) -> Result<(), String> {
    let index = library
        .entries
        .iter()
        .position(|design| design.name() == name)
        .ok_or("expected named design is missing")?;
    free_command(store, park, view, library, DesignCommand::Select(index))
}

fn save_park(session: &SaveSession, park: &mut Park, view: &mut ViewState) -> Result<(), String> {
    if persistence::perform_action(session, park, view, SaveAction::Save) {
        Ok(())
    } else {
        Err(view.save_notice.clone())
    }
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let save = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(save.directory()).map_err(|error| error.to_string())?;
    let store = DesignStore::new(save.directory().to_path_buf());
    let mut park = demonstration_park();
    track_construction::install_access(&mut park)?;
    ride_choice::seek(
        &mut park,
        |park| park.train().is_some_and(|t| t.phase == TrainPhase::Closed),
        "closed design proof train",
    )?;
    track_construction::draft(&mut park, true)?;
    let original =
        TrackDesign::from_draft(ORIGINAL, park.track_draft()).map_err(|e| e.to_string())?;
    let mut view = ViewState {
        track_editor: TrackEditor {
            active: true,
            ..Default::default()
        },
        designs_open: true,
        ..Default::default()
    };
    let mut library = DesignLibrary::default();
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::Open,
    )?;
    require(
        library.available && library.entries.is_empty(),
        "fresh library was not empty",
    )?;
    library.name = ORIGINAL.into();
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::SaveNew,
    )?;
    require(
        store.load(ORIGINAL).map_err(|e| e.to_string())? == original,
        "saved geometry differs from the exact graded draft",
    )?;
    let first_catalog = catalog(&store)?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::SaveNew,
    )?;
    require(
        catalog(&store)? == first_catalog && library.entries.len() == 1,
        "duplicate-name refusal replaced the catalog",
    )?;
    save_park(session, &mut park, &mut view)?;
    write_new(
        &save.directory().join("expected-backup.park.json"),
        &bytes(&park)?,
    )?;
    write_new(
        &save.directory().join("expected-original.track-design.json"),
        original.to_json().map_err(|e| e.to_string())?.as_bytes(),
    )?;

    library.name = "Working copy".into();
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::SaveNew,
    )?;
    // Real picker regrading becomes the replacement geometry of a named design.
    view.designs_open = false;
    track_construction::edit_draft_piece(
        &mut park,
        &mut view,
        TrackBuildKind::Brake,
        TilePos { x: 14, y: 5 },
        11,
        TrackDirection::North,
    )?;
    view.designs_open = true;
    library.name = REVISED.into();
    let revised =
        TrackDesign::from_draft(REVISED, park.track_draft()).map_err(|e| e.to_string())?;
    let pending_catalog = catalog(&store)?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::UpdateSelected,
    )?;
    require(
        library.confirmation == Some(DesignCommand::UpdateSelected)
            && catalog(&store)? == pending_catalog,
        "update wrote before confirmation",
    )?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::Cancel,
    )?;
    require(
        library.confirmation.is_none() && catalog(&store)? == pending_catalog,
        "cancelled update changed the library",
    )?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::UpdateSelected,
    )?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::Confirm,
    )?;
    require(
        store.load(REVISED).map_err(|e| e.to_string())? == revised
            && store.load("Working copy").is_err(),
        "confirmed update did not rename and replace exact geometry",
    )?;
    let revised_catalog = catalog(&store)?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::DeleteSelected,
    )?;
    require(
        library.confirmation == Some(DesignCommand::DeleteSelected),
        "delete did not request confirmation",
    )?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::Cancel,
    )?;
    require(
        catalog(&store)? == revised_catalog,
        "cancelled delete changed saved designs",
    )?;
    library.name = "Temporary layout".into();
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::SaveNew,
    )?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::DeleteSelected,
    )?;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::Confirm,
    )?;
    require(
        store.load("Temporary layout").is_err() && library.entries.len() == 2,
        "confirmed delete did not remove only the selected design",
    )?;
    require(
        store.load(ORIGINAL).map_err(|e| e.to_string())? == original
            && store.load(REVISED).map_err(|e| e.to_string())? == revised,
        "library management changed an unrelated design",
    )?;
    save_park(session, &mut park, &mut view)?;
    write_new(
        &save.directory().join("expected-primary.park.json"),
        &bytes(&park)?,
    )?;
    write_new(
        &save.directory().join("expected-revised.track-design.json"),
        revised.to_json().map_err(|e| e.to_string())?.as_bytes(),
    )?;
    write_new(
        &save.directory().join("expected-library.json"),
        &catalog(&store)?,
    )?;
    image_proof(save.directory(), &[original, revised], false)?;
    write_new(&save.directory().join("writer-designs.txt"), format!(
        "Parkbound {}\nExact free save and duplicate refusal: PASS\nUpdate and rename only after confirmation: PASS\nUpdate/delete cancellation preserves catalog bytes: PASS\nConfirmed delete preserves the other designs: PASS\nCatalog commands preserve all park bytes and draft history: PASS\nDerived thumbnail and four preview rotations recorded without changing catalog format: PASS\n",
        env!("CARGO_PKG_VERSION")
    ).as_bytes())?;
    Ok(())
}

fn expected_design(directory: &Path, name: &str) -> Result<TrackDesign, String> {
    let json = fs::read_to_string(directory.join(name)).map_err(|e| e.to_string())?;
    TrackDesign::from_json(&json).map_err(|e| e.to_string())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let save = session.store.as_ref().ok_or("invalid --save-dir")?;
    let original_store = DesignStore::new(save.directory().to_path_buf());
    let original = expected_design(save.directory(), "expected-original.track-design.json")?;
    let revised = expected_design(save.directory(), "expected-revised.track-design.json")?;
    image_proof(save.directory(), &[original.clone(), revised.clone()], true)?;
    let primary = fs::read(save.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(save.backup_path()).map_err(|e| e.to_string())?;
    let catalog_bytes = catalog(&original_store)?;
    require(
        catalog_bytes
            == fs::read(save.directory().join("expected-library.json"))
                .map_err(|e| e.to_string())?,
        "fresh-process catalog bytes differ from the writer",
    )?;
    require(
        original_store.list().map_err(|e| e.to_string())?.len() == 2
            && original_store.load(ORIGINAL).map_err(|e| e.to_string())? == original
            && original_store.load(REVISED).map_err(|e| e.to_string())? == revised,
        "fresh process failed to preserve exact names and geometry",
    )?;

    // All new Save operations use a new copied profile. Writer slots remain immutable.
    let scratch = save.directory().join("verification-session");
    fs::create_dir(&scratch).map_err(|e| e.to_string())?;
    let roundtrip =
        SaveSession::from_arguments(["--save-dir".into(), scratch.clone().into_os_string()]);
    let roundtrip_save = roundtrip
        .store
        .as_ref()
        .ok_or("invalid verification session")?;
    write_new(&roundtrip_save.save_path(), &primary)?;
    write_new(&roundtrip_save.backup_path(), &backup)?;
    let store = DesignStore::new(scratch);
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    write_new(&store.catalog_path(), &catalog_bytes)?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(&roundtrip, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    require(
        park == read_expected(&save.directory().join("expected-primary.park.json"))?
            && view.speed == Speed::Paused
            && history(&view) == (0, 0),
        "fresh primary did not load exactly, paused, without history",
    )?;
    let loaded = park.clone();
    let original_live = live_bytes(&park)?;
    let original_cash = park.cash_cents();
    let mut library = DesignLibrary::default();
    view.track_editor.active = true;
    view.designs_open = true;
    free_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::Open,
    )?;
    select(&store, &mut park, &mut view, &mut library, ORIGINAL)?;
    let preview_before = bytes(&park)?;
    let preview_history = history(&view);
    let preview_quote = park
        .track_construction_quote_for(original.pieces())
        .map_err(|e| e.to_string())?;
    require(
        crate::design_preview::status_text(&park, &library).contains("Buy "),
        "complete saved design has no preview quote",
    )?;
    require(
        bytes(&park)? == preview_before && history(&view) == preview_history,
        "preview replaced the current draft or live state",
    )?;
    for command in [
        DesignCommand::MoveX(10),
        DesignCommand::MoveY(2),
        DesignCommand::Height(4),
        DesignCommand::Turn,
    ] {
        free_command(&store, &mut park, &mut view, &mut library, command)?;
    }
    let transformed = original
        .transformed(TilePos { x: 18, y: 12 }, 12, 1)
        .map_err(|e| e.to_string())?;
    design_library::perform_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::LoadDraft,
    );
    require(
        park.track_draft() == transformed
            && history(&view) == (1, 0)
            && !view.designs_open
            && view.track_editor.active,
        "transformed design did not load as one editor action",
    )?;
    require(
        live_bytes(&park)? == original_live && park.cash_cents() == original_cash,
        "loading a transformed design changed live park bytes or money",
    )?;
    let transformed_park = park.clone();
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Undo);
    require(
        park == loaded && history(&view) == (0, 1),
        "Undo did not recover the previous exact draft",
    )?;
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Redo);
    require(
        park == transformed_park && history(&view) == (1, 0),
        "Redo did not recover transformed geometry",
    )?;
    view.designs_open = true;
    select(&store, &mut park, &mut view, &mut library, ORIGINAL)?;
    library.anchor = TilePos { x: 0, y: 0 };
    let counts = history(&view);
    design_library::perform_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::LoadDraft,
    );
    require(
        park == transformed_park && history(&view) == counts && view.designs_open,
        "invalid placement changed draft, history or live state",
    )?;

    let entries = library.entries.clone();
    save_park(&roundtrip, &mut park, &mut view)?;
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Undo);
    if !persistence::perform_action(&roundtrip, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    require(
        park == transformed_park
            && view.speed == Speed::Paused
            && history(&view) == (0, 0)
            && library.entries == entries
            && catalog(&store)? == catalog_bytes,
        "save/load did not retain the transformed draft and independent library while clearing history",
    )?;
    if !persistence::perform_action(&roundtrip, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    require(
        park == loaded && history(&view) == (0, 0) && catalog(&store)? == catalog_bytes,
        "backup restoration changed the independent library or retained session history",
    )?;
    view.designs_open = true;
    view.track_editor.active = true;
    select(&store, &mut park, &mut view, &mut library, ORIGINAL)?;
    design_library::perform_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::LoadDraft,
    );
    require(
        park.track_draft() == original.pieces()
            && live_bytes(&park)? == original_live
            && park.cash_cents() == original_cash,
        "original design load was not free and exact",
    )?;
    let quote = park.track_construction_quote().map_err(|e| e.to_string())?;
    require(
        quote == preview_quote,
        "preview quote differed from the unchanged park's loaded design quote",
    )?;
    require(
        quote.construction_cents > 0 && quote.affordable(),
        "design Apply proof requires a paid affordable loop",
    )?;
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Apply);
    require(
        park.cash_cents() == original_cash + quote.refund_cents - quote.construction_cents
            && history(&view) == (0, 0),
        "Apply did not settle the exact quote and clear draft history",
    )?;
    require(
        park.circuit()
            .is_some_and(|circuit| circuit.specs() == track_construction::specs()),
        "Apply did not install the named design loop",
    )?;
    let paid = park.clone();
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Undo);
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Redo);
    require(
        park == paid && catalog(&store)? == catalog_bytes,
        "draft history reversed Apply or installation changed the saved library",
    )?;
    require(
        fs::read(save.save_path()).map_err(|e| e.to_string())? == primary
            && fs::read(save.backup_path()).map_err(|e| e.to_string())? == backup
            && catalog(&original_store)? == catalog_bytes,
        "verification changed the writer's original save slots or catalog",
    )?;
    write_new(&save.directory().join("verification-designs.txt"), format!(
        "Parkbound {}\nFresh-process named library preserves exact ordered grades and Unicode-safe names: PASS\nProduction anchor, height and clockwise rotation controls load one undoable draft action: PASS\nLoad, Undo and Redo preserve live park bytes and money: PASS\nOut-of-bounds placement preserves draft and history: PASS\nSave/Load and Backup retain independent library; draft history resets: PASS\nOriginal valid design installs through normal paid Apply; history cannot reverse the purchase: PASS\nWriter's original primary, backup and catalog bytes remain unchanged: PASS\nFresh-process thumbnails and four rotation previews match exact saved geometry pixels: PASS\nRead-only preview keeps unrelated draft and matches later Apply quote: PASS\n",
        env!("CARGO_PKG_VERSION")
    ).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_designs_reopen_transform_and_settle_only_at_apply() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound named design proof é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-designs", "verify-designs"] {
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
        assert!(directory.join("writer-designs.txt").is_file());
        assert!(directory.join("verification-designs.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }
}
