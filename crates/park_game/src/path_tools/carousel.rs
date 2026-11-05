//! Deliberate placement validates the complete three-by-three footprint.
use super::*;
use park_sim::{CarouselError, RideKind};

fn build_error(error: CarouselError) -> String {
    match error {
        CarouselError::InsufficientFunds {
            required_cents,
            available_cents,
        } => format!(
            "Need {} more to build.",
            crate::ui::money(required_cents - available_cents)
        ),
        error => error.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CarouselPreview {
    pub kind: RideKind,
    pub origin: TilePos,
    pub height: u8,
    pub valid: bool,
    pub demolish: bool,
}

pub(super) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    let height = view.path_tool.height;
    let kind = view.path_tool.flat_ride_kind;
    let cost = match kind {
        RideKind::Swing => park_sim::SWING_BUILD_COST_CENTS,
        RideKind::DropTower => park_sim::DROP_TOWER_BUILD_COST_CENTS,
        _ => park_sim::CAROUSEL_BUILD_COST_CENTS,
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
    let quote = match kind {
        RideKind::Swing => park.swing_build_quote(origin, height),
        RideKind::DropTower => park.drop_tower_build_quote(origin, height),
        _ => park.carousel_build_quote(origin, height),
    };
    let reason = quote.err().map(build_error).or_else(|| {
        occluded.then(|| "Surface hidden here. Rotate or use underground view.".into())
    });
    if !occluded {
        view.path_tool.carousel_preview = Some(CarouselPreview {
            kind,
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
            .unwrap_or_else(|| format!("3 x 3 / height {height} / click to build."))
    );
    if !click {
        return;
    }
    if let Some(reason) = reason {
        view.path_notice = Some(format!("{kind} not placed: {reason}"));
        return;
    }
    let placed = match kind {
        RideKind::Swing => park.place_swing(origin, height),
        RideKind::DropTower => park.place_drop_tower(origin, height),
        _ => park.place_carousel(origin, height),
    };
    match placed {
        Ok(id) => {
            view.select_ride(id);
            let object = park.carousel(id).and_then(|c| c.objects.first()).copied();
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
