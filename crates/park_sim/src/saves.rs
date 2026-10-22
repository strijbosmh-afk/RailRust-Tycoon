//! Versioned simulation-only saves. JSON never deserializes directly into Park:
//! bounded DTOs are validated first, then derived circuit geometry is rebuilt.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::marker::PhantomData;

mod construction;
use construction::{LegacyFinanceKind, VersionTenFinanceTotals, VersionTenParkData};

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{
    Guest, GuestState, MAX_MAP_SIZE, MAX_TERRAIN_HEIGHT, ObjectId, ObjectKind, Park,
    STATION_DWELL_TICKS, SurfaceObject, TRACK_UNITS, Tile, TrackKind, TrackPiece, TrackPieceSpec,
    Train, TrainCircuit, TrainPhase, WALK_TICKS,
};

/// Both readers and writers reject larger files before JSON parsing/returning.
pub const MAX_SAVE_BYTES: usize = 8 * 1024 * 1024;
const MAX_TILES: usize = (MAX_MAP_SIZE * MAX_MAP_SIZE) as usize;
pub(crate) const MAX_OBJECTS: usize = 4096;
pub(crate) const MAX_PATHS: usize = 1024;
const MAX_GUESTS: usize = 256;
const MAX_CIRCUIT_PIECES: usize = 2048;
const MAX_TOTAL_ROUTE_IDS: usize = 65_536;
const SCHEMA_VERSION: u32 = 11;
const CONTENT_VERSION: u32 = 11;
const FORMAT: &str = "parkbound-save";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SaveError {
    TooLarge { actual: usize, max: usize },
    Json(String),
    UnsupportedSchema(u32),
    UnsupportedContent(u32),
    InvalidState(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge { actual, max } => {
                write!(formatter, "save has {actual} bytes; maximum is {max}")
            }
            Self::Json(message) => write!(formatter, "invalid save JSON: {message}"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported save schema version {version}")
            }
            Self::UnsupportedContent(version) => {
                write!(
                    formatter,
                    "unsupported simulation content version {version}"
                )
            }
            Self::InvalidState(message) => write!(formatter, "invalid saved park: {message}"),
        }
    }
}

impl std::error::Error for SaveError {}

fn invalid(message: impl Into<String>) -> SaveError {
    SaveError::InvalidState(message.into())
}

fn json_error(error: serde_json::Error) -> SaveError {
    // A malicious unknown field name must not produce megabytes of UI text.
    SaveError::Json(error.to_string().chars().take(512).collect())
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "P: Deserialize<'de>"))]
struct SaveEnvelope<P = ParkData> {
    format: String,
    schema_version: u32,
    content_version: u32,
    #[serde(deserialize_with = "strict_object")]
    park: P,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "O: Deserialize<'de>, G: Deserialize<'de>, R: Deserialize<'de>"))]
struct VersionSevenParkData<O = SurfaceObject, G = VersionSevenGuest, R = crate::Ride> {
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<O>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<G>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
    #[serde(deserialize_with = "required_object_option")]
    ride: Option<R>,
    next_ride_id: u64,
    cash_cents: u64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "O: Deserialize<'de>, G: Deserialize<'de>, R: Deserialize<'de>"))]
struct ParkData<O = SurfaceObject, G = Guest, R = crate::Ride> {
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<O>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<G>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
    #[serde(deserialize_with = "required_object_option")]
    ride: Option<R>,
    next_ride_id: u64,
    cash_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    services: Vec<crate::Service>,
    next_service_id: u64,
    #[serde(deserialize_with = "strict_object")]
    finances: crate::FinanceLedger,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_CAROUSEL_RECORDS }>")]
    carousels: Vec<crate::CarouselRide>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionNineParkData {
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<SurfaceObject>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<VersionNineGuest>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
    #[serde(deserialize_with = "required_object_option")]
    ride: Option<crate::Ride>,
    next_ride_id: u64,
    cash_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    services: Vec<crate::Service>,
    next_service_id: u64,
    #[serde(deserialize_with = "strict_object")]
    finances: VersionNineFinanceLedger,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionNineFinanceEntry {
    id: u64,
    tick: u64,
    #[serde(deserialize_with = "string_enum")]
    kind: LegacyFinanceKind,
    #[serde(deserialize_with = "required_option")]
    service: Option<crate::ServiceId>,
    amount_cents: u64,
    balance_cents: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionNineFinanceLedger {
    opening_tick: u64,
    opening_cash_cents: u64,
    opening_ride_revenue_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    opening_services: Vec<crate::ServiceFinanceOpening>,
    archived_entries: u64,
    archived_through_tick: u64,
    #[serde(deserialize_with = "strict_object")]
    archived: VersionTenFinanceTotals,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    archived_services: Vec<crate::ServiceFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::FINANCE_LEDGER_CAPACITY }>")]
    entries: Vec<VersionNineFinanceEntry>,
    next_entry_id: u64,
}
impl VersionNineFinanceLedger {
    fn migrate(self, ride: Option<crate::RideId>) -> crate::FinanceLedger {
        crate::FinanceLedger {
            opening_tick: self.opening_tick,
            opening_cash_cents: self.opening_cash_cents,
            opening_ride_revenue_cents: self.opening_ride_revenue_cents,
            opening_rides: ride
                .map(|ride| crate::RideFinanceTotals {
                    ride,
                    revenue_cents: self.opening_ride_revenue_cents,
                    construction_cents: 0,
                    demolition_refunds_cents: 0,
                })
                .into_iter()
                .collect(),
            opening_services: self.opening_services,
            archived_entries: self.archived_entries,
            archived_through_tick: self.archived_through_tick,
            archived_rides: ride
                .filter(|_| self.archived.ride_revenue_cents != 0)
                .map(|ride| crate::RideFinanceTotals {
                    ride,
                    revenue_cents: self.archived.ride_revenue_cents,
                    construction_cents: 0,
                    demolition_refunds_cents: 0,
                })
                .into_iter()
                .collect(),
            archived: self.archived.into(),
            archived_services: self.archived_services,
            entries: self
                .entries
                .into_iter()
                .map(|entry| crate::FinanceEntry {
                    id: entry.id,
                    tick: entry.tick,
                    kind: entry.kind.into(),
                    ride: (matches!(entry.kind, LegacyFinanceKind::RideRevenue))
                        .then_some(ride)
                        .flatten(),
                    service: entry.service,
                    amount_cents: entry.amount_cents,
                    balance_cents: entry.balance_cents,
                })
                .collect(),
            next_entry_id: self.next_entry_id,
        }
    }
}
impl VersionNineParkData {
    fn migrate(self) -> ParkData {
        let finances = self
            .finances
            .migrate(self.ride.as_ref().map(|ride| ride.id));
        ParkData {
            width: self.width,
            height: self.height,
            tiles: self.tiles,
            ticks: self.ticks,
            objects: self.objects,
            next_object_id: self.next_object_id,
            guests: self.guests.into_iter().map(Into::into).collect(),
            next_guest_id: self.next_guest_id,
            crowd_seed: self.crowd_seed,
            crowd_rng: self.crowd_rng,
            circuit: self.circuit,
            train: self.train,
            ride: self.ride,
            next_ride_id: self.next_ride_id,
            cash_cents: self.cash_cents,
            services: self.services,
            next_service_id: self.next_service_id,
            finances,
            carousels: Vec::new(),
        }
    }
}

/// Strict pre-carousel guest shape. New owner histories may not be smuggled
/// into a document declaring an earlier schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionNineGuest {
    id: crate::GuestId,
    #[serde(deserialize_with = "required_option")]
    current: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    next: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    destination: Option<ObjectId>,
    #[serde(deserialize_with = "strict_object")]
    anchor: crate::TilePos,
    height: u8,
    #[serde(deserialize_with = "string_enum")]
    anchor_slope: crate::PathSlope,
    progress: u16,
    #[serde(deserialize_with = "string_enum")]
    state: GuestState,
    arrivals: u64,
    rides_completed: u64,
    wallet_cents: u32,
    spent_cents: u64,
    service_spent_cents: u64,
    services_used: u64,
    #[serde(deserialize_with = "strict_object")]
    needs: crate::GuestNeeds,
    #[serde(deserialize_with = "required_object_option")]
    service_visit: Option<crate::ServiceVisit>,
    service_cooldown: u16,
    #[serde(deserialize_with = "strict_object")]
    preferences: crate::GuestPreferences,
    satisfaction: u8,
    #[serde(deserialize_with = "string_enum")]
    thought: crate::GuestThought,
    thought_tick: u64,
    ratings_given: u64,
    rating_total: u64,
    #[serde(deserialize_with = "required_option")]
    last_rating: Option<u8>,
    #[serde(deserialize_with = "required_object_option")]
    ride_visit: Option<crate::RideVisit>,
    ride_cooldown: u16,
    #[serde(deserialize_with = "bounded_route")]
    route: VecDeque<ObjectId>,
    wait_ticks: u16,
}
impl From<VersionNineGuest> for Guest {
    fn from(old: VersionNineGuest) -> Self {
        Self {
            id: old.id,
            current: old.current,
            next: old.next,
            destination: old.destination,
            anchor: old.anchor,
            height: old.height,
            anchor_slope: old.anchor_slope,
            progress: old.progress,
            state: old.state,
            arrivals: old.arrivals,
            rides_completed: old.rides_completed,
            wallet_cents: old.wallet_cents,
            spent_cents: old.spent_cents,
            service_spent_cents: old.service_spent_cents,
            services_used: old.services_used,
            needs: old.needs,
            service_visit: old.service_visit,
            service_cooldown: old.service_cooldown,
            preferences: old.preferences,
            satisfaction: old.satisfaction,
            thought: old.thought,
            thought_tick: old.thought_tick,
            ratings_given: old.ratings_given,
            rating_total: old.rating_total,
            last_rating: old.last_rating,
            ride_visit: old.ride_visit,
            ride_cooldown: old.ride_cooldown,
            route: old.route,
            wait_ticks: old.wait_ticks,
            ride_history: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionEightParkData {
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<SurfaceObject>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<VersionNineGuest>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
    #[serde(deserialize_with = "required_object_option")]
    ride: Option<crate::Ride>,
    next_ride_id: u64,
    cash_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    services: Vec<VersionEightService>,
    next_service_id: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionEightService {
    id: crate::ServiceId,
    #[serde(deserialize_with = "required_option")]
    object: Option<ObjectId>,
    #[serde(deserialize_with = "string_enum")]
    kind: crate::ServiceKind,
    #[serde(deserialize_with = "string_enum")]
    facing: crate::PathDirection,
    open: bool,
    price_cents: u32,
    served: u64,
    revenue_cents: u64,
    unaffordable_rejections: u64,
}
impl VersionEightParkData {
    fn migrate(self) -> ParkData {
        ParkData {
            width: self.width,
            height: self.height,
            tiles: self.tiles,
            ticks: self.ticks,
            objects: self.objects,
            next_object_id: self.next_object_id,
            guests: self.guests.into_iter().map(Into::into).collect(),
            next_guest_id: self.next_guest_id,
            crowd_seed: self.crowd_seed,
            crowd_rng: self.crowd_rng,
            circuit: self.circuit,
            train: self.train,
            ride: self.ride,
            next_ride_id: self.next_ride_id,
            cash_cents: self.cash_cents,
            next_service_id: self.next_service_id,
            services: self
                .services
                .into_iter()
                .map(|old| crate::Service {
                    id: old.id,
                    object: old.object,
                    kind: old.kind,
                    facing: old.facing,
                    open: old.open,
                    price_cents: old.price_cents,
                    served: old.served,
                    revenue_cents: old.revenue_cents,
                    unaffordable_rejections: old.unaffordable_rejections,
                    upkeep_progress: 0,
                    upkeep_cents: 0,
                    supplies_cents: 0,
                    closed_for_funds: false,
                })
                .collect(),
            finances: crate::FinanceLedger::default(),
            carousels: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CircuitData {
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_CIRCUIT_PIECES>")]
    pieces: Vec<CircuitPieceData>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CircuitPieceData {
    object: ObjectId,
    #[serde(deserialize_with = "strict_object")]
    spec: TrackPieceSpec,
}

/// Serde normally also accepts positional arrays for structs and tagged maps
/// for unit enums. The on-disk JSON schema deliberately permits neither.
pub(crate) fn strict_object<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ObjectVisitor<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> de::Visitor<'de> for ObjectVisitor<T> {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a JSON object")
        }

        fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }
    deserializer.deserialize_map(ObjectVisitor(PhantomData))
}

pub(crate) fn string_enum<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let name = String::deserialize(deserializer)?;
    T::deserialize(de::value::StringDeserializer::new(name))
}

struct StrictObject<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for StrictObject<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        strict_object(deserializer).map(Self)
    }
}

pub(crate) fn required_object_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<StrictObject<T>>::deserialize(deserializer).map(|value| value.map(|object| object.0))
}

pub(crate) fn bounded_objects<'de, D, T, const LIMIT: usize>(
    deserializer: D,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    bounded_vec::<D, StrictObject<T>, LIMIT>(deserializer)
        .map(|objects| objects.into_iter().map(|object| object.0).collect())
}

