//! Shared production appeal fixtures; presentation never fabricates reviews.
use super::*;
use crate::{save_check::appeal, ui::rides::RideTab};
use park_sim::{GuestId, RideId, RideVisitPhase};

fn present(park: &Park, view: &mut ViewState, ride: RideId, guest: Option<GuestId>) {
    view.help = false;
    view.rides_open = guest.is_none();
    view.ride_tab = RideTab::Operations;
    view.guests_open = guest.is_some();
    view.services_open = false;
    view.finance_open = false;
    view.designs_open = false;
    view.selected_ride = Some(ride);
    view.selected_guest = guest;
    view.guest_page = guest.map_or(0, |id| {
        park.guests().iter().position(|g| g.id == id).unwrap() / 8
    });
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.path_notice = None;
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = 1.2;
    view.pan = Vec2::ZERO;
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let (ride, guest) = if matches!(scene, Scene::AppealChoiceLow | Scene::AppealChoiceGraded) {
        let pair =
            appeal::choice_pair(park).expect("capture requires an observed layout-choice flip");
        let guest = pair.guest;
        let ride = if matches!(scene, Scene::AppealChoiceLow) {
            pair.coaster
        } else {
            pair.carousel
        };
        *park = if matches!(scene, Scene::AppealChoiceLow) {
            pair.low
        } else {
            pair.graded
        };
        (ride, Some(guest))
    } else {
        let owners = appeal::install(park).unwrap();
        let guest = if matches!(scene, Scene::AppealUnrated) {
            None
        } else {
            Some(appeal::earn_reviews(park, owners).unwrap())
        };
        (
            owners[0],
            if matches!(scene, Scene::AppealGuest) {
                guest
            } else {
                None
            },
        )
    };
    appeal::inspect(park).unwrap();
    let original = park.clone();
    let saved = park.to_save_bytes().unwrap();
    present(park, view, ride, guest);
    assert_eq!(*park, original);
    assert_eq!(park.to_save_bytes().unwrap(), saved);
    assert_scene(park, view, scene);
}

pub(super) fn assert_scene(park: &Park, view: &ViewState, scene: Scene) {
    assert!(view.capture_active);
    assert_eq!(view.speed, Speed::Paused);
    let coaster = park.ride().unwrap().id;
    let profile = park.ride_appeal_profile(coaster).unwrap();
    match scene {
        Scene::AppealUnrated => {
            assert!(view.rides_open && !view.guests_open);
            assert_eq!(profile.review_count, 0);
            assert_eq!(profile.reputation_rating_tenths, 30);
            assert_eq!(park.ticks(), 0);
        }
        Scene::AppealReviewed | Scene::AppealGuest => {
            assert!(
                park.all_rides()
                    .all(|r| r.rating_count > 0 && r.completed > 0 && r.revenue_cents > 0)
            );
            assert!(park.all_rides().any(|r| {
                park.ride_appeal_profile(r.id)
                    .unwrap()
                    .reputation_rating_tenths
                    != 30
            }));
            assert_eq!(view.guests_open, matches!(scene, Scene::AppealGuest));
        }
        Scene::AppealChoiceLow | Scene::AppealChoiceGraded => {
            assert!(view.guests_open && !view.rides_open);
            assert_eq!(profile.review_count, 0);
            assert!(
                park.all_rides()
                    .all(|r| !park.ride_is_running(r.id) && r.rating_count == 0)
            );
            let fit = park
                .guest_ride_appeal(view.selected_guest.unwrap(), coaster)
                .unwrap();
            assert_eq!(
                fit.layout_penalty == 0,
                matches!(scene, Scene::AppealChoiceLow)
            );
        }
        _ => panic!("not an appeal scene"),
    }
    if let Some(id) = view.selected_guest {
        let guest = park.guests().iter().find(|g| g.id == id).unwrap();
        assert!(guest.ride_visit.as_ref().is_some_and(
            |v| Some(v.ride) == view.selected_ride && v.phase == RideVisitPhase::Approaching
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appeal_presentation_preserves_real_state_at_all_rotations() {
        let mut park = crate::demonstration_park();
        let owners = appeal::install(&mut park).unwrap();
        let guest = appeal::earn_reviews(&mut park, owners).unwrap();
        let before = park.clone();
        let saved = park.to_save_bytes().unwrap();
        for rotation in 0..4 {
            let mut view = ViewState {
                rotation,
                capture_active: true,
                ..default()
            };
            present(&park, &mut view, owners[0], Some(guest));
            assert_scene(&park, &view, Scene::AppealGuest);
            assert_eq!(park, before);
            assert_eq!(park.to_save_bytes().unwrap(), saved);
            assert_eq!(view.rotation, rotation);
        }
    }
}
