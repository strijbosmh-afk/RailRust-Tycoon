//! Production quotes and real cash commands, with no synthetic balance edits.
use super::*;
use crate::save_check::{construction, ride_choice};
use park_sim::{PathSlope, RideId};

fn prepare_state(park: &mut Park, scene: Scene) -> Option<RideId> {
    match scene {
        Scene::ConstructionPrice => None,
        Scene::ConstructionUnaffordable => {
            construction::exhaust_build_funds(park).unwrap();
            None
        }
        Scene::ConstructionRefund => Some(construction::build(park).unwrap()),
        Scene::ConstructionRemoved | Scene::ConstructionFinance => {
            let id = construction::build(park).unwrap();
            construction::demolish(park, id).unwrap();
            if matches!(scene, Scene::ConstructionFinance) {
                Some(construction::build(park).unwrap())
            } else {
                None
            }
        }
        _ => unreachable!(),
    }
}

fn present(park: &mut Park, view: &mut ViewState, scene: Scene, selected: Option<RideId>) {
    let before = park.clone();
    view.help = false;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = matches!(scene, Scene::ConstructionFinance);
    view.rides_open = false;
    view.selected_ride = selected;
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.path_notice = None;
    let center =
        world::path_build_center(ride_choice::ORIGIN, park, view.rotation, 8, PathSlope::Flat)
            .unwrap();
    match scene {
        Scene::ConstructionPrice | Scene::ConstructionUnaffordable => {
            view.path_tool.mode = PathTool::Carousel;
            view.path_tool.height = 8;
            let point = [
                Vec2::ZERO,
                Vec2::new(-12.0, 0.0),
                Vec2::new(12.0, 0.0),
                Vec2::new(0.0, 7.0),
                Vec2::new(0.0, -7.0),
            ]
            .into_iter()
            .map(|offset| center + offset)
            .find(|point| {
                world::pick_sloped_path_plane(*point, park, view.rotation, 8, PathSlope::Flat)
                    == Some(ride_choice::ORIGIN)
                    && !world::sloped_path_plane_occluded(
                        *point,
                        park,
                        view.rotation,
                        8,
                        PathSlope::Flat,
                        view.view_mode,
                    )
            })
            .expect("construction preview needs a visible footprint origin");
            path_tools::point_at(park, view, point, false);
            assert_eq!(
                view.path_tool.carousel_preview.unwrap().valid,
                matches!(scene, Scene::ConstructionPrice)
            );
            assert!(view.path_tool.hint.contains("$300.00"));
        }
        Scene::ConstructionRefund => {
            let body = park.carousel(selected.unwrap()).unwrap();
            let (id, point) = body
                .objects
                .iter()
                .find_map(|id| {
                    let surface = SurfaceId::Object(*id);
                    let point = world::surface_center(park, surface, view.rotation)?;
                    (world::pick_surface(point, park, view.rotation, view.view_mode)
                        == Some(surface))
                    .then_some((*id, point))
                })
                .expect("refund preview needs a visible carousel cell");
            view.path_tool.mode = PathTool::Demolish;
            view.hovered = Some(SurfaceId::Object(id));
            path_tools::point_at(park, view, point, false);
            assert!(
                view.path_tool
                    .carousel_preview
                    .is_some_and(|p| p.valid && p.demolish)
            );
            assert!(view.path_tool.hint.contains("$150.00"));
        }
        Scene::ConstructionRemoved => {
            view.path_notice = Some(
                "Carousel removed: $150.00 refunded. Paid cost and refund remain in Finances [N]."
                    .into(),
            );
        }
        Scene::ConstructionFinance => {}
        _ => unreachable!(),
    }
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = 2.0;
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    assert_eq!(
        *park, before,
        "rendering a quote must never spend or refund money"
    );
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
    fn construction_scenes_quote_real_commands_without_mutating_money_during_presentation() {
        for scene in [
            Scene::ConstructionPrice,
            Scene::ConstructionRefund,
            Scene::ConstructionUnaffordable,
            Scene::ConstructionFinance,
            Scene::ConstructionRemoved,
        ] {
            let mut park = crate::demonstration_park();
            let selected = prepare_state(&mut park, scene);
            let expected = ride_choice::bytes(&park).unwrap();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            for rotation in 0..4 {
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    view.rotation = rotation;
                    view.view_mode = mode;
                    present(&mut park, &mut view, scene, selected);
                    assert_eq!(expected, park.to_save_bytes().unwrap());
                }
            }
        }
    }
}
