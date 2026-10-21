//! Shared, integer path geometry. A slope rises one height step across a tile.

use crate::{Axis, ObjectKind, SurfaceObject, TilePos};

/// Sixty units per tile/height step represent half steps and all 30 walk ticks exactly.
pub const PATH_UNITS: i32 = 60;
pub const SLOPED_PATH_CLEARANCE: u8 = 3;
pub const MAX_SLOPED_PATH_HEIGHT: u8 = crate::MAX_TERRAIN_HEIGHT - SLOPED_PATH_CLEARANCE;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PathDirection {
    North,
    East,
    South,
    West,
}

impl PathDirection {
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    pub const fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    pub const fn offset(self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::East => (1, 0),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        }
    }

    pub fn neighbor(self, pos: TilePos) -> Option<TilePos> {
        let (x, y) = self.offset();
        Some(TilePos {
            x: pos.x.checked_add_signed(x)?,
            y: pos.y.checked_add_signed(y)?,
        })
    }
}

/// Direction of rise; North is negative map y and East is positive map x.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PathSlope {
    #[default]
    Flat,
    North,
    East,
    South,
    West,
}

impl PathSlope {
    pub const fn direction(self) -> Option<PathDirection> {
        match self {
            Self::Flat => None,
            Self::North => Some(PathDirection::North),
            Self::East => Some(PathDirection::East),
            Self::South => Some(PathDirection::South),
            Self::West => Some(PathDirection::West),
        }
    }

    pub const fn axis(self) -> Axis {
        match self {
            Self::North | Self::South => Axis::Y,
            _ => Axis::X,
        }
    }

    pub const fn clearance(self) -> u8 {
        if matches!(self, Self::Flat) {
            crate::PATH_CLEARANCE
        } else {
            SLOPED_PATH_CLEARANCE
        }
    }
}

/// Unrotated map coordinates and elevation, all in PATH_UNITS. Tile centers are integral tiles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PathPoint {
    pub x: i32,
    pub y: i32,
    pub height: i32,
}

impl SurfaceObject {
    /// Only an enabled walking edge has a height. Slopes have two opposite ends.
    pub fn path_edge_height(&self, edge: PathDirection) -> Option<u8> {
        if self.kind != ObjectKind::Path {
            return None;
        }
        match self.path_slope.direction() {
            None => Some(self.height),
            Some(high) if edge == high => self.height.checked_add(1),
            Some(high) if edge == high.opposite() => Some(self.height),
            _ => None,
        }
    }

    /// Height on the walking plane, with local x/y offsets from the tile center.
    /// Offsets must be within half a tile; heights are returned in PATH_UNITS.
    pub fn path_height_at(&self, offset_x: i32, offset_y: i32) -> Option<i32> {
        let half = PATH_UNITS / 2;
        if self.kind != ObjectKind::Path
            || !(-half..=half).contains(&offset_x)
            || !(-half..=half).contains(&offset_y)
        {
            return None;
        }
        let base = i32::from(self.height) * PATH_UNITS;
        Some(match self.path_slope.direction() {
            None => base,
            Some(direction) => {
                let (dx, dy) = direction.offset();
                base + half + dx * offset_x + dy * offset_y
            }
        })
    }

    pub fn path_center(&self) -> Option<PathPoint> {
        Some(PathPoint {
            x: self.pos.x as i32 * PATH_UNITS,
            y: self.pos.y as i32 * PATH_UNITS,
            height: self.path_height_at(0, 0)?,
        })
    }
}

/// Cardinal neighbors connect only through enabled ends at identical edge heights.
pub fn paths_connect(a: &SurfaceObject, b: &SurfaceObject) -> bool {
    PathDirection::ALL.into_iter().any(|edge| {
        edge.neighbor(a.pos) == Some(b.pos)
            && a.path_edge_height(edge)
                .is_some_and(|height| b.path_edge_height(edge.opposite()) == Some(height))
    })
}
