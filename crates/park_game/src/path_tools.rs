//! Transient construction controls. Only successful edits enter the Park/save.

use bevy::prelude::*;
use park_sim::{
    MAX_PATH_HEIGHT, MAX_SLOPED_PATH_HEIGHT, ObjectId, ObjectKind, Park, PathDirection, PathSlope,
    PlacementError, RideAccessError, RideId, SurfaceId, TilePos,
};

use crate::terrain_tools::{self, TerrainPreview};
use crate::{ViewState, world};

mod entrance;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum PathTool {
    #[default]
    Inspect,
    Build,
    BuildQueue,
    Entrance,
    Exit,
    Demolish,
    RaiseTerrain,
    LowerTerrain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PathPreview {
    pub pos: TilePos,
    pub height: u8,
    pub slope: PathSlope,
    pub queue_owner: Option<RideId>,
    pub entrance_facing: Option<PathDirection>,
    pub exit_gate: bool,
    pub valid: bool,
    pub demolish: bool,
}

#[derive(Clone, Debug)]
pub(super) struct ToolState {
    pub mode: PathTool,
    pub height: u8,
    pub slope: PathSlope,
    pub entrance_facing: PathDirection,
    pub exit_facing: PathDirection,
    pub preview: Option<PathPreview>,
    pub terrain_preview: Option<TerrainPreview>,
    pub hint: String,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            mode: PathTool::Inspect,
            height: 8,
            slope: PathSlope::Flat,
            entrance_facing: PathDirection::North,
            exit_facing: PathDirection::South,
            preview: None,
            terrain_preview: None,
            hint: "Choose a build tool.\nRemove paths, entrances or exits.".into(),
        }
    }
}

impl ToolState {
    pub fn set_mode(&mut self, mode: PathTool) {
        self.mode = mode;
        self.height = self.height.min(self.max_height());
        self.clear_pointer();
    }

    pub fn adjust_height(&mut self, delta: i8) {
        self.height = (i16::from(self.height) + i16::from(delta))
            .clamp(0, i16::from(self.max_height())) as u8;
        self.clear_pointer();
    }

    pub fn max_height(&self) -> u8 {
        if matches!(self.mode, PathTool::Entrance | PathTool::Exit) || self.slope == PathSlope::Flat
        {
            MAX_PATH_HEIGHT
        } else {
            MAX_SLOPED_PATH_HEIGHT
        }
    }

    pub fn cycle_shape(&mut self) {
        if matches!(self.mode, PathTool::Entrance | PathTool::Exit) {
            let facing = if self.mode == PathTool::Exit {
                &mut self.exit_facing
            } else {
                &mut self.entrance_facing
            };
            *facing = match *facing {
                PathDirection::North => PathDirection::East,
                PathDirection::East => PathDirection::South,
                PathDirection::South => PathDirection::West,
                PathDirection::West => PathDirection::North,
            };
            self.clear_pointer();
            return;
        }
        self.slope = match self.slope {
            PathSlope::Flat => PathSlope::North,
            PathSlope::North => PathSlope::East,
            PathSlope::East => PathSlope::South,
            PathSlope::South => PathSlope::West,
            PathSlope::West => PathSlope::Flat,
        };
        self.height = self.height.min(self.max_height());
        self.set_mode(if self.mode == PathTool::BuildQueue {
            PathTool::BuildQueue
        } else {
            PathTool::Build
        });
    }

    pub fn clear_pointer(&mut self) {
        self.preview = None;
        self.terrain_preview = None;
        self.hint = match self.mode {
            PathTool::Inspect => "Choose a build tool.\nRemove paths, entrances or exits.",
            PathTool::Build => "Point at the height plane.\nClick once per path tile.",
            PathTool::BuildQueue => {
                "Build a queue for Meadow Flyer.\nJoin one public path to its gate."
            }
            PathTool::Entrance => "Place beside the station.\nC turns the gate toward it.",
            PathTool::Exit => "Place beside the station.\nConnect its outer edge to public path.",
            PathTool::Demolish => "Point to a path, entrance or exit.\nClick once to remove it.",
            PathTool::RaiseTerrain => {
                "Point to visible grass or sand.\nClick once to raise one step."
            }
            PathTool::LowerTerrain => {
                "Point to visible grass or sand.\nClick once to lower one step."
            }
        }
        .into();
    }
}

pub(super) fn shape_name(slope: PathSlope) -> &'static str {
    match slope {
        PathSlope::Flat => "Level path",
        PathSlope::North => "Rise north",
        PathSlope::East => "Rise east",
        PathSlope::South => "Rise south",
        PathSlope::West => "Rise west",
    }
}

