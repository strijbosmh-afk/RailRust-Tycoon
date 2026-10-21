//! Actual construction and moving guests on a two-height branch.
use super::*;
use park_sim::{Axis, GuestState, PathSlope};

const RAMP: TilePos = TilePos { x: 13, y: 21 };

fn point(park: &Park, view: &ViewState, pos: TilePos, height: u8, slope: PathSlope) -> Vec2 {
    let center = world::path_build_center(pos, park, view.rotation, height, slope).unwrap();
    for offset in [
        Vec2::ZERO,
        Vec2::new(-12.0, 0.0),
        Vec2::new(12.0, 0.0),
        Vec2::new(0.0, 7.0),
        Vec2::new(0.0, -7.0),
    ] {
        let point = center + offset;
        if world::pick_sloped_path_plane(point, park, view.rotation, height, slope) == Some(pos)
            && !world::sloped_path_plane_occluded(
                point,
                park,
                view.rotation,
                height,
                slope,
                view.view_mode,
            )
        {
            return point;
        }
    }
    center
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    assert!(scene.is_slope_tool());
    for (pos, height, slope) in [
        (TilePos { x: 13, y: 22 }, 9, PathSlope::Flat),
        (TilePos { x: 14, y: 22 }, 9, PathSlope::East),
        (TilePos { x: 15, y: 22 }, 10, PathSlope::Flat),
    ] {
        park.place_sloped_path(pos, height, slope).unwrap();
    }
    view.path_tool = ToolState {
        mode: PathTool::Build,
        height: 8,
        slope: PathSlope::South,
        ..default()
    };
    view.selected = None;
    if matches!(scene, Scene::SlopeBlocked) {
        park.place_object(RAMP, ObjectKind::Track, Axis::X, 9, 2)
            .unwrap();
    }
    let mut focus = point(park, view, RAMP, 8, PathSlope::South);
    let before = park.clone();
    view.hovered = world::pick_surface(focus, park, view.rotation, view.view_mode);
    path_tools::point_at(park, view, focus, !matches!(scene, Scene::SlopeValid));
    if matches!(scene, Scene::SlopeValid | Scene::SlopeBlocked) {
        assert_eq!(*park, before);
        if matches!(scene, Scene::SlopeValid) {
            assert!(view.path_tool.preview.is_some_and(|preview| preview.valid));
        } else {
            assert!(
                view.path_notice
                    .as_ref()
                    .unwrap()
                    .contains("No path placed")
            );
        }
    } else {
        let Some(SurfaceId::Object(ramp)) = view.selected else {
            panic!("slope capture must build the bridge");
        };
        assert_eq!(park.object(ramp).unwrap().path_slope, PathSlope::South);
        if !matches!(scene, Scene::SlopeBuilt) {
            let mut guest_id = None;
            for _ in 0..18_000 {
                guest_id = park
                    .guests()
                    .iter()
                    .find(|guest| {
                        guest.current == Some(ramp)
                            && guest.state == GuestState::Walking
                            && guest.progress == 7
                    })
                    .map(|guest| guest.id);
                if guest_id.is_some() {
                    break;
                }
                park.advance(Speed::Normal);
            }
            let guest_id = guest_id.expect("capture needs a guest travelling on its bridge");
            if matches!(scene, Scene::SlopeRemoved | Scene::SlopeRecovered) {
                focus =
                    world::surface_center(park, SurfaceId::Object(ramp), view.rotation).unwrap();
                assert_eq!(
                    world::pick_surface(focus, park, view.rotation, view.view_mode),
                    Some(SurfaceId::Object(ramp))
                );
                view.path_tool.set_mode(PathTool::Demolish);
                view.hovered = Some(SurfaceId::Object(ramp));
                path_tools::point_at(park, view, focus, true);
                assert!(park.object(ramp).is_none());
                let guest = park
                    .guests()
                    .iter()
                    .find(|guest| guest.id == guest_id)
                    .unwrap();
                assert_eq!(guest.state, GuestState::Lost);
                assert_eq!(guest.anchor_slope, PathSlope::South);
                if matches!(scene, Scene::SlopeRecovered) {
                    view.path_tool.set_mode(PathTool::Build);
                    path_tools::point_at(park, view, focus, true);
                    let Some(SurfaceId::Object(rebuilt)) = view.selected else {
                        panic!("slope must rebuild");
                    };
                    assert!(rebuilt > ramp);
                    for _ in 0..120 {
                        park.advance(Speed::Normal);
                    }
                    assert_ne!(
                        park.guests()
                            .iter()
                            .find(|g| g.id == guest_id)
                            .unwrap()
                            .state,
                        GuestState::Lost
                    );
                }
            }
        }
    }
    view.zoom = 3.0;
    view.pan = focus - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slope_capture_commands_preserve_valid_continuation_in_all_rotations_and_views() {
        for rotation in 0..4 {
            for view_mode in [ViewMode::Surface, ViewMode::Underground] {
                for scene in [
                    Scene::SlopeValid,
                    Scene::SlopeBuilt,
                    Scene::SlopeBlocked,
                    Scene::SlopeCrowd,
                    Scene::SlopeRemoved,
                    Scene::SlopeRecovered,
                ] {
                    let mut park = crate::demonstration_park();
                    let mut view = ViewState {
                        rotation,
                        view_mode,
                        speed: Speed::Paused,
                        ..default()
                    };
                    prepare(&mut park, &mut view, scene);
                    assert_eq!(
                        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                        park
                    );
                    assert_eq!(park.guests().len(), 12);
                    assert!(park.train().is_some());
                    assert_eq!(view.speed, Speed::Paused);
                }
            }
        }
    }
}
