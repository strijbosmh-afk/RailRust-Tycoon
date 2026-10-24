//! Production track commands and previews on a real, guest-usable new circuit.
use super::*;
use crate::{
    save_check::{ride_choice, track_construction},
    track_tools,
};
use park_sim::{TrackBuildKind, TrackDirection};

fn prepare_state(park: &mut Park, scene: Scene) {
    track_construction::install_access(park).unwrap();
    ride_choice::seek(
        park,
        |p| p.train().is_some_and(|t| t.phase == TrainPhase::Closed),
        "closed track capture",
    )
    .unwrap();
    track_construction::draft(park, !matches!(scene, Scene::TrackDraft)).unwrap();
    if matches!(scene, Scene::TrackInvalid) {
        park.set_track_draft_piece(TrackBuildKind::Lift.spec(
            TilePos { x: 15, y: 8 },
            10,
            TrackDirection::North,
        ))
        .unwrap();
        assert!(park.validate_track_draft().is_err());
        track_construction::reject_apply(park).unwrap();
    }
    if matches!(scene, Scene::TrackBuilt | Scene::TrackOccupied) {
        track_construction::apply(park).unwrap();
        track_construction::occupied(park).unwrap();
    }
    if matches!(scene, Scene::TrackOccupied) {
        track_construction::reject_apply(park).unwrap();
    }
}

fn present(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let before = park.clone();
    view.help = false;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = false;
    view.rides_open = false;
    view.selected_ride = None;
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.path_notice = None;
    view.track_editor = track_tools::TrackEditor {
        active: !matches!(scene, Scene::TrackBuilt),
        ..default()
    };
    let anchor = match scene {
        Scene::TrackInvalid => {
            let pos = TilePos { x: 13, y: 10 };
            let spec = TrackBuildKind::Lift.spec(pos, 8, TrackDirection::East);
            view.track_editor.kind = TrackBuildKind::Lift;
            assert!(park.validate_track_draft_piece(spec).is_err());
            view.track_editor.notice = format!(
                "Height mismatch: {}",
                park.validate_track_draft().unwrap_err()
            );
            pos
        }
        Scene::TrackOccupied => {
            let mut copy = park.clone();
            view.track_editor.kind = TrackBuildKind::Lift;
            view.track_editor.height = 9;
            view.track_editor.notice =
                format!("Apply refused: {}", copy.apply_track_draft().unwrap_err());
            TilePos { x: 11, y: 7 }
        }
        Scene::TrackReady => {
            view.track_editor.kind = TrackBuildKind::SteepDrop;
            view.track_editor.height = 10;
            view.track_editor.notice =
                "Graded loop ready to apply.\nAll endpoint heights match.".into();
            TilePos { x: 11, y: 7 }
        }
        Scene::TrackBuilt => {
            view.path_notice = Some("Paying guests climb the new chain lift.".into());
            TilePos { x: 11, y: 7 }
        }
        Scene::TrackDraft => {
            view.track_editor.kind = TrackBuildKind::Lift;
            view.track_editor.direction = TrackDirection::North;
            view.track_editor.height = 9;
            view.track_editor.notice =
                "14 graded pieces saved in the draft.\nMatch every endpoint before Apply.".into();
            TilePos { x: 15, y: 8 }
        }
        _ => unreachable!(),
    };
    if matches!(scene, Scene::TrackDraft | Scene::TrackInvalid) {
        let height = view.track_editor.height;
        let point = world::tile_center(anchor, park.width(), view.rotation)
            + Vec2::Y * f32::from(height) * world::HEIGHT_STEP;
        assert_eq!(
            world::pick_track_plane(point, park, view.rotation, height),
            Some(anchor)
        );
        track_tools::point_at(park, view, point, false);
        assert_eq!(
            view.track_editor.preview.unwrap().valid,
            matches!(scene, Scene::TrackDraft)
        );
    }
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = if matches!(scene, Scene::TrackInvalid) {
        1.8
    } else {
        1.25
    };
    let center = world::tile_center(anchor, park.width(), view.rotation)
        + Vec2::Y * 8.0 * world::HEIGHT_STEP;
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    assert_eq!(
        *park, before,
        "track preview changed a saved draft or live circuit"
    );
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    prepare_state(park, scene);
    ride_choice::bytes(park).unwrap();
    present(park, view, scene);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn track_scenes_present_saved_drafts_refusal_and_real_paid_trains_without_edits() {
        for scene in [
            Scene::TrackDraft,
            Scene::TrackInvalid,
            Scene::TrackReady,
            Scene::TrackBuilt,
            Scene::TrackOccupied,
        ] {
            let mut park = crate::demonstration_park();
            prepare_state(&mut park, scene);
            let bytes = ride_choice::bytes(&park).unwrap();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            for rotation in 0..4 {
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    view.rotation = rotation;
                    view.view_mode = mode;
                    present(&mut park, &mut view, scene);
                    assert_eq!(bytes, park.to_save_bytes().unwrap());
                }
            }
        }
    }
}
