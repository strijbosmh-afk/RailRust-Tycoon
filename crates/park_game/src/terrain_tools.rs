//! Paid, atomic terrain brushes. Preview and commit share simulation planning.

use park_sim::{MAX_TERRAIN_HEIGHT, Park, SurfaceId, TerrainBrushError, TerrainBrushKind, TilePos};

use crate::{ViewState, path_tools::PathTool, world};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TerrainPreview {
    pub pos: TilePos,
    pub height: u8,
    pub target_height: u8,
    pub valid: bool,
    pub raising: bool,
    pub radius: u8,
    pub kind: TerrainBrushKind,
}

fn rejection(error: TerrainBrushError) -> String {
    match error {
        TerrainBrushError::ObjectIntersection { object, .. } => {
            format!("Object #{} blocks terrain brush.", object.0)
        }
        TerrainBrushError::OutOfBounds(_) => "Outside the park.".into(),
        TerrainBrushError::LandNotOwned(_) => "Buy this land before landscaping.".into(),
        TerrainBrushError::HeightLimit(_) => "A tile is already at its height limit.".into(),
        TerrainBrushError::WaterNeedsLevel(_) => {
            "Lower every tile to ground level before flooding.".into()
        }
        TerrainBrushError::WaterCannotBeRaised(_) => {
            "Paint land before changing water height.".into()
        }
        TerrainBrushError::RadiusTooLarge(_) => "Brush is too large.".into(),
        TerrainBrushError::InsufficientFunds {
            required_cents,
            available_cents,
        } => {
            format!(
                "Need {} more for terrain.",
                crate::ui::money(required_cents - available_cents)
            )
        }
    }
}

/// The normal picker supplies visible terrain faces, including cliffs. An
/// object hit is never a fallback route to the ground beneath it.
pub(super) fn point_at(park: &mut Park, view: &mut ViewState, click: bool) {
    let kind = match view.path_tool.mode {
        PathTool::RaiseTerrain => TerrainBrushKind::Raise,
        PathTool::LowerTerrain => TerrainBrushKind::Lower,
        PathTool::PaintWater => TerrainBrushKind::Water,
        PathTool::PaintGrass => TerrainBrushKind::Grass,
        PathTool::PaintSand => TerrainBrushKind::Sand,
        _ => return,
    };
    let unavailable = if view.view_mode == world::ViewMode::Underground {
        Some("Return to Surface view to edit land.")
    } else if !matches!(view.hovered, Some(SurfaceId::Terrain(_))) {
        Some("Point to visible terrain.")
    } else {
        None
    };
    if let Some(reason) = unavailable {
        view.path_tool.hint = reason.into();
        if click {
            view.path_notice = Some(format!("Terrain unchanged: {reason}"));
        }
        return;
    }
    let Some(SurfaceId::Terrain(pos)) = view.hovered else {
        return;
    };
    let Some(tile) = park.tile(pos).copied() else {
        return;
    };
    let target_height = match kind {
        TerrainBrushKind::Raise => tile.height.saturating_add(1).min(MAX_TERRAIN_HEIGHT),
        TerrainBrushKind::Lower => tile.height.saturating_sub(1),
        _ => tile.height,
    };
    let radius = view.path_tool.brush_radius;
    let quote = park.quote_terrain_brush(pos, radius, kind);
    let reason = match quote {
        Err(error) => Some(rejection(error)),
        Ok(quote) if quote.changed_tiles == 0 => Some("Brush would not change terrain.".into()),
        Ok(quote) if !quote.affordable() => Some(rejection(TerrainBrushError::InsufficientFunds {
            required_cents: quote.cost_cents,
            available_cents: quote.available_cents,
        })),
        Ok(_) => None,
    };
    view.path_tool.terrain_preview = Some(TerrainPreview {
        pos,
        height: tile.height,
        target_height,
        valid: reason.is_none(),
        raising: kind == TerrainBrushKind::Raise,
        radius,
        kind,
    });
    let verb = match kind {
        TerrainBrushKind::Raise => "Raise",
        TerrainBrushKind::Lower => "Lower",
        TerrainBrushKind::Water => "Water",
        TerrainBrushKind::Grass => "Grass",
        TerrainBrushKind::Sand => "Sand",
    };
    view.path_tool.hint = reason.clone().unwrap_or_else(|| {
        let quote = quote.expect("valid brush has a quote");
        format!(
            "{verb} / radius {radius} / {} tiles\nClick to spend {}.",
            quote.changed_tiles,
            crate::ui::money(quote.cost_cents)
        )
    });
    if !click {
        return;
    }
    if let Some(reason) = reason {
        view.path_notice = Some(format!("Terrain unchanged: {reason}"));
        return;
    }
    match park.apply_terrain_brush(pos, radius, kind) {
        Ok(quote) => {
            view.selected = Some(SurfaceId::Terrain(pos));
            view.terrain_custom = true;
            view.terrain_error = None;
            view.scene_dirty = true;
            view.path_tool.clear_pointer();
            view.path_notice = Some(format!(
                "{verb}: {} tiles at {}, {} / spent {}.",
                quote.changed_tiles,
                pos.x + 1,
                pos.y + 1,
                crate::ui::money(quote.cost_cents)
            ));
        }
        Err(error) => view.path_notice = Some(format!("Terrain unchanged: {}", rejection(error))),
    }
}
