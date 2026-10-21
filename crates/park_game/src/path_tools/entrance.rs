//! Entrance construction uses the same deliberate-click and visibility rules as paths.
use super::*;

pub(super) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    let height = view.path_tool.height;
    let facing = view.path_tool.entrance_facing;
    let Some(pos) =
        world::pick_sloped_path_plane(point, park, view.rotation, height, PathSlope::Flat)
    else {
        view.path_tool.hint = "Outside the park at this height.".into();
        if click {
            view.path_notice = Some("No entrance placed: outside the park.".into());
        }
        return;
    };
    let owner = park.ride().map(|ride| ride.id);
    let validation = owner
        .ok_or_else(|| "An entrance needs an existing ride.".to_string())
        .and_then(|ride| {
            park.validate_ride_entrance(pos, height, facing, ride)
                .map_err(|error| ride_placement_message(error, PathSlope::Flat))
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
            valid: reason.is_none(),
            demolish: false,
        });
    }
    view.path_tool.hint = reason.clone().unwrap_or_else(|| {
        format!(
            "Gate at {}, {} / height {height}\nClick to place the ride entrance.",
            pos.x + 1,
            pos.y + 1
        )
    });
    if !click {
        return;
    }
    if let Some(reason) = reason {
        view.path_notice = Some(format!("No entrance placed: {reason}"));
        return;
    }
    match park.place_ride_entrance(pos, height, facing, owner.expect("validated ride")) {
        Ok(id) => {
            view.selected = Some(SurfaceId::Object(id));
            view.hovered = view.selected;
            view.scene_dirty = true;
            view.path_tool.clear_pointer();
            view.path_notice = Some(format!(
                "Entrance #{} placed. Connect its outer edge to a queue.",
                id.0
            ));
        }
        Err(error) => {
            view.path_notice = Some(format!(
                "No entrance placed: {}",
                ride_placement_message(error, PathSlope::Flat)
            ))
        }
    }
}
