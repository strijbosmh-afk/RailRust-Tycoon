//! Coaster test dashboard scenes built through the production test boundary.
use super::*;
use crate::{
    coaster_testing::{self, CoasterTesting},
    save_check::{operations, ride_choice},
    ui::rides::RideTab,
};
use park_sim::{RideId, RideOperatingSettings};

fn run_without_mutation(park: &Park, view: &mut ViewState, id: RideId, succeeds: bool) {
    let before = park.clone();
    let bytes = park.to_save_bytes().unwrap();
    assert_eq!(
        coaster_testing::run_selected(park, &mut view.coaster_testing, Some(id)),
        succeeds
    );
    assert_eq!(*park, before);
    assert_eq!(park.to_save_bytes().unwrap(), bytes);
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    view.coaster_testing = CoasterTesting::default();
    park.set_train_open(false);
    let mut id = park.ride().unwrap().id;
    if matches!(scene, Scene::TestingReady) {
        coaster_testing::can_run(park, Some(id))
            .expect("ready capture requires a closed, stopped, empty coaster");
    }
    if matches!(scene, Scene::TestingOperating | Scene::TestingCarousel) {
        let (coaster, carousel) = operations::install(park).unwrap();
        id = if matches!(scene, Scene::TestingCarousel) {
            carousel
        } else {
            park.set_ride_open(coaster, true).unwrap();
            ride_choice::seek(
                park,
                |park| ride_choice::paid_running(park, coaster),
                "test refusal coaster has real paid passengers",
            )
            .unwrap();
            coaster
        };
        run_without_mutation(park, view, id, false);
    } else if matches!(scene, Scene::TestingComplete | Scene::TestingChanged) {
        run_without_mutation(park, view, id, true);
        if matches!(scene, Scene::TestingChanged) {
            let old = view.coaster_testing.report.clone().unwrap();
            park.set_ride_operating_settings(
                id,
                RideOperatingSettings {
                    station_wait_ticks: old.input.settings.station_wait_ticks + 30,
                    ..old.input.settings
                },
            )
            .unwrap();
            let before = park.clone();
            view.coaster_testing.refresh(park, Some(id));
            assert!(view.coaster_testing.report.is_none());
            assert!(!old.input.matches(park));
            assert_eq!(*park, before);
        }
    }
    ride_choice::bytes(park).unwrap();
    view.help = false;
    view.rides_open = true;
    view.ride_tab = RideTab::Test;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = false;
    view.selected_ride = Some(id);
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
    use park_sim::RideKind;

    #[test]
    fn testing_scenes_expose_real_results_staleness_and_operating_refusals() {
        for scene in [
            Scene::TestingReady,
            Scene::TestingComplete,
            Scene::TestingChanged,
            Scene::TestingOperating,
            Scene::TestingCarousel,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            prepare(&mut park, &mut view, scene);
            assert!(view.rides_open);
            assert_eq!(view.ride_tab, RideTab::Test);
            assert_eq!(view.speed, Speed::Paused);
            assert!(!view.coaster_testing.run_pending && !view.coaster_testing.show_pending);
            let id = view.selected_ride.unwrap();
            assert_eq!(
                view.coaster_testing.report.is_some(),
                matches!(scene, Scene::TestingComplete)
            );
            assert_eq!(
                coaster_testing::can_run(&park, Some(id)).is_ok(),
                !matches!(scene, Scene::TestingOperating | Scene::TestingCarousel)
            );
            match scene {
                Scene::TestingComplete => {
                    let report = view.coaster_testing.report.as_ref().unwrap();
                    assert!(report.input.matches(&park));
                    assert!(report.moving_ticks > 0 && report.layout.drops > 0);
                    assert_eq!(park.ticks(), 0);
                }
                Scene::TestingChanged => {
                    assert!(
                        view.coaster_testing
                            .notice
                            .as_ref()
                            .unwrap()
                            .contains("fresh test")
                    );
                    assert_eq!(
                        park.ride_operating_settings(id).unwrap().station_wait_ticks,
                        120
                    );
                }
                Scene::TestingOperating => {
                    assert!(ride_choice::paid_running(&park, id));
                    assert!(view.coaster_testing.notice.is_some());
                }
                Scene::TestingCarousel => {
                    assert_eq!(park.ride_kind(id), Some(RideKind::Carousel));
                    assert!(
                        view.coaster_testing
                            .notice
                            .as_ref()
                            .unwrap()
                            .contains("carousel")
                    );
                }
                Scene::TestingReady => assert!(view.coaster_testing.notice.is_none()),
                _ => unreachable!(),
            }
            assert_eq!(
                Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                park
            );
        }
    }
}
