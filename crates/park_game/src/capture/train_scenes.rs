//! Actual car settings, valid occupied seats and readable minimum-load refusal.
use super::*;
use crate::ride_operations::{self, OperatingField as Field, OperationCommand as Command};

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let cars = match scene {
        Scene::CarsThree => 3,
        Scene::CarsTwo | Scene::CarsMinimum => 2,
        _ => 1,
    };
    let id = crate::save_check::trains::install(park, cars).unwrap();
    if matches!(scene, Scene::CarsMinimum) {
        assert!(ride_operations::perform_command(
            park,
            view,
            id,
            Command::Adjust(Field::MinimumPassengers, 4)
        ));
        let before = park.clone();
        assert!(!ride_operations::perform_command(
            park,
            view,
            id,
            Command::Adjust(Field::Cars, -1)
        ));
        assert_eq!(*park, before);
    }
    if matches!(scene, Scene::CarsRiding) {
        park.set_ride_open(id, true).unwrap();
        crate::save_check::ride_choice::seek(
            park,
            |p| crate::save_check::ride_choice::paid_running(p, id),
            "short train riding",
        )
        .unwrap();
    }
    view.help = false;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = false;
    view.rides_open = true;
    view.ride_tab = crate::ui::rides::RideTab::Operations;
    view.selected_ride = Some(id);
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = 1.2;
    view.pan = Vec2::ZERO;
    if matches!(scene, Scene::CarsMap) {
        view.rides_open = false;

        crate::focus_train(park, view);
        view.zoom = 3.5;
    }
    assert_eq!(park.train_cars().len(), usize::from(cars));
    assert_eq!(
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
        *park
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn car_captures_use_real_configuration_and_paid_occupancy() {
        for scene in [
            Scene::CarsOne,
            Scene::CarsTwo,
            Scene::CarsThree,
            Scene::CarsMinimum,
            Scene::CarsRiding,
            Scene::CarsMap,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState::default();
            prepare(&mut park, &mut view, scene);
            if matches!(scene, Scene::CarsRiding) {
                assert!(park.ride_journey_stats(park.ride().unwrap().id).riding > 0);
            }
            if matches!(scene, Scene::CarsMinimum) {
                assert!(
                    view.ride_operating_notice
                        .unwrap()
                        .contains("Lower minimum load")
                );
            }
        }
    }
}
