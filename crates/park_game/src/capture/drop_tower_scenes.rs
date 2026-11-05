//! A paid drop_tower departure in the full world and the three-ride management view.
use super::*;
use crate::save_check::drop_tower;

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let (mut fixture, carousel, ride) = drop_tower::fixture().expect("drop_tower capture fixture");
    let mut ready = false;
    for _ in 0..60_000 {
        if fixture
            .ride_by_id(carousel)
            .is_some_and(|row| row.completed > 0)
            && drop_tower::paid_drop_tower_running(&fixture, ride)
        {
            ready = true;
            break;
        }
        fixture.advance(Speed::Normal);
    }
    assert!(
        ready,
        "drop_tower capture needs a paid departure after carousel visits"
    );
    *park = fixture;
    view.help = false;
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.selected_ride = Some(ride);
    view.rides_open = matches!(scene, Scene::DropTowerRides);
    view.finance_open = false;
    view.guests_open = false;
    view.zoom = if view.rides_open { 2.0 } else { 1.7 };
    let focus = if view.rides_open {
        TilePos { x: 16, y: 16 }
    } else {
        TilePos { x: 11, y: 15 }
    };
    view.pan = world::tile_center(focus, MAP_SIZE, view.rotation);
    view.path_notice = Some(
        if view.rides_open {
            "Carousel and Beacon Drop operate as independent paid rides."
        } else {
            "Beacon Drop carries six guests through a lift and drop cycle."
        }
        .into(),
    );
    view.scene_dirty = true;
}
