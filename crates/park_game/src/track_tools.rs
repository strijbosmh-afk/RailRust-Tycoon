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
    CopyInstalled,
    Undo,
    Redo,
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
    pub(crate) history: DraftHistory,
}

const HISTORY_LIMIT: usize = 64;

/// Session-only draft snapshots. Never store the live park or its finances here.
#[derive(Clone, Debug, Default)]
pub(crate) struct DraftHistory {
    undo: Vec<Vec<TrackPieceSpec>>,
    redo: Vec<Vec<TrackPieceSpec>>,
    current: Option<Vec<TrackPieceSpec>>,
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
            history: DraftHistory::default(),
        }
    }
}

impl TrackEditor {
    pub(crate) fn can_undo(&self) -> bool {
        self.undo_len() != 0
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.redo_len() != 0
    }

    pub(crate) fn undo_len(&self) -> usize {
        self.history.undo.len()
    }

    pub(crate) fn redo_len(&self) -> usize {
        self.history.redo.len()
    }

    fn reset_history(&mut self, draft: &[TrackPieceSpec]) {
        self.history.undo.clear();
        self.history.redo.clear();
        self.history.current = Some(draft.to_vec());
    }

    /// Fixtures and other callers can replace the draft outside this editor.
    /// Old snapshots must not undo an unrelated externally supplied draft.
    fn synchronize_history(&mut self, draft: &[TrackPieceSpec]) {
        if self.history.current.as_deref() != Some(draft) {
            let previously_observed = self.history.current.is_some();
            self.reset_history(draft);
            if previously_observed {
                self.draft_changed();
            }
        }
    }

    fn record_change(&mut self, before: Vec<TrackPieceSpec>, after: &[TrackPieceSpec]) -> bool {
        self.synchronize_history(&before);
        if before == after {
            return false;
        }
        if self.history.undo.len() == HISTORY_LIMIT {
            self.history.undo.remove(0);
        }
        self.history.undo.push(before);
        self.history.redo.clear();
        self.history.current = Some(after.to_vec());
        self.draft_changed();
        true
    }

    fn draft_changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.preview = None;
    }

    fn restore_history(
        &mut self,
        park: &mut Park,
        undo: bool,
    ) -> Result<bool, park_sim::TrackConstructionError> {
        self.synchronize_history(park.track_draft());
        let next = if undo {
            self.history.undo.last()
        } else {
            self.history.redo.last()
        };
        let Some(next) = next.cloned() else {
            return Ok(false);
        };
        let before = park.track_draft().to_vec();
        // Leave both stacks intact if restoration is refused.
        if !park.replace_track_draft(&next)? {
            return Ok(false);
        }
        if undo {
            self.history.undo.pop();
            self.history.redo.push(before);
        } else {
            self.history.redo.pop();
            self.history.undo.push(before);
        }
        self.history.current = Some(next);
        self.draft_changed();
        Ok(true)
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
    view.track_editor.synchronize_history(park.track_draft());
    view.track_editor.revision = view.track_editor.revision.wrapping_add(1);
    view.track_editor.preview = None;
    view.track_editor.notice = "Drafting and Clear are free.\nApply buys / 50% paid refund.".into();
}

pub(crate) fn queue(view: &mut ViewState, command: TrackCommand) {
    if view.track_editor.active && !view.edits_blocked() {
        // A shortcut and its button can arrive in one frame. Treat that as
        // one history step while preserving the order of different commands.
        if matches!(command, TrackCommand::Undo | TrackCommand::Redo)
            && view.track_editor.pending.contains(&command)
        {
            view.ui_pointer_consumed = true;
            return;
        }
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
        apply_command(&mut sim.0, &mut view, command);
    }
}

