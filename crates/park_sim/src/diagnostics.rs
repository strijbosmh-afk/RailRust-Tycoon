//! Opt-in, deterministic measurement inputs and production navigation probes.
//!
//! Fixture creation is setup work, not a simulation measurement. Callers own
//! warmup, timers, iteration counts, and reporting. No diagnostic code runs from
//! `Park::advance`, and no simulation clock or random state changes during a
//! graph probe. These inputs exercise the current single-circuit model: they
//! cannot establish performance for a future park with fifty working rides.

use std::collections::VecDeque;

use crate::crowds::{PathGraph, next_random};
use crate::{
    Axis, Guest, GuestId, GuestState, ObjectId, ObjectKind, Park, TerrainProfile, TilePos,
    TrainError, WALK_TICKS,
};

const GRID_ORIGIN: u32 = 80;
const GRID_SIDE: u32 = 32;
const PATH_HEIGHT: u8 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticCase {
    Baseline,
    SaveScale,
    Stress,
}

impl DiagnosticCase {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::SaveScale => "save-scale",
            Self::Stress => "stress",
        }
    }
}

/// Actual fixture counts, independent of any aspirational performance target.
/// `save_supported` describes the unchanged codec limits for this fixture;
/// the stress case deliberately exceeds its maximum of 256 guests.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct FixtureMetadata {
    pub case: DiagnosticCase,
    pub seed: u64,
    pub map_width: u32,
    pub map_height: u32,
    pub tiles: usize,
    pub guests: usize,
    pub paths: usize,
    pub objects: usize,
    pub rides: usize,
    pub trains: usize,
    pub track_pieces: usize,
    pub save_supported: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticFixture {
    pub park: Park,
    pub metadata: FixtureMetadata,
}

