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
            notice: "Drafting and Clear are free.\nApply buys / 50% paid refund.".into(),
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
        TrackBuildKind::Lift => "Chain lift",
        TrackBuildKind::Drop => "Drop -1",
        TrackBuildKind::SteepDrop => "Drop -2",
        TrackBuildKind::Brake => "Brakes",
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
    view.track_editor.notice = "Drafting and Clear are free.\nApply buys / 50% paid refund.".into();
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
                view.track_editor.height = view
                    .track_editor
                    .height
                    .clamp(kind.min_height(), kind.max_height());
                view.track_editor.notice = format!(
                    "{} / new {}\nPgUp/Dn sets entry height.",
                    kind_name(kind),
                    crate::ui::money(kind.cost_cents())
                );
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
                    .clamp(
                        i16::from(view.track_editor.kind.min_height()),
                        i16::from(view.track_editor.kind.max_height()),
                    ) as u8;
            }
            TrackCommand::Erase => view.track_editor.erase = true,
            TrackCommand::Clear => {
                sim.0.clear_track_draft();
                view.track_editor.revision = view.track_editor.revision.wrapping_add(1);
                view.track_editor.notice =
                    "Draft cleared / no charge.\nLive coaster stays unchanged.".into();
            }
            TrackCommand::Apply => {
                let quote = sim.0.track_construction_quote().ok();
                match sim.0.apply_track_draft() {
                    Ok(()) => {
                        view.scene_dirty = true;
                        view.selected = None;
                        view.hovered = None;
                        view.track_editor.revision = view.track_editor.revision.wrapping_add(1);
                        let payment = quote.map_or_else(
                            || "Track applied".into(),
                            |quote| {
                                if quote.refund_cents > quote.construction_cents {
                                    format!(
                                        "Applied / received {}",
                                        crate::ui::money(
                                            quote.refund_cents - quote.construction_cents
                                        )
                                    )
                                } else {
                                    format!(
                                        "Applied / paid {}",
                                        crate::ui::money(quote.required_cents())
                                    )
                                }
                            },
                        );
                        view.track_editor.notice =
                            format!("{payment}\nCoaster closed; connect gates.");
                    }
                    Err(error) => view.track_editor.notice = apply_error_message(error),
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
    // Anchor the tile grid at the piece's midpoint height. Entry-height picking
    // shifts a two-step drop's visible center onto the next diagonal tile.
    let ports = editor.kind.spec(
        park_sim::TilePos { x: 0, y: 0 },
        editor.height,
        editor.direction,
    );
    let plane_shift =
        (f32::from(ports.entry.height) - f32::from(ports.exit.height)) * 0.5 * world::HEIGHT_STEP;
    let Some(pos) = world::pick_track_plane(
        point + Vec2::Y * plane_shift,
        park,
        view.rotation,
        editor.height,
    ) else {
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
                    "{} / {}, {} / {}-{}\nMatch next entry to this exit.",
                    kind_name(editor.kind),
                    pos.x + 1,
                    pos.y + 1,
                    spec.entry.height,
                    spec.exit.height
                );
            }
            Err(error) => editor.notice = error_message(park, error),
        }
    }
}

pub(crate) fn apply_error_message(error: park_sim::TrackConstructionError) -> String {
    use park_sim::TrackConstructionError as Error;
    match error {
        Error::CloseRide => "Apply refused: train active.\nClose it; wait for train stop.".into(),
        Error::GuestsStillVisiting => {
            "Apply refused: active guests.\nWait for everyone to leave.".into()
        }
        Error::InsufficientFunds {
            required_cents,
            available_cents,
        } => format!(
            "Apply refused: short {}\nCash {}. Draft kept.",
            crate::ui::money(required_cents.saturating_sub(available_cents)),
            crate::ui::money(available_cents)
        ),
        _ => "Apply refused: invalid draft.\nCheck joins and clearance.".into(),
    }
}

pub(crate) fn status(park: &Park) -> String {
    let count = park.track_draft().len();
    match park.validate_track_draft() {
        Ok(_) => format!("{count} pieces / complete loop\nClose and empty before Apply."),
        Err(error) => error_message(park, error),
    }
}

/// The price uses current cash and live paid pieces, including faulted remnants.
/// Calling this read-only quote never buys, sells or mutates the draft.
pub(crate) fn quote_text(park: &Park) -> String {
    use crate::ui::money;
    match park.track_construction_quote() {
        Ok(quote) => {
            let net = if quote.refund_cents > quote.construction_cents {
                format!(
                    "Receive {}",
                    money(quote.refund_cents - quote.construction_cents)
                )
            } else {
                format!("Pay {}", money(quote.required_cents()))
            };
            let balance = quote.balance_after().map_or_else(
                || {
                    format!(
                        "Short {} / cannot apply",
                        money(quote.required_cents().saturating_sub(quote.available_cents))
                    )
                },
                |after| format!("After {}", money(after)),
            );
            format!(
                "Buy {} / Refund {}\n{net} / Cash {}\n{balance}",
                money(quote.construction_cents),
                money(quote.refund_cents),
                money(quote.available_cents)
            )
        }
        Err(_) => format!(
            "Complete a loop for its quote.\nDraft changes are free.\nCash {}",
            money(park.cash_cents())
        ),
    }
}