/// The same draft-only command path is used by input, capture and replay checks.
pub(crate) fn apply_command(park: &mut Park, view: &mut ViewState, command: TrackCommand) {
    view.track_editor.synchronize_history(park.track_draft());
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
            let before = park.track_draft().to_vec();
            park.clear_track_draft();
            view.track_editor.record_change(before, park.track_draft());
            view.track_editor.notice =
                "Draft cleared / no charge.\nLive coaster stays unchanged.".into();
        }
        TrackCommand::CopyInstalled => {
            let before = park.track_draft().to_vec();
            match park.copy_installed_track_to_draft() {
                Ok(changed) => {
                    view.track_editor.record_change(before, park.track_draft());
                    view.track_editor.notice = if changed {
                        format!(
                            "{} installed pieces copied.\nEdit freely; Apply buys changes.",
                            park.track_draft().len()
                        )
                    } else {
                        "Draft matches installed track.\nLive track is unchanged.".into()
                    };
                }
                Err(_) => {
                    view.track_editor.notice =
                        "No complete installed circuit.\nBuild a loop before copying.".into();
                }
            }
        }
        TrackCommand::Undo | TrackCommand::Redo => {
            let undo = command == TrackCommand::Undo;
            view.track_editor.notice = match view.track_editor.restore_history(park, undo) {
                Ok(true) => format!(
                    "Draft edit {}.\nLive track and cash unchanged.",
                    if undo { "undone" } else { "restored" }
                ),
                Ok(false) => format!(
                    "Nothing to {} in this draft.\nLive track is unchanged.",
                    if undo { "undo" } else { "redo" }
                ),
                Err(_) => "Draft restore refused.\nHistory and live track kept.".into(),
            };
        }
        TrackCommand::Apply => {
            let quote = park.track_construction_quote().ok();
            match park.apply_track_draft() {
                Ok(()) => {
                    view.track_editor.reset_history(park.track_draft());
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
                                    crate::ui::money(quote.refund_cents - quote.construction_cents)
                                )
                            } else {
                                format!(
                                    "Applied / paid {}",
                                    crate::ui::money(quote.required_cents())
                                )
                            }
                        },
                    );
                    view.track_editor.notice = format!("{payment}\nCoaster closed; connect gates.");
                }
                Err(error) => view.track_editor.notice = apply_error_message(error),
            }
        }
        TrackCommand::Done => {
            view.track_editor.active = false;
            view.track_editor.preview = None;
            view.ui_pointer_consumed = true;
        }
    }
}

/// Library loads use the same bounded, draft-only history as pointer edits.
pub(crate) fn load_design_draft(
    park: &mut Park,
    view: &mut ViewState,
    specs: &[TrackPieceSpec],
) -> Result<bool, park_sim::TrackConstructionError> {
    let before = park.track_draft().to_vec();
    let changed = park.replace_track_draft(specs)?;
    view.track_editor.record_change(before, park.track_draft());
    view.track_editor.preview = None;
    Ok(changed)
}