pub(super) fn placement_message(error: PlacementError) -> String {
    match error {
        PlacementError::Occupied { pos, object } => format!(
            "Tile {}, {} is occupied by object #{}.",
            pos.x + 1,
            pos.y + 1,
            object.0
        ),
        PlacementError::TerrainIntersection { terrain_height, .. } => {
            format!("Path is below the ground.\nGround height: {terrain_height}.")
        }
        PlacementError::HeightOutOfRange { .. } => {
            format!("Choose a path height from 0 to {MAX_PATH_HEIGHT}.")
        }
        PlacementError::OutOfBounds { .. } => "Outside the park.".into(),
        PlacementError::CapacityExceeded { resource, max } => {
            format!("Park limit reached: {max} {resource}.")
        }
        PlacementError::IdExhausted => "No more park objects can be created.".into(),
        PlacementError::EmptyClearance => "A path needs two height steps of clearance.".into(),
        PlacementError::FixtureMapSize { .. } => "Demo paths require a 32 x 32 park.".into(),
        PlacementError::EntranceRequiresRide => "Choose the ride Entrance tool.".into(),
    }
}

fn ride_placement_message(error: RideAccessError, slope: PathSlope) -> String {
    match error {
        RideAccessError::Placement(PlacementError::HeightOutOfRange { .. })
            if slope != PathSlope::Flat =>
        {
            format!("Choose a slope low end from 0 to {MAX_SLOPED_PATH_HEIGHT}.")
        }
        RideAccessError::Placement(error) => placement_message(error),
        error => error.to_string(),
    }
}

/// Called only for a focused, unblocked map pointer, after camera input.
/// Revalidate against current authoritative state on every deliberate click.
pub(super) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    view.path_tool.preview = None;
    view.path_tool.terrain_preview = None;
    match view.path_tool.mode {
        PathTool::RaiseTerrain | PathTool::LowerTerrain => {
            terrain_tools::point_at(park, view, click)
        }
        PathTool::Inspect => {
            if click {
                view.selected = view.hovered;
            }
        }
        PathTool::Entrance | PathTool::Exit => entrance::point_at(park, view, point, click),
        PathTool::Build | PathTool::BuildQueue => {
            let queue = view.path_tool.mode == PathTool::BuildQueue;
            let owner = queue.then(|| park.ride().map(|ride| ride.id)).flatten();
            let name = if queue { "Queue" } else { "Path" };
            let height = view.path_tool.height;
            let slope = view.path_tool.slope;
            let Some(pos) =
                world::pick_sloped_path_plane(point, park, view.rotation, height, slope)
            else {
                view.path_tool.hint = "Outside the park at this height.".into();
                if click {
                    view.path_notice = Some("No path placed: outside the park.".into());
                }
                return;
            };
            let validation = if queue {
                owner
                    .ok_or_else(|| "A queue needs an existing ride.".to_string())
                    .and_then(|ride| {
                        park.validate_queue_path(pos, height, slope, ride)
                            .map_err(|error| ride_placement_message(error, slope))
                    })
            } else {
                park.validate_sloped_path(pos, height, slope)
                    .map_err(|error| {
                        if matches!(error, PlacementError::HeightOutOfRange { .. })
                            && slope != PathSlope::Flat
                        {
                            format!("Choose a slope low end from 0 to {MAX_SLOPED_PATH_HEIGHT}.")
                        } else {
                            placement_message(error)
                        }
                    })
            };
            let occluded = world::sloped_path_plane_occluded(
                point,
                park,
                view.rotation,
                height,
                slope,
                view.view_mode,
            );
            let reason = validation.err().or_else(|| {
                occluded.then(|| "Surface hidden here.\nRotate or use underground view.".into())
            });
            if !occluded {
                view.path_tool.preview = Some(PathPreview {
                    pos,
                    height,
                    slope,
                    queue_owner: owner,
                    entrance_facing: None,
                    exit_gate: false,
                    valid: reason.is_none(),
                    demolish: false,
                });
            }
            view.path_tool.hint = reason.clone().unwrap_or_else(|| {
                if queue {
                    format!(
                        "Queue / {} -> {}\nClick once for Meadow Flyer.",
                        height,
                        height + u8::from(slope != PathSlope::Flat)
                    )
                } else if slope == PathSlope::Flat {
                    format!(
                        "Tile {}, {} / height {height}\nClick to place path.",
                        pos.x + 1,
                        pos.y + 1
                    )
                } else {
                    format!(
                        "{} / {} -> {}\nClick to place one ramp.",
                        shape_name(slope),
                        height,
                        height.saturating_add(1)
                    )
                }
            });
            if click {
                if let Some(reason) = reason {
                    view.path_notice = Some(format!("No path placed: {reason}"));
                } else {
                    let placed = if queue {
                        park.place_queue_path(pos, height, slope, owner.expect("validated ride"))
                            .map_err(|error| ride_placement_message(error, slope))
                    } else {
                        park.place_sloped_path(pos, height, slope)
                            .map_err(placement_message)
                    };
                    match placed {
                        Ok(id) => {
                            view.selected = Some(SurfaceId::Object(id));
                            view.hovered = view.selected;
                            view.scene_dirty = true;
                            view.path_tool.clear_pointer();
                            view.path_notice = Some(format!(
                                "{name} #{} placed at {}, {} / height {height}.",
                                id.0,
                                pos.x + 1,
                                pos.y + 1
                            ));
                        }
                        Err(error) => {
                            view.path_notice = Some(format!("No path placed: {error}"));
                        }
                    }
                }
            }
        }
        PathTool::Demolish => {
            let target = match view.hovered {
                Some(SurfaceId::Object(id)) => park.object(id).copied().filter(|object| {
                    matches!(
                        object.kind,
                        ObjectKind::Path | ObjectKind::RideEntrance | ObjectKind::RideExit
                    )
                }),
                _ => None,
            };
            if let Some(object) = target {
                view.path_tool.preview = Some(PathPreview {
                    pos: object.pos,
                    height: object.height,
                    slope: object.path_slope,
                    queue_owner: object.queue_owner,
                    entrance_facing: gate_facing(park, object.id),
                    exit_gate: object.kind == ObjectKind::RideExit,
                    valid: park.object_removal_block_reason(object.id).is_none(),
                    demolish: true,
                });
                let name = object_tool_name(object.kind);
                view.path_tool.hint = park.object_removal_block_reason(object.id).map_or_else(
                    || {
                        format!(
                            "{name} #{} / tile {}, {}\nClick to remove it.",
                            object.id.0,
                            object.pos.x + 1,
                            object.pos.y + 1
                        )
                    },
                    str::to_string,
                );
                if click {
                    remove_path(park, view, object.id);
                }
            } else {
                view.path_tool.hint =
                    "Point to a path, entrance or exit.\nTrack and land stay in place.".into();
                if click {
                    view.path_notice = Some("No path removed: point to a visible path.".into());
                }
            }
        }
    }
}

