//! Persistent, uncommitted track plans and atomic single-coaster replacement.
use std::collections::BTreeMap;

use crate::{
    ObjectKind, Park, PathSlope, PlacementError, TilePos, TrackDirection, TrackKind, TrackPiece,
    TrackPieceSpec, TrackPort, Train, TrainCircuit, TrainError, TrainPhase,
};

pub const MAX_TRACK_DRAFT_PIECES: usize = 256;
pub const TRACK_REFUND_PERCENT: u8 = 50;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrackConstructionQuote {
    pub construction_cents: u64,
    pub refund_cents: u64,
    pub retained_pieces: usize,
    pub new_pieces: usize,
    pub removed_pieces: usize,
    pub available_cents: u64,
}
impl TrackConstructionQuote {
    pub fn required_cents(self) -> u64 {
        self.construction_cents.saturating_sub(self.refund_cents)
    }
    pub fn affordable(self) -> bool {
        self.balance_after().is_some()
    }
    pub fn balance_after(self) -> Option<u64> {
        self.available_cents
            .checked_add(self.refund_cents)?
            .checked_sub(self.construction_cents)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaidTrackPiece {
    pub object: crate::ObjectId,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub spec: TrackPieceSpec,
    pub paid_cents: u64,
}

/// Paid remnants survive a circuit fault so demolition cannot erase their basis.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackEconomy {
    #[serde(
        deserialize_with = "crate::saves::bounded_objects::<_, _, { crate::saves::MAX_OBJECTS }>"
    )]
    pub paid_pieces: Vec<PaidTrackPiece>,
    pub construction_cents: u64,
    pub refund_cents: u64,
}

