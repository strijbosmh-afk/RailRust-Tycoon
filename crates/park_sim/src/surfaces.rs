//! Stable identities and shared occupied-space rules for the overlap proof.

use crate::{MAX_TERRAIN_HEIGHT, Park, TerrainHeightError, TilePos};

/// A persistent object identity. Removed identities are never allocated again.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(pub u64);

/// The selected simulation object, independent of camera view and visibility.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SurfaceId {
    Terrain(TilePos),
    Object(ObjectId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectKind {
    Path,
    Track,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Axis {
    X,
    Y,
}

/// A proof object occupies one tile and the half-open height range
/// `[height, height + clearance)`. Height is a simulation unit, not render depth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceObject {
    pub id: ObjectId,
    pub pos: TilePos,
    pub kind: ObjectKind,
    pub axis: Axis,
    pub height: u8,
    pub clearance: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementError {
    OutOfBounds { pos: TilePos },
    EmptyClearance,
    HeightOutOfRange { height: u8, clearance: u8, max: u8 },
    TerrainIntersection { pos: TilePos, terrain_height: u8 },
    Occupied { pos: TilePos, object: ObjectId },
    IdExhausted,
    FixtureMapSize { width: u32, height: u32 },
}

impl std::fmt::Display for PlacementError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds { pos } => write!(
                formatter,
                "object tile ({}, {}) is outside the park",
                pos.x, pos.y
            ),
            Self::EmptyClearance => write!(formatter, "object clearance must be positive"),
            Self::HeightOutOfRange {
                height,
                clearance,
                max,
            } => write!(
                formatter,
                "object height {height} plus clearance {clearance} exceeds {max} steps"
            ),
            Self::TerrainIntersection {
                pos,
                terrain_height,
            } => write!(
                formatter,
                "object at ({}, {}) intersects terrain at height {terrain_height}",
                pos.x, pos.y
            ),
            Self::Occupied { pos, object } => write!(
                formatter,
                "object at ({}, {}) overlaps object {}",
                pos.x, pos.y, object.0
            ),
            Self::IdExhausted => {
                write!(formatter, "no more stable object identities are available")
            }
            Self::FixtureMapSize { width, height } => write!(
                formatter,
                "overlap fixture requires a 32 by 32 park, got {width} by {height}"
            ),
        }
    }
}

impl std::error::Error for PlacementError {}

impl Park {
    pub fn objects(&self) -> &[SurfaceObject] {
        &self.objects
    }

    pub fn object(&self, id: ObjectId) -> Option<&SurfaceObject> {
        self.objects.iter().find(|object| object.id == id)
    }

    pub fn surface_position(&self, surface: SurfaceId) -> Option<TilePos> {
        match surface {
            SurfaceId::Terrain(pos) => self.tile(pos).map(|_| pos),
            SurfaceId::Object(id) => self.object(id).map(|object| object.pos),
        }
    }

    pub fn surface_height(&self, surface: SurfaceId) -> Option<u8> {
        match surface {
            SurfaceId::Terrain(pos) => self.tile(pos).map(|tile| tile.height),
            SurfaceId::Object(id) => self.object(id).map(|object| object.height),
        }
    }

    /// Missing identities are not underground. Buried track becomes exposed
    /// when terrain is lowered without moving or replacing the track itself.
    pub fn is_underground(&self, id: ObjectId) -> bool {
        self.object(id).is_some_and(|object| {
            self.tile(object.pos).is_some_and(|tile| {
                object.kind == ObjectKind::Track
                    && u16::from(object.height) + u16::from(object.clearance)
                        <= u16::from(tile.height)
            })
        })
    }

    /// Highest selectable surface, excluding the empty clearance above it.
    pub fn max_surface_height(&self) -> u8 {
        self.objects
            .iter()
            .map(|object| object.height)
            .fold(self.max_terrain_height(), u8::max)
    }

