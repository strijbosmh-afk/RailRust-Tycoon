//! Complete ride snapshots reached through production construction and ticks.
use super::*;
use park_sim::{PathDirection, PathSlope, RIDE_TRANSFER_TICKS, RideId, RideVisitPhase};

const EXIT: TilePos = TilePos { x: 8, y: 9 };
const SEEK_LIMIT: usize = 18_000;

fn roundtrip(park: &Park) {
    let bytes = park
        .to_save_bytes()
        .expect("journey capture must serialize");
    assert_eq!(Park::from_save_bytes(&bytes).unwrap(), *park);
}

fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool, label: &str) {
    for _ in 0..SEEK_LIMIT {
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!("journey capture did not reach {label} within {SEEK_LIMIT} ticks");
}

fn has_transfer(park: &Park, phase: RideVisitPhase) -> bool {
    park.guests().iter().any(|guest| {
        guest.ride_visit.as_ref().is_some_and(|visit| {
            visit.phase == phase
                && visit
                    .passenger
                    .as_ref()
                    .and_then(|passenger| passenger.transfer.as_ref())
                    .is_some_and(|transfer| transfer.progress == RIDE_TRANSFER_TICKS / 4)
        })
    })
}

fn riding(park: &Park) -> bool {
    park.train().is_some_and(|train| train.speed > 0)
        && park.guests().iter().any(|guest| {
            guest.ride_visit.as_ref().is_some_and(|visit| {
                visit.phase == RideVisitPhase::Riding
                    && visit
                        .passenger
                        .as_ref()
                        .is_some_and(|passenger| passenger.departed && !passenger.completed_lap)
            })
        })
}

fn tool_place(park: &mut Park, view: &mut ViewState, pos: TilePos, mode: PathTool) {
    view.path_tool = ToolState {
        mode,
        height: 8,
        exit_facing: PathDirection::South,
        ..default()
    };
    view.selected = None;
    let center = world::path_build_center(pos, park, view.rotation, 8, PathSlope::Flat).unwrap();
    let point = std::iter::once(center)
        .chain((-24..=24).step_by(4).flat_map(|y| {
            (-28..=28)
                .step_by(4)
                .map(move |x| center + Vec2::new(x as f32, y as f32))
        }))
        .find(|point| {
            world::pick_sloped_path_plane(*point, park, view.rotation, 8, PathSlope::Flat)
                == Some(pos)
                && !world::sloped_path_plane_occluded(
                    *point,
                    park,
                    view.rotation,
                    8,
                    PathSlope::Flat,
                    view.view_mode,
                )
        })
        .expect("journey construction target must be visible");
    view.hovered = world::pick_surface(point, park, view.rotation, view.view_mode);
    path_tools::point_at(park, view, point, true);
    let Some(SurfaceId::Object(id)) = view.selected else {
        panic!(
            "journey construction failed at {pos:?}: {:?}",
            view.path_notice
        );
    };
    let object = park.object(id).unwrap();
    assert_eq!(object.pos, pos);
    assert_eq!(object.height, 8);
    assert_eq!(
        object.kind,
        if mode == PathTool::Exit {
            ObjectKind::RideExit
        } else {
            ObjectKind::Path
        }
    );
}

pub(super) fn install(park: &mut Park, view: &mut ViewState) -> RideId {
    let rotation = view.rotation;
    let mode = view.view_mode;
    // Fixture construction uses an explicit inspection view. The final image
    // preserves the requested view; no guest or passenger state is injected.
    view.rotation = 0;
    view.view_mode = ViewMode::Underground;
    queue_scenes::prepare(park, view, Scene::QueueConnected);
    for x in 8..=13 {
        tool_place(park, view, TilePos { x, y: 8 }, PathTool::Build);
    }
    for y in 9..=11 {
        tool_place(park, view, TilePos { x: 13, y }, PathTool::Build);
    }
    tool_place(park, view, EXIT, PathTool::Exit);
    view.rotation = rotation;
    view.view_mode = mode;
    let ride = park.ride().unwrap().id;
    assert_eq!(
        park.ride_exit_access(ride).status,
        park_sim::RideExitStatus::Ready
    );
    roundtrip(park);
    ride
}

fn prepare_state(park: &mut Park, view: &mut ViewState, scene: Scene) {
    assert!(scene.is_journey());
    let ids: Vec<_> = park.guests().iter().map(|guest| guest.id).collect();
    let circuit = park.circuit().cloned();
    let ride = install(park, view);
    match scene {
        Scene::JourneyBoarding => seek(
            park,
            |park| has_transfer(park, RideVisitPhase::Boarding),
            "physical boarding transfer",
        ),
        Scene::JourneyRiding => seek(
            park,
            |park| riding(park) && park.train().unwrap().phase == TrainPhase::Lift,
            "occupied train on the lift",
        ),
        Scene::JourneyUnloading => seek(
            park,
            |park| has_transfer(park, RideVisitPhase::Exiting),
            "physical exit transfer",
        ),
        Scene::JourneyCompleted => seek(
            park,
            |park| {
                park.guests().iter().any(|guest| {
                    guest.rides_completed > 0
                        && guest.ride_visit.is_none()
                        && guest
                            .current
                            .and_then(|id| park.object(id))
                            .is_some_and(|path| {
                                path.kind == ObjectKind::Path && path.queue_owner.is_none()
                            })
                })
            },
            "completed trip on a public path",
        ),
        Scene::JourneyExitBlocked | Scene::JourneyRecovered => {
            seek(park, riding, "occupied moving train before exit demolition");
            let old = park.ride().unwrap().exit.as_ref().unwrap().object;
            let ticks = park.ticks();
            park.remove_object(old)
                .expect("unused exit may be removed during the lap");
            assert_eq!(park.ticks(), ticks);
            roundtrip(park);
            seek(
                park,
                |park| {
                    let stats = park.ride_journey_stats(ride);
                    stats.held && stats.riding > 0 && park.train().unwrap().speed == 0
                },
                "passengers held after returning to a missing exit",
            );
            roundtrip(park);
            if matches!(scene, Scene::JourneyRecovered) {
                let before = park.ride_journey_stats(ride).completed;
                let fresh = park
                    .place_ride_exit(EXIT, 8, PathDirection::South, ride)
                    .unwrap();
                assert!(fresh > old);
                roundtrip(park);
                seek(
                    park,
                    |park| park.ride_journey_stats(ride).completed > before,
                    "completed public return after fresh-ID exit repair",
                );
            }
        }
        _ => unreachable!(),
    }
    assert_eq!(
        park.guests()
            .iter()
            .map(|guest| guest.id)
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(park.circuit(), circuit.as_ref());
    assert_eq!(park.ride().unwrap().id, ride);
    roundtrip(park);
}

fn present(park: &Park, view: &mut ViewState, scene: Scene) {
    view.path_tool = ToolState::default();
    view.selected = None;
    view.hovered = None;
    view.speed = Speed::Paused;
    view.path_notice = Some(
        match scene {
            Scene::JourneyBoarding => "Guests board through the entrance into reserved seats.",
            Scene::JourneyRiding => "Passengers follow the moving train around the circuit.",
            Scene::JourneyUnloading => "Guests leave through the exit toward the public path.",
            Scene::JourneyCompleted => "Ride completed. A guest has returned to the public path.",
            Scene::JourneyExitBlocked => "Passengers wait at the station for an exit connection.",
            Scene::JourneyRecovered => "Exit restored. Passengers can finish their journey.",
            _ => unreachable!(),
        }
        .into(),
    );
    let center = if matches!(scene, Scene::JourneyRiding) {
        let train = park.train().unwrap();
        let piece = park.circuit().unwrap().pieces[train.piece_index].object;
        world::surface_center(park, SurfaceId::Object(piece), view.rotation).unwrap()
    } else {
        world::path_build_center(
            TilePos { x: 8, y: 10 },
            park,
            view.rotation,
            8,
            PathSlope::Flat,
        )
        .unwrap()
    };
    view.zoom = 2.2;
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    prepare_state(park, view, scene);
    present(park, view, scene);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journey_scenes_reach_real_phases_and_present_every_rotation_without_mutation() {
        for profile in [TerrainProfile::Flat, TerrainProfile::Terraced] {
            for scene in [
                Scene::JourneyBoarding,
                Scene::JourneyRiding,
                Scene::JourneyUnloading,
                Scene::JourneyCompleted,
                Scene::JourneyExitBlocked,
                Scene::JourneyRecovered,
            ] {
                let mut park = crate::demonstration_park();
                park.try_apply_terrain_profile(profile).unwrap();
                let mut view = ViewState::default();
                prepare_state(&mut park, &mut view, scene);
                let before = park.clone();
                for rotation in 0..4 {
                    for mode in [ViewMode::Surface, ViewMode::Underground] {
                        view.rotation = rotation;
                        view.view_mode = mode;
                        present(&park, &mut view, scene);
                        assert_eq!(park, before);
                        assert_eq!(view.speed, Speed::Paused);
                        assert!(view.pan.is_finite());
                        assert!(view.path_tool.preview.is_none());
                    }
                }
            }
        }
    }
}
