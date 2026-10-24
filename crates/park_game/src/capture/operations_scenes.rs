//! Real operating commands on closed rides, plus naturally occupied refusal.
use super::*;
use crate::{
    ride_operations::{self, OperatingField, OperationCommand},
    save_check::{operations, ride_choice},
};

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let (coaster, carousel) = operations::install(park).unwrap();
    let selected = if matches!(scene, Scene::OperationsCarousel) {
        carousel
    } else {
        coaster
    };
    if matches!(scene, Scene::OperationsLimits) {
        for field in [
            OperatingField::StationWait,
            OperatingField::LiftSpeed,
            OperatingField::BrakeSpeed,
        ] {
            assert!(ride_operations::perform_command(
                park,
                view,
                coaster,
                OperationCommand::Adjust(field, 127)
            ));
        }
    }
    if matches!(scene, Scene::OperationsRunning) {
        park.set_ride_open(coaster, true).unwrap();
        park.set_ride_open(carousel, true).unwrap();
        ride_choice::seek(
            park,
            |p| ride_choice::paid_running(p, coaster),
            "tuned coaster has paid passengers",
        )
        .unwrap();
        assert!(!ride_operations::perform_command(
            park,
            view,
            coaster,
            OperationCommand::Adjust(OperatingField::StationWait, 1)
        ));
    }
    ride_choice::bytes(park).unwrap();
    view.help = false;
    view.rides_open = true;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = false;
    view.selected_ride = Some(selected);
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = 1.2;
    view.pan = Vec2::ZERO;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_scenes_have_real_settings_and_running_refusal() {
        for scene in [
            Scene::OperationsCoaster,
            Scene::OperationsCarousel,
            Scene::OperationsLimits,
            Scene::OperationsRunning,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            prepare(&mut park, &mut view, scene);
            assert!(view.rides_open);
            assert_eq!(view.speed, Speed::Paused);
            assert_eq!(
                park.ride_operating_settings_status(view.selected_ride.unwrap())
                    .is_err(),
                matches!(scene, Scene::OperationsRunning)
            );
            assert_eq!(
                Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                park
            );
        }
    }
}