pub(super) fn remove_selected(park: &mut Park, view: &mut ViewState) {
    let target = view.selected.and_then(|surface| match surface {
        SurfaceId::Object(id)
            if park.object(id).is_some_and(|o| {
                matches!(
                    o.kind,
                    ObjectKind::Path | ObjectKind::RideEntrance | ObjectKind::RideExit
                )
            }) =>
        {
            world::surface_center(park, surface, view.rotation)
                .filter(|point| {
                    world::pick_surface(*point, park, view.rotation, view.view_mode)
                        == Some(surface)
                })
                .map(|_| id)
        }
        _ => None,
    });
    if let Some(id) = target {
        remove_path(park, view, id);
    } else {
        view.path_notice = Some("Select a visible path or gate to remove it.".into());
    }
}

fn remove_path(park: &mut Park, view: &mut ViewState, id: ObjectId) {
    let Some(object) = park.object(id).copied().filter(|object| {
        matches!(
            object.kind,
            ObjectKind::Path | ObjectKind::RideEntrance | ObjectKind::RideExit
        )
    }) else {
        return;
    };
    let name = object_tool_name(object.kind);
    if let Some(reason) = park.object_removal_block_reason(id) {
        view.path_notice = Some(format!("{name} not removed: {reason}"));
        return;
    }
    if park.remove_object(id).is_none() {
        view.path_notice = Some(format!(
            "{name} not removed: it is unavailable or occupied."
        ));
        return;
    }
    if view.selected == Some(SurfaceId::Object(id)) {
        view.selected = None;
    }
    view.hovered = None;
    view.path_tool.clear_pointer();
    view.scene_dirty = true;
    view.path_notice = Some(format!(
        "{name} #{} removed. Connections will be recalculated.",
        id.0
    ));
}

fn gate_facing(park: &Park, id: ObjectId) -> Option<PathDirection> {
    let ride = park.ride()?;
    ride.entrance
        .iter()
        .chain(ride.exit.iter())
        .find(|gate| gate.object == id)
        .map(|gate| gate.facing)
}

fn object_tool_name(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::RideEntrance => "Entrance",
        ObjectKind::RideExit => "Exit",
        _ => "Path",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_tool_keeps_its_facing_separate_from_entrance_and_path_shape() {
        let mut tool = ToolState {
            slope: PathSlope::West,
            ..default()
        };
        tool.set_mode(PathTool::Exit);
        assert_eq!(tool.exit_facing, PathDirection::South);
        assert_eq!(tool.max_height(), MAX_PATH_HEIGHT);
        tool.cycle_shape();
        assert_eq!(tool.mode, PathTool::Exit);
        assert_eq!(tool.exit_facing, PathDirection::West);
        assert_eq!(tool.entrance_facing, PathDirection::North);
        assert_eq!(tool.slope, PathSlope::West);
        tool.set_mode(PathTool::Entrance);
        tool.cycle_shape();
        assert_eq!(tool.entrance_facing, PathDirection::East);
        assert_eq!(tool.exit_facing, PathDirection::West);
        tool.set_mode(PathTool::Inspect);
        assert!(tool.preview.is_none());
    }
}
