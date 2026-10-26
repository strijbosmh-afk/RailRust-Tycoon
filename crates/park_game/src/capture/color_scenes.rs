//! Real two-ride painting and classic reset; no forged geometry or money.
use super::*;
use crate::{
    save_check::{colors, value},
    ui::rides::RideTab,
};
pub(super) fn prepare(p: &mut Park, view: &mut ViewState, scene: Scene) {
    *p = if matches!(scene, Scene::ColorsFaulted) {
        let mut p = value::faulted().unwrap();
        let id = p.ride().unwrap().id;
        p.set_ride_appearance(park_sim::RideAppearance {
            ride: id,
            structure: park_sim::RidePaint::Violet,
            supports: park_sim::RidePaint::Sky,
            vehicles: park_sim::RidePaint::Lagoon,
        })
        .unwrap();
        p
    } else {
        colors::install().unwrap()
    };
    let coaster = p.ride().unwrap().id;
    let carousel = p
        .carousels()
        .iter()
        .find(|c| !c.objects.is_empty())
        .map(|c| c.ride.id);
    let id = if matches!(scene, Scene::ColorsCarousel | Scene::ColorsCarouselWorld) {
        carousel.unwrap()
    } else {
        coaster
    };
    if matches!(scene, Scene::ColorsReset) {
        for id in crate::ui::rides::active_ids(p) {
            p.set_ride_appearance(park_sim::RideAppearance {
                ride: id,
                ..Default::default()
            })
            .unwrap();
        }
    }
    view.help = false;
    view.guests_open = false;
    view.finance_open = false;
    view.services_open = false;
    view.designs_open = false;
    view.rides_open = matches!(
        scene,
        Scene::ColorsCoaster | Scene::ColorsCarousel | Scene::ColorsReset
    );
    view.selected_ride = Some(id);
    view.ride_tab = RideTab::Colors;
    view.paint_notice = None;
    view.paint_commands.clear();
    view.speed = Speed::Paused;
    view.path_tool = ToolState::default();
    view.scene_dirty = true;
    if matches!(scene, Scene::ColorsRotated) {
        view.rotation = 2;
    }
    if matches!(scene, Scene::ColorsCars | Scene::ColorsCarouselWorld) {
        crate::focus_train(p, view);
        view.zoom = 3.2;
    }
    if matches!(scene, Scene::ColorsUnderground) {
        view.view_mode = ViewMode::Underground;
    }
    colors::check(p).unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_capture_states_are_strict_and_preserve_actual_paid_visits() {
        for scene in [
            Scene::ColorsCoaster,
            Scene::ColorsCarousel,
            Scene::ColorsWorld,
            Scene::ColorsCars,
            Scene::ColorsCarouselWorld,
            Scene::ColorsRotated,
            Scene::ColorsReset,
            Scene::ColorsFaulted,
            Scene::ColorsUnderground,
        ] {
            let mut p = crate::demonstration_park();
            let mut v = ViewState::default();
            prepare(&mut p, &mut v, scene);
            assert_eq!(v.speed, Speed::Paused);
            if !matches!(scene, Scene::ColorsReset | Scene::ColorsFaulted) {
                assert!(p.guests().iter().any(|g| {
                    g.ride_visit
                        .as_ref()
                        .and_then(|v| v.passenger.as_ref())
                        .is_some_and(|p| p.fare_paid)
                }));
            }
        }
    }
}