pub(crate) fn error_message(park: &Park, error: park_sim::TrackConstructionError) -> String {
    use park_sim::{TrackConstructionError as Error, TrainError};
    match error {
        Error::InsufficientFunds {
            required_cents,
            available_cents,
        } => format!(
            "short {}\nCash {}. Draft kept.",
            crate::ui::money(required_cents.saturating_sub(available_cents)),
            crate::ui::money(available_cents)
        ),
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
        Error::Track(TrainError::EmptyCircuit | TrainError::StationCount { count: 0 }) => {
            "Add one station.\nJoin track into a closed loop.".into()
        }
        Error::Track(TrainError::StationCount { .. }) => "Keep one station in the loop.".into(),
        Error::DisconnectedPieces => {
            "Join all pieces into one loop.\nErase disconnected pieces.".into()
        }
        Error::UnsupportedPiece => "Check piece height limits.\nLift +1 / drops -1 or -2.".into(),
        Error::Track(TrainError::Placement(park_sim::PlacementError::Occupied { pos, .. })) => {
            format!(
                "Tile {}, {} is occupied.\nChange route or rail height.",
                pos.x + 1,
                pos.y + 1
            )
        }
        Error::Track(TrainError::Placement(park_sim::PlacementError::TerrainIntersection {
            pos,
            ..
        })) => format!(
            "Tile {}, {} crosses land.\nChange land or rail height.",
            pos.x + 1,
            pos.y + 1
        ),
        Error::Track(TrainError::Placement(park_sim::PlacementError::HeightOutOfRange {
            ..
        })) => "Track exceeds height limit.\nLower this piece's entry.".into(),
        Error::Track(TrainError::Placement(park_sim::PlacementError::OutOfBounds { .. })) => {
            "Track must stay in the park.".into()
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

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::{TRACK_UNITS, TilePos};

    #[test]
    fn actual_sampled_midpoints_build_and_replace_the_same_tile_for_all_grades_and_rotations() {
        for rotation in 0..4 {
            for kind in [
                TrackBuildKind::Station,
                TrackBuildKind::Straight,
                TrackBuildKind::CurveLeft,
                TrackBuildKind::CurveRight,
                TrackBuildKind::Lift,
                TrackBuildKind::Drop,
                TrackBuildKind::SteepDrop,
                TrackBuildKind::Brake,
            ] {
                for direction in [
                    TrackDirection::North,
                    TrackDirection::East,
                    TrackDirection::South,
                    TrackDirection::West,
                ] {
                    let mut park = Park::new(32, 32);
                    let expected = kind.spec(TilePos { x: 14, y: 17 }, 8, direction);
                    let samples = expected.samples();
                    let midpoint = samples[samples.len() / 2];
                    let x = midpoint.x as f32 / TRACK_UNITS as f32;
                    let y = midpoint.y as f32 / TRACK_UNITS as f32;
                    let height = midpoint.height as f32 / TRACK_UNITS as f32;
                    let last = park.width().saturating_sub(1) as f32;
                    let (u, v) = match rotation {
                        0 => (x, y),
                        1 => (last - y, x),
                        2 => (last - x, last - y),
                        _ => (y, last - x),
                    };
                    let point = Vec2::new(
                        (u - v) * world::TILE_WIDTH * 0.5,
                        -(u + v - last) * world::TILE_HEIGHT * 0.5
                            + (height + 0.3) * world::HEIGHT_STEP,
                    );
                    let mut view = ViewState {
                        rotation,
                        track_editor: TrackEditor {
                            active: true,
                            kind,
                            direction,
                            height: 8,
                            ..default()
                        },
                        ..default()
                    };
                    let before = park.clone();
                    point_at(&mut park, &mut view, point, false);
                    assert_eq!(park, before);
                    assert_eq!(
                        view.track_editor.preview,
                        Some(TrackPreview {
                            spec: expected,
                            valid: true,
                            erase: false,
                        }),
                        "{kind:?}, {direction:?}, rotation {rotation}"
                    );
                    point_at(&mut park, &mut view, point, true);
                    assert_eq!(park.track_draft(), &[expected]);
                    point_at(&mut park, &mut view, point, true);
                    assert_eq!(park.track_draft(), &[expected]);
                }
            }
        }
    }
}
