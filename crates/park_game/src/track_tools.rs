//! Deliberate, saved track drafting. Only Apply replaces the live coaster.

use bevy::prelude::*;
use park_sim::{Park, TrackBuildKind, TrackDirection, TrackPieceSpec};

use crate::{Simulation, ViewState, world};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TrackCommand {
    Piece(TrackBuildKind),
    Turn,
    Height(i8),
    Erase,
    Clear,
    Apply,
    Done,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TrackPreview {
    pub spec: TrackPieceSpec,
    pub valid: bool,
    pub erase: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct TrackEditor {
    pub active: bool,
    pub kind: TrackBuildKind,
    pub direction: TrackDirection,
    pub height: u8,
    pub erase: bool,
    pub preview: Option<TrackPreview>,
    pub revision: u64,
    pub pending: Vec<TrackCommand>,
    pub notice: String,
}

impl Default for TrackEditor {
    fn default() -> Self {
        Self {
            active: false,
            kind: TrackBuildKind::Station,
            direction: TrackDirection::East,
            height: 8,
            erase: false,
            preview: None,
            revision: 0,
            pending: Vec::new(),
            notice: "Click to draft. Cyan rails are saved.\nApply replaces the existing coaster."
                .into(),
        }
    }
}

pub(crate) fn direction_name(direction: TrackDirection) -> &'static str {
    match direction {
        TrackDirection::North => "North",
        TrackDirection::East => "East",
        TrackDirection::South => "South",
        TrackDirection::West => "West",
    }
}

pub(crate) fn kind_name(kind: TrackBuildKind) -> &'static str {
    match kind {
        TrackBuildKind::Station => "Station",
        TrackBuildKind::Straight => "Straight",
        TrackBuildKind::CurveLeft => "Left curve",
        TrackBuildKind::CurveRight => "Right curve",
    }
}

pub(crate) fn begin(park: &Park, view: &mut ViewState) {
    if view.capture_active
        || view.diagnostic_case.is_some()
        || view
            .save_pending
            .is_some_and(|action| action != crate::SaveAction::Save)
    {
        return;
    }
    view.set_modal(false, false, false, false);
    view.modal_transition_pending = true;
    view.path_tool
        .set_mode(crate::path_tools::PathTool::Inspect);
    view.selected_ride = park.ride().map(|ride| ride.id);
    view.selected = None;
    view.track_editor.active = true;
    view.track_editor.revision = view.track_editor.revision.wrapping_add(1);
    view.track_editor.preview = None;
    view.track_editor.notice =
        "Click to draft. Cyan rails are saved.\nApply replaces the existing coaster.".into();
}

pub(crate) fn queue(view: &mut ViewState, command: TrackCommand) {
    if view.track_editor.active && !view.edits_blocked() {
        view.track_editor.pending.push(command);
        view.ui_pointer_consumed = true;
    }
}

pub(crate) fn apply_commands(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    let commands = std::mem::take(&mut view.track_editor.pending);
    if commands.is_empty() || !view.track_editor.active || view.edits_blocked() {
        return;
    }
    // Closing the editor wins over an Apply/Clear received in the same frame.
    if commands.contains(&TrackCommand::Done) {
        view.track_editor.active = false;
        view.track_editor.preview = None;
        view.ui_pointer_consumed = true;
        return;
    }
    for command in commands {
        view.track_editor.preview = None;
        match command {
            TrackCommand::Piece(kind) => {
                view.track_editor.kind = kind;
                view.track_editor.erase = false;
            }
            TrackCommand::Turn => {
                view.track_editor.direction = match view.track_editor.direction {
                    TrackDirection::North => TrackDirection::East,
                    TrackDirection::East => TrackDirection::South,
                    TrackDirection::South => TrackDirection::West,
                    TrackDirection::West => TrackDirection::North,
                };
            }
            TrackCommand::Height(delta) => {
                view.track_editor.height = (i16::from(view.track_editor.height) + i16::from(delta))
                    .clamp(0, i16::from(park_sim::MAX_TERRAIN_HEIGHT - 2))
                    as u8;
            }
            TrackCommand::Erase => view.track_editor.erase = true,
            TrackCommand::Clear => {
                sim.0.clear_track_draft();
                view.track_editor.revision = view.track_editor.revision.wrapping_add(1);
                view.track_editor.notice =
                    "Draft cleared. Start with a station.\nThe live coaster is unchanged.".into();
            }
            TrackCommand::Apply => {
                match sim.0.apply_track_draft() {
                    Ok(()) => {
                        view.scene_dirty = true;
                        view.selected = None;
                        view.hovered = None;
                        view.track_editor.revision = view.track_editor.revision.wrapping_add(1);
                        view.track_editor.notice = "Track applied; coaster stays closed.\nReconnect gates, then open with O.".into();
                    }
                    Err(error) => {
                        view.track_editor.notice =
                            format!("Apply refused: {}", error_message(&sim.0, error))
                    }
                }
            }
            TrackCommand::Done => unreachable!(),
        }
    }
}

