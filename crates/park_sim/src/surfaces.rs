//! Stable identities and shared occupied-space rules for the overlap proof.

use crate::{MAX_TERRAIN_HEIGHT, Park, PathSlope, TerrainHeightError, TilePos};

/// Reserved height above the deck of a player-built level path.
pub const PATH_CLEARANCE: u8 = 2;
pub const MAX_PATH_HEIGHT: u8 = MAX_TERRAIN_HEIGHT - PATH_CLEARANCE;

/// A persistent object identity. Removed identities are never allocated again.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct ObjectId(pub u64);

/// The selected simulation object, independent of camera view and visibility.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SurfaceId {
    Terrain(TilePos),
    Object(ObjectId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ObjectKind {
    Path,
    Track,
    RideEntrance,
    RideExit,
    Service,
    Carousel,
    SceneryTree,
    FlowerBed,
}

pub const MAX_SCENERY_OBJECTS: usize = 1024;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneryKind {
    Tree,
    FlowerBed,
}
impl SceneryKind {
    pub const fn object_kind(self) -> ObjectKind {
        match self {
            Self::Tree => ObjectKind::SceneryTree,
            Self::FlowerBed => ObjectKind::FlowerBed,
        }
    }
    pub const fn clearance(self) -> u8 {
        match self {
            Self::Tree => 3,
            Self::FlowerBed => 1,
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::Tree => "Tree",
            Self::FlowerBed => "Flower bed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Axis {
    X,
    Y,
}

/// A proof object occupies one tile and the half-open height range
/// `[height, height + clearance)`. Height is a simulation unit, not render depth.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceObject {
    pub id: ObjectId,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub pos: TilePos,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub kind: ObjectKind,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub axis: Axis,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub path_slope: PathSlope,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub queue_owner: Option<crate::RideId>,
    pub height: u8,
    pub clearance: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementError {
    OutOfBounds {
        pos: TilePos,
    },
    LandNotOwned {
        pos: TilePos,
    },
    WaterNeedsBridge {
        pos: TilePos,
    },
    EmptyClearance,
    HeightOutOfRange {
        height: u8,
        clearance: u8,
        max: u8,
    },
    TerrainIntersection {
        pos: TilePos,
        terrain_height: u8,
    },
    Occupied {
        pos: TilePos,
        object: ObjectId,
    },
    IdExhausted,
    CapacityExceeded {
        resource: &'static str,
        max: usize,
    },
    FixtureMapSize {
        width: u32,
        height: u32,
    },
    EntranceRequiresRide,
    ServiceRequiresDefinition,
    CarouselRequiresDefinition,
    SceneryRequiresTool,
    SceneryRequiresLand,
    InsufficientFunds {
        required_cents: u64,
        available_cents: u64,
    },
}

impl std::fmt::Display for PlacementError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientFunds {
                required_cents,
                available_cents,
            } => write!(
                formatter,
                "Construction costs ${}.{:02}; park cash is ${}.{:02}",
                required_cents / 100,
                required_cents % 100,
                available_cents / 100,
                available_cents % 100
            ),
            Self::OutOfBounds { pos } => write!(
                formatter,
                "object tile ({}, {}) is outside the park",
                pos.x, pos.y
            ),
            Self::LandNotOwned { pos } => {
                write!(formatter, "tile ({}, {}) is not owned", pos.x, pos.y)
            }
            Self::WaterNeedsBridge { pos } => {
                write!(
                    formatter,
                    "tile ({}, {}) is water; build above it",
                    pos.x, pos.y
                )
            }
            Self::EmptyClearance => write!(formatter, "object clearance must be positive"),
            Self::CarouselRequiresDefinition => write!(
                formatter,
                "build the complete three by three carousel footprint"
            ),
            Self::SceneryRequiresTool => write!(formatter, "use the scenery construction tool"),
            Self::SceneryRequiresLand => write!(formatter, "scenery needs grass or sand"),
            Self::ServiceRequiresDefinition => write!(
                formatter,
                "a service needs a kind and an outward-facing counter"
            ),
            Self::EntranceRequiresRide => write!(
                formatter,
                "an entrance needs a ride and a station-facing direction"
            ),
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
            Self::CapacityExceeded { resource, max } => {
                write!(
                    formatter,
                    "the park already has the maximum of {max} {resource}"
                )
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

    /// Missing identities are not underground. Buried track and paths become
    /// exposed when terrain is lowered without moving or replacing them.
    pub fn is_underground(&self, id: ObjectId) -> bool {
        self.object(id).is_some_and(|object| {
            self.tile(object.pos).is_some_and(|tile| {
                matches!(object.kind, ObjectKind::Track | ObjectKind::Path)
                    && u16::from(object.height) + u16::from(object.clearance)
                        <= u16::from(tile.height)
            })
        })
    }

    /// Highest selectable surface, excluding the empty clearance above it.
    pub fn max_surface_height(&self) -> u8 {
        let object_height = self
            .objects
            .iter()
            .map(|object| object.height + u8::from(object.path_slope != PathSlope::Flat))
            .fold(self.max_terrain_height(), u8::max);
        self.circuit
            .as_ref()
            .into_iter()
            .flat_map(|circuit| &circuit.pieces)
            .flat_map(|piece| &piece.samples)
            .map(|point| (point.height / crate::TRACK_UNITS) as u8)
            .fold(object_height, u8::max)
    }

    /// Validate the complete placement before allocating an ID or changing state.
    /// Paths and track can lie on/above terrain or be completely buried;
    /// their occupied ranges must never straddle the terrain surface.
    pub fn place_object(
        &mut self,
        pos: TilePos,
        kind: ObjectKind,
        axis: Axis,
        height: u8,
        clearance: u8,
    ) -> Result<ObjectId, PlacementError> {
        if kind == ObjectKind::Carousel {
            return Err(PlacementError::CarouselRequiresDefinition);
        }
        if matches!(kind, ObjectKind::SceneryTree | ObjectKind::FlowerBed) {
            return Err(PlacementError::SceneryRequiresTool);
        }
        if kind == ObjectKind::Service {
            return Err(PlacementError::ServiceRequiresDefinition);
        }
        if matches!(kind, ObjectKind::RideEntrance | ObjectKind::RideExit) {
            return Err(PlacementError::EntranceRequiresRide);
        }
        let next_id = self.validate_object_placement(pos, kind, height, clearance)?;
        let id = self.insert_validated_object(pos, kind, axis, height, clearance, next_id);
        self.reconcile_ride_visits();
        Ok(id)
    }

    fn validate_scenery_placement(
        &self,
        pos: TilePos,
        kind: SceneryKind,
    ) -> Result<u64, PlacementError> {
        let tile = self.tile(pos).ok_or(PlacementError::OutOfBounds { pos })?;
        if !tile.owned {
            return Err(PlacementError::LandNotOwned { pos });
        }
        if tile.terrain == crate::Terrain::Water {
            return Err(PlacementError::SceneryRequiresLand);
        }
        if self.objects.len() >= crate::saves::MAX_OBJECTS {
            return Err(PlacementError::CapacityExceeded {
                resource: "objects",
                max: crate::saves::MAX_OBJECTS,
            });
        }
        if self
            .objects
            .iter()
            .filter(|object| matches!(object.kind, ObjectKind::SceneryTree | ObjectKind::FlowerBed))
            .count()
            >= MAX_SCENERY_OBJECTS
        {
            return Err(PlacementError::CapacityExceeded {
                resource: "scenery",
                max: MAX_SCENERY_OBJECTS,
            });
        }
        self.validate_object_placement(pos, kind.object_kind(), tile.height, kind.clearance())
    }

    pub fn validate_scenery(&self, pos: TilePos, kind: SceneryKind) -> Result<(), PlacementError> {
        self.validate_scenery_placement(pos, kind).map(|_| ())
    }

    pub fn place_scenery(
        &mut self,
        pos: TilePos,
        kind: SceneryKind,
    ) -> Result<ObjectId, PlacementError> {
        let next_id = self.validate_scenery_placement(pos, kind)?;
        let height = self.tile(pos).expect("validated tile").height;
        Ok(self.insert_validated_object(
            pos,
            kind.object_kind(),
            Axis::X,
            height,
            kind.clearance(),
            next_id,
        ))
    }

    /// Preview the exact player path placement without changing authoritative
    /// state, allocating an identity, or warming/invalidating connectivity.
    pub fn validate_path(&self, pos: TilePos, height: u8) -> Result<(), PlacementError> {
        self.validate_sloped_path(pos, height, PathSlope::Flat)
    }

    /// Place one level path with the same rules used by its read-only preview.
    /// Player construction stays within the save format's path/object limits;
    /// generic fixture construction intentionally retains its existing limits.
    pub fn place_path(&mut self, pos: TilePos, height: u8) -> Result<ObjectId, PlacementError> {
        self.place_sloped_path(pos, height, PathSlope::Flat)
    }

    pub fn validate_sloped_path(
        &self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
    ) -> Result<(), PlacementError> {
        self.validate_path_placement(pos, height, slope).map(|_| ())
    }

    /// The lower edge is `height`; slopes reserve the complete column up to high edge +2.
    pub fn place_sloped_path(
        &mut self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
    ) -> Result<ObjectId, PlacementError> {
        let next_id = self.validate_path_placement(pos, height, slope)?;
        let id = self.insert_validated_object(
            pos,
            ObjectKind::Path,
            slope.axis(),
            height,
            slope.clearance(),
            next_id,
        );
        self.objects
            .last_mut()
            .expect("just inserted path")
            .path_slope = slope;
        self.reconcile_ride_visits();
        Ok(id)
    }

    pub(crate) fn validate_path_placement(
        &self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
    ) -> Result<u64, PlacementError> {
        let next_id =
            self.validate_object_placement(pos, ObjectKind::Path, height, slope.clearance())?;
        if self.objects.len() >= crate::saves::MAX_OBJECTS {
            return Err(PlacementError::CapacityExceeded {
                resource: "objects",
                max: crate::saves::MAX_OBJECTS,
            });
        }
        if self
            .objects
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .count()
            >= crate::saves::MAX_PATHS
        {
            return Err(PlacementError::CapacityExceeded {
                resource: "paths",
                max: crate::saves::MAX_PATHS,
            });
        }
        Ok(next_id)
    }

    /// Return the next allocator value only after all geometric rules pass.
    /// Both generic construction and the player preview use this validation.
    pub(crate) fn validate_object_placement(
        &self,
        pos: TilePos,
        kind: ObjectKind,
        height: u8,
        clearance: u8,
    ) -> Result<u64, PlacementError> {
        let tile = self.tile(pos).ok_or(PlacementError::OutOfBounds { pos })?;
        if !tile.owned {
            return Err(PlacementError::LandNotOwned { pos });
        }
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
        if tile.terrain == crate::Terrain::Water && height < 2 {
            return Err(PlacementError::WaterNeedsBridge { pos });
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
        self.next_object_id
            .checked_add(1)
            .ok_or(PlacementError::IdExhausted)
    }

    fn insert_validated_object(
        &mut self,
        pos: TilePos,
        kind: ObjectKind,
        axis: Axis,
        height: u8,
        clearance: u8,
        next_id: u64,
    ) -> ObjectId {
        self.insert_surface_object(
            SurfaceObject {
                id: ObjectId(0),
                pos,
                kind,
                axis,
                path_slope: PathSlope::Flat,
                queue_owner: None,
                height,
                clearance,
            },
            next_id,
        )
    }

    pub(crate) fn insert_surface_object(
        &mut self,
        mut object: SurfaceObject,
        next_id: u64,
    ) -> ObjectId {
        let id = ObjectId(self.next_object_id);
        object.id = id;
        let public = object.is_public_path();
        self.objects.push(object);
        self.next_object_id = next_id;
        if public {
            self.path_cache.invalidate();
        }
        id
    }

    /// Remove only the requested object, retaining the order and IDs of others.
    pub fn remove_object(&mut self, id: ObjectId) -> Option<SurfaceObject> {
        if self.object_removal_block_reason(id).is_some() {
            return None;
        }
        let index = self.objects.iter().position(|object| object.id == id)?;
        let service_refund = self.service_by_object(id).map(|service| {
            (
                service.id,
                self.service_demolition_quote(service.id)
                    .expect("active service body"),
            )
        });
        if self.objects[index].kind == ObjectKind::Carousel {
            let removed = self.objects[index];
            self.remove_carousel(self.object_ride(id)?).ok()?;
            return Some(removed);
        }
        let removed = self.objects.remove(index);
        if self.visitors.entry == Some(id) {
            self.visitors.entry = None;
        }
        self.refund_removed_track(id);
        if removed.is_public_path() {
            self.path_cache.invalidate();
        }
        if removed.kind == ObjectKind::Path {
            self.reconcile_guests_after_removal(removed);
        }
        self.reconcile_mechanic_after_removal(id);
        self.reconcile_grounds_after_removal(id);
        self.reconcile_train_after_removal(id);
        for ride in self.all_rides_mut() {
            if ride.entrance.is_some_and(|gate| gate.object == id) {
                ride.entrance = None;
            }
            if ride.exit.is_some_and(|gate| gate.object == id) {
                ride.exit = None;
            }
        }
        self.reconcile_ride_visits();
        for service in &mut self.services {
            if service.object == Some(id) {
                service.object = None;
                service.open = false;
                service.demolition_refund_cents =
                    service_refund.expect("removed service has a quote").1;
            }
        }
        self.reconcile_service_visits();
        if let Some((service, refund)) = service_refund {
            self.record_cash(
                crate::FinanceKind::ServiceDemolitionRefund,
                Some(service),
                refund,
            );
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
                    && object.queue_owner.is_none()
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

pub(crate) fn terrain_allows(kind: ObjectKind, height: u8, clearance: u8, terrain: u8) -> bool {
    if matches!(kind, ObjectKind::SceneryTree | ObjectKind::FlowerBed) {
        return height == terrain;
    }
    height >= terrain
        || (matches!(kind, ObjectKind::Track | ObjectKind::Path)
            && u16::from(height) + u16::from(clearance) <= u16::from(terrain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Guest, GuestId, GuestState, Speed, TerrainProfile};

    #[test]
    fn scenery_is_grounded_bounded_saved_and_removable() {
        let mut park = Park::new(32, 32);
        let tree_pos = TilePos { x: 3, y: 3 };
        let flower_pos = TilePos { x: 4, y: 3 };
        park.set_terrain_height(tree_pos, 8).unwrap();
        park.set_terrain_height(flower_pos, 8).unwrap();
        let before = park.clone();
        assert_eq!(park.validate_scenery(tree_pos, SceneryKind::Tree), Ok(()));
        assert_eq!(park, before, "preview must not mutate the park");
        let tree = park.place_scenery(tree_pos, SceneryKind::Tree).unwrap();
        let flower = park
            .place_scenery(flower_pos, SceneryKind::FlowerBed)
            .unwrap();
        assert_eq!(park.object(tree).unwrap().kind, ObjectKind::SceneryTree);
        assert_eq!(park.object(flower).unwrap().kind, ObjectKind::FlowerBed);
        assert!(matches!(
            park.validate_scenery(tree_pos, SceneryKind::FlowerBed),
            Err(PlacementError::Occupied { .. })
        ));
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
        assert!(matches!(
            park.set_terrain_height(tree_pos, park.tile(tree_pos).unwrap().height + 1),
            Err(crate::TerrainHeightError::ObjectIntersection { .. })
        ));
        assert!(matches!(
            park.set_terrain_height(tree_pos, park.tile(tree_pos).unwrap().height - 1),
            Err(crate::TerrainHeightError::ObjectIntersection { .. })
        ));
        assert!(park.remove_object(tree).is_some());
        assert_eq!(
            park.validate_scenery(tree_pos, SceneryKind::FlowerBed),
            Ok(())
        );
        assert!(
            park.place_scenery(tree_pos, SceneryKind::FlowerBed)
                .unwrap()
                .0
                > flower.0
        );
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );

        let water_pos = TilePos { x: 5, y: 3 };
        park.tiles[(water_pos.y * park.width + water_pos.x) as usize].terrain =
            crate::Terrain::Water;
        assert_eq!(
            park.validate_scenery(water_pos, SceneryKind::Tree),
            Err(PlacementError::SceneryRequiresLand)
        );
    }

    #[test]
    fn scenery_capacity_rejects_next_placement_without_allocating_an_id() {
        let mut park = Park::new(64, 64);
        let land: Vec<_> = (0..park.height)
            .flat_map(|y| (0..park.width).map(move |x| TilePos { x, y }))
            .filter(|pos| park.validate_scenery(*pos, SceneryKind::FlowerBed).is_ok())
            .take(MAX_SCENERY_OBJECTS + 1)
            .collect();
        assert_eq!(land.len(), MAX_SCENERY_OBJECTS + 1);
        for pos in &land[..MAX_SCENERY_OBJECTS] {
            park.place_scenery(*pos, SceneryKind::FlowerBed).unwrap();
        }
        let snapshot = park.clone();
        assert_eq!(
            park.place_scenery(land[MAX_SCENERY_OBJECTS], SceneryKind::FlowerBed),
            Err(PlacementError::CapacityExceeded {
                resource: "scenery",
                max: MAX_SCENERY_OBJECTS
            })
        );
        assert_eq!(park, snapshot);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn large_park_builds_past_old_path_and_scenery_limits_then_reloads_and_demolishes() {
        let mut park = Park::new(128, 128);
        let paths: Vec<_> = (0..1025)
            .map(|index| {
                park.place_path(
                    TilePos {
                        x: index % 128,
                        y: index / 128,
                    },
                    2,
                )
                .unwrap()
            })
            .collect();
        let flowers: Vec<_> = (0..513)
            .map(|index| {
                park.place_scenery(
                    TilePos {
                        x: index % 128,
                        y: 100 + index / 128,
                    },
                    SceneryKind::FlowerBed,
                )
                .unwrap()
            })
            .collect();
        assert_eq!(park.objects().len(), 1538);
        assert!(park.public_path_route(paths[0], paths[1024]).is_some());
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(loaded, park);
        assert!(loaded.public_path_route(paths[0], paths[1024]).is_some());
        assert!(loaded.remove_object(paths[0]).is_some());
        assert!(loaded.remove_object(flowers[0]).is_some());
        assert_eq!(loaded.objects().len(), 1536);
        assert_eq!(
            Park::from_save_bytes(&loaded.to_save_bytes().unwrap()).unwrap(),
            loaded
        );
    }

    #[test]
    fn player_path_preview_and_commit_share_rules_without_preview_side_effects() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(71).unwrap();
        park.advance(Speed::Normal);
        park.set_terrain_height(TilePos { x: 0, y: 0 }, 4).unwrap();
        let cases = [
            (TilePos { x: 32, y: 0 }, 8, false),
            (TilePos { x: 0, y: 0 }, 3, false),
            (TilePos { x: 13, y: 16 }, 8, false),
            (TilePos { x: 13, y: 16 }, 0, false),
            (TilePos { x: 0, y: 0 }, MAX_PATH_HEIGHT + 1, false),
            (TilePos { x: 0, y: 0 }, u8::MAX, false),
            (TilePos { x: 0, y: 0 }, 4, true),
            (TilePos { x: 13, y: 16 }, 2, true),
            (TilePos { x: 13, y: 16 }, 6, true),
            (TilePos { x: 0, y: 0 }, MAX_PATH_HEIGHT, true),
        ];
        for (pos, height, allowed) in cases {
            let mut candidate = park.clone();
            let cache = candidate.path_cache_stats();
            let preview = candidate.validate_path(pos, height);
            assert_eq!(preview.is_ok(), allowed, "{pos:?} at {height}");
            assert_eq!(candidate, park);
            assert_eq!(candidate.path_cache_stats(), cache);
            let committed = candidate.place_path(pos, height);
            assert_eq!(committed.map(|_| ()), preview);
            if let Ok(id) = committed {
                let object = candidate.object(id).unwrap();
                assert_eq!((object.kind, object.axis), (ObjectKind::Path, Axis::X));
                assert_eq!(object.clearance, PATH_CLEARANCE);
                assert_eq!(candidate.ticks(), park.ticks());
                assert_eq!(candidate.guests(), park.guests());
                assert_eq!(
                    candidate.path_cache_stats().invalidations,
                    cache.invalidations + 1
                );
                assert!(!candidate.path_cache_stats().cached);
            } else {
                assert_eq!(candidate, park);
                assert_eq!(candidate.path_cache_stats(), cache);
            }
        }
        park.next_object_id = u64::MAX;
        let snapshot = park.clone();
        let cache = park.path_cache_stats();
        assert_eq!(
            park.validate_path(TilePos { x: 0, y: 0 }, 4),
            Err(PlacementError::IdExhausted)
        );
        assert_eq!(
            park.place_path(TilePos { x: 0, y: 0 }, 4),
            Err(PlacementError::IdExhausted)
        );
        assert_eq!(park, snapshot);
        assert_eq!(park.path_cache_stats(), cache);
    }

    #[test]
    fn player_path_cap_is_shared_with_saves_and_demolition_reopens_capacity() {
        let mut park = Park::new(64, 64);
        for index in 0..crate::saves::MAX_PATHS {
            park.place_path(
                TilePos {
                    x: (index % 64) as u32,
                    y: (index / 64) as u32,
                },
                2,
            )
            .unwrap();
        }
        assert!(park.to_save_bytes().is_ok());
        let pos = TilePos { x: 0, y: 0 };
        let snapshot = park.clone();
        let cache = park.path_cache_stats();
        let error = PlacementError::CapacityExceeded {
            resource: "paths",
            max: crate::saves::MAX_PATHS,
        };
        assert_eq!(park.validate_path(pos, 4), Err(error));
        assert_eq!(park.place_path(pos, 4), Err(error));
        assert_eq!(park, snapshot);
        assert_eq!(park.path_cache_stats(), cache);
        let old = park.objects()[0].id;
        park.remove_object(old).unwrap();
        let replacement = park.place_path(pos, 2).unwrap();
        assert!(replacement.0 > snapshot.objects().last().unwrap().id.0);
        assert!(park.to_save_bytes().is_ok());
        // Explicit diagnostic constructors keep their preexisting freedom.
        park.place_object(pos, ObjectKind::Path, Axis::Y, 0, PATH_CLEARANCE)
            .unwrap();
        assert!(park.to_save_bytes().is_err());
    }

    #[test]
    fn player_path_respects_total_object_capacity_before_allocating() {
        let mut park = Park::new(32, 32);
        for index in 0..crate::saves::MAX_OBJECTS - 1 {
            park.place_object(
                TilePos {
                    x: (index % 32) as u32,
                    y: ((index / 32) % 32) as u32,
                },
                ObjectKind::Track,
                Axis::Y,
                (index / 1024 * 2 + 2) as u8,
                2,
            )
            .unwrap();
        }
        park.place_path(TilePos { x: 31, y: 31 }, 12).unwrap();
        assert!(park.to_save_bytes().is_ok());
        let snapshot = park.clone();
        let error = PlacementError::CapacityExceeded {
            resource: "objects",
            max: crate::saves::MAX_OBJECTS,
        };
        let pos = TilePos { x: 0, y: 0 };
        assert_eq!(park.validate_path(pos, 14), Err(error));
        assert_eq!(park.place_path(pos, 14), Err(error));
        assert_eq!(park, snapshot);
        assert_eq!(park.path_cache_stats(), snapshot.path_cache_stats());
    }

    #[test]
    fn player_bridge_demolition_and_rebuild_recover_a_guest_with_fresh_ids() {
        let mut park = Park::new(8, 8);
        let start = park.place_path(TilePos { x: 1, y: 1 }, 0).unwrap();
        let bridge = park.place_path(TilePos { x: 2, y: 1 }, 0).unwrap();
        let end = park.place_path(TilePos { x: 3, y: 1 }, 0).unwrap();
        park.crowd_seed = Some(99);
        park.crowd_rng = 99;
        park.next_guest_id = 2;
        park.guests.push(Guest {
            id: GuestId(1),
            current: Some(start),
            next: None,
            destination: None,
            anchor: TilePos { x: 1, y: 1 },
            height: 0,
            anchor_slope: PathSlope::Flat,
            progress: 0,
            state: GuestState::Waiting,
            arrivals: 0,
            rides_completed: 0,
            ride_history: Vec::new(),
            wallet_cents: crate::INITIAL_GUEST_WALLET_CENTS,
            spent_cents: 0,
            service_spent_cents: 0,
            services_used: 0,
            needs: crate::GuestNeeds::default(),
            service_visit: None,
            service_cooldown: 0,
            preferences: crate::GuestPreferences::default(),
            satisfaction: 70,
            thought: crate::GuestThought::Exploring,
            thought_tick: 0,
            ratings_given: 0,
            rating_total: 0,
            last_rating: None,
            ride_visit: None,
            ride_cooldown: 0,
            route: Default::default(),
            wait_ticks: 0,
        });
        park.advance(Speed::Normal);
        assert_eq!(park.guests()[0].next, Some(bridge));
        assert_eq!(park.path_neighbors(start), vec![bridge]);
        let ticks = park.ticks();
        park.remove_object(bridge).unwrap();
        assert_eq!(park.ticks(), ticks);
        assert!(park.path_neighbors(start).is_empty());
        assert!(park.path_neighbors(end).is_empty());
        assert_eq!(park.guests()[0].state, GuestState::Lost);
        assert!(park.guests()[0].next.is_none());
        assert!(park.guests()[0].destination.is_none());
        assert!(park.guests()[0].route.is_empty());
        for _ in 0..60 {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.guests()[0].current, Some(start));
        assert_eq!(park.guests()[0].state, GuestState::Lost);
        let replacement = park.place_path(TilePos { x: 2, y: 1 }, 0).unwrap();
        assert!(replacement.0 > end.0);
        assert_eq!(park.path_neighbors(start), vec![replacement]);
        assert_eq!(park.path_neighbors(end), vec![replacement]);
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        for _ in 0..180 {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
            assert_eq!(park, loaded);
        }
        assert_eq!(park.guests()[0].id, GuestId(1));
        assert!(park.guests()[0].arrivals > 0);
        assert_ne!(park.guests()[0].state, GuestState::Lost);
    }

    #[test]
    fn constructed_and_demolished_paths_preserve_same_tick_save_continuation() {
        for target in ["current", "next", "destination"] {
            let mut park = Park::new(32, 32);
            park.install_crowd_fixture(71).unwrap();
            park.install_train_fixture().unwrap();
            park.advance(Speed::Normal);
            let extension = park.place_path(TilePos { x: 13, y: 21 }, 8).unwrap();
            let guest = &park.guests()[0];
            let removed = match target {
                "current" => guest.current.unwrap(),
                "next" => guest.next.unwrap(),
                _ => guest.destination.unwrap(),
            };
            let removed_object = park.remove_object(removed).unwrap();
            let ticks = park.ticks();
            assert_eq!(park.guests()[0].state, GuestState::Lost);
            assert!(!park.path_neighbors(extension).contains(&removed));
            for guest in park.guests() {
                assert!(
                    !guest
                        .current
                        .into_iter()
                        .chain(guest.next)
                        .chain(guest.destination)
                        .chain(guest.route.iter().copied())
                        .any(|id| id == removed)
                );
            }
            let bytes = park.to_save_bytes().unwrap();
            let mut loaded = Park::from_save_bytes(&bytes).unwrap();
            assert_eq!(park, loaded);
            assert_eq!(bytes, loaded.to_save_bytes().unwrap());
            park.advance(Speed::Paused);
            loaded.advance(Speed::Paused);
            assert_eq!(park.ticks(), ticks);
            assert_eq!(park, loaded);
            for tick in 0..240 {
                if tick == 30 {
                    let id = park
                        .place_path(removed_object.pos, removed_object.height)
                        .unwrap();
                    let loaded_id = loaded
                        .place_path(removed_object.pos, removed_object.height)
                        .unwrap();
                    assert_eq!(id, loaded_id);
                    assert!(id.0 > extension.0);
                }
                if tick == 90 {
                    assert_eq!(
                        park.remove_object(extension),
                        loaded.remove_object(extension)
                    );
                }
                if tick == 120 {
                    assert_eq!(
                        park.place_path(TilePos { x: 13, y: 21 }, 8),
                        loaded.place_path(TilePos { x: 13, y: 21 }, 8)
                    );
                }
                park.advance(Speed::Fast);
                loaded.advance(Speed::Fast);
                assert_eq!(park, loaded, "{target}, continuation tick {tick}");
            }
            assert_eq!(
                park.to_save_bytes().unwrap(),
                loaded.to_save_bytes().unwrap()
            );
        }
    }

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
            (ObjectKind::Path, 3, 2),
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
    fn buried_public_path_connects_to_surface_and_survives_save_and_exposure() {
        let mut park = Park::new(4, 4);
        let outside = TilePos { x: 0, y: 1 };
        let tunnel = TilePos { x: 1, y: 1 };
        park.set_terrain_height(tunnel, 4).unwrap();
        let approach = park.place_path(outside, 0).unwrap();
        let underground = park.place_path(tunnel, 0).unwrap();
        assert!(park.is_underground(underground));
        assert_eq!(
            park.public_path_route(approach, underground),
            Some(std::collections::VecDeque::from([underground]))
        );
        assert!(matches!(
            park.place_path(tunnel, 3),
            Err(PlacementError::TerrainIntersection { .. })
        ));
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(loaded, park);
        loaded.set_terrain_height(tunnel, 0).unwrap();
        assert!(!loaded.is_underground(underground));
        assert_eq!(
            loaded.public_path_route(approach, underground),
            Some(std::collections::VecDeque::from([underground]))
        );
        assert_eq!(
            Park::from_save_bytes(&loaded.to_save_bytes().unwrap()).unwrap(),
            loaded
        );
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
        park.place_object(TilePos { x: 12, y: 16 }, ObjectKind::Path, Axis::X, 5, 2)
            .unwrap();
        let snapshot = park.clone();
        assert!(matches!(
            park.try_apply_terrain_profile(TerrainProfile::Terraced),
            Err(TerrainHeightError::ObjectIntersection { .. })
        ));
        assert_eq!(park, snapshot);
    }
}
