//! Construction and derived access checks for the single existing ride.
//! Queues do not admit, transport or board public guests in this stage.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Axis, ObjectId, ObjectKind, Park, PathDirection, PathSlope, PlacementError, SurfaceObject,
    TilePos, TrackDirection, TrackKind, paths_connect,
};

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct RideId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ride {
    pub id: RideId,
    #[serde(deserialize_with = "crate::saves::required_object_option")]
    pub entrance: Option<RideEntrance>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideEntrance {
    pub object: ObjectId,
    /// Points from the entrance tile toward its adjacent station.
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub facing: PathDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideAccessStatus {
    Ready,
    RideUnavailable,
    NoEntrance,
    StationDisconnected,
    NoQueue,
    Disconnected,
    Branch,
    Loop,
    NoPublicEntry,
    MultiplePublicEntries,
    PublicEntryNotAtEnd,
    ForeignQueue,
}

impl std::fmt::Display for RideAccessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Ready => "Queue connected",
            Self::RideUnavailable => "Ride circuit unavailable",
            Self::NoEntrance => "Build a ride entrance",
            Self::StationDisconnected => "Entrance does not reach station",
            Self::NoQueue => "Build queue from entrance",
            Self::Disconnected => "Queue has disconnected pieces",
            Self::Branch => "Queue has a branch",
            Self::Loop => "Queue has a loop",
            Self::NoPublicEntry => "Queue needs a public path",
            Self::MultiplePublicEntries => "Queue has multiple public connections",
            Self::PublicEntryNotAtEnd => "Connect public path at queue end",
            Self::ForeignQueue => "Queue touches another ride's queue",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RideAccess {
    pub status: RideAccessStatus,
    pub queue_tiles: usize,
    /// Populated only for Ready, in entrance-to-public order.
    pub ordered_queue: Vec<ObjectId>,
    pub public_entry: Option<ObjectId>,
    pub station: Option<ObjectId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RideAccessError {
    Placement(PlacementError),
    UnknownRide(RideId),
    CircuitUnavailable,
    EntranceAlreadyExists,
    NotBesideStation,
    WrongStationHeight { expected: u8, actual: u8 },
    InvalidStationSide,
}

impl From<PlacementError> for RideAccessError {
    fn from(value: PlacementError) -> Self {
        Self::Placement(value)
    }
}

impl std::fmt::Display for RideAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Placement(error) => error.fmt(f),
            Self::UnknownRide(id) => write!(f, "ride #{} does not exist", id.0),
            Self::CircuitUnavailable => {
                f.write_str("repair the ride circuit before placing an entrance")
            }
            Self::EntranceAlreadyExists => f.write_str("remove the existing ride entrance first"),
            Self::NotBesideStation => f.write_str("face an adjacent station tile"),
            Self::WrongStationHeight { expected, actual } => write!(
                f,
                "station height is {expected}; entrance height is {actual}"
            ),
            Self::InvalidStationSide => {
                f.write_str("place the entrance beside the station, away from its track ends")
            }
        }
    }
}
impl std::error::Error for RideAccessError {}

fn axis(facing: PathDirection) -> Axis {
    match facing {
        PathDirection::North | PathDirection::South => Axis::Y,
        _ => Axis::X,
    }
}

impl Park {
    pub fn ride(&self) -> Option<&Ride> {
        self.ride.as_ref()
    }

    fn require_ride(&self, id: RideId) -> Result<&Ride, RideAccessError> {
        self.ride
            .as_ref()
            .filter(|ride| ride.id == id)
            .ok_or(RideAccessError::UnknownRide(id))
    }

    pub fn validate_queue_path(
        &self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
        ride: RideId,
    ) -> Result<(), RideAccessError> {
        self.require_ride(ride)?;
        self.validate_path_placement(pos, height, slope)?;
        Ok(())
    }

