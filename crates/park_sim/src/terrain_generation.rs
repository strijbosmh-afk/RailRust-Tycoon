//! Reproducible empty-map terrain shaping. The generated tile heights, rather
//! than the seed, are authoritative and already persist in every park save.
use crate::{Park, Terrain, TerrainHeightError, TilePos};

fn corner(seed: u64, x: u32, y: u32) -> u32 {
    let mut value = seed
        ^ u64::from(x).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ u64::from(y).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    ((value ^ (value >> 31)) % 7) as u32
}

fn height(seed: u64, pos: TilePos, width: u32, depth: u32, terrain: Terrain) -> u8 {
    if terrain != Terrain::Grass {
        return 0;
    }
    let gx = pos.x / 8;
    let gy = pos.y / 8;
    let fx = pos.x % 8;
    let fy = pos.y % 8;
    let top = corner(seed, gx, gy) * (8 - fx) + corner(seed, gx + 1, gy) * fx;
    let bottom = corner(seed, gx, gy + 1) * (8 - fx) + corner(seed, gx + 1, gy + 1) * fx;
    let blended = (top * (8 - fy) + bottom * fy + 32) / 64;
    let border = pos
        .x
        .min(pos.y)
        .min(width - 1 - pos.x)
        .min(depth - 1 - pos.y)
        / 2;
    blended.min(border) as u8
}

impl Park {
    /// Shape an empty map into low rolling hills from a stable seed. All
    /// changes are committed together; an occupied map is left unchanged.
    pub fn generate_rolling_terrain(&mut self, seed: u64) -> Result<(), TerrainHeightError> {
        if let Some(object) = self.objects.first() {
            return Err(TerrainHeightError::ObjectIntersection {
                pos: object.pos,
                object: object.id,
            });
        }
        let heights: Vec<u8> = self
            .tiles
            .iter()
            .enumerate()
            .map(|(index, tile)| {
                let pos = TilePos {
                    x: index as u32 % self.width,
                    y: index as u32 / self.width,
                };
                height(seed, pos, self.width, self.height, tile.terrain)
            })
            .collect();
        for (tile, height) in self.tiles.iter_mut().zip(heights) {
            tile.height = height;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling_hills_are_seeded_saved_and_do_not_change_occupied_maps() {
        let mut first = Park::new(64, 64);
        first.generate_rolling_terrain(17).unwrap();
        let mut same = Park::new(64, 64);
        same.generate_rolling_terrain(17).unwrap();
        let mut different = Park::new(64, 64);
        different.generate_rolling_terrain(18).unwrap();
        assert_eq!(first, same);
        assert_ne!(first, different);
        assert!(first.max_terrain_height() > 0);
        assert_eq!(
            Park::from_save_bytes(&first.to_save_bytes().unwrap()).unwrap(),
            first
        );
        let site = TilePos { x: 0, y: 0 };
        first.place_path(site, 0).unwrap();
        let before = first.clone();
        assert!(matches!(
            first.generate_rolling_terrain(19),
            Err(TerrainHeightError::ObjectIntersection { .. })
        ));
        assert_eq!(first, before);
    }
}
