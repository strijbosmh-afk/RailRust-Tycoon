//! Engine-independent park state and a deterministic fixed-step simulation clock.
//!
//! Rendering, input, and wall-clock time belong to the application. Keeping those
//! concerns outside this crate makes the simulation usable in headless tests.

mod access_planning;
mod admission;
mod carousel;
mod crowds;
pub mod diagnostics;
mod economy;
mod experience;
mod finances;
mod ride_cashflow;
pub use ride_cashflow::RideCashflow;
mod journeys;
mod path_geometry;
mod ride_access;
#[cfg(test)]
mod ride_access_tests;
mod ride_appeal;
mod ride_choice;
#[cfg(test)]
mod starter_design_tests;
mod train_configuration;
pub use train_configuration::{
    DEFAULT_COASTER_CARS, MAX_COASTER_CARS, MIN_COASTER_CARS, SEATS_PER_COASTER_CAR,
    TrainConfigurationError,
};
mod ride_dispatch;
mod ride_opening;
mod ride_operations;
pub use ride_opening::RideOpeningError;
mod ride_testing;
mod saves;
mod services;
#[cfg(test)]
mod slope_tests;
mod surfaces;
mod track_construction;
mod track_designs;
mod trains;

pub use access_planning::{
    RideAccessBuildKind, RideAccessConnection, RideAccessPlan, RideAccessSuggestion,
    RideGateCandidate,
};
pub use admission::{
    QUEUE_PATIENCE_TICKS, RIDE_COOLDOWN_TICKS, RideQueueStats, RideVisit, RideVisitPhase,
};
pub use carousel::{
    CAROUSEL_BUILD_COST_CENTS, CAROUSEL_REFUND_PERCENT, CAROUSEL_RUN_TICKS, CAROUSEL_SEATS,
    CarouselError, CarouselPhase, CarouselRide, MAX_CAROUSEL_RECORDS, MAX_RIDE_RECORDS, RideKind,
};
pub use crowds::{
    CrowdStats, Guest, GuestId, GuestPosition, GuestRideHistory, GuestState, PathCacheStats,
    WALK_TICKS,
};
pub use economy::{
    EconomyError, INITIAL_GUEST_WALLET_CENTS, INITIAL_PARK_CASH_CENTS, MAX_TICKET_PRICE_CENTS,
    PRICE_STEP_CENTS, RideEconomyStats,
};
pub use experience::{GuestExperienceSummary, GuestPreferences, GuestThought, RideExperienceStats};
pub use finances::{
    FINANCE_LEDGER_CAPACITY, FinanceEntry, FinanceKind, FinanceLedger, FinanceStats, FinanceTotals,
    RideFinanceTotals, SERVICE_UPKEEP_TICKS, ServiceFinanceOpening, ServiceFinanceTotals,
};
pub use journeys::{
    PassengerJourney, RIDE_TRANSFER_TICKS, RideExitAccess, RideExitStatus, RideGuestPosition,
    RideJourneyStats, RideTransfer, TRAIN_SEATS,
};
pub use path_geometry::{
    MAX_SLOPED_PATH_HEIGHT, PATH_UNITS, PathDirection, PathPoint, PathSlope, SLOPED_PATH_CLEARANCE,
    paths_connect,
};
pub use ride_access::{Ride, RideAccess, RideAccessError, RideAccessStatus, RideEntrance, RideId};
pub use ride_appeal::{GuestRideAppeal, RideAppealProfile};
pub use ride_dispatch::{
    EXTRA_WAIT_STEP_TICKS, MAX_DISPATCH_PASSENGERS, MAX_EXTRA_WAIT_TICKS, MIN_EXTRA_WAIT_TICKS,
    RideDispatchError, RideDispatchPolicy, RideDispatchProgress,
};
pub use ride_operations::{
    BRAKE_SPEED_STEP, CAROUSEL_CYCLE_STEP_TICKS, EQUIPMENT_SPEED_STEP, LIFT_SPEED_STEP,
    MAX_BRAKE_SPEED, MAX_CAROUSEL_CYCLE_TICKS, MAX_LIFT_SPEED, MAX_STATION_WAIT_TICKS,
    MIN_BRAKE_SPEED, MIN_CAROUSEL_CYCLE_TICKS, MIN_LIFT_SPEED, MIN_STATION_WAIT_TICKS,
    RideOperatingError, RideOperatingSettings, STATION_WAIT_STEP_TICKS,
};
pub use ride_testing::{
    COASTER_TEST_LIMITATIONS, CoasterLayoutMetrics, CoasterLayoutRatings, CoasterTestError,
    CoasterTestInput, CoasterTestReport, MAX_COASTER_TEST_TICKS,
};
pub use saves::{
    CONTENT_VERSION as SAVE_CONTENT_VERSION, MAX_SAVE_BYTES, SCHEMA_VERSION as SAVE_SCHEMA_VERSION,
    SaveError,
};
pub use services::{
    GuestNeeds, GuestNeedsSummary, MAX_SERVICE_PRICE_CENTS, MAX_SERVICES, NEED_CRITICAL_THRESHOLD,
    NEED_SEEK_THRESHOLD, SERVICE_COOLDOWN_TICKS, SERVICE_PRICE_STEP_CENTS, SERVICE_REFUND_PERCENT,
    SERVICE_USE_TICKS, SERVICE_WAIT_TICKS, Service, ServiceAccess, ServiceAccessStatus,
    ServiceError, ServiceId, ServiceKind, ServiceStats, ServiceVisit, ServiceVisitPhase,
};
pub use surfaces::{
    Axis, MAX_PATH_HEIGHT, ObjectId, ObjectKind, PATH_CLEARANCE, PlacementError, SurfaceId,
    SurfaceObject,
};
pub use track_construction::{
    MAX_TRACK_DRAFT_PIECES, PaidTrackPiece, TRACK_REFUND_PERCENT, TrackBuildKind,
    TrackConstructionError, TrackConstructionQuote, TrackEconomy, track_piece_cost_cents,
};
pub use track_designs::{
    MAX_TRACK_DESIGN_BYTES, MAX_TRACK_DESIGN_NAME_CHARS, TRACK_DESIGN_VERSION, TrackDesign,
    TrackDesignError,
};
pub use trains::{
    STATION_DWELL_TICKS, TRACK_UNITS, TrackDirection, TrackKind, TrackPiece, TrackPieceSpec,
    TrackPoint, TrackPort, Train, TrainCircuit, TrainError, TrainPhase, TrainPose,
};

