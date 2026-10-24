//! Real mid-wait and paid-departure loading-policy dashboard states.
use super::*;
use crate::{
    ride_operations::{self, OperatingField, OperationCommand},
    save_check::{dispatch, ride_choice},
    ui::rides::RideTab,
};

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let waiting = matches!(scene, Scene::DispatchWaiting);
    let owners = if waiting {
        let owners = dispatch::install_empty(park).unwrap();
        dispatch::open_empty_wait(park, owners).unwrap();
        owners
    } else {
        let owners = dispatch::install_paid(park).unwrap();
        dispatch::open_paid(park, owners).unwrap();
        owners
    };
    view.ride_operating_notice = None;
    if !waiting {
        let before = park.clone();
        let bytes = park.to_save_bytes().unwrap();
        assert!(!ride_operations::perform_command(
            park,
            view,
            owners[0],
            OperationCommand::Adjust(OperatingField::MinimumPassengers, 1),
        ));
        assert_eq!(*park, before);
        assert_eq!(park.to_save_bytes().unwrap(), bytes);
    }
    ride_choice::bytes(park).unwrap();
    view.help = false;
    view.rides_open = true;
    view.ride_tab = RideTab::Operations;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = false;
    view.selected_ride = Some(owners[0]);
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
    fn dispatch_scenes_show_positive_empty_waits_or_real_paid_running_rides() {
        for scene in [Scene::DispatchWaiting, Scene::DispatchPaid] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            prepare(&mut park, &mut view, scene);
            assert!(view.rides_open);
            assert_eq!(view.ride_tab, RideTab::Operations);
            assert_eq!(view.speed, Speed::Paused);
            let owners = [park.ride().unwrap().id, park.carousels()[0].ride.id];
            for id in owners {
                let progress = park.ride_dispatch_progress(id).unwrap();
                if matches!(scene, Scene::DispatchWaiting) {
                    assert!(progress.waiting_for_load);
                    // Carousel's legacy departure boundary is one tick before
                    // the train's, so their positive counters need not match.
                    assert!(progress.extra_waited_ticks >= 17);
                    assert!(progress.extra_waited_ticks < progress.maximum_extra_wait_ticks);
                    assert_eq!(progress.seated_passengers, 0);
                    assert!(!park.ride_is_running(id));
                } else {
                    assert_eq!(progress.minimum_passengers, 1);
                    assert_eq!(progress.extra_waited_ticks, 0);
                    assert!(ride_choice::paid_running(&park, id));
                    assert!(
                        view.ride_operating_notice
                            .as_ref()
                            .unwrap()
                            .contains("unchanged")
                    );
                }
            }
            assert_eq!(
                Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                park
            );
        }
    }
}