/// Explicitly present null is supported; omission is a malformed schema.
pub(crate) fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

pub(crate) fn bounded_vec<'de, D, T, const LIMIT: usize>(
    deserializer: D,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct BoundedVisitor<T, const LIMIT: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const LIMIT: usize> de::Visitor<'de> for BoundedVisitor<T, LIMIT> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "an array of at most {LIMIT} entries")
        }

        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            if sequence.size_hint().is_some_and(|size| size > LIMIT) {
                return Err(de::Error::custom("save collection exceeds its limit"));
            }
            let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0).min(LIMIT));
            while let Some(value) = sequence.next_element()? {
                if values.len() == LIMIT {
                    return Err(de::Error::custom("save collection exceeds its limit"));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_seq(BoundedVisitor::<T, LIMIT>(PhantomData))
}

pub(crate) fn bounded_route<'de, D>(deserializer: D) -> Result<VecDeque<ObjectId>, D::Error>
where
    D: Deserializer<'de>,
{
    bounded_vec::<D, ObjectId, MAX_PATHS>(deserializer).map(VecDeque::from)
}

impl Park {
    /// Serialize all authoritative state with a strict versioned schema.
    /// Rendering, speed selection, camera, and wall-clock time are not saved.
    /// Writers use the same validation as readers, including proof size limits.
    pub fn to_save_bytes(&self) -> Result<Vec<u8>, SaveError> {
        validate_counts(self)?;
        let data = ParkData::from_park(self);
        let rebuilt = data.clone().into_park()?;
        if rebuilt != *self {
            return Err(invalid(
                "derived circuit geometry disagrees with its saved specs",
            ));
        }
        let envelope = SaveEnvelope {
            format: FORMAT.into(),
            schema_version: SCHEMA_VERSION,
            content_version: CONTENT_VERSION,
            park: data,
        };
        let bytes = serde_json::to_vec(&envelope).map_err(json_error)?;
        check_bytes(bytes.len())?;
        Ok(bytes)
    }

    /// Parse into an independent candidate; failure never changes a live park.
    /// Limits are checked before parsing arrays or doing geometry/graph work.
    pub fn from_save_bytes(bytes: &[u8]) -> Result<Self, SaveError> {
        check_bytes(bytes.len())?;
        // Read only the envelope first. IgnoredAny does not allocate the park
        // graph; each version is then decoded through its own strict, bounded DTO.
        let header: SaveEnvelope<de::IgnoredAny> = parse_save(bytes)?;
        if header.format != FORMAT {
            return Err(invalid("unrecognized save format"));
        }
        if !matches!(
            header.schema_version,
            1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | SCHEMA_VERSION
        ) {
            return Err(SaveError::UnsupportedSchema(header.schema_version));
        }
        if header.content_version != header.schema_version {
            return Err(SaveError::UnsupportedContent(header.content_version));
        }
        let mut data = if header.schema_version == 1 {
            let legacy: SaveEnvelope<LegacyParkData<LegacyObject, LegacyGuest>> =
                parse_save(bytes)?;
            legacy.park.migrate()
        } else if header.schema_version == 2 {
            let legacy: SaveEnvelope<LegacyParkData<VersionTwoObject, VersionTwoGuest>> =
                parse_save(bytes)?;
            legacy.park.migrate()
        } else if header.schema_version == 3 {
            let legacy: SaveEnvelope<
                VersionFiveParkData<SurfaceObject, VersionTwoGuest, LegacyRide>,
            > = parse_save(bytes)?;
            legacy.park.migrate()
        } else if header.schema_version == 4 {
            let legacy: SaveEnvelope<
                VersionFiveParkData<SurfaceObject, VersionFourGuest, VersionFourRide>,
            > = parse_save(bytes)?;
            legacy.park.migrate()
        } else if header.schema_version == 5 {
            let legacy: SaveEnvelope<VersionFiveParkData> = parse_save(bytes)?;
            legacy.park.migrate()
        } else if header.schema_version == 6 {
            let legacy: SaveEnvelope<
                VersionSevenParkData<SurfaceObject, VersionSixGuest, VersionSixRide>,
            > = parse_save(bytes)?;
            legacy.park.migrate_experience()
        } else if header.schema_version == 7 {
            let legacy: SaveEnvelope<VersionSevenParkData> = parse_save(bytes)?;
            legacy.park.migrate_services()
        } else if header.schema_version == 8 {
            let legacy: SaveEnvelope<VersionEightParkData> = parse_save(bytes)?;
            legacy.park.migrate()
        } else if header.schema_version == 9 {
            let legacy: SaveEnvelope<VersionNineParkData> = parse_save(bytes)?;
            legacy.park.migrate()
        } else if header.schema_version == 10 {
            let legacy: SaveEnvelope<VersionTenParkData> = parse_save(bytes)?;
            legacy.park.migrate()
        } else {
            let envelope: SaveEnvelope = parse_save(bytes)?;
            envelope.park
        };
        if header.schema_version <= 8 {
            data.initialize_finances();
        }
        if header.schema_version <= 9 {
            data.initialize_ride_history()?;
        }
        data.into_park()
    }
}

fn parse_save<T: de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SaveError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = strict_object(&mut deserializer).map_err(json_error)?;
    deserializer.end().map_err(json_error)?;
    Ok(value)
}

/// Version one has no slope fields. Reject those fields instead of accepting
/// version-two state disguised as an older save. Migration adds only Flat.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyObject {
    id: ObjectId,
    #[serde(deserialize_with = "strict_object")]
    pos: crate::TilePos,
    #[serde(deserialize_with = "string_enum")]
    kind: LegacyObjectKind,
    #[serde(deserialize_with = "string_enum")]
    axis: crate::Axis,
    height: u8,
    clearance: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyGuest {
    id: crate::GuestId,
    #[serde(deserialize_with = "required_option")]
    current: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    next: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    destination: Option<ObjectId>,
    #[serde(deserialize_with = "strict_object")]
    anchor: crate::TilePos,
    height: u8,
    progress: u16,
    #[serde(deserialize_with = "string_enum")]
    state: GuestState,
    arrivals: u64,
    #[serde(deserialize_with = "bounded_route")]
    route: VecDeque<ObjectId>,
    wait_ticks: u16,
}

