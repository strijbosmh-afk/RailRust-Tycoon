//! Admission snapshots reached by actual complete simulation ticks.
use super::*;
use park_sim::{GuestState, PathSlope, RideAccessStatus, RideId, RideVisitPhase, WALK_TICKS};

const SEEK_LIMIT: usize = 12_000;

fn roundtrip(park: &Park) {
    let bytes = park
        .to_save_bytes()
        .expect("admission capture state must serialize");
    assert_eq!(Park::from_save_bytes(&bytes).unwrap(), *park);
}

fn seek(park: &mut Park, ready: impl Fn(&Park) -> bool, description: &str) {
    for _ in 0..SEEK_LIMIT {
        if ready(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!("admission capture did not reach {description} within {SEEK_LIMIT} ticks");
}

fn moving_ramp(park: &Park) -> bool {
    park.guests().iter().any(|guest| {
        guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
            && guest.state == GuestState::Walking
            && guest.progress == 7
            && [guest.current, guest.next].into_iter().flatten().any(|id| {
                park.object(id).is_some_and(|path| {
                    path.queue_owner.is_some() && path.path_slope != PathSlope::Flat
                })
            })
    })
}

fn full(park: &Park, ride: RideId) -> bool {
    let stats = park.ride_queue_stats(ride);
    stats.capacity == 4 && stats.queueing == stats.capacity && stats.waiting > 0 && !stats.draining
}

fn prepare_state(park: &mut Park, view: &mut ViewState, scene: Scene) {
    assert!(scene.is_admission());
    let guest_ids: Vec<_> = park.guests().iter().map(|guest| guest.id).collect();
    let circuit = park.circuit().cloned();
    // This uses the already verified deliberate queue and gate tool commands.
    queue_scenes::prepare(park, view, Scene::QueueConnected);
    let ride = park.ride().unwrap().id;
    let initial = park.ride_queue_stats(ride);
    roundtrip(park);
    match scene {
        Scene::AdmissionApproach => seek(
            park,
            |park| {
                let stats = park.ride_queue_stats(ride);
                stats.seeking > 0 && stats.queueing == 0
            },
            "a public-path approach before admission",
        ),
        Scene::AdmissionMoving => seek(park, moving_ramp, "a guest moving across a queue ramp"),
        Scene::AdmissionFull => seek(
            park,
            |park| full(park, ride),
            "the four-member queue capacity",
        ),
        Scene::AdmissionLeaving => {
            seek(
                park,
                |park| full(park, ride),
                "the full queue before its patience expires",
            );
            seek(
                park,
                |park| {
                    let stats = park.ride_queue_stats(ride);
                    stats.draining
                        && stats.leaving > 0
                        && stats.abandoned > initial.abandoned
                        && park.guests().iter().any(|guest| {
                            guest
                                .ride_visit
                                .as_ref()
                                .is_some_and(|visit| visit.phase == RideVisitPhase::Leaving)
                                && guest.state == GuestState::Walking
                                && guest.progress > 0
                                && guest.progress < WALK_TICKS
                        })
                },
                "physical return after patience expiration",
            );
            assert!(park.train().unwrap().open);
            assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
        }
        Scene::AdmissionCancelled | Scene::AdmissionRecovered => {
            seek(
                park,
                |park| full(park, ride),
                "an occupied queue before demolition",
            );
            let before = park.ride_queue_stats(ride);
            let path = park
                .guests()
                .iter()
                .find_map(|guest| {
                    guest
                        .current
                        .and_then(|id| park.object(id))
                        .filter(|path| {
                            path.queue_owner == Some(ride) && path.pos == (TilePos { x: 8, y: 13 })
                        })
                        .copied()
                })
                .or_else(|| {
                    park.guests().iter().find_map(|guest| {
                        guest
                            .current
                            .and_then(|id| park.object(id))
                            .filter(|path| path.queue_owner == Some(ride))
                            .copied()
                    })
                })
                .expect("full queue has an occupied path");
            let ticks = park.ticks();
            park.remove_object(path.id)
                .expect("occupied queue demolition must succeed");
            assert_eq!(
                park.ticks(),
                ticks,
                "demolition must reconcile while paused"
            );
            let cancelled = park.ride_queue_stats(ride);
            assert!(cancelled.draining && cancelled.leaving > 0);
            assert!(cancelled.abandoned > before.abandoned);
            roundtrip(park);
            if matches!(scene, Scene::AdmissionRecovered) {
                let replacement = park
                    .place_queue_path(path.pos, path.height, path.path_slope, ride)
                    .unwrap();
                assert!(replacement > path.id);
                roundtrip(park);
                seek(
                    park,
                    |park| {
                        let stats = park.ride_queue_stats(ride);
                        stats.joined > before.joined
                            && stats.queueing > 0
                            && stats.stranded == 0
                            && !stats.draining
                    },
                    "renewed admission after fresh-ID repair and physical return",
                );
                assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
            }
        }
        _ => unreachable!(),
    }
    assert_eq!(
        park.guests()
            .iter()
            .map(|guest| guest.id)
            .collect::<Vec<_>>(),
        guest_ids
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
            Scene::AdmissionApproach => "Guests are approaching Meadow Flyer.",
            Scene::AdmissionMoving => "Guests move toward the front of the queue.",
            Scene::AdmissionFull => "Queue capacity reached. No additional guest is admitted.",
            Scene::AdmissionLeaving => "Patience expired. The queue is returning to public paths.",
            Scene::AdmissionCancelled => {
                "Queue interrupted. Guests return or wait for a connection."
            }
            Scene::AdmissionRecovered => "Connection restored. Guests can join the queue again.",
            _ => unreachable!(),
        }
        .into(),
    );
    let start = world::path_build_center(
        TilePos { x: 8, y: 11 },
        park,
        view.rotation,
        8,
        PathSlope::Flat,
    )
    .unwrap();
    let end = world::path_build_center(
        TilePos { x: 8, y: 16 },
        park,
        view.rotation,
        8,
        PathSlope::Flat,
    )
    .unwrap();
    view.zoom = 2.5;
    view.pan = (start + end) * 0.5
        - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
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
    fn admission_captures_reach_real_phases_and_keep_all_rotated_views_read_only() {
        for profile in [TerrainProfile::Flat, TerrainProfile::Terraced] {
            for scene in [
                Scene::AdmissionApproach,
                Scene::AdmissionMoving,
                Scene::AdmissionFull,
                Scene::AdmissionLeaving,
                Scene::AdmissionCancelled,
                Scene::AdmissionRecovered,
            ] {
                let mut park = crate::demonstration_park();
                park.try_apply_terrain_profile(profile).unwrap();
                let mut view = ViewState::default();
                // Reach each real phase once, then reuse it across view-only combinations.
                prepare_state(&mut park, &mut view, scene);
                let before = park.clone();
                for rotation in 0..4 {
                    for view_mode in [ViewMode::Surface, ViewMode::Underground] {
                        view.rotation = rotation;
                        view.view_mode = view_mode;
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
