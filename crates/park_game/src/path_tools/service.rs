//! Service placement uses the ordinary height plane, visibility and edit order.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServicePreview {
    pub pos: TilePos,
    pub height: u8,
    pub kind: ServiceKind,
    pub facing: PathDirection,
    pub valid: bool,
    pub demolish: bool,
}

pub(super) fn point_at(
    park: &mut Park,
    view: &mut ViewState,
    point: Vec2,
    click: bool,
    kind: ServiceKind,
) {
    let height = view.path_tool.height;
    let facing = view.path_tool.service_facing;
    let Some(pos) =
        world::pick_sloped_path_plane(point, park, view.rotation, height, PathSlope::Flat)
    else {
        view.path_tool.hint = "Outside the park at this height.".into();
        if click {
            view.path_notice = Some("No facility placed: outside the park.".into());
        }
        return;
    };
    let validation =
        park.validate_service(pos, height, facing, kind)
            .map_err(|error| match error {
                PlacementError::HeightOutOfRange { .. } => {
                    "Choose a facility height from 0 to 29.".into()
                }
                PlacementError::TerrainIntersection { terrain_height, .. } => {
                    format!("Facility is below ground.\nGround height: {terrain_height}.")
                }
                error => placement_message(error),
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
        view.path_tool.service_preview = Some(ServicePreview {
            pos,
            height,
            kind,
            facing,
            valid: reason.is_none(),
            demolish: false,
        });
    }
    view.path_tool.hint = reason.clone().unwrap_or_else(|| {
        format!(
            "{kind} / tile {}, {}\nServing side must meet public path.",
            pos.x + 1,
            pos.y + 1
        )
    });
    if !click {
        return;
    }
    if let Some(reason) = reason {
        view.path_notice = Some(format!("No facility placed: {reason}"));
        return;
    }
    match park.place_service(pos, height, facing, kind) {
        Ok(id) => {
            let object = park
                .service(id)
                .and_then(|service| service.object)
                .expect("new service object");
            view.selected = Some(SurfaceId::Object(object));
            view.hovered = view.selected;
            view.selected_service = Some(id);
            view.scene_dirty = true;
            view.path_tool.clear_pointer();
            view.path_notice = Some(format!("{kind} #{} placed.\nM opens its settings.", id.0));
        }
        Err(error) => {
            view.path_notice = Some(format!("No facility placed: {}", placement_message(error)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_facing_and_height_are_independent_of_path_and_gate_shapes() {
        let mut tool = ToolState {
            slope: PathSlope::South,
            entrance_facing: PathDirection::West,
            exit_facing: PathDirection::East,
            height: 30,
            ..default()
        };
        tool.set_mode(PathTool::Service(ServiceKind::Drink));
        assert_eq!(tool.height, 29);
        tool.cycle_shape();
        assert_eq!(tool.mode, PathTool::Service(ServiceKind::Drink));
        assert_eq!(tool.service_facing, PathDirection::East);
        assert_eq!(tool.slope, PathSlope::South);
        assert_eq!(tool.entrance_facing, PathDirection::West);
        assert_eq!(tool.exit_facing, PathDirection::East);
        tool.adjust_height(127);
        assert_eq!(tool.height, 29);
        tool.adjust_height(-127);
        assert_eq!(tool.height, 0);
    }

    #[test]
    fn facility_preview_commit_rejection_and_visible_removal_are_atomic() {
        for rotation in 0..4 {
            let mut park = Park::new(32, 32);
            let pos = TilePos { x: 14, y: 18 };
            let mut view = ViewState {
                rotation,
                ..default()
            };
            view.path_tool
                .set_mode(PathTool::Service(ServiceKind::Food));
            let point = world::path_build_center(pos, &park, rotation, 8, PathSlope::Flat).unwrap();
            let before = park.clone();
            super::point_at(&mut park, &mut view, point, false, ServiceKind::Food);
            assert_eq!(park, before);
            assert!(view.path_tool.service_preview.unwrap().valid);
            super::point_at(&mut park, &mut view, point, true, ServiceKind::Food);
            let id = view.selected_service.unwrap();
            let object = park.service(id).unwrap().object.unwrap();
            let built = park.clone();
            super::point_at(&mut park, &mut view, point, true, ServiceKind::Food);
            assert_eq!(park, built, "occupied placement changes no IDs or state");
            assert!(
                view.path_notice
                    .as_ref()
                    .unwrap()
                    .starts_with("No facility placed:")
            );
            view.selected = Some(SurfaceId::Object(object));
            remove_selected(&mut park, &mut view);
            assert!(park.object(object).is_none());
            assert!(
                park.service(id).unwrap().object.is_none(),
                "retired history survives demolition"
            );
            assert!(view.selected.is_none());
        }
    }
}
