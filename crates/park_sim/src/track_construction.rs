//! Persistent, uncommitted level-track plans and atomic single-coaster replacement.
use std::collections::BTreeMap;

use crate::{
    ObjectKind, Park, PathSlope, PlacementError, TilePos, TrackDirection, TrackKind, TrackPiece,
    TrackPieceSpec, TrackPort, Train, TrainCircuit, TrainError, TrainPhase,
};

pub const MAX_TRACK_DRAFT_PIECES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackBuildKind {
    Station,
    Straight,
    CurveLeft,
    CurveRight,
}

impl TrackBuildKind {
    /// Direction is the incoming direction of travel, not the entry edge.
    pub fn spec(self, pos: TilePos, height: u8, direction: TrackDirection) -> TrackPieceSpec {
        use TrackDirection::{East, North, South, West};
        let exit = match self {
            Self::Station | Self::Straight => direction,
            Self::CurveLeft => match direction {
                North => West,
                West => South,
                South => East,
                East => North,
            },
            Self::CurveRight => match direction {
                North => East,
                East => South,
                South => West,
                West => North,
            },
        };
        TrackPieceSpec {
            pos,
            kind: match self {
                Self::Station => TrackKind::Station,
                Self::Straight => TrackKind::Straight,
                Self::CurveLeft | Self::CurveRight => TrackKind::Curve,
            },
            entry: TrackPort {
                edge: direction.opposite(),
                height,
            },
            exit: TrackPort { edge: exit, height },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrackConstructionError {
    Track(TrainError),
    DraftFull,
    DuplicateTile(TilePos),
    UnsupportedPiece,
    IncompleteLoop(TilePos),
    DisconnectedPieces,
    CloseRide,
    GuestsStillVisiting,
}

impl From<TrainError> for TrackConstructionError {
    fn from(error: TrainError) -> Self {
        Self::Track(error)
    }
}
impl From<PlacementError> for TrackConstructionError {
    fn from(error: PlacementError) -> Self {
        Self::Track(error.into())
    }
}
impl std::fmt::Display for TrackConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Track(error) => error.fmt(f),
            Self::DraftFull => write!(
                f,
                "a track draft supports at most {MAX_TRACK_DRAFT_PIECES} pieces"
            ),
            Self::DuplicateTile(pos) => {
                write!(f, "draft tile ({}, {}) contains two pieces", pos.x, pos.y)
            }
            Self::UnsupportedPiece => {
                f.write_str("use level stations, straights, and quarter-turn curves")
            }
            Self::IncompleteLoop(pos) => {
                write!(f, "connect the track leaving ({}, {})", pos.x, pos.y)
            }
            Self::DisconnectedPieces => {
                f.write_str("all draft pieces must belong to one directed loop")
            }
            Self::CloseRide => f.write_str("close the coaster and wait for its train to stop"),
            Self::GuestsStillVisiting => {
                f.write_str("let every coaster guest finish leaving before applying track")
            }
        }
    }
}
impl std::error::Error for TrackConstructionError {}

impl Park {
    pub fn track_draft(&self) -> &[TrackPieceSpec] {
        &self.track_draft
    }

    /// Preview does not allocate identities or alter navigation cache state.
    /// The currently owned circuit is replaceable; unrelated track is occupied.
    pub fn validate_track_draft_piece(
        &self,
        spec: TrackPieceSpec,
    ) -> Result<(), TrackConstructionError> {
        validate_piece_shape(self, spec)?;
        if !self.track_draft.iter().any(|piece| piece.pos == spec.pos)
            && self.track_draft.len() >= MAX_TRACK_DRAFT_PIECES
        {
            return Err(TrackConstructionError::DraftFull);
        }
        self.validate_draft_clearance(spec)
    }

    /// A second click on a draft tile replaces its piece. Nothing live changes.
    pub fn set_track_draft_piece(
        &mut self,
        spec: TrackPieceSpec,
    ) -> Result<(), TrackConstructionError> {
        self.validate_track_draft_piece(spec)?;
        if let Some(piece) = self
            .track_draft
            .iter_mut()
            .find(|piece| piece.pos == spec.pos)
        {
            *piece = spec;
        } else {
            self.track_draft.push(spec);
        }
        Ok(())
    }

    pub fn remove_track_draft_piece(&mut self, pos: TilePos) -> bool {
        let before = self.track_draft.len();
        self.track_draft.retain(|piece| piece.pos != pos);
        before != self.track_draft.len()
    }