pub fn track_piece_cost_cents(spec: TrackPieceSpec) -> u64 {
    match spec.kind {
        TrackKind::Station => 10_000,
        TrackKind::Straight => 1_000,
        TrackKind::Curve => 1_500,
        TrackKind::Lift => 2_500,
        TrackKind::Drop if spec.entry.height.abs_diff(spec.exit.height) == 2 => 2_000,
        TrackKind::Drop => 1_500,
        TrackKind::Brake => 2_000,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackBuildKind {
    Station,
    Straight,
    CurveLeft,
    CurveRight,
    Lift,
    Drop,
    SteepDrop,
    Brake,
}

impl TrackBuildKind {
    pub const fn cost_cents(self) -> u64 {
        match self {
            Self::Station => 10_000,
            Self::Straight => 1_000,
            Self::CurveLeft | Self::CurveRight | Self::Drop => 1_500,
            Self::Lift => 2_500,
            Self::SteepDrop | Self::Brake => 2_000,
        }
    }
    /// Lowest entry height that preserves this piece's complete grade.
    pub const fn min_height(self) -> u8 {
        match self {
            Self::Drop => 1,
            Self::SteepDrop => 2,
            _ => 0,
        }
    }

    /// Highest entry height with two steps of clearance above both endpoints.
    pub const fn max_height(self) -> u8 {
        match self {
            Self::Lift => crate::MAX_TERRAIN_HEIGHT - 3,
            _ => crate::MAX_TERRAIN_HEIGHT - 2,
        }
    }

    /// Height is the entry height. Direction is incoming travel, not the entry
    /// edge. Out-of-range grades produce invalid specs for preview validation;
    /// they never flatten or shorten into a different supported piece.
    pub fn spec(self, pos: TilePos, height: u8, direction: TrackDirection) -> TrackPieceSpec {
        use TrackDirection::{East, North, South, West};
        let exit = match self {
            Self::Station
            | Self::Straight
            | Self::Lift
            | Self::Drop
            | Self::SteepDrop
            | Self::Brake => direction,
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
                Self::Lift => TrackKind::Lift,
                Self::Drop | Self::SteepDrop => TrackKind::Drop,
                Self::Brake => TrackKind::Brake,
            },
            entry: TrackPort {
                edge: direction.opposite(),
                height,
            },
            exit: TrackPort {
                edge: exit,
                height: match self {
                    Self::Lift => height.saturating_add(1),
                    Self::Drop => height.checked_sub(1).unwrap_or(u8::MAX),
                    Self::SteepDrop => height.checked_sub(2).unwrap_or(u8::MAX),
                    _ => height,
                },
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrackConstructionError {
    Track(TrainError),
    DraftFull,
    DuplicateTile(TilePos),
    UnsupportedPiece,
    NoInstalledCircuit,
    IncompleteLoop(TilePos),
    DisconnectedPieces,
    CloseRide,
    GuestsStillVisiting,
    InsufficientFunds {
        required_cents: u64,
        available_cents: u64,
    },
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
                f.write_str("use level stations, straights, curves and brakes, one-step lifts, or one/two-step drops")
            }
            Self::NoInstalledCircuit => f.write_str("no complete installed coaster to copy"),
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
            Self::InsufficientFunds { required_cents, available_cents } => write!(
                f, "track needs ${}.{:02}; cash is ${}.{:02}",
                required_cents / 100, required_cents % 100,
                available_cents / 100, available_cents % 100
            ),
        }
    }
}
impl std::error::Error for TrackConstructionError {}

impl Park {
    pub fn track_economy(&self) -> &TrackEconomy {
        &self.track_economy
    }

    pub fn track_demolition_refund_cents(&self, id: crate::ObjectId) -> u64 {
        self.track_economy
            .paid_pieces
            .iter()
            .find(|piece| piece.object == id)
            .map_or(0, |piece| piece.paid_cents / 2)
    }

    pub(crate) fn refund_removed_track(&mut self, id: crate::ObjectId) {
        let Some(index) = self
            .track_economy
            .paid_pieces
            .iter()
            .position(|piece| piece.object == id)
        else {
            return;
        };
        let refund = self.track_economy.paid_pieces.remove(index).paid_cents / 2;
        self.track_economy.refund_cents += refund;
        self.record_ride_transaction(
            crate::FinanceKind::TrackRefund,
            self.ride.as_ref().expect("paid track has a ride owner").id,
            refund,
        );
    }

    /// Read-only estimate. Insufficient cash is represented in the quote so the
    /// editor can show both gross amounts and the net affordable balance.
    pub fn track_construction_quote(
        &self,
    ) -> Result<TrackConstructionQuote, TrackConstructionError> {
        let specs = self.validate_track_draft()?;
        Ok(self.quote_track_specs(&specs))
    }

    fn quote_track_specs(&self, specs: &[TrackPieceSpec]) -> TrackConstructionQuote {
        let old = self.circuit.as_ref();
        let retained_pieces = specs
            .iter()
            .filter(|spec| old.is_some_and(|c| c.specs.contains(spec)))
            .count();
        let construction_cents = specs
            .iter()
            .filter(|spec| old.is_none_or(|c| !c.specs.contains(spec)))
            .map(|spec| track_piece_cost_cents(*spec))
            .sum();
        let mut refund_cents = 0;
        let mut removed_pieces = 0;
        if let Some(circuit) = old {
            for (piece, spec) in circuit.pieces.iter().zip(&circuit.specs) {
                if !specs.contains(spec) {
                    removed_pieces += 1;
                    refund_cents += self.track_demolition_refund_cents(piece.object);
                }
            }
        }
        TrackConstructionQuote {
            construction_cents,
            refund_cents,
            retained_pieces,
            new_pieces: specs.len() - retained_pieces,
            removed_pieces,
            available_cents: self.cash_cents,
        }
    }
    pub fn track_draft(&self) -> &[TrackPieceSpec] {
        &self.track_draft
    }

    /// Restore an exact draft snapshot without touching the installed ride.
    /// Drafts may be incomplete or obstructed by later world changes; Apply
    /// rechecks connectivity, clearance, operating state and affordability.
    pub fn replace_track_draft(
        &mut self,
        specs: &[TrackPieceSpec],
    ) -> Result<bool, TrackConstructionError> {
        validate_draft_structure(self, specs)?;
        if self.track_draft == specs {
            return Ok(false);
        }
        self.track_draft = specs.to_vec();
        Ok(true)
    }

    /// Copy installed specifications exactly, even while the coaster is open
    /// or occupied. A missing or faulted circuit leaves the current plan intact.
    pub fn copy_installed_track_to_draft(&mut self) -> Result<bool, TrackConstructionError> {
        if self
            .train
            .as_ref()
            .is_none_or(|train| train.phase == TrainPhase::Fault)
        {
            return Err(TrackConstructionError::NoInstalledCircuit);
        }
        let specs = self
            .circuit
            .as_ref()
            .ok_or(TrackConstructionError::NoInstalledCircuit)?
            .specs
            .clone();
        self.validate_train_circuit()?;
        self.replace_track_draft(&specs)
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
        let additions = ordered
            .iter()
            .filter(|spec| {
                self.circuit
                    .as_ref()
                    .is_none_or(|c| !c.specs.contains(spec))
            })
            .count();
        self.next_object_id
            .checked_add(additions as u64)
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
        let quote = self.quote_track_specs(&specs);
        if !quote.affordable() {
            return Err(TrackConstructionError::InsufficientFunds {
                required_cents: quote.required_cents(),
                available_cents: self.cash_cents,
            });
        }
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
        let retained: Vec<_> = self
            .circuit
            .as_ref()
            .into_iter()
            .flat_map(|c| c.pieces.iter().zip(&c.specs))
            .filter(|(_, spec)| specs.contains(spec))
            .map(|(piece, spec)| (piece.object, *spec))
            .collect();
        candidate.objects.retain(|object| {
            !self.owns_circuit_object(object.id) || retained.iter().any(|(id, _)| *id == object.id)
        });
        candidate.track_economy.paid_pieces.retain(|piece| {
            !self.owns_circuit_object(piece.object)
                || retained.iter().any(|(id, _)| *id == piece.object)
        });
        let mut pieces = Vec::with_capacity(specs.len());
        let mut total_length = 0_u32;
        let mut station_stop = 0;
        for (spec, samples) in specs.iter().zip(geometry) {
            let (axis, height, clearance) = crate::trains::envelope(*spec);
            let object =
                if let Some((id, _)) = retained.iter().find(|(_, previous)| previous == spec) {
                    *id
                } else {
                    let object = crate::ObjectId(candidate.next_object_id);
                    candidate.next_object_id += 1; // Checked for the whole transaction above.
                    candidate.track_economy.paid_pieces.push(PaidTrackPiece {
                        object,
                        spec: *spec,
                        paid_cents: track_piece_cost_cents(*spec),
                    });
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
                    object
                };
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
        candidate
            .track_economy
            .paid_pieces
            .sort_by_key(|piece| piece.object);
        candidate.track_economy.construction_cents += quote.construction_cents;
        candidate.track_economy.refund_cents += quote.refund_cents;
        let owner = candidate.ride.as_ref().expect("circuit has owner").id;
        candidate.record_ride_transaction(
            crate::FinanceKind::TrackRefund,
            owner,
            quote.refund_cents,
        );
        candidate.record_ride_transaction(
            crate::FinanceKind::TrackConstruction,
            owner,
            quote.construction_cents,
        );
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
        let (_, height, clearance) = crate::trains::envelope(spec);
        let top = u16::from(height) + u16::from(clearance);
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
    let highest = spec.entry.height.max(spec.exit.height);
    if highest > crate::MAX_TERRAIN_HEIGHT - 2 {
        return Err(PlacementError::HeightOutOfRange {
            height: highest,
            clearance: 2,
            max: crate::MAX_TERRAIN_HEIGHT,
        }
        .into());
    }
    if !crate::trains::valid_piece_ports(spec) {
        return Err(TrackConstructionError::UnsupportedPiece);
    }
    Ok(())
}

/// Pending plans may become blocked by later world edits. Save only structural
/// constraints here; completion and occupied-space checks belong to Apply.
pub(crate) fn validate_saved_draft(park: &Park) -> Result<(), TrackConstructionError> {
    validate_draft_structure(park, &park.track_draft)
}

fn validate_draft_structure(
    park: &Park,
    specs: &[TrackPieceSpec],
) -> Result<(), TrackConstructionError> {
    if specs.len() > MAX_TRACK_DRAFT_PIECES {
        return Err(TrackConstructionError::DraftFull);
    }
    let mut occupied = std::collections::BTreeSet::new();
    for spec in specs {
        validate_piece_shape(park, *spec)?;
        if !occupied.insert((spec.pos.x, spec.pos.y)) {
            return Err(TrackConstructionError::DuplicateTile(spec.pos));
        }
    }
    Ok(())
}

pub(crate) fn validate_track_economy(park: &Park) -> Result<(), String> {
    let economy = &park.track_economy;
    if economy.paid_pieces.len() > crate::saves::MAX_OBJECTS
        || (park.ride.is_none() && *economy != TrackEconomy::default())
        || !economy.construction_cents.is_multiple_of(500)
        || !economy.refund_cents.is_multiple_of(250)
        || (economy.construction_cents != 0 && economy.construction_cents < 1_000)
        || (economy.refund_cents != 0 && economy.refund_cents < 500)
    {
        return Err("track economy owner, amounts or bounds are invalid".into());
    }
    let mut previous = None;
    let mut basis = 0_u64;
    for paid in &economy.paid_pieces {
        if previous.is_some_and(|id| id >= paid.object)
            || paid.paid_cents != track_piece_cost_cents(paid.spec)
            || validate_piece_shape(park, paid.spec).is_err()
        {
            return Err("paid track record, order or fixed basis is invalid".into());
        }
        previous = Some(paid.object);
        let object = park
            .object(paid.object)
            .ok_or("paid track object is missing")?;
        let (axis, height, clearance) = crate::trains::envelope(paid.spec);
        if object.kind != ObjectKind::Track
            || object.pos != paid.spec.pos
            || object.axis != axis
            || object.height != height
            || object.clearance != clearance
            || park.circuit.as_ref().is_some_and(|circuit| {
                circuit
                    .pieces
                    .iter()
                    .zip(&circuit.specs)
                    .any(|(piece, spec)| piece.object == paid.object && *spec != paid.spec)
            })
        {
            return Err("paid track specification disagrees with its object or circuit".into());
        }
        basis = basis
            .checked_add(paid.paid_cents)
            .ok_or("paid track sum overflows")?;
    }
    if economy
        .refund_cents
        .checked_mul(2)
        .and_then(|refund| refund.checked_add(basis))
        != Some(economy.construction_cents)
    {
        return Err("track purchases, remaining basis and refunds do not conserve funds".into());
    }
    Ok(())
}

#[cfg(test)]
mod economy_tests;
#[cfg(test)]
mod editing_tests;
#[cfg(test)]
mod tests;