pub(crate) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    view.track_editor.preview = None;
    let editor = &mut view.track_editor;
    editor.synchronize_history(park.track_draft());
    if editor.erase {
        let Some(spec) = world::pick_track_draft(point, park, view.rotation, view.view_mode) else {
            return;
        };
        editor.preview = Some(TrackPreview {
            spec,
            valid: true,
            erase: true,
        });
        let before = click.then(|| park.track_draft().to_vec());
        if click && park.remove_track_draft_piece(spec.pos) {
            editor.record_change(before.unwrap(), park.track_draft());
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
        let before = park.track_draft().to_vec();
        match park.set_track_draft_piece(spec) {
            Ok(()) => {
                editor.record_change(before, park.track_draft());
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

/// Stable entry-height tile center for pointer checks and replay fixtures.
pub(crate) fn build_center(park: &Park, pos: park_sim::TilePos, height: u8, rotation: u8) -> Vec2 {
    world::tile_center(pos, park.width(), rotation)
        + Vec2::Y * (f32::from(height) + 0.3) * world::HEIGHT_STEP
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::{TRACK_UNITS, TilePos};

    fn editor_view() -> ViewState {
        ViewState {
            track_editor: TrackEditor {
                active: true,
                ..default()
            },
            ..default()
        }
    }

    fn place(park: &mut Park, view: &mut ViewState, pos: TilePos, kind: TrackBuildKind) {
        view.track_editor.kind = kind;
        view.track_editor.erase = false;
        let point = build_center(park, pos, view.track_editor.height, view.rotation);
        point_at(park, view, point, true);
    }

    fn history(view: &ViewState) -> (usize, usize) {
        (view.track_editor.undo_len(), view.track_editor.redo_len())
    }

    fn assert_only_draft_changed(before: &Park, after: &Park) {
        let mut expected = before.clone();
        expected.replace_track_draft(after.track_draft()).unwrap();
        assert_eq!(*after, expected);
        assert_eq!(after.path_cache_stats(), before.path_cache_stats());
    }

    #[test]
    fn history_records_actual_point_edits_and_keeps_redo_on_no_op_or_refused_edits() {
        let mut park = Park::new(32, 32);
        let initial = park.clone();
        let mut view = editor_view();
        let pos = TilePos { x: 14, y: 17 };
        place(&mut park, &mut view, pos, TrackBuildKind::Straight);
        let straight = park.track_draft().to_vec();
        place(&mut park, &mut view, pos, TrackBuildKind::Brake);
        assert_eq!(history(&view), (2, 0));
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        assert_eq!(park.track_draft(), straight);
        assert_eq!(history(&view), (1, 1));
        let revision = view.track_editor.revision;
        place(&mut park, &mut view, pos, TrackBuildKind::Straight);
        assert_eq!(history(&view), (1, 1));
        assert_eq!(view.track_editor.revision, revision);
        assert_only_draft_changed(&initial, &park);

        let blocked = TilePos { x: 18, y: 17 };
        park.place_path(blocked, 8).unwrap();
        let before_refusal = park.clone();
        place(&mut park, &mut view, blocked, TrackBuildKind::Straight);
        assert_eq!(park, before_refusal);
        assert_eq!(history(&view), (1, 1));
        assert_eq!(view.track_editor.revision, revision);
        apply_command(&mut park, &mut view, TrackCommand::CopyInstalled);
        assert_eq!(park, before_refusal);
        assert_eq!(history(&view), (1, 1));

        place(
            &mut park,
            &mut view,
            TilePos { x: 15, y: 17 },
            TrackBuildKind::Straight,
        );
        assert_eq!(history(&view), (2, 0));
        assert!(!view.track_editor.can_redo());
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        assert_eq!(park, before_refusal);
    }

    #[test]
    fn history_bounds_snapshots_and_restores_only_the_draft() {
        let mut park = Park::new(32, 32);
        let initial = park.clone();
        let mut view = editor_view();
        let pos = TilePos { x: 14, y: 17 };
        let mut earliest_retained = Vec::new();
        for edit in 1_u32..=70 {
            let kind = if edit.is_multiple_of(2) {
                TrackBuildKind::Brake
            } else {
                TrackBuildKind::Straight
            };
            place(&mut park, &mut view, pos, kind);
            if edit == 6 {
                earliest_retained = park.track_draft().to_vec();
            }
        }
        let latest = park.track_draft().to_vec();
        assert_eq!(history(&view), (HISTORY_LIMIT, 0));
        for _ in 0..HISTORY_LIMIT {
            assert!(view.track_editor.can_undo());
            apply_command(&mut park, &mut view, TrackCommand::Undo);
            assert_only_draft_changed(&initial, &park);
        }
        assert_eq!(park.track_draft(), earliest_retained);
        assert_eq!(history(&view), (0, HISTORY_LIMIT));
        let exhausted = park.clone();
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        assert_eq!(park, exhausted);
        assert_eq!(history(&view), (0, HISTORY_LIMIT));
        for _ in 0..HISTORY_LIMIT {
            apply_command(&mut park, &mut view, TrackCommand::Redo);
        }
        assert_eq!(park.track_draft(), latest);
        assert_eq!(history(&view), (HISTORY_LIMIT, 0));
        assert_only_draft_changed(&initial, &park);
    }

    #[test]
    fn copy_clear_and_erase_share_history_and_reopening_keeps_it() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let initial = park.clone();
        let mut view = editor_view();
        apply_command(&mut park, &mut view, TrackCommand::CopyInstalled);
        let copied = park.track_draft().to_vec();
        assert_eq!(copied, park.circuit().unwrap().specs());
        assert_eq!(history(&view), (1, 0));
        apply_command(&mut park, &mut view, TrackCommand::Clear);
        assert!(park.track_draft().is_empty());
        apply_command(&mut park, &mut view, TrackCommand::Clear);
        assert_eq!(history(&view), (2, 0));
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        apply_command(&mut park, &mut view, TrackCommand::CopyInstalled);
        assert_eq!(history(&view), (1, 1), "identical copy keeps redo");
        apply_command(&mut park, &mut view, TrackCommand::Done);
        begin(&park, &mut view);
        assert_eq!(history(&view), (1, 1));
        assert_eq!(park.track_draft(), copied);

        // Pick an actual sampled point of a copied level piece for erase.
        let piece = copied
            .iter()
            .find(|piece| piece.entry.height == piece.exit.height)
            .copied()
            .unwrap();
        let point = build_center(&park, piece.pos, piece.entry.height, view.rotation);
        view.track_editor.erase = true;
        point_at(&mut park, &mut view, point, true);
        assert_eq!(park.track_draft().len(), copied.len() - 1);
        assert_eq!(history(&view), (2, 0));
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        assert_eq!(park.track_draft(), copied);
        assert_only_draft_changed(&initial, &park);
    }

    #[test]
    fn outside_draft_replacement_discards_stale_undo_and_redo() {
        let mut park = Park::new(32, 32);
        let mut view = editor_view();
        let pos = TilePos { x: 14, y: 17 };
        place(&mut park, &mut view, pos, TrackBuildKind::Straight);
        place(&mut park, &mut view, pos, TrackBuildKind::Brake);
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        assert_eq!(history(&view), (1, 1));
        let external =
            TrackBuildKind::Station.spec(TilePos { x: 12, y: 13 }, 8, TrackDirection::South);
        park.replace_track_draft(&[external]).unwrap();
        let replaced = park.clone();
        let revision = view.track_editor.revision;
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        assert_eq!(park, replaced);
        assert_eq!(history(&view), (0, 0));
        assert_ne!(view.track_editor.revision, revision);
        apply_command(&mut park, &mut view, TrackCommand::Redo);
        assert_eq!(park, replaced);
    }

    #[test]
    fn refused_apply_keeps_history_and_successful_apply_is_a_history_boundary() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let mut view = editor_view();
        apply_command(&mut park, &mut view, TrackCommand::CopyInstalled);
        apply_command(&mut park, &mut view, TrackCommand::Clear);
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        let copied = park.clone();
        assert_eq!(history(&view), (1, 1));
        park.set_train_open(true);
        apply_command(&mut park, &mut view, TrackCommand::Apply);
        assert!(view.track_editor.notice.starts_with("Apply refused"));
        assert_eq!(history(&view), (1, 1));
        park.set_train_open(false);
        apply_command(&mut park, &mut view, TrackCommand::Apply);
        assert!(view.track_editor.notice.starts_with("Applied"));
        assert_eq!(history(&view), (0, 0));
        assert_eq!(park.track_draft(), copied.track_draft());
        let applied = park.clone();
        apply_command(&mut park, &mut view, TrackCommand::Undo);
        assert_eq!(park, applied, "undo cannot cross a successful Apply");
    }

    #[test]
    fn queue_deduplicates_same_history_step_but_keeps_distinct_command_order() {
        let mut view = editor_view();
        for command in [
            TrackCommand::Undo,
            TrackCommand::Undo,
            TrackCommand::Piece(TrackBuildKind::Brake),
            TrackCommand::Redo,
            TrackCommand::Redo,
        ] {
            queue(&mut view, command);
        }
        assert_eq!(
            view.track_editor.pending,
            [
                TrackCommand::Undo,
                TrackCommand::Piece(TrackBuildKind::Brake),
                TrackCommand::Redo,
            ]
        );
    }

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