/// Versions two and three have slopes but no admission state.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionTwoGuest {
    id: crate::GuestId,
    #[serde(deserialize_with = "required_option")]
    current: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    next: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    destination: Option<ObjectId>,
    #[serde(deserialize_with = "strict_object")]
    anchor: crate::TilePos,
    height: u8,
    #[serde(deserialize_with = "string_enum")]
    anchor_slope: crate::PathSlope,
    progress: u16,
    #[serde(deserialize_with = "string_enum")]
    state: GuestState,
    arrivals: u64,
    #[serde(deserialize_with = "bounded_route")]
    route: VecDeque<ObjectId>,
    wait_ticks: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyRide {
    id: crate::RideId,
    #[serde(deserialize_with = "required_object_option")]
    entrance: Option<crate::RideEntrance>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFourRide {
    id: crate::RideId,
    #[serde(deserialize_with = "required_object_option")]
    entrance: Option<crate::RideEntrance>,
    draining: bool,
    next_queue_ticket: u64,
    joined: u64,
    turned_away: u64,
    abandoned: u64,
}
#[derive(Deserialize)]
enum VersionFourPhase {
    Approaching,
    Queueing,
    Leaving,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFourVisit {
    ride: crate::RideId,
    #[serde(deserialize_with = "string_enum")]
    phase: VersionFourPhase,
    #[serde(deserialize_with = "required_option")]
    ticket: Option<u64>,
    waited_ticks: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFourGuest {
    id: crate::GuestId,
    #[serde(deserialize_with = "required_option")]
    current: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    next: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    destination: Option<ObjectId>,
    #[serde(deserialize_with = "strict_object")]
    anchor: crate::TilePos,
    height: u8,
    #[serde(deserialize_with = "string_enum")]
    anchor_slope: crate::PathSlope,
    progress: u16,
    #[serde(deserialize_with = "string_enum")]
    state: GuestState,
    arrivals: u64,
    #[serde(deserialize_with = "required_object_option")]
    ride_visit: Option<VersionFourVisit>,
    ride_cooldown: u16,
    #[serde(deserialize_with = "bounded_route")]
    route: VecDeque<ObjectId>,
    wait_ticks: u16,
}
impl From<VersionFourRide> for crate::Ride {
    fn from(old: VersionFourRide) -> Self {
        Self {
            id: old.id,
            entrance: old.entrance,
            exit: None,
            draining: old.draining,
            next_queue_ticket: old.next_queue_ticket,
            joined: old.joined,
            turned_away: old.turned_away,
            abandoned: old.abandoned,
            boarded: 0,
            exited: 0,
            completed: 0,
            ticket_price_cents: 0,
            revenue_cents: 0,
            unaffordable_rejections: 0,
            value_rejections: 0,
            rating_count: 0,
            rating_total: 0,
        }
    }
}
impl From<VersionFourGuest> for Guest {
    fn from(old: VersionFourGuest) -> Self {
        Self {
            id: old.id,
            current: old.current,
            next: old.next,
            destination: old.destination,
            anchor: old.anchor,
            height: old.height,
            anchor_slope: old.anchor_slope,
            progress: old.progress,
            state: old.state,
            arrivals: old.arrivals,
            rides_completed: 0,
            wallet_cents: crate::INITIAL_GUEST_WALLET_CENTS,
            spent_cents: 0,
            service_spent_cents: 0,
            services_used: 0,
            ride_history: Vec::new(),
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
            ride_visit: old.ride_visit.map(|visit| crate::RideVisit {
                ride: visit.ride,
                phase: match visit.phase {
                    VersionFourPhase::Approaching => crate::RideVisitPhase::Approaching,
                    VersionFourPhase::Queueing => crate::RideVisitPhase::Queueing,
                    VersionFourPhase::Leaving => crate::RideVisitPhase::Leaving,
                },
                ticket: visit.ticket,
                waited_ticks: visit.waited_ticks,
                passenger: None,
            }),
            ride_cooldown: old.ride_cooldown,
            route: old.route,
            wait_ticks: old.wait_ticks,
        }
    }
}

impl From<VersionTwoGuest> for Guest {
    fn from(guest: VersionTwoGuest) -> Self {
        Self {
            id: guest.id,
            current: guest.current,
            next: guest.next,
            destination: guest.destination,
            anchor: guest.anchor,
            height: guest.height,
            anchor_slope: guest.anchor_slope,
            progress: guest.progress,
            state: guest.state,
            arrivals: guest.arrivals,
            rides_completed: 0,
            wallet_cents: crate::INITIAL_GUEST_WALLET_CENTS,
            spent_cents: 0,
            service_spent_cents: 0,
            services_used: 0,
            ride_history: Vec::new(),
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
            route: guest.route,
            wait_ticks: guest.wait_ticks,
            ride_visit: None,
            ride_cooldown: 0,
        }
    }
}

impl From<LegacyRide> for crate::Ride {
    fn from(legacy: LegacyRide) -> Self {
        let mut ride = Self::new(legacy.id);
        ride.entrance = legacy.entrance;
        ride
    }
}

/// Versions three through five have no cash field. Decode them separately so
/// a missing v6 field, or an economy field disguised as v5, is never accepted.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "O: Deserialize<'de>, G: Deserialize<'de>, R: Deserialize<'de>"))]
struct VersionFiveParkData<O = SurfaceObject, G = VersionFiveGuest, R = VersionFiveRide> {
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<O>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<G>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
    #[serde(deserialize_with = "required_object_option")]
    ride: Option<R>,
    next_ride_id: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFiveRide {
    id: crate::RideId,
    #[serde(deserialize_with = "required_object_option")]
    entrance: Option<crate::RideEntrance>,
    #[serde(deserialize_with = "required_object_option")]
    exit: Option<crate::RideEntrance>,
    draining: bool,
    next_queue_ticket: u64,
    joined: u64,
    turned_away: u64,
    abandoned: u64,
    boarded: u64,
    exited: u64,
    completed: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFivePassenger {
    seat: u8,
    departed: bool,
    completed_lap: bool,
    #[serde(deserialize_with = "required_object_option")]
    transfer: Option<crate::RideTransfer>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFiveVisit {
    ride: crate::RideId,
    #[serde(deserialize_with = "string_enum")]
    phase: crate::RideVisitPhase,
    #[serde(deserialize_with = "required_option")]
    ticket: Option<u64>,
    waited_ticks: u32,
    #[serde(deserialize_with = "required_object_option")]
    passenger: Option<VersionFivePassenger>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFiveGuest {
    id: crate::GuestId,
    #[serde(deserialize_with = "required_option")]
    current: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    next: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    destination: Option<ObjectId>,
    #[serde(deserialize_with = "strict_object")]
    anchor: crate::TilePos,
    height: u8,
    #[serde(deserialize_with = "string_enum")]
    anchor_slope: crate::PathSlope,
    progress: u16,
    #[serde(deserialize_with = "string_enum")]
    state: GuestState,
    arrivals: u64,
    rides_completed: u64,
    #[serde(deserialize_with = "required_object_option")]
    ride_visit: Option<VersionFiveVisit>,
    ride_cooldown: u16,
    #[serde(deserialize_with = "bounded_route")]
    route: VecDeque<ObjectId>,
    wait_ticks: u16,
}
impl From<VersionFiveRide> for crate::Ride {
    fn from(old: VersionFiveRide) -> Self {
        Self {
            id: old.id,
            entrance: old.entrance,
            exit: old.exit,
            draining: old.draining,
            next_queue_ticket: old.next_queue_ticket,
            joined: old.joined,
            turned_away: old.turned_away,
            abandoned: old.abandoned,
            boarded: old.boarded,
            exited: old.exited,
            completed: old.completed,
            ticket_price_cents: 0,
            revenue_cents: 0,
            unaffordable_rejections: 0,
            value_rejections: 0,
            rating_count: 0,
            rating_total: 0,
        }
    }
}
impl From<VersionFiveGuest> for Guest {
    fn from(old: VersionFiveGuest) -> Self {
        Self {
            id: old.id,
            current: old.current,
            next: old.next,
            destination: old.destination,
            anchor: old.anchor,
            height: old.height,
            anchor_slope: old.anchor_slope,
            progress: old.progress,
            state: old.state,
            arrivals: old.arrivals,
            rides_completed: old.rides_completed,
            wallet_cents: crate::INITIAL_GUEST_WALLET_CENTS,
            spent_cents: 0,
            service_spent_cents: 0,
            services_used: 0,
            ride_history: Vec::new(),
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
            ride_visit: old.ride_visit.map(|visit| crate::RideVisit {
                ride: visit.ride,
                phase: visit.phase,
                ticket: visit.ticket,
                waited_ticks: visit.waited_ticks,
                passenger: visit.passenger.map(|passenger| crate::PassengerJourney {
                    seat: passenger.seat,
                    departed: passenger.departed,
                    completed_lap: passenger.completed_lap,
                    fare_cents: 0,
                    fare_paid: passenger.departed,
                    transfer: passenger.transfer,
                }),
            }),
            ride_cooldown: old.ride_cooldown,
            route: old.route,
            wait_ticks: old.wait_ticks,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionSixRide {
    id: crate::RideId,
    #[serde(deserialize_with = "required_object_option")]
    entrance: Option<crate::RideEntrance>,
    #[serde(deserialize_with = "required_object_option")]
    exit: Option<crate::RideEntrance>,
    draining: bool,
    next_queue_ticket: u64,
    joined: u64,
    turned_away: u64,
    abandoned: u64,
    boarded: u64,
    exited: u64,
    completed: u64,
    ticket_price_cents: u32,
    revenue_cents: u64,
    unaffordable_rejections: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionSevenGuest {
    id: crate::GuestId,
    #[serde(deserialize_with = "required_option")]
    current: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    next: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    destination: Option<ObjectId>,
    #[serde(deserialize_with = "strict_object")]
    anchor: crate::TilePos,
    height: u8,
    #[serde(deserialize_with = "string_enum")]
    anchor_slope: crate::PathSlope,
    progress: u16,
    #[serde(deserialize_with = "string_enum")]
    state: GuestState,
    arrivals: u64,
    rides_completed: u64,
    wallet_cents: u32,
    spent_cents: u64,
    #[serde(deserialize_with = "strict_object")]
    preferences: crate::GuestPreferences,
    satisfaction: u8,
    #[serde(deserialize_with = "version_seven_thought")]
    thought: crate::GuestThought,
    thought_tick: u64,
    ratings_given: u64,
    rating_total: u64,
    #[serde(deserialize_with = "required_option")]
    last_rating: Option<u8>,
    #[serde(deserialize_with = "required_object_option")]
    ride_visit: Option<crate::RideVisit>,
    ride_cooldown: u16,
    #[serde(deserialize_with = "bounded_route")]
    route: VecDeque<ObjectId>,
    wait_ticks: u16,
}
fn version_seven_thought<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<crate::GuestThought, D::Error> {
    use crate::GuestThought::*;
    let thought = string_enum(deserializer)?;
    if matches!(
        thought,
        Exploring
            | HeadingToRide
            | Queueing
            | NotEnoughMoney
            | TooExpensive
            | QueueFull
            | CannotReachRide
            | WaitingTooLong
            | RideCancelled
            | EnjoyedRide
            | DisappointedByRide
    ) {
        Ok(thought)
    } else {
        Err(de::Error::custom(
            "guest thought was not supported in schema 7",
        ))
    }
}
impl From<VersionSevenGuest> for Guest {
    fn from(old: VersionSevenGuest) -> Self {
        Self {
            id: old.id,
            current: old.current,
            next: old.next,
            destination: old.destination,
            anchor: old.anchor,
            height: old.height,
            anchor_slope: old.anchor_slope,
            progress: old.progress,
            state: old.state,
            arrivals: old.arrivals,
            rides_completed: old.rides_completed,
            wallet_cents: old.wallet_cents,
            spent_cents: old.spent_cents,
            preferences: old.preferences,
            satisfaction: old.satisfaction,
            thought: old.thought,
            thought_tick: old.thought_tick,
            ratings_given: old.ratings_given,
            rating_total: old.rating_total,
            last_rating: old.last_rating,
            ride_visit: old.ride_visit,
            ride_cooldown: old.ride_cooldown,
            route: old.route,
            wait_ticks: old.wait_ticks,
            needs: crate::GuestNeeds::default(),
            service_visit: None,
            service_cooldown: 0,
            service_spent_cents: 0,
            services_used: 0,
            ride_history: Vec::new(),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionSixGuest {
    id: crate::GuestId,
    #[serde(deserialize_with = "required_option")]
    current: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    next: Option<ObjectId>,
    #[serde(deserialize_with = "required_option")]
    destination: Option<ObjectId>,
    #[serde(deserialize_with = "strict_object")]
    anchor: crate::TilePos,
    height: u8,
    #[serde(deserialize_with = "string_enum")]
    anchor_slope: crate::PathSlope,
    progress: u16,
    #[serde(deserialize_with = "string_enum")]
    state: GuestState,
    arrivals: u64,
    rides_completed: u64,
    wallet_cents: u32,
    spent_cents: u64,
    #[serde(deserialize_with = "required_object_option")]
    ride_visit: Option<crate::RideVisit>,
    ride_cooldown: u16,
    #[serde(deserialize_with = "bounded_route")]
    route: VecDeque<ObjectId>,
    wait_ticks: u16,
}
impl From<VersionSixRide> for crate::Ride {
    fn from(old: VersionSixRide) -> Self {
        Self {
            id: old.id,
            entrance: old.entrance,
            exit: old.exit,
            draining: old.draining,
            next_queue_ticket: old.next_queue_ticket,
            joined: old.joined,
            turned_away: old.turned_away,
            abandoned: old.abandoned,
            boarded: old.boarded,
            exited: old.exited,
            completed: old.completed,
            ticket_price_cents: old.ticket_price_cents,
            revenue_cents: old.revenue_cents,
            unaffordable_rejections: old.unaffordable_rejections,
            value_rejections: 0,
            rating_count: 0,
            rating_total: 0,
        }
    }
}
impl From<VersionSixGuest> for Guest {
    fn from(old: VersionSixGuest) -> Self {
        Self {
            id: old.id,
            current: old.current,
            next: old.next,
            destination: old.destination,
            anchor: old.anchor,
            height: old.height,
            anchor_slope: old.anchor_slope,
            progress: old.progress,
            state: old.state,
            arrivals: old.arrivals,
            rides_completed: old.rides_completed,
            wallet_cents: old.wallet_cents,
            spent_cents: old.spent_cents,
            service_spent_cents: 0,
            services_used: 0,
            ride_history: Vec::new(),
            needs: crate::GuestNeeds::default(),
            service_visit: None,
            service_cooldown: 0,
            ride_visit: old.ride_visit,
            ride_cooldown: old.ride_cooldown,
            route: old.route,
            wait_ticks: old.wait_ticks,
            preferences: crate::GuestPreferences::default(),
            satisfaction: 70,
            thought: crate::GuestThought::Exploring,
            thought_tick: 0,
            ratings_given: 0,
            rating_total: 0,
            last_rating: None,
        }
    }
}

fn migrate_guest_experience<G: Into<Guest>>(guests: Vec<G>, seed: Option<u64>) -> Vec<Guest> {
    guests
        .into_iter()
        .map(|guest| {
            let mut guest = guest.into();
            guest.preferences = crate::GuestPreferences::for_guest(seed.unwrap_or(0), guest.id);
            guest.needs = crate::GuestNeeds::for_guest(seed.unwrap_or(0), guest.id);
            guest
        })
        .collect()
}

impl<O: Into<SurfaceObject>, G: Into<Guest>, R: Into<crate::Ride>> VersionSevenParkData<O, G, R> {
    fn migrate_services(self) -> ParkData {
        let guests = self
            .guests
            .into_iter()
            .map(|guest| {
                let mut guest = guest.into();
                guest.needs = crate::GuestNeeds::for_guest(self.crowd_seed.unwrap_or(0), guest.id);
                guest
            })
            .collect();
        ParkData {
            width: self.width,
            height: self.height,
            tiles: self.tiles,
            ticks: self.ticks,
            objects: self.objects.into_iter().map(Into::into).collect(),
            next_object_id: self.next_object_id,
            guests,
            next_guest_id: self.next_guest_id,
            crowd_seed: self.crowd_seed,
            crowd_rng: self.crowd_rng,
            circuit: self.circuit,
            train: self.train,
            ride: self.ride.map(Into::into),
            next_ride_id: self.next_ride_id,
            cash_cents: self.cash_cents,
            services: Vec::new(),
            next_service_id: 1,
            finances: crate::FinanceLedger::default(),
            carousels: Vec::new(),
        }
    }
    fn migrate_experience(self) -> ParkData {
        ParkData {
            width: self.width,
            height: self.height,
            tiles: self.tiles,
            ticks: self.ticks,
            objects: self.objects.into_iter().map(Into::into).collect(),
            next_object_id: self.next_object_id,
            guests: migrate_guest_experience(self.guests, self.crowd_seed),
            next_guest_id: self.next_guest_id,
            crowd_seed: self.crowd_seed,
            crowd_rng: self.crowd_rng,
            circuit: self.circuit,
            train: self.train,
            ride: self.ride.map(Into::into),
            next_ride_id: self.next_ride_id,
            cash_cents: self.cash_cents,
            services: Vec::new(),
            next_service_id: 1,
            finances: crate::FinanceLedger::default(),
            carousels: Vec::new(),
        }
    }
}

impl<O: Into<SurfaceObject>, G: Into<Guest>, R: Into<crate::Ride>> VersionFiveParkData<O, G, R> {
    fn migrate(self) -> ParkData {
        ParkData {
            width: self.width,
            height: self.height,
            tiles: self.tiles,
            ticks: self.ticks,
            objects: self.objects.into_iter().map(Into::into).collect(),
            next_object_id: self.next_object_id,
            guests: migrate_guest_experience(self.guests, self.crowd_seed),
            next_guest_id: self.next_guest_id,
            crowd_seed: self.crowd_seed,
            crowd_rng: self.crowd_rng,
            circuit: self.circuit,
            train: self.train,
            ride: self.ride.map(Into::into),
            next_ride_id: self.next_ride_id,
            cash_cents: crate::INITIAL_PARK_CASH_CENTS,
            services: Vec::new(),
            next_service_id: 1,
            finances: crate::FinanceLedger::default(),
            carousels: Vec::new(),
        }
    }
}

/// The strict pre-ride park shape: new ride fields are not accepted in old saves.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "O: Deserialize<'de>, G: Deserialize<'de>"))]
struct LegacyParkData<O, G> {
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<O>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<G>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionTwoObject {
    id: ObjectId,
    #[serde(deserialize_with = "strict_object")]
    pos: crate::TilePos,
    #[serde(deserialize_with = "string_enum")]
    kind: LegacyObjectKind,
    #[serde(deserialize_with = "string_enum")]
    axis: crate::Axis,
    #[serde(deserialize_with = "string_enum")]
    path_slope: crate::PathSlope,
    height: u8,
    clearance: u8,
}

#[derive(Deserialize)]
enum LegacyObjectKind {
    Path,
    Track,
}

impl From<LegacyObjectKind> for ObjectKind {
    fn from(kind: LegacyObjectKind) -> Self {
        match kind {
            LegacyObjectKind::Path => Self::Path,
            LegacyObjectKind::Track => Self::Track,
        }
    }
}

impl From<LegacyObject> for SurfaceObject {
    fn from(object: LegacyObject) -> Self {
        Self {
            id: object.id,
            pos: object.pos,
            kind: object.kind.into(),
            axis: object.axis,
            path_slope: crate::PathSlope::Flat,
            queue_owner: None,
            height: object.height,
            clearance: object.clearance,
        }
    }
}

impl From<VersionTwoObject> for SurfaceObject {
    fn from(object: VersionTwoObject) -> Self {
        Self {
            id: object.id,
            pos: object.pos,
            kind: object.kind.into(),
            axis: object.axis,
            path_slope: object.path_slope,
            queue_owner: None,
            height: object.height,
            clearance: object.clearance,
        }
    }
}

impl From<LegacyGuest> for Guest {
    fn from(guest: LegacyGuest) -> Self {
        Self {
            id: guest.id,
            current: guest.current,
            next: guest.next,
            destination: guest.destination,
            anchor: guest.anchor,
            height: guest.height,
            anchor_slope: crate::PathSlope::Flat,
            progress: guest.progress,
            state: guest.state,
            arrivals: guest.arrivals,
            rides_completed: 0,
            wallet_cents: crate::INITIAL_GUEST_WALLET_CENTS,
            spent_cents: 0,
            service_spent_cents: 0,
            services_used: 0,
            ride_history: Vec::new(),
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
            route: guest.route,
            wait_ticks: guest.wait_ticks,
        }
    }
}

impl<O: Into<SurfaceObject>, G: Into<Guest>> LegacyParkData<O, G> {
    fn migrate(self) -> ParkData {
        let ride = self
            .train
            .as_ref()
            .map(|_| crate::Ride::new(crate::RideId(1)));
        let next_ride_id = if ride.is_some() { 2 } else { 1 };
        ParkData {
            width: self.width,
            height: self.height,
            tiles: self.tiles,
            ticks: self.ticks,
            objects: self.objects.into_iter().map(Into::into).collect(),
            guests: migrate_guest_experience(self.guests, self.crowd_seed),
            next_object_id: self.next_object_id,
            next_guest_id: self.next_guest_id,
            crowd_seed: self.crowd_seed,
            crowd_rng: self.crowd_rng,
            circuit: self.circuit,
            train: self.train,
            ride,
            next_ride_id,
            cash_cents: crate::INITIAL_PARK_CASH_CENTS,
            services: Vec::new(),
            next_service_id: 1,
            finances: crate::FinanceLedger::default(),
            carousels: Vec::new(),
        }
    }
}

fn check_bytes(actual: usize) -> Result<(), SaveError> {
    if actual > MAX_SAVE_BYTES {
        return Err(SaveError::TooLarge {
            actual,
            max: MAX_SAVE_BYTES,
        });
    }
    Ok(())
}

impl ParkData {
    fn initialize_ride_history(&mut self) -> Result<(), SaveError> {
        if let Some(ride) = &self.ride {
            for guest in &mut self.guests {
                guest.ride_history = vec![crate::GuestRideHistory {
                    ride: ride.id,
                    spent_cents: guest
                        .spent_cents
                        .checked_sub(guest.service_spent_cents)
                        .ok_or_else(|| invalid("legacy service spending exceeds total spending"))?,
                    completed: guest.rides_completed,
                    rating_count: guest.ratings_given,
                    rating_total: guest.rating_total,
                }];
            }
        }
        Ok(())
    }
    fn initialize_finances(&mut self) {
        let mut opening_services: Vec<_> = self
            .services
            .iter()
            .map(|service| crate::ServiceFinanceOpening {
                service: service.id,
                revenue_cents: service.revenue_cents,
                served: service.served,
            })
            .collect();
        opening_services.sort_by_key(|entry| entry.service);
        self.finances = crate::FinanceLedger {
            opening_tick: self.ticks,
            opening_cash_cents: self.cash_cents,
            opening_ride_revenue_cents: self.ride.as_ref().map_or(0, |ride| ride.revenue_cents),
            opening_rides: self
                .ride
                .as_ref()
                .map(|ride| crate::RideFinanceTotals {
                    ride: ride.id,
                    revenue_cents: ride.revenue_cents,
                    construction_cents: 0,
                    demolition_refunds_cents: 0,
                })
                .into_iter()
                .collect(),
            opening_services,
            archived_through_tick: self.ticks,
            ..Default::default()
        };
    }
    fn from_park(park: &Park) -> Self {
        Self {
            width: park.width,
            height: park.height,
            tiles: park.tiles.clone(),
            ticks: park.ticks,
            objects: park.objects.clone(),
            next_object_id: park.next_object_id,
            guests: park.guests.clone(),
            next_guest_id: park.next_guest_id,
            crowd_seed: park.crowd_seed,
            crowd_rng: park.crowd_rng,
            circuit: park.circuit.as_ref().map(|circuit| CircuitData {
                pieces: circuit
                    .pieces
                    .iter()
                    .zip(&circuit.specs)
                    .map(|(piece, spec)| CircuitPieceData {
                        object: piece.object,
                        spec: *spec,
                    })
                    .collect(),
            }),
            train: park.train.clone(),
            ride: park.ride.clone(),
            next_ride_id: park.next_ride_id,
            cash_cents: park.cash_cents,
            services: park.services.clone(),
            next_service_id: park.next_service_id,
            finances: park.finances.clone(),
            carousels: park.carousels.clone(),
        }
    }

    fn into_park(self) -> Result<Park, SaveError> {
        if !(1..=MAX_MAP_SIZE).contains(&self.width) || !(1..=MAX_MAP_SIZE).contains(&self.height) {
            return Err(invalid("map dimensions are outside 1..=256"));
        }
        if self.tiles.len() != (self.width * self.height) as usize {
            return Err(invalid("tile count does not match map dimensions"));
        }
        if self
            .tiles
            .iter()
            .any(|tile| tile.height > MAX_TERRAIN_HEIGHT)
        {
            return Err(invalid("terrain height exceeds the supported range"));
        }
        let circuit_data = self.circuit;
        let mut park = Park {
            width: self.width,
            height: self.height,
            tiles: self.tiles,
            ticks: self.ticks,
            objects: self.objects,
            next_object_id: self.next_object_id,
            guests: self.guests,
            next_guest_id: self.next_guest_id,
            crowd_seed: self.crowd_seed,
            crowd_rng: self.crowd_rng,
            circuit: None,
            train: self.train,
            ride: self.ride,
            next_ride_id: self.next_ride_id,
            cash_cents: self.cash_cents,
            // Derived navigation is built lazily after validated loading.
            services: self.services,
            next_service_id: self.next_service_id,
            finances: self.finances,
            carousels: self.carousels,
            // Neither its contents nor diagnostic counters belong in JSON.
            path_cache: crate::crowds::PathCache::default(),
        };
        validate_counts(&park)?;
        let objects = validate_objects(&park)?;
        validate_guests(&park, &objects)?;
        park.circuit = circuit_data
            .map(|data| rebuild_circuit(&park, &objects, data))
            .transpose()?;
        validate_train(&park)?;
        crate::carousel::validate_carousels(&park).map_err(invalid)?;
        crate::ride_access::validate_ride_state(&park).map_err(invalid)?;
        let objects = park
            .objects
            .iter()
            .map(|object| (object.id, object))
            .collect();
        crate::admission::validate_admission(&park, &objects).map_err(invalid)?;
        crate::journeys::validate_journeys(&park, &objects).map_err(invalid)?;
        crate::economy::validate_economy(&park).map_err(invalid)?;
        crate::experience::validate_experience(&park).map_err(invalid)?;
        crate::services::validate_services(&park).map_err(invalid)?;
        crate::finances::validate_finances(&park).map_err(invalid)?;
        Ok(park)
    }
}

fn validate_counts(park: &Park) -> Result<(), SaveError> {
    if park.tiles.len() > MAX_TILES
        || park.objects.len() > MAX_OBJECTS
        || park.guests.len() > MAX_GUESTS
        || park
            .objects
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .count()
            > MAX_PATHS
        || park.circuit.as_ref().is_some_and(|circuit| {
            circuit.pieces.len() > MAX_CIRCUIT_PIECES || circuit.specs.len() > MAX_CIRCUIT_PIECES
        })
    {
        return Err(invalid(
            "a simulation collection exceeds the supported save limit",
        ));
    }
    let mut route_ids = 0;
    for guest in &park.guests {
        if guest.route.len() > MAX_PATHS {
            return Err(invalid("a guest route exceeds the path limit"));
        }
        route_ids += guest.route.len() + usize::from(guest.next.is_some());
    }
    if route_ids > MAX_TOTAL_ROUTE_IDS {
        return Err(invalid("total guest route work exceeds the save limit"));
    }
    Ok(())
}

fn validate_objects(park: &Park) -> Result<BTreeMap<ObjectId, &SurfaceObject>, SaveError> {
    let mut objects = BTreeMap::new();
    let mut occupied: BTreeMap<(u32, u32), Vec<(u8, u16)>> = BTreeMap::new();
    for object in &park.objects {
        if object.id.0 == 0 || objects.insert(object.id, object).is_some() {
            return Err(invalid("object identities must be nonzero and unique"));
        }
        let tile = park
            .tile(object.pos)
            .ok_or_else(|| invalid("object lies outside the map"))?;
        let top = u16::from(object.height) + u16::from(object.clearance);
        if object.clearance == 0 || top > u16::from(MAX_TERRAIN_HEIGHT) {
            return Err(invalid("object has an invalid height or clearance"));
        }
        if object.path_slope != crate::PathSlope::Flat
            && (object.kind != ObjectKind::Path
                || object.clearance != crate::SLOPED_PATH_CLEARANCE
                || object.axis != object.path_slope.axis())
        {
            return Err(invalid(
                "sloped paths require a matching axis, one step and three-step clearance",
            ));
        }
        if !crate::surfaces::terrain_allows(
            object.kind,
            object.height,
            object.clearance,
            tile.height,
        ) {
            return Err(invalid("object intersects terrain"));
        }
        occupied
            .entry((object.pos.x, object.pos.y))
            .or_default()
            .push((object.height, top));
    }
    if park.next_object_id == 0
        || objects
            .keys()
            .next_back()
            .is_some_and(|id| id.0 >= park.next_object_id)
    {
        return Err(invalid(
            "next object identity would reuse an existing identity",
        ));
    }
    for intervals in occupied.values_mut() {
        intervals.sort_unstable();
        if intervals
            .windows(2)
            .any(|pair| u16::from(pair[1].0) < pair[0].1)
        {
            return Err(invalid("object occupied spaces overlap"));
        }
    }
    Ok(objects)
}

fn validate_guests(
    park: &Park,
    objects: &BTreeMap<ObjectId, &SurfaceObject>,
) -> Result<(), SaveError> {
    let mut ids = BTreeSet::new();
    if park.crowd_seed.is_none() && (!park.guests.is_empty() || park.crowd_rng != 0) {
        return Err(invalid("crowd state has no initialized random seed"));
    }
    for guest in &park.guests {
        if guest.id.0 == 0 || !ids.insert(guest.id) {
            return Err(invalid("guest identities must be nonzero and unique"));
        }
        if park.tile(guest.anchor).is_none()
            || guest.height >= MAX_TERRAIN_HEIGHT
            || (guest.anchor_slope != crate::PathSlope::Flat
                && guest.height > crate::MAX_SLOPED_PATH_HEIGHT)
            || guest.progress >= WALK_TICKS
            || guest.wait_ticks > 30
        {
            return Err(invalid("guest anchor, progress or wait is out of range"));
        }
        if guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| crate::journeys::passenger_phase(visit.phase))
        {
            continue;
        }
        let path = |id| {
            objects
                .get(&id)
                .copied()
                .filter(|object| guest.can_walk_path(object))
                .ok_or_else(|| invalid("guest references a missing path"))
        };
        if let Some(current) = guest.current {
            let current_path = path(current)?;
            if current_path.pos != guest.anchor
                || current_path.height != guest.height
                || current_path.path_slope != guest.anchor_slope
            {
                return Err(invalid("guest anchor does not match its current path"));
            }
        } else if guest.state != GuestState::Lost {
            return Err(invalid("only a lost guest may have no current path"));
        }
        match guest.state {
            GuestState::Walking => {
                let current = guest
                    .current
                    .ok_or_else(|| invalid("walking guest lacks current path"))?;
                let next = guest
                    .next
                    .ok_or_else(|| invalid("walking guest lacks next path"))?;
                let destination = guest
                    .destination
                    .ok_or_else(|| invalid("walking guest lacks destination"))?;
                if guest.wait_ticks != 0 {
                    return Err(invalid("walking guest still has a dwell timer"));
                }
                let mut visited = BTreeSet::from([current]);
                let mut previous = path(current)?;
                let mut last = next;
                for id in std::iter::once(next).chain(guest.route.iter().copied()) {
                    let following = path(id)?;
                    if !visited.insert(id) || !crate::paths_connect(previous, following) {
                        return Err(invalid("guest route loops or crosses disconnected paths"));
                    }
                    previous = following;
                    last = id;
                }
                if last != destination {
                    return Err(invalid("guest route does not end at its destination"));
                }
            }
            GuestState::Waiting | GuestState::Lost => {
                if guest.next.is_some()
                    || guest.destination.is_some()
                    || !guest.route.is_empty()
                    || guest.progress != 0
                    || guest.state == GuestState::Lost && guest.wait_ticks != 0
                {
                    return Err(invalid("stationary guest retains walking state"));
                }
            }
        }
    }
    if park.next_guest_id == 0 || ids.last().is_some_and(|id| id.0 >= park.next_guest_id) {
        return Err(invalid(
            "next guest identity would reuse an existing identity",
        ));
    }
    Ok(())
}

fn rebuild_circuit(
    park: &Park,
    objects: &BTreeMap<ObjectId, &SurfaceObject>,
    data: CircuitData,
) -> Result<TrainCircuit, SaveError> {
    if data.pieces.len() > MAX_CIRCUIT_PIECES {
        return Err(invalid("circuit exceeds the piece limit"));
    }
    let specs: Vec<_> = data.pieces.iter().map(|piece| piece.spec).collect();
    let geometry = crate::trains::validate_specs(park, &specs)
        .map_err(|error| invalid(format!("circuit: {error}")))?;
    let mut used = BTreeSet::new();
    let mut pieces = Vec::with_capacity(specs.len());
    let mut total_length = 0_u32;
    let mut station_stop = 0;
    for (saved, samples) in data.pieces.into_iter().zip(geometry) {
        if !used.insert(saved.object) {
            return Err(invalid("circuit uses an object more than once"));
        }
        let object = objects
            .get(&saved.object)
            .ok_or_else(|| invalid("circuit references a missing object"))?;
        let (axis, height, clearance) = crate::trains::envelope(saved.spec);
        if object.kind != ObjectKind::Track
            || object.pos != saved.spec.pos
            || object.axis != axis
            || object.height != height
            || object.clearance != clearance
        {
            return Err(invalid("circuit piece does not match its occupied object"));
        }
        let length: u32 = samples
            .windows(2)
            .map(|pair| crate::trains::point_distance(pair[0], pair[1]))
            .sum();
        if length <= crate::trains::MAX_SPEED {
            return Err(invalid("circuit piece is too short"));
        }
        if saved.spec.kind == TrackKind::Station {
            station_stop = total_length
                .checked_add(3 * TRACK_UNITS as u32 / 4)
                .ok_or_else(|| invalid("circuit station distance overflow"))?;
        }
        total_length = total_length
            .checked_add(length)
            .ok_or_else(|| invalid("circuit length overflow"))?;
        pieces.push(TrackPiece {
            object: saved.object,
            kind: saved.spec.kind,
            samples,
            length,
        });
    }
    if total_length < 4 * TRACK_UNITS as u32 {
        return Err(invalid("circuit is too short"));
    }
    Ok(TrainCircuit {
        pieces,
        total_length,
        specs,
        station_stop,
    })
}

fn validate_train(park: &Park) -> Result<(), SaveError> {
    let (circuit, train) = match (&park.circuit, &park.train) {
        (None, None) => return Ok(()),
        (None, Some(train)) => {
            if train.phase != TrainPhase::Fault
                || train.open
                || train.departed
                || train.speed != 0
                || train.distance != 0
                || train.piece_index != 0
                || train.dwell_ticks != 0
                || train.lap_progress != 0
            {
                return Err(invalid(
                    "train without circuit must be stopped in its fault state",
                ));
            }
            return Ok(());
        }
        (Some(_), None) => return Err(invalid("circuit has no train")),
        (Some(circuit), Some(train)) => (circuit, train),
    };
    if train.distance >= circuit.total_length
        || train.lap_progress >= circuit.total_length
        || train.piece_index >= circuit.pieces.len()
        || circuit.locate(train.distance).0 != train.piece_index
        || train.speed > crate::trains::MAX_SPEED
        || train.dwell_ticks > STATION_DWELL_TICKS
    {
        return Err(invalid(
            "train position, piece index, speed or dwell is out of range",
        ));
    }
    if train.departed {
        let expected_phase = match circuit.pieces[train.piece_index].kind {
            TrackKind::Station => TrainPhase::Station,
            TrackKind::Lift => TrainPhase::Lift,
            TrackKind::Drop => TrainPhase::Drop,
            TrackKind::Brake => TrainPhase::Brake,
            TrackKind::Curve | TrackKind::Straight => TrainPhase::Cruising,
        };
        if train.phase != expected_phase
            || train.dwell_ticks != 0
            || train.speed < 20
            || train.lap_progress == 0
            || train.distance != (circuit.station_stop + train.lap_progress) % circuit.total_length
        {
            return Err(invalid(
                "moving train phase/progress does not match its circuit",
            ));
        }
    } else {
        let expected_phase = if train.open {
            TrainPhase::Station
        } else {
            TrainPhase::Closed
        };
        if train.phase != expected_phase
            || train.distance != circuit.station_stop
            || train.lap_progress != 0
            || train.speed != 0
            || !train.open && train.dwell_ticks != 0
        {
            return Err(invalid("parked train must occupy its station stop"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, Speed, TerrainProfile, TilePos};
    use serde_json::{Value, json};

    fn fixture() -> Park {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(20_261_020).unwrap();
        park.install_train_fixture().unwrap();
        park
    }

    fn advance(park: &mut Park, ticks: u32) {
        for _ in 0..ticks {
            park.advance(Speed::Normal);
        }
    }

    fn value(park: &Park) -> Value {
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap()
    }

    fn decode(value: &Value) -> Result<Park, SaveError> {
        Park::from_save_bytes(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn v9_migration_attributes_paid_passengers_and_archived_cash_without_rewriting_history() {
        let mut source = crate::experience::tests::fixture(true, 200);
        for (x, kind) in [
            (2, crate::ServiceKind::Food),
            (3, crate::ServiceKind::Drink),
        ] {
            source
                .place_service(TilePos { x, y: 24 }, 8, crate::PathDirection::North, kind)
                .unwrap();
        }
        let paid = |park: &Park| {
            park.guests.iter().any(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .and_then(|visit| visit.passenger.as_ref())
                    .is_some_and(|p| p.fare_paid && !p.completed_lap)
            })
        };
        for _ in 0..24_000 {
            if source.finances.archived_entries > 0 && paid(&source) {
                break;
            }
            source.advance(Speed::Normal);
        }
        assert!(source.finances.archived_entries > 0 && paid(&source));
        let mut old = value(&source);
        crate::finances::tests::strip_carousel_fields(&mut old);
        old["schema_version"] = json!(9);
        old["content_version"] = json!(9);
        let bytes = serde_json::to_vec(&old).unwrap();
        let migrated = Park::from_save_bytes(&bytes).unwrap();
        let mut projection = value(&migrated);
        crate::finances::tests::strip_carousel_fields(&mut projection);
        projection["schema_version"] = json!(9);
        projection["content_version"] = json!(9);
        assert_eq!(projection, old);
        assert!(migrated.carousels.is_empty());
        let id = migrated.ride().unwrap().id;
        for (old, guest) in source.guests.iter().zip(&migrated.guests) {
            assert_eq!(
                guest.ride_history,
                vec![crate::GuestRideHistory {
                    ride: id,
                    spent_cents: old.spent_cents - old.service_spent_cents,
                    completed: old.rides_completed,
                    rating_count: old.ratings_given,
                    rating_total: old.rating_total,
                }]
            );
        }
        for entry in &migrated.finances.entries {
            assert_eq!(
                entry.ride,
                (entry.kind == crate::FinanceKind::RideRevenue).then_some(id)
            );
        }
        assert_continuation(migrated, 800);
        for (path, key, added) in [
            ("/park", "carousels", json!([])),
            ("/park/guests/0", "ride_history", json!([])),
            ("/park/finances", "opening_rides", json!([])),
            ("/park/finances/entries/0", "ride", Value::Null),
        ] {
            let mut forged = old.clone();
            forged.pointer_mut(path).unwrap()[key] = added;
            assert!(
                decode(&forged).is_err(),
                "legacy accepted new field {path}/{key}"
            );
        }
        assert_eq!(serde_json::to_vec(&old).unwrap(), bytes);
    }

    #[test]
    fn current_carousel_and_owner_fields_are_required_bounded_and_strict() {
        let mut park = fixture();
        let id = park.place_carousel(TilePos { x: 2, y: 24 }, 8).unwrap();
        assert_continuation(park.clone(), 100);
        park.remove_carousel(id).unwrap();
        assert_continuation(park.clone(), 100);
        let base = value(&park);
        let mut missing_carousels = base.clone();
        missing_carousels["park"]
            .as_object_mut()
            .unwrap()
            .remove("carousels");
        assert!(decode(&missing_carousels).is_err());
        for field in ["opening_rides", "archived_rides"] {
            let mut forged = base.clone();
            forged["park"]["finances"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(decode(&forged).is_err());
        }
        let mut missing = base.clone();
        missing["park"]["guests"][0]
            .as_object_mut()
            .unwrap()
            .remove("ride_history");
        assert!(decode(&missing).is_err());
        let mut too_many = base.clone();
        too_many["park"]["carousels"] = json!(vec![base["park"]["carousels"][0].clone(); 9]);
        assert!(decode(&too_many).is_err());
        let mut mixed = base;
        mixed["schema_version"] = json!(9);
        assert!(decode(&mixed).is_err());
    }

    fn legacy_value(park: &Park) -> Value {
        let mut data = version_two_value(park);
        data["schema_version"] = json!(1);
        data["content_version"] = json!(1);
        for object in data["park"]["objects"].as_array_mut().unwrap() {
            assert_eq!(object["path_slope"], json!("Flat"));
            object.as_object_mut().unwrap().remove("path_slope");
        }
        for guest in data["park"]["guests"].as_array_mut().unwrap() {
            assert_eq!(guest["anchor_slope"], json!("Flat"));
            guest.as_object_mut().unwrap().remove("anchor_slope");
        }
        data
    }

    fn version_two_value(park: &Park) -> Value {
        let mut data = version_three_value(park);
        assert!(park.ride().is_none_or(|ride| ride.entrance.is_none()));
        data["schema_version"] = json!(2);
        data["content_version"] = json!(2);
        data["park"].as_object_mut().unwrap().remove("ride");
        data["park"].as_object_mut().unwrap().remove("next_ride_id");
        for object in data["park"]["objects"].as_array_mut().unwrap() {
            assert_eq!(object["queue_owner"], Value::Null);
            object.as_object_mut().unwrap().remove("queue_owner");
        }
        data
    }

    fn version_three_value(park: &Park) -> Value {
        let mut data = version_four_value(park);
        data["schema_version"] = json!(3);
        data["content_version"] = json!(3);
        for guest in data["park"]["guests"].as_array_mut().unwrap() {
            assert_eq!(guest["ride_visit"], Value::Null);
            assert_eq!(guest["ride_cooldown"], json!(0));
            guest.as_object_mut().unwrap().remove("ride_visit");
            guest.as_object_mut().unwrap().remove("ride_cooldown");
        }
        if let Some(ride) = data["park"]["ride"].as_object_mut() {
            for field in [
                "draining",
                "next_queue_ticket",
                "joined",
                "turned_away",
                "abandoned",
            ] {
                ride.remove(field);
            }
        }
        data
    }

    fn version_four_value(park: &Park) -> Value {
        let mut data = version_five_value(park);
        data["schema_version"] = json!(4);
        data["content_version"] = json!(4);
        for guest in data["park"]["guests"].as_array_mut().unwrap() {
            assert_eq!(guest["rides_completed"], json!(0));
            guest.as_object_mut().unwrap().remove("rides_completed");
            if let Some(visit) = guest["ride_visit"].as_object_mut() {
                assert_eq!(visit["passenger"], Value::Null);
                visit.remove("passenger");
            }
        }
        if let Some(ride) = data["park"]["ride"].as_object_mut() {
            assert_eq!(ride["exit"], Value::Null);
            for field in ["exit", "boarded", "exited", "completed"] {
                ride.remove(field);
            }
        }
        data
    }

    fn version_five_value(park: &Park) -> Value {
        let mut data = crate::experience::tests::strip_feedback(value(park));
        data["schema_version"] = json!(5);
        data["content_version"] = json!(5);
        assert_eq!(
            data["park"]["cash_cents"],
            json!(crate::INITIAL_PARK_CASH_CENTS)
        );
        data["park"].as_object_mut().unwrap().remove("cash_cents");
        for guest in data["park"]["guests"].as_array_mut().unwrap() {
            assert_eq!(guest["spent_cents"], json!(0));
            guest.as_object_mut().unwrap().remove("wallet_cents");
            guest.as_object_mut().unwrap().remove("spent_cents");
            if let Some(passenger) = guest
                .pointer_mut("/ride_visit/passenger")
                .and_then(Value::as_object_mut)
            {
                assert_eq!(passenger["fare_cents"], json!(0));
                passenger.remove("fare_cents");
                passenger.remove("fare_paid");
            }
        }
        if let Some(ride) = data["park"]["ride"].as_object_mut() {
            for field in [
                "ticket_price_cents",
                "revenue_cents",
                "unaffordable_rejections",
            ] {
                assert_eq!(ride[field], json!(0));
                ride.remove(field);
            }
        }
        data
    }

    fn assert_continuation(mut original: Park, ticks: u32) {
        let bytes = original.to_save_bytes().unwrap();
        let mut loaded = Park::from_save_bytes(&bytes).unwrap();
        assert_eq!(loaded, original);
        assert_eq!(loaded.to_save_bytes().unwrap(), bytes);
        for index in 0..ticks {
            let speed =
                [Speed::Paused, Speed::Normal, Speed::Fast, Speed::Fastest][index as usize % 4];
            original.advance(speed);
            loaded.advance(speed);
            assert_eq!(loaded, original);
        }
        assert_eq!(
            loaded.to_save_bytes().unwrap(),
            original.to_save_bytes().unwrap()
        );
    }

    #[test]
    fn empty_small_rectangular_and_maximum_maps_roundtrip_exactly() {
        for (width, height) in [(1, 1), (7, 3), (256, 256)] {
            let mut park = Park::new(width, height);
            park.apply_terrain_profile(TerrainProfile::Terraced);
            park.set_terrain_height(TilePos { x: 0, y: 0 }, 32).unwrap();
            assert_continuation(park, 8);
        }
    }

    #[test]
    fn legacy_flat_migration_preserves_live_lost_faulted_and_empty_continuation() {
        let mut live = fixture();
        advance(&mut live, 47);
        assert!(
            live.guests
                .iter()
                .any(|guest| guest.state == GuestState::Walking && guest.progress > 0)
        );
        live.set_train_open(false);
        // Vector order is authoritative and need not follow stable ID order.
        live.objects.reverse();
        let mut lost = live.clone();
        let current = lost.guests[0].current.unwrap();
        lost.remove_object(current).unwrap();
        assert_eq!(lost.guests[0].current, None);
        let track = lost.circuit().unwrap().pieces[4].object;
        lost.remove_object(track).unwrap();
        assert_eq!(lost.train().unwrap().phase, TrainPhase::Fault);
        for mut original in [Park::new(2, 3), fixture(), live, lost] {
            crate::finances::tests::reset_finances(&mut original);
            let old_data = legacy_value(&original);
            let old_bytes = serde_json::to_vec(&old_data).unwrap();
            let mut migrated = Park::from_save_bytes(&old_bytes).unwrap();
            assert_eq!(migrated, original);
            let modern = value(&migrated);
            assert_eq!(modern["schema_version"], json!(11));
            assert_eq!(modern["content_version"], json!(11));
            assert_eq!(serde_json::to_vec(&old_data).unwrap(), old_bytes);
            for index in 0..600 {
                let speed = [Speed::Paused, Speed::Normal, Speed::Fast, Speed::Fastest][index % 4];
                original.advance(speed);
                migrated.advance(speed);
                assert_eq!(migrated, original);
            }
            assert_eq!(
                migrated.to_save_bytes().unwrap(),
                original.to_save_bytes().unwrap()
            );
        }
    }

    #[test]
    fn version_two_migration_preserves_slope_walks_lost_anchors_faults_and_ride_identity() {
        let mut live = fixture();
        let low = live.place_path(TilePos { x: 2, y: 24 }, 8).unwrap();
        let slope = live
            .place_sloped_path(TilePos { x: 3, y: 24 }, 8, crate::PathSlope::East)
            .unwrap();
        let high = live.place_path(TilePos { x: 4, y: 24 }, 9).unwrap();
        let guest = &mut live.guests[0];
        guest.current = Some(low);
        guest.next = Some(slope);
        guest.destination = Some(high);
        guest.anchor = TilePos { x: 2, y: 24 };
        guest.height = 8;
        guest.anchor_slope = crate::PathSlope::Flat;
        guest.state = GuestState::Walking;
        guest.progress = 7;
        guest.wait_ticks = 0;
        guest.route = VecDeque::from([high]);
        advance(&mut live, 30);
        assert_eq!(live.guests[0].current, Some(slope));
        assert_eq!(live.guests[0].anchor_slope, crate::PathSlope::East);
        let mut lost = live.clone();
        lost.remove_object(slope).unwrap();
        let track = lost.circuit().unwrap().pieces[0].object;
        lost.remove_object(track).unwrap();
        for mut original in [live, lost] {
            crate::finances::tests::reset_finances(&mut original);
            let legacy = version_two_value(&original);
            let migrated = decode(&legacy).unwrap();
            assert_eq!(migrated, original);
            assert_eq!(migrated.ride().unwrap().id, crate::RideId(1));
            assert!(migrated.ride().unwrap().entrance.is_none());
            assert_continuation(migrated, 600);
            for (path, name, added) in [
                ("/park", "ride", Value::Null),
                ("/park", "next_ride_id", json!(2)),
                ("/park/objects/0", "queue_owner", Value::Null),
            ] {
                let mut malformed = legacy.clone();
                malformed.pointer_mut(path).unwrap()[name] = added;
                assert!(decode(&malformed).is_err(), "old schema accepted {name}");
            }
        }
    }

    #[test]
    fn version_four_migrates_live_queue_and_cancelled_visits_without_changing_future() {
        let mut park = fixture();
        let ride = park.ride().unwrap().id;
        park.place_ride_entrance(
            TilePos { x: 8, y: 11 },
            8,
            crate::PathDirection::North,
            ride,
        )
        .unwrap();
        for (y, height, slope) in [
            (12, 8, crate::PathSlope::Flat),
            (13, 8, crate::PathSlope::South),
            (14, 9, crate::PathSlope::Flat),
            (15, 8, crate::PathSlope::North),
        ] {
            park.place_queue_path(TilePos { x: 8, y }, height, slope, ride)
                .unwrap();
        }
        advance(&mut park, 10);
        let seeking = park.clone();
        for _ in 0..2000 {
            if park.guests.iter().any(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == crate::RideVisitPhase::Queueing)
                    && guest.progress == 7
            }) {
                break;
            }
            park.advance(Speed::Normal);
        }
        assert!(park.guests.iter().any(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == crate::RideVisitPhase::Queueing)
        }));
        let moving = park.clone();
        park.set_train_open(false);
        let leaving = park.clone();
        let removed = park
            .guests
            .iter()
            .find(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.ticket.is_some())
            })
            .and_then(|guest| guest.current)
            .unwrap();
        park.remove_object(removed).unwrap();
        for mut original in [seeking, moving, leaving, park] {
            crate::experience::tests::reset_feedback(&mut original);
            let legacy = version_four_value(&original);
            let mut migrated = decode(&legacy).unwrap();
            assert_eq!(original, migrated);
            for _ in 0..600 {
                original.advance(Speed::Normal);
                migrated.advance(Speed::Normal);
                assert_eq!(original, migrated);
            }
            assert_eq!(
                original.to_save_bytes().unwrap(),
                migrated.to_save_bytes().unwrap()
            );
            for (path, field, value) in [
                ("/park/ride", "exit", Value::Null),
                ("/park/ride", "boarded", json!(0)),
                ("/park/guests/0", "rides_completed", json!(0)),
            ] {
                let mut malformed = legacy.clone();
                malformed.pointer_mut(path).unwrap()[field] = value;
                assert!(decode(&malformed).is_err());
            }
            if let Some(index) = legacy["park"]["guests"]
                .as_array()
                .unwrap()
                .iter()
                .position(|guest| !guest["ride_visit"].is_null())
            {
                let mut malformed = legacy.clone();
                malformed["park"]["guests"][index]["ride_visit"]["passenger"] = Value::Null;
                assert!(decode(&malformed).is_err());
                let mut malformed = legacy.clone();
                malformed["park"]["guests"][index]["ride_visit"]["phase"] = json!("Boarding");
                assert!(decode(&malformed).is_err());
            }
        }
        let current = value(&fixture());
        for (schema, content) in [
            (5, 4),
            (4, 5),
            (5, 3),
            (3, 5),
            (5, 2),
            (2, 5),
            (5, 1),
            (1, 5),
        ] {
            let mut invalid = current.clone();
            invalid["schema_version"] = json!(schema);
            invalid["content_version"] = json!(content);
            assert!(decode(&invalid).is_err());
        }
    }

    #[test]
    fn version_three_migrates_queues_and_faults_without_inventing_visits() {
        let mut live = fixture();
        advance(&mut live, 47);
        let ride = live.ride().unwrap().id;
        live.place_ride_entrance(
            TilePos { x: 8, y: 11 },
            8,
            crate::PathDirection::North,
            ride,
        )
        .unwrap();
        for y in 12..=15 {
            live.place_queue_path(TilePos { x: 8, y }, 8, crate::PathSlope::Flat, ride)
                .unwrap();
        }
        let mut fault = live.clone();
        fault
            .remove_object(fault.circuit().unwrap().pieces[0].object)
            .unwrap();
        let current = fault.guests[0].current.unwrap();
        fault.remove_object(current).unwrap();
        for mut original in [live, fault] {
            crate::finances::tests::reset_finances(&mut original);
            let legacy = version_three_value(&original);
            let mut migrated = decode(&legacy).unwrap();
            assert_eq!(migrated, original);
            for _ in 0..900 {
                original.advance(Speed::Normal);
                migrated.advance(Speed::Normal);
                assert_eq!(migrated, original);
            }
            assert_eq!(
                migrated.to_save_bytes().unwrap(),
                original.to_save_bytes().unwrap()
            );
            for (path, field, added) in [
                ("/park/guests/0", "ride_visit", Value::Null),
                ("/park/guests/0", "ride_cooldown", json!(0)),
                ("/park/ride", "joined", json!(0)),
            ] {
                let mut malformed = legacy.clone();
                malformed.pointer_mut(path).unwrap()[field] = added;
                assert!(decode(&malformed).is_err(), "old schema accepted {field}");
            }
        }
        let modern = value(&fixture());
        for (schema, content) in [(4, 3), (3, 4), (4, 2), (2, 4), (4, 1), (1, 4)] {
            let mut malformed = modern.clone();
            malformed["schema_version"] = json!(schema);
            malformed["content_version"] = json!(content);
            assert!(decode(&malformed).is_err());
        }
    }

    #[test]
    fn version_three_validates_ride_owners_entrances_and_public_only_guest_references() {
        let mut park = fixture();
        let ride = park.ride().unwrap().id;
        let entrance = park
            .place_ride_entrance(
                TilePos { x: 8, y: 11 },
                8,
                crate::PathDirection::North,
                ride,
            )
            .unwrap();
        let queue = park
            .place_queue_path(TilePos { x: 8, y: 12 }, 8, crate::PathSlope::Flat, ride)
            .unwrap();
        let gate_index = park.objects.iter().position(|o| o.id == entrance).unwrap();
        let queue_index = park.objects.iter().position(|o| o.id == queue).unwrap();
        let track_index = park
            .objects
            .iter()
            .position(|o| o.kind == ObjectKind::Track)
            .unwrap();
        let baseline = value(&park);
        for (path, replacement) in [
            ("/park/ride/id".into(), json!(0)),
            ("/park/next_ride_id".into(), json!(1)),
            ("/park/ride".into(), Value::Null),
            ("/park/ride/entrance/object".into(), json!(u64::MAX)),
            ("/park/ride/entrance/object".into(), json!(queue.0)),
            ("/park/ride/entrance/facing".into(), json!("Up")),
            ("/park/ride/entrance/facing".into(), json!({"North":null})),
            (
                format!("/park/objects/{queue_index}/queue_owner"),
                json!(999),
            ),
            (format!("/park/objects/{queue_index}/clearance"), json!(1)),
            (
                format!("/park/objects/{track_index}/queue_owner"),
                json!(ride.0),
            ),
            (
                format!("/park/objects/{gate_index}/queue_owner"),
                json!(ride.0),
            ),
            (format!("/park/objects/{gate_index}/axis"), json!("X")),
            (format!("/park/objects/{gate_index}/clearance"), json!(3)),
            (
                format!("/park/objects/{gate_index}/path_slope"),
                json!("North"),
            ),
            (format!("/park/objects/{gate_index}/kind"), json!("Path")),
        ] {
            let mut altered = baseline.clone();
            *altered.pointer_mut(&path).unwrap() = replacement;
            assert!(decode(&altered).is_err(), "accepted {path}");
        }
        for (path, field) in [
            ("/park", "ride"),
            ("/park", "next_ride_id"),
            ("/park/ride", "entrance"),
            ("/park/objects/0", "queue_owner"),
        ] {
            let mut altered = baseline.clone();
            altered
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(decode(&altered).is_err(), "missing required {path}/{field}");
        }
        for target in [queue, entrance] {
            let mut altered = baseline.clone();
            let object = park.object(target).unwrap();
            let guest = &mut altered["park"]["guests"][0];
            guest["current"] = json!(target.0);
            guest["anchor"] = json!({"x":object.pos.x,"y":object.pos.y});
            guest["height"] = json!(object.height);
            assert!(
                decode(&altered).is_err(),
                "public guest entered private access object"
            );
        }
        let mut altered = baseline;
        altered["park"]["ride"]["entrance"] = Value::Null;
        assert!(
            decode(&altered).is_err(),
            "orphan physical entrance must reject"
        );
    }

    #[test]
    fn schema_versions_are_strict_pairs_and_slope_fields_are_required_only_in_v2() {
        let park = fixture();
        let legacy = legacy_value(&park);
        let modern = value(&park);
        for (schema, content) in [
            (0, 0),
            (12, 12),
            (10, 9),
            (9, 10),
            (7, 6),
            (6, 7),
            (6, 5),
            (5, 6),
            (1, 2),
            (2, 1),
            (2, 0),
            (1, 9),
            (3, 2),
            (2, 3),
        ] {
            let mut altered = modern.clone();
            altered["schema_version"] = json!(schema);
            altered["content_version"] = json!(content);
            assert!(
                decode(&altered).is_err(),
                "accepted versions {schema}/{content}"
            );
        }
        for (path, field) in [
            ("/park/objects/0", "path_slope"),
            ("/park/guests/0", "anchor_slope"),
        ] {
            let mut old_with_new_field = legacy.clone();
            old_with_new_field.pointer_mut(path).unwrap()[field] = json!("Flat");
            assert!(decode(&old_with_new_field).is_err());
            let mut missing = modern.clone();
            missing
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(decode(&missing).is_err());
            for malformed in [
                json!(null),
                json!("Northeast"),
                json!({"Flat":null}),
                json!(2),
            ] {
                let mut altered = modern.clone();
                altered.pointer_mut(path).unwrap()[field] = malformed;
                assert!(decode(&altered).is_err());
            }
        }
    }

    #[test]
    fn v2_rejects_invalid_slope_shape_track_anchor_and_route_endpoints() {
        let mut park = Park::new(4, 4);
        let low = park.place_path(TilePos { x: 0, y: 1 }, 4).unwrap();
        let slope = park
            .place_sloped_path(TilePos { x: 1, y: 1 }, 4, crate::PathSlope::East)
            .unwrap();
        let high = park.place_path(TilePos { x: 2, y: 1 }, 5).unwrap();
        park.crowd_seed = Some(1);
        park.crowd_rng = 1;
        park.next_guest_id = 2;
        park.guests.push(Guest {
            id: crate::GuestId(1),
            current: Some(low),
            next: Some(slope),
            destination: Some(high),
            anchor: TilePos { x: 0, y: 1 },
            height: 4,
            anchor_slope: crate::PathSlope::Flat,
            progress: 7,
            state: GuestState::Walking,
            arrivals: 0,
            rides_completed: 0,
            wallet_cents: crate::INITIAL_GUEST_WALLET_CENTS,
            spent_cents: 0,
            service_spent_cents: 0,
            services_used: 0,
            ride_history: Vec::new(),
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
            route: VecDeque::from([high]),
            wait_ticks: 0,
        });
        let baseline = value(&park);
        for (path, replacement) in [
            ("/park/objects/1/kind", json!("Track")),
            ("/park/objects/1/clearance", json!(2)),
            ("/park/objects/1/clearance", json!(4)),
            ("/park/objects/1/axis", json!("Y")),
            ("/park/objects/1/height", json!(30)),
            ("/park/objects/2/height", json!(4)),
            ("/park/guests/0/anchor_slope", json!("East")),
        ] {
            let mut altered = baseline.clone();
            *altered.pointer_mut(path).unwrap() = replacement;
            assert!(decode(&altered).is_err(), "accepted {path}");
        }
        let mut side = baseline;
        side["park"]["objects"][1]["path_slope"] = json!("North");
        side["park"]["objects"][1]["axis"] = json!("Y");
        assert!(
            decode(&side).is_err(),
            "wrong-side routes must fail even with valid slope shape"
        );
        park.remove_object(low).unwrap();
        let mut lost = value(&park);
        lost["park"]["guests"][0]["anchor_slope"] = json!("East");
        lost["park"]["guests"][0]["height"] = json!(30);
        assert!(decode(&lost).is_err());
    }

    #[test]
    fn complete_state_roundtrips_at_station_travel_pending_close_and_closed() {
        let initial = fixture();
        assert_continuation(initial.clone(), 140);
        let mut dwelling = initial.clone();
        advance(&mut dwelling, 17);
        assert_continuation(dwelling, 100);
        let mut travelling = initial.clone();
        advance(&mut travelling, 450);
        assert!(travelling.train().unwrap().departed);
        assert_continuation(travelling.clone(), 700);
        travelling.set_train_open(false);
        assert_continuation(travelling.clone(), 1_500);
        advance(&mut travelling, 3_600);
        assert_eq!(travelling.train().unwrap().phase, TrainPhase::Closed);
        assert_continuation(travelling, 100);
    }

    #[test]
    fn lost_deleted_and_restored_paths_preserve_rng_routes_and_id_allocation() {
        let mut original = fixture();
        advance(&mut original, 600);
        let removed = original.guests()[0].current.unwrap();
        original.remove_object(removed).unwrap();
        assert_eq!(original.guests()[0].current, None);
        assert_eq!(original.guests()[0].state, GuestState::Lost);
        assert_continuation(original.clone(), 250);
        let mut loaded = Park::from_save_bytes(&original.to_save_bytes().unwrap()).unwrap();
        original.restore_crowd_paths().unwrap();
        loaded.restore_crowd_paths().unwrap();
        assert_eq!(loaded, original);
        assert!(original.object(removed).is_none());
        assert_continuation(original, 600);
    }

    #[test]
    fn isolated_lost_guest_and_faulted_train_survive_roundtrip_and_repair() {
        let mut park = fixture();
        advance(&mut park, 1_800);
        let current = park.guests()[0].current.unwrap();
        for id in park.path_neighbors(current) {
            park.remove_object(id).unwrap();
        }
        advance(&mut park, 90);
        assert!(park.crowd_stats().lost > 0);
        let track = park.circuit().unwrap().pieces[12].object;
        park.remove_object(track).unwrap();
        assert_eq!(park.train().unwrap().phase, TrainPhase::Fault);
        assert_continuation(park.clone(), 90);
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        park.install_train_fixture().unwrap();
        loaded.install_train_fixture().unwrap();
        assert_eq!(park, loaded);
        assert_eq!(loaded.train().unwrap().phase, TrainPhase::Closed);
    }

    #[test]
    fn allocator_gaps_and_saturated_counters_are_preserved() {
        let mut park = fixture();
        let id = park
            .place_object(TilePos { x: 2, y: 2 }, ObjectKind::Path, Axis::X, 0, 2)
            .unwrap();
        park.remove_object(id).unwrap();
        park.ticks = u64::MAX;
        park.guests[0].arrivals = u64::MAX;
        park.train.as_mut().unwrap().laps = u64::MAX;
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(park, loaded);
        loaded.advance(Speed::Fastest);
        assert_eq!(park, loaded);
        let new_id = loaded
            .place_object(TilePos { x: 2, y: 2 }, ObjectKind::Path, Axis::X, 0, 2)
            .unwrap();
        assert!(new_id.0 > id.0);
    }

    #[test]
    fn circuit_caches_are_omitted_and_rebuilt_without_resetting_train() {
        let mut park = fixture();
        advance(&mut park, 600);
        let data = value(&park);
        let piece = &data["park"]["circuit"]["pieces"][0];
        assert!(piece.get("samples").is_none());
        assert!(piece.get("length").is_none());
        assert!(data["park"]["circuit"].get("station_stop").is_none());
        assert!(data["park"]["circuit"].get("total_length").is_none());
        let loaded = decode(&data).unwrap();
        assert_eq!(loaded.circuit(), park.circuit());
        assert_eq!(loaded.train(), park.train());
        assert_eq!(loaded, park);
        let mut invalid_cache = park;
        invalid_cache.circuit.as_mut().unwrap().total_length += 1;
        assert!(invalid_cache.to_save_bytes().is_err());
    }

    #[test]
    fn malformed_truncated_duplicate_unknown_and_unsupported_json_reject() {
        let park = fixture();
        let bytes = park.to_save_bytes().unwrap();
        for invalid_bytes in [
            &b""[..],
            &b"{"[..],
            &b"null"[..],
            &b"\xff"[..],
            &bytes[..bytes.len() - 1],
            &b"[]"[..],
        ] {
            assert!(Park::from_save_bytes(invalid_bytes).is_err());
        }
        let text = String::from_utf8(bytes.clone()).unwrap();
        let duplicate = text.replacen(
            "\"schema_version\":11",
            "\"schema_version\":11,\"schema_version\":11",
            1,
        );
        assert!(Park::from_save_bytes(duplicate.as_bytes()).is_err());
        assert!(Park::from_save_bytes(format!("{text} {{}}").as_bytes()).is_err());
        let too_big_integer = text.replacen("\"ticks\":0", "\"ticks\":18446744073709551616", 1);
        assert!(Park::from_save_bytes(too_big_integer.as_bytes()).is_err());
        for (field, replacement, expected) in [
            (
                "schema_version",
                json!(999),
                SaveError::UnsupportedSchema(999),
            ),
            (
                "content_version",
                json!(CONTENT_VERSION + 1),
                SaveError::UnsupportedContent(CONTENT_VERSION + 1),
            ),
        ] {
            let mut altered = value(&park);
            altered[field] = replacement;
            assert_eq!(decode(&altered), Err(expected));
        }
        let mut altered = value(&park);
        altered["format"] = json!("another-game");
        assert!(decode(&altered).is_err());
    }

    #[test]
    fn unknown_nested_fields_and_missing_nullable_fields_are_rejected() {
        let baseline = value(&fixture());
        let positional_envelope =
            json!([FORMAT, SCHEMA_VERSION, CONTENT_VERSION, baseline["park"]]);
        assert!(decode(&positional_envelope).is_err());
        for (path, positional) in [
            (
                "/park/tiles/0",
                json!([baseline["park"]["tiles"][0]["terrain"], 0]),
            ),
            ("/park/objects/0/pos", json!([8, 16])),
            (
                "/park/guests/0/anchor",
                json!([
                    baseline["park"]["guests"][0]["anchor"]["x"],
                    baseline["park"]["guests"][0]["anchor"]["y"]
                ]),
            ),
            ("/park/circuit/pieces/0/spec/entry", json!(["West", 8])),
            ("/park/tiles/0/terrain", json!({"Grass": null})),
            ("/park/train/phase", json!({"Station": null})),
        ] {
            let mut altered = baseline.clone();
            *altered.pointer_mut(path).unwrap() = positional;
            assert!(
                decode(&altered).is_err(),
                "non-schema JSON representation at {path}"
            );
        }
        for path in [
            "",
            "/park",
            "/park/tiles/0",
            "/park/objects/0",
            "/park/objects/0/pos",
            "/park/guests/0",
            "/park/guests/0/anchor",
            "/park/train",
            "/park/circuit",
            "/park/circuit/pieces/0",
            "/park/circuit/pieces/0/spec",
            "/park/circuit/pieces/0/spec/entry",
        ] {
            let mut altered = baseline.clone();
            altered
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), json!(1));
            assert!(decode(&altered).is_err(), "unknown field at {path}");
        }
        for (path, field) in [
            ("/park", "crowd_seed"),
            ("/park", "circuit"),
            ("/park", "train"),
            ("/park/guests/0", "current"),
            ("/park/guests/0", "next"),
            ("/park/guests/0", "destination"),
        ] {
            let mut altered = baseline.clone();
            altered
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(decode(&altered).is_err(), "missing nullable {path}/{field}");
        }
    }

    #[test]
    fn numeric_dimensions_object_ids_and_clearances_validate_before_use() {
        let baseline = value(&fixture());
        for (path, replacement) in [
            ("/park/width", json!(0)),
            ("/park/height", json!(257)),
            ("/park/ticks", json!(-1)),
            ("/park/ticks", json!(1.25)),
            ("/park/tiles/0/height", json!(33)),
            ("/park/tiles/0/terrain", json!("Lava")),
            ("/park/objects/0/id", json!(0)),
            (
                "/park/objects/1/id",
                baseline["park"]["objects"][0]["id"].clone(),
            ),
            ("/park/next_object_id", json!(1)),
            ("/park/objects/0/pos/x", json!(32)),
            ("/park/objects/0/height", json!(31)),
            ("/park/objects/0/clearance", json!(0)),
            ("/park/objects/0/axis", json!("Z")),
            ("/park/objects/0/kind", json!("Support")),
            (
                "/park/objects/1/pos",
                baseline["park"]["objects"][0]["pos"].clone(),
            ),
        ] {
            let mut altered = baseline.clone();
            *altered.pointer_mut(path).unwrap() = replacement;
            assert!(decode(&altered).is_err(), "accepted {path}");
        }
        let mut altered = baseline.clone();
        altered["park"]["tiles"].as_array_mut().unwrap().pop();
        assert!(decode(&altered).is_err());
        let mut altered = baseline;
        altered["park"]["tiles"][16 * 32 + 8]["height"] = json!(9);
        assert!(
            decode(&altered).is_err(),
            "terrain must not intersect path clearance"
        );
    }

    #[test]
    fn guest_references_route_connectivity_and_state_are_validated() {
        let mut park = fixture();
        let index = (0..300)
            .find_map(|_| {
                advance(&mut park, 1);
                park.guests
                    .iter()
                    .position(|guest| guest.state == GuestState::Walking && !guest.route.is_empty())
            })
            .expect("a seeded guest must choose a route longer than one edge");
        let baseline = value(&park);
        let path = format!("/park/guests/{index}");
        for (field, replacement) in [
            ("id", json!(0)),
            ("current", json!(u64::MAX)),
            ("current", Value::Null),
            ("next", Value::Null),
            ("destination", Value::Null),
            (
                "destination",
                baseline["park"]["guests"][index]["current"].clone(),
            ),
            ("height", json!(7)),
            ("progress", json!(WALK_TICKS)),
            ("wait_ticks", json!(1)),
            ("state", json!("Waiting")),
            ("state", json!("Unknown")),
            ("anchor", json!({"x":32,"y":0})),
        ] {
            let mut altered = baseline.clone();
            altered.pointer_mut(&path).unwrap()[field] = replacement;
            assert!(decode(&altered).is_err(), "accepted guest {field}");
        }
        let mut altered = baseline.clone();
        altered["park"]["guests"][index]["route"][0] =
            altered["park"]["guests"][index]["current"].clone();
        assert!(decode(&altered).is_err());
        let mut altered = baseline.clone();
        altered["park"]["next_guest_id"] = json!(1);
        assert!(decode(&altered).is_err());
        let mut altered = baseline.clone();
        altered["park"]["guests"][1]["id"] = altered["park"]["guests"][0]["id"].clone();
        assert!(decode(&altered).is_err());
        let mut altered = baseline;
        altered["park"]["crowd_seed"] = Value::Null;
        assert!(decode(&altered).is_err());
    }

    #[test]
    fn train_position_phase_operation_and_circuit_references_are_validated() {
        let mut park = fixture();
        advance(&mut park, 200);
        let baseline = value(&park);
        for (path, replacement) in [
            (
                "/park/train/distance",
                json!(park.train().unwrap().distance + 1),
            ),
            (
                "/park/train/lap_progress",
                json!(park.train().unwrap().lap_progress + 1),
            ),
            ("/park/train/piece_index", json!(usize::MAX)),
            ("/park/train/phase", json!("Closed")),
            ("/park/train/phase", json!("Fault")),
            ("/park/train/speed", json!(0)),
            ("/park/train/speed", json!(97)),
            ("/park/train/dwell_ticks", json!(1)),
            ("/park/train/departed", json!(false)),
            ("/park/circuit", Value::Null),
            ("/park/train", Value::Null),
            ("/park/circuit/pieces/0/object", json!(u64::MAX)),
            (
                "/park/circuit/pieces/1/object",
                baseline["park"]["circuit"]["pieces"][0]["object"].clone(),
            ),
            ("/park/circuit/pieces/1/spec/exit/height", json!(15)),
            ("/park/circuit/pieces/45/spec/exit/edge", json!("West")),
        ] {
            let mut altered = baseline.clone();
            *altered.pointer_mut(path).unwrap() = replacement;
            assert!(decode(&altered).is_err(), "accepted {path}");
        }
        let mut altered = value(&fixture());
        altered["park"]["train"]["speed"] = json!(20);
        assert!(
            decode(&altered).is_err(),
            "dwelling train cannot have speed"
        );
        let mut altered = value(&fixture());
        altered["park"]["circuit"]["pieces"]
            .as_array_mut()
            .unwrap()
            .clear();
        assert!(decode(&altered).is_err());
    }

    #[test]
    fn byte_collection_and_route_work_limits_reject_without_panicking() {
        let too_large = vec![b' '; MAX_SAVE_BYTES + 1];
        assert_eq!(
            Park::from_save_bytes(&too_large),
            Err(SaveError::TooLarge {
                actual: MAX_SAVE_BYTES + 1,
                max: MAX_SAVE_BYTES
            })
        );
        let baseline = value(&fixture());
        for (path, count) in [
            ("/park/objects", MAX_OBJECTS + 1),
            ("/park/guests", MAX_GUESTS + 1),
            ("/park/tiles", MAX_TILES + 1),
            ("/park/circuit/pieces", MAX_CIRCUIT_PIECES + 1),
        ] {
            let mut altered = baseline.clone();
            let sample = altered.pointer(path).unwrap()[0].clone();
            *altered.pointer_mut(path).unwrap() = Value::Array(vec![sample; count]);
            assert!(
                matches!(decode(&altered), Err(SaveError::Json(message)) if message.contains("limit"))
            );
        }
        let mut altered = baseline.clone();
        altered["park"]["guests"][0]["route"] = json!(vec![1; MAX_PATHS + 1]);
        assert!(
            matches!(decode(&altered), Err(SaveError::Json(message)) if message.contains("limit"))
        );
        let mut altered = baseline.clone();
        let mut guest = altered["park"]["guests"][0].clone();
        guest["route"] = json!(vec![1; 257]);
        altered["park"]["guests"] = Value::Array(vec![guest; MAX_GUESTS]);
        assert!(
            matches!(decode(&altered), Err(SaveError::InvalidState(message)) if message.contains("route work"))
        );
        let mut altered = baseline;
        let path = altered["park"]["objects"][0].clone();
        altered["park"]["objects"] = Value::Array(vec![path; MAX_PATHS + 1]);
        assert!(
            matches!(decode(&altered), Err(SaveError::InvalidState(message)) if message.contains("limit"))
        );
    }

    #[test]
    fn rejected_loads_cannot_mutate_the_live_park_and_writers_reject_invalid_state() {
        let park = fixture();
        let before = park.clone();
        let mut broken = value(&park);
        broken["park"]["guests"][0]["current"] = json!(u64::MAX);
        assert!(decode(&broken).is_err());
        assert_eq!(park, before);
        let mut invalid_park = park.clone();
        invalid_park.guests[0].wait_ticks = 99;
        assert!(invalid_park.to_save_bytes().is_err());
        let mut invalid_park = park;
        invalid_park.next_object_id = 1;
        assert!(invalid_park.to_save_bytes().is_err());
    }
}
