//! Live aggregate condition and a located visitor, without changing guest fields.
use super::*;

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    assert!(matches!(scene, Scene::GuestStats | Scene::GuestFound));
    if matches!(scene, Scene::GuestStats) {
        for _ in 0..6000 {
            if park.guests().iter().any(|guest| {
                guest.needs.hunger >= 70 || guest.needs.thirst >= 70 || guest.needs.toilet >= 70
            }) {
                break;
            }
            park.advance(Speed::Normal);
        }
        assert!(park.guests().iter().any(|guest| {
            guest.needs.hunger >= 70 || guest.needs.thirst >= 70 || guest.needs.toilet >= 70
        }));
        view.guests_open = true;
        view.guest_stats_view = true;
        view.guest_flow_view = false;
    } else {
        let id = park.guests().iter().map(|guest| guest.id).max().unwrap();
        view.selected_guest = Some(id);
        assert!(crate::ui::guests::focus_selected(park, view));
        view.guests_open = false;
    }
    view.speed = Speed::Paused;
    view.help = false;
    view.rides_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = false;
    view.path_tool = ToolState::default();
    view.scene_dirty = true;
}
