//! Production track commands and previews on a real, guest-usable new circuit.
use super::*;
use crate::{
    save_check::{ride_choice, track_construction},
    track_tools,
};
use park_sim::{TrackBuildKind, TrackDirection};

const PAID_SITE: TilePos = TilePos { x: 14, y: 5 };

fn is_editing_scene(scene: Scene) -> bool {
    matches!(
        scene,
        Scene::TrackCopied
            | Scene::TrackEdited
            | Scene::TrackUndone
            | Scene::TrackRedone
            | Scene::TrackHistoryEmpty
    )
}

fn paid_object(park: &Park) -> park_sim::ObjectId {
    park.track_economy()
        .paid_pieces
        .iter()
        .find(|p| p.spec.pos == PAID_SITE)
        .unwrap()
        .object
}

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
    if matches!(
        scene,
        Scene::TrackReplacement | Scene::TrackRefund | Scene::TrackFinance
    ) {
        track_construction::apply(park).unwrap();
    }
    if matches!(scene, Scene::TrackReplacement | Scene::TrackFinance) {
        park.set_track_draft_piece(TrackBuildKind::Brake.spec(
            PAID_SITE,
            11,
            TrackDirection::North,
        ))
        .unwrap();
        let quote = park.track_construction_quote().unwrap();
        assert_eq!((quote.construction_cents, quote.refund_cents), (2_000, 500));
    }
    if matches!(scene, Scene::TrackFinance) {
        track_construction::apply(park).unwrap();
        let object = paid_object(park);
        assert_eq!(park.track_demolition_refund_cents(object), 1_000);
        park.remove_object(object).unwrap();
    }
    if matches!(scene, Scene::TrackUnaffordable) {
        crate::save_check::facility_construction::spend_down(park).unwrap();
        assert!(!park.track_construction_quote().unwrap().affordable());
        track_construction::reject_apply(park).unwrap();
    }
    if is_editing_scene(scene) {
        track_construction::apply(park).unwrap();
        park.clear_track_draft();
    }
}

fn prepare_editor(park: &mut Park, view: &mut ViewState, scene: Scene) {
    if !is_editing_scene(scene) {
        return;
    }
    use track_tools::TrackCommand;
    view.track_editor = track_tools::TrackEditor {
        active: true,
        ..default()
    };
    let before = park.clone();
    track_tools::apply_command(park, view, TrackCommand::CopyInstalled);
    assert_eq!(view.track_editor.undo_len(), 1);
    assert_eq!(park.track_draft(), park.circuit().unwrap().specs());
    if !matches!(scene, Scene::TrackCopied | Scene::TrackHistoryEmpty) {
        track_construction::edit_draft_piece(
            park,
            view,
            TrackBuildKind::Brake,
            PAID_SITE,
            11,
            TrackDirection::North,
        )
        .unwrap();
        if matches!(scene, Scene::TrackUndone | Scene::TrackRedone) {
            track_tools::apply_command(park, view, TrackCommand::Undo);
        }
        if matches!(scene, Scene::TrackRedone) {
            track_tools::apply_command(park, view, TrackCommand::Redo);
        }
    }
    if matches!(scene, Scene::TrackHistoryEmpty) {
        // Session history is deliberately absent after a real save roundtrip.
        *park = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        view.track_editor = track_tools::TrackEditor {
            active: true,
            notice: "Loaded draft / history is empty.\nNew edits can be undone.".into(),
            ..default()
        };
    }
    let mut after = park.clone();
    after.clear_track_draft();
    assert_eq!(
        after, before,
        "editor capture preparation changed live track or money"
    );
}

