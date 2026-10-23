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
        view.path_tool.hint = format!(
            "Build {}\nOutside the park at this height.",
            crate::ui::money(kind.build_cost_cents())
        );
        if click {
            view.path_notice = Some("No facility placed: outside the park.".into());
        }
        return;
    };
    let validation =
        park.service_build_quote(pos, height, facing, kind)
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
    view.path_tool.hint = format!(
        "Build {} / Cash {}\n{}",
        crate::ui::money(kind.build_cost_cents()),
        crate::ui::money(park.cash_cents()),
        reason
            .as_deref()
            .unwrap_or("Face the counter toward public path.")
    );
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
            view.path_notice = Some(format!(
                "Built for {}.\nCash {} / M: settings.",
                crate::ui::money(kind.build_cost_cents()),
                crate::ui::money(park.cash_cents())
            ));
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
        for (rotation, kind) in (0..4).flat_map(|rotation| {
            [ServiceKind::Food, ServiceKind::Drink, ServiceKind::Toilet]
                .map(|kind| (rotation, kind))
        }) {
            let mut park = Park::new(32, 32);
            let pos = TilePos { x: 14, y: 18 };
            let mut view = ViewState {
                rotation,
                ..default()
            };
            view.path_tool.set_mode(PathTool::Service(kind));
            let point = world::path_build_center(pos, &park, rotation, 8, PathSlope::Flat).unwrap();
            let before = park.clone();
            super::point_at(&mut park, &mut view, point, false, kind);
            assert_eq!(park, before);
            assert!(view.path_tool.service_preview.unwrap().valid);
            assert!(view.path_tool.hint.contains(&format!(
                "Build {} / Cash $1000.00",
                crate::ui::money(kind.build_cost_cents())
            )));
            super::point_at(&mut park, &mut view, point, true, kind);
            let id = view.selected_service.unwrap();
            let object = park.service(id).unwrap().object.unwrap();
            assert_eq!(
                park.cash_cents(),
                before.cash_cents() - kind.build_cost_cents()
            );
            assert_eq!(
                park.service(id).unwrap().construction_cost_cents,
                kind.build_cost_cents()
            );
            let entry = park.finance_ledger().entries.last().unwrap();
            assert_eq!(
                (entry.kind, entry.service, entry.ride, entry.amount_cents),
                (
                    park_sim::FinanceKind::ServiceConstruction,
                    Some(id),
                    None,
                    kind.build_cost_cents()
                )
            );
            let built = park.clone();
            super::point_at(&mut park, &mut view, point, true, kind);
            assert_eq!(park, built, "occupied placement changes no IDs or state");
            assert!(
                view.path_notice
                    .as_ref()
                    .unwrap()
                    .starts_with("No facility placed:")
            );
            view.path_tool.set_mode(PathTool::Demolish);
            view.hovered = Some(SurfaceId::Object(object));
            super::super::point_at(&mut park, &mut view, point, false);
            assert_eq!(park, built, "refund previews are read-only");
            assert!(
                view.path_tool
                    .service_preview
                    .is_some_and(|p| p.demolish && p.valid)
            );
            assert!(view.path_tool.hint.contains(&format!(
                "Refund {}",
                crate::ui::money(kind.build_cost_cents() / 2)
            )));
            view.selected = Some(SurfaceId::Object(object));
            remove_selected(&mut park, &mut view);
            assert!(park.object(object).is_none());
            assert!(
                park.service(id).unwrap().object.is_none(),
                "retired history survives demolition"
            );
            assert!(view.selected.is_none());
            assert_eq!(
                park.cash_cents(),
                before.cash_cents() - kind.build_cost_cents() / 2
            );
            let entry = park.finance_ledger().entries.last().unwrap();
            assert_eq!(
                (entry.kind, entry.service, entry.ride, entry.amount_cents),
                (
                    park_sim::FinanceKind::ServiceDemolitionRefund,
                    Some(id),
                    None,
                    kind.build_cost_cents() / 2
                )
            );
            let removed = park.clone();
            view.selected = Some(SurfaceId::Object(object));
            remove_selected(&mut park, &mut view);
            assert_eq!(park, removed, "stale selections never refund twice");
        }
    }

    #[test]
    fn unaffordable_service_preview_stays_red_and_click_leaves_every_field_unchanged() {
        for kind in [ServiceKind::Food, ServiceKind::Drink, ServiceKind::Toilet] {
            let mut park = Park::new(32, 32);
            let pos = TilePos { x: 14, y: 18 };
            while park.cash_cents() >= kind.build_cost_cents() {
                let id = park
                    .place_service(pos, 8, PathDirection::North, kind)
                    .unwrap();
                park.remove_object(park.service(id).unwrap().object.unwrap())
                    .unwrap();
            }
            let original = park.clone();
            let mut view = ViewState::default();
            view.path_tool.set_mode(PathTool::Service(kind));
            let point = world::path_build_center(pos, &park, 0, 8, PathSlope::Flat).unwrap();
            for click in [false, true] {
                super::point_at(&mut park, &mut view, point, click, kind);
                assert_eq!(park, original);
                assert!(view.path_tool.service_preview.is_some_and(|p| !p.valid));
                assert!(view.path_tool.hint.contains(&format!(
                    "Build {}",
                    crate::ui::money(kind.build_cost_cents())
                )));
                assert!(view.path_tool.hint.contains(&format!(
                    "Need {} more",
                    crate::ui::money(kind.build_cost_cents() - park.cash_cents())
                )));
                assert!(!view.scene_dirty);
            }
        }
    }
}
