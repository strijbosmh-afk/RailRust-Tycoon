//! Deterministic path navigation and recovery for the small crowd proof.
//!
//! Paths connect through every cardinal edge at the same height. Axis controls
//! their decoration, not connectivity. Slopes, queues, admission, needs, and
//! ride boarding are later gameplay; these guests visit path nodes.

use std::collections::{BTreeMap, VecDeque};

use crate::{Axis, ObjectId, ObjectKind, Park, PlacementError, SurfaceObject, TilePos};

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
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CrowdStats {
    pub total: usize,
    pub walking: usize,
    pub waiting: usize,
    pub lost: usize,
    pub arrivals: u64,
}

impl Park {
    pub fn guests(&self) -> &[Guest] {
        &self.guests
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
        let from = guest
            .current
            .and_then(|id| self.object(id))
            .map_or(guest.anchor, |path| path.pos);
        let to = guest
            .next
            .and_then(|id| self.object(id))
            .map_or(from, |path| path.pos);
        Some(GuestPosition {
            from,
            to,
            height: guest.height,
            progress: guest.progress,
            steps: WALK_TICKS,
        })
    }

    /// Cardinal path neighbors in stable ID order. Track, a diagonal, and a
    /// vertically stacked path do not form a walkable connection.
    pub fn path_neighbors(&self, id: ObjectId) -> Vec<ObjectId> {
        let Some(path) = self.object(id).filter(|path| path.kind == ObjectKind::Path) else {
            return Vec::new();
        };
        let mut neighbors: Vec<_> = self
            .objects
            .iter()
            .filter(|other| paths_connect(path, other))
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
                    path.kind == ObjectKind::Path
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
    /// fresh IDs; a conflicting object rejects the complete restoration.
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
                    && path.kind == ObjectKind::Path
                    && path.height == FIXTURE_HEIGHT
                    && path.clearance == 2
            }) {
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
            }
            // A partially travelled edge returns to its retained start anchor.
            // Recovery can move at most to an incident, same-height path.
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
        // A deliberately small graph rebuilt from authoritative objects. It has
        // no stale connectivity cache after placement/deletion. Profile before
        // replacing this proof implementation with a large-park graph cache.
        let graph = PathGraph::new(&self.objects);
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

fn paths_connect(a: &SurfaceObject, b: &SurfaceObject) -> bool {
    a.kind == ObjectKind::Path
        && b.kind == ObjectKind::Path
        && a.height == b.height
        && a.pos.x.abs_diff(b.pos.x) + a.pos.y.abs_diff(b.pos.y) == 1
}

/// SplitMix64; the full wrapping integer state belongs to Park and is cloned.
fn next_random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

struct PathGraph {
    nodes: BTreeMap<ObjectId, SurfaceObject>,
    edges: BTreeMap<ObjectId, Vec<ObjectId>>,
}

impl PathGraph {
    fn new(objects: &[SurfaceObject]) -> Self {
        let nodes: BTreeMap<_, _> = objects
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
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

    fn step_guest(&self, guest: &mut Guest, rng: &mut u64) {
        if guest.current.is_none() {
            // The same tile wins over an adjacent tile. Never search farther,
            // move between heights, or jump to an unrelated component.
            let recovered = self
                .nodes
                .values()
                .filter_map(|path| {
                    let distance =
                        path.pos.x.abs_diff(guest.anchor.x) + path.pos.y.abs_diff(guest.anchor.y);
                    (path.height == guest.height && distance <= 1)
                        .then_some((distance, path.id, *path))
                })
                .min_by_key(|(distance, id, _)| (*distance, *id));
            let Some((_, id, path)) = recovered else {
                guest.state = GuestState::Lost;
                return;
            };
            guest.current = Some(id);
            guest.anchor = path.pos;
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
        let mut route = VecDeque::new();
        let mut cursor = destination;
        while cursor != current {
            route.push_front(cursor);
            cursor = parents[&cursor].expect("reachable non-start path has a parent");
        }
        guest.destination = Some(destination);
        guest.next = route.pop_front();
        guest.route = route;
        guest.progress = 0;
        guest.state = GuestState::Walking;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TerrainProfile};

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
}