fn present(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let before = park.clone();
    view.help = false;
    view.guests_open = false;
    view.services_open = false;
    view.finance_open = matches!(scene, Scene::TrackFinance);
    view.rides_open = false;
    view.selected_ride = None;
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.path_notice = None;
    if !is_editing_scene(scene) {
        view.track_editor = track_tools::TrackEditor {
            active: !matches!(
                scene,
                Scene::TrackBuilt | Scene::TrackRefund | Scene::TrackFinance
            ),
            ..default()
        };
    }
    let anchor = match scene {
        Scene::TrackCopied
        | Scene::TrackEdited
        | Scene::TrackUndone
        | Scene::TrackRedone
        | Scene::TrackHistoryEmpty => PAID_SITE,
        Scene::TrackInvalid => {
            let pos = TilePos { x: 13, y: 10 };
            let spec = TrackBuildKind::Lift.spec(pos, 8, TrackDirection::East);
            view.track_editor.kind = TrackBuildKind::Lift;
            assert!(park.validate_track_draft_piece(spec).is_err());
            view.track_editor.notice =
                "Lift endpoint does not match.\nBridge blocks the hovered lift.".into();
            pos
        }
        Scene::TrackOccupied => {
            let mut copy = park.clone();
            view.track_editor.kind = TrackBuildKind::Lift;
            view.track_editor.height = 9;
            view.track_editor.notice =
                track_tools::apply_error_message(copy.apply_track_draft().unwrap_err());
            TilePos { x: 11, y: 7 }
        }
        Scene::TrackReady => {
            view.track_editor.kind = TrackBuildKind::SteepDrop;
            view.track_editor.height = 10;
            view.track_editor.notice =
                "Graded loop ready to apply.\nAll endpoint heights match.".into();
            TilePos { x: 11, y: 7 }
        }
        Scene::TrackReplacement => {
            view.track_editor.kind = TrackBuildKind::Brake;
            view.track_editor.height = 11;
            view.track_editor.notice =
                "One straight becomes brakes.\n$5 refund toward $20 brakes.".into();
            PAID_SITE
        }
        Scene::TrackUnaffordable => {
            let mut copy = park.clone();
            view.track_editor.notice =
                track_tools::apply_error_message(copy.apply_track_draft().unwrap_err());
            TilePos { x: 11, y: 7 }
        }
        Scene::TrackRefund => {
            view.selected = Some(SurfaceId::Object(paid_object(park)));
            view.path_notice =
                Some("Inspect paid track, then Delete to remove and refund half.".into());
            PAID_SITE
        }
        Scene::TrackFinance => {
            view.path_notice =
                Some("Paid replacement and rail demolition retained in coaster finances.".into());
            PAID_SITE
        }
        Scene::TrackBuilt => {
            view.path_notice = Some("Paying guests climb the new chain lift.".into());
            TilePos { x: 11, y: 7 }
        }
        Scene::TrackDraft => {
            view.track_editor.kind = TrackBuildKind::Lift;
            view.track_editor.direction = TrackDirection::North;
            view.track_editor.height = 9;
            view.track_editor.notice = "14 draft pieces saved.\nJoin all ends before Apply.".into();
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
    prepare_editor(park, view, scene);
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
            Scene::TrackReplacement,
            Scene::TrackUnaffordable,
            Scene::TrackRefund,
            Scene::TrackFinance,
            Scene::TrackCopied,
            Scene::TrackEdited,
            Scene::TrackUndone,
            Scene::TrackRedone,
            Scene::TrackHistoryEmpty,
        ] {
            let mut park = crate::demonstration_park();
            prepare_state(&mut park, scene);
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            prepare_editor(&mut park, &mut view, scene);
            let bytes = ride_choice::bytes(&park).unwrap();
            let counts = (view.track_editor.undo_len(), view.track_editor.redo_len());
            for rotation in 0..4 {
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    view.rotation = rotation;
                    view.view_mode = mode;
                    present(&mut park, &mut view, scene);
                    assert_eq!(bytes, park.to_save_bytes().unwrap());
                    assert_eq!(
                        (view.track_editor.undo_len(), view.track_editor.redo_len()),
                        counts
                    );
                }
            }
        }
    }
}