    pub fn place_queue_path(
        &mut self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
        ride: RideId,
    ) -> Result<ObjectId, RideAccessError> {
        self.require_ride(ride)?;
        let next = self.validate_path_placement(pos, height, slope)?;
        Ok(self.insert_surface_object(
            SurfaceObject {
                id: ObjectId(0),
                pos,
                kind: ObjectKind::Path,
                axis: slope.axis(),
                path_slope: slope,
                queue_owner: Some(ride),
                height,
                clearance: slope.clearance(),
            },
            next,
        ))
    }

    pub fn validate_ride_entrance(
        &self,
        pos: TilePos,
        height: u8,
        facing: PathDirection,
        ride: RideId,
    ) -> Result<(), RideAccessError> {
        let owner = self.require_ride(ride)?;
        if owner.entrance.is_some() {
            return Err(RideAccessError::EntranceAlreadyExists);
        }
        self.validate_object_placement(
            pos,
            ObjectKind::RideEntrance,
            height,
            crate::PATH_CLEARANCE,
        )?;
        if self.objects.len() >= crate::saves::MAX_OBJECTS {
            return Err(PlacementError::CapacityExceeded {
                resource: "objects",
                max: crate::saves::MAX_OBJECTS,
            }
            .into());
        }
        self.entrance_station(pos, height, facing)?;
        Ok(())
    }

    pub fn place_ride_entrance(
        &mut self,
        pos: TilePos,
        height: u8,
        facing: PathDirection,
        ride: RideId,
    ) -> Result<ObjectId, RideAccessError> {
        self.validate_ride_entrance(pos, height, facing, ride)?;
        // The complete read-only validation above proves this allocator step.
        let next = self.next_object_id + 1;
        let object = self.insert_surface_object(
            SurfaceObject {
                id: ObjectId(0),
                pos,
                kind: ObjectKind::RideEntrance,
                axis: axis(facing),
                path_slope: PathSlope::Flat,
                queue_owner: None,
                height,
                clearance: crate::PATH_CLEARANCE,
            },
            next,
        );
        self.ride.as_mut().expect("validated ride").entrance =
            Some(RideEntrance { object, facing });
        Ok(object)
    }

    fn entrance_station(
        &self,
        pos: TilePos,
        height: u8,
        facing: PathDirection,
    ) -> Result<ObjectId, RideAccessError> {
        let circuit = self
            .circuit
            .as_ref()
            .ok_or(RideAccessError::CircuitUnavailable)?;
        let station_pos = facing
            .neighbor(pos)
            .ok_or(RideAccessError::NotBesideStation)?;
        let (piece, spec) = circuit
            .pieces
            .iter()
            .zip(&circuit.specs)
            .find(|(_, spec)| spec.kind == TrackKind::Station && spec.pos == station_pos)
            .ok_or(RideAccessError::NotBesideStation)?;
        let station_side = match facing.opposite() {
            PathDirection::North => TrackDirection::North,
            PathDirection::East => TrackDirection::East,
            PathDirection::South => TrackDirection::South,
            PathDirection::West => TrackDirection::West,
        };
        if spec.entry.edge == station_side || spec.exit.edge == station_side {
            return Err(RideAccessError::InvalidStationSide);
        }
        if height != spec.entry.height {
            return Err(RideAccessError::WrongStationHeight {
                expected: spec.entry.height,
                actual: height,
            });
        }
        Ok(piece.object)
    }

    fn gate_connects(
        &self,
        gate: &SurfaceObject,
        path: &SurfaceObject,
        require_owner: bool,
    ) -> bool {
        let Some(ride) = &self.ride else {
            return false;
        };
        let Some(entrance) = &ride.entrance else {
            return false;
        };
        if gate.kind != ObjectKind::RideEntrance
            || gate.id != entrance.object
            || path.kind != ObjectKind::Path
            || (require_owner && path.queue_owner != Some(ride.id))
        {
            return false;
        }
        let outer = entrance.facing.opposite();
        outer.neighbor(gate.pos) == Some(path.pos)
            && path.path_edge_height(entrance.facing) == Some(gate.height)
    }