    /// Validate the complete placement before allocating an ID or changing state.
    /// Paths must lie on/above terrain. Track can also be completely buried;
    /// its occupied range must never straddle the terrain surface.
    pub fn place_object(
        &mut self,
        pos: TilePos,
        kind: ObjectKind,
        axis: Axis,
        height: u8,
        clearance: u8,
    ) -> Result<ObjectId, PlacementError> {
        let tile = self.tile(pos).ok_or(PlacementError::OutOfBounds { pos })?;
        if clearance == 0 {
            return Err(PlacementError::EmptyClearance);
        }
        let top = u16::from(height) + u16::from(clearance);
        if top > u16::from(MAX_TERRAIN_HEIGHT) {
            return Err(PlacementError::HeightOutOfRange {
                height,
                clearance,
                max: MAX_TERRAIN_HEIGHT,
            });
        }
        if !terrain_allows(kind, height, clearance, tile.height) {
            return Err(PlacementError::TerrainIntersection {
                pos,
                terrain_height: tile.height,
            });
        }
        if let Some(object) = self.objects.iter().find(|object| {
            object.pos == pos
                && u16::from(height) < u16::from(object.height) + u16::from(object.clearance)
                && u16::from(object.height) < top
        }) {
            return Err(PlacementError::Occupied {
                pos,
                object: object.id,
            });
        }
        let next_id = self
            .next_object_id
            .checked_add(1)
            .ok_or(PlacementError::IdExhausted)?;
        let id = ObjectId(self.next_object_id);
        self.objects.push(SurfaceObject {
            id,
            pos,
            kind,
            axis,
            height,
            clearance,
        });
        self.next_object_id = next_id;
        Ok(id)
    }

    /// Remove only the requested object, retaining the order and IDs of others.
    pub fn remove_object(&mut self, id: ObjectId) -> Option<SurfaceObject> {
        let index = self.objects.iter().position(|object| object.id == id)?;
        let removed = self.objects.remove(index);
        if removed.kind == ObjectKind::Path {
            self.reconcile_guests_after_removal(removed);
        }
        Some(removed)
    }

    /// Install a deterministic 21-object overlap scene as one atomic operation.
    /// Repeating installation preserves matching objects and their identities.
    /// Other existing objects are retained; any collision rejects the whole edit.
    pub fn install_overlap_fixture(&mut self) -> Result<(), PlacementError> {
        if (self.width(), self.height()) != (32, 32) {
            return Err(PlacementError::FixtureMapSize {
                width: self.width(),
                height: self.height(),
            });
        }
        let mut candidate = self.clone();
        let path = (8..=19).map(|x| (TilePos { x, y: 16 }, ObjectKind::Path, Axis::X, 8));
        let track = (12..=20).map(|y| (TilePos { x: 13, y }, ObjectKind::Track, Axis::Y, 0));
        for (pos, kind, axis, height) in path.chain(track) {
            if !candidate.objects.iter().any(|object| {
                object.pos == pos
                    && object.kind == kind
                    && object.axis == axis
                    && object.height == height
                    && object.clearance == 2
            }) {
                candidate.place_object(pos, kind, axis, height, 2)?;
            }
        }
        *self = candidate;
        Ok(())
    }

    pub(crate) fn validate_terrain_clearance(
        &self,
        pos: TilePos,
        height: u8,
    ) -> Result<(), TerrainHeightError> {
        for object in self.objects.iter().filter(|object| object.pos == pos) {
            if !terrain_allows(object.kind, object.height, object.clearance, height) {
                return Err(TerrainHeightError::ObjectIntersection {
                    pos,
                    object: object.id,
                });
            }
        }
        Ok(())
    }
}