/// Number of normal-speed simulation ticks in one second.
pub const TICKS_PER_SECOND: u64 = 30;

/// Largest supported number of tiles along either edge of a foundation map.
pub const MAX_MAP_SIZE: u32 = 256;

/// Highest permitted terrain elevation, measured in abstract simulation steps.
/// The presentation layer decides how many pixels or world units a step uses.
pub const MAX_TERRAIN_HEIGHT: u8 = 32;

/// An integer tile coordinate, with the origin at the map's first corner.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Terrain {
    Grass,
    Water,
    Sand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tile {
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub terrain: Terrain,
    /// Elevation in abstract simulation steps, between 0 and MAX_TERRAIN_HEIGHT.
    pub height: u8,
}

/// Repeatable elevation presets. Terrain materials and park time are preserved.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TerrainProfile {
    #[default]
    Flat,
    Terraced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainHeightError {
    OutOfBounds { pos: TilePos },
    HeightTooHigh { height: u8, max: u8 },
    ObjectIntersection { pos: TilePos, object: ObjectId },
}

impl std::fmt::Display for TerrainHeightError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds { pos } => {
                write!(formatter, "tile ({}, {}) is outside the park", pos.x, pos.y)
            }
            Self::HeightTooHigh { height, max } => {
                write!(
                    formatter,
                    "terrain height {height} exceeds the maximum of {max} steps"
                )
            }
            Self::ObjectIntersection { pos, object } => write!(
                formatter,
                "terrain at ({}, {}) would intersect object {}",
                pos.x, pos.y, object.0
            ),
        }
    }
}

