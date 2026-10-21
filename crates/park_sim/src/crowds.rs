//! Deterministic path navigation and recovery for the small crowd proof.
//!
//! Flat paths connect through four cardinal edges; one-step slopes connect
//! only through their opposite matching-height ends. Guests visit path nodes.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use crate::{
    Axis, ObjectId, ObjectKind, PATH_UNITS, Park, PathDirection, PathPoint, PathSlope,
    PlacementError, SurfaceObject, TilePos, paths_connect,
};

/// One tile of movement takes one second of authoritative simulation time.
pub const WALK_TICKS: u16 = 30;
const DWELL_TICKS: u16 = 30;
const FIXTURE_HEIGHT: u8 = 8;
const FIXTURE_GUESTS: usize = 12;

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct GuestId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GuestState {
    Walking,
    Waiting,
    Lost,
}

/// A guest's identities and integer movement state are authoritative.
///
/// `current` is the starting path during movement; `next` is its adjacent
/// destination for this edge. Lost guests retain a world anchor even when
/// their supporting path has been deleted. No removed object ID is retained.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guest {
    pub id: GuestId,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub current: Option<ObjectId>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub next: Option<ObjectId>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub destination: Option<ObjectId>,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub anchor: TilePos,
    pub height: u8,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub anchor_slope: PathSlope,
    pub progress: u16,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub state: GuestState,
    pub arrivals: u64,
    #[serde(deserialize_with = "crate::saves::bounded_route")]
    pub(crate) route: VecDeque<ObjectId>,
    pub(crate) wait_ticks: u16,
}

/// Presentation may interpolate the integer edge without advancing the sim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuestPosition {
    pub from: TilePos,
    pub to: TilePos,
    pub height: u8,
    pub progress: u16,
    pub steps: u16,
    pub point: PathPoint,
}

impl Guest {
    /// Sample the physical center→shared edge→center walk without any ID scan.
    /// Callers with an object index may use this directly; missing current
    /// objects retain the removed path's half-height center through anchor_slope.
    pub fn position(
        &self,
        current: Option<&SurfaceObject>,
        next: Option<&SurfaceObject>,
    ) -> GuestPosition {
        let anchor = self.anchor_path();
        let current = current
            .filter(|path| path.is_public_path())
            .unwrap_or(&anchor);
        let next = next.filter(|path| path.is_public_path() && paths_connect(current, path));
        let from = current.path_center().expect("current or anchor is a path");
        let to = next.and_then(SurfaceObject::path_center).unwrap_or(from);
        let point = if let Some(next) = next {
            let edge = PathDirection::ALL
                .into_iter()
                .find(|direction| direction.neighbor(current.pos) == Some(next.pos))
                .expect("connected cardinal paths");
            let middle = PathPoint {
                x: (from.x + to.x) / 2,
                y: (from.y + to.y) / 2,
                height: i32::from(current.path_edge_height(edge).expect("connected end"))
                    * PATH_UNITS,
            };
            let half = WALK_TICKS / 2;
            let progress = self.progress.min(WALK_TICKS);
            let (a, b, t) = if progress <= half {
                (from, middle, progress)
            } else {
                (middle, to, progress - half)
            };
            let mix = |a: i32, b: i32| a + (b - a) * i32::from(t) / i32::from(half);
            PathPoint {
                x: mix(a.x, b.x),
                y: mix(a.y, b.y),
                height: mix(a.height, b.height),
            }
        } else {
            from
        };
        GuestPosition {
            from: current.pos,
            to: next.map_or(current.pos, |path| path.pos),
            height: current.height,
            progress: self.progress,
            steps: WALK_TICKS,
            point,
        }
    }

