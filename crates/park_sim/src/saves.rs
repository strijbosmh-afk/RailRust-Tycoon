//! Versioned simulation-only saves. JSON never deserializes directly into Park:
//! bounded DTOs are validated first, then derived circuit geometry is rebuilt.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::marker::PhantomData;

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
const SCHEMA_VERSION: u32 = 1;
const CONTENT_VERSION: u32 = 1;
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
struct SaveEnvelope {
    format: String,
    schema_version: u32,
    content_version: u32,
    #[serde(deserialize_with = "strict_object")]
    park: ParkData,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParkData {
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<SurfaceObject>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<Guest>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
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

fn required_object_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<StrictObject<T>>::deserialize(deserializer).map(|value| value.map(|object| object.0))
}

fn bounded_objects<'de, D, T, const LIMIT: usize>(deserializer: D) -> Result<Vec<T>, D::Error>
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

fn bounded_vec<'de, D, T, const LIMIT: usize>(deserializer: D) -> Result<Vec<T>, D::Error>
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
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        let envelope: SaveEnvelope = strict_object(&mut deserializer).map_err(json_error)?;
        deserializer.end().map_err(json_error)?;
        if envelope.format != FORMAT {
            return Err(invalid("unrecognized save format"));
        }
        if envelope.schema_version != SCHEMA_VERSION {
            return Err(SaveError::UnsupportedSchema(envelope.schema_version));
        }
        if envelope.content_version != CONTENT_VERSION {
            return Err(SaveError::UnsupportedContent(envelope.content_version));
        }
        envelope.park.into_park()
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
            // Derived navigation is built lazily after validated loading.
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
            || guest.progress >= WALK_TICKS
            || guest.wait_ticks > 30
        {
            return Err(invalid("guest anchor, progress or wait is out of range"));
        }
        let path = |id| {
            objects
                .get(&id)
                .copied()
                .filter(|object| object.kind == ObjectKind::Path && object.height == guest.height)
                .ok_or_else(|| invalid("guest references a missing path or a different height"))
        };
        if let Some(current) = guest.current {
            let current_path = path(current)?;
            if current_path.pos != guest.anchor {
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
                    if !visited.insert(id)
                        || previous.pos.x.abs_diff(following.pos.x)
                            + previous.pos.y.abs_diff(following.pos.y)
                            != 1
                    {
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
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        );
        assert!(Park::from_save_bytes(duplicate.as_bytes()).is_err());
        assert!(Park::from_save_bytes(format!("{text} {{}}").as_bytes()).is_err());
        let too_big_integer = text.replacen("\"ticks\":0", "\"ticks\":18446744073709551616", 1);
        assert!(Park::from_save_bytes(too_big_integer.as_bytes()).is_err());
        for (field, replacement, expected) in [
            ("schema_version", json!(2), SaveError::UnsupportedSchema(2)),
            (
                "content_version",
                json!(9),
                SaveError::UnsupportedContent(9),
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
