//! A deterministic closed-circuit train proof, independent of the renderer.
//!
//! Motion is deliberately approximate: a chain lift fixes speed, downhill track
//! adds speed, rolling resistance removes speed, and brakes slow the train.
//! Positions and arc distances are integer fixed point. Height contributes one
//! quarter as much as tile distance to arc length, matching the abstract small
//! elevation steps. This is not a general rigid-body or coaster physics engine.

use crate::{Axis, ObjectId, ObjectKind, Park, PlacementError, TilePos};

/// Fixed-point units per ground tile and per abstract height step.
pub const TRACK_UNITS: i32 = 1024;
pub const STATION_DWELL_TICKS: u16 = 90;
pub(crate) const MAX_SPEED: u32 = 96;
const LIFT_SPEED: u32 = 24;
const BRAKE_SPEED: u32 = 20;
const CAR_SPACING: u32 = 320;
const CAR_HALF_LENGTH: u32 = 120;
const SAMPLES_PER_PIECE: i64 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TrackKind {
    Station,
    Lift,
    Drop,
    Brake,
    Curve,
    Straight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TrackDirection {
    North,
    East,
    South,
    West,
}

impl TrackDirection {
    fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    fn offset(self) -> (i32, i32) {
        let half = TRACK_UNITS / 2;
        match self {
            Self::North => (0, -half),
            Self::East => (half, 0),
            Self::South => (0, half),
            Self::West => (-half, 0),
        }
    }
}

/// A tile-edge endpoint. Entry and exit heights are whole simulation steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackPort {
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub edge: TrackDirection,
    pub height: u8,
}