    fn anchor_path(&self) -> SurfaceObject {
        SurfaceObject {
            id: ObjectId(0),
            pos: self.anchor,
            kind: ObjectKind::Path,
            axis: self.anchor_slope.axis(),
            path_slope: self.anchor_slope,
            queue_owner: None,
            height: self.height,
            clearance: self.anchor_slope.clearance(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CrowdStats {
    pub total: usize,
    pub walking: usize,
    pub waiting: usize,
    pub lost: usize,
    pub arrivals: u64,
}

/// Local diagnostic work counts, excluded from saves and Park equality.
/// `reuses` counts crowd ticks that reused a valid graph; read-only neighbor
/// queries do not change counters. `invalidations` counts discarded warm
/// graphs, so several path edits before the next tick discard only one graph.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize)]
pub struct PathCacheStats {
    pub builds: u64,
    pub reuses: u64,
    pub invalidations: u64,
    pub cached: bool,
}

/// Clones may share an immutable graph. Editing one Park drops only that
/// Park's reference; it cannot change another clone's topology or counters.
#[derive(Clone, Debug, Default)]
pub(crate) struct PathCache {
    graph: Option<Arc<PathGraph>>,
    stats: PathCacheStats,
}

impl PathCache {
    pub(crate) fn invalidate(&mut self) {
        if self.graph.take().is_some() {
            self.stats.invalidations = self.stats.invalidations.saturating_add(1);
        }
        self.stats.cached = false;
    }

    fn graph(&mut self, objects: &[SurfaceObject]) -> &PathGraph {
        if self.graph.is_none() {
            self.graph = Some(Arc::new(PathGraph::new(objects)));
            self.stats.builds = self.stats.builds.saturating_add(1);
            self.stats.cached = true;
        } else {
            self.stats.reuses = self.stats.reuses.saturating_add(1);
        }
        self.graph.as_deref().expect("graph built or retained")
    }
}

impl Park {
    pub fn guests(&self) -> &[Guest] {
        &self.guests
    }

    pub fn path_cache_stats(&self) -> PathCacheStats {
        self.path_cache.stats
    }

    pub fn crowd_stats(&self) -> CrowdStats {
        let mut stats = CrowdStats {
            total: self.guests.len(),
            ..CrowdStats::default()
        };
        for guest in &self.guests {
            match guest.state {
                GuestState::Walking => stats.walking += 1,
                GuestState::Waiting => stats.waiting += 1,
                GuestState::Lost => stats.lost += 1,
            }
            stats.arrivals = stats.arrivals.saturating_add(guest.arrivals);
        }
        stats
    }

    pub fn guest_position(&self, id: GuestId) -> Option<GuestPosition> {
        let guest = self.guests.iter().find(|guest| guest.id == id)?;
        Some(guest.position(
            guest.current.and_then(|id| self.object(id)),
            guest.next.and_then(|id| self.object(id)),
        ))
    }

    /// Cardinal path neighbors in stable ID order. Track, a diagonal, and a
    /// vertically stacked path do not form a walkable connection.
    pub fn path_neighbors(&self, id: ObjectId) -> Vec<ObjectId> {
        if let Some(graph) = &self.path_cache.graph {
            return graph.edges.get(&id).cloned().unwrap_or_default();
        }
        let Some(path) = self.object(id).filter(|path| path.is_public_path()) else {
            return Vec::new();
        };
        let mut neighbors: Vec<_> = self
            .objects
            .iter()
            .filter(|other| other.is_public_path() && paths_connect(path, other))
            .map(|other| other.id)
            .collect();
        neighbors.sort_unstable();
        neighbors
    }

    /// Add a loop, a four-way junction, a branch, and twelve seeded guests.
    ///
    /// Installation is atomic and idempotent. Calling it again restores missing
    /// fixture paths but preserves guests, their random state, identities and
    /// time, even when the caller supplies a different seed.
    pub fn install_crowd_fixture(&mut self, seed: u64) -> Result<(), PlacementError> {
        let mut candidate = self.clone();
        candidate.install_overlap_fixture()?;
        candidate.restore_crowd_paths()?;
        if candidate.crowd_seed.is_none() {
            candidate.crowd_seed = Some(seed);
            candidate.crowd_rng = seed;
            let fixture_positions = fixture_paths();
            let paths: Vec<_> = candidate
                .objects
                .iter()
                .filter(|path| {
                    path.is_public_path()
                        && path.height == FIXTURE_HEIGHT
                        && fixture_positions.iter().any(|(pos, _)| *pos == path.pos)
                })
                .copied()
                .collect();
            for index in 0..FIXTURE_GUESTS {
                let next_id = candidate
                    .next_guest_id
                    .checked_add(1)
                    .ok_or(PlacementError::IdExhausted)?;
                let path =
                    paths[(next_random(&mut candidate.crowd_rng) % paths.len() as u64) as usize];
                candidate.guests.push(Guest {
                    id: GuestId(candidate.next_guest_id),
                    current: Some(path.id),
                    next: None,
                    destination: None,
                    anchor: path.pos,
                    height: path.height,
                    anchor_slope: path.path_slope,
                    progress: 0,
                    state: GuestState::Waiting,
                    arrivals: 0,
                    route: VecDeque::new(),
                    wait_ticks: index as u16 * 2,
                });
                candidate.next_guest_id = next_id;
            }
        }
        *self = candidate;
        Ok(())
    }

    /// Restore only missing proof paths as one atomic edit. Existing objects,
    /// guests, clock, seed state and identities are retained. New objects receive
    /// fresh IDs; a collision or player construction limit rejects the complete
    /// restoration. Generic object placement remains available to diagnostics.
    pub fn restore_crowd_paths(&mut self) -> Result<(), PlacementError> {
        if (self.width(), self.height()) != (32, 32) {
            return Err(PlacementError::FixtureMapSize {
                width: self.width(),
                height: self.height(),
            });
        }
        let mut candidate = self.clone();
        for (pos, axis) in fixture_paths() {
            if !candidate.objects.iter().any(|path| {
                path.pos == pos
                    && path.is_public_path()
                    && path.height == FIXTURE_HEIGHT
                    && path.clearance == 2
            }) {
                candidate.validate_path(pos, FIXTURE_HEIGHT)?;
                candidate.place_object(pos, ObjectKind::Path, axis, FIXTURE_HEIGHT, 2)?;
            }
        }
        *self = candidate;
        Ok(())
    }

    pub(crate) fn reconcile_guests_after_removal(&mut self, removed: SurfaceObject) {
        for guest in &mut self.guests {
            let affected = guest.current == Some(removed.id)
                || guest.next == Some(removed.id)
                || guest.destination == Some(removed.id)
                || guest.route.contains(&removed.id);
            if !affected {
                continue;
            }
            if guest.current == Some(removed.id) {
                guest.current = None;
                guest.anchor = removed.pos;
                guest.height = removed.height;
                guest.anchor_slope = removed.path_slope;
            }
            // A partially travelled edge returns to its retained start anchor.
            // Recovery can move at most to a path incident at a matching end.
            guest.next = None;
            guest.destination = None;
            guest.route.clear();
            guest.progress = 0;
            guest.wait_ticks = 0;
            guest.state = GuestState::Lost;
        }
    }

    pub(crate) fn step_crowd(&mut self) {
        if self.guests.is_empty() {
            return;
        }
        // Object edits invalidate derived connectivity before another tick or
        // query can use it. Stable topology needs no allocation or graph rebuild.
        let graph = self.path_cache.graph(&self.objects);
        for guest in &mut self.guests {
            graph.step_guest(guest, &mut self.crowd_rng);
        }
    }
}

fn fixture_paths() -> Vec<(TilePos, Axis)> {
    let mut paths: Vec<_> = (8..=19).map(|x| (TilePos { x, y: 16 }, Axis::X)).collect();
    for y in 17..=20 {
        paths.push((TilePos { x: 8, y }, Axis::Y));
        paths.push((TilePos { x: 19, y }, Axis::Y));
    }
    paths.extend((9..=18).map(|x| (TilePos { x, y: 20 }, Axis::X)));
    paths.extend((12..=15).map(|y| (TilePos { x: 13, y }, Axis::Y)));
    paths.extend((17..=19).map(|y| (TilePos { x: 13, y }, Axis::Y)));
    paths
}

/// SplitMix64; the full wrapping integer state belongs to Park and is cloned.
pub(crate) fn next_random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PathGraph {
    pub(crate) nodes: BTreeMap<ObjectId, SurfaceObject>,
    pub(crate) edges: BTreeMap<ObjectId, Vec<ObjectId>>,
}

impl PathGraph {
    pub(crate) fn new(objects: &[SurfaceObject]) -> Self {
        let nodes: BTreeMap<_, _> = objects
            .iter()
            .filter(|object| object.is_public_path())
            .map(|object| (object.id, *object))
            .collect();
        // Placement/save validation prevents duplicate walking ends at the
        // same tile and height. Indexing ends avoids comparing every path pair.
        let by_edge: BTreeMap<_, _> = nodes
            .values()
            .flat_map(|path| {
                PathDirection::ALL.into_iter().filter_map(|edge| {
                    path.path_edge_height(edge)
                        .map(|height| ((path.pos.x, path.pos.y, edge, height), path.id))
                })
            })
            .collect();
        let edges = nodes
            .values()
            .map(|path| {
                let mut neighbors: Vec<_> = PathDirection::ALL
                    .into_iter()
                    .filter_map(|edge| {
                        let height = path.path_edge_height(edge)?;
                        let pos = edge.neighbor(path.pos)?;
                        by_edge
                            .get(&(pos.x, pos.y, edge.opposite(), height))
                            .copied()
                    })
                    .collect();
                // The previous all-pairs scan used ascending ObjectId order.
                // Preserve that order, not cardinal/spatial traversal order,
                // to retain BFS tie-breaking and all seeded guest decisions.
                neighbors.sort_unstable();
                (path.id, neighbors)
            })
            .collect();
        Self { nodes, edges }
    }

    /// Stable breadth-first traversal: shortest routes, deterministic ties.
    fn parents_from(&self, start: ObjectId) -> BTreeMap<ObjectId, Option<ObjectId>> {
        let mut parents = BTreeMap::from([(start, None)]);
        let mut queue = VecDeque::from([start]);
        while let Some(current) = queue.pop_front() {
            for &neighbor in &self.edges[&current] {
                if let std::collections::btree_map::Entry::Vacant(entry) = parents.entry(neighbor) {
                    entry.insert(Some(current));
                    queue.push_back(neighbor);
                }
            }
        }
        parents
    }

    /// Diagnostic queries use the same complete traversal and reconstruction
    /// as a guest choosing a destination. They never advance a guest or RNG.
    pub(crate) fn route(
        &self,
        start: ObjectId,
        destination: ObjectId,
    ) -> Option<VecDeque<ObjectId>> {
        if !self.nodes.contains_key(&start) || !self.nodes.contains_key(&destination) {
            return None;
        }
        let parents = self.parents_from(start);
        parents
            .contains_key(&destination)
            .then(|| route_from_parents(&parents, start, destination))
    }

    fn step_guest(&self, guest: &mut Guest, rng: &mut u64) {
        if guest.current.is_none() {
            // The exact removed footprint wins over an incident path. A slope
            // retains both end heights: side neighbors and wrong levels cannot
            // become recovery targets, even when their centers are nearby.
            let anchor = guest.anchor_path();
            let recovered = self
                .nodes
                .values()
                .filter_map(|path| {
                    let distance =
                        path.pos.x.abs_diff(guest.anchor.x) + path.pos.y.abs_diff(guest.anchor.y);
                    ((distance == 0
                        && path.height == guest.height
                        && path.path_slope == guest.anchor_slope)
                        || paths_connect(&anchor, path))
                    .then_some((distance, path.id, *path))
                })
                .min_by_key(|(distance, id, _)| (*distance, *id));
            let Some((_, id, path)) = recovered else {
                guest.state = GuestState::Lost;
                return;
            };
            guest.current = Some(id);
            guest.anchor = path.pos;
            guest.height = path.height;
            guest.anchor_slope = path.path_slope;
        }
        let current = guest.current.expect("a retained or recovered current path");
        if guest.wait_ticks > 0 {
            guest.wait_ticks -= 1;
            guest.state = GuestState::Waiting;
            return;
        }
        if let Some(next) = guest.next {
            guest.progress += 1;
            if guest.progress >= WALK_TICKS {
                let path = self.nodes[&next];
                guest.current = Some(next);
                guest.anchor = path.pos;
                guest.height = path.height;
                guest.anchor_slope = path.path_slope;
                guest.progress = 0;
                guest.next = None;
                if guest.destination == Some(next) {
                    guest.arrivals = guest.arrivals.saturating_add(1);
                    guest.destination = None;
                    guest.route.clear();
                    guest.state = GuestState::Waiting;
                    guest.wait_ticks = DWELL_TICKS;
                } else {
                    guest.next = guest.route.pop_front();
                    guest.state = GuestState::Walking;
                }
            }
            return;
        }
        let parents = self.parents_from(current);
        let candidates: Vec<_> = parents
            .keys()
            .copied()
            .filter(|id| *id != current)
            .collect();
        if candidates.is_empty() {
            guest.state = GuestState::Lost;
            guest.destination = None;
            guest.route.clear();
            return;
        }
        let destination = candidates[(next_random(rng) % candidates.len() as u64) as usize];
        let mut route = route_from_parents(&parents, current, destination);
        guest.destination = Some(destination);
        guest.next = route.pop_front();
        guest.route = route;
        guest.progress = 0;
        guest.state = GuestState::Walking;
    }
}

fn route_from_parents(
    parents: &BTreeMap<ObjectId, Option<ObjectId>>,
    start: ObjectId,
    destination: ObjectId,
) -> VecDeque<ObjectId> {
    let mut route = VecDeque::new();
    let mut cursor = destination;
    while cursor != start {
        route.push_front(cursor);
        cursor = parents[&cursor].expect("reachable non-start path has a parent");
    }
    route
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TerrainProfile};

    /// Independent pre-cache graph construction retained as a regression
    /// oracle. Its all-pairs scan must agree with indexed topology and ID order.
    fn all_pairs_graph(objects: &[SurfaceObject]) -> PathGraph {
        let nodes: BTreeMap<_, _> = objects
            .iter()
            .filter(|object| object.is_public_path())
            .map(|object| (object.id, *object))
            .collect();
        let edges = nodes
            .values()
            .map(|path| {
                let neighbors = nodes
                    .values()
                    .filter(|other| paths_connect(path, other))
                    .map(|other| other.id)
                    .collect();
                (path.id, neighbors)
            })
            .collect();
        PathGraph { nodes, edges }
    }

    fn advance_forcing_fresh_graph(park: &mut Park, speed: Speed) {
        for _ in 0..speed.multiplier() {
            park.path_cache.invalidate();
            park.advance(Speed::Normal);
        }
    }

    fn fixture(seed: u64) -> Park {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(seed).unwrap();
        park
    }

    fn advance(park: &mut Park, ticks: u32) {
        for _ in 0..ticks {
            park.advance(Speed::Normal);
        }
    }

    fn add_path(park: &mut Park, x: u32, y: u32, height: u8) -> ObjectId {
        park.place_object(TilePos { x, y }, ObjectKind::Path, Axis::X, height, 2)
            .unwrap()
    }

    fn add_guest(park: &mut Park, current: ObjectId) {
        let path = *park.object(current).unwrap();
        park.guests.push(Guest {
            id: GuestId(1),
            current: Some(current),
            next: None,
            destination: None,
            anchor: path.pos,
            height: path.height,
            anchor_slope: path.path_slope,
            progress: 0,
            state: GuestState::Waiting,
            arrivals: 0,
            route: VecDeque::new(),
            wait_ticks: 0,
        });
    }

    fn assert_valid_references(park: &Park) {
        for guest in park.guests() {
            for id in guest
                .current
                .into_iter()
                .chain(guest.next)
                .chain(guest.destination)
                .chain(guest.route.iter().copied())
            {
                let path = park.object(id).expect("guest retained a removed ID");
                assert_eq!(path.kind, ObjectKind::Path);
                assert_eq!(path.height, guest.height);
            }
            if let (Some(from), Some(to)) = (guest.current, guest.next) {
                assert!(park.path_neighbors(from).contains(&to));
            }
            assert!(guest.progress < WALK_TICKS);
        }
    }

    #[test]
    fn connectivity_handles_junctions_heights_tracks_diagonals_and_missing_ids() {
        let mut park = Park::new(8, 8);
        let center = add_path(&mut park, 3, 3, 2);
        let mut neighbors = vec![
            add_path(&mut park, 2, 3, 2),
            add_path(&mut park, 4, 3, 2),
            add_path(&mut park, 3, 2, 2),
            add_path(&mut park, 3, 4, 2),
        ];
        neighbors.sort_unstable();
        add_path(&mut park, 2, 2, 2);
        let upper = add_path(&mut park, 3, 3, 4);
        let track = park
            .place_object(TilePos { x: 3, y: 2 }, ObjectKind::Track, Axis::Y, 4, 2)
            .unwrap();
        assert_eq!(park.path_neighbors(center), neighbors);
        assert!(park.path_neighbors(upper).is_empty());
        assert!(park.path_neighbors(track).is_empty());
        assert!(park.path_neighbors(ObjectId(u64::MAX)).is_empty());
        let edge = add_path(&mut park, 0, 0, 0);
        assert!(park.path_neighbors(edge).is_empty());
    }

    #[test]
    fn seed_replays_full_state_and_distinct_seeds_change_the_crowd() {
        let mut a = fixture(42);
        let mut b = fixture(42);
        let other = fixture(43);
        assert_eq!(a, b);
        assert_ne!(a.guests(), other.guests());
        for _ in 0..2400 {
            a.advance(Speed::Normal);
            b.advance(Speed::Normal);
        }
        assert_eq!(a, b);
        assert_eq!(a.crowd_stats().total, 12);
        assert!(a.crowd_stats().arrivals > 12);
        assert_eq!(a.crowd_stats().lost, 0);
        assert_valid_references(&a);
    }

    #[test]
    fn speed_grouping_executes_equivalent_complete_ticks_and_pause_is_inert() {
        let initial = fixture(7);
        let mut normal = initial.clone();
        let mut fast = initial.clone();
        let mut fastest = initial;
        advance(&mut normal, 1200);
        for _ in 0..600 {
            fast.advance(Speed::Fast);
        }
        for _ in 0..300 {
            fastest.advance(Speed::Fastest);
        }
        assert_eq!(normal, fast);
        assert_eq!(normal, fastest);
        let snapshot = fastest.clone();
        for _ in 0..100 {
            fastest.advance(Speed::Paused);
        }
        assert_eq!(fastest, snapshot);
        fastest.ticks = u64::MAX;
        let snapshot = fastest.clone();
        fastest.advance(Speed::Fastest);
        assert_eq!(fastest, snapshot);
    }

    #[test]
    fn movement_has_integer_interpolation_and_reaches_a_real_destination() {
        let mut park = Park::new(4, 4);
        let start = add_path(&mut park, 1, 1, 0);
        let end = add_path(&mut park, 2, 1, 0);
        add_guest(&mut park, start);
        park.advance(Speed::Normal);
        assert_eq!(park.guests()[0].destination, Some(end));
        advance(&mut park, 15);
        let position = park.guest_position(GuestId(1)).unwrap();
        assert_eq!(position.from, TilePos { x: 1, y: 1 });
        assert_eq!(position.to, TilePos { x: 2, y: 1 });
        assert_eq!((position.progress, position.steps), (15, 30));
        advance(&mut park, 15);
        let guest = &park.guests()[0];
        assert_eq!(guest.current, Some(end));
        assert_eq!(guest.state, GuestState::Waiting);
        assert_eq!(guest.arrivals, 1);
        assert!(guest.next.is_none() && guest.destination.is_none());
        assert!(park.guest_position(GuestId(100)).is_none());
    }

    #[test]
    fn deleting_occupied_path_preserves_guest_and_recovers_only_to_incident_path() {
        let mut park = Park::new(8, 8);
        let start = add_path(&mut park, 1, 1, 0);
        let end = add_path(&mut park, 2, 1, 0);
        let third = add_path(&mut park, 3, 1, 0);
        add_guest(&mut park, start);
        advance(&mut park, 8);
        park.remove_object(start).unwrap();
        let guest = &park.guests()[0];
        assert_eq!(guest.id, GuestId(1));
        assert_eq!(guest.state, GuestState::Lost);
        assert_eq!(guest.current, None);
        assert_eq!(guest.anchor, TilePos { x: 1, y: 1 });
        assert_valid_references(&park);
        park.advance(Speed::Normal);
        assert_eq!(park.guests()[0].current, Some(end));
        assert_eq!(park.guests()[0].next, Some(third));
        assert_eq!(park.guests()[0].state, GuestState::Walking);
    }

    #[test]
    fn deletion_of_next_destination_or_later_route_clears_stale_references() {
        for target in ["next", "destination", "route"] {
            let mut park = Park::new(8, 8);
            let ids: Vec<_> = (1..=5).map(|x| add_path(&mut park, x, 1, 0)).collect();
            add_guest(&mut park, ids[0]);
            let guest = &mut park.guests[0];
            guest.next = Some(ids[1]);
            guest.destination = Some(ids[4]);
            guest.route = VecDeque::from([ids[2], ids[3], ids[4]]);
            guest.state = GuestState::Walking;
            guest.progress = 12;
            let removed = match target {
                "next" => ids[1],
                "destination" => ids[4],
                _ => ids[2],
            };
            park.remove_object(removed).unwrap();
            let guest = &park.guests()[0];
            assert_eq!(guest.current, Some(ids[0]));
            assert_eq!(guest.state, GuestState::Lost);
            assert!(guest.next.is_none() && guest.destination.is_none() && guest.route.is_empty());
            assert_eq!(guest.progress, 0);
            assert_valid_references(&park);
            advance(&mut park, 90);
            assert_valid_references(&park);
        }
    }

    #[test]
    fn isolated_guest_waits_and_recovers_after_new_id_reconnects_its_component() {
        let mut park = Park::new(8, 8);
        let start = add_path(&mut park, 1, 1, 0);
        let bridge = add_path(&mut park, 2, 1, 0);
        let end = add_path(&mut park, 3, 1, 0);
        add_guest(&mut park, start);
        park.remove_object(bridge);
        advance(&mut park, 120);
        assert_eq!(park.guests()[0].current, Some(start));
        assert_eq!(park.guests()[0].state, GuestState::Lost);
        assert_eq!(park.guests()[0].arrivals, 0);
        let replacement = add_path(&mut park, 2, 1, 0);
        assert!(replacement.0 > end.0);
        advance(&mut park, 120);
        assert!(park.guests()[0].arrivals > 0);
        assert_ne!(park.guests()[0].state, GuestState::Lost);
        assert_valid_references(&park);
    }

    #[test]
    fn fully_stranded_guest_never_teleports_to_distant_or_stacked_paths() {
        let mut park = Park::new(8, 8);
        let start = add_path(&mut park, 1, 1, 0);
        add_guest(&mut park, start);
        park.remove_object(start);
        add_path(&mut park, 5, 5, 0);
        add_path(&mut park, 1, 1, 2);
        advance(&mut park, 120);
        assert_eq!(park.guests()[0].current, None);
        assert_eq!(park.guests()[0].state, GuestState::Lost);
        assert_eq!(park.guests()[0].anchor, TilePos { x: 1, y: 1 });
        assert_eq!(park.crowd_stats().total, 1);
        let replacement = add_path(&mut park, 1, 1, 0);
        add_path(&mut park, 2, 1, 0);
        park.advance(Speed::Normal);
        assert_eq!(park.guests()[0].current, Some(replacement));
        assert_eq!(park.guests()[0].state, GuestState::Walking);
    }

    #[test]
    fn fixture_restore_preserves_time_guests_seed_and_surviving_ids() {
        let mut park = fixture(0);
        assert_eq!(park.objects().len(), 46);
        assert_eq!(park.guests().len(), 12);
        let snapshot = park.clone();
        park.install_crowd_fixture(999).unwrap();
        assert_eq!(park, snapshot);
        advance(&mut park, 300);
        let removed = park.guests()[0].current.unwrap();
        park.remove_object(removed);
        let before_restore = park.clone();
        park.restore_crowd_paths().unwrap();
        assert_eq!(park.guests(), before_restore.guests());
        assert_eq!(park.ticks(), before_restore.ticks());
        assert_eq!(park.crowd_rng, before_restore.crowd_rng);
        for path in before_restore.objects() {
            assert_eq!(park.object(path.id), Some(path));
        }
        assert!(park.object(removed).is_none());
        assert_eq!(park.objects().len(), 46);
        let snapshot = park.clone();
        park.restore_crowd_paths().unwrap();
        assert_eq!(park, snapshot);
        advance(&mut park, 300);
        assert_eq!(park.crowd_stats().lost, 0);
        assert_valid_references(&park);
        let guests = park.guests().to_vec();
        park.try_apply_terrain_profile(TerrainProfile::Flat)
            .unwrap();
        park.try_apply_terrain_profile(TerrainProfile::Terraced)
            .unwrap();
        assert_eq!(park.guests(), guests);
    }

    #[test]
    fn rejected_crowd_install_and_restore_are_atomic_including_rng_and_ids() {
        let mut park = Park::new(32, 32);
        park.place_object(TilePos { x: 13, y: 19 }, ObjectKind::Track, Axis::Y, 8, 2)
            .unwrap();
        let snapshot = park.clone();
        assert!(park.install_crowd_fixture(123).is_err());
        assert_eq!(park, snapshot);
        let mut park = fixture(123);
        let path = park
            .objects()
            .iter()
            .find(|path| path.kind == ObjectKind::Path && path.pos == TilePos { x: 13, y: 19 })
            .copied()
            .unwrap();
        park.remove_object(path.id);
        park.place_object(path.pos, ObjectKind::Track, Axis::Y, 8, 2)
            .unwrap();
        let snapshot = park.clone();
        assert!(park.restore_crowd_paths().is_err());
        assert_eq!(park, snapshot);
        let mut small = Park::new(4, 4);
        let snapshot = small.clone();
        assert!(small.install_crowd_fixture(1).is_err());
        assert_eq!(small, snapshot);
    }

    #[test]
    fn fixture_restore_at_path_capacity_rolls_back_even_a_partial_repair() {
        let mut park = fixture(123);
        let removed: Vec<_> = park
            .objects()
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .take(2)
            .map(|object| object.id)
            .collect();
        for id in &removed {
            park.remove_object(*id).unwrap();
        }
        let paths = park
            .objects()
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .count();
        let mut added = Vec::new();
        for index in 0..crate::saves::MAX_PATHS - paths {
            added.push(
                park.place_path(
                    TilePos {
                        x: (index % 32) as u32,
                        y: (index / 32) as u32,
                    },
                    10,
                )
                .unwrap(),
            );
        }
        for remaining_capacity in [0, 1] {
            if remaining_capacity == 1 {
                park.remove_object(added.pop().unwrap()).unwrap();
            }
            park.advance(Speed::Normal);
            let snapshot = park.clone();
            let bytes = park.to_save_bytes().unwrap();
            let cache = park.path_cache_stats();
            assert!(cache.cached);
            assert_eq!(
                park.restore_crowd_paths(),
                Err(PlacementError::CapacityExceeded {
                    resource: "paths",
                    max: crate::saves::MAX_PATHS,
                })
            );
            assert_eq!(park, snapshot);
            assert_eq!(park.path_cache_stats(), cache);
            assert_eq!(park.to_save_bytes().unwrap(), bytes);
            assert!(removed.iter().all(|id| park.object(*id).is_none()));
        }
    }

    #[test]
    fn guest_destinations_stay_inside_their_reachable_component() {
        let mut park = Park::new(8, 8);
        let start = add_path(&mut park, 1, 1, 0);
        let local = add_path(&mut park, 2, 1, 0);
        let distant = add_path(&mut park, 6, 6, 0);
        add_path(&mut park, 6, 7, 0);
        add_guest(&mut park, start);
        for _ in 0..600 {
            park.advance(Speed::Normal);
            let guest = &park.guests()[0];
            assert!(guest.current == Some(start) || guest.current == Some(local));
            assert_ne!(guest.destination, Some(distant));
            assert_valid_references(&park);
        }
        assert!(park.guests()[0].arrivals > 0);
    }

    #[test]
    fn indexed_graph_preserves_all_pairs_topology_and_stable_tie_order() {
        use crate::diagnostics::{DiagnosticCase, build_fixture};
        for case in [DiagnosticCase::Baseline, DiagnosticCase::SaveScale] {
            let mut park = build_fixture(case, 17).unwrap().park;
            // A loaded object vector need not be sorted by stable identity.
            park.objects.reverse();
            let indexed = PathGraph::new(&park.objects);
            let reference = all_pairs_graph(&park.objects);
            assert_eq!(indexed, reference);
            let ids: Vec<_> = indexed.nodes.keys().copied().collect();
            for index in 0..12 {
                let from = ids[index * 13 % ids.len()];
                let to = ids[(index * 37 + 1) % ids.len()];
                assert_eq!(indexed.route(from, to), reference.route(from, to));
            }
        }
        let mut park = Park::new(6, 6);
        for (x, y, height) in [
            (0, 0, 0),
            (1, 0, 0),
            (0, 1, 0),
            (0, 0, 2),
            (0, 1, 2),
            (1, 1, 2),
            (2, 2, 2),
            (5, 5, 0),
        ] {
            add_path(&mut park, x, y, height);
        }
        park.place_object(TilePos { x: 2, y: 0 }, ObjectKind::Track, Axis::Y, 0, 2)
            .unwrap();
        assert_eq!(
            PathGraph::new(&park.objects),
            all_pairs_graph(&park.objects)
        );
        assert_eq!(PathGraph::new(&[]), all_pairs_graph(&[]));
    }

    #[test]
    fn unchanged_ticks_reuse_graph_and_only_path_edits_invalidate_it() {
        let mut park = fixture(91);
        assert_eq!(park.path_cache_stats(), PathCacheStats::default());
        park.advance(Speed::Paused);
        assert_eq!(park.path_cache_stats(), PathCacheStats::default());
        park.advance(Speed::Normal);
        park.advance(Speed::Fastest);
        assert_eq!(
            park.path_cache_stats(),
            PathCacheStats {
                builds: 1,
                reuses: 4,
                invalidations: 0,
                cached: true,
            }
        );
        let before = park.path_cache_stats();
        let occupied = *park
            .objects()
            .iter()
            .find(|object| object.kind == ObjectKind::Path)
            .unwrap();
        assert!(
            park.place_object(
                occupied.pos,
                occupied.kind,
                occupied.axis,
                occupied.height,
                occupied.clearance
            )
            .is_err()
        );
        assert_eq!(park.remove_object(ObjectId(u64::MAX)), None);
        park.restore_crowd_paths().unwrap();
        park.try_apply_terrain_profile(TerrainProfile::Flat)
            .unwrap();
        let track = park
            .place_object(TilePos { x: 1, y: 1 }, ObjectKind::Track, Axis::X, 0, 2)
            .unwrap();
        park.remove_object(track).unwrap();
        park.install_train_fixture().unwrap();
        park.advance(Speed::Paused);
        assert_eq!(park.path_cache_stats(), before);

        let neighbor = add_path(&mut park, 7, 16, 8);
        assert!(!park.path_cache_stats().cached);
        assert!(park.path_neighbors(occupied.id).contains(&neighbor));
        assert_eq!(park.path_cache_stats().invalidations, 1);
        park.advance(Speed::Normal);
        assert_eq!(park.path_cache_stats().builds, 2);
        assert!(park.path_cache_stats().cached);
        park.remove_object(neighbor).unwrap();
        // Queries are correct immediately, without a clock tick rebuilding.
        assert!(!park.path_neighbors(occupied.id).contains(&neighbor));
        assert!(park.path_neighbors(neighbor).is_empty());
        let before_pause = park.path_cache_stats();
        park.advance(Speed::Paused);
        assert_eq!(park.path_cache_stats(), before_pause);
        park.advance(Speed::Fast);
        assert_eq!(park.path_cache_stats().builds, 3);
        assert_eq!(park.path_cache_stats().reuses, 5);
        assert_eq!(park.path_cache_stats().invalidations, 2);
    }

    #[test]
    fn shared_clone_cache_and_failed_atomic_restoration_remain_independent() {
        let mut park = fixture(4);
        park.advance(Speed::Normal);
        let unchanged = park.clone();
        assert!(Arc::ptr_eq(
            park.path_cache.graph.as_ref().unwrap(),
            unchanged.path_cache.graph.as_ref().unwrap()
        ));
        let early = park
            .objects()
            .iter()
            .find(|path| path.kind == ObjectKind::Path && path.pos == TilePos { x: 8, y: 16 })
            .copied()
            .unwrap();
        let late = park
            .objects()
            .iter()
            .find(|path| path.kind == ObjectKind::Path && path.pos == TilePos { x: 13, y: 19 })
            .copied()
            .unwrap();
        park.remove_object(early.id).unwrap();
        park.remove_object(late.id).unwrap();
        assert!(unchanged.path_cache_stats().cached);
        assert!(!unchanged.path_neighbors(early.id).is_empty());
        assert!(park.path_neighbors(early.id).is_empty());
        park.place_object(late.pos, ObjectKind::Track, Axis::Y, 8, 2)
            .unwrap();
        park.advance(Speed::Normal);
        let before = park.clone();
        let before_stats = park.path_cache_stats();
        // The candidate can restore the early path before the later collision.
        assert!(park.restore_crowd_paths().is_err());
        assert_eq!(park, before);
        assert_eq!(park.path_cache_stats(), before_stats);
        assert!(Arc::ptr_eq(
            park.path_cache.graph.as_ref().unwrap(),
            before.path_cache.graph.as_ref().unwrap()
        ));
        assert_eq!(unchanged.path_cache_stats().builds, 1);
        assert_eq!(unchanged.path_cache_stats().invalidations, 0);
    }

    #[test]
    fn cached_and_forced_fresh_continuations_match_through_edits_load_and_speeds() {
        let mut cached = fixture(12345);
        cached.install_train_fixture().unwrap();
        let mut fresh = cached.clone();
        for frame in 0..900 {
            match frame {
                80 => {
                    let current = cached.guests()[0].current.unwrap();
                    let mut removed = cached.path_neighbors(current);
                    removed.push(current);
                    for id in removed {
                        cached.remove_object(id).unwrap();
                        fresh.remove_object(id).unwrap();
                    }
                    assert!(cached.crowd_stats().lost > 0);
                }
                140 | 330 => {
                    cached.restore_crowd_paths().unwrap();
                    fresh.restore_crowd_paths().unwrap();
                }
                230 => {
                    let destination = cached
                        .guests()
                        .iter()
                        .find_map(|guest| guest.destination)
                        .expect("seeded crowd has a destination");
                    cached.remove_object(destination).unwrap();
                    fresh.remove_object(destination).unwrap();
                }
                400 => {
                    let bytes = cached.to_save_bytes().unwrap();
                    cached = Park::from_save_bytes(&bytes).unwrap();
                    assert_eq!(cached.path_cache_stats(), PathCacheStats::default());
                    assert_eq!(cached, fresh);
                    assert_eq!(cached.to_save_bytes().unwrap(), bytes);
                }
                600 => {
                    cached
                        .try_apply_terrain_profile(TerrainProfile::Flat)
                        .unwrap();
                    fresh
                        .try_apply_terrain_profile(TerrainProfile::Flat)
                        .unwrap();
                }
                700 => {
                    cached.install_crowd_fixture(999).unwrap();
                    fresh.install_crowd_fixture(999).unwrap();
                }
                _ => {}
            }
            let speed = [Speed::Normal, Speed::Fast, Speed::Fastest, Speed::Paused][frame % 4];
            cached.advance(speed);
            advance_forcing_fresh_graph(&mut fresh, speed);
            assert_eq!(
                cached, fresh,
                "different authoritative state at frame {frame}"
            );
        }
        assert!(cached.path_cache_stats().reuses > 500);
        assert!(fresh.path_cache_stats().builds > 1_000);
        assert_valid_references(&cached);
    }

    #[test]
    fn cache_warmth_never_changes_save_bytes_or_reordered_load_continuation() {
        let mut park = fixture(1);
        park.install_train_fixture().unwrap();
        park.objects.reverse();
        park.guests.reverse();
        let bytes = park.to_save_bytes().unwrap();
        let cold = Park::from_save_bytes(&bytes).unwrap();
        assert_eq!(cold, park);
        assert_eq!(cold.objects(), park.objects());
        assert_eq!(cold.guests(), park.guests());
        park.path_cache.graph(&park.objects);
        assert!(park.path_cache_stats().cached);
        assert_eq!(park, cold);
        assert_eq!(park.to_save_bytes().unwrap(), bytes);
        let mut loaded = Park::from_save_bytes(&bytes).unwrap();
        assert!(!loaded.path_cache_stats().cached);
        for _ in 0..600 {
            park.advance(Speed::Fast);
            advance_forcing_fresh_graph(&mut loaded, Speed::Fast);
            assert_eq!(park, loaded);
        }
        assert_eq!(
            park.to_save_bytes().unwrap(),
            loaded.to_save_bytes().unwrap()
        );
    }
}