pub(crate) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    view.track_editor.preview = None;
    let editor = &mut view.track_editor;
    if editor.erase {
        let Some(spec) = world::pick_track_draft(point, park, view.rotation, view.view_mode) else {
            return;
        };
        editor.preview = Some(TrackPreview {
            spec,
            valid: true,
            erase: true,
        });
        if click && park.remove_track_draft_piece(spec.pos) {
            editor.revision = editor.revision.wrapping_add(1);
            editor.preview = None;
            editor.notice = format!(
                "Removed draft tile {}, {}.\nLive track is unchanged.",
                spec.pos.x + 1,
                spec.pos.y + 1
            );
        }
        return;
    }
    let Some(pos) = world::pick_track_plane(point, park, view.rotation, editor.height) else {
        return;
    };
    let spec = editor.kind.spec(pos, editor.height, editor.direction);
    let valid = park.validate_track_draft_piece(spec).is_ok();
    editor.preview = Some(TrackPreview {
        spec,
        valid,
        erase: false,
    });
    if click {
        match park.set_track_draft_piece(spec) {
            Ok(()) => {
                editor.revision = editor.revision.wrapping_add(1);
                editor.notice = format!(
                    "{} at {}, {} / height {}.\nJoin arrows into one closed loop.",
                    kind_name(editor.kind),
                    pos.x + 1,
                    pos.y + 1,
                    editor.height
                );
            }
            Err(error) => editor.notice = error_message(park, error),
        }
    }
}

pub(crate) fn status(park: &Park) -> String {
    let count = park.track_draft().len();
    match park.validate_track_draft() {
        Ok(_) => format!(
            "{count} pieces / complete loop\nClose coaster; wait until empty.\nTrack building is free."
        ),
        Err(error) => format!("{count} draft pieces\n{}", error_message(park, error)),
    }
}

pub(crate) fn error_message(park: &Park, error: park_sim::TrackConstructionError) -> String {
    use park_sim::{TrackConstructionError as Error, TrainError};
    match error {
        Error::IncompleteLoop(pos) => {
            let piece = park.track_draft().iter().find(|piece| piece.pos == pos);
            piece.map_or_else(
                || "Join every piece into one loop.".into(),
                |piece| {
                    format!(
                        "Tile {}, {}: {} / height {}.\nMatch next entry arrow.",
                        pos.x + 1,
                        pos.y + 1,
                        direction_name(piece.exit.edge),
                        piece.exit.height
                    )
                },
            )
        }
        Error::Track(TrainError::StationCount { count: 0 }) => {
            "Add one station, then a closed loop.".into()
        }
        Error::Track(TrainError::StationCount { .. }) => {
            "Keep exactly one station in the loop.".into()
        }
        Error::DisconnectedPieces => {
            "Join all pieces into one loop.\nErase disconnected pieces.".into()
        }
        error => error.to_string(),
    }
}

/// Stable world tile used by ghost/pointer tests and capture fixtures.
#[cfg(test)]
pub(crate) fn build_center(park: &Park, pos: park_sim::TilePos, height: u8, rotation: u8) -> Vec2 {
    world::tile_center(pos, park.width(), rotation)
        + Vec2::Y * (f32::from(height) + 0.3) * world::HEIGHT_STEP
}
