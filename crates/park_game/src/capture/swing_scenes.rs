//! A paid swing departure in the full world and the two-ride management view.
use super::*;
use crate::save_check::swing;

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let (mut fixture, carousel, ride) = swing::fixture().expect("swing capture fixture");
    let mut ready = false;
    for _ in 0..60_000 {
        if fixture
            .ride_by_id(carousel)
            .is_some_and(|row| row.completed > 0)
            && swing::paid_swing_running(&fixture, ride)
        {
            ready = true;
            break;
        }
        fixture.advance(Speed::Normal);
    }
    assert!(
        ready,
        "swing capture needs a paid departure after carousel visits"
    );
    *park = fixture;
    view.help = false;
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.selected_ride = Some(ride);
    view.rides_open = matches!(scene, Scene::SwingRides);
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
            "Carousel and Skyward Swing operate as independent paid rides."
        } else {
            "Skyward Swing carries six guests through a pendulum cycle."
        }
        .into(),
    );
    view.scene_dirty = true;
}
