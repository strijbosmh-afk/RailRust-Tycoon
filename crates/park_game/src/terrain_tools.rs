//! Single-tile player terrain commands. Preview and commit share validation.

use park_sim::{MAX_TERRAIN_HEIGHT, Park, SurfaceId, Terrain, TerrainHeightError, TilePos};

use crate::{ViewState, path_tools::PathTool, world};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TerrainPreview {
    pub pos: TilePos,
    pub height: u8,
    pub target_height: u8,
    pub valid: bool,
    pub raising: bool,
}

fn rejection(error: TerrainHeightError, height: u8) -> String {
    match error {
        TerrainHeightError::ObjectIntersection { object, .. } => {
            format!("Object #{} blocks terrain height {height}.", object.0)
        }
        TerrainHeightError::OutOfBounds { .. } => "Outside the park.".into(),
        TerrainHeightError::HeightTooHigh { max, .. } => {
            format!("Terrain cannot rise above height {max}.")
        }
    }
}

/// The normal map picker supplies the nearest visible terrain face, including
/// cliffs. Objects are never a fallback route to the ground beneath them.
/// The caller applies focus/UI/drag/capture/load gates before dispatch.
pub(super) fn point_at(park: &mut Park, view: &mut ViewState, click: bool) {
    let raising = view.path_tool.mode == PathTool::RaiseTerrain;
    let unavailable = if view.view_mode == world::ViewMode::Underground {
        Some("Return to Surface view to edit land.")
    } else if !matches!(view.hovered, Some(SurfaceId::Terrain(_))) {
        Some("Point to visible grass or sand.")
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
    let target_height = if raising {
        tile.height.saturating_add(1).min(MAX_TERRAIN_HEIGHT)
    } else {
        tile.height.saturating_sub(1)
    };
    let reason = if tile.terrain == Terrain::Water {
        Some("Water editing is not available.".into())
    } else if target_height == tile.height {
        Some(format!("Ground is already at height {}.", tile.height))
    } else {
        park.validate_terrain_height(pos, target_height)
            .err()
            .map(|error| rejection(error, target_height))
    };
    view.path_tool.terrain_preview = Some(TerrainPreview {
        pos,
        height: tile.height,
        target_height,
        valid: reason.is_none(),
        raising,
    });
    let verb = if raising { "Raise" } else { "Lower" };
    view.path_tool.hint = reason.clone().unwrap_or_else(|| {
        format!(
            "Tile {}, {} / {} -> {target_height}\nClick to {} one step.",
            pos.x + 1,
            pos.y + 1,
            tile.height,
            verb.to_lowercase()
        )
    });
    if !click {
        return;
    }
    if let Some(reason) = reason {
        view.path_notice = Some(format!("Terrain unchanged: {reason}"));
        return;
    }
    match park.set_terrain_height(pos, target_height) {
        Ok(()) => {
            view.selected = Some(SurfaceId::Terrain(pos));
            view.terrain_custom = true;
            view.terrain_error = None;
            view.scene_dirty = true;
            view.path_tool.clear_pointer();
            view.path_notice = Some(format!(
                "{} tile {}, {}: {} -> {target_height}.",
                if raising { "Raised" } else { "Lowered" },
                pos.x + 1,
                pos.y + 1,
                tile.height
            ));
        }
        Err(error) => {
            view.path_notice = Some(format!(
                "Terrain unchanged: {}",
                rejection(error, target_height)
            ));
        }
    }
}
