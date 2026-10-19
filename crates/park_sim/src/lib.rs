//! Engine-independent park state and a deterministic fixed-step simulation clock.
//!
//! Rendering, input, and wall-clock time belong to the application. Keeping those
//! concerns outside this crate makes the simulation usable in headless tests.

/// Number of normal-speed simulation ticks in one second.
pub const TICKS_PER_SECOND: u64 = 30;

/// Largest supported number of tiles along either edge of a foundation map.
pub const MAX_MAP_SIZE: u32 = 256;

/// An integer tile coordinate, with the origin at the map's first corner.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Terrain {
    Grass,
    Water,
    Sand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tile {
    pub terrain: Terrain,
    /// Height in future terrain steps. The foundation starts entirely flat.
    pub height: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Speed {
    Paused,
    #[default]
    Normal,
    Fast,
    Fastest,
}

impl Speed {
    /// Simulation steps to perform for each application fixed update.
    pub const fn multiplier(self) -> u32 {
        match self {
            Self::Paused => 0,
            Self::Normal => 1,
            Self::Fast => 2,
            Self::Fastest => 4,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Park {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
    ticks: u64,
}

impl Park {
    /// Create a deterministic park with a broad meadow and a sand-edged lake.
    ///
    /// # Panics
    ///
    /// Panics if either dimension is outside `1..=MAX_MAP_SIZE`. Validating
    /// before allocation prevents empty maps and unbounded allocations.
    pub fn new(width: u32, height: u32) -> Self {
        assert!(
            (1..=MAX_MAP_SIZE).contains(&width) && (1..=MAX_MAP_SIZE).contains(&height),
            "park dimensions must each be between 1 and {MAX_MAP_SIZE} tiles"
        );

        let mut tiles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                tiles.push(Tile {
                    terrain: terrain_at(TilePos { x, y }, width, height),
                    height: 0,
                });
            }
        }

        Self {
            width,
            height,
            tiles,
            ticks: 0,
        }
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Look up a tile without allowing an out-of-bounds x to wrap into a row.
    pub fn tile(&self, pos: TilePos) -> Option<&Tile> {
        if pos.x >= self.width || pos.y >= self.height {
            return None;
        }
        self.tiles.get((pos.y * self.width + pos.x) as usize)
    }

    pub const fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Whole seconds elapsed in simulation time, independent of rendering.
    pub const fn elapsed_seconds(&self) -> u64 {
        self.ticks / TICKS_PER_SECOND
    }

    /// Advance once per application fixed update (30 updates per real second).
    ///
    /// Saturation prevents the clock from wrapping if it reaches its limit.
    pub fn advance(&mut self, speed: Speed) {
        self.ticks = self.ticks.saturating_add(u64::from(speed.multiplier()));
    }
}

fn terrain_at(pos: TilePos, width: u32, height: u32) -> Terrain {
    // Integer coordinates in 0..1000 keep generation platform-independent.
    // Sampling tile centers also handles 1×1 and other very small maps.
    let x = i64::from((pos.x * 2 + 1) * 500 / width);
    let y = i64::from((pos.y * 2 + 1) * 500 / height);

    // Two overlapping ellipses create a gently irregular lake in one corner.
    // Their radii leave the park center and entrance edge open for building.
    let main_lake = ellipse_score(x - 840, y - 230, 135, 145);
    let small_bay = ellipse_score(x - 890, y - 310, 95, 115);
    let shoreline = main_lake.min(small_bay);

    if shoreline <= 10_000 {
        Terrain::Water
    } else if shoreline <= 15_000 {
        Terrain::Sand
    } else {
        Terrain::Grass
    }
}

fn ellipse_score(dx: i64, dy: i64, radius_x: i64, radius_y: i64) -> i64 {
    dx * dx * 10_000 / (radius_x * radius_x) + dy * dy * 10_000 / (radius_y * radius_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_dimensions_and_bounds_are_respected() {
        let park = Park::new(7, 3);
        assert_eq!((park.width(), park.height()), (7, 3));
        assert_eq!(park.tiles.len(), 21);
        assert!(park.tile(TilePos { x: 0, y: 0 }).is_some());
        assert!(park.tile(TilePos { x: 6, y: 2 }).is_some());
        for pos in [
            TilePos { x: 7, y: 0 },
            TilePos { x: 0, y: 3 },
            TilePos { x: u32::MAX, y: 0 },
            TilePos { x: 0, y: u32::MAX },
        ] {
            assert_eq!(park.tile(pos), None, "out-of-bounds tile {pos:?}");
        }
    }

    #[test]
    fn terrain_is_repeatable_flat_and_leaves_a_central_meadow() {
        let first = Park::new(32, 32);
        assert_eq!(first, Park::new(32, 32));
        for terrain in [Terrain::Grass, Terrain::Water, Terrain::Sand] {
            assert!(first.tiles.iter().any(|tile| tile.terrain == terrain));
        }
        assert!(first.tiles.iter().all(|tile| tile.height == 0));
        for y in 10..22 {
            for x in 10..22 {
                assert_eq!(
                    first.tile(TilePos { x, y }).unwrap().terrain,
                    Terrain::Grass
                );
            }
        }
    }

    #[test]
    fn smallest_and_largest_maps_are_supported() {
        let smallest = Park::new(1, 1);
        assert_eq!(smallest.tiles.len(), 1);
        assert_eq!(smallest.tiles[0].terrain, Terrain::Grass);
        let largest = Park::new(MAX_MAP_SIZE, MAX_MAP_SIZE);
        assert_eq!(largest.tiles.len(), (MAX_MAP_SIZE * MAX_MAP_SIZE) as usize);
    }

    #[test]
    #[should_panic(expected = "park dimensions must each be between")]
    fn zero_dimension_is_rejected() {
        Park::new(0, 32);
    }

    #[test]
    #[should_panic(expected = "park dimensions must each be between")]
    fn excessive_dimension_is_rejected_before_allocation() {
        Park::new(32, u32::MAX);
    }

    #[test]
    fn clock_obeys_pause_and_each_speed() {
        let mut park = Park::new(1, 1);
        for _ in 0..TICKS_PER_SECOND {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.ticks(), 30);
        assert_eq!(park.elapsed_seconds(), 1);
        park.advance(Speed::Paused);
        assert_eq!(park.ticks(), 30);
        park.advance(Speed::Fast);
        assert_eq!(park.ticks(), 32);
        park.advance(Speed::Fastest);
        assert_eq!(park.ticks(), 36);
        park.advance(Speed::Normal);
        assert_eq!(park.ticks(), 37);
    }

    #[test]
    fn clock_does_not_wrap_at_its_limit() {
        let mut park = Park::new(1, 1);
        park.ticks = u64::MAX - 1;
        park.advance(Speed::Fastest);
        assert_eq!(park.ticks(), u64::MAX);
    }
}