/// Build a fresh input at tick zero with stable IDs and a stored seeded RNG.
///
/// Baseline is the normal 32 by 32 demonstration park. The larger cases use a
/// 256 by 256 terraced map and a connected 32 by 32 square of paths at tiles
/// (80,80) through (111,111), above the terrain. Every path edge is a production
/// cardinal connection. Their one working circuit is the normal 46-piece ride,
/// installed through the production circuit validator, away from the paths.
/// SaveScale has 256 guests; Stress has 5,000. Initial waits of 0 through 30
/// ticks spread their first route requests over one second of simulation.
///
/// Guests subsequently use only ordinary `Park::advance` and crowd routing.
pub fn build_fixture(case: DiagnosticCase, seed: u64) -> Result<DiagnosticFixture, TrainError> {
    let mut park = if case == DiagnosticCase::Baseline {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(seed)?;
        park.install_train_fixture()?;
        park
    } else {
        let mut source = Park::new(32, 32);
        source.install_train_fixture()?;
        let specs = &source
            .circuit
            .as_ref()
            .ok_or(TrainError::EmptyCircuit)?
            .specs;
        let mut park = Park::new(256, 256);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_train_circuit(specs)?;
        for y in GRID_ORIGIN..GRID_ORIGIN + GRID_SIDE {
            for x in GRID_ORIGIN..GRID_ORIGIN + GRID_SIDE {
                park.place_object(TilePos { x, y }, ObjectKind::Path, Axis::X, PATH_HEIGHT, 2)?;
            }
        }
        park
    };
    if case != DiagnosticCase::Baseline {
        let guest_count = if case == DiagnosticCase::SaveScale {
            256
        } else {
            5_000
        };
        let paths: Vec<_> = park
            .objects
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .copied()
            .collect();
        park.crowd_seed = Some(seed);
        park.crowd_rng = seed;
        for index in 0..guest_count {
            let path = paths[(next_random(&mut park.crowd_rng) % paths.len() as u64) as usize];
            park.guests.push(Guest {
                id: GuestId(park.next_guest_id),
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
                wait_ticks: (index % (usize::from(WALK_TICKS) + 1)) as u16,
            });
            // These fixed diagnostic sizes are far below identity exhaustion.
            park.next_guest_id += 1;
        }
    }
    let metadata = FixtureMetadata {
        case,
        seed,
        map_width: park.width(),
        map_height: park.height(),
        tiles: park.tiles.len(),
        guests: park.guests.len(),
        paths: park
            .objects
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .count(),
        objects: park.objects.len(),
        rides: usize::from(park.circuit.is_some()),
        trains: usize::from(park.train.is_some()),
        track_pieces: park
            .circuit
            .as_ref()
            .map_or(0, |circuit| circuit.pieces.len()),
        save_supported: case != DiagnosticCase::Stress,
    };
    Ok(DiagnosticFixture { park, metadata })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct GraphStats {
    pub nodes: usize,
    pub undirected_edges: usize,
}

/// An immutable snapshot of the production path graph, for isolated measurement.
/// Building includes collecting a stable ID list for choosing query endpoints.
/// Rebuild after changing park objects; this snapshot does not update itself.
pub struct DiagnosticPathGraph {
    graph: PathGraph,
    ids: Vec<ObjectId>,
}

impl DiagnosticPathGraph {
    pub fn build(park: &Park) -> Self {
        let graph = PathGraph::new(park.objects());
        let ids = graph.nodes.keys().copied().collect();
        Self { graph, ids }
    }

    pub fn stats(&self) -> GraphStats {
        GraphStats {
            nodes: self.graph.nodes.len(),
            undirected_edges: self.graph.edges.values().map(Vec::len).sum::<usize>() / 2,
        }
    }

    /// Stable ascending object identities, suitable for deterministic queries.
    pub fn node_ids(&self) -> &[ObjectId] {
        &self.ids
    }

    /// Shortest path excluding `from` and including `to`. Equal valid endpoints
    /// yield an empty route; absent or disconnected endpoints yield `None`.
    ///
    /// This runs the same full breadth-first traversal and tie ordering used by
    /// the crowd when choosing a destination, not a faster early-exit substitute.
    pub fn route(&self, from: ObjectId, to: ObjectId) -> Option<Vec<ObjectId>> {
        self.graph
            .route(from, to)
            .map(|route| route.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SaveError, Speed};

    #[test]
    fn baseline_is_the_unchanged_gameplay_fixture() {
        let fixture = build_fixture(DiagnosticCase::Baseline, 42).unwrap();
        let mut expected = Park::new(32, 32);
        expected.apply_terrain_profile(TerrainProfile::Terraced);
        expected.install_crowd_fixture(42).unwrap();
        expected.install_train_fixture().unwrap();
        assert_eq!(fixture.park, expected);
        assert_eq!((fixture.metadata.guests, fixture.metadata.paths), (12, 37));
        assert_eq!((fixture.metadata.rides, fixture.metadata.trains), (1, 1));
        assert_eq!(fixture.metadata.track_pieces, 46);
        assert!(fixture.metadata.save_supported);
    }

    #[test]
    fn large_fixtures_are_repeatable_and_keep_actual_capacity_limits() {
        for (case, guests) in [
            (DiagnosticCase::SaveScale, 256),
            (DiagnosticCase::Stress, 5_000),
        ] {
            let fixture = build_fixture(case, 711).unwrap();
            assert_eq!(fixture, build_fixture(case, 711).unwrap());
            assert_eq!(fixture.metadata.guests, guests);
            assert_eq!(fixture.metadata.tiles, 65_536);
            assert_eq!(fixture.metadata.paths, 1_024);
            assert_eq!(fixture.metadata.objects, 1_070);
            assert_eq!((fixture.metadata.rides, fixture.metadata.trains), (1, 1));
            assert_eq!(fixture.park.ticks(), 0);
            assert_eq!(fixture.park.next_guest_id, guests as u64 + 1);
            for (index, guest) in fixture.park.guests().iter().enumerate() {
                assert_eq!(guest.id, GuestId(index as u64 + 1));
                assert!(guest.wait_ticks <= WALK_TICKS);
                assert_eq!(
                    fixture.park.object(guest.current.unwrap()).unwrap().pos,
                    guest.anchor
                );
            }
            if case == DiagnosticCase::SaveScale {
                let bytes = fixture.park.to_save_bytes().unwrap();
                assert_eq!(Park::from_save_bytes(&bytes).unwrap(), fixture.park);
            } else {
                assert!(!fixture.metadata.save_supported);
                assert!(matches!(
                    fixture.park.to_save_bytes(),
                    Err(SaveError::InvalidState(_))
                ));
            }
        }
        let first = build_fixture(DiagnosticCase::SaveScale, 711).unwrap();
        let other = build_fixture(DiagnosticCase::SaveScale, 712).unwrap();
        assert_ne!(first.park.guests(), other.park.guests());
        assert_eq!(first.park.objects(), other.park.objects());
    }

    #[test]
    fn probes_use_production_connections_and_leave_all_park_state_unchanged() {
        let fixture = build_fixture(DiagnosticCase::SaveScale, 91).unwrap();
        let before = fixture.park.clone();
        let graph = DiagnosticPathGraph::build(&fixture.park);
        assert_eq!(
            graph.stats(),
            GraphStats {
                nodes: 1_024,
                undirected_edges: 1_984,
            }
        );
        let from = graph.node_ids()[0];
        let to = *graph.node_ids().last().unwrap();
        let route = graph.route(from, to).unwrap();
        assert_eq!(route.len(), 62);
        let mut current = from;
        for next in route {
            assert!(fixture.park.path_neighbors(current).contains(&next));
            current = next;
        }
        assert_eq!(current, to);
        assert_eq!(graph.route(from, from), Some(Vec::new()));
        assert_eq!(graph.route(ObjectId(0), to), None);
        assert_eq!(graph.route(from, ObjectId(u64::MAX)), None);
        assert_eq!(fixture.park, before);

        let mut disconnected = Park::new(4, 4);
        let a = disconnected
            .place_object(TilePos { x: 0, y: 0 }, ObjectKind::Path, Axis::X, 8, 2)
            .unwrap();
        let b = disconnected
            .place_object(TilePos { x: 3, y: 3 }, ObjectKind::Path, Axis::X, 8, 2)
            .unwrap();
        assert_eq!(DiagnosticPathGraph::build(&disconnected).route(a, b), None);
    }

    #[test]
    fn actual_guest_routes_match_the_probed_production_route() {
        let mut first = build_fixture(DiagnosticCase::SaveScale, 123).unwrap().park;
        let mut second = first.clone();
        let graph = DiagnosticPathGraph::build(&first);
        first.advance(Speed::Normal);
        second.advance(Speed::Normal);
        assert_eq!(first, second);
        assert_eq!(first.ticks(), 1);
        let walking: Vec<_> = first
            .guests()
            .iter()
            .filter(|guest| guest.state == GuestState::Walking)
            .collect();
        assert!(!walking.is_empty());
        for guest in walking {
            let expected = graph
                .route(guest.current.unwrap(), guest.destination.unwrap())
                .unwrap();
            let actual: Vec<_> = guest
                .next
                .into_iter()
                .chain(guest.route.iter().copied())
                .collect();
            assert_eq!(actual, expected);
        }
        let bytes = first.to_save_bytes().unwrap();
        assert_eq!(Park::from_save_bytes(&bytes).unwrap(), first);
    }
}
