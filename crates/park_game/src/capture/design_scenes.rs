//! Design captures use an in-memory catalog and never read or write a player's files.
use super::*;
use crate::{
    design_library::{DesignCommand, DesignLibrary},
    save_check::track_construction,
    track_tools::{self, TrackCommand},
};
use park_sim::TrackDesign;

const NAME: &str = "Woodland Lift and Drop Circuit 1";

fn prepare_state(park: &mut Park, view: &mut ViewState, library: &mut DesignLibrary, scene: Scene) {
    let live = {
        let mut clone = park.clone();
        clone.clear_track_draft();
        clone
    };
    track_tools::apply_command(park, view, TrackCommand::CopyInstalled);
    let design = TrackDesign::from_draft(NAME, &track_construction::specs()).unwrap();
    assert_eq!(NAME.chars().count(), 32);
    *library = DesignLibrary::default();
    library.available = true;
    library.name = "My track".into();
    library.notice = "Save the current draft, or load a saved design to edit.\nLoading does not buy or replace live track.".into();
    if !matches!(scene, Scene::LibraryEmpty) {
        library.entries.push(design.clone());
        library.selected = Some(0);
        library.name = NAME.into();
        (library.anchor, library.height) = design.anchor();
        library.notice = "Saved design ready. Adjust placement or load it into the draft.".into();
    }
    match scene {
        Scene::LibraryEmpty => library.name_focused = true,
        Scene::LibraryTransformed | Scene::LoadedDesign => {
            library.anchor = TilePos { x: 18, y: 12 };
            library.height = 12;
            library.rotation = 1;
            if matches!(scene, Scene::LoadedDesign) {
                let specs = design
                    .transformed(library.anchor, library.height, library.rotation)
                    .unwrap();
                track_tools::load_design_draft(park, view, &specs).unwrap();
                view.track_editor.notice = format!(
                    "{} design pieces loaded.\nUndo restores the previous draft.",
                    specs.len()
                );
            }
        }
        Scene::LibraryConfirmUpdate => {
            library.confirmation = Some(DesignCommand::UpdateSelected);
        }
        Scene::LibraryConfirmDelete => {
            library.confirmation = Some(DesignCommand::DeleteSelected);
        }
        Scene::LibraryInvalid => {
            library.anchor = TilePos { x: 0, y: 0 };
            let error = design
                .transformed(library.anchor, library.height, library.rotation)
                .unwrap_err();
            library.notice =
                format!("Design does not fit: {error}\nAdjust position, height or rotation.");
        }
        Scene::LibrarySaved => {}
        _ => unreachable!(),
    }
    let mut after = park.clone();
    after.clear_track_draft();
    assert_eq!(
        after, live,
        "design capture preparation changed the live park"
    );
}

fn present(park: &Park, view: &mut ViewState, library: &DesignLibrary, scene: Scene) {
    view.help = false;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.rides_open = false;
    view.designs_open = !matches!(scene, Scene::LoadedDesign);
    view.selected_ride = None;
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.path_notice = None;
    view.track_editor.active = true;
    view.track_editor.preview = None;
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = 1.15;
    let anchor = if matches!(scene, Scene::LoadedDesign) {
        library.anchor
    } else {
        TilePos { x: 12, y: 8 }
    };
    view.pan = world::tile_center(anchor, park.width(), view.rotation)
        + Vec2::Y
            * (f32::from(library.height) - f32::from(park.max_surface_height()) * 0.5)
            * world::HEIGHT_STEP;
}

pub(super) fn prepare(
    park: &mut Park,
    view: &mut ViewState,
    library: &mut DesignLibrary,
    scene: Scene,
) {
    prepare_state(park, view, library, scene);
    present(park, view, library, scene);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library_bytes(library: &DesignLibrary) -> Vec<u8> {
        serde_json::to_vec(&(
            library
                .entries
                .iter()
                .map(|d| d.to_json().unwrap())
                .collect::<Vec<_>>(),
            library.selected,
            &library.name,
            library.name_focused,
            library.anchor,
            library.height,
            library.rotation,
            &library.notice,
            format!("{:?}", library.confirmation),
            library.available,
        ))
        .unwrap()
    }

    #[test]
    fn named_design_presentation_preserves_park_history_and_library_in_every_view() {
        for scene in [
            Scene::LibraryEmpty,
            Scene::LibrarySaved,
            Scene::LibraryTransformed,
            Scene::LibraryConfirmUpdate,
            Scene::LibraryConfirmDelete,
            Scene::LibraryInvalid,
            Scene::LoadedDesign,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            let mut library = DesignLibrary::default();
            prepare_state(&mut park, &mut view, &mut library, scene);
            let park_bytes = park.to_save_bytes().unwrap();
            let catalog_bytes = library_bytes(&library);
            let counts = (view.track_editor.undo_len(), view.track_editor.redo_len());
            for rotation in 0..4 {
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    view.rotation = rotation;
                    view.view_mode = mode;
                    present(&park, &mut view, &library, scene);
                    assert_eq!(park_bytes, park.to_save_bytes().unwrap());
                    assert_eq!(catalog_bytes, library_bytes(&library));
                    assert_eq!(
                        counts,
                        (view.track_editor.undo_len(), view.track_editor.redo_len())
                    );
                }
            }
        }
    }
}