    pub fn clear_track_draft(&mut self) {
        self.track_draft.clear();
    }

    /// Order independently placed pieces from the unique station, following
    /// directed tile edges. Incomplete drafts remain editable and saveable.
    pub fn validate_track_draft(&self) -> Result<Vec<TrackPieceSpec>, TrackConstructionError> {
        validate_saved_draft(self)?;
        let stations: Vec<_> = self
            .track_draft
            .iter()
            .filter(|piece| piece.kind == TrackKind::Station)
            .collect();
        if stations.len() != 1 {
            return Err(TrainError::StationCount {
                count: stations.len(),
            }
            .into());
        }
        let by_tile: BTreeMap<_, _> = self
            .track_draft
            .iter()
            .map(|piece| ((piece.pos.x, piece.pos.y), *piece))
            .collect();
        let start = *stations[0];
        let mut ordered = Vec::with_capacity(self.track_draft.len());
        let mut current = start;
        loop {
            if ordered
                .iter()
                .any(|piece: &TrackPieceSpec| piece.pos == current.pos)
            {
                return Err(TrackConstructionError::DisconnectedPieces);
            }
            ordered.push(current);
            let (dx, dy) = match current.exit.edge {
                TrackDirection::North => (0_i64, -1_i64),
                TrackDirection::East => (1, 0),
                TrackDirection::South => (0, 1),
                TrackDirection::West => (-1, 0),
            };
            let x = i64::from(current.pos.x) + dx;
            let y = i64::from(current.pos.y) + dy;
            let next = u32::try_from(x)
                .ok()
                .zip(u32::try_from(y).ok())
                .and_then(|pos| by_tile.get(&pos))
                .copied()
                .filter(|next| {
                    next.entry.edge == current.exit.edge.opposite()
                        && next.entry.height == current.exit.height
                })
                .ok_or(TrackConstructionError::IncompleteLoop(current.pos))?;
            if next.pos == start.pos {
                break;
            }
            current = next;
        }
        if ordered.len() != self.track_draft.len() {
            return Err(TrackConstructionError::DisconnectedPieces);
        }
        crate::trains::validate_specs(self, &ordered)?;
        for spec in &ordered {
            self.validate_draft_clearance(*spec)?;
        }
        // A current circuit needs no new capacity or stable identities.
        if self
            .circuit
            .as_ref()
            .is_some_and(|circuit| circuit.specs == ordered)
        {
            return Ok(ordered);
        }
        let retained = self
            .objects
            .iter()
            .filter(|object| !self.owns_circuit_object(object.id))
            .count();
        if retained + ordered.len() > crate::saves::MAX_OBJECTS {
            return Err(PlacementError::CapacityExceeded {
                resource: "objects",
                max: crate::saves::MAX_OBJECTS,
            }
            .into());
        }
        self.next_object_id
            .checked_add(ordered.len() as u64)
            .ok_or(PlacementError::IdExhausted)?;
        Ok(ordered)
    }

