//! Read-only access guidance. Every proposed tile still uses the normal
//! placement validator; suggestions never reserve space or construct anything.
use std::collections::BTreeSet;

use crate::{
    ObjectId, ObjectKind, Park, PathDirection, PathSlope, RideAccess, RideAccessError,
    RideAccessStatus, RideEntrance, RideExitAccess, RideExitStatus, RideId, SurfaceObject, TilePos,
    TrackKind, paths_connect,
};

mod bridge;
#[cfg(test)]
mod bridge_tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideAccessBuildKind {
    Entrance,
    Exit,
    Queue,
    PublicPath,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideAccessSuggestion {
    pub kind: RideAccessBuildKind,
    pub pos: TilePos,
    pub height: u8,
    /// Points toward the station, gate or path that this proposal joins.
    pub facing: PathDirection,
    pub slope: PathSlope,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideAccessConnection {
    Entrance(RideAccessStatus),
    Exit(RideExitStatus),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideGateCandidate {
    pub pos: TilePos,
    pub height: u8,
    pub facing: PathDirection,
    /// Access status after placing only this gate in an isolated projection.
    pub connection: RideAccessConnection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RideAccessPlan {
    pub ride: RideId,
    pub available: bool,
    pub ride_open: bool,
    pub ride_running: bool,
    pub passengers: usize,
    pub entrance: RideAccess,
    pub exit: RideExitAccess,
    pub entrance_candidates: Vec<RideGateCandidate>,
    pub exit_candidates: Vec<RideGateCandidate>,
    /// Ranked, currently legal next tiles, not a reserved or complete route.
    pub suggestions: Vec<RideAccessSuggestion>,
    pub advice: &'static str,
}

impl Park {
    /// Derive access guidance without changing objects, guests, allocators,
    /// money, time, RNG or navigation caches. Consumers should cache this report
    /// until the park changes and revalidate a selected tile before building.
    pub fn ride_access_plan(&self, id: RideId) -> Result<RideAccessPlan, RideAccessError> {
        let ride = self.require_ride(id)?;
        let mut plan = RideAccessPlan {
            ride: id,
            available: self.ride_available(id),
            ride_open: self.ride_is_open(id),
            ride_running: self.ride_is_running(id),
            passengers: self
                .guests
                .iter()
                .filter(|g| {
                    g.ride_visit
                        .as_ref()
                        .is_some_and(|v| v.ride == id && v.passenger.is_some())
                })
                .count(),
            entrance: self.ride_access(id),
            exit: self.ride_exit_access(id),
            entrance_candidates: Vec::new(),
            exit_candidates: Vec::new(),
            suggestions: Vec::new(),
            advice: "Restore this ride before planning its access.",
        };
        if !plan.available {
            return Ok(plan);
        }

        // At most one park copy per report, shared by all gate/queue projections.
        // Existing portals and complete access need no copy at all.
        let mut projection = None;
        for (pos, height, facing) in self.access_gate_sites(id) {
            if self.validate_ride_entrance(pos, height, facing, id).is_ok() {
                let projected = project_gate(
                    projection.get_or_insert_with(|| self.clone()),
                    id,
                    pos,
                    height,
                    facing,
                    RideAccessBuildKind::Entrance,
                );
                plan.entrance_candidates.push(RideGateCandidate {
                    pos,
                    height,
                    facing,
                    connection: projected,
                });
            }
            if self.validate_ride_exit(pos, height, facing, id).is_ok() {
                let projected = project_gate(
                    projection.get_or_insert_with(|| self.clone()),
                    id,
                    pos,
                    height,
                    facing,
                    RideAccessBuildKind::Exit,
                );
                plan.exit_candidates.push(RideGateCandidate {
                    pos,
                    height,
                    facing,
                    connection: projected,
                });
            }
        }
        let rank = |candidate: &RideGateCandidate| {
            (
                connection_rank(candidate.connection),
                self.distance_to_public(candidate.pos, candidate.height),
                candidate.pos.y,
                candidate.pos.x,
                candidate.facing,
            )
        };
        plan.entrance_candidates.sort_by_key(rank);
        plan.exit_candidates.sort_by_key(rank);
        if let Some(candidate) = plan.entrance_candidates.first() {
            plan.suggestions
                .push(gate_suggestion(*candidate, RideAccessBuildKind::Entrance));
        }
        // When both gates are missing, avoid proposing the same tile twice.
        if let Some(candidate) = plan.exit_candidates.iter().find(|candidate| {
            !plan
                .suggestions
                .iter()
                .any(|s| s.pos == candidate.pos && s.height == candidate.height)
        }) {
            plan.suggestions
                .push(gate_suggestion(*candidate, RideAccessBuildKind::Exit));
        }

        if matches!(
            plan.entrance.status,
            RideAccessStatus::NoQueue | RideAccessStatus::Disconnected
        ) && let Some(entrance) = ride.entrance
            && let Some(gate) = self.object(entrance.object)
            && let Some(pos) = entrance.facing.opposite().neighbor(gate.pos)
        {
            let mut queues = Vec::new();
            for suggestion in self.access_path_choices(
                pos,
                gate.height,
                entrance.facing,
                RideAccessBuildKind::Queue,
                id,
            ) {
                let candidate = projection.get_or_insert_with(|| self.clone());
                candidate.objects.push(path_object(
                    suggestion,
                    ObjectId(candidate.next_object_id),
                    Some(id),
                ));
                let status = candidate.ride_access(id).status;
                candidate.objects.pop();
                if matches!(
                    status,
                    RideAccessStatus::Ready
                        | RideAccessStatus::NoPublicEntry
                        | RideAccessStatus::Disconnected
                ) {
                    queues.push((
                        connection_rank(RideAccessConnection::Entrance(status)),
                        suggestion,
                    ));
                }
            }
            queues.sort_by_key(|(rank, suggestion)| {
                (
                    *rank,
                    self.distance_to_public(suggestion.pos, suggestion.height),
                    slope_rank(suggestion.slope),
                    suggestion.height,
                )
            });
            plan.suggestions
                .extend(queues.into_iter().map(|(_, suggestion)| suggestion));
        }

        let mut public = Vec::new();
        let mut public_queue_end = None;
        if plan.entrance.status == RideAccessStatus::NoPublicEntry
            && let Some(entrance) = ride.entrance
            && let Some(gate) = self.object(entrance.object)
        {
            let owned: Vec<_> = self
                .objects
                .iter()
                .filter(|o| o.kind == ObjectKind::Path && o.queue_owner == Some(id))
                .collect();
            // NoPublicEntry already proves a single unbranched, connected chain.
            let tail = owned.iter().find(|path| {
                owned.len() == 1
                    || (!portal_touches(path, gate, entrance.facing)
                        && owned
                            .iter()
                            .filter(|other| paths_connect(path, other))
                            .count()
                            == 1)
            });
            if let Some(tail) = tail {
                public_queue_end = Some(tail.id);
                self.add_public_neighbors(tail, id, &mut public);
            }
        }
        if plan.exit.status == RideExitStatus::NoPublicExit
            && let Some(exit) = ride.exit
            && let Some(gate) = self.object(exit.object)
            && let Some(pos) = exit.facing.opposite().neighbor(gate.pos)
        {
            public.extend(self.access_path_choices(
                pos,
                gate.height,
                exit.facing,
                RideAccessBuildKind::PublicPath,
                id,
            ));
        }
        if plan.exit.status == RideExitStatus::PublicDisconnected
            && let (Some(landing), Some(entry)) =
                (plan.exit.public_exit, plan.entrance.public_entry)
        {
            self.add_public_bridge_choices(landing, entry, id, &mut public);
        }
        // A public tile must not create a second queue entry or attach to the
        // middle of a queue. Every advertised placement also passes its normal
        // capacity, allocator, terrain and occupied-space validator.
        public.retain(|suggestion| {
            let proposed = path_object(*suggestion, ObjectId(0), None);
            if self.objects.iter().any(|path| {
                path.queue_owner.is_some_and(|owner| owner != id) && paths_connect(&proposed, path)
            }) {
                return false;
            }
            let queue_contacts: Vec<_> = self
                .objects
                .iter()
                .filter(|path| path.queue_owner == Some(id) && paths_connect(&proposed, path))
                .map(|path| path.id)
                .collect();
            match plan.entrance.status {
                RideAccessStatus::NoPublicEntry => {
                    queue_contacts.is_empty()
                        || (queue_contacts.len() == 1
                            && Some(queue_contacts[0]) == public_queue_end)
                }
                _ => queue_contacts.is_empty(),
            }
        });
        public.sort_by_key(|suggestion| {
            let proposed = path_object(*suggestion, ObjectId(0), None);
            let joins_public = self
                .objects
                .iter()
                .any(|path| path.is_public_path() && paths_connect(&proposed, path));
            let joins_queue_end = public_queue_end
                .and_then(|id| self.object(id))
                .is_some_and(|tail| paths_connect(&proposed, tail));
            let distance = if plan.exit.status == RideExitStatus::PublicDisconnected {
                plan.entrance
                    .public_entry
                    .and_then(|id| self.object(id))
                    .map(|entry| tile_distance(suggestion.pos, suggestion.height, entry))
                    .unwrap_or_else(|| self.distance_to_public(suggestion.pos, suggestion.height))
            } else {
                self.distance_to_public(suggestion.pos, suggestion.height)
            };
            (
                public_queue_end.is_some() && !joins_queue_end,
                !joins_public,
                distance,
                slope_rank(suggestion.slope),
                suggestion.pos.y,
                suggestion.pos.x,
                suggestion.height,
                suggestion.facing,
            )
        });
        public.dedup();
        plan.suggestions.extend(public);
        plan.advice = access_advice(&plan);
        Ok(plan)
    }

    fn access_gate_sites(&self, id: RideId) -> Vec<(TilePos, u8, PathDirection)> {
        let mut stations = Vec::new();
        if let Some(carousel) = self.carousel(id) {
            for dy in 0..3 {
                for dx in 0..3 {
                    stations.push((
                        TilePos {
                            x: carousel.origin.x + dx,
                            y: carousel.origin.y + dy,
                        },
                        carousel.height,
                    ));
                }
            }
        } else if let Some(circuit) = &self.circuit {
            stations.extend(
                circuit
                    .specs
                    .iter()
                    .filter(|spec| spec.kind == TrackKind::Station)
                    .map(|spec| (spec.pos, spec.entry.height)),
            );
        }
        let mut sites = BTreeSet::new();
        for (station, height) in stations {
            for outward in PathDirection::ALL {
                if let Some(pos) = outward.neighbor(station)
                    && self.tile(pos).is_some()
                    && self
                        .entrance_station(id, pos, height, outward.opposite())
                        .is_ok()
                {
                    sites.insert((pos.y, pos.x, height, outward.opposite()));
                }
            }
        }
        sites
            .into_iter()
            .map(|(y, x, height, facing)| (TilePos { x, y }, height, facing))
            .collect()
    }

    fn distance_to_public(&self, pos: TilePos, height: u8) -> u32 {
        self.objects
            .iter()
            .filter(|o| o.is_public_path())
            .map(|o| tile_distance(pos, height, o))
            .min()
            .unwrap_or(u32::MAX)
    }

    fn access_path_choices(
        &self,
        pos: TilePos,
        edge_height: u8,
        toward: PathDirection,
        kind: RideAccessBuildKind,
        ride: RideId,
    ) -> Vec<RideAccessSuggestion> {
        [
            (Some(edge_height), PathSlope::Flat),
            (edge_height.checked_sub(1), direction_slope(toward)),
            (Some(edge_height), direction_slope(toward.opposite())),
        ]
        .into_iter()
        .filter_map(|(height, slope)| {
            let height = height?;
            let valid = if kind == RideAccessBuildKind::Queue {
                self.validate_queue_path(pos, height, slope, ride).is_ok()
            } else {
                self.validate_sloped_path(pos, height, slope).is_ok()
            };
            valid.then_some(RideAccessSuggestion {
                kind,
                pos,
                height,
                facing: toward,
                slope,
            })
        })
        .collect()
    }

    fn add_public_neighbors(
        &self,
        anchor: &SurfaceObject,
        ride: RideId,
        choices: &mut Vec<RideAccessSuggestion>,
    ) {
        for direction in PathDirection::ALL {
            if let (Some(pos), Some(height)) = (
                direction.neighbor(anchor.pos),
                anchor.path_edge_height(direction),
            ) {
                choices.extend(self.access_path_choices(
                    pos,
                    height,
                    direction.opposite(),
                    RideAccessBuildKind::PublicPath,
                    ride,
                ));
            }
        }
    }

    fn add_public_bridge_choices(
        &self,
        landing: ObjectId,
        entry: ObjectId,
        ride: RideId,
        choices: &mut Vec<RideAccessSuggestion>,
    ) {
        let graph = crate::crowds::PathGraph::new(&self.objects);
        let component = |start| {
            let mut found = BTreeSet::new();
            let mut pending = vec![start];
            while let Some(id) = pending.pop() {
                if found.insert(id) {
                    pending.extend(graph.edges.get(&id).into_iter().flatten().copied());
                }
            }
            found
        };
        let destinations: Vec<_> = component(entry)
            .iter()
            .filter_map(|id| graph.nodes.get(id))
            .collect();
        let origins: Vec<_> = component(landing)
            .iter()
            .filter_map(|id| graph.nodes.get(id))
            .collect();
        choices.extend(bridge::first_step(self, ride, &origins, &destinations));
    }
}

fn project_gate(
    park: &mut Park,
    ride: RideId,
    pos: TilePos,
    height: u8,
    facing: PathDirection,
    kind: RideAccessBuildKind,
) -> RideAccessConnection {
    let object = ObjectId(park.next_object_id);
    park.objects.push(SurfaceObject {
        id: object,
        pos,
        kind: if kind == RideAccessBuildKind::Entrance {
            ObjectKind::RideEntrance
        } else {
            ObjectKind::RideExit
        },
        axis: facing.axis(),
        path_slope: PathSlope::Flat,
        queue_owner: None,
        height,
        clearance: crate::PATH_CLEARANCE,
    });
    let portal = RideEntrance { object, facing };
    let connection = if kind == RideAccessBuildKind::Entrance {
        let previous = park.ride_by_id_mut(ride).unwrap().entrance.replace(portal);
        let status = park.ride_access(ride).status;
        park.ride_by_id_mut(ride).unwrap().entrance = previous;
        RideAccessConnection::Entrance(status)
    } else {
        let previous = park.ride_by_id_mut(ride).unwrap().exit.replace(portal);
        let status = park.ride_exit_access(ride).status;
        park.ride_by_id_mut(ride).unwrap().exit = previous;
        RideAccessConnection::Exit(status)
    };
    park.objects.pop();
    connection
}

fn gate_suggestion(
    candidate: RideGateCandidate,
    kind: RideAccessBuildKind,
) -> RideAccessSuggestion {
    RideAccessSuggestion {
        kind,
        pos: candidate.pos,
        height: candidate.height,
        facing: candidate.facing,
        slope: PathSlope::Flat,
    }
}

fn path_object(
    suggestion: RideAccessSuggestion,
    id: ObjectId,
    owner: Option<RideId>,
) -> SurfaceObject {
    SurfaceObject {
        id,
        pos: suggestion.pos,
        kind: ObjectKind::Path,
        axis: suggestion.slope.axis(),
        height: suggestion.height,
        clearance: suggestion.slope.clearance(),
        path_slope: suggestion.slope,
        queue_owner: owner,
    }
}

fn portal_touches(path: &SurfaceObject, gate: &SurfaceObject, facing: PathDirection) -> bool {
    facing.opposite().neighbor(gate.pos) == Some(path.pos)
        && path.path_edge_height(facing) == Some(gate.height)
}

fn direction_slope(direction: PathDirection) -> PathSlope {
    match direction {
        PathDirection::North => PathSlope::North,
        PathDirection::East => PathSlope::East,
        PathDirection::South => PathSlope::South,
        PathDirection::West => PathSlope::West,
    }
}

fn slope_rank(slope: PathSlope) -> u8 {
    match slope {
        PathSlope::Flat => 0,
        PathSlope::North => 1,
        PathSlope::East => 2,
        PathSlope::South => 3,
        PathSlope::West => 4,
    }
}

fn tile_distance(pos: TilePos, height: u8, path: &SurfaceObject) -> u32 {
    pos.x.abs_diff(path.pos.x)
        + pos.y.abs_diff(path.pos.y)
        + u32::from(height.abs_diff(path.height)) * 2
}

fn connection_rank(connection: RideAccessConnection) -> u8 {
    match connection {
        RideAccessConnection::Entrance(RideAccessStatus::Ready)
        | RideAccessConnection::Exit(RideExitStatus::Ready) => 0,
        RideAccessConnection::Entrance(RideAccessStatus::NoPublicEntry)
        | RideAccessConnection::Exit(RideExitStatus::PublicDisconnected) => 1,
        RideAccessConnection::Entrance(
            RideAccessStatus::PublicEntryNotAtEnd | RideAccessStatus::MultiplePublicEntries,
        ) => 2,
        RideAccessConnection::Entrance(RideAccessStatus::Disconnected) => 3,
        RideAccessConnection::Entrance(RideAccessStatus::NoQueue)
        | RideAccessConnection::Exit(RideExitStatus::NoPublicExit) => 4,
        _ => 5,
    }
}

fn access_advice(plan: &RideAccessPlan) -> &'static str {
    match plan.entrance.status {
        RideAccessStatus::RideUnavailable => "Restore this ride before planning its access.",
        RideAccessStatus::NoEntrance if plan.entrance_candidates.is_empty() => {
            "No legal entrance site; check station-side space, terrain and construction limits."
        }
        RideAccessStatus::NoEntrance => "Place an entrance beside the station, facing the ride.",
        RideAccessStatus::StationDisconnected => {
            "Move the entrance beside the current station at its height."
        }
        RideAccessStatus::NoQueue
            if !plan
                .suggestions
                .iter()
                .any(|s| s.kind == RideAccessBuildKind::Queue) =>
        {
            "No legal queue tile outside the entrance; check deck height, terrain, space and construction limits."
        }
        RideAccessStatus::NoQueue => "Build a queue outward from the entrance at its deck height.",
        RideAccessStatus::Disconnected => {
            "Join every owned queue tile into one chain leading to the entrance."
        }
        RideAccessStatus::Branch => {
            "Remove queue branches so each guest follows one route to the entrance."
        }
        RideAccessStatus::Loop => "Break the queue loop and connect its end to the entrance.",
        RideAccessStatus::NoPublicEntry
            if !plan
                .suggestions
                .iter()
                .any(|s| s.kind == RideAccessBuildKind::PublicPath) =>
        {
            "No legal public connection at the queue end; clear space or reshape the queue."
        }
        RideAccessStatus::NoPublicEntry => "Connect a public path to the far end of the queue.",
        RideAccessStatus::MultiplePublicEntries => {
            "Keep exactly one public connection at the far end of the queue."
        }
        RideAccessStatus::PublicEntryNotAtEnd => {
            "Move the public connection to the far end of the queue."
        }
        RideAccessStatus::ForeignQueue => "Separate this queue from the other ride's queue.",
        RideAccessStatus::Ready => match plan.exit.status {
            RideExitStatus::NoExit if plan.exit_candidates.is_empty() => {
                "No legal exit site; check station-side space, terrain and construction limits."
            }
            RideExitStatus::NoExit => "Place an exit beside the station, facing the ride.",
            RideExitStatus::StationDisconnected => {
                "Move the exit beside the current station at its height."
            }
            RideExitStatus::NoPublicExit
                if !plan
                    .suggestions
                    .iter()
                    .any(|s| s.kind == RideAccessBuildKind::PublicPath) =>
            {
                "No legal public path outside the exit; check deck height, terrain, space and construction limits."
            }
            RideExitStatus::NoPublicExit => {
                "Build a public path outward from the exit at its deck height."
            }
            RideExitStatus::PublicDisconnected => {
                "The exit gate joins the ride. Continue its public path to the queue-entry network."
            }
            RideExitStatus::Ready => {
                "Entrance, queue and exit are connected. Extend their public network to reach your guests."
            }
            RideExitStatus::RideUnavailable => "Restore this ride before planning its access.",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TrackBuildKind, TrackDesign, TrackDirection};

    fn coaster(turns: u8, height: u8) -> (Park, RideId) {
        use TrackBuildKind::{CurveRight, Station, Straight};
        use TrackDirection::{East, North, South, West};
        let specs: Vec<_> = [
            (1, 0, Station, East),
            (2, 0, CurveRight, East),
            (2, 1, Straight, South),
            (2, 2, CurveRight, South),
            (1, 2, Straight, West),
            (0, 2, CurveRight, West),
            (0, 1, Straight, North),
            (0, 0, CurveRight, North),
        ]
        .into_iter()
        .map(|(x, y, kind, direction)| kind.spec(TilePos { x, y }, 8, direction))
        .collect();
        let design = TrackDesign::from_draft("Access fixture", &specs).unwrap();
        let specs = design
            .transformed(TilePos { x: 10, y: 10 }, height, turns)
            .unwrap();
        let mut park = Park::new(32, 32);
        park.replace_track_draft(&specs).unwrap();
        park.apply_track_draft().unwrap();
        let id = park.ride().unwrap().id;
        (park, id)
    }

    fn valid_suggestions(park: &Park, id: RideId, report: &RideAccessPlan) {
        for suggestion in &report.suggestions {
            let result = match suggestion.kind {
                RideAccessBuildKind::Entrance => park.validate_ride_entrance(
                    suggestion.pos,
                    suggestion.height,
                    suggestion.facing,
                    id,
                ),
                RideAccessBuildKind::Exit => park.validate_ride_exit(
                    suggestion.pos,
                    suggestion.height,
                    suggestion.facing,
                    id,
                ),
                RideAccessBuildKind::Queue => park.validate_queue_path(
                    suggestion.pos,
                    suggestion.height,
                    suggestion.slope,
                    id,
                ),
                RideAccessBuildKind::PublicPath => park
                    .validate_sloped_path(suggestion.pos, suggestion.height, suggestion.slope)
                    .map_err(RideAccessError::from),
            };
            assert_eq!(result, Ok(()), "invalid suggestion {suggestion:?}");
        }
    }

    fn unchanged_report(park: &Park, id: RideId) -> RideAccessPlan {
        let before = park.clone();
        let bytes = park.to_save_bytes().unwrap();
        let cache = park.path_cache_stats();
        let report = park.ride_access_plan(id).unwrap();
        assert_eq!(park.ride_access_plan(id).unwrap(), report);
        valid_suggestions(park, id, &report);
        assert_eq!(park, &before);
        assert_eq!(park.to_save_bytes().unwrap(), bytes);
        assert_eq!(park.path_cache_stats(), cache);
        let loaded = Park::from_save_bytes(&bytes).unwrap();
        assert_eq!(loaded.ride_access_plan(id).unwrap(), report);
        let mut reordered = park.clone();
        reordered.objects.reverse();
        assert_eq!(reordered.ride_access_plan(id).unwrap(), report);
        report
    }

    #[test]
    fn paid_rotated_and_raised_stations_propose_only_valid_lateral_gates() {
        for turns in 0..4 {
            for height in [0, 8, 20, 30] {
                let (mut park, id) = coaster(turns, height);
                let report = unchanged_report(&park, id);
                assert_eq!(report.entrance_candidates.len(), 2);
                assert_eq!(report.exit_candidates.len(), 2);
                assert_eq!(report.suggestions.len(), 2);
                assert_ne!(report.suggestions[0].pos, report.suggestions[1].pos);
                for candidate in &report.entrance_candidates {
                    assert_eq!(candidate.height, height);
                    assert_eq!(
                        candidate.facing.neighbor(candidate.pos),
                        Some(TilePos { x: 10, y: 10 })
                    );
                    assert_eq!(
                        candidate.facing.axis(),
                        if turns % 2 == 0 {
                            crate::Axis::Y
                        } else {
                            crate::Axis::X
                        }
                    );
                    let mut built = park.clone();
                    built
                        .place_ride_entrance(candidate.pos, height, candidate.facing, id)
                        .unwrap();
                    assert_eq!(
                        candidate.connection,
                        RideAccessConnection::Entrance(built.ride_access(id).status)
                    );
                }
                let gate = report.suggestions[0];
                park.place_ride_entrance(gate.pos, gate.height, gate.facing, id)
                    .unwrap();
                let after = unchanged_report(&park, id);
                assert!(after.entrance_candidates.is_empty());
                assert_eq!(after.exit_candidates.len(), 1);
                // A new, unapplied station draft must not redirect installed access.
                park.replace_track_draft(&[TrackBuildKind::Station.spec(
                    TilePos { x: 2, y: 2 },
                    8,
                    TrackDirection::East,
                )])
                .unwrap();
                assert_eq!(park.ride_access_plan(id).unwrap(), after);
            }
        }
    }

    #[test]
    fn carousel_perimeter_map_edges_terrain_and_occupancy_match_placement_rules() {
        let mut park = Park::new(32, 32);
        let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        let report = unchanged_report(&park, id);
        assert_eq!(report.entrance_candidates.len(), 12);
        assert_eq!(report.exit_candidates.len(), 12);
        let occupied = report.entrance_candidates[0];
        let raised = report.entrance_candidates[1];
        park.place_path(occupied.pos, occupied.height).unwrap();
        park.set_terrain_height(raised.pos, raised.height + 1)
            .unwrap();
        let blocked = unchanged_report(&park, id);
        assert_eq!(blocked.entrance_candidates.len(), 10);
        assert!(
            blocked
                .entrance_candidates
                .iter()
                .all(|c| c.pos != occupied.pos && c.pos != raised.pos)
        );
        for candidate in blocked.entrance_candidates {
            park.place_path(candidate.pos, candidate.height).unwrap();
        }
        let blocked = unchanged_report(&park, id);
        assert!(blocked.entrance_candidates.is_empty() && blocked.exit_candidates.is_empty());
        assert!(blocked.suggestions.is_empty());
        assert!(blocked.advice.contains("No legal entrance"));
        let mut edge = Park::new(3, 4);
        let id = edge.place_carousel(TilePos { x: 0, y: 0 }, 0).unwrap();
        let edge = unchanged_report(&edge, id);
        assert_eq!(edge.entrance_candidates.len(), 3);
        assert!(
            edge.entrance_candidates
                .iter()
                .all(|c| c.pos.y == 3 && c.facing == PathDirection::North)
        );
    }

    #[test]
    fn matching_sloped_queue_is_ranked_ahead_of_disconnected_level_options() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let id = park.ride().unwrap().id;
        park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, id)
            .unwrap();
        park.place_path(TilePos { x: 8, y: 13 }, 7).unwrap();
        let report = unchanged_report(&park, id);
        let queue = report
            .suggestions
            .iter()
            .find(|s| s.kind == RideAccessBuildKind::Queue)
            .unwrap();
        assert_eq!(
            (queue.pos, queue.height, queue.slope),
            (TilePos { x: 8, y: 12 }, 7, PathSlope::North)
        );
        park.place_queue_path(queue.pos, queue.height, queue.slope, id)
            .unwrap();
        assert_eq!(park.ride_access(id).status, RideAccessStatus::Ready);
        let report = unchanged_report(&park, id);
        let exit = report.exit_candidates[0];
        park.place_ride_exit(exit.pos, exit.height, exit.facing, id)
            .unwrap();
        park.place_path(TilePos { x: 8, y: 7 }, 7).unwrap();
        let report = unchanged_report(&park, id);
        let public = report
            .suggestions
            .iter()
            .find(|s| s.kind == RideAccessBuildKind::PublicPath)
            .unwrap();
        assert_eq!(
            (public.pos, public.height, public.slope),
            (TilePos { x: 8, y: 8 }, 7, PathSlope::South)
        );
        park.place_sloped_path(public.pos, public.height, public.slope)
            .unwrap();
        assert_eq!(
            park.ride_exit_access(id).status,
            RideExitStatus::PublicDisconnected
        );
    }

    #[test]
    fn public_suggestions_never_attach_to_queue_middle_or_foreign_queue() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let foreign = park.ride().unwrap().id;
        let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        park.place_ride_entrance(TilePos { x: 18, y: 23 }, 8, PathDirection::East, id)
            .unwrap();
        for (x, y) in [
            (17, 23),
            (17, 22),
            (17, 21),
            (17, 20),
            (18, 20),
            (19, 20),
            (19, 19),
            (19, 18),
        ] {
            park.place_queue_path(TilePos { x, y }, 8, PathSlope::Flat, id)
                .unwrap();
        }
        park.place_ride_exit(TilePos { x: 20, y: 21 }, 8, PathDirection::South, id)
            .unwrap();
        assert_eq!(park.ride_access(id).status, RideAccessStatus::NoPublicEntry);
        let report = unchanged_report(&park, id);
        assert!(
            report
                .suggestions
                .iter()
                .any(|s| s.kind == RideAccessBuildKind::PublicPath
                    && s.pos == TilePos { x: 20, y: 18 }
                    && s.slope == PathSlope::Flat)
        );
        assert!(
            !report
                .suggestions
                .iter()
                .any(|s| s.kind == RideAccessBuildKind::PublicPath
                    && s.pos == TilePos { x: 20, y: 20 }
                    && s.slope == PathSlope::Flat)
        );
        park.place_queue_path(TilePos { x: 21, y: 18 }, 8, PathSlope::Flat, foreign)
            .unwrap();
        let report = unchanged_report(&park, id);
        assert!(
            !report
                .suggestions
                .iter()
                .any(|s| s.kind == RideAccessBuildKind::PublicPath
                    && s.pos == TilePos { x: 20, y: 18 }
                    && s.slope == PathSlope::Flat)
        );
        for proposal in report
            .suggestions
            .iter()
            .filter(|s| s.kind == RideAccessBuildKind::PublicPath)
        {
            let mut built = park.clone();
            built
                .place_sloped_path(proposal.pos, proposal.height, proposal.slope)
                .unwrap();
            assert!(matches!(
                built.ride_access(id).status,
                RideAccessStatus::Ready | RideAccessStatus::NoPublicEntry
            ));
            assert_eq!(built.ride_access(foreign), park.ride_access(foreign));
        }
    }

    #[test]
    fn public_bridge_suggestions_connect_existing_networks_without_extra_queue_entries() {
        let mut park = Park::new(32, 32);
        let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        park.place_ride_entrance(TilePos { x: 18, y: 23 }, 8, PathDirection::East, id)
            .unwrap();
        park.place_queue_path(TilePos { x: 17, y: 23 }, 8, PathSlope::Flat, id)
            .unwrap();
        for y in 20..=23 {
            park.place_path(TilePos { x: 16, y }, 8).unwrap();
        }
        park.place_ride_exit(TilePos { x: 20, y: 21 }, 8, PathDirection::South, id)
            .unwrap();
        park.place_path(TilePos { x: 20, y: 20 }, 8).unwrap();
        assert_eq!(park.ride_access(id).status, RideAccessStatus::Ready);
        assert_eq!(
            park.ride_exit_access(id).status,
            RideExitStatus::PublicDisconnected
        );
        for _ in 0..6 {
            let report = unchanged_report(&park, id);
            if report.exit.status == RideExitStatus::Ready {
                break;
            }
            let public = report
                .suggestions
                .iter()
                .find(|s| s.kind == RideAccessBuildKind::PublicPath)
                .unwrap();
            park.place_sloped_path(public.pos, public.height, public.slope)
                .unwrap();
            assert_eq!(park.ride_access(id).status, RideAccessStatus::Ready);
        }
        let report = unchanged_report(&park, id);
        assert_eq!(report.exit.status, RideExitStatus::Ready);
        assert!(report.suggestions.is_empty());
    }

    #[test]
    fn unavailable_unknown_live_and_allocator_exhausted_reports_are_read_only() {
        let (mut park, carousel) = crate::carousel::tests::fixture(true);
        let coaster_id = park.ride().unwrap().id;
        for _ in 0..6000 {
            if park.guests.iter().any(|g| {
                g.ride_visit
                    .as_ref()
                    .is_some_and(|v| v.ride == carousel && v.passenger.is_some())
            }) {
                break;
            }
            park.advance(Speed::Normal);
        }
        let report = unchanged_report(&park, carousel);
        assert!(report.ride_open && report.passengers > 0);
        assert_eq!(report.ride_running, park.ride_is_running(carousel));
        assert_eq!(
            park.ride_access_plan(RideId(999)),
            Err(RideAccessError::UnknownRide(RideId(999)))
        );
        let (mut damaged, id) = coaster(0, 8);
        let track = damaged.circuit().unwrap().pieces[0].object;
        damaged.remove_object(track).unwrap();
        let report = unchanged_report(&damaged, id);
        assert!(!report.available);
        assert!(
            report.entrance_candidates.is_empty()
                && report.exit_candidates.is_empty()
                && report.suggestions.is_empty()
        );
        let (mut capped, id) = coaster(0, 8);
        capped.next_object_id = u64::MAX;
        let before = capped.clone();
        let report = capped.ride_access_plan(id).unwrap();
        assert!(report.entrance_candidates.is_empty() && report.suggestions.is_empty());
        assert_eq!(capped, before);
        assert_eq!(
            park.ride_access_plan(coaster_id).unwrap().entrance,
            park.ride_access(coaster_id)
        );
    }
}
