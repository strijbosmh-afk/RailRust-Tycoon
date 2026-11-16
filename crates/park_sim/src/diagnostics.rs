//! Opt-in, deterministic measurement inputs and production navigation probes.
//!
//! Fixture creation is setup work, not a simulation measurement. Callers own
//! warmup, timers, iteration counts, and reporting. No diagnostic code runs from
//! `Park::advance`, and no simulation clock or random state changes during a
//! graph probe. Legacy cases retain their original content. Beta cases use the
//! current catalogue, staff, management and production guest-arrival rules.

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
    Starter,
    MediumMixed,
    LargeMixed,
    CrowdCapacity,
}

impl DiagnosticCase {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::SaveScale => "save-scale",
            Self::Stress => "stress",
            Self::Starter => "starter",
            Self::MediumMixed => "medium-mixed",
            Self::LargeMixed => "large-mixed",
            Self::CrowdCapacity => "crowd-capacity",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        [
            Self::Baseline,
            Self::SaveScale,
            Self::Stress,
            Self::Starter,
            Self::MediumMixed,
            Self::LargeMixed,
            Self::CrowdCapacity,
        ]
        .into_iter()
        .find(|case| case.name() == name)
    }
}

/// Actual fixture counts, independent of any aspirational performance target.
/// `save_supported` describes the current codec limits for this fixture.
/// The legacy 5,000-guest stress case exceeds the 512-guest save limit.
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
pub fn build_fixture(case: DiagnosticCase, seed: u64) -> Result<DiagnosticFixture, String> {
    let park = match case {
        DiagnosticCase::Baseline | DiagnosticCase::SaveScale | DiagnosticCase::Stress => {
            return build_legacy_fixture(case, seed).map_err(|error| error.to_string());
        }
        DiagnosticCase::Starter => {
            let mut park = Park::new(32, 32);
            park.install_ground_starter_crowd(seed)
                .map_err(|error| error.to_string())?;
            park.install_ground_starter_coaster()
                .map_err(|error| error.to_string())?;
            park.enable_park_life();
            park.enable_management();
            park
        }
        _ => build_mixed_fixture(case, seed)?,
    };
    let metadata = fixture_metadata(&park, case, seed);
    Ok(DiagnosticFixture { park, metadata })
}

fn build_legacy_fixture(case: DiagnosticCase, seed: u64) -> Result<DiagnosticFixture, TrainError> {
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
                rides_completed: 0,
                ride_history: Vec::new(),
                wallet_cents: crate::INITIAL_GUEST_WALLET_CENTS,
                spent_cents: 0,
                service_spent_cents: 0,
                services_used: 0,
                needs: crate::GuestNeeds::for_guest(seed, GuestId(park.next_guest_id)),
                service_visit: None,
                service_cooldown: 0,
                preferences: crate::GuestPreferences::for_guest(seed, GuestId(park.next_guest_id)),
                satisfaction: 70,
                thought: crate::GuestThought::Exploring,
                thought_tick: 0,
                ratings_given: 0,
                rating_total: 0,
                last_rating: None,
                ride_visit: None,
                ride_cooldown: 0,
                route: VecDeque::new(),
                wait_ticks: (index % (usize::from(WALK_TICKS) + 1)) as u16,
            });
            // These fixed diagnostic sizes are far below identity exhaustion.
            park.next_guest_id += 1;
        }
    }
    let metadata = fixture_metadata(&park, case, seed);
    Ok(DiagnosticFixture { park, metadata })
}

fn fixture_metadata(park: &Park, case: DiagnosticCase, seed: u64) -> FixtureMetadata {
    FixtureMetadata {
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
        rides: park
            .all_rides()
            .filter(|ride| park.ride_available(ride.id))
            .count(),
        trains: park
            .coaster_ids()
            .iter()
            .map(|id| park.coaster_trains(*id).len())
            .sum(),
        track_pieces: park
            .coaster_ids()
            .iter()
            .filter_map(|id| park.coaster_circuit(*id))
            .map(|circuit| circuit.pieces.len())
            .sum(),
        save_supported: case != DiagnosticCase::Stress,
    }
}

