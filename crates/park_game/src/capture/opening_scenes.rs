//! Real checked opening/refusal states at the minimum supported window.
use super::*;
use crate::{
    ride_operations::{self, OperationCommand},
    save_check::{operations, ride_choice},
    ui::rides::RideTab,
};

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    park.set_train_open(false);
    let mut id = park.ride().unwrap().id;
    if !matches!(scene, Scene::OpeningEntrance) {
        let (coaster, carousel) = operations::install(park).unwrap();
        id = if matches!(scene, Scene::OpeningCarousel) {
            carousel
        } else {
            coaster
        };
    }
    if matches!(scene, Scene::OpeningExitPath) {
        let path = park.ride_exit_access(id).public_exit.unwrap();
        park.remove_object(path);
    }
    let succeeds = matches!(scene, Scene::OpeningPaid);
    if matches!(
        scene,
        Scene::OpeningEntrance | Scene::OpeningExitPath | Scene::OpeningPaid
    ) {
        let before = park.clone();
        assert_eq!(
            ride_operations::perform_command(park, view, id, OperationCommand::ToggleOpen),
            succeeds
        );
        if !succeeds {
            assert_eq!(*park, before);
        }
    }
    if succeeds {
        ride_choice::seek(
            park,
            |p| ride_choice::paid_running(p, id),
            "checked opening carries paid passengers",
        )
        .unwrap();
    }
    view.help = false;
    view.rides_open = true;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = false;
    view.selected_ride = Some(id);
    view.ride_tab = RideTab::Operations;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    ride_choice::bytes(park).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opening_scenes_use_actual_checked_refusals_readiness_and_paid_passengers() {
        for scene in [
            Scene::OpeningEntrance,
            Scene::OpeningExitPath,
            Scene::OpeningReady,
            Scene::OpeningCarousel,
            Scene::OpeningPaid,
        ] {
            let mut park = Park::new(32, 32);
            park.apply_terrain_profile(TerrainProfile::Terraced);
            park.install_crowd_fixture(42).unwrap();
            park.install_train_fixture().unwrap();
            let mut view = ViewState::default();
            prepare(&mut park, &mut view, scene);
            let id = view.selected_ride.unwrap();
            assert_eq!(
                park.ride_opening_status(id).is_ok(),
                !matches!(scene, Scene::OpeningEntrance | Scene::OpeningExitPath)
            );
            assert_eq!(park.ride_is_open(id), matches!(scene, Scene::OpeningPaid));
        }
    }
}