/// Construction input for a one-tile piece. Circuit installation is atomic.
/// Curves are level quarter turns; lifts rise one step; drops descend one or two.
/// Other pieces are level straights. Exactly one station is required.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackPieceSpec {
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub pos: TilePos,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub kind: TrackKind,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub entry: TrackPort,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub exit: TrackPort,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TrackPoint {
    pub x: i32,
    pub y: i32,
    pub height: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackPiece {
    /// The same occupied-space object used by selection and clearance.
    pub object: ObjectId,
    pub kind: TrackKind,
    /// Ordered centerline samples, including both exact tile-edge endpoints.
    pub samples: Vec<TrackPoint>,
    /// Arc length in fixed distance units; vertical steps count as quarter tiles.
    pub length: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrainCircuit {
    pub pieces: Vec<TrackPiece>,
    pub total_length: u32,
    pub(crate) specs: Vec<TrackPieceSpec>,
    pub(crate) station_stop: u32,
}

impl TrainCircuit {
    /// Sample the authoritative polyline at a wrapping arc distance.
    pub fn sample(&self, distance: u32) -> TrackPoint {
        let (index, local) = self.locate(distance);
        let piece = &self.pieces[index];
        let mut remaining = local;
        for pair in piece.samples.windows(2) {
            let length = point_distance(pair[0], pair[1]);
            if remaining < length {
                return interpolate(pair[0], pair[1], remaining, length);
            }
            remaining -= length;
        }
        // Valid circuits have positive lengths and a final sample. Integer
        // rounding at an endpoint resolves to that endpoint deterministically.
        piece.samples[piece.samples.len() - 1]
    }

    pub(crate) fn locate(&self, distance: u32) -> (usize, u32) {
        let mut remaining = distance % self.total_length;
        for (index, piece) in self.pieces.iter().enumerate() {
            if remaining < piece.length {
                return (index, remaining);
            }
            remaining -= piece.length;
        }
        unreachable!("validated circuit lengths sum to total_length")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TrainPhase {
    Station,
    Lift,
    Drop,
    Brake,
    Cruising,
    Closed,
    Fault,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Train {
    /// A close request finishes the current lap; it does not pause the park.
    pub open: bool,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub phase: TrainPhase,
    /// Fixed distance units per simulation tick (30 ticks/second).
    pub speed: u32,
    /// Lead-car center measured along the circuit's sampled polyline.
    pub distance: u32,
    pub laps: u64,
    pub dwell_ticks: u16,
    pub piece_index: usize,
    pub(crate) lap_progress: u32,
    pub(crate) departed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrainPose {
    pub center: TrackPoint,
    pub front: TrackPoint,
    pub rear: TrackPoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrainError {
    Placement(PlacementError),
    EmptyCircuit,
    StationCount { count: usize },
    InvalidPiece { index: usize },
    Disconnected { from: usize, to: usize },
    DuplicateObject { object: ObjectId },
    InvalidLength,
    MissingObject { object: ObjectId },
    ObjectMismatch { object: ObjectId },
    CircuitAlreadyInstalled,
}

impl std::fmt::Display for TrainError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Placement(error) => error.fmt(formatter),
            Self::EmptyCircuit => write!(formatter, "a train circuit must contain track"),
            Self::StationCount { count } => {
                write!(
                    formatter,
                    "a train circuit needs one station, found {count}"
                )
            }
            Self::InvalidPiece { index } => {
                write!(formatter, "track piece {index} has invalid ports or grade")
            }
            Self::Disconnected { from, to } => {
                write!(
                    formatter,
                    "track piece {from} does not connect to piece {to}"
                )
            }
            Self::DuplicateObject { object } => {
                write!(formatter, "track object {} is used twice", object.0)
            }
            Self::InvalidLength => write!(formatter, "train circuit has an invalid length"),
            Self::MissingObject { object } => {
                write!(formatter, "train circuit object {} is missing", object.0)
            }
            Self::ObjectMismatch { object } => {
                write!(
                    formatter,
                    "train circuit object {} has a different envelope",
                    object.0
                )
            }
            Self::CircuitAlreadyInstalled => {
                write!(formatter, "a different train circuit is already installed")
            }
        }
    }
}

impl std::error::Error for TrainError {}

impl From<PlacementError> for TrainError {
    fn from(value: PlacementError) -> Self {
        Self::Placement(value)
    }
}

impl Park {
    pub fn circuit(&self) -> Option<&TrainCircuit> {
        self.circuit.as_ref()
    }

    pub fn train(&self) -> Option<&Train> {
        self.train.as_ref()
    }

    /// Three compact cars. All three fit inside the one-tile station at rest:
    /// their outermost ends are 8 and 888 units along its 1024-unit centerline.
    /// Faulted or absent circuits produce no poses.
    pub fn train_cars(&self) -> Vec<TrainPose> {
        let (Some(circuit), Some(train)) = (&self.circuit, &self.train) else {
            return Vec::new();
        };
        if train.phase == TrainPhase::Fault {
            return Vec::new();
        }
        (0..3)
            .map(|index| {
                let center = wrap_sub(train.distance, index * CAR_SPACING, circuit.total_length);
                TrainPose {
                    center: circuit.sample(center),
                    front: circuit.sample((center + CAR_HALF_LENGTH) % circuit.total_length),
                    rear: circuit.sample(wrap_sub(center, CAR_HALF_LENGTH, circuit.total_length)),
                }
            })
            .collect()
    }

    /// Closing while dwelling parks immediately. Closing in motion completes
    /// this lap. Reopening a parked train begins a fresh full dwell. Commands
    /// work while globally paused, but no distance or dwell time elapses.
    pub fn set_train_open(&mut self, open: bool) {
        let Some(train) = &mut self.train else {
            return;
        };
        if train.phase == TrainPhase::Fault || train.open == open {
            return;
        }
        train.open = open;
        if !train.departed {
            train.speed = 0;
            train.dwell_ticks = if open { STATION_DWELL_TICKS } else { 0 };
            train.phase = if open {
                TrainPhase::Station
            } else {
                TrainPhase::Closed
            };
        }
        self.reconcile_ride_visits();
    }

    /// Install the fixed 46-piece PRF-03 loop above either terrain preset.
    /// Repeating a healthy install is a no-op. After a missing track faults the
    /// ride, installation restores missing pieces with fresh IDs and parks the
    /// train closed, preserving its lap count and all unrelated park state.
    pub fn install_train_fixture(&mut self) -> Result<(), TrainError> {
        if (self.width(), self.height()) != (32, 32) {
            return Err(PlacementError::FixtureMapSize {
                width: self.width(),
                height: self.height(),
            }
            .into());
        }
        self.install_train_circuit(&fixture_specs())
    }

    /// Install a validated closed circuit, committing objects and train only
    /// after all ports, grades, lengths, terrain and occupied spaces succeed.
    /// This proof supports one circuit. Replacing a live circuit is not editing.
    pub fn install_train_circuit(&mut self, specs: &[TrackPieceSpec]) -> Result<(), TrainError> {
        let geometry = validate_specs(self, specs)?;
        if let Some(circuit) = &self.circuit {
            if circuit.specs == specs {
                return self.validate_train_circuit();
            }
            return Err(TrainError::CircuitAlreadyInstalled);
        }
        let mut candidate = self.clone();
        let mut pieces: Vec<TrackPiece> = Vec::with_capacity(specs.len());
        let mut total_length = 0_u32;
        let mut station_stop = 0;
        for (spec, samples) in specs.iter().zip(geometry) {
            let (axis, height, clearance) = envelope(*spec);
            let matching = candidate.objects.iter().find(|object| {
                object.pos == spec.pos
                    && object.kind == ObjectKind::Track
                    && object.axis == axis
                    && object.height == height
                    && object.clearance == clearance
            });
            let object = match matching {
                Some(object) => object.id,
                None => {
                    candidate.place_object(spec.pos, ObjectKind::Track, axis, height, clearance)?
                }
            };
            if pieces.iter().any(|piece| piece.object == object) {
                return Err(TrainError::DuplicateObject { object });
            }
            let length = samples
                .windows(2)
                .map(|pair| point_distance(pair[0], pair[1]))
                .sum();
            if length <= MAX_SPEED {
                return Err(TrainError::InvalidLength);
            }
            if spec.kind == TrackKind::Station {
                station_stop = total_length + 3 * TRACK_UNITS as u32 / 4;
            }
            total_length = total_length
                .checked_add(length)
                .ok_or(TrainError::InvalidLength)?;
            pieces.push(TrackPiece {
                object,
                kind: spec.kind,
                samples,
                length,
            });
        }
        if total_length < 4 * TRACK_UNITS as u32 {
            return Err(TrainError::InvalidLength);
        }
        let circuit = TrainCircuit {
            pieces,
            total_length,
            specs: specs.to_vec(),
            station_stop,
        };
        let restoring = candidate.train.is_some();
        if candidate.ride.is_none() {
            let next = candidate
                .next_ride_id
                .checked_add(1)
                .ok_or(PlacementError::IdExhausted)?;
            candidate.ride = Some(crate::Ride::new(crate::RideId(candidate.next_ride_id)));
            candidate.next_ride_id = next;
        }
        let laps = candidate.train.as_ref().map_or(0, |train| train.laps);
        candidate.train = Some(Train {
            open: !restoring,
            phase: if restoring {
                TrainPhase::Closed
            } else {
                TrainPhase::Station
            },
            speed: 0,
            distance: station_stop,
            laps,
            dwell_ticks: if restoring { 0 } else { STATION_DWELL_TICKS },
            piece_index: circuit.locate(station_stop).0,
            lap_progress: 0,
            departed: false,
        });
        candidate.circuit = Some(circuit);
        candidate.validate_train_circuit()?;
        *self = candidate;
        Ok(())
    }

    /// Check that every sampled piece still has its exact occupied envelope.
    pub fn validate_train_circuit(&self) -> Result<(), TrainError> {
        let Some(circuit) = &self.circuit else {
            return Err(TrainError::EmptyCircuit);
        };
        for (piece, spec) in circuit.pieces.iter().zip(&circuit.specs) {
            let object = self.object(piece.object).ok_or(TrainError::MissingObject {
                object: piece.object,
            })?;
            let (axis, height, clearance) = envelope(*spec);
            if object.kind != ObjectKind::Track
                || object.pos != spec.pos
                || object.axis != axis
                || object.height != height
                || object.clearance != clearance
            {
                return Err(TrainError::ObjectMismatch {
                    object: piece.object,
                });
            }
        }
        Ok(())
    }

    pub(crate) fn reconcile_train_after_removal(&mut self, removed: ObjectId) {
        if self
            .circuit
            .as_ref()
            .is_some_and(|circuit| circuit.pieces.iter().any(|piece| piece.object == removed))
        {
            // Discard the complete reference set, not just the removed sample.
            self.circuit = None;
            if let Some(train) = &mut self.train {
                train.open = false;
                train.phase = TrainPhase::Fault;
                train.speed = 0;
                train.distance = 0;
                train.piece_index = 0;
                train.dwell_ticks = 0;
                train.departed = false;
                train.lap_progress = 0;
            }
        }
    }

    pub(crate) fn step_train(&mut self) {
        let (Some(circuit), Some(train)) = (&self.circuit, &mut self.train) else {
            return;
        };
        if !train.open && !train.departed || train.phase == TrainPhase::Fault {
            return;
        }
        if !train.departed {
            if train.dwell_ticks > 0 {
                train.dwell_ticks -= 1;
                return;
            }
            train.departed = true;
            train.speed = BRAKE_SPEED;
        }
        let kind = circuit.pieces[train.piece_index].kind;
        train.speed = match kind {
            TrackKind::Lift => LIFT_SPEED,
            TrackKind::Drop => (train.speed + 3).min(MAX_SPEED),
            TrackKind::Brake | TrackKind::Station => train.speed.saturating_sub(5).max(BRAKE_SPEED),
            TrackKind::Curve | TrackKind::Straight => {
                if train.speed > 32 {
                    train.speed - 1
                } else {
                    train.speed.saturating_add(1).min(32)
                }
            }
        };
        // Carry every unit across piece boundaries. Every validated piece is
        // longer than MAX_SPEED, so a tick cannot jump over a lift/brake phase.
        // The sole station stop consumes overshoot exactly once per full lap.
        let remaining = circuit.total_length - train.lap_progress;
        if train.speed >= remaining {
            train.laps = train.laps.saturating_add(1);
            train.distance = circuit.station_stop;
            train.piece_index = circuit.locate(train.distance).0;
            train.speed = 0;
            train.lap_progress = 0;
            train.departed = false;
            train.dwell_ticks = if train.open { STATION_DWELL_TICKS } else { 0 };
            train.phase = if train.open {
                TrainPhase::Station
            } else {
                TrainPhase::Closed
            };
            return;
        }
        train.lap_progress += train.speed;
        train.distance = (circuit.station_stop + train.lap_progress) % circuit.total_length;
        train.piece_index = circuit.locate(train.distance).0;
        train.phase = match circuit.pieces[train.piece_index].kind {
            TrackKind::Station => TrainPhase::Station,
            TrackKind::Lift => TrainPhase::Lift,
            TrackKind::Drop => TrainPhase::Drop,
            TrackKind::Brake => TrainPhase::Brake,
            TrackKind::Curve | TrackKind::Straight => TrainPhase::Cruising,
        };
    }
}

pub(crate) fn validate_specs(
    park: &Park,
    specs: &[TrackPieceSpec],
) -> Result<Vec<Vec<TrackPoint>>, TrainError> {
    if specs.is_empty() {
        return Err(TrainError::EmptyCircuit);
    }
    let stations = specs
        .iter()
        .filter(|piece| piece.kind == TrackKind::Station)
        .count();
    if stations != 1 {
        return Err(TrainError::StationCount { count: stations });
    }
    let mut geometry = Vec::with_capacity(specs.len());
    for (index, spec) in specs.iter().enumerate() {
        if park.tile(spec.pos).is_none() {
            return Err(PlacementError::OutOfBounds { pos: spec.pos }.into());
        }
        let straight = spec.entry.edge.opposite() == spec.exit.edge;
        let grade = i16::from(spec.exit.height) - i16::from(spec.entry.height);
        let valid = match spec.kind {
            TrackKind::Curve => !straight && spec.entry.edge != spec.exit.edge && grade == 0,
            TrackKind::Lift => straight && grade == 1,
            TrackKind::Drop => straight && (-2..=-1).contains(&grade),
            TrackKind::Station | TrackKind::Brake | TrackKind::Straight => straight && grade == 0,
        };
        if !valid {
            return Err(TrainError::InvalidPiece { index });
        }
        geometry.push(piece_samples(*spec));
    }
    for index in 0..geometry.len() {
        let next = (index + 1) % geometry.len();
        if geometry[index].last() != geometry[next].first()
            || specs[index].exit.edge.opposite() != specs[next].entry.edge
        {
            return Err(TrainError::Disconnected {
                from: index,
                to: next,
            });
        }
    }
    Ok(geometry)
}

pub(crate) fn envelope(spec: TrackPieceSpec) -> (Axis, u8, u8) {
    let height = spec.entry.height.min(spec.exit.height);
    let clearance = spec.entry.height.abs_diff(spec.exit.height) + 2;
    let axis = match spec.entry.edge {
        TrackDirection::East | TrackDirection::West => Axis::X,
        TrackDirection::North | TrackDirection::South => Axis::Y,
    };
    (axis, height, clearance)
}

fn piece_samples(spec: TrackPieceSpec) -> Vec<TrackPoint> {
    let center = TrackPoint {
        x: spec.pos.x as i32 * TRACK_UNITS,
        y: spec.pos.y as i32 * TRACK_UNITS,
        height: i32::from(spec.entry.height) * TRACK_UNITS,
    };
    let endpoint = |port: TrackPort| {
        let (dx, dy) = port.edge.offset();
        TrackPoint {
            x: center.x + dx,
            y: center.y + dy,
            height: i32::from(port.height) * TRACK_UNITS,
        }
    };
    let from = endpoint(spec.entry);
    let to = endpoint(spec.exit);
    (0..=SAMPLES_PER_PIECE)
        .map(|step| {
            if spec.kind == TrackKind::Curve {
                let before = SAMPLES_PER_PIECE - step;
                let curve = |a: i32, middle: i32, b: i32| {
                    ((i64::from(a) * before * before
                        + 2 * i64::from(middle) * before * step
                        + i64::from(b) * step * step)
                        / (SAMPLES_PER_PIECE * SAMPLES_PER_PIECE)) as i32
                };
                TrackPoint {
                    x: curve(from.x, center.x, to.x),
                    y: curve(from.y, center.y, to.y),
                    height: from.height,
                }
            } else {
                interpolate(from, to, step as u32, SAMPLES_PER_PIECE as u32)
            }
        })
        .collect()
}

pub(crate) fn point_distance(from: TrackPoint, to: TrackPoint) -> u32 {
    let dx = i64::from(to.x) - i64::from(from.x);
    let dy = i64::from(to.y) - i64::from(from.y);
    let dz = (i64::from(to.height) - i64::from(from.height)) / 4;
    (dx * dx + dy * dy + dz * dz).isqrt() as u32
}

fn interpolate(from: TrackPoint, to: TrackPoint, distance: u32, length: u32) -> TrackPoint {
    let mix = |a: i32, b: i32| {
        (i64::from(a) + (i64::from(b) - i64::from(a)) * i64::from(distance) / i64::from(length))
            as i32
    };
    TrackPoint {
        x: mix(from.x, to.x),
        y: mix(from.y, to.y),
        height: mix(from.height, to.height),
    }
}

fn wrap_sub(distance: u32, amount: u32, length: u32) -> u32 {
    (distance + length - amount % length) % length
}

fn fixture_specs() -> Vec<TrackPieceSpec> {
    use TrackDirection::{East, North, South, West};
    let mut specs = Vec::new();
    let mut add = |x, y, kind, entry, exit, from, to| {
        specs.push(TrackPieceSpec {
            pos: TilePos { x, y },
            kind,
            entry: TrackPort {
                edge: entry,
                height: from,
            },
            exit: TrackPort {
                edge: exit,
                height: to,
            },
        });
    };
    add(8, 10, TrackKind::Station, West, East, 8, 8);
    for x in 9..=16 {
        add(x, 10, TrackKind::Lift, West, East, (x - 1) as u8, x as u8);
    }
    for x in 17..=23 {
        add(x, 10, TrackKind::Straight, West, East, 16, 16);
    }
    add(24, 10, TrackKind::Curve, West, North, 16, 16);
    for y in (5..=9).rev() {
        add(24, y, TrackKind::Straight, South, North, 16, 16);
    }
    add(24, 4, TrackKind::Curve, South, West, 16, 16);
    for x in (20..=23).rev() {
        let from = 16 - (23 - x) * 2;
        add(
            x,
            4,
            TrackKind::Drop,
            East,
            West,
            from as u8,
            (from - 2) as u8,
        );
    }
    for x in (8..=19).rev() {
        add(x, 4, TrackKind::Straight, East, West, 8, 8);
    }
    add(7, 4, TrackKind::Curve, East, South, 8, 8);
    for y in 5..=9 {
        let kind = if y >= 8 {
            TrackKind::Brake
        } else {
            TrackKind::Straight
        };
        add(7, y, kind, North, South, 8, 8);
    }
    add(7, 10, TrackKind::Curve, North, East, 8, 8);
    specs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TerrainHeightError, TerrainProfile};

    fn fixture() -> Park {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        park
    }

    fn advance(park: &mut Park, ticks: usize) {
        for _ in 0..ticks {
            park.advance(Speed::Normal);
        }
    }

    fn finish_lap(park: &mut Park) {
        let before = park.train().unwrap().laps;
        for _ in 0..3_600 {
            park.advance(Speed::Normal);
            if park.train().unwrap().laps > before {
                return;
            }
        }
        panic!("the fixture must complete a lap within 120 simulation seconds");
    }

    #[test]
    fn fixture_is_closed_shared_geometry_and_all_cars_fit_station() {
        let park = fixture();
        let circuit = park.circuit().unwrap();
        assert_eq!(circuit.pieces.len(), 46);
        assert_eq!(
            circuit.total_length,
            circuit.pieces.iter().map(|piece| piece.length).sum::<u32>()
        );
        assert_eq!(park.validate_train_circuit(), Ok(()));
        assert_eq!(park.max_surface_height(), 16);
        for (index, piece) in circuit.pieces.iter().enumerate() {
            let next = &circuit.pieces[(index + 1) % circuit.pieces.len()];
            assert_eq!(piece.samples.last(), next.samples.first());
            let object = park.object(piece.object).unwrap();
            assert_eq!(object.kind, ObjectKind::Track);
            assert!(piece.length > MAX_SPEED);
            for point in &piece.samples {
                assert!((point.x - object.pos.x as i32 * TRACK_UNITS).abs() <= TRACK_UNITS / 2);
                assert!((point.y - object.pos.y as i32 * TRACK_UNITS).abs() <= TRACK_UNITS / 2);
                assert!(point.height >= i32::from(object.height) * TRACK_UNITS);
                assert!(
                    point.height <= i32::from(object.height + object.clearance - 2) * TRACK_UNITS
                );
            }
        }
        let station_start = 8 * TRACK_UNITS - TRACK_UNITS / 2;
        let station_end = station_start + TRACK_UNITS;
        let cars = park.train_cars();
        assert_eq!(cars.len(), 3);
        for car in &cars {
            for point in [car.rear, car.center, car.front] {
                assert!((station_start..=station_end).contains(&point.x));
                assert_eq!(point.y, 10 * TRACK_UNITS);
                assert_eq!(point.height, 8 * TRACK_UNITS);
            }
        }
        assert_eq!(cars[0].center.x - cars[1].center.x, CAR_SPACING as i32);
        assert_eq!(cars[1].center.x - cars[2].center.x, CAR_SPACING as i32);
        assert_eq!(circuit.sample(0), circuit.sample(circuit.total_length));
    }

    #[test]
    fn motion_visits_lift_drop_brake_and_returns_to_dwell_with_one_lap() {
        let mut park = fixture();
        let initial = park.train().unwrap().clone();
        advance(&mut park, STATION_DWELL_TICKS as usize);
        assert_eq!(park.train().unwrap().distance, initial.distance);
        assert_eq!(park.train().unwrap().speed, 0);
        assert_eq!(park.train().unwrap().dwell_ticks, 0);
        let mut sequence = Vec::new();
        let mut last_kind = None;
        let mut maximum_drop_speed = 0;
        let mut minimum_brake_speed = MAX_SPEED;
        let mut lift_speed_seen = false;
        for _ in 0..3_600 {
            park.advance(Speed::Normal);
            let train = park.train().unwrap();
            assert!(train.speed <= MAX_SPEED);
            if train.laps > 0 {
                break;
            }
            let kind = park.circuit().unwrap().pieces[train.piece_index].kind;
            if matches!(kind, TrackKind::Lift | TrackKind::Drop | TrackKind::Brake)
                && Some(kind) != last_kind
            {
                sequence.push(kind);
                last_kind = Some(kind);
            }
            if kind == TrackKind::Lift && train.speed == LIFT_SPEED {
                lift_speed_seen = true;
            }
            if kind == TrackKind::Drop {
                maximum_drop_speed = maximum_drop_speed.max(train.speed);
            }
            if kind == TrackKind::Brake {
                minimum_brake_speed = minimum_brake_speed.min(train.speed);
            }
        }
        assert_eq!(
            sequence,
            [TrackKind::Lift, TrackKind::Drop, TrackKind::Brake]
        );
        assert!(lift_speed_seen);
        assert_eq!(maximum_drop_speed, MAX_SPEED);
        assert_eq!(minimum_brake_speed, BRAKE_SPEED);
        let train = park.train().unwrap();
        assert_eq!(train.laps, 1);
        assert_eq!(train.phase, TrainPhase::Station);
        assert_eq!(train.dwell_ticks, STATION_DWELL_TICKS);
        assert_eq!(train.distance, initial.distance);
        assert_eq!(train.speed, 0);
        finish_lap(&mut park);
        assert_eq!(park.train().unwrap().laps, 2);
    }

    #[test]
    fn distance_carries_across_piece_boundaries_without_losing_overshoot() {
        let mut park = fixture();
        advance(&mut park, 91);
        let mut crossings = 0;
        for _ in 0..3_600 {
            let before = park.train().unwrap().clone();
            let length = park.circuit().unwrap().total_length;
            park.advance(Speed::Normal);
            let after = park.train().unwrap();
            if after.laps > before.laps {
                assert_eq!(after.laps, before.laps + 1);
                assert_eq!(after.lap_progress, 0);
                break;
            }
            assert_eq!(after.distance, (before.distance + after.speed) % length);
            if after.piece_index != before.piece_index {
                crossings += 1;
                assert_eq!(after.piece_index, (before.piece_index + 1) % 46);
            }
        }
        assert_eq!(crossings, 46);
    }

    #[test]
    fn close_finishes_current_lap_and_reopen_starts_full_dwell() {
        let mut park = fixture();
        advance(&mut park, 250);
        let before = park.train().unwrap().clone();
        assert!(before.departed);
        park.set_train_open(false);
        let requested = park.train().unwrap();
        assert!(!requested.open);
        assert_eq!(requested.distance, before.distance);
        assert_eq!(requested.speed, before.speed);
        finish_lap(&mut park);
        assert_eq!(park.train().unwrap().phase, TrainPhase::Closed);
        assert_eq!(park.train().unwrap().laps, 1);
        let parked = park.train().unwrap().clone();
        advance(&mut park, 500);
        assert_eq!(park.train().unwrap(), &parked);
        park.set_train_open(true);
        assert_eq!(park.train().unwrap().dwell_ticks, STATION_DWELL_TICKS);
        advance(&mut park, STATION_DWELL_TICKS as usize);
        assert_eq!(park.train().unwrap().distance, parked.distance);
        park.advance(Speed::Normal);
        assert_ne!(park.train().unwrap().distance, parked.distance);
    }

    #[test]
    fn station_close_is_immediate_and_pause_freezes_dwell_or_motion() {
        let mut park = fixture();
        advance(&mut park, 12);
        park.set_train_open(false);
        let closed = park.clone();
        assert_eq!(park.train().unwrap().phase, TrainPhase::Closed);
        for _ in 0..100 {
            park.advance(Speed::Paused);
        }
        assert_eq!(park, closed);
        park.set_train_open(true);
        let opened = park.clone();
        for _ in 0..100 {
            park.advance(Speed::Paused);
        }
        assert_eq!(park, opened);
        advance(&mut park, 150);
        park.set_train_open(false);
        let moving_closed = park.clone();
        park.advance(Speed::Paused);
        assert_eq!(park, moving_closed);
        park.set_train_open(true);
        assert_eq!(
            park.train().unwrap().distance,
            moving_closed.train().unwrap().distance
        );
        assert_eq!(park.train().unwrap().dwell_ticks, 0);
        assert!(park.train().unwrap().departed);
    }

    #[test]
    fn complete_ticks_preserve_train_crowd_replay_and_speed_equivalence() {
        let mut normal = Park::new(32, 32);
        normal.install_crowd_fixture(57).unwrap();
        normal.install_train_fixture().unwrap();
        let mut fast = normal.clone();
        let mut fastest = normal.clone();
        let mut chunked = normal.clone();
        advance(&mut normal, 6_000);
        for _ in 0..3_000 {
            fast.advance(Speed::Fast);
        }
        for _ in 0..1_500 {
            fastest.advance(Speed::Fastest);
        }
        for _ in 0..600 {
            chunked.advance(Speed::Paused);
            chunked.advance(Speed::Normal);
            chunked.advance(Speed::Fast);
            chunked.advance(Speed::Fastest);
            chunked.advance(Speed::Normal);
            chunked.advance(Speed::Fast);
        }
        assert_eq!(normal, fast);
        assert_eq!(normal, fastest);
        assert_eq!(normal, chunked);
        assert!(normal.train().unwrap().laps >= 2);
    }

    #[test]
    fn installation_and_profiles_preserve_crowd_clock_rng_and_existing_ids() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(93).unwrap();
        advance(&mut park, 500);
        let before = park.clone();
        park.install_train_fixture().unwrap();
        assert_eq!(park.ticks, before.ticks);
        assert_eq!(park.guests, before.guests);
        assert_eq!(park.crowd_rng, before.crowd_rng);
        assert_eq!(park.next_guest_id, before.next_guest_id);
        for object in before.objects() {
            assert_eq!(park.object(object.id), Some(object));
        }
        advance(&mut park, 500);
        let installed = park.clone();
        park.install_train_fixture().unwrap();
        assert_eq!(park, installed);
        park.try_apply_terrain_profile(TerrainProfile::Terraced)
            .unwrap();
        assert_eq!(park.objects, installed.objects);
        assert_eq!(park.circuit, installed.circuit);
        assert_eq!(park.train, installed.train);
        park.try_apply_terrain_profile(TerrainProfile::Flat)
            .unwrap();
        assert_eq!(park, installed);
    }

    #[test]
    fn failed_installations_are_atomic_even_after_partial_object_allocation() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(7).unwrap();
        advance(&mut park, 34);
        // The final fixture corner collides, after 45 candidate placements.
        park.place_object(TilePos { x: 7, y: 10 }, ObjectKind::Path, Axis::X, 8, 2)
            .unwrap();
        let before = park.clone();
        assert!(matches!(
            park.install_train_fixture(),
            Err(TrainError::Placement(PlacementError::Occupied { .. }))
        ));
        assert_eq!(park, before);
        let mut small = Park::new(16, 16);
        let before = small.clone();
        assert!(matches!(
            small.install_train_fixture(),
            Err(TrainError::Placement(PlacementError::FixtureMapSize { .. }))
        ));
        assert_eq!(small, before);
        let mut exhausted = Park::new(32, 32);
        exhausted.next_object_id = u64::MAX - 1;
        let before = exhausted.clone();
        assert_eq!(
            exhausted.install_train_fixture(),
            Err(TrainError::Placement(PlacementError::IdExhausted))
        );
        assert_eq!(exhausted, before);
    }

    #[test]
    fn invalid_ports_grades_last_endpoint_and_station_count_reject_atomically() {
        let mut park = Park::new(32, 32);
        let before = park.clone();
        assert_eq!(
            park.install_train_circuit(&[]),
            Err(TrainError::EmptyCircuit)
        );
        for (index, mutate) in [
            (1, 0),  // A zero-length same-edge piece.
            (1, 1),  // An uphill grade steeper than the lift contract.
            (45, 2), // Valid piece shape, but last exit fails to meet first.
            (1, 3),  // A disconnected height endpoint in the middle.
        ] {
            let mut specs = fixture_specs();
            match mutate {
                0 => specs[index].exit.edge = specs[index].entry.edge,
                1 => specs[index].exit.height += 2,
                2 => {
                    specs[index].entry.height += 1;
                    specs[index].exit.height += 1;
                }
                _ => {
                    specs[index].entry.height += 1;
                    specs[index].exit.height += 1;
                }
            }
            assert!(park.install_train_circuit(&specs).is_err());
            assert_eq!(park, before);
        }
        let mut specs = fixture_specs();
        specs[0].kind = TrackKind::Straight;
        assert_eq!(
            park.install_train_circuit(&specs),
            Err(TrainError::StationCount { count: 0 })
        );
        specs[0].kind = TrackKind::Station;
        specs[9].kind = TrackKind::Station;
        assert_eq!(
            park.install_train_circuit(&specs),
            Err(TrainError::StationCount { count: 2 })
        );
        assert_eq!(park, before);
        // Keep all preceding joints valid and break only the closing joint.
        let mut specs = fixture_specs();
        specs[45].exit.edge = TrackDirection::West;
        assert_eq!(
            park.install_train_circuit(&specs),
            Err(TrainError::Disconnected { from: 45, to: 0 })
        );
        assert_eq!(park, before);
        let mut specs = fixture_specs();
        specs[0].pos.x = u32::MAX;
        assert!(matches!(
            park.install_train_circuit(&specs),
            Err(TrainError::Placement(PlacementError::OutOfBounds { .. }))
        ));
        assert_eq!(park, before);
    }

    #[test]
    fn slope_and_corner_envelopes_reject_overlaps_and_terrain_atomically() {
        let mut park = fixture();
        // Drop 16->14 occupies [14,18), including the high end and car clearance.
        let drop = TilePos { x: 23, y: 4 };
        let before = park.clone();
        assert!(matches!(
            park.place_object(drop, ObjectKind::Path, Axis::X, 17, 1),
            Err(PlacementError::Occupied { .. })
        ));
        assert_eq!(park, before);
        assert!(matches!(
            park.set_terrain_height(drop, 15),
            Err(TerrainHeightError::ObjectIntersection { .. })
        ));
        assert_eq!(park, before);
        assert!(matches!(
            park.place_object(TilePos { x: 24, y: 4 }, ObjectKind::Path, Axis::Y, 16, 2),
            Err(PlacementError::Occupied { .. })
        ));
        assert_eq!(park, before);
        // Touching the reserved upper boundary remains legal.
        park.place_object(drop, ObjectKind::Path, Axis::X, 18, 2)
            .unwrap();
    }

    #[test]
    fn removing_circuit_track_faults_without_stale_refs_and_repair_parks_closed() {
        let mut park = fixture();
        finish_lap(&mut park);
        advance(&mut park, 170);
        let id = park.circuit().unwrap().pieces[12].object;
        let before = park.clone();
        park.remove_object(id).unwrap();
        assert_eq!(park.circuit(), None);
        assert_eq!(park.train().unwrap().phase, TrainPhase::Fault);
        assert!(park.train_cars().is_empty());
        let fault = park.train().unwrap().clone();
        park.set_train_open(true);
        advance(&mut park, 50);
        assert_eq!(park.train().unwrap(), &fault);
        let repaired_at = park.ticks();
        park.install_train_fixture().unwrap();
        assert_eq!(park.ticks(), repaired_at);
        assert_eq!(park.train().unwrap().phase, TrainPhase::Closed);
        assert_eq!(park.train().unwrap().laps, before.train().unwrap().laps);
        assert_eq!(park.validate_train_circuit(), Ok(()));
        assert_eq!(park.object(id), None);
        assert!(park.circuit().unwrap().pieces[12].object.0 > id.0);
        for piece in &before.circuit().unwrap().pieces {
            if piece.object != id {
                assert_eq!(park.object(piece.object), before.object(piece.object));
            }
        }
    }

    #[test]
    fn unrelated_removal_and_absent_train_controls_have_no_effect_on_train() {
        let mut park = Park::new(32, 32);
        park.set_train_open(true);
        assert!(park.train_cars().is_empty());
        assert_eq!(park.train(), None);
        park.install_overlap_fixture().unwrap();
        park.install_train_fixture().unwrap();
        let before = park.train().unwrap().clone();
        let id = park
            .objects()
            .iter()
            .find(|object| {
                object.pos == TilePos { x: 13, y: 16 } && object.kind == ObjectKind::Track
            })
            .unwrap()
            .id;
        park.remove_object(id).unwrap();
        assert_eq!(park.train().unwrap(), &before);
        assert_eq!(park.validate_train_circuit(), Ok(()));
    }

    #[test]
    fn reference_validation_reports_missing_and_mismatched_envelopes() {
        let mut park = fixture();
        let id = park.circuit().unwrap().pieces[0].object;
        // Exercise validation independently of the public removal fault path.
        let index = park
            .objects
            .iter()
            .position(|object| object.id == id)
            .unwrap();
        park.objects[index].clearance += 1;
        assert_eq!(
            park.validate_train_circuit(),
            Err(TrainError::ObjectMismatch { object: id })
        );
        park.objects.remove(index);
        assert_eq!(
            park.validate_train_circuit(),
            Err(TrainError::MissingObject { object: id })
        );
    }
}