    /// Commit only after full validation. Preserve the coaster identity,
    /// historical statistics, finances, portals, guests, RNG and lap count.
    /// A faulted train may be replaced, but unowned remnants are never claimed.
    pub fn apply_track_draft(&mut self) -> Result<(), TrackConstructionError> {
        if self.train.as_ref().is_some_and(|train| {
            train.open
                || train.departed
                || train.speed != 0
                || !matches!(train.phase, TrainPhase::Closed | TrainPhase::Fault)
        }) {
            return Err(TrackConstructionError::CloseRide);
        }
        if self.ride.as_ref().is_some_and(|ride| {
            self.guests.iter().any(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.ride == ride.id)
            })
        }) {
            return Err(TrackConstructionError::GuestsStillVisiting);
        }
        let specs = self.validate_track_draft()?;
        // Re-applying an unchanged plan is a no-op, including stable track IDs.
        if self
            .circuit
            .as_ref()
            .is_some_and(|circuit| circuit.specs == specs)
        {
            return Ok(());
        }
        let geometry = crate::trains::validate_specs(self, &specs)?;
        let mut candidate = self.clone();
        candidate
            .objects
            .retain(|object| !self.owns_circuit_object(object.id));
        let mut pieces = Vec::with_capacity(specs.len());
        let mut total_length = 0_u32;
        let mut station_stop = 0;
        for (spec, samples) in specs.iter().zip(geometry) {
            let (axis, height, clearance) = crate::trains::envelope(*spec);
            let object = crate::ObjectId(candidate.next_object_id);
            candidate.next_object_id += 1; // Checked for the whole transaction above.
            candidate.objects.push(crate::SurfaceObject {
                id: object,
                pos: spec.pos,
                kind: ObjectKind::Track,
                axis,
                height,
                clearance,
                path_slope: PathSlope::Flat,
                queue_owner: None,
            });
            let length: u32 = samples
                .windows(2)
                .map(|pair| crate::trains::point_distance(pair[0], pair[1]))
                .sum();
            if length <= crate::trains::MAX_SPEED {
                return Err(TrainError::InvalidLength.into());
            }
            if spec.kind == TrackKind::Station {
                station_stop = total_length + 3 * crate::TRACK_UNITS as u32 / 4;
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
        if total_length < 4 * crate::TRACK_UNITS as u32 {
            return Err(TrainError::InvalidLength.into());
        }
        if candidate.ride.is_none() {
            let next = candidate
                .next_ride_id
                .checked_add(1)
                .ok_or(PlacementError::IdExhausted)?;
            candidate.ride = Some(crate::Ride::new(crate::RideId(candidate.next_ride_id)));
            candidate.next_ride_id = next;
        }
        let circuit = TrainCircuit {
            pieces,
            total_length,
            specs,
            station_stop,
        };
        candidate.train = Some(Train {
            open: false,
            phase: TrainPhase::Closed,
            speed: 0,
            distance: station_stop,
            laps: self.train.as_ref().map_or(0, |train| train.laps),
            dwell_ticks: 0,
            piece_index: circuit.locate(station_stop).0,
            lap_progress: 0,
            departed: false,
        });
        candidate.circuit = Some(circuit);
        candidate.validate_train_circuit()?;
        *self = candidate;
        Ok(())
    }

    fn owns_circuit_object(&self, id: crate::ObjectId) -> bool {
        self.circuit
            .as_ref()
            .is_some_and(|circuit| circuit.pieces.iter().any(|piece| piece.object == id))
    }

    fn validate_draft_clearance(&self, spec: TrackPieceSpec) -> Result<(), TrackConstructionError> {
        let tile = self
            .tile(spec.pos)
            .expect("shape validation checks tile bounds");
        let height = spec.entry.height;
        let top = u16::from(height) + 2;
        if height < tile.height && top > u16::from(tile.height) {
            return Err(PlacementError::TerrainIntersection {
                pos: spec.pos,
                terrain_height: tile.height,
            }
            .into());
        }
        if let Some(object) = self.objects.iter().find(|object| {
            object.pos == spec.pos
                && !self.owns_circuit_object(object.id)
                && u16::from(height) < u16::from(object.height) + u16::from(object.clearance)
                && u16::from(object.height) < top
        }) {
            return Err(PlacementError::Occupied {
                pos: spec.pos,
                object: object.id,
            }
            .into());
        }
        Ok(())
    }
}

fn validate_piece_shape(park: &Park, spec: TrackPieceSpec) -> Result<(), TrackConstructionError> {
    if park.tile(spec.pos).is_none() {
        return Err(PlacementError::OutOfBounds { pos: spec.pos }.into());
    }
    if spec.entry.height > crate::MAX_TERRAIN_HEIGHT - 2 {
        return Err(PlacementError::HeightOutOfRange {
            height: spec.entry.height,
            clearance: 2,
            max: crate::MAX_TERRAIN_HEIGHT,
        }
        .into());
    }
    let straight = spec.entry.edge.opposite() == spec.exit.edge;
    if spec.entry.height != spec.exit.height
        || !match spec.kind {
            TrackKind::Station | TrackKind::Straight => straight,
            TrackKind::Curve => !straight && spec.entry.edge != spec.exit.edge,
            _ => false,
        }
    {
        return Err(TrackConstructionError::UnsupportedPiece);
    }
    Ok(())
}

/// Pending plans may become blocked by later world edits. Save only structural
/// constraints here; completion and occupied-space checks belong to Apply.
pub(crate) fn validate_saved_draft(park: &Park) -> Result<(), TrackConstructionError> {
    if park.track_draft.len() > MAX_TRACK_DRAFT_PIECES {
        return Err(TrackConstructionError::DraftFull);
    }
    let mut occupied = std::collections::BTreeSet::new();
    for spec in &park.track_draft {
        validate_piece_shape(park, *spec)?;
        if !occupied.insert((spec.pos.x, spec.pos.y)) {
            return Err(TrackConstructionError::DuplicateTile(spec.pos));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