    /// Physical deck joins, separate from public guest traversal. A transient
    /// queue preview can connect to an existing gate without an allocated ID.
    pub fn geometric_paths_connect(&self, a: &SurfaceObject, b: &SurfaceObject) -> bool {
        paths_connect(a, b) || self.gate_connects(a, b, true) || self.gate_connects(b, a, true)
    }

    pub fn geometric_path_neighbors(&self, id: ObjectId) -> Vec<ObjectId> {
        let Some(path) = self.object(id) else {
            return Vec::new();
        };
        let mut neighbors: Vec<_> = self
            .objects
            .iter()
            .filter(|other| self.geometric_paths_connect(path, other))
            .map(|other| other.id)
            .collect();
        neighbors.sort_unstable();
        neighbors
    }

    /// Deterministic priority: unavailable ride, absent/broken entrance, foreign
    /// contact, empty queue, branch, loop, disconnected fragments, public entry.
    /// All owned pieces must belong to the single entrance-to-public chain.
    pub fn ride_access(&self, id: RideId) -> RideAccess {
        let owned: BTreeMap<_, _> = self
            .objects
            .iter()
            .filter(|o| o.kind == ObjectKind::Path && o.queue_owner == Some(id))
            .map(|o| (o.id, o))
            .collect();
        let mut report = RideAccess {
            status: RideAccessStatus::RideUnavailable,
            queue_tiles: owned.len(),
            ordered_queue: Vec::new(),
            public_entry: None,
            station: None,
        };
        let Some(ride) = self.ride.as_ref().filter(|ride| ride.id == id) else {
            return report;
        };
        if self.circuit.is_none() {
            return report;
        }
        let Some(entrance) = &ride.entrance else {
            report.status = RideAccessStatus::NoEntrance;
            return report;
        };
        let Some(gate) = self.object(entrance.object) else {
            report.status = RideAccessStatus::NoEntrance;
            return report;
        };
        let Ok(station) = self.entrance_station(gate.pos, gate.height, entrance.facing) else {
            report.status = RideAccessStatus::StationDisconnected;
            return report;
        };
        report.station = Some(station);
        // Index physical ends once. Save/placement clearance forbids duplicate
        // ends at the same position/height, including vertically stacked paths.
        let paths: BTreeMap<_, _> = self
            .objects
            .iter()
            .filter(|o| o.kind == ObjectKind::Path)
            .map(|o| (o.id, o))
            .collect();
        let by_end: BTreeMap<_, _> = paths
            .values()
            .flat_map(|path| {
                PathDirection::ALL.into_iter().filter_map(|edge| {
                    path.path_edge_height(edge)
                        .map(|height| ((path.pos.x, path.pos.y, edge, height), path.id))
                })
            })
            .collect();
        let mut edges: BTreeMap<ObjectId, Vec<ObjectId>> = BTreeMap::new();
        let mut public_edges = Vec::new();
        let mut foreign = paths.values().any(|path| {
            path.queue_owner.is_some_and(|owner| owner != id)
                && self.gate_connects(gate, path, false)
        });
        for (&object, path) in &owned {
            let mut neighbors = Vec::new();
            for edge in PathDirection::ALL {
                let Some(height) = path.path_edge_height(edge) else {
                    continue;
                };
                let Some(pos) = edge.neighbor(path.pos) else {
                    continue;
                };
                let Some(other_id) = by_end.get(&(pos.x, pos.y, edge.opposite(), height)) else {
                    continue;
                };
                match paths[other_id].queue_owner {
                    Some(owner) if owner == id => neighbors.push(*other_id),
                    Some(_) => foreign = true,
                    None => public_edges.push((object, *other_id)),
                }
            }
            neighbors.sort_unstable();
            edges.insert(object, neighbors);
        }
        if foreign {
            report.status = RideAccessStatus::ForeignQueue;
            return report;
        }
        if owned.is_empty() {
            report.status = RideAccessStatus::NoQueue;
            return report;
        }
        let roots: Vec<_> = owned
            .values()
            .filter(|path| self.gate_connects(gate, path, true))
            .map(|path| path.id)
            .collect();
        if roots.len() > 1
            || edges.values().any(|neighbors| neighbors.len() > 2)
            || roots.first().is_some_and(|root| edges[root].len() > 1)
        {
            report.status = RideAccessStatus::Branch;
            return report;
        }
        // With degree <=2 every component is a chain or cycle. Check every
        // component, including fragments not currently attached to the gate.
        let mut seen = BTreeSet::new();
        for &start in owned.keys() {
            if seen.contains(&start) {
                continue;
            }
            let mut pending = vec![start];
            let mut nodes = 0;
            let mut edge_count = 0;
            while let Some(node) = pending.pop() {
                if !seen.insert(node) {
                    continue;
                }
                nodes += 1;
                edge_count += edges[&node].len();
                pending.extend(edges[&node].iter().copied());
            }
            if edge_count / 2 >= nodes {
                report.status = RideAccessStatus::Loop;
                return report;
            }
        }
        let Some(&root) = roots.first() else {
            report.status = RideAccessStatus::Disconnected;
            return report;
        };
        let mut ordered = Vec::new();
        let mut previous = None;
        let mut current = root;
        loop {
            ordered.push(current);
            let next = edges[&current]
                .iter()
                .copied()
                .find(|id| Some(*id) != previous);
            let Some(next) = next else {
                break;
            };
            previous = Some(current);
            current = next;
        }
        if ordered.len() != owned.len() {
            report.status = RideAccessStatus::Disconnected;
            return report;
        }
        if public_edges.is_empty() {
            report.status = RideAccessStatus::NoPublicEntry;
            return report;
        }
        if public_edges.len() != 1 {
            report.status = RideAccessStatus::MultiplePublicEntries;
            return report;
        }
        let (queue_end, public) = public_edges[0];
        if Some(&queue_end) != ordered.last() {
            report.status = RideAccessStatus::PublicEntryNotAtEnd;
            return report;
        }
        report.status = RideAccessStatus::Ready;
        report.ordered_queue = ordered;
        report.public_entry = Some(public);
        report
    }
}

