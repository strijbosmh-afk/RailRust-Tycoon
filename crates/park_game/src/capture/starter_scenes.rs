//! Controlled views of the actual bundled designs; captures never touch user libraries.
use super::*;
use crate::{design_library::DesignLibrary, starter_designs};

pub(super) fn prepare(
    park: &mut Park,
    view: &mut ViewState,
    library: &mut DesignLibrary,
    scene: Scene,
) {
    let index = match scene {
        Scene::StarterWinder => 1,
        Scene::StarterClimb => 2,
        _ => 0,
    };
    *park = Park::new(32, 32);
    *library = DesignLibrary::default();
    library.available = true;
    library.entries = starter_designs::designs().unwrap();
    library.selected = Some(index);
    let design = &library.entries[index];
    library.name = design.name().into();
    (library.anchor, library.height) = design.anchor();
    library.notice = "Starter coaster selected. Load into draft, then Apply to build.".into();
    view.help = false;
    view.rides_open = false;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = true;
    view.path_tool = ToolState::default();
    view.track_editor.active = true;
    view.selected = None;
    view.hovered = None;
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.pan = Vec2::ZERO;
    view.zoom = 1.2;
    if matches!(scene, Scene::StarterPaid) {
        *park = crate::save_check::starters::install(0).unwrap();
        let id = crate::save_check::starters::open_paid(park).unwrap();
        view.designs_open = false;
        view.rides_open = true;
        view.selected_ride = Some(id);
        view.ride_tab = crate::ui::rides::RideTab::Operations;
        view.track_editor.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starter_captures_have_three_real_designs_and_an_actual_paid_coaster() {
        for scene in [
            Scene::StarterPocket,
            Scene::StarterWinder,
            Scene::StarterClimb,
            Scene::StarterPaid,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState::default();
            let mut library = DesignLibrary::default();
            prepare(&mut park, &mut view, &mut library, scene);
            assert_eq!(library.entries.len(), 3);
            if matches!(scene, Scene::StarterPaid) {
                assert!(crate::save_check::ride_choice::paid_running(
                    &park,
                    park.ride().unwrap().id
                ));
            } else {
                let design = &library.entries[library.selected.unwrap()];
                assert!(park.track_construction_quote_for(design.pieces()).is_ok());
            }
            assert_eq!(
                Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                park
            );
        }
    }
}
