//! Service images seek natural needs and real use through production commands.
use super::*;
use crate::save_check::services;
use park_sim::{GuestId, GuestThought, PathDirection, PathSlope, ServiceId, ServiceVisitPhase};

fn install(park: &mut Park, view: &mut ViewState) {
    let original = (view.rotation, view.view_mode);
    view.rotation = 0;
    view.view_mode = ViewMode::Underground;
    for (pos, kind) in services::SITES {
        view.path_tool = ToolState {
            mode: PathTool::Service(kind),
            height: 8,
            service_facing: PathDirection::North,
            ..default()
        };
        view.selected = None;
        let center = world::path_build_center(pos, park, 0, 8, PathSlope::Flat).unwrap();
        let point = std::iter::once(center)
            .chain((-24..=24).step_by(4).flat_map(|y| {
                (-28..=28)
                    .step_by(4)
                    .map(move |x| center + Vec2::new(x as f32, y as f32))
            }))
            .find(|point| {
                world::pick_sloped_path_plane(*point, park, 0, 8, PathSlope::Flat) == Some(pos)
                    && !world::sloped_path_plane_occluded(
                        *point,
                        park,
                        0,
                        8,
                        PathSlope::Flat,
                        ViewMode::Underground,
                    )
            })
            .expect("service construction footprint visible");
        view.hovered = world::pick_surface(point, park, 0, ViewMode::Underground);
        path_tools::point_at(park, view, point, true);
        let Some(SurfaceId::Object(id)) = view.selected else {
            panic!(
                "service tool did not construct {kind:?}: {:?}",
                view.path_notice
            );
        };
        let service = park.service_by_object(id).expect("new facility metadata");
        assert_eq!(service.kind, kind);
        assert_eq!(service.facing, PathDirection::North);
        assert_eq!(park.object(id).unwrap().pos, pos);
        assert!(park.service_access(service.id).public_path.is_some());
    }
    (view.rotation, view.view_mode) = original;
}

fn completed_thought(thought: GuestThought) -> bool {
    matches!(
        thought,
        GuestThought::AteFood | GuestThought::HadDrink | GuestThought::UsedToilet
    )
}

fn prepare_state(
    park: &mut Park,
    view: &mut ViewState,
    scene: Scene,
) -> (GuestId, Option<ServiceId>) {
    assert!(scene.is_service());
    park.set_train_open(false);
    install(park, view);
    if matches!(scene, Scene::ServiceNeeds) {
        services::seek(
            park,
            |p| {
                p.guests()
                    .iter()
                    .any(|g| g.needs.hunger >= 60 || g.needs.thirst >= 60 || g.needs.toilet >= 60)
            },
            "naturally rising needs",
        )
        .unwrap();
        let id = park
            .guests()
            .iter()
            .find(|g| g.needs.hunger >= 60 || g.needs.thirst >= 60 || g.needs.toilet >= 60)
            .unwrap()
            .id;
        return (id, None);
    }
    if matches!(scene, Scene::ServiceServed) {
        services::seek(
            park,
            |p| {
                p.guests().iter().any(|g| {
                    g.services_used > 0 && g.service_visit.is_none() && completed_thought(g.thought)
                })
            },
            "natural served guest",
        )
        .unwrap();
        let id = park
            .guests()
            .iter()
            .find(|g| {
                g.services_used > 0 && g.service_visit.is_none() && completed_thought(g.thought)
            })
            .unwrap()
            .id;
        return (id, None);
    }
    let phase = if matches!(scene, Scene::ServiceApproaching) {
        ServiceVisitPhase::Approaching
    } else {
        ServiceVisitPhase::Using
    };
    services::seek(
        park,
        |p| services::guest_in_phase(p, phase).is_some(),
        "actual service phase",
    )
    .unwrap();
    let id = services::guest_in_phase(park, phase).unwrap();
    let guest = services::guest(park, id).clone();
    let service = guest.service_visit.as_ref().unwrap().service;
    if matches!(scene, Scene::ServiceBlocked | Scene::ServiceRecovered) {
        let kind = park.service(service).unwrap().kind;
        let removed = services::cancel_access(park, id).unwrap();
        if matches!(scene, Scene::ServiceRecovered) {
            services::repair_access(park, removed).unwrap();
            let thought = match kind {
                park_sim::ServiceKind::Food => GuestThought::AteFood,
                park_sim::ServiceKind::Drink => GuestThought::HadDrink,
                park_sim::ServiceKind::Toilet => GuestThought::UsedToilet,
            };
            services::seek(
                park,
                |p| {
                    let current = services::guest(p, id);
                    current.services_used > guest.services_used
                        && current.service_visit.is_none()
                        && current.thought == thought
                },
                "the cancelled guest reaches the repaired service and is served",
            )
            .unwrap();
        }
    }
    services::bytes(park).unwrap();
    (id, Some(service))
}

fn present(
    park: &Park,
    view: &mut ViewState,
    scene: Scene,
    selected: (GuestId, Option<ServiceId>),
) {
    view.path_tool = ToolState::default();
    view.selected = None;
    view.hovered = None;
    view.path_notice = None;
    view.speed = Speed::Paused;
    view.help = false;
    view.guests_open = !matches!(scene, Scene::ServiceNeeds | Scene::ServiceUsing);
    view.services_open = matches!(scene, Scene::ServiceUsing);
    view.selected_service = selected.1;
    view.selected_guest = Some(selected.0);
    let mut ids: Vec<_> = park.guests().iter().map(|g| g.id).collect();
    ids.sort();
    view.guest_page = ids.iter().position(|id| *id == selected.0).unwrap() / 8;
    let center = world::path_build_center(
        TilePos { x: 14, y: 20 },
        park,
        view.rotation,
        8,
        PathSlope::Flat,
    )
    .unwrap();
    view.zoom = 2.0;
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let selected = prepare_state(park, view, scene);
    present(park, view, scene, selected);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn service_scenes_reach_natural_states_and_all_views_preserve_authoritative_data() {
        for terrain in [TerrainProfile::Flat, TerrainProfile::Terraced] {
            for scene in [
                Scene::ServiceNeeds,
                Scene::ServiceApproaching,
                Scene::ServiceUsing,
                Scene::ServiceServed,
                Scene::ServiceBlocked,
                Scene::ServiceRecovered,
            ] {
                let mut park = crate::demonstration_park();
                park.try_apply_terrain_profile(terrain).unwrap();
                let mut view = ViewState {
                    capture_active: true,
                    ..default()
                };
                let selected = prepare_state(&mut park, &mut view, scene);
                let saved = services::bytes(&park).unwrap();
                for rotation in 0..4 {
                    for mode in [ViewMode::Surface, ViewMode::Underground] {
                        view.rotation = rotation;
                        view.view_mode = mode;
                        present(&park, &mut view, scene, selected);
                        assert_eq!(saved, park.to_save_bytes().unwrap());
                        assert_eq!(view.selected_guest, Some(selected.0));
                        assert!(!(view.guests_open && view.services_open));
                        assert_eq!(view.speed, Speed::Paused);
                        assert!(view.pan.is_finite() && view.save_pending.is_none());
                    }
                }
            }
        }
    }
}
