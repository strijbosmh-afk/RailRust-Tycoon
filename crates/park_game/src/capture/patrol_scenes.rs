//! Naturally dirty park with one visible, player-defined handyman patrol area.
use super::*;

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let entry = park
        .objects()
        .iter()
        .find(|path| path.is_public_path() && path.pos == (TilePos { x: 13, y: 16 }))
        .expect("capture public entry")
        .id;
    park.set_visitor_entry(entry).unwrap();
    park.set_handyman_assigned(true);
    park.set_handyman_assigned(false);
    for _ in 0..12_000 {
        if !park.ground_messes().is_empty() {
            break;
        }
        park.advance(Speed::Normal);
    }
    let dirty = park
        .ground_messes()
        .first()
        .expect("natural patrol dirt")
        .path;
    let patrol: Vec<_> = park
        .objects()
        .iter()
        .filter(|path| {
            path.is_public_path()
                && path.id != dirty
                && path.pos.y == 16
                && (12..=14).contains(&path.pos.x)
        })
        .map(|path| path.id)
        .collect();
    assert!(!patrol.is_empty(), "capture patrol paths");
    let selected = patrol[0];
    for path in patrol {
        assert_eq!(park.toggle_handyman_patrol(path), Ok(true));
    }
    park.set_handyman_assigned(true);
    assert!(park.grounds_status().uncovered_messes > 0);
    view.help = false;
    view.rides_open = false;
    view.services_open = false;
    view.guests_open = matches!(scene, Scene::PatrolStats);
    view.guest_stats_view = view.guests_open;
    view.guest_flow_view = false;
    view.selected = Some(SurfaceId::Object(selected));
    view.selected_ride = None;
    view.path_tool = ToolState::default();
    view.zoom = if view.guests_open { 1.0 } else { 4.0 };
    let path = park.object(selected).unwrap().pos;
    view.pan = world::elevated_tile_center(park, path, view.rotation).unwrap()
        - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.path_notice = Some(
        "Cyan path marks show the handyman's patrol area. Empty area means the whole park.".into(),
    );
    view.set_speed(Speed::Paused);
    view.scene_dirty = true;
}