impl std::error::Error for TerrainHeightError {}

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

#[derive(Clone, Debug)]
pub struct Park {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
    ticks: u64,
    objects: Vec<SurfaceObject>,
    next_object_id: u64,
    guests: Vec<Guest>,
    next_guest_id: u64,
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    circuit: Option<TrainCircuit>,
    track_draft: Vec<TrackPieceSpec>,
    track_economy: TrackEconomy,
    train: Option<Train>,
    ride: Option<Ride>,
    carousels: Vec<CarouselRide>,
    ride_operating_settings: Vec<ride_operations::RideOperatingSettingsRecord>,
    ride_dispatch: Vec<ride_dispatch::RideDispatchRecord>,
    coaster_cars: u8,
    next_ride_id: u64,
    cash_cents: u64,
    finances: FinanceLedger,
    services: Vec<Service>,
    next_service_id: u64,
    path_cache: crowds::PathCache,
}

/// Cache warmth and diagnostic counters are not simulation state. Existing
/// authoritative fields, including ordered objects/guests and RNG, retain
/// their previous equality semantics across cloning and save/load.
impl PartialEq for Park {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.tiles == other.tiles
            && self.ticks == other.ticks
            && self.objects == other.objects
            && self.next_object_id == other.next_object_id
            && self.guests == other.guests
            && self.next_guest_id == other.next_guest_id
            && self.crowd_seed == other.crowd_seed
            && self.crowd_rng == other.crowd_rng
            && self.circuit == other.circuit
            && self.track_draft == other.track_draft
            && self.track_economy == other.track_economy
            && self.train == other.train
            && self.ride == other.ride
            && self.carousels == other.carousels
            && self.ride_operating_settings == other.ride_operating_settings
            && self.ride_dispatch == other.ride_dispatch
            && self.coaster_cars == other.coaster_cars
            && self.next_ride_id == other.next_ride_id
            && self.cash_cents == other.cash_cents
            && self.finances == other.finances
            && self.services == other.services
            && self.next_service_id == other.next_service_id
    }
}

