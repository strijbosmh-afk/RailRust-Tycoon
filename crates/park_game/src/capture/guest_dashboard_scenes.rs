//! Live aggregate condition and a located visitor, without changing guest fields.
use super::*;

pub(super) fn prepare(
    park: &mut Park,
    view: &mut ViewState,
    scene: Scene,
    mut history: Option<&mut crate::ui::guest_stats::GuestStatsHistory>,
) {
    assert!(matches!(
        scene,
        Scene::GuestStats | Scene::GuestTrends | Scene::GuestTriage | Scene::GuestFound
    ));
    if matches!(
        scene,
        Scene::GuestStats | Scene::GuestTrends | Scene::GuestTriage
    ) {
        if let Some(history) = history.as_deref_mut() {
            history.observe(park, view.park_generation);
        }
        let limit = if matches!(scene, Scene::GuestTrends) {
            3_600
        } else {
            6_000
        };
        for _ in 0..limit {
            if park.guests().iter().any(|guest| {
                guest.needs.hunger >= 70 || guest.needs.thirst >= 70 || guest.needs.toilet >= 70
            }) && !matches!(scene, Scene::GuestTrends)
            {
                break;
            }
            park.advance(Speed::Normal);
            if let Some(history) = history.as_deref_mut() {
                history.observe(park, view.park_generation);
            }
        }
        if !matches!(scene, Scene::GuestTrends) {
            assert!(park.guests().iter().any(|guest| {
                guest.needs.hunger >= 70 || guest.needs.thirst >= 70 || guest.needs.toilet >= 70
            }));
        }
        view.guests_open = true;
        view.guest_stats_view = !matches!(scene, Scene::GuestTriage);
        if matches!(scene, Scene::GuestTriage) {
            view.guest_filter = crate::ui::guests::GuestFilter::Urgent;
        }
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
