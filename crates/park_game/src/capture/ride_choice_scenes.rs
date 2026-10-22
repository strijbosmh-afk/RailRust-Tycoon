//! Production placement and naturally occupied independent ride scenes.
use super::*;
use crate::save_check::ride_choice;
use park_sim::{GuestId, RideId, RideVisitPhase};

fn prepare_state(park: &mut Park, scene: Scene) -> (Option<RideId>, Option<GuestId>) {
    if matches!(scene, Scene::CarouselValid | Scene::CarouselBlocked) {
        if matches!(scene, Scene::CarouselBlocked) {
            park.place_path(TilePos { x: 21, y: 24 }, 8).unwrap();
        }
        return (None, None);
    }
    if matches!(scene, Scene::CarouselBuilt) {
        return (
            Some(park.place_carousel(ride_choice::ORIGIN, 8).unwrap()),
            None,
        );
    }
    let (coaster, carousel) = ride_choice::install(park).unwrap();
    let mut selected_guest = None;
    if matches!(scene, Scene::DualBoarding) {
        ride_choice::seek(
            park,
            |p| {
                p.guests().iter().any(|g| {
                    g.ride_visit.as_ref().is_some_and(|v| {
                        v.ride == carousel
                            && v.phase == RideVisitPhase::Boarding
                            && v.passenger
                                .as_ref()
                                .and_then(|s| s.transfer.as_ref())
                                .is_some_and(|t| t.progress > 15)
                    })
                })
            },
            "visible carousel transfer",
        )
        .unwrap();
    } else {
        ride_choice::seek(
            park,
            |p| ride_choice::paid_running(p, coaster) && ride_choice::paid_running(p, carousel),
            "both paid rides in motion",
        )
        .unwrap();
        if matches!(scene, Scene::RideChoice) {
            park.set_ride_ticket_price(coaster, 10_000).unwrap();
            // An already approaching guest does not prove a choice made after
            // the price change. Record the exact newly approaching guest.
            selected_guest = Some(ride_choice::seek_new_choice(park, carousel).unwrap());
        } else if matches!(scene, Scene::RideClosed | Scene::CarouselRebuilt) {
            park.set_ride_open(carousel, false).unwrap();
            ride_choice::seek(
                park,
                |p| {
                    !p.guests()
                        .iter()
                        .any(|g| g.ride_visit.as_ref().is_some_and(|v| v.ride == carousel))
                },
                "closed carousel empties while coaster operates",
            )
            .unwrap();
            if matches!(scene, Scene::CarouselRebuilt) {
                park.remove_carousel(carousel).unwrap();
                let old_paths: Vec<_> = park
                    .objects()
                    .iter()
                    .filter(|o| o.queue_owner == Some(carousel))
                    .map(|o| o.id)
                    .collect();
                for id in old_paths {
                    park.remove_object(id).unwrap();
                }
                let rebuilt = ride_choice::install_carousel(park).unwrap();
                assert!(rebuilt.0 > carousel.0);
                ride_choice::seek(
                    park,
                    |p| ride_choice::paid_running(p, rebuilt),
                    "rebuilt carousel earns a new receipt",
                )
                .unwrap();
                return (Some(rebuilt), None);
            }
        }
    }
    (Some(carousel), selected_guest)
}

fn present(
    park: &Park,
    view: &mut ViewState,
    scene: Scene,
    (selected, selected_guest): (Option<RideId>, Option<GuestId>),
) {
    view.help = false;
    view.guests_open = matches!(scene, Scene::RideChoice);
    view.services_open = false;
    view.finance_open = matches!(scene, Scene::RideFinance);
    view.rides_open = matches!(scene, Scene::Rides);
    view.selected_ride = selected;
    view.selected_guest = if view.guests_open {
        let guest = park
            .guests()
            .iter()
            .find(|g| Some(g.id) == selected_guest)
            .expect("choice capture retains the guest who chose after the price command");
        assert!(guest.ride_visit.as_ref().is_some_and(|v| {
            Some(v.ride) == selected && v.phase == RideVisitPhase::Approaching
        }));
        Some(guest.id)
    } else {
        None
    };
    if let Some(id) = view.selected_guest {
        view.guest_page = park.guests().iter().position(|g| g.id == id).unwrap() / 8;
    }
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.path_notice = Some(match scene {
        Scene::CarouselBuilt => {
            "Carousel placed closed. Connect its entrance, queue and exit.".into()
        }
        Scene::DualBoarding => "Guests board their chosen ride through its own gate.".into(),
        Scene::DualRiding => "Two independent rides, each carrying its own paid passengers.".into(),
        Scene::RideClosed => "Carousel closed and emptied; the coaster keeps operating.".into(),
        Scene::CarouselRebuilt => {
            "Rebuilt carousel has a fresh identity; old revenue remains in history.".into()
        }
        _ => "Each ride owns its passengers, income and reviews.".into(),
    });
    if matches!(scene, Scene::CarouselValid | Scene::CarouselBlocked) {
        let valid = park.validate_carousel(ride_choice::ORIGIN, 8).is_ok();
        assert_eq!(valid, matches!(scene, Scene::CarouselValid));
        view.path_tool.mode = PathTool::Carousel;
        view.path_tool.height = 8;
        view.path_tool.carousel_preview = Some(path_tools::CarouselPreview {
            origin: ride_choice::ORIGIN,
            height: 8,
            valid,
            demolish: false,
        });
        view.path_notice = Some(
            if valid {
                "Click to build a closed 3×3 carousel."
            } else {
                "Blocked footprint: clear all nine tiles first."
            }
            .into(),
        );
    }
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = if matches!(scene, Scene::DualRiding) {
        1.2
    } else {
        2.0
    };
    let center = world::path_build_center(
        if matches!(scene, Scene::DualRiding) {
            TilePos { x: 16, y: 16 }
        } else {
            TilePos { x: 19, y: 22 }
        },
        park,
        view.rotation,
        8,
        park_sim::PathSlope::Flat,
    )
    .unwrap();
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let selected = prepare_state(park, scene);
    ride_choice::bytes(park).unwrap();
    present(park, view, scene, selected);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn second_ride_scenes_use_real_construction_passengers_money_and_read_only_views() {
        for scene in [
            Scene::CarouselValid,
            Scene::CarouselBlocked,
            Scene::CarouselBuilt,
            Scene::DualBoarding,
            Scene::DualRiding,
            Scene::RideChoice,
            Scene::Rides,
            Scene::RideClosed,
            Scene::RideFinance,
            Scene::CarouselRebuilt,
        ] {
            let mut park = crate::demonstration_park();
            let selected = prepare_state(&mut park, scene);
            let bytes = ride_choice::bytes(&park).unwrap();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            for rotation in 0..4 {
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    view.rotation = rotation;
                    view.view_mode = mode;
                    present(&park, &mut view, scene, selected);
                    assert_eq!(bytes, park.to_save_bytes().unwrap());
                    assert_eq!(view.speed, Speed::Paused);
                    assert!(view.pan.is_finite());
                }
            }
        }
    }
}
