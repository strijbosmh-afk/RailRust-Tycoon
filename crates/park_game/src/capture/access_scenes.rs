//! Deterministic read-only access checklists from real ride topology.
use super::*;

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let id = if matches!(scene, Scene::AccessReady) {
        crate::save_check::ride_choice::install(park).unwrap().0
    } else {
        park.set_train_open(false);
        let id = park.ride().unwrap().id;
        if matches!(scene, Scene::AccessUnavailable) {
            let rail = park.circuit().unwrap().pieces[3].object;
            park.remove_object(rail).unwrap();
        }
        id
    };
    view.help = false;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = false;
    view.rides_open = true;
    view.ride_access_open = true;
    view.selected_ride = Some(id);
    view.ride_access_plan = Some(park.ride_access_plan(id).unwrap());
    view.ride_access_notice = None;
    view.prepared_access = None;
    view.path_tool = ToolState::default();
    view.selected = None;
    view.hovered = None;
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = 1.2;
    view.pan = Vec2::ZERO;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn access_captures_have_actual_ready_missing_and_unavailable_topology() {
        for scene in [
            Scene::AccessNeeds,
            Scene::AccessReady,
            Scene::AccessUnavailable,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            prepare(&mut park, &mut view, scene);
            let plan = view.ride_access_plan.as_ref().unwrap();
            assert!(view.rides_open && view.ride_access_open);
            assert_eq!(plan.available, !matches!(scene, Scene::AccessUnavailable));
            if matches!(scene, Scene::AccessReady) {
                assert_eq!(plan.entrance.status, park_sim::RideAccessStatus::Ready);
                assert_eq!(plan.exit.status, park_sim::RideExitStatus::Ready);
            }
            if matches!(scene, Scene::AccessNeeds) {
                assert!(!plan.suggestions.is_empty());
            }
            assert!(view.prepared_access.is_none() && view.ride_access_commands.is_empty());
            assert_eq!(
                Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                park
            );
        }
    }
}
