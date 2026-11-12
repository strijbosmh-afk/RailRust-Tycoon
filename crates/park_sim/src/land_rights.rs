//! Persistent tile ownership and atomic adjacent land purchases.
use crate::{FinanceKind, Park, TilePos};

pub const LAND_TILE_PRICE_CENTS: u64 = 500;

pub(crate) const fn owned_by_default() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LandRightsError {
    OutOfBounds(TilePos),
    AlreadyOwned(TilePos),
    NotAdjacent(TilePos),
    InsufficientFunds {
        required_cents: u64,
        available_cents: u64,
    },
}

impl std::fmt::Display for LandRightsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds(pos) => write!(f, "tile {}, {} is outside the park", pos.x, pos.y),
            Self::AlreadyOwned(pos) => write!(f, "tile {}, {} is already owned", pos.x, pos.y),
            Self::NotAdjacent(pos) => write!(f, "tile {}, {} must touch owned land", pos.x, pos.y),
            Self::InsufficientFunds {
                required_cents,
                available_cents,
            } => write!(
                f,
                "land costs ${}.{:02}; cash is ${}.{:02}",
                required_cents / 100,
                required_cents % 100,
                available_cents / 100,
                available_cents % 100
            ),
        }
    }
}

impl std::error::Error for LandRightsError {}

impl Park {
    pub fn owns_land(&self, pos: TilePos) -> bool {
        self.tile(pos).is_some_and(|tile| tile.owned)
    }

    /// Empty large sandboxes begin with an owned 32-tile square and a narrow
    /// route to the west map edge. Existing parks and 32-tile fixtures retain
    /// all their land. This setup is only legal before any construction.
    pub fn prepare_large_sandbox_land(&mut self) -> bool {
        if self.width < 64
            || self.height < 64
            || !self.objects.is_empty()
            || !self.guests.is_empty()
            || self.tiles.iter().any(|tile| !tile.owned)
        {
            return false;
        }
        let mid_x = self.width / 2;
        let mid_y = self.height / 2;
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            let x = index as u32 % self.width;
            let y = index as u32 / self.width;
            tile.owned = (x >= mid_x - 16 && x < mid_x + 16 && y >= mid_y - 16 && y < mid_y + 16)
                || (y == mid_y && x <= mid_x);
        }
        true
    }

    pub fn quote_land_purchase(&self, pos: TilePos) -> Result<u64, LandRightsError> {
        let tile = self.tile(pos).ok_or(LandRightsError::OutOfBounds(pos))?;
        if tile.owned {
            return Err(LandRightsError::AlreadyOwned(pos));
        }
        let adjacent = [
            pos.x.checked_sub(1).map(|x| TilePos { x, y: pos.y }),
            pos.x.checked_add(1).map(|x| TilePos { x, y: pos.y }),
            pos.y.checked_sub(1).map(|y| TilePos { x: pos.x, y }),
            pos.y.checked_add(1).map(|y| TilePos { x: pos.x, y }),
        ];
        if !adjacent
            .into_iter()
            .flatten()
            .any(|neighbor| self.owns_land(neighbor))
        {
            return Err(LandRightsError::NotAdjacent(pos));
        }
        if self.cash_cents < LAND_TILE_PRICE_CENTS {
            return Err(LandRightsError::InsufficientFunds {
                required_cents: LAND_TILE_PRICE_CENTS,
                available_cents: self.cash_cents,
            });
        }
        Ok(LAND_TILE_PRICE_CENTS)
    }

    pub fn buy_land(&mut self, pos: TilePos) -> Result<(), LandRightsError> {
        let cost = self.quote_land_purchase(pos)?;
        self.tiles[(pos.y * self.width + pos.x) as usize].owned = true;
        self.record_cash(FinanceKind::LandPurchase, None, cost);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TerrainBrushKind, TilePos};

    #[test]
    fn large_sandbox_purchase_unlocks_only_adjacent_land_and_survives_save() {
        let mut park = Park::new(64, 64);
        assert!(park.prepare_large_sandbox_land());
        let edge = TilePos { x: 48, y: 32 };
        let distant = TilePos { x: 55, y: 32 };
        assert!(park.owns_land(TilePos { x: 47, y: 32 }));
        assert!(!park.owns_land(edge));
        let before = park.clone();
        assert!(matches!(
            park.validate_path(edge, 0),
            Err(crate::PlacementError::LandNotOwned { .. })
        ));
        assert_eq!(
            park.apply_terrain_brush(edge, 0, TerrainBrushKind::Sand),
            Err(crate::TerrainBrushError::LandNotOwned(edge))
        );
        assert_eq!(
            park.quote_land_purchase(distant),
            Err(LandRightsError::NotAdjacent(distant))
        );
        assert_eq!(park, before);
        assert_eq!(park.quote_land_purchase(edge), Ok(LAND_TILE_PRICE_CENTS));
        park.buy_land(edge).unwrap();
        assert!(park.owns_land(edge));
        assert_eq!(
            park.cash_cents(),
            before.cash_cents() - LAND_TILE_PRICE_CENTS
        );
        park.apply_terrain_brush(edge, 0, TerrainBrushKind::Sand)
            .unwrap();
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }
}
