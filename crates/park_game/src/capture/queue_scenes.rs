//! Queue/access fixtures prepared through the same commands as normal tools.
use super::*;
use park_sim::{Axis, ObjectId, PathDirection, PathSlope, RideAccessStatus, RideId};

const GATE: TilePos = TilePos { x: 8, y: 11 };
const BRIDGE: TilePos = TilePos { x: 8, y: 13 };
const QUEUE: [(TilePos, u8, PathSlope); 4] = [
    (TilePos { x: 8, y: 12 }, 8, PathSlope::Flat),
    (BRIDGE, 8, PathSlope::South),
    (TilePos { x: 8, y: 14 }, 9, PathSlope::Flat),
    (TilePos { x: 8, y: 15 }, 8, PathSlope::North),
];

fn pointer_candidates(center: Vec2) -> impl Iterator<Item = Vec2> {
    std::iter::once(center).chain((-24..=24).step_by(4).flat_map(move |y| {
        (-28..=28)
            .step_by(4)
            .map(move |x| center + Vec2::new(x as f32, y as f32))
    }))
}

fn build_point(park: &Park, view: &ViewState, pos: TilePos, height: u8, slope: PathSlope) -> Vec2 {
    let center = world::path_build_center(pos, park, view.rotation, height, slope).unwrap();
    pointer_candidates(center)
        .find(|point| {
            world::pick_sloped_path_plane(*point, park, view.rotation, height, slope) == Some(pos)
                && !world::sloped_path_plane_occluded(
                    *point,
                    park,
                    view.rotation,
                    height,
                    slope,
                    view.view_mode,
                )
        })
        .unwrap_or(center)
}

fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    view.hovered = world::pick_surface(point, park, view.rotation, view.view_mode);
    path_tools::point_at(park, view, point, click);
}

fn build_queue(
    park: &mut Park,
    view: &mut ViewState,
    pos: TilePos,
    height: u8,
    slope: PathSlope,
    ride: RideId,
) -> ObjectId {
    view.path_tool = ToolState {
        mode: PathTool::BuildQueue,
        height,
        slope,
        ..default()
    };
    let point = build_point(park, view, pos, height, slope);
    point_at(park, view, point, true);
    let Some(SurfaceId::Object(id)) = view.selected else {
        panic!(
            "queue capture must place its requested path: {:?}",
            view.path_notice
        );
    };
    let object = park.object(id).unwrap();
    assert_eq!(object.pos, pos);
    assert_eq!(object.height, height);
    assert_eq!(object.path_slope, slope);
    assert_eq!(object.queue_owner, Some(ride));
    id
}

fn remove_visible(park: &mut Park, view: &mut ViewState, id: ObjectId) {
    let center = world::surface_center(park, SurfaceId::Object(id), view.rotation).unwrap();
    let point = pointer_candidates(center)
        .find(|point| {
            world::pick_surface(*point, park, view.rotation, view.view_mode)
                == Some(SurfaceId::Object(id))
        })
        .expect("capture demolition target must be visible");
    view.path_tool.set_mode(PathTool::Demolish);
    point_at(park, view, point, true);
    assert!(park.object(id).is_none());
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    assert!(matches!(
        scene,
        Scene::QueueEntrance
            | Scene::QueueConnected
            | Scene::QueueDisconnected
            | Scene::QueueBranch
            | Scene::QueueBlocked
            | Scene::QueueRebuilt
    ));
    let before = park.clone();
    let ride = park
        .ride()
        .expect("queue fixture requires the train ride")
        .id;
    let mut bridge = None;
    for (pos, height, slope) in QUEUE {
        let id = build_queue(park, view, pos, height, slope, ride);
        if pos == BRIDGE {
            bridge = Some(id);
        }
    }
    if matches!(scene, Scene::QueueBlocked) {
        park.place_object(GATE, ObjectKind::Track, Axis::X, 8, 2)
            .unwrap();
    }
    view.path_tool = ToolState {
        mode: PathTool::Entrance,
        height: 8,
        entrance_facing: PathDirection::North,
        ..default()
    };
    view.selected = None;
    let gate_point = build_point(park, view, GATE, 8, PathSlope::Flat);
    let before_gate = park.clone();
    point_at(
        park,
        view,
        gate_point,
        !matches!(scene, Scene::QueueEntrance),
    );
    if matches!(scene, Scene::QueueEntrance | Scene::QueueBlocked) {
        assert_eq!(*park, before_gate);
        assert_eq!(park.ride_access(ride).status, RideAccessStatus::NoEntrance);
        if matches!(scene, Scene::QueueEntrance) {
            assert!(view.path_tool.preview.is_some_and(|preview| {
                preview.valid && preview.entrance_facing == Some(PathDirection::North)
            }));
        } else {
            assert!(view.path_notice.is_some());
            assert!(park.ride().unwrap().entrance.is_none());
        }
    } else {
        let gate = park.ride().unwrap().entrance.as_ref().unwrap().object;
        assert_eq!(park.object(gate).unwrap().kind, ObjectKind::RideEntrance);
        assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
        match scene {
            Scene::QueueDisconnected | Scene::QueueRebuilt => {
                let removed = bridge.unwrap();
                remove_visible(park, view, removed);
                assert_eq!(
                    park.ride_access(ride).status,
                    RideAccessStatus::Disconnected
                );
                if matches!(scene, Scene::QueueRebuilt) {
                    let replacement = build_queue(park, view, BRIDGE, 8, PathSlope::South, ride);
                    assert!(replacement > removed);
                    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
                }
            }
            Scene::QueueBranch => {
                build_queue(
                    park,
                    view,
                    TilePos { x: 9, y: 14 },
                    9,
                    PathSlope::Flat,
                    ride,
                );
                assert_eq!(park.ride_access(ride).status, RideAccessStatus::Branch);
            }
            Scene::QueueConnected => {}
            _ => unreachable!(),
        }
    }
    // Construction must not silently advance or replace unrelated simulation state.
    assert_eq!(park.ticks(), before.ticks());
    assert_eq!(park.guests(), before.guests());
    assert_eq!(park.train(), before.train());
    assert_eq!(park.circuit(), before.circuit());
    for object in before.objects() {
        assert_eq!(park.object(object.id), Some(object));
    }
    let far = world::path_build_center(
        TilePos { x: 8, y: 16 },
        park,
        view.rotation,
        8,
        PathSlope::Flat,
    )
    .unwrap();
    let focus = (gate_point + far) * 0.5;
    view.zoom = 2.5;
    view.pan = focus - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_capture_commands_preserve_unrelated_state_and_valid_saves_in_all_views() {
        for profile in [TerrainProfile::Flat, TerrainProfile::Terraced] {
            for rotation in 0..4 {
                for view_mode in [ViewMode::Surface, ViewMode::Underground] {
                    for scene in [
                        Scene::QueueEntrance,
                        Scene::QueueConnected,
                        Scene::QueueDisconnected,
                        Scene::QueueBranch,
                        Scene::QueueBlocked,
                        Scene::QueueRebuilt,
                    ] {
                        let mut park = crate::demonstration_park();
                        park.try_apply_terrain_profile(profile).unwrap();
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
                        assert_eq!(view.speed, Speed::Paused);
                    }
                }
            }
        }
    }
}