impl Eq for Park {}

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
            objects: Vec::new(),
            next_object_id: 1,
            guests: Vec::new(),
            next_guest_id: 1,
            crowd_seed: None,
            crowd_rng: 0,
            circuit: None,
            track_draft: Vec::new(),
            track_economy: TrackEconomy::default(),
            train: None,
            ride: None,
            carousels: Vec::new(),
            ride_operating_settings: Vec::new(),
            ride_dispatch: Vec::new(),
            coaster_cars: DEFAULT_COASTER_CARS,
            next_ride_id: 1,
            cash_cents: INITIAL_PARK_CASH_CENTS,
            finances: FinanceLedger::default(),
            services: Vec::new(),
            next_service_id: 1,
            path_cache: crowds::PathCache::default(),
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

    /// Replace elevations without changing terrain materials, positions, or time.
    ///
    /// The terraced preset leaves a four-tile border clear and keeps lake and
    /// shore tiles at zero. Small maps may remain flat to preserve that border.
    ///
    /// # Panics
    ///
    /// Panics without changing the park if a placed object would intersect the
    /// new terrain. Use `try_apply_terrain_profile` for fallible player actions.
    pub fn apply_terrain_profile(&mut self, profile: TerrainProfile) {
        self.try_apply_terrain_profile(profile)
            .expect("terrain profile must preserve placed-object clearance");
    }

    /// Validate every proposed height before replacing any part of the terrain.
    /// Object identities, elevations, materials, and time remain unchanged.
    pub fn try_apply_terrain_profile(
        &mut self,
        profile: TerrainProfile,
    ) -> Result<(), TerrainHeightError> {
        let heights: Vec<_> = self
            .tiles
            .iter()
            .enumerate()
            .map(|(index, tile)| {
                let pos = TilePos {
                    x: index as u32 % self.width,
                    y: index as u32 / self.width,
                };
                match profile {
                    TerrainProfile::Flat => 0,
                    TerrainProfile::Terraced if tile.terrain == Terrain::Grass => {
                        terrace_height(pos, self.width, self.height)
                    }
                    TerrainProfile::Terraced => 0,
                }
            })
            .collect();
        for (index, &height) in heights.iter().enumerate() {
            let pos = TilePos {
                x: index as u32 % self.width,
                y: index as u32 / self.width,
            };
            self.validate_terrain_clearance(pos, height)?;
        }
        for (tile, height) in self.tiles.iter_mut().zip(heights) {
            tile.height = height;
        }
        Ok(())
    }

    /// Preview a single elevation using exactly the setter's bounds and
    /// occupied-space rules. Validation never changes state or path caches.
    /// Every terrain material is supported by this low-level simulation API;
    /// the application's editing tools may impose additional gameplay rules.
    pub fn validate_terrain_height(
        &self,
        pos: TilePos,
        height: u8,
    ) -> Result<(), TerrainHeightError> {
        if pos.x >= self.width || pos.y >= self.height {
            return Err(TerrainHeightError::OutOfBounds { pos });
        }
        if height > MAX_TERRAIN_HEIGHT {
            return Err(TerrainHeightError::HeightTooHigh {
                height,
                max: MAX_TERRAIN_HEIGHT,
            });
        }
        self.validate_terrain_clearance(pos, height)
    }

    /// Set a single elevation after validating the coordinate and height.
    ///
    /// An error leaves the entire park unchanged. This low-level setter permits
    /// every terrain material and preserves placed-object clearance. Ownership,
    /// cost, and other gameplay editing rules belong in the future command layer.
    pub fn set_terrain_height(
        &mut self,
        pos: TilePos,
        height: u8,
    ) -> Result<(), TerrainHeightError> {
        self.validate_terrain_height(pos, height)?;
        self.tiles[(pos.y * self.width + pos.x) as usize].height = height;
        Ok(())
    }

    pub fn max_terrain_height(&self) -> u8 {
        self.tiles.iter().map(|tile| tile.height).max().unwrap_or(0)
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
    /// Every accelerated tick executes all authoritative systems. The clock
    /// stops at its limit together with the simulation rather than wrapping.
    pub fn advance(&mut self, speed: Speed) {
        for _ in 0..speed.multiplier() {
            if self.ticks == u64::MAX {
                break;
            }
            self.ticks += 1;
            self.step_service_upkeep();
            self.step_needs();
            self.step_crowd();
            self.step_ride_journeys();
            self.step_train();
            self.step_carousels();
            self.reconcile_passenger_laps();
        }
    }
}

