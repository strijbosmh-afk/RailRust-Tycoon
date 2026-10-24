//! Read-only ride access assistance. Only the existing map tools build objects.
use bevy::{prelude::*, window::PrimaryWindow};
use park_sim::{Park, RideAccessBuildKind, RideAccessSuggestion, RideId, SurfaceId};

use crate::{
    SaveAction, Simulation, ViewState,
    path_tools::{self, PathTool},
    ui, world,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccessCommand {
    ShowOperations,
    ShowAccess,
    FindEntrance,
    FindExit,
    Prepare(RideAccessBuildKind),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PreparedAccess {
    ride: RideId,
    suggestion: RideAccessSuggestion,
    cursor: Option<Vec2>,
    point: Vec2,
    rotation: u8,
    view_mode: world::ViewMode,
}

pub(crate) fn queue(view: &mut ViewState, command: AccessCommand) {
    if !view.ride_access_commands.contains(&command) {
        view.ride_access_commands.push(command);
    }
}

fn blocked(view: &ViewState) -> bool {
    !view.rides_open
        || view.help
        || view.guests_open
        || view.services_open
        || view.finance_open
        || view.designs_open
        || view.capture_active
        || view.diagnostic_case.is_some()
        || view.modal_input_blocked
        || view.modal_transition_pending
        || view
            .save_pending
            .is_some_and(|action| action != SaveAction::Save)
}

pub(crate) fn command_enabled(park: &Park, view: &ViewState, command: AccessCommand) -> bool {
    if blocked(view) || view.ride_browse_pending != 0 {
        return false;
    }
    if matches!(
        command,
        AccessCommand::ShowOperations | AccessCommand::ShowAccess
    ) {
        return true;
    }
    if view.ride_tab != ui::rides::RideTab::Access {
        return false;
    }
    let Some(id) = ui::rides::selected_id(park, view) else {
        return false;
    };
    match command {
        AccessCommand::FindEntrance => park
            .ride_by_id(id)
            .is_some_and(|ride| ride.entrance.is_some()),
        AccessCommand::FindExit => park.ride_by_id(id).is_some_and(|ride| ride.exit.is_some()),
        AccessCommand::Prepare(kind) => view.ride_access_plan.as_ref().is_some_and(|plan| {
            plan.ride == id
                && plan.available
                && plan
                    .suggestions
                    .iter()
                    .any(|suggestion| suggestion.kind == kind)
        }),
        _ => true,
    }
}

pub(crate) fn apply_commands(
    mut sim: ResMut<Simulation>,
    mut view: ResMut<ViewState>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let commands = std::mem::take(&mut view.ride_access_commands);
    if commands.is_empty()
        || blocked(&view)
        || view.ride_browse_pending != 0
        || keys
            .as_ref()
            .is_some_and(|keys| keys.just_pressed(KeyCode::F9))
    {
        return;
    }
    if let Some(command) = commands.iter().find(|command| {
        matches!(
            command,
            AccessCommand::ShowAccess | AccessCommand::ShowOperations
        )
    }) {
        view.ride_tab = if *command == AccessCommand::ShowAccess {
            ui::rides::RideTab::Access
        } else {
            ui::rides::RideTab::Operations
        };
        view.coaster_testing.run_pending = false;
        view.ride_access_plan = None;
        view.ride_operating_commands.clear();
        view.ride_access_notice = None;
        return;
    }
    if view.ride_tab != ui::rides::RideTab::Access {
        return;
    }
    let Some(id) = ui::rides::selected_id(&sim.0, &view) else {
        view.ride_access_notice = Some("Choose an available ride first.".into());
        return;
    };
    // A single interaction prepares one tool, even if conflicting commands
    // arrive together. Preparation closes the panel and consumes that frame.
    if let Some(&command) = commands.first() {
        perform_command(&mut sim.bypass_change_detection().0, &mut view, id, command);
        if let Some(prepared) = &mut view.prepared_access {
            prepared.cursor = windows.single().ok().and_then(Window::cursor_position);
        }
    }
}

fn focus(park: &Park, view: &mut ViewState, point: Vec2) {
    view.pan = point - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

fn tool(kind: RideAccessBuildKind) -> PathTool {
    match kind {
        RideAccessBuildKind::Entrance => PathTool::Entrance,
        RideAccessBuildKind::Exit => PathTool::Exit,
        RideAccessBuildKind::Queue => PathTool::BuildQueue,
        RideAccessBuildKind::PublicPath => PathTool::Build,
    }
}

pub(crate) fn suggestion_point(
    park: &Park,
    view: &ViewState,
    suggestion: RideAccessSuggestion,
) -> Option<Vec2> {
    visible_suggestion_point(park, view.rotation, view.view_mode, suggestion)
}

fn visible_suggestion_point(
    park: &Park,
    rotation: u8,
    mode: world::ViewMode,
    suggestion: RideAccessSuggestion,
) -> Option<Vec2> {
    let center = world::path_build_center(
        suggestion.pos,
        park,
        rotation,
        suggestion.height,
        suggestion.slope,
    )?;
    // A rail, gate post or cliff may cover the middle of a legal path tile.
    // Keep using the ordinary inverse projection and visibility rules, but
    // consider its exposed edges before declaring the whole tile hidden.
    if suggestion_point_visible(park, rotation, mode, suggestion, center) {
        return Some(center);
    }
    let mut offsets: Vec<_> = (-24i32..=24)
        .step_by(4)
        .flat_map(|y| (-28i32..=28).step_by(4).map(move |x| (x, y)))
        .filter(|&(x, y)| x != 0 || y != 0)
        .collect();
    offsets.sort_by_key(|&(x, y)| (x * x + y * y, y, x));
    offsets
        .into_iter()
        .map(|(x, y)| center + Vec2::new(x as f32, y as f32))
        .find(|point| suggestion_point_visible(park, rotation, mode, suggestion, *point))
}

fn suggestion_point_visible(
    park: &Park,
    rotation: u8,
    mode: world::ViewMode,
    suggestion: RideAccessSuggestion,
    point: Vec2,
) -> bool {
    world::pick_sloped_path_plane(point, park, rotation, suggestion.height, suggestion.slope)
        == Some(suggestion.pos)
        && !world::sloped_path_plane_occluded(
            point,
            park,
            rotation,
            suggestion.height,
            suggestion.slope,
            mode,
        )
}

pub(crate) fn perform_command(
    park: &mut Park,
    view: &mut ViewState,
    id: RideId,
    command: AccessCommand,
) -> bool {
    if matches!(
        command,
        AccessCommand::ShowAccess | AccessCommand::ShowOperations
    ) {
        view.ride_tab = if command == AccessCommand::ShowAccess {
            ui::rides::RideTab::Access
        } else {
            ui::rides::RideTab::Operations
        };
        view.coaster_testing.run_pending = false;
        view.ride_access_plan = None;
        view.ride_operating_commands.clear();
        return true;
    }
    let Some(ride) = park.ride_by_id(id) else {
        view.ride_access_notice = Some("This ride is no longer available.".into());
        return false;
    };
    if matches!(
        command,
        AccessCommand::FindEntrance | AccessCommand::FindExit
    ) {
        let gate = if command == AccessCommand::FindExit {
            ride.exit
        } else {
            ride.entrance
        };
        let Some(object) = gate.and_then(|gate| park.object(gate.object)) else {
            view.ride_access_notice = Some("That gate is no longer available.".into());
            return false;
        };
        let Some(point) = world::path_build_center(
            object.pos,
            park,
            view.rotation,
            object.height,
            park_sim::PathSlope::Flat,
        ) else {
            return false;
        };
        let selected = object.id;
        view.set_rides_modal(false);
        view.modal_transition_pending = true;
        view.select_ride(id);
        view.track_editor.active = false;
        view.path_tool.set_mode(PathTool::Inspect);
        view.selected = Some(SurfaceId::Object(selected));
        view.path_notice = Some("Gate selected. Open Rides [Y] for the access checklist.".into());
        focus(park, view, point);
        return true;
    }
    let AccessCommand::Prepare(kind) = command else {
        return false;
    };
    let plan = match park.ride_access_plan(id) {
        Ok(plan) => plan,
        Err(error) => {
            view.ride_access_notice = Some(error.to_string());
            return false;
        }
    };
    let Some(suggestion) = plan
        .suggestions
        .iter()
        .find(|suggestion| suggestion.kind == kind)
        .copied()
        .filter(|_| plan.available)
    else {
        view.ride_access_notice = Some(plan.advice.into());
        return false;
    };
    let visible = suggestion_point(park, view, suggestion)
        .map(|point| (point, view.view_mode))
        .or_else(|| {
            visible_suggestion_point(
                park,
                view.rotation,
                world::ViewMode::Underground,
                suggestion,
            )
            .map(|point| (point, world::ViewMode::Underground))
        });
    let Some((point, mode)) = visible else {
        view.ride_access_notice = Some(
            "Suggested tile is hidden by another object. Rotate the view, then prepare again."
                .into(),
        );
        return false;
    };
    view.set_rides_modal(false);
    view.modal_transition_pending = true;
    view.select_ride(id);
    view.track_editor.active = false;
    view.track_editor.pending.clear();
    view.path_tool.set_mode(tool(kind));
    view.path_tool.height = suggestion.height;
    view.path_tool.slope = suggestion.slope;
    view.path_tool.entrance_facing = suggestion.facing;
    view.path_tool.exit_facing = suggestion.facing;
    view.selected = None;
    view.prepared_access = Some(PreparedAccess {
        ride: id,
        suggestion,
        cursor: None,
        point,
        rotation: view.rotation,
        view_mode: mode,
    });
    view.view_mode = mode;
    path_tools::point_at(park, view, point, false);
    view.path_notice = Some(format!(
        "Tile {}, {} / h{}. Click a ghost edge clear of track. Y: Rides.",
        suggestion.pos.x + 1,
        suggestion.pos.y + 1,
        suggestion.height
    ));
    focus(park, view, point);
    true
}

/// Keep the suggested ghost visible after the panel closes. It never turns a
/// closing UI click, or a click at the old cursor position, into construction.
pub(crate) fn hold_prepared_preview(
    park: &mut Park,
    view: &mut ViewState,
    cursor: Vec2,
    click: bool,
    panning: bool,
) -> bool {
    let Some(mut prepared) = view.prepared_access else {
        return false;
    };
    let suggestion = prepared.suggestion;
    if view.help
        || view.guests_open
        || view.services_open
        || view.finance_open
        || view.rides_open
        || view.designs_open
        || view.capture_active
        || view.diagnostic_case.is_some()
        || panning
        || view.track_editor.active
        || view
            .save_pending
            .is_some_and(|action| action != SaveAction::Save)
        || ui::rides::selected_id(park, view) != Some(prepared.ride)
        || view.path_tool.mode != tool(suggestion.kind)
        || view.path_tool.height != suggestion.height
        || view.path_tool.slope != suggestion.slope
        || (suggestion.kind == RideAccessBuildKind::Entrance
            && view.path_tool.entrance_facing != suggestion.facing)
        || (suggestion.kind == RideAccessBuildKind::Exit
            && view.path_tool.exit_facing != suggestion.facing)
    {
        view.prepared_access = None;
        return false;
    }
    if prepared
        .cursor
        .is_some_and(|initial| initial.distance_squared(cursor) > 1.0)
    {
        view.prepared_access = None;
        return false;
    }
    prepared.cursor = Some(cursor);
    let point = if prepared.rotation == view.rotation
        && prepared.view_mode == view.view_mode
        && suggestion_point_visible(
            park,
            view.rotation,
            view.view_mode,
            suggestion,
            prepared.point,
        ) {
        Some(prepared.point)
    } else {
        suggestion_point(park, view, suggestion)
    };
    if let Some(point) = point {
        prepared.point = point;
        prepared.rotation = view.rotation;
        prepared.view_mode = view.view_mode;
        path_tools::point_at(park, view, point, false);
    } else {
        view.path_tool.preview = None;
        view.path_tool.hint = "Suggested tile hidden.\nRotate the view to expose it.".into();
        view.path_notice = Some("Suggestion hidden. Rotate the view and prepare it again.".into());
        view.prepared_access = None;
        view.hovered = None;
        return true;
    }
    view.prepared_access = Some(prepared);
    if click && !view.ui_pointer_consumed {
        view.path_notice = Some("Move to the suggested tile, then click to build.".into());
    }
    view.hovered = None;
    true
}

#[cfg(test)]
#[path = "access_planning_input_tests.rs"]
mod tests;
