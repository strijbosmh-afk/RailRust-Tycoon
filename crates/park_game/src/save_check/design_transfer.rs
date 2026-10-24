//! Export in one process and import the same individual file in a fresh process.
//! File choices are injected into the production completion boundary; native
//! picker gestures and overwrite-prompt acceptance remain separate UI checks.
use super::*;
use crate::{
    design_library::{self, DesignCommand, DesignLibrary},
    design_store::DesignStore,
    design_transfer, design_transfer_file, track_tools,
};
use park_sim::{TrackBuildKind, TrackDesign, TrackDirection};

const NAME: &str = "Shared woodland grade é";

fn require(value: bool, message: &str) -> Result<(), String> {
    if value { Ok(()) } else { Err(message.into()) }
}

fn history(view: &ViewState) -> (usize, usize) {
    (view.track_editor.undo_len(), view.track_editor.redo_len())
}

fn unrelated_draft(park: &mut Park, view: &mut ViewState) -> Result<(), String> {
    track_tools::load_design_draft(
        park,
        view,
        &[
            TrackBuildKind::Station.spec(TilePos { x: 15, y: 15 }, 8, TrackDirection::North),
            TrackBuildKind::Lift.spec(TilePos { x: 15, y: 14 }, 8, TrackDirection::North),
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn transfer(
    store: &DesignStore,
    park: &mut Park,
    view: &mut ViewState,
    library: &mut DesignLibrary,
    command: DesignCommand,
    choice: Result<Option<std::path::PathBuf>, String>,
) -> Result<bool, String> {
    let before = park.to_save_bytes().map_err(|e| e.to_string())?;
    let counts = history(view);
    design_library::perform_command(store, park, view, library, command);
    let request = library
        .transfer_request
        .take()
        .ok_or("transfer request not queued")?;
    let success = design_transfer::complete_transfer(store, library, request, choice);
    require(
        before == park.to_save_bytes().map_err(|e| e.to_string())? && counts == history(view),
        "file transfer changed park, draft, money or undo/redo history",
    )?;
    Ok(success)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let save = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(save.directory()).map_err(|e| e.to_string())?;
    let store = DesignStore::new(save.directory().to_path_buf());
    let mut park = demonstration_park();
    let mut view = ViewState {
        speed: Speed::Paused,
        designs_open: true,
        ..Default::default()
    };
    view.track_editor.active = true;
    let original =
        TrackDesign::from_draft(NAME, &track_construction::specs()).map_err(|e| e.to_string())?;
    track_tools::load_design_draft(&mut park, &mut view, original.pieces())
        .map_err(|e| e.to_string())?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    store.save_new(&original).map_err(|e| e.to_string())?;
    unrelated_draft(&mut park, &mut view)?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    view.designs_open = true;
    let mut library = DesignLibrary::default();
    design_library::perform_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::Open,
    );
    library.name = "Unsaved editor name".into();
    library.rotation = 1;
    library.anchor = TilePos { x: 18, y: 12 };
    library.height = 12;
    let originals = [save.save_path(), save.backup_path(), store.catalog_path()]
        .into_iter()
        .map(|path| {
            fs::read(&path)
                .map(|bytes| (path, bytes))
                .map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let exported = save.directory().join("shared circuit é.parktrack.json");
    require(
        transfer(
            &store,
            &mut park,
            &mut view,
            &mut library,
            DesignCommand::Export,
            Ok(Some(exported.clone())),
        )?,
        "export failed",
    )?;
    require(
        design_transfer_file::read_design(&exported).map_err(|e| e.to_string())? == original,
        "export used an unsaved draft, edited name or placement transform",
    )?;
    // Model a separately approved overwrite; the picker itself is not exercised.
    fs::write(&exported, b"old export bytes").map_err(|e| e.to_string())?;
    require(
        transfer(
            &store,
            &mut park,
            &mut view,
            &mut library,
            DesignCommand::Export,
            Ok(Some(exported.clone())),
        )?,
        "approved overwrite failed",
    )?;
    require(
        !transfer(
            &store,
            &mut park,
            &mut view,
            &mut library,
            DesignCommand::Export,
            Ok(Some(save.save_path())),
        )?,
        "export overwrote a park save",
    )?;
    for command in [DesignCommand::Import, DesignCommand::Export] {
        require(
            !transfer(
                &store,
                &mut park,
                &mut view,
                &mut library,
                command,
                Ok(None),
            )?,
            "cancelled choice succeeded",
        )?;
    }
    for (index, (path, bytes)) in originals.iter().enumerate() {
        require(
            fs::read(path).map_err(|e| e.to_string())? == *bytes,
            "export/cancel changed a park slot or catalog",
        )?;
        write_new(
            &save
                .directory()
                .join(format!("expected-source-{index}.bin")),
            bytes,
        )?;
    }
    write_new(
        &save.directory().join("expected-export.json"),
        original.to_json().map_err(|e| e.to_string())?.as_bytes(),
    )?;
    write_new(&save.directory().join("writer-design-transfer.txt"), format!(
        "Parkbound {}\nSelected persisted geometry exported, not current draft/name/placement: PASS\nApproved destination replacement roundtrips exact grades and Unicode name: PASS\nPark-save destination refusal and both cancelled dialogs preserve source bytes: PASS\nAll transfer commands preserve live park, draft, money and history: PASS\nNative file picker gestures: NOT CLAIMED\n", env!("CARGO_PKG_VERSION")
    ).as_bytes())?;
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let save = session.store.as_ref().ok_or("invalid --save-dir")?;
    let exported = save.directory().join("shared circuit é.parktrack.json");
    let expected_bytes =
        fs::read(save.directory().join("expected-export.json")).map_err(|e| e.to_string())?;
    require(
        fs::read(&exported).map_err(|e| e.to_string())? == expected_bytes,
        "export bytes differ across processes",
    )?;
    let expected = design_transfer_file::read_design(&exported).map_err(|e| e.to_string())?;
    let destination = save.directory().join("recipient library é");
    fs::create_dir(&destination).map_err(|e| e.to_string())?;
    let store = DesignStore::new(destination.clone());
    let mut park = demonstration_park();
    let mut view = ViewState {
        speed: Speed::Paused,
        designs_open: true,
        ..Default::default()
    };
    view.track_editor.active = true;
    unrelated_draft(&mut park, &mut view)?;
    let mut library = DesignLibrary::default();
    design_library::perform_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::Open,
    );
    require(
        transfer(
            &store,
            &mut park,
            &mut view,
            &mut library,
            DesignCommand::Import,
            Ok(Some(exported.clone())),
        )?,
        "fresh-process import failed",
    )?;
    require(
        library.selected_design() == Some(&expected),
        "import lost exact geometry or selection",
    )?;
    require(
        transfer(
            &store,
            &mut park,
            &mut view,
            &mut library,
            DesignCommand::Import,
            Ok(Some(exported.clone())),
        )?,
        "duplicate import failed",
    )?;
    let duplicate = library
        .selected_design()
        .ok_or("duplicate import not selected")?
        .clone();
    require(
        duplicate.name() == format!("{NAME} (2)") && duplicate.pieces() == expected.pieces(),
        "duplicate import did not keep both exact designs",
    )?;
    require(
        DesignStore::new(destination.clone())
            .load(NAME)
            .map_err(|e| e.to_string())?
            == expected,
        "reopened recipient library lost original design",
    )?;
    require(
        DesignStore::new(destination.clone())
            .load(duplicate.name())
            .map_err(|e| e.to_string())?
            == duplicate,
        "reopened recipient library lost numbered duplicate",
    )?;
    let catalog_before = fs::read(store.catalog_path()).map_err(|e| e.to_string())?;
    let bad = destination.join("invalid.parktrack.json");
    for malformed in [
        b"{}".to_vec(),
        expected
            .to_json()
            .map_err(|e| e.to_string())?
            .replace("\"version\": 1", "\"version\": 999")
            .into_bytes(),
        vec![b' '; park_sim::MAX_TRACK_DESIGN_BYTES + 1],
    ] {
        fs::write(&bad, &malformed).map_err(|e| e.to_string())?;
        require(
            !transfer(
                &store,
                &mut park,
                &mut view,
                &mut library,
                DesignCommand::Import,
                Ok(Some(bad.clone())),
            )?,
            "invalid import succeeded",
        )?;
        require(
            fs::read(&bad).map_err(|e| e.to_string())? == malformed,
            "invalid import rewrote source file",
        )?;
    }
    require(
        !transfer(
            &store,
            &mut park,
            &mut view,
            &mut library,
            DesignCommand::Import,
            Err("injected file-picker failure".into()),
        )?,
        "failed dialog imported",
    )?;
    require(
        fs::read(store.catalog_path()).map_err(|e| e.to_string())? == catalog_before,
        "refused import changed catalog bytes",
    )?;
    let before_load = park.clone();
    let counts = history(&view);
    design_library::perform_command(
        &store,
        &mut park,
        &mut view,
        &mut library,
        DesignCommand::LoadDraft,
    );
    require(
        park.track_draft() == expected.pieces() && view.track_editor.undo_len() == counts.0 + 1,
        "imported design did not load as one draft edit",
    )?;
    track_tools::apply_command(&mut park, &mut view, track_tools::TrackCommand::Undo);
    require(
        park == before_load,
        "Undo of imported design changed live park",
    )?;
    for (index, path) in [
        save.save_path(),
        save.backup_path(),
        DesignStore::new(save.directory().to_path_buf()).catalog_path(),
    ]
    .iter()
    .enumerate()
    {
        require(
            fs::read(path).map_err(|e| e.to_string())?
                == fs::read(
                    save.directory()
                        .join(format!("expected-source-{index}.bin")),
                )
                .map_err(|e| e.to_string())?,
            "writer source bytes changed",
        )?;
    }
    require(
        fs::read(&exported).map_err(|e| e.to_string())? == expected_bytes,
        "import rewrote exported design",
    )?;
    write_new(&save.directory().join("verification-design-transfer.txt"), format!(
        "Parkbound {}\nFresh-process exported file preserves exact version-1 name, piece order and grades: PASS\nImports add and select original plus numbered duplicate without overwriting: PASS\nReopened recipient catalog retains both designs: PASS\nMalformed, unsupported, oversized and cancelled/failed choices preserve files and park: PASS\nImported design loads as one undoable draft edit; Undo restores original full park: PASS\nWriter primary, backup, catalog and export bytes remain unchanged: PASS\nNative file picker gestures: NOT CLAIMED\n", env!("CARGO_PKG_VERSION")
    ).as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_design_reopens_in_an_independent_library_without_altering_the_park() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound design transfer proof é {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-design-transfer", "verify-design-transfer"] {
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
        assert!(directory.join("writer-design-transfer.txt").is_file());
        assert!(directory.join("verification-design-transfer.txt").is_file());
        fs::remove_dir_all(directory).unwrap();
    }
}