fn terrace_height(pos: TilePos, width: u32, height: u32) -> u8 {
    let edge = pos
        .x
        .min(pos.y)
        .min(width - 1 - pos.x)
        .min(height - 1 - pos.y);
    if edge < 4 {
        return 0;
    }
    // The same integer tile-center coordinates used for lake generation keep
    // profiles repeatable across platforms and rectangular map dimensions.
    let x = i64::from((pos.x * 2 + 1) * 500 / width);
    let y = i64::from((pos.y * 2 + 1) * 500 / height);
    let distance = ellipse_score(x - 400, y - 500, 260, 280);
    match distance {
        0..=1_600 => 6,
        1_601..=4_900 => 4,
        4_901..=10_000 => 2,
        _ => 0,
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

    #[test]
    fn terraced_profile_is_repeatable_multilevel_and_nested() {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        let mut second = Park::new(32, 32);
        second.apply_terrain_profile(TerrainProfile::Terraced);
        assert_eq!(park, second);
        assert_eq!(park.max_terrain_height(), 6);
        for level in [0, 2, 4, 6] {
            assert!(park.tiles.iter().any(|tile| tile.height == level));
        }
        // Cross the central hill from the flat border towards its summit.
        for (x, expected) in [(3, 0), (5, 2), (8, 4), (12, 6)] {
            assert_eq!(park.tile(TilePos { x, y: 16 }).unwrap().height, expected);
        }
        let snapshot = park.clone();
        park.apply_terrain_profile(TerrainProfile::Terraced);
        assert_eq!(
            park, snapshot,
            "reapplying a preset must not accumulate height"
        );
    }

    #[test]
    fn profiles_preserve_materials_dimensions_clock_and_flat_regression() {
        let mut park = Park::new(32, 32);
        for _ in 0..47 {
            park.advance(Speed::Fast);
        }
        let original = park.clone();
        park.apply_terrain_profile(TerrainProfile::Terraced);
        assert_eq!((park.width(), park.height(), park.ticks()), (32, 32, 94));
        for (index, tile) in park.tiles.iter().enumerate() {
            assert_eq!(tile.terrain, original.tiles[index].terrain);
            let x = index as u32 % park.width;
            let y = index as u32 / park.width;
            if tile.terrain != Terrain::Grass || x < 4 || y < 4 || x >= 28 || y >= 28 {
                assert_eq!(tile.height, 0, "shore, water, and border must stay flat");
            }
        }
        park.apply_terrain_profile(TerrainProfile::Flat);
        assert_eq!(park, original);
        assert_eq!(park.max_terrain_height(), 0);
        assert_eq!(TerrainProfile::default(), TerrainProfile::Flat);
    }

    #[test]
    fn profiles_handle_small_rectangular_and_maximum_maps() {
        for (width, height) in [
            (1, 1),
            (7, 3),
            (9, 9),
            (20, 12),
            (MAX_MAP_SIZE, MAX_MAP_SIZE),
        ] {
            let mut park = Park::new(width, height);
            park.apply_terrain_profile(TerrainProfile::Terraced);
            assert_eq!(park.tiles.len(), (width * height) as usize);
            assert!(
                park.tiles
                    .iter()
                    .all(|tile| tile.height <= MAX_TERRAIN_HEIGHT)
            );
            if width < 9 || height < 9 {
                assert_eq!(park.max_terrain_height(), 0);
            }
        }
    }

    #[test]
    fn terrain_setter_changes_only_requested_height_including_extremes() {
        let mut park = Park::new(32, 32);
        park.advance(Speed::Fastest);
        let original = park.clone();
        let pos = TilePos { x: 12, y: 16 };
        assert_eq!(park.set_terrain_height(pos, MAX_TERRAIN_HEIGHT), Ok(()));
        assert_eq!(park.tile(pos).unwrap().height, MAX_TERRAIN_HEIGHT);
        assert_eq!(park.max_terrain_height(), MAX_TERRAIN_HEIGHT);
        assert_eq!(park.ticks(), original.ticks());
        assert_eq!(park.set_terrain_height(pos, 0), Ok(()));
        assert_eq!(park, original);
    }

    fn live_terrain_fixture() -> Park {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(123).unwrap();
        park.install_train_fixture().unwrap();
        for _ in 0..41 {
            park.advance(Speed::Normal);
        }
        assert!(park.path_cache_stats().cached);
        park
    }

    fn assert_terrain_preview_commit(park: &Park, pos: TilePos, height: u8, allowed: bool) {
        let before_cache = park.path_cache_stats();
        let before_bytes = park.to_save_bytes().unwrap();
        let preview = park.validate_terrain_height(pos, height);
        assert_eq!(preview.is_ok(), allowed, "{pos:?} at {height}");
        assert_eq!(park.to_save_bytes().unwrap(), before_bytes);
        assert_eq!(park.path_cache_stats(), before_cache);
        let mut changed = park.clone();
        assert_eq!(changed.set_terrain_height(pos, height), preview);
        let mut expected = park.clone();
        if allowed {
            // Whole-Park equality checks every authoritative field: only this
            // elevation may differ, including during crowd and train motion.
            expected.tiles[(pos.y * expected.width + pos.x) as usize].height = height;
        }
        assert_eq!(changed, expected);
        assert_eq!(changed.path_cache_stats(), before_cache);
        assert_eq!(
            changed.to_save_bytes().unwrap(),
            expected.to_save_bytes().unwrap()
        );
    }

    #[test]
    fn terrain_validation_matches_commit_at_map_and_height_boundaries_without_mutation() {
        let park = live_terrain_fixture();
        for height in [0, MAX_TERRAIN_HEIGHT, MAX_TERRAIN_HEIGHT + 1, u8::MAX] {
            assert_terrain_preview_commit(
                &park,
                TilePos { x: 0, y: 0 },
                height,
                height <= MAX_TERRAIN_HEIGHT,
            );
        }
        for pos in [
            TilePos { x: 32, y: 0 },
            TilePos { x: 0, y: 32 },
            TilePos {
                x: u32::MAX,
                y: u32::MAX,
            },
        ] {
            assert_eq!(
                park.validate_terrain_height(pos, u8::MAX),
                Err(TerrainHeightError::OutOfBounds { pos })
            );
            assert_terrain_preview_commit(&park, pos, 0, false);
        }
    }

    #[test]
    fn terrain_validation_preserves_all_materials_and_every_other_live_state_field() {
        let park = live_terrain_fixture();
        for terrain in [Terrain::Grass, Terrain::Sand, Terrain::Water] {
            let pos = (0..park.height)
                .flat_map(|y| (0..park.width).map(move |x| TilePos { x, y }))
                .find(|pos| {
                    park.tile(*pos).unwrap().terrain == terrain
                        && !park.objects.iter().any(|object| object.pos == *pos)
                })
                .unwrap();
            for height in [1, MAX_TERRAIN_HEIGHT, 0] {
                assert_terrain_preview_commit(&park, pos, height, true);
            }
        }
    }

    #[test]
    fn terrain_validation_respects_path_and_buried_graded_track_envelopes() {
        let park = live_terrain_fixture();
        let overlap = TilePos { x: 13, y: 16 };
        // A ground track occupies [0,2); the path occupies [8,10).
        // Terrain can expose the track at zero, or cover it up to the path.
        for height in [0, 1, 2, 7, 8, 9, MAX_TERRAIN_HEIGHT] {
            assert_terrain_preview_commit(&park, overlap, height, matches!(height, 0 | 2..=8));
        }
        let graded = park
            .objects
            .iter()
            .find(|object| object.kind == ObjectKind::Track && object.clearance > 2)
            .unwrap();
        let top = graded.height + graded.clearance;
        for (height, allowed) in [
            (graded.height, true),
            (graded.height + 1, false),
            (top - 1, false),
            (top, true),
            (MAX_TERRAIN_HEIGHT, true),
        ] {
            assert_terrain_preview_commit(&park, graded.pos, height, allowed);
        }
        let mut buried = park.clone();
        buried.set_terrain_height(graded.pos, top).unwrap();
        assert!(buried.is_underground(graded.id));
        assert_terrain_preview_commit(&buried, graded.pos, top - 1, false);
        buried
            .set_terrain_height(graded.pos, graded.height)
            .unwrap();
        assert!(!buried.is_underground(graded.id));
        assert_eq!(buried.objects(), park.objects());
        assert_eq!(buried.train(), park.train());
    }

    #[test]
    fn invalid_terrain_edits_are_atomic_and_never_wrap_rows() {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        let snapshot = park.clone();
        for pos in [
            TilePos { x: 32, y: 0 },
            TilePos { x: 0, y: 32 },
            TilePos {
                x: u32::MAX,
                y: u32::MAX,
            },
        ] {
            assert_eq!(
                park.set_terrain_height(pos, 2),
                Err(TerrainHeightError::OutOfBounds { pos })
            );
            assert_eq!(park, snapshot);
        }
        for height in [MAX_TERRAIN_HEIGHT + 1, u8::MAX] {
            assert_eq!(
                park.set_terrain_height(TilePos { x: 12, y: 16 }, height),
                Err(TerrainHeightError::HeightTooHigh {
                    height,
                    max: MAX_TERRAIN_HEIGHT
                })
            );
            assert_eq!(park, snapshot);
        }
    }
}