fn build_mixed_fixture(case: DiagnosticCase, seed: u64) -> Result<Park, String> {
    let mut park = build_catalogue_showcase()?;
    park.crowd_seed = Some(seed);
    park.crowd_rng = seed;
    if matches!(
        case,
        DiagnosticCase::LargeMixed | DiagnosticCase::CrowdCapacity
    ) {
        let terrain = Park::new(256, 256);
        park.width = terrain.width;
        park.height = terrain.height;
        park.tiles = terrain.tiles;
        for tile in &mut park.tiles {
            tile.terrain = crate::Terrain::Grass;
        }
        // A long public avenue connects the occupied district to the far map
        // edge and exercises routing and chunk visibility over a large area.
        for x in 62..=240 {
            park.place_path(TilePos { x, y: 1 }, 0)
                .map_err(|e| e.to_string())?;
        }
        for y in 2..=240 {
            park.place_path(TilePos { x: 240, y }, 0)
                .map_err(|e| e.to_string())?;
        }
    }
    park.enable_park_life();
    for role in crate::StaffRole::ALL {
        park.hire_park_staff(role)?;
    }
    let shelters: Vec<_> = park
        .objects()
        .iter()
        .filter(|p| p.is_public_path())
        .step_by(12)
        .take(12)
        .map(|p| p.id)
        .collect();
    for path in shelters {
        park.toggle_path_shelter(path)?;
    }
    if case == DiagnosticCase::CrowdCapacity {
        // Admit real parties through the production entry operation. Their
        // wallets, histories and party membership follow the normal rules.
        while park.guests.len() < crate::VISITOR_ACTIVE_CAP {
            park.visitors.arrival_progress = crate::VISITOR_ARRIVAL_TICKS - 1;
            park.step_visitor_arrivals();
        }
    }
    park.enable_management();
    park.set_research(
        crate::ResearchPriority::Facilities,
        crate::ResearchFunding::High,
    )?;
    park.start_promotion(crate::Promotion::Family)?;
    park.to_save_bytes()
        .map_err(|e| format!("Mixed fixture validation: {e}"))?;
    Ok(park)
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

/// Complete catalogue park for acceptance captures and deterministic continuation checks.
/// Its recorded historical admission receipts are confined to this diagnostic fixture.
pub fn build_catalogue_showcase() -> Result<Park, String> {
    use crate::{PathDirection, PathSlope, RideModel, ServiceKind};
    let mut park = Park::new(64, 64);
    park.apply_terrain_profile(TerrainProfile::Flat);
    for tile in &mut park.tiles {
        tile.terrain = crate::Terrain::Grass;
    }
    // A mature fixture history supplies the construction budget. The same
    // receipt writer used by admissions keeps the bounded ledger consistent.
    for visit in 0..1000 {
        park.ticks = (visit + 1) * u64::from(crate::VISITOR_ARRIVAL_TICKS);
        park.record_cash(crate::FinanceKind::ParkAdmission, None, 1900);
    }
    park.ticks += crate::VISITOR_STAY_TICKS;
    park.crowd_seed = Some(0x706861736536);
    park.crowd_rng = 0x706861736536;
    park.next_guest_id = 1001;
    park.visitors.spawned = 1000;
    park.visitors.admission_revenue_cents = 1_900_000;
    park.visitors.departed = crate::DepartedVisitors {
        count: 1000,
        highest_id: 1000,
        wallet_cents: 100_000,
        spent_cents: 1_900_000,
        ..Default::default()
    };
    for row in 0..4 {
        for x in 1..=61 {
            park.place_path(TilePos { x, y: 1 + row * 14 }, 0)
                .map_err(|e| e.to_string())?;
        }
    }
    for y in 2..=57 {
        if ![15, 29, 43].contains(&y) {
            park.place_path(TilePos { x: 1, y }, 0)
                .map_err(|e| e.to_string())?;
        }
    }
    for (index, model) in RideModel::ALL.into_iter().enumerate() {
        let origin = TilePos {
            x: 4 + (index as u32 % 4) * 15,
            y: 4 + (index as u32 / 4) * 14,
        };
        let id = park
            .build_catalogue_ride(model, origin, 0, 0)
            .map_err(|e| format!("{model}: {e}"))?;
        let entrance_x = origin.x + u32::from(model.tracked());
        park.place_ride_entrance(
            TilePos {
                x: entrance_x,
                y: origin.y - 1,
            },
            0,
            PathDirection::South,
            id,
        )
        .map_err(|e| e.to_string())?;
        park.place_queue_path(
            TilePos {
                x: entrance_x,
                y: origin.y - 2,
            },
            0,
            PathSlope::Flat,
            id,
        )
        .map_err(|e| e.to_string())?;
        if model.transport() {
            park.place_ride_exit(
                TilePos {
                    x: origin.x + 7,
                    y: origin.y + 7,
                },
                0,
                PathDirection::North,
                id,
            )
            .map_err(|e| e.to_string())?;
            for x in origin.x - 2..=origin.x + 7 {
                park.place_path(TilePos { x, y: origin.y + 8 }, 0)
                    .map_err(|e| e.to_string())?;
            }
            for y in origin.y - 2..origin.y + 8 {
                park.place_path(TilePos { x: origin.x - 2, y }, 0)
                    .map_err(|e| e.to_string())?;
            }
        } else {
            let exit_x = entrance_x + 2;
            park.place_ride_exit(
                TilePos {
                    x: exit_x,
                    y: origin.y - 1,
                },
                0,
                PathDirection::South,
                id,
            )
            .map_err(|e| e.to_string())?;
            park.place_path(
                TilePos {
                    x: exit_x,
                    y: origin.y - 2,
                },
                0,
            )
            .map_err(|e| e.to_string())?;
        }
        park.set_ride_ticket_price(id, 100)
            .map_err(|e| e.to_string())?;
        park.set_ride_open_checked(id, true)
            .map_err(|e| format!("{model}: {e}"))?;
    }
    for (index, kind) in ServiceKind::ALL.into_iter().enumerate() {
        let x = 4 + index as u32 * 3;
        park.place_service(TilePos { x, y: 58 }, 0, PathDirection::North, kind)
            .map_err(|e| e.to_string())?;
        for dx in 2..=x {
            if !park
                .objects
                .iter()
                .any(|o| o.is_public_path() && o.pos == TilePos { x: dx, y: 57 })
            {
                park.place_path(TilePos { x: dx, y: 57 }, 0)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    let entry = park
        .objects
        .iter()
        .find(|o| o.is_public_path() && o.pos == TilePos { x: 1, y: 1 })
        .unwrap()
        .id;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.set_visitor_arrivals(true).map_err(|e| e.to_string())?;
    Ok(park)
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
    fn beta_fixtures_use_current_counts_and_production_persistence() {
        for case in [
            DiagnosticCase::Starter,
            DiagnosticCase::MediumMixed,
            DiagnosticCase::LargeMixed,
            DiagnosticCase::CrowdCapacity,
        ] {
            let fixture = build_fixture(case, 73).unwrap();
            assert_eq!(fixture.metadata.case, case);
            assert!(fixture.metadata.save_supported);
            assert_eq!(
                Park::from_save_bytes(&fixture.park.to_save_bytes().unwrap()).unwrap(),
                fixture.park
            );
            assert_eq!(fixture, build_fixture(case, 73).unwrap());
            if case != DiagnosticCase::Starter {
                assert_eq!(fixture.metadata.rides, 14);
                assert_eq!(fixture.park.services().len(), 8);
                assert_eq!(fixture.park.park_staff().len(), 5);
            }
            if case == DiagnosticCase::CrowdCapacity {
                assert_eq!(fixture.metadata.guests, crate::VISITOR_ACTIVE_CAP);
                assert_eq!(
                    fixture.park.visitor_flow_stats().tracked,
                    crate::VISITOR_ACTIVE_CAP
                );
            }
        }
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