fn terrain_allows(kind: ObjectKind, height: u8, clearance: u8, terrain: u8) -> bool {
    height >= terrain
        || (kind == ObjectKind::Track
            && u16::from(height) + u16::from(clearance) <= u16::from(terrain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TerrainProfile};

    #[test]
    fn touching_ranges_stack_but_overlaps_reject_without_allocating_ids() {
        let mut park = Park::new(4, 4);
        let pos = TilePos { x: 1, y: 1 };
        let first = park
            .place_object(pos, ObjectKind::Path, Axis::X, 0, 2)
            .unwrap();
        let upper = park
            .place_object(pos, ObjectKind::Track, Axis::Y, 2, 3)
            .unwrap();
        let snapshot = park.clone();
        for (height, clearance) in [(0, 1), (1, 2), (3, 1), (1, 5)] {
            assert!(matches!(
                park.place_object(pos, ObjectKind::Path, Axis::Y, height, clearance),
                Err(PlacementError::Occupied { .. })
            ));
            assert_eq!(park, snapshot);
        }
        assert_eq!(park.remove_object(first).unwrap().id, first);
        let replacement = park
            .place_object(pos, ObjectKind::Path, Axis::X, 0, 2)
            .unwrap();
        assert!(replacement.0 > upper.0);
        assert_eq!(park.object(upper), snapshot.object(upper));
        assert_eq!(park.max_surface_height(), 2);
    }

    #[test]
    fn terrain_accepts_surface_and_buried_track_but_rejects_partial_intersections() {
        let mut park = Park::new(4, 4);
        let pos = TilePos { x: 1, y: 1 };
        park.set_terrain_height(pos, 4).unwrap();
        let track = park
            .place_object(pos, ObjectKind::Track, Axis::X, 2, 2)
            .unwrap();
        let path = park
            .place_object(pos, ObjectKind::Path, Axis::Y, 4, 2)
            .unwrap();
        assert!(park.is_underground(track));
        assert!(!park.is_underground(path));
        let snapshot = park.clone();
        for (kind, height, clearance) in [
            (ObjectKind::Path, 0, 2),
            (ObjectKind::Track, 3, 2),
            (ObjectKind::Track, 0, 5),
        ] {
            assert!(matches!(
                park.place_object(pos, kind, Axis::X, height, clearance),
                Err(PlacementError::TerrainIntersection { .. })
            ));
            assert_eq!(park, snapshot);
        }
        for height in [3, 5] {
            assert!(matches!(
                park.set_terrain_height(pos, height),
                Err(TerrainHeightError::ObjectIntersection { .. })
            ));
            assert_eq!(park, snapshot);
        }
        park.set_terrain_height(pos, 2).unwrap();
        assert!(!park.is_underground(track));
        assert_eq!(park.object(track), snapshot.object(track));
    }

    #[test]
    fn invalid_placements_preserve_state_including_id_allocation() {
        let mut park = Park::new(4, 4);
        let pos = TilePos { x: 1, y: 1 };
        let snapshot = park.clone();
        for outside in [
            TilePos { x: 4, y: 0 },
            TilePos { x: 0, y: 4 },
            TilePos {
                x: u32::MAX,
                y: u32::MAX,
            },
        ] {
            assert_eq!(
                park.place_object(outside, ObjectKind::Path, Axis::X, 0, 2),
                Err(PlacementError::OutOfBounds { pos: outside })
            );
            assert_eq!(park, snapshot);
        }
        assert_eq!(
            park.place_object(pos, ObjectKind::Path, Axis::X, 0, 0),
            Err(PlacementError::EmptyClearance)
        );
        for (height, clearance) in [(32, 1), (31, 2), (255, 255)] {
            assert!(matches!(
                park.place_object(pos, ObjectKind::Path, Axis::X, height, clearance),
                Err(PlacementError::HeightOutOfRange { .. })
            ));
            assert_eq!(park, snapshot);
        }
        let id = park
            .place_object(pos, ObjectKind::Path, Axis::X, 30, 2)
            .unwrap();
        assert_eq!(id, ObjectId(1));
        assert_eq!(park.max_surface_height(), 30);
        park.next_object_id = u64::MAX;
        let snapshot = park.clone();
        assert_eq!(
            park.place_object(TilePos { x: 0, y: 0 }, ObjectKind::Path, Axis::X, 0, 2),
            Err(PlacementError::IdExhausted)
        );
        assert_eq!(park, snapshot);
    }

    #[test]
    fn missing_and_removed_surface_references_never_resolve() {
        let mut park = Park::new(4, 4);
        let pos = TilePos { x: 1, y: 1 };
        let id = park
            .place_object(pos, ObjectKind::Path, Axis::X, 4, 2)
            .unwrap();
        assert_eq!(park.surface_position(SurfaceId::Object(id)), Some(pos));
        assert_eq!(park.surface_height(SurfaceId::Object(id)), Some(4));
        assert_eq!(park.surface_position(SurfaceId::Terrain(pos)), Some(pos));
        assert_eq!(park.surface_height(SurfaceId::Terrain(pos)), Some(0));
        park.remove_object(id).unwrap();
        let snapshot = park.clone();
        for missing in [id, ObjectId(0), ObjectId(u64::MAX)] {
            assert_eq!(park.object(missing), None);
            assert_eq!(park.surface_position(SurfaceId::Object(missing)), None);
            assert_eq!(park.surface_height(SurfaceId::Object(missing)), None);
            assert!(!park.is_underground(missing));
            assert_eq!(park.remove_object(missing), None);
            assert_eq!(park, snapshot);
        }
        let outside = SurfaceId::Terrain(TilePos { x: 4, y: 0 });
        assert_eq!(park.surface_position(outside), None);
        assert_eq!(park.surface_height(outside), None);
    }

    #[test]
    fn fixture_is_repeatable_and_profiles_preserve_ids_heights_and_time() {
        let mut park = Park::new(32, 32);
        park.advance(Speed::Fastest);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_overlap_fixture().unwrap();
        assert_eq!(park.objects().len(), 21);
        assert_eq!(park.max_surface_height(), 8);
        let snapshot = park.clone();
        park.install_overlap_fixture().unwrap();
        assert_eq!(park, snapshot);
        let track_ids: Vec<_> = park
            .objects()
            .iter()
            .filter(|object| object.kind == ObjectKind::Track)
            .map(|object| object.id)
            .collect();
        assert_eq!(track_ids.len(), 9);
        assert!(track_ids.iter().all(|id| park.is_underground(*id)));
        park.try_apply_terrain_profile(TerrainProfile::Flat)
            .unwrap();
        assert_eq!(park.objects(), snapshot.objects());
        assert_eq!(park.ticks(), snapshot.ticks());
        assert!(track_ids.iter().all(|id| !park.is_underground(*id)));
        park.install_overlap_fixture().unwrap();
        park.try_apply_terrain_profile(TerrainProfile::Terraced)
            .unwrap();
        assert_eq!(park, snapshot);
        let mut second = Park::new(32, 32);
        second.install_overlap_fixture().unwrap();
        assert_eq!(second.objects(), park.objects());
    }

    #[test]
    fn failed_fixture_installation_rolls_back_partial_work_and_ids() {
        let mut park = Park::new(32, 32);
        park.place_object(TilePos { x: 19, y: 16 }, ObjectKind::Track, Axis::Y, 8, 2)
            .unwrap();
        let snapshot = park.clone();
        assert!(matches!(
            park.install_overlap_fixture(),
            Err(PlacementError::Occupied { .. })
        ));
        assert_eq!(park, snapshot);
        let mut small = Park::new(20, 32);
        let snapshot = small.clone();
        assert_eq!(
            small.install_overlap_fixture(),
            Err(PlacementError::FixtureMapSize {
                width: 20,
                height: 32
            })
        );
        assert_eq!(small, snapshot);
    }

    #[test]
    fn conflicting_profile_rejects_every_height_change_atomically() {
        let mut park = Park::new(32, 32);
        park.place_object(TilePos { x: 12, y: 16 }, ObjectKind::Path, Axis::X, 0, 2)
            .unwrap();
        let snapshot = park.clone();
        assert!(matches!(
            park.try_apply_terrain_profile(TerrainProfile::Terraced),
            Err(TerrainHeightError::ObjectIntersection { .. })
        ));
        assert_eq!(park, snapshot);
    }
}
