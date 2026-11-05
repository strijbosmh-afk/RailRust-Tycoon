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
        Scene::GuestStats
            | Scene::GroundsStats
            | Scene::BinsStats
            | Scene::GuestTrends
            | Scene::GuestTriage
            | Scene::GuestRepair
            | Scene::GuestRepairMap
            | Scene::RestFacility
            | Scene::RestFacilityMap
            | Scene::GuestFound
    ));
    if matches!(scene, Scene::RestFacility | Scene::RestFacilityMap) {
        let pos = TilePos { x: 12, y: 21 };
        let id = park
            .place_service(
                pos,
                8,
                park_sim::PathDirection::North,
                park_sim::ServiceKind::Rest,
            )
            .unwrap();
        view.selected_service = Some(id);
        view.guests_open = false;
        if matches!(scene, Scene::RestFacilityMap) {
            view.selected = park.service(id).unwrap().object.map(SurfaceId::Object);
            view.zoom = 4.0;
            view.pan = world::elevated_tile_center(park, pos, view.rotation)
                .expect("Rest position")
                - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
        }
    } else if matches!(scene, Scene::GuestRepair | Scene::GuestRepairMap) {
        let id = park.guests()[0].id;
        let path = park.guests()[0].current.unwrap();
        park.remove_object(path).unwrap();
        view.selected_guest = Some(id);
        view.guest_filter = crate::ui::guests::GuestFilter::Lost;
        view.guests_open = true;
        if matches!(scene, Scene::GuestRepairMap) {
            assert!(crate::ui::guests::prepare_repair(park, view));
        }
    } else if matches!(scene, Scene::GroundsStats | Scene::BinsStats) {
        let entry = park
            .objects()
            .iter()
            .find(|path| path.is_public_path() && path.pos == (TilePos { x: 13, y: 16 }))
            .expect("capture public path")
            .id;
        park.set_visitor_entry(entry).expect("capture entry");
        if matches!(scene, Scene::BinsStats) {
            assert_eq!(park.toggle_ground_bin(entry), Ok(true));
        }
        park.set_handyman_assigned(true);
        park.set_handyman_assigned(false);
        for _ in 0..12_000 {
            if !park.ground_messes().is_empty() {
                break;
            }
            park.advance(Speed::Normal);
        }
        assert!(!park.ground_messes().is_empty());
        park.set_handyman_assigned(true);
        view.guests_open = true;
        view.guest_stats_view = true;
        view.guest_flow_view = false;
    } else if matches!(
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
    view.services_open = matches!(scene, Scene::RestFacility);
    view.finance_open = false;
    view.designs_open = false;
    if !matches!(scene, Scene::GuestRepairMap) {
        view.path_tool = ToolState::default();
    }
    view.scene_dirty = true;
}
