//! Deliberate placement validates the complete three-by-three footprint.
use super::*;
use park_sim::{RideKind, RideModel};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CarouselPreview {
    pub kind: RideKind,
    pub model: RideModel,
    pub rotation: u8,
    pub origin: TilePos,
    pub height: u8,
    pub valid: bool,
    pub demolish: bool,
}

pub(super) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    let height = view.path_tool.height;
    let model = view.path_tool.ride_model;
    let kind = model.base_kind();
    let rotation = view.path_tool.ride_rotation;
    let cost = if model.tracked() {
        model
            .layout(TilePos { x: 0, y: 0 }, height, rotation)
            .map(|specs| {
                specs
                    .into_iter()
                    .map(park_sim::track_piece_cost_cents)
                    .sum()
            })
            .unwrap_or(0)
    } else {
        model.flat_cost()
    };
    let Some(origin) =
        world::pick_sloped_path_plane(point, park, view.rotation, height, PathSlope::Flat)
    else {
        view.path_tool.hint = format!(
            "Build {}\nOutside the park at this height.",
            crate::ui::money(cost)
        );
        if click {
            view.path_notice = Some(format!("{kind} not placed: outside the park."));
        }
        return;
    };
    let occluded = world::sloped_path_plane_occluded(
        point,
        park,
        view.rotation,
        height,
        PathSlope::Flat,
        view.view_mode,
    );
    let quote = park.catalogue_quote(model, origin, height, rotation);
    let reason = quote
        .as_ref()
        .err()
        .map(|error| {
            if cost > park.cash_cents() {
                format!(
                    "Need {} more to build.",
                    crate::ui::money(cost - park.cash_cents())
                )
            } else {
                error.to_string()
            }
        })
        .or_else(|| {
            occluded.then(|| "Surface hidden here. Rotate or use underground view.".into())
        });
    if !occluded {
        view.path_tool.carousel_preview = Some(CarouselPreview {
            kind,
            model,
            rotation,
            origin,
            height,
            valid: reason.is_none(),
            demolish: false,
        });
    }
    view.path_tool.hint = format!(
        "Build {} / Cash {}\n{}",
        crate::ui::money(cost),
        crate::ui::money(park.cash_cents()),
        reason
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| format!(
                "{} x {} / height {height} / C rotates / click to build.",
                model.dimensions().0,
                model.dimensions().1
            ))
    );
    if !click {
        return;
    }
    if let Some(reason) = reason {
        view.path_notice = Some(format!("{kind} not placed: {reason}"));
        return;
    }
    let placed = park.build_catalogue_ride(model, origin, height, rotation);
    match placed {
        Ok(id) => {
            view.select_ride(id);
            let object = park
                .carousel(id)
                .and_then(|c| c.objects.first())
                .copied()
                .or_else(|| {
                    park.coaster_circuit(id)
                        .and_then(|c| c.pieces.first())
                        .map(|p| p.object)
                });
            view.selected = object.map(SurfaceId::Object);
            view.hovered = view.selected;
            view.scene_dirty = true;
            view.path_tool.set_mode(PathTool::Inspect);
            view.path_notice = Some(format!(
                "Built for {}.\nCash {} / add J/I/K access.",
                crate::ui::money(cost),
                crate::ui::money(park.cash_cents())
            ));
        }
        Err(error) => view.path_notice = Some(format!("{kind} not placed: {error}")),
    }
}
