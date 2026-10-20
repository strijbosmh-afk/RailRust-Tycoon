//! Transient construction controls. Only successful edits enter the Park/save.

use bevy::prelude::*;
use park_sim::{MAX_PATH_HEIGHT, ObjectId, ObjectKind, Park, PlacementError, SurfaceId, TilePos};

use crate::{ViewState, world};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum PathTool {
    #[default]
    Inspect,
    Build,
    Demolish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PathPreview {
    pub pos: TilePos,
    pub height: u8,
    pub valid: bool,
    pub demolish: bool,
}

#[derive(Clone, Debug)]
pub(super) struct ToolState {
    pub mode: PathTool,
    pub height: u8,
    pub preview: Option<PathPreview>,
    pub hint: String,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            mode: PathTool::Inspect,
            height: 8,
            preview: None,
            hint: "Choose Path to build.\nDemolish removes paths only.".into(),
        }
    }
}

impl ToolState {
    pub fn set_mode(&mut self, mode: PathTool) {
        self.mode = mode;
        self.clear_pointer();
    }

    pub fn adjust_height(&mut self, delta: i8) {
        self.height =
            (i16::from(self.height) + i16::from(delta)).clamp(0, i16::from(MAX_PATH_HEIGHT)) as u8;
        self.clear_pointer();
    }

    pub fn clear_pointer(&mut self) {
        self.preview = None;
        self.hint = match self.mode {
            PathTool::Inspect => "Choose Path to build.\nDemolish removes paths only.",
            PathTool::Build => "Point at the height plane.\nClick once per path tile.",
            PathTool::Demolish => "Point to a visible path.\nClick once to remove it.",
        }
        .into();
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
    }
}

/// Called only for a focused, unblocked map pointer, after camera input.
/// Revalidate against current authoritative state on every deliberate click.
pub(super) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    view.path_tool.preview = None;
    match view.path_tool.mode {
        PathTool::Inspect => {
            if click {
                view.selected = view.hovered;
            }
        }
        PathTool::Build => {
            let height = view.path_tool.height;
            let Some(pos) = world::pick_path_plane(point, park, view.rotation, height) else {
                view.path_tool.hint = "Outside the park at this height.".into();
                if click {
                    view.path_notice = Some("No path placed: outside the park.".into());
                }
                return;
            };
            let validation = park.validate_path(pos, height);
            let occluded =
                world::path_plane_occluded(point, park, view.rotation, height, view.view_mode);
            let reason = validation.err().map(placement_message).or_else(|| {
                occluded.then(|| "Surface hidden here.\nRotate or use underground view.".into())
            });
            if !occluded {
                view.path_tool.preview = Some(PathPreview {
                    pos,
                    height,
                    valid: reason.is_none(),
                    demolish: false,
                });
            }
            view.path_tool.hint = reason.clone().unwrap_or_else(|| {
                format!(
                    "Tile {}, {} / height {height}\nClick to place path.",
                    pos.x + 1,
                    pos.y + 1
                )
            });
            if click {
                if let Some(reason) = reason {
                    view.path_notice = Some(format!("No path placed: {reason}"));
                } else {
                    match park.place_path(pos, height) {
                        Ok(id) => {
                            view.selected = Some(SurfaceId::Object(id));
                            view.hovered = view.selected;
                            view.scene_dirty = true;
                            view.path_tool.clear_pointer();
                            view.path_notice = Some(format!(
                                "Path #{} placed at {}, {} / height {height}.",
                                id.0,
                                pos.x + 1,
                                pos.y + 1
                            ));
                        }
                        Err(error) => {
                            view.path_notice =
                                Some(format!("No path placed: {}", placement_message(error)));
                        }
                    }
                }
            }
        }
        PathTool::Demolish => {
            let target = match view.hovered {
                Some(SurfaceId::Object(id)) => park
                    .object(id)
                    .copied()
                    .filter(|object| object.kind == ObjectKind::Path),
                _ => None,
            };
            if let Some(object) = target {
                view.path_tool.preview = Some(PathPreview {
                    pos: object.pos,
                    height: object.height,
                    valid: true,
                    demolish: true,
                });
                view.path_tool.hint = format!(
                    "Path #{} / tile {}, {}\nClick to remove path.",
                    object.id.0,
                    object.pos.x + 1,
                    object.pos.y + 1
                );
                if click {
                    remove_path(park, view, object.id);
                }
            } else {
                view.path_tool.hint =
                    "Point to a visible path.\nTrack and land stay in place.".into();
                if click {
                    view.path_notice = Some("No path removed: point to a visible path.".into());
                }
            }
        }
    }
}

pub(super) fn remove_selected(park: &mut Park, view: &mut ViewState) {
    let target = view.selected.and_then(|surface| match surface {
        SurfaceId::Object(id) if park.object(id).is_some_and(|o| o.kind == ObjectKind::Path) => {
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
        view.path_notice = Some("Select a visible path to remove it.".into());
    }
}

fn remove_path(park: &mut Park, view: &mut ViewState, id: ObjectId) {
    if park
        .object(id)
        .is_none_or(|object| object.kind != ObjectKind::Path)
    {
        return;
    }
    park.remove_object(id);
    if view.selected == Some(SurfaceId::Object(id)) {
        view.selected = None;
    }
    view.hovered = None;
    view.path_tool.clear_pointer();
    view.scene_dirty = true;
    view.path_notice = Some(format!(
        "Path #{} removed. Guest routes will be recalculated.",
        id.0
    ));
}
