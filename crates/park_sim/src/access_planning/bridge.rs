//! Bounded return-route guidance. A route may initially move away from the
//! public network to get around the station, a low track or another obstacle.
use super::*;
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
};

const MAX_SEARCH_STATES: usize = 4096;
const MAX_NEW_TILES: u8 = 32;
type Key = (u32, u32, u8, u8);

struct Node {
    tile: RideAccessSuggestion,
    first: RideAccessSuggestion,
    parent: Option<usize>,
    depth: u8,
    cost: u32,
}

fn key(tile: RideAccessSuggestion) -> Key {
    (tile.pos.y, tile.pos.x, tile.height, slope_rank(tile.slope))
}

fn distance(tile: RideAccessSuggestion, targets: &[&SurfaceObject]) -> u32 {
    targets
        .iter()
        .map(|target| tile_distance(tile.pos, tile.height, target))
        .min()
        .unwrap_or(u32::MAX)
}

struct Search<'a> {
    park: &'a Park,
    targets: &'a [&'a SurfaceObject],
    queue_paths: Vec<&'a SurfaceObject>,
    nodes: Vec<Node>,
    best: BTreeMap<Key, u32>,
    pending: BinaryHeap<Reverse<(u32, u32, Key, usize)>>,
}

impl Search<'_> {
    fn add(&mut self, tile: RideAccessSuggestion, parent: Option<usize>) {
        if self.nodes.len() >= MAX_SEARCH_STATES {
            return;
        }
        let proposed = path_object(tile, ObjectId(0), None);
        // The queue is already connected. A return path must never add another
        // entry to it, or join any other ride's queue, even on a later step.
        if self
            .queue_paths
            .iter()
            .any(|queue| paths_connect(&proposed, queue))
        {
            return;
        }
        let (first, depth, cost) = match parent {
            Some(index) => {
                let previous = &self.nodes[index];
                if previous.depth >= MAX_NEW_TILES {
                    return;
                }
                // Future route pieces are not inserted into the live park.
                // Reject every revisit of a tile to prevent tentative pieces
                // from colliding with each other, including stacked slopes.
                let mut ancestor = Some(index);
                while let Some(index) = ancestor {
                    let node = &self.nodes[index];
                    if node.tile.pos == tile.pos {
                        return;
                    }
                    ancestor = node.parent;
                }
                (previous.first, previous.depth + 1, previous.cost)
            }
            None => (tile, 1, 0),
        };
        let cost = cost + 1 + u32::from(tile.slope != PathSlope::Flat);
        if self
            .best
            .get(&key(tile))
            .is_some_and(|previous| *previous <= cost)
        {
            return;
        }
        self.best.insert(key(tile), cost);
        let priority = cost + distance(tile, self.targets);
        let index = self.nodes.len();
        self.nodes.push(Node {
            tile,
            first,
            parent,
            depth,
            cost,
        });
        self.pending
            .push(Reverse((priority, cost, key(tile), index)));
    }
}

pub(super) fn first_step(
    park: &Park,
    ride: RideId,
    origins: &[&SurfaceObject],
    targets: &[&SurfaceObject],
) -> Option<RideAccessSuggestion> {
    let mut search = Search {
        park,
        targets,
        queue_paths: park
            .objects
            .iter()
            .filter(|path| path.queue_owner.is_some())
            .collect(),
        nodes: Vec::new(),
        best: BTreeMap::new(),
        pending: BinaryHeap::new(),
    };
    let mut initial = Vec::new();
    for origin in origins {
        park.add_public_neighbors(origin, ride, &mut initial);
    }
    initial.sort_by_key(|tile| (distance(*tile, targets), key(*tile), tile.facing));
    initial.dedup();
    for tile in initial {
        search.add(tile, None);
    }
    while let Some(Reverse((_, cost, tile_key, index))) = search.pending.pop() {
        if search.best[&tile_key] != cost {
            continue;
        }
        let node = &search.nodes[index];
        let proposed = path_object(node.tile, ObjectId(0), None);
        if targets
            .iter()
            .any(|target| paths_connect(&proposed, target))
        {
            return Some(node.first);
        }
        let mut next = Vec::new();
        search.park.add_public_neighbors(&proposed, ride, &mut next);
        next.sort_by_key(|tile| (distance(*tile, targets), key(*tile), tile.facing));
        for tile in next {
            search.add(tile, Some(index));
        }
    }
    // Exhaustion is not permission to advertise an arbitrary partial detour.
    None
}
