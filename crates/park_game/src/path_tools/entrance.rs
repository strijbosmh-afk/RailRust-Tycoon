//! Ride gates use the same deliberate-click and visibility rules as paths.
use super::*;

pub(super) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    let height = view.path_tool.height;
    let exit = view.path_tool.mode == PathTool::Exit;
    let name = if exit { "exit" } else { "entrance" };
    let facing = if exit {
        view.path_tool.exit_facing
    } else {
        view.path_tool.entrance_facing
    };
    let Some(pos) =
        world::pick_sloped_path_plane(point, park, view.rotation, height, PathSlope::Flat)
    else {
        view.path_tool.hint = "Outside the park at this height.".into();
        if click {
            view.path_notice = Some(format!("No {name} placed: outside the park."));
        }
        return;
    };
    let owner = crate::ui::rides::selected_id(park, view);
    let validation = owner
        .ok_or_else(|| "A gate needs an existing ride.".to_string())
        .and_then(|ride| {
            let result = if exit {
                park.validate_ride_exit(pos, height, facing, ride)
            } else {
                park.validate_ride_entrance(pos, height, facing, ride)
            };
            result.map_err(|error| ride_placement_message(error, PathSlope::Flat))
        });
    let occluded = world::sloped_path_plane_occluded(
        point,
        park,
        view.rotation,
        height,
        PathSlope::Flat,
        view.view_mode,
    );
    let reason = validation.err().or_else(|| {
        occluded.then(|| "Surface hidden here.\nRotate or use underground view.".into())
    });
    if !occluded {
        view.path_tool.preview = Some(PathPreview {
            pos,
            height,
            slope: PathSlope::Flat,
            queue_owner: owner,
            entrance_facing: Some(facing),
            exit_gate: exit,
            valid: reason.is_none(),
            demolish: false,
        });
    }
    view.path_tool.hint = reason.clone().unwrap_or_else(|| {
        format!(
            "Tile {}, {} / height {height}\nClick to place the ride {name}.",
            pos.x + 1,
            pos.y + 1
        )
    });
    if !click {
        return;
    }
    if let Some(reason) = reason {
        view.path_notice = Some(format!("No {name} placed: {reason}"));
        return;
    }
    let result = if exit {
        park.place_ride_exit(pos, height, facing, owner.expect("validated ride"))
    } else {
        park.place_ride_entrance(pos, height, facing, owner.expect("validated ride"))
    };
    match result {
        Ok(id) => {
            view.selected = Some(SurfaceId::Object(id));
            view.hovered = view.selected;
            view.scene_dirty = true;
            view.path_tool.clear_pointer();
            let name = if exit { "Exit" } else { "Entrance" };
            let connection = if exit { "public path" } else { "a queue" };
            view.path_notice = Some(format!(
                "{name} #{} placed. Connect its outer edge to {connection}.",
                id.0
            ));
        }
        Err(error) => {
            view.path_notice = Some(format!(
                "No {name} placed: {}",
                ride_placement_message(error, PathSlope::Flat)
            ))
        }
    }
}
