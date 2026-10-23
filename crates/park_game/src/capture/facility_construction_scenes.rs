//! Real paid facility commands and read-only price/refund previews.
use super::*;
use crate::save_check::{facility_construction, ride_choice};
use park_sim::{PathDirection, PathSlope, ServiceId, ServiceKind};

const SITE: TilePos = TilePos { x: 15, y: 21 };

fn prepare_state(park: &mut Park, scene: Scene) -> Option<ServiceId> {
    match scene {
        Scene::FacilityPrice => None,
        Scene::FacilityUnaffordable => {
            facility_construction::spend_down(park).unwrap();
            None
        }
        Scene::FacilityRefund => {
            Some(facility_construction::build(park, SITE, 8, ServiceKind::Drink).unwrap())
        }
        Scene::FacilityRemoved => {
            let id = facility_construction::build(park, SITE, 8, ServiceKind::Drink).unwrap();
            facility_construction::demolish(park, id).unwrap();
            None
        }
        Scene::FacilityFinance => {
            let mut drink = None;
            for (pos, kind) in crate::save_check::services::SITES {
                let id = facility_construction::build(park, pos, 8, kind).unwrap();
                if kind == ServiceKind::Drink {
                    drink = Some(id);
                }
            }
            facility_construction::demolish(park, drink.unwrap()).unwrap();
            Some(facility_construction::build(park, SITE, 8, ServiceKind::Drink).unwrap())
        }
        _ => unreachable!(),
    }
}

fn present(park: &mut Park, view: &mut ViewState, scene: Scene, selected: Option<ServiceId>) {
    let before = park.clone();
    view.help = false;
    view.guests_open = false;
    view.services_open = false;
    view.rides_open = false;
    view.finance_open = matches!(scene, Scene::FacilityFinance);
    view.selected_service = selected;
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.path_notice = None;
    let center = world::path_build_center(SITE, park, view.rotation, 8, PathSlope::Flat).unwrap();
    if matches!(scene, Scene::FacilityPrice | Scene::FacilityUnaffordable) {
        view.path_tool.mode = PathTool::Service(ServiceKind::Drink);
        view.path_tool.height = 8;
        view.path_tool.service_facing = PathDirection::North;
        let point = std::iter::once(center)
            .chain((-24..=24).step_by(4).flat_map(|y| {
                (-28..=28)
                    .step_by(4)
                    .map(move |x| center + Vec2::new(x as f32, y as f32))
            }))
            .find(|p| {
                world::pick_sloped_path_plane(*p, park, view.rotation, 8, PathSlope::Flat)
                    == Some(SITE)
                    && !world::sloped_path_plane_occluded(
                        *p,
                        park,
                        view.rotation,
                        8,
                        PathSlope::Flat,
                        view.view_mode,
                    )
            })
            .expect("facility price preview needs a visible site");
        path_tools::point_at(park, view, point, false);
        assert_eq!(
            view.path_tool.service_preview.unwrap().valid,
            matches!(scene, Scene::FacilityPrice)
        );
        assert!(view.path_tool.hint.contains("$80.00"));
    } else if matches!(scene, Scene::FacilityRefund) {
        let object = park.service(selected.unwrap()).unwrap().object.unwrap();
        let surface = SurfaceId::Object(object);
        let center = world::surface_center(park, surface, view.rotation).unwrap();
        let point = std::iter::once(center)
            .chain((-24..=24).step_by(4).flat_map(|y| {
                (-28..=28)
                    .step_by(4)
                    .map(move |x| center + Vec2::new(x as f32, y as f32))
            }))
            .find(|p| world::pick_surface(*p, park, view.rotation, view.view_mode) == Some(surface))
            .expect("refund preview needs a visible facility");
        view.path_tool.mode = PathTool::Demolish;
        view.hovered = Some(surface);
        path_tools::point_at(park, view, point, false);
        assert!(
            view.path_tool
                .service_preview
                .is_some_and(|p| p.valid && p.demolish)
        );
        assert!(view.path_tool.hint.contains("$40.00"));
    } else if matches!(scene, Scene::FacilityRemoved) {
        view.path_notice = Some(
            "Drinks removed: $40.00 refunded. Public path and financial history retained.".into(),
        );
    }
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = 2.0;
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    assert_eq!(
        *park, before,
        "showing a facility quote must not change simulation or cash"
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
    fn paid_facility_captures_show_real_quotes_without_spending_during_presentation() {
        for scene in [
            Scene::FacilityPrice,
            Scene::FacilityRefund,
            Scene::FacilityUnaffordable,
            Scene::FacilityFinance,
            Scene::FacilityRemoved,
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
                    present(&mut park, &mut view, scene, selected);
                    assert_eq!(bytes, park.to_save_bytes().unwrap());
                }
            }
        }
    }
}