/// Persisted references may remain disconnected after circuit damage/repair,
/// but ownership, physical gate shape and identity references must be sound.
pub(crate) fn validate_ride_state(park: &Park) -> Result<(), String> {
    if park.ride.is_some() != park.train.is_some() {
        return Err("ride identity and train presence disagree".into());
    }
    if park.next_ride_id == 0 {
        return Err("next ride identity is zero".into());
    }
    let ride_id = park.ride.as_ref().map(|ride| ride.id);
    if ride_id.is_some_and(|id| id.0 == 0 || id.0 >= park.next_ride_id) {
        return Err("ride identity would be reused".into());
    }
    let entrance = park.ride.as_ref().and_then(|ride| ride.entrance.as_ref());
    for object in &park.objects {
        if let Some(owner) = object.queue_owner
            && (object.kind != ObjectKind::Path
                || Some(owner) != ride_id
                || object.clearance != object.path_slope.clearance())
        {
            return Err("queue owner, path kind or clearance is invalid".into());
        }
        if object.kind == ObjectKind::RideEntrance {
            let Some(entrance) = entrance.filter(|entrance| entrance.object == object.id) else {
                return Err("entrance object has no ride metadata".into());
            };
            if object.path_slope != PathSlope::Flat
                || object.clearance != crate::PATH_CLEARANCE
                || object.axis != axis(entrance.facing)
                || object.queue_owner.is_some()
            {
                return Err("entrance has an invalid shape or facing axis".into());
            }
        }
    }
    if entrance.is_some_and(|entrance| {
        park.object(entrance.object)
            .is_none_or(|o| o.kind != ObjectKind::RideEntrance)
    }) {
        return Err("ride entrance references a missing entrance object".into());
    }
    Ok(())
}
