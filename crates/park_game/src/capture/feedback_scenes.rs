//! Inspect real guest feedback; only transient presentation is set directly.
use super::*;
use crate::save_check::{economy, feedback};
use park_sim::{GuestId, GuestThought, PathSlope};

fn prepare_state(park: &mut Park, view: &mut ViewState, scene: Scene) -> GuestId {
    assert!(scene.is_feedback());
    if matches!(scene, Scene::FeedbackGuests) {
        // Exercise the second page without inventing a review or guest state.
        return park.guests().iter().map(|guest| guest.id).max().unwrap();
    }
    if matches!(scene, Scene::FeedbackQueue | Scene::FeedbackCancelled) {
        let rotation = view.rotation;
        let mode = view.view_mode;
        view.rotation = 0;
        view.view_mode = ViewMode::Underground;
        queue_scenes::prepare(park, view, Scene::QueueConnected);
        view.rotation = rotation;
        view.view_mode = mode;
    } else {
        journey_scenes::install(park, view);
    }
    let ride = park.ride().unwrap().id;
    let value_scene = matches!(scene, Scene::FeedbackValue | Scene::FeedbackRecovered);
    economy::set_price_checked(park, if value_scene { 1100 } else { 200 }).unwrap();
    let selected = match scene {
        Scene::FeedbackValue | Scene::FeedbackRecovered => {
            feedback::seek(
                park,
                |p| feedback::has_thought(p, GuestThought::TooExpensive).is_some(),
                "real value refusal",
            )
            .unwrap();
            let rejected = feedback::has_thought(park, GuestThought::TooExpensive).unwrap();
            let guest = park
                .guests()
                .iter()
                .find(|guest| guest.id == rejected)
                .unwrap();
            assert!(guest.wallet_cents >= 1100 && !guest.willing_to_pay(1100));
            assert!(park.ride_experience_stats(ride).value_rejections > 0);
            assert_eq!(park.ride_economy_stats(ride).unaffordable_rejections, 0);
            if matches!(scene, Scene::FeedbackRecovered) {
                economy::set_price_checked(park, 200).unwrap();
                feedback::seek(
                    park,
                    |p| {
                        p.guests().iter().any(|guest| {
                            guest.id == rejected
                                && guest.ratings_given > 0
                                && guest.ride_visit.is_none()
                                && guest.thought == GuestThought::EnjoyedRide
                        })
                    },
                    "the previously rejecting guest completes an affordable good-value ride",
                )
                .unwrap();
            }
            rejected
        }
        Scene::FeedbackQueue | Scene::FeedbackCancelled => {
            feedback::seek(
                park,
                |p| feedback::has_thought(p, GuestThought::WaitingTooLong).is_some(),
                "actual queue discomfort",
            )
            .unwrap();
            assert_eq!(park.ride_experience_stats(ride).rating_count, 0);
            if matches!(scene, Scene::FeedbackCancelled) {
                feedback::close_checked(park).unwrap()
            } else {
                feedback::has_thought(park, GuestThought::WaitingTooLong).unwrap()
            }
        }
        Scene::FeedbackSatisfied => {
            feedback::seek(
                park,
                |p| {
                    p.guests().iter().any(|guest| {
                        guest.ratings_given > 0
                            && guest.thought == GuestThought::EnjoyedRide
                            && guest.ride_visit.is_none()
                    })
                },
                "a genuine positive public-return review",
            )
            .unwrap();
            feedback::has_thought(park, GuestThought::EnjoyedRide).unwrap()
        }
        _ => unreachable!(),
    };
    feedback::check(park).unwrap();
    let bytes = park.to_save_bytes().unwrap();
    assert_eq!(Park::from_save_bytes(&bytes).unwrap(), *park);
    selected
}

fn present(park: &Park, view: &mut ViewState, selected: GuestId) {
    view.path_tool = ToolState::default();
    view.selected = None;
    view.hovered = None;
    view.path_notice = None;
    view.speed = Speed::Paused;
    view.help = false;
    view.guests_open = true;
    view.selected_guest = Some(selected);
    let mut ids: Vec<_> = park.guests().iter().map(|guest| guest.id).collect();
    ids.sort();
    view.guest_page = ids.iter().position(|id| *id == selected).unwrap() / 8;
    view.zoom = 2.0;
    let center = world::path_build_center(
        TilePos { x: 8, y: 13 },
        park,
        view.rotation,
        8,
        PathSlope::South,
    )
    .unwrap();
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let selected = prepare_state(park, view, scene);
    present(park, view, selected);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feedback_scenes_reach_real_thoughts_and_reviews_without_view_mutation() {
        for terrain in [TerrainProfile::Flat, TerrainProfile::Terraced] {
            for scene in [
                Scene::FeedbackValue,
                Scene::FeedbackQueue,
                Scene::FeedbackSatisfied,
                Scene::FeedbackCancelled,
                Scene::FeedbackRecovered,
                Scene::FeedbackGuests,
            ] {
                let mut park = crate::demonstration_park();
                park.try_apply_terrain_profile(terrain).unwrap();
                for _ in 0..600 {
                    feedback::advance_checked(&mut park).unwrap();
                }
                let mut view = ViewState {
                    capture_active: true,
                    ..default()
                };
                let selected = prepare_state(&mut park, &mut view, scene);
                let saved = park.to_save_bytes().unwrap();
                for rotation in 0..4 {
                    for mode in [ViewMode::Surface, ViewMode::Underground] {
                        view.rotation = rotation;
                        view.view_mode = mode;
                        present(&park, &mut view, selected);
                        assert_eq!(saved, park.to_save_bytes().unwrap());
                        assert!(view.guests_open && view.selected_guest == Some(selected));
                        assert!(view.pan.is_finite());
                        assert_eq!(view.speed, Speed::Paused);
                        assert!(view.save_pending.is_none());
                    }
                }
            }
        }
    }
}
