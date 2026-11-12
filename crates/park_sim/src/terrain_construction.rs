//! Atomic paid terrain brushes. A quote and its commit use the same plan.
use crate::{FinanceKind, MAX_TERRAIN_HEIGHT, ObjectId, Park, Terrain, Tile, TilePos};

pub const MAX_TERRAIN_BRUSH_RADIUS: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainBrushKind {
    Raise,
    Lower,
    Grass,
    Sand,
    Water,
}

impl TerrainBrushKind {
    const fn cost_cents(self) -> u64 {
        match self {
            Self::Raise | Self::Lower => 100,
            Self::Grass | Self::Sand => 50,
            Self::Water => 200,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerrainBrushQuote {
    pub changed_tiles: usize,
    pub cost_cents: u64,
    pub available_cents: u64,
}

impl TerrainBrushQuote {
    pub fn affordable(self) -> bool {
        self.cost_cents <= self.available_cents
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainBrushError {
    OutOfBounds(TilePos),
    LandNotOwned(TilePos),
    RadiusTooLarge(u8),
    HeightLimit(TilePos),
    WaterNeedsLevel(TilePos),
    WaterCannotBeRaised(TilePos),
    ObjectIntersection {
        pos: TilePos,
        object: ObjectId,
    },
    InsufficientFunds {
        required_cents: u64,
        available_cents: u64,
    },
}

impl std::fmt::Display for TerrainBrushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds(pos) => write!(f, "tile {}, {} is outside the park", pos.x, pos.y),
            Self::LandNotOwned(pos) => write!(f, "tile {}, {} is not owned", pos.x, pos.y),
            Self::RadiusTooLarge(radius) => write!(
                f,
                "brush radius {radius} exceeds {MAX_TERRAIN_BRUSH_RADIUS}"
            ),
            Self::HeightLimit(pos) => write!(f, "tile {}, {} is at its height limit", pos.x, pos.y),
            Self::WaterNeedsLevel(pos) => write!(
                f,
                "lower tile {}, {} to ground level before flooding",
                pos.x, pos.y
            ),
            Self::WaterCannotBeRaised(pos) => write!(
                f,
                "paint land on tile {}, {} before changing its height",
                pos.x, pos.y
            ),
            Self::ObjectIntersection { pos, object } => {
                write!(f, "object #{} blocks tile {}, {}", object.0, pos.x, pos.y)
            }
            Self::InsufficientFunds {
                required_cents,
                available_cents,
            } => write!(
                f,
                "landscaping costs ${}.{:02}; park cash is ${}.{:02}",
                required_cents / 100,
                required_cents % 100,
                available_cents / 100,
                available_cents % 100
            ),
        }
    }
}

impl std::error::Error for TerrainBrushError {}

impl Park {
    fn terrain_brush_plan(
        &self,
        center: TilePos,
        radius: u8,
        kind: TerrainBrushKind,
    ) -> Result<Vec<(usize, Tile)>, TerrainBrushError> {
        if self.tile(center).is_none() {
            return Err(TerrainBrushError::OutOfBounds(center));
        }
        if radius > MAX_TERRAIN_BRUSH_RADIUS {
            return Err(TerrainBrushError::RadiusTooLarge(radius));
        }
        let mut changes = Vec::new();
        let radius = u32::from(radius);
        for y in
            center.y.saturating_sub(radius)..=center.y.saturating_add(radius).min(self.height - 1)
        {
            for x in center.x.saturating_sub(radius)
                ..=center.x.saturating_add(radius).min(self.width - 1)
            {
                let pos = TilePos { x, y };
                let index = (y * self.width + x) as usize;
                let current = self.tiles[index];
                if !current.owned {
                    return Err(TerrainBrushError::LandNotOwned(pos));
                }
                let mut next = current;
                match kind {
                    TerrainBrushKind::Raise | TerrainBrushKind::Lower => {
                        if current.terrain == Terrain::Water {
                            return Err(TerrainBrushError::WaterCannotBeRaised(pos));
                        }
                        next.height = if kind == TerrainBrushKind::Raise {
                            current
                                .height
                                .checked_add(1)
                                .filter(|height| *height <= MAX_TERRAIN_HEIGHT)
                        } else {
                            current.height.checked_sub(1)
                        }
                        .ok_or(TerrainBrushError::HeightLimit(pos))?;
                    }
                    TerrainBrushKind::Grass => next.terrain = Terrain::Grass,
                    TerrainBrushKind::Sand => next.terrain = Terrain::Sand,
                    TerrainBrushKind::Water => {
                        if current.height != 0 {
                            return Err(TerrainBrushError::WaterNeedsLevel(pos));
                        }
                        next.terrain = Terrain::Water;
                    }
                }
                if next == current {
                    continue;
                }
                self.validate_terrain_clearance(pos, next.height)
                    .map_err(|error| match error {
                        crate::TerrainHeightError::ObjectIntersection { object, .. } => {
                            TerrainBrushError::ObjectIntersection { pos, object }
                        }
                        _ => TerrainBrushError::OutOfBounds(pos),
                    })?;
                if next.terrain == Terrain::Water {
                    // A bridge may remain above the water, but a ground object
                    // cannot become submerged by painting its supporting tile.
                    if let Some(object) = self
                        .objects
                        .iter()
                        .find(|object| object.pos == pos && object.height < 2)
                    {
                        return Err(TerrainBrushError::ObjectIntersection {
                            pos,
                            object: object.id,
                        });
                    }
                }
                changes.push((index, next));
            }
        }
        Ok(changes)
    }

    pub fn quote_terrain_brush(
        &self,
        center: TilePos,
        radius: u8,
        kind: TerrainBrushKind,
    ) -> Result<TerrainBrushQuote, TerrainBrushError> {
        let changed_tiles = self.terrain_brush_plan(center, radius, kind)?.len();
        Ok(TerrainBrushQuote {
            changed_tiles,
            cost_cents: changed_tiles as u64 * kind.cost_cents(),
            available_cents: self.cash_cents,
        })
    }

    pub fn apply_terrain_brush(
        &mut self,
        center: TilePos,
        radius: u8,
        kind: TerrainBrushKind,
    ) -> Result<TerrainBrushQuote, TerrainBrushError> {
        let plan = self.terrain_brush_plan(center, radius, kind)?;
        let quote = TerrainBrushQuote {
            changed_tiles: plan.len(),
            cost_cents: plan.len() as u64 * kind.cost_cents(),
            available_cents: self.cash_cents,
        };
        if !quote.affordable() {
            return Err(TerrainBrushError::InsufficientFunds {
                required_cents: quote.cost_cents,
                available_cents: quote.available_cents,
            });
        }
        for (index, tile) in plan {
            self.tiles[index] = tile;
        }
        self.record_cash(FinanceKind::Landscaping, None, quote.cost_cents);
        Ok(quote)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brush_quote_commit_save_and_rejection_are_atomic() {
        let mut park = Park::new(64, 64);
        let center = TilePos { x: 10, y: 10 };
        let before = park.clone();
        let quote = park
            .quote_terrain_brush(center, 1, TerrainBrushKind::Raise)
            .unwrap();
        assert_eq!(quote.changed_tiles, 9);
        assert_eq!(quote.cost_cents, 900);
        assert_eq!(park, before);
        park.apply_terrain_brush(center, 1, TerrainBrushKind::Raise)
            .unwrap();
        assert_eq!(park.cash_cents(), before.cash_cents() - 900);
        assert_eq!(park.tile(center).unwrap().height, 1);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
        let unchanged = park.clone();
        assert_eq!(
            park.apply_terrain_brush(center, 1, TerrainBrushKind::Water),
            Err(TerrainBrushError::WaterNeedsLevel(TilePos { x: 9, y: 9 }))
        );
        assert_eq!(park, unchanged);
        assert_eq!(
            park.apply_terrain_brush(center, 3, TerrainBrushKind::Sand),
            Err(TerrainBrushError::RadiusTooLarge(3))
        );
        assert_eq!(park, unchanged);
    }

    #[test]
    fn water_respects_ground_objects_but_allows_clear_bridges() {
        let mut park = Park::new(32, 32);
        let pos = TilePos { x: 5, y: 5 };
        let path = park.place_path(pos, 0).unwrap();
        let before = park.clone();
        assert!(
            matches!(park.apply_terrain_brush(pos, 0, TerrainBrushKind::Water), Err(TerrainBrushError::ObjectIntersection { object, .. }) if object == path)
        );
        assert_eq!(park, before);
        park.remove_object(path);
        park.place_path(pos, 2).unwrap();
        park.apply_terrain_brush(pos, 0, TerrainBrushKind::Water)
            .unwrap();
        assert_eq!(park.tile(pos).unwrap().terrain, Terrain::Water);
        assert!(matches!(
            park.validate_path(pos, 0),
            Err(crate::PlacementError::WaterNeedsBridge { .. })
        ));
    }

    #[test]
    fn archived_landscaping_receipts_still_conserve_cash_after_save() {
        let mut park = Park::new(64, 64);
        for index in 0..140 {
            let pos = TilePos {
                x: index % 50,
                y: 5 + index / 50,
            };
            let kind = if park.tile(pos).unwrap().terrain == Terrain::Sand {
                TerrainBrushKind::Grass
            } else {
                TerrainBrushKind::Sand
            };
            let quote = park.apply_terrain_brush(pos, 0, kind).unwrap();
            assert_eq!(quote.cost_cents, 50);
        }
        assert!(park.finance_ledger().archived_entries > 0);
        assert_eq!(park.finance_stats().construction_cents, 7_000);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }
}
