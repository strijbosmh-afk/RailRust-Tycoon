//! Isometric surfaces shared by rendering, selection, and picking.

use bevy::{
    asset::RenderAssetUsages, color::ColorToComponents, mesh::Indices, prelude::*,
    render::render_resource::PrimitiveTopology,
};
use park_sim::{Axis, GuestState, ObjectKind, Park, SurfaceId, SurfaceObject, Terrain, TilePos};

mod guest_render;
mod layout;
mod static_chunks;
mod train_render;

pub use guest_render::{GuestRenderStats, GuestRenderer};
pub use layout::{path_focus, scene_bounds};
pub use static_chunks::{StaticChunkGeometry, build_static_chunks};

pub const TILE_WIDTH: f32 = 64.0;
pub const TILE_HEIGHT: f32 = 32.0;
pub const HEIGHT_STEP: f32 = 16.0;
const HALF_WIDTH: f32 = TILE_WIDTH / 2.0;
const HALF_HEIGHT: f32 = TILE_HEIGHT / 2.0;
const BASE_DEPTH: f32 = 20.0;
const SOIL_BOTTOM: f32 = -27.0 / HEIGHT_STEP;
const DETAIL_BIAS: f32 = 0.004;
const HOVER_BIAS: f32 = 0.012;
const SELECTION_BIAS: f32 = 0.016;
type Ink = [f32; 4];

/// The cutaway removes terrain and scenery, including their hit targets.
/// Its low context grid is a guide, not a selectable replacement terrain.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewMode {
    #[default]
    Surface,
    Underground,
}

/// Project a simulation tile into the centered, clockwise-rotated map.
pub fn tile_center(pos: TilePos, size: u32, rotation: u8) -> Vec2 {
    let (x, y) = rotate(pos, size, rotation);
    Vec2::new(
        (x as f32 - y as f32) * HALF_WIDTH,
        -(x as f32 + y as f32 - (size as f32 - 1.0)) * HALF_HEIGHT,
    )
}

/// Height-aware roof center; the stable simulation coordinate never changes.
pub fn elevated_tile_center(park: &Park, pos: TilePos, rotation: u8) -> Option<Vec2> {
    if park.width() != park.height() {
        return None;
    }
    let height = f32::from(park.tile(pos)?.height) * HEIGHT_STEP;
    Some(tile_center(pos, park.width(), rotation) + Vec2::Y * height)
}

/// Center of a surface's visible roof; object base heights remain simulation data.
pub fn surface_center(park: &Park, id: SurfaceId, rotation: u8) -> Option<Vec2> {
    if park.width() != park.height() {
        return None;
    }
    match id {
        SurfaceId::Terrain(pos) => elevated_tile_center(park, pos, rotation),
        SurfaceId::Object(id) => {
            let object = park.object(id)?;
            if let Some(piece) = train_render::piece_for(park, id) {
                return train_render::piece_center(park, piece, rotation);
            }
            Some(object_point(
                object,
                0.0,
                0.0,
                object_top(object),
                park.width(),
                rotation,
            ))
        }
    }
}

/// Pick the nearest selectable face, with the exact visibility rules used by
/// scene and outline geometry. Hidden objects never become fallback hits.
pub fn pick_surface(world: Vec2, park: &Park, rotation: u8, mode: ViewMode) -> Option<SurfaceId> {
    if !world.is_finite() || park.width() != park.height() {
        return None;
    }
    let size = park.width();
    let column = world.x / HALF_WIDTH;
    if column.abs() > size as f32 {
        return None;
    }
    let mut closest: Option<(f32, (u32, u32, u64), SurfaceId)> = None;
    let mut consider = |surface: Surface, tie, id| {
        let Some(depth) = surface.hit_depth(world) else {
            return;
        };
        if closest.is_none_or(|(old_depth, old_tie, _)| {
            depth > old_depth + 0.0001 || ((depth - old_depth).abs() <= 0.0001 && tie > old_tie)
        }) {
            closest = Some((depth, tie, id));
        }
    };
    // Every surface stays within its tile's horizontal diamond bounds.
    // At most three columns per rotated row can contain the pointer.
    for y in 0..size {
        let min_x = (y as f32 + column - 1.0).ceil().max(0.0) as u32;
        let max_x = (y as f32 + column + 1.0).floor().min(size as f32 - 1.0);
        if max_x < min_x as f32 {
            continue;
        }
        for x in min_x..=max_x as u32 {
            let pos = unrotate(x, y, size, rotation);
            if mode == ViewMode::Surface {
                for surface in column_surfaces(park, pos, rotation).into_iter().flatten() {
                    consider(surface, (x + y, x, 0), SurfaceId::Terrain(pos));
                }
            }
        }
    }
    for object in park.objects() {
        if object_visible(park, object, mode) {
            let (x, y) = rotate(object.pos, size, rotation);
            for surface in rendered_object_surfaces(park, object, rotation) {
                consider(
                    surface,
                    (x + y, x, object.id.0),
                    SurfaceId::Object(object.id),
                );
            }
        }
    }
    closest.map(|(_, _, id)| id)
}

#[cfg(test)]
fn pick_tile(world: Vec2, park: &Park, rotation: u8) -> Option<TilePos> {
    match pick_surface(world, park, rotation, ViewMode::Surface)? {
        SurfaceId::Terrain(pos) => Some(pos),
        SurfaceId::Object(_) => None,
    }
}

fn unrotate(x: u32, y: u32, size: u32, rotation: u8) -> TilePos {
    let last = size - 1;
    match rotation % 4 {
        0 => TilePos { x, y },
        1 => TilePos { x: y, y: last - x },
        2 => TilePos {
            x: last - x,
            y: last - y,
        },
        _ => TilePos { x: last - y, y: x },
    }
}

fn rotate(pos: TilePos, size: u32, rotation: u8) -> (u32, u32) {
    let last = size.saturating_sub(1);
    match rotation % 4 {
        0 => (pos.x, pos.y),
        1 => (last - pos.y, pos.x),
        2 => (last - pos.x, last - pos.y),
        _ => (pos.y, last - pos.x),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SurfaceKind {
    Top,
    LeftCliff,
    RightCliff,
}

#[derive(Clone, Copy)]
struct Surface {
    vertices: [Vec3; 4],
    kind: SurfaceKind,
}

impl Surface {
    fn has_area(self) -> bool {
        let [a, b, c, d] = self.vertices.map(Vec3::truncate);
        let area = (b - a).perp_dot(c - a) + (c - a).perp_dot(d - a);
        let extent = [b, c, d]
            .into_iter()
            .map(|point| point.distance(a))
            .fold(0.0_f32, f32::max);
        // Subtract the origin before taking cross products. Absolute screen
        // coordinates lose precision on thin faces far from the map center.
        // Exact edge-on slopes can retain a few float ulps of apparent area;
        // a two-thousandth-pixel thickness is below both rendering and picking.
        area.abs() > extent * 0.002
    }

    /// Depth is the face's affine plane, matching GPU interpolation for
    /// horizontal roofs, vertical cliffs, and inclined track ribbons.
    fn depth_at(self, point: Vec2) -> f32 {
        let [a, b, c, d] = self.vertices;
        for (b, c) in [(b, c), (c, d)] {
            let ab = b.truncate() - a.truncate();
            let ac = c.truncate() - a.truncate();
            let determinant = ab.perp_dot(ac);
            if determinant.abs() > 0.00001 {
                let relative = point - a.truncate();
                let u = relative.perp_dot(ac) / determinant;
                let v = ab.perp_dot(relative) / determinant;
                return a.z + u * (b.z - a.z) + v * (c.z - a.z);
            }
        }
        a.z
    }

    fn point(self, point: Vec2, bias: f32) -> Vec3 {
        point.extend(self.depth_at(point) + bias)
    }

    fn hit_depth(self, point: Vec2) -> Option<f32> {
        if !self.has_area() {
            return None;
        }
        // Hit the two emitted triangles directly. A polygon-edge predicate
        // with an absolute cross-product epsilon disagrees with rasterization
        // along the very thin upper edge of a rail viewed exactly side-on.
        let [a, b, c, d] = self.vertices;
        for (b, c) in [(b, c), (c, d)] {
            let ab = b.truncate() - a.truncate();
            let ac = c.truncate() - a.truncate();
            let determinant = ab.perp_dot(ac);
            if determinant.abs() <= 0.00001 {
                continue;
            }
            let relative = point - a.truncate();
            let u = relative.perp_dot(ac) / determinant;
            let v = ab.perp_dot(relative) / determinant;
            if u >= -0.00001 && v >= -0.00001 && u + v <= 1.00001 {
                return Some(a.z + u * (b.z - a.z) + v * (c.z - a.z));
            }
        }
        None
    }
}

/// u/v are rotated ground coordinates; height is in simulation terrain steps.
/// Increasing u and v together moves toward the camera. Keeping this depth
/// on every vertex gives both vertical cliffs and horizontal roofs the same
/// physical ordering, rather than giving a whole tall tile one painter rank.
fn project(u: f32, v: f32, height: f32, size: u32) -> Vec3 {
    Vec3::new(
        (u - v) * HALF_WIDTH,
        -(u + v - (size as f32 - 1.0)) * HALF_HEIGHT + height * HEIGHT_STEP,
        BASE_DEPTH + u + v,
    )
}

fn column_surfaces(park: &Park, pos: TilePos, rotation: u8) -> [Option<Surface>; 3] {
    let Some(tile) = park.tile(pos) else {
        return [None; 3];
    };
    let size = park.width();
    let (x, y) = rotate(pos, size, rotation);
    let (u, v) = (x as f32, y as f32);
    let height = f32::from(tile.height);
    let top = Surface {
        vertices: [
            project(u - 0.5, v - 0.5, height, size),
            project(u - 0.5, v + 0.5, height, size),
            project(u + 0.5, v + 0.5, height, size),
            project(u + 0.5, v - 0.5, height, size),
        ],
        kind: SurfaceKind::Top,
    };
    let neighbor_height = |x: u32, y: u32| {
        if x >= size || y >= size {
            SOIL_BOTTOM
        } else {
            f32::from(
                park.tile(unrotate(x, y, size, rotation))
                    .expect("neighbor is inside square park")
                    .height,
            )
        }
    };
    let left_height = neighbor_height(x, y + 1);
    let right_height = neighbor_height(x + 1, y);
    let left = (height > left_height).then(|| Surface {
        vertices: [
            top.vertices[1],
            top.vertices[2],
            project(u + 0.5, v + 0.5, left_height, size),
            project(u - 0.5, v + 0.5, left_height, size),
        ],
        kind: SurfaceKind::LeftCliff,
    });
    let right = (height > right_height).then(|| Surface {
        vertices: [
            top.vertices[2],
            top.vertices[3],
            project(u + 0.5, v - 0.5, right_height, size),
            project(u + 0.5, v + 0.5, right_height, size),
        ],
        kind: SurfaceKind::RightCliff,
    });
    [Some(top), left, right]
}

fn object_visible(park: &Park, object: &SurfaceObject, mode: ViewMode) -> bool {
    mode == ViewMode::Underground || !park.is_underground(object.id)
}

fn object_top(object: &SurfaceObject) -> f32 {
    f32::from(object.height)
        + match object.kind {
            ObjectKind::Path => 0.22,
            ObjectKind::Track => 0.12,
        }
}

fn object_point(
    object: &SurfaceObject,
    along: f32,
    across: f32,
    height: f32,
    size: u32,
    rotation: u8,
) -> Vec2 {
    let (x, y) = rotate(object.pos, size, rotation);
    let (dx, dy) = match object.axis {
        Axis::X => (along, across),
        Axis::Y => (across, along),
    };
    let (du, dv) = match rotation % 4 {
        0 => (dx, dy),
        1 => (-dy, dx),
        2 => (-dx, -dy),
        _ => (dy, -dx),
    };
    project(x as f32 + du, y as f32 + dv, height, size).truncate()
}

/// Unrotated footprint. Path decks reach every connected cardinal edge;
/// their axis selects plank orientation, not the legal walking directions.
fn object_bounds(park: &Park, object: &SurfaceObject) -> (f32, f32, f32, f32) {
    let across = match object.kind {
        ObjectKind::Path => 0.34,
        ObjectKind::Track => 0.31,
    };
    let (half_x, half_y) = match object.axis {
        Axis::X => (0.5, across),
        Axis::Y => (across, 0.5),
    };
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (-half_x, half_x, -half_y, half_y);
    if object.kind == ObjectKind::Path {
        for neighbor in park
            .path_neighbors(object.id)
            .into_iter()
            .filter_map(|id| park.object(id))
        {
            if neighbor.pos.x < object.pos.x {
                min_x = -0.5;
            } else if neighbor.pos.x > object.pos.x {
                max_x = 0.5;
            } else if neighbor.pos.y < object.pos.y {
                min_y = -0.5;
            } else if neighbor.pos.y > object.pos.y {
                max_y = 0.5;
            }
        }
    }
    (min_x, max_x, min_y, max_y)
}

/// A horizontal deck and its two front faces. Detail strokes are clipped to
/// the top, so rendering, picking, and outlines share the connected footprint.
fn object_surfaces(park: &Park, object: &SurfaceObject, rotation: u8) -> [Surface; 3] {
    let size = park.width();
    let (x, y) = rotate(object.pos, size, rotation);
    let (u, v) = (x as f32, y as f32);
    let (min_x, max_x, min_y, max_y) = object_bounds(park, object);
    let (min_u, max_u, min_v, max_v) = match rotation % 4 {
        0 => (min_x, max_x, min_y, max_y),
        1 => (-max_y, -min_y, min_x, max_x),
        2 => (-max_x, -min_x, -max_y, -min_y),
        _ => (min_y, max_y, -max_x, -min_x),
    };
    let height = object_top(object);
    let bottom = f32::from(object.height);
    let top = Surface {
        vertices: [
            project(u + min_u, v + min_v, height, size),
            project(u + min_u, v + max_v, height, size),
            project(u + max_u, v + max_v, height, size),
            project(u + max_u, v + min_v, height, size),
        ],
        kind: SurfaceKind::Top,
    };
    [
        top,
        Surface {
            vertices: [
                top.vertices[1],
                top.vertices[2],
                project(u + max_u, v + max_v, bottom, size),
                project(u + min_u, v + max_v, bottom, size),
            ],
            kind: SurfaceKind::LeftCliff,
        },
        Surface {
            vertices: [
                top.vertices[2],
                top.vertices[3],
                project(u + max_u, v + min_v, bottom, size),
                project(u + max_u, v + max_v, bottom, size),
            ],
            kind: SurfaceKind::RightCliff,
        },
    ]
}

fn selectable_surfaces(park: &Park, id: SurfaceId, rotation: u8, mode: ViewMode) -> Vec<Surface> {
    match id {
        SurfaceId::Terrain(pos) if mode == ViewMode::Surface => {
            column_surfaces(park, pos, rotation)
                .into_iter()
                .flatten()
                .collect()
        }
        SurfaceId::Object(id) => park
            .object(id)
            .filter(|object| object_visible(park, object, mode))
            .map(|object| rendered_object_surfaces(park, object, rotation))
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn rendered_object_surfaces(park: &Park, object: &SurfaceObject, rotation: u8) -> Vec<Surface> {
    if let Some(piece) = train_render::piece_for(park, object.id) {
        train_render::piece_faces(park, piece, rotation)
            .into_iter()
            .map(|face| face.surface)
            .collect()
    } else {
        object_surfaces(park, object, rotation).to_vec()
    }
}

/// Moving cars share physical world depth with terrain and sampled rails.
pub fn train_mesh(park: &Park, rotation: u8, mode: ViewMode) -> Mesh {
    train_render::train_mesh(park, rotation, mode)
}

/// Projected circuit bounds, including the height of its sampled hills.
pub fn train_focus(park: &Park, rotation: u8) -> Option<Vec2> {
    train_render::train_focus(park, rotation)
}

fn paint_objects(mesh: &mut Painter, park: &Park, rotation: u8, mode: ViewMode) {
    for object in park.objects() {
        if !object_visible(park, object, mode) {
            continue;
        }
        if let Some(piece) = train_render::piece_for(park, object.id) {
            for face in train_render::piece_faces(park, piece, rotation) {
                mesh.surface(face.surface, face.ink);
            }
            mesh.end_group(PaintLayer::Object);
            continue;
        }
        let surfaces = object_surfaces(park, object, rotation);
        let top = surfaces[0];
        for surface in surfaces {
            let ink = match (object.kind, surface.kind) {
                (ObjectKind::Path, SurfaceKind::Top) => rgb(224, 189, 128),
                (ObjectKind::Path, SurfaceKind::LeftCliff) => rgb(155, 108, 65),
                (ObjectKind::Path, SurfaceKind::RightCliff) => rgb(116, 76, 47),
                (ObjectKind::Track, SurfaceKind::Top) => rgb(65, 79, 84),
                (ObjectKind::Track, SurfaceKind::LeftCliff) => rgb(48, 60, 65),
                (ObjectKind::Track, SurfaceKind::RightCliff) => rgb(37, 48, 54),
            };
            mesh.surface(surface, ink);
        }
        let point = |along, across| {
            object_point(
                object,
                along,
                across,
                object_top(object),
                park.width(),
                rotation,
            )
        };
        match object.kind {
            ObjectKind::Path => {
                let (min_x, max_x, min_y, max_y) = object_bounds(park, object);
                let (min_across, max_across) = match object.axis {
                    Axis::X => (min_y, max_y),
                    Axis::Y => (min_x, max_x),
                };
                // Original timber planks and a dark inset edge make the
                // raised walkway readable. A connected junction has no
                // dark border painted across its open walking edge.
                for along in [-0.4, -0.2, 0.0, 0.2, 0.4] {
                    mesh.surface_line(
                        top,
                        point(along, min_across + 0.02),
                        point(along, max_across - 0.02),
                        0.8,
                        DETAIL_BIAS,
                        rgb(188, 143, 88),
                    );
                }
                for across in [
                    (min_across > -0.5).then_some(min_across + 0.05),
                    (max_across < 0.5).then_some(max_across - 0.05),
                ]
                .into_iter()
                .flatten()
                {
                    mesh.surface_line(
                        top,
                        point(-0.5, across),
                        point(0.5, across),
                        1.8,
                        DETAIL_BIAS * 2.0,
                        rgb(139, 91, 54),
                    );
                }
            }
            ObjectKind::Track => {
                for along in [-0.4, -0.2, 0.0, 0.2, 0.4] {
                    mesh.surface_line(
                        top,
                        point(along, -0.26),
                        point(along, 0.26),
                        2.3,
                        DETAIL_BIAS,
                        rgb(146, 109, 78),
                    );
                }
                for across in [-0.17, 0.17] {
                    mesh.surface_line(
                        top,
                        point(-0.5, across),
                        point(0.5, across),
                        2.2,
                        DETAIL_BIAS * 2.0,
                        rgb(213, 224, 220),
                    );
                }
            }
        }
        mesh.end_group(PaintLayer::Object);
    }
}

fn cutaway_painter(park: &Park, rotation: u8, grid: bool, record_groups: bool) -> Painter {
    let mut mesh = Painter::with_groups(record_groups);
    let size = park.width();
    for y in 0..size {
        for x in 0..size {
            let (u, v) = (x as f32, y as f32);
            let floor = Surface {
                vertices: [
                    project(u - 0.5, v - 0.5, -1.0, size),
                    project(u - 0.5, v + 0.5, -1.0, size),
                    project(u + 0.5, v + 0.5, -1.0, size),
                    project(u + 0.5, v - 0.5, -1.0, size),
                ],
                kind: SurfaceKind::Top,
            };
            let pos = unrotate(x, y, size, rotation);
            let height = park.tile(pos).expect("floor stays inside park").height;
            let shade = i16::from(height.min(6)) * 2;
            mesh.surface(floor, varied(199, 210, 199, -shade));
            if grid {
                for edge in [0, 1] {
                    mesh.surface_line(
                        floor,
                        floor.vertices[edge].truncate(),
                        floor.vertices[edge + 1].truncate(),
                        0.65,
                        DETAIL_BIAS,
                        rgb(173, 190, 179),
                    );
                }
            }
            mesh.end_group(PaintLayer::Terrain);
        }
    }
    paint_objects(&mut mesh, park, rotation, ViewMode::Underground);
    mesh
}

/// Build terrain, soil edges, trees, and a decorative entrance in one mesh.
/// The material should be opaque white so per-vertex colors and depth apply.
pub fn build_scene(park: &Park, rotation: u8, grid: bool, mode: ViewMode) -> Mesh {
    scene_painter(park, rotation, grid, mode, false).finish()
}

fn scene_painter(
    park: &Park,
    rotation: u8,
    grid: bool,
    mode: ViewMode,
    record_groups: bool,
) -> Painter {
    assert_eq!(
        park.width(),
        park.height(),
        "isometric scene expects a square park"
    );
    if mode == ViewMode::Underground {
        return cutaway_painter(park, rotation, grid, record_groups);
    }
    let size = park.width();
    let mut mesh = Painter::with_groups(record_groups);
    let half_x = size as f32 * HALF_WIDTH;
    let half_y = size as f32 * HALF_HEIGHT;
    let top = Vec2::new(0.0, half_y);
    let right = Vec2::new(half_x, 0.0);
    let bottom = Vec2::new(0.0, -half_y);
    let left = Vec2::new(-half_x, 0.0);

    // A low, deliberately crisp shadow makes the whole map read as a diorama.
    mesh.polygon(
        &[
            top + Vec2::new(10.0, -12.0),
            left + Vec2::new(-7.0, -20.0),
            bottom + Vec2::new(10.0, -45.0),
            right + Vec2::new(18.0, -20.0),
        ],
        -10.0,
        rgb(24, 46, 48),
    );
    mesh.end_group(PaintLayer::Shadow);

    for y in 0..size {
        for x in 0..size {
            let pos = TilePos { x, y };
            let tile = park.tile(pos).expect("loop stays inside the park");
            let center =
                elevated_tile_center(park, pos, rotation).expect("loop stays inside square park");
            let variation = (noise(pos) % 5) as i16 - 2;
            let color = match tile.terrain {
                Terrain::Grass => varied(108, 153, 88, variation),
                Terrain::Water => varied(74, 157, 165, variation),
                Terrain::Sand => varied(213, 196, 145, variation),
            };
            let surfaces = column_surfaces(park, pos, rotation);
            let top = surfaces[0].expect("every tile has a roof");
            for surface in surfaces.into_iter().flatten() {
                let ink = match surface.kind {
                    SurfaceKind::Top => color,
                    SurfaceKind::LeftCliff => varied(143, 113, 75, variation),
                    SurfaceKind::RightCliff => varied(112, 87, 58, variation),
                };
                mesh.surface(surface, ink);
                if surface.kind != SurfaceKind::Top {
                    let a = surface.vertices[0].truncate();
                    let b = surface.vertices[1].truncate();
                    let face_height = a.y - surface.vertices[3].y;
                    let mut layer = HEIGHT_STEP;
                    while layer < face_height {
                        mesh.surface_line(
                            surface,
                            a - Vec2::Y * layer,
                            b - Vec2::Y * layer,
                            0.8,
                            DETAIL_BIAS,
                            rgb(157, 126, 83),
                        );
                        layer += HEIGHT_STEP;
                    }
                }
            }

            match tile.terrain {
                Terrain::Water => {
                    if !noise(pos).is_multiple_of(3) {
                        let offset = (noise(pos) % 11) as f32 - 5.0;
                        mesh.surface_line(
                            top,
                            center + Vec2::new(-11.0, offset),
                            center + Vec2::new(8.0, offset),
                            1.1,
                            DETAIL_BIAS,
                            rgb(133, 197, 192),
                        );
                        mesh.surface_line(
                            top,
                            center + Vec2::new(2.0, offset + 4.0),
                            center + Vec2::new(9.0, offset + 4.0),
                            0.8,
                            DETAIL_BIAS,
                            rgb(105, 179, 181),
                        );
                    }
                }
                Terrain::Grass if noise(pos).is_multiple_of(17) => {
                    let tuft = center + Vec2::new(-3.0, -1.0);
                    mesh.surface_line(
                        top,
                        tuft,
                        tuft + Vec2::new(-2.0, 3.0),
                        0.9,
                        DETAIL_BIAS,
                        rgb(132, 173, 105),
                    );
                    mesh.surface_line(
                        top,
                        tuft + Vec2::new(3.0, 0.0),
                        tuft + Vec2::new(4.0, 4.0),
                        0.9,
                        DETAIL_BIAS,
                        rgb(132, 173, 105),
                    );
                }
                Terrain::Sand if noise(pos).is_multiple_of(5) => {
                    mesh.surface_ellipse(
                        top,
                        center + Vec2::new(4.0, 1.0),
                        2.4,
                        1.2,
                        DETAIL_BIAS,
                        rgb(189, 176, 133),
                    );
                }
                _ => {}
            }
            if grid {
                let stroke = match tile.terrain {
                    Terrain::Grass => rgb(100, 143, 82),
                    Terrain::Water => rgb(70, 150, 158),
                    Terrain::Sand => rgb(201, 185, 137),
                };
                // Two edges per tile avoid double-width interior grid lines.
                mesh.surface_line(
                    top,
                    center + Vec2::Y * HALF_HEIGHT,
                    center + Vec2::X * HALF_WIDTH,
                    0.65,
                    DETAIL_BIAS * 2.0,
                    stroke,
                );
                mesh.surface_line(
                    top,
                    center + Vec2::X * HALF_WIDTH,
                    center - Vec2::Y * HALF_HEIGHT,
                    0.65,
                    DETAIL_BIAS * 2.0,
                    stroke,
                );
            }
            mesh.end_group(PaintLayer::Terrain);
        }
    }

    // Scenery is anchored to simulation coordinates, so it follows rotation.
    // The generous central clearing is reserved for later construction tools.
    let mut trees = Vec::new();
    for y in 0..size {
        for x in 0..size {
            let pos = TilePos { x, y };
            let edge = x.min(y).min(size - 1 - x).min(size - 1 - y);
            let entrance_clearance = x <= 4 && y.abs_diff(size / 2) <= 3;
            if edge < 4
                && !entrance_clearance
                && noise(pos).is_multiple_of(5)
                && park
                    .tile(pos)
                    .is_some_and(|tile| tile.terrain == Terrain::Grass)
            {
                trees.push(pos);
            }
        }
    }
    // Also preserve painter order if a caller uses a blending material.
    trees.sort_by(|a, b| {
        tile_center(*b, size, rotation)
            .y
            .total_cmp(&tile_center(*a, size, rotation).y)
    });
    for pos in trees {
        let base = elevated_tile_center(park, pos, rotation).expect("valid scenery tile");
        let top = column_surfaces(park, pos, rotation)[0].expect("valid scenery roof");
        tree(&mut mesh, base, top, noise(pos));
        mesh.end_group(PaintLayer::Scenery);
    }

    if size >= 12 {
        let entrance = TilePos { x: 1, y: size / 2 };
        let base = elevated_tile_center(park, entrance, rotation).expect("valid entrance tile");
        let top = column_surfaces(park, entrance, rotation)[0].expect("valid entrance roof");
        entrance_arch(&mut mesh, base, top);
        mesh.end_group(PaintLayer::Scenery);
    }
    paint_objects(&mut mesh, park, rotation, mode);
    mesh
}

/// Outline only the selected owner's visible surfaces. A small depth bias
/// prevents z-fighting while nearer terrain and objects still hide its edges.
pub fn selection_mesh(
    hover: Option<SurfaceId>,
    selected: Option<SurfaceId>,
    park: &Park,
    rotation: u8,
    mode: ViewMode,
) -> Mesh {
    let mut mesh = Painter::default();
    if park.width() != park.height() {
        return mesh.finish();
    }
    for (id, width, bias, ink) in [
        (hover, 1.4, HOVER_BIAS, rgb(247, 246, 216)),
        (selected, 3.0, SELECTION_BIAS, rgb(255, 205, 96)),
    ] {
        if let Some(id) = id {
            for surface in selectable_surfaces(park, id, rotation, mode) {
                mesh.surface_outline(surface, width, bias, ink);
            }
        }
    }
    mesh.finish()
}

/// Moving people have their own mesh so a simulation tick never has to
/// regenerate the static terrain. They are decorations of authoritative guest
/// state for now; clicking through a person still inspects the path beneath.
pub fn guest_mesh(park: &Park, rotation: u8, mode: ViewMode) -> Mesh {
    let mut mesh = Painter::default();
    if park.width() != park.height() {
        return mesh.finish();
    }
    for guest in park.guests() {
        if guest
            .current
            .and_then(|id| park.object(id))
            .is_some_and(|object| !object_visible(park, object, mode))
        {
            continue;
        }
        let Some(base) = guest_anchor(park, guest.id, rotation) else {
            continue;
        };
        let point = base.truncate();
        // Clip the contact shadow to the actual supporting path decks. While
        // crossing a tile edge the two roofs form the continuous support.
        for object in [guest.current, guest.next]
            .into_iter()
            .flatten()
            .filter_map(|id| park.object(id))
        {
            let top = object_surfaces(park, object, rotation)[0];
            mesh.surface_ellipse(top, point, 5.3, 2.6, DETAIL_BIAS * 3.0, rgb(145, 112, 77));
        }
        // A tiny billboard has the depth of its feet, so nearer terrain or
        // decks can hide it. The bias only separates it from its own roof.
        let depth = base.z + 0.04;
        let stride = if guest.state == GuestState::Walking {
            ((f32::from(guest.progress) / f32::from(park_sim::WALK_TICKS)) * std::f32::consts::TAU)
                .sin()
                * 1.8
        } else {
            0.0
        };
        for (side, step) in [(-1.8, stride), (1.8, -stride)] {
            mesh.line(
                point + Vec2::new(side, 7.5),
                point + Vec2::new(side + step, 0.8),
                2.4,
                depth,
                rgb(43, 62, 68),
            );
        }
        let shirt = if guest.state == GuestState::Lost {
            rgb(235, 112, 44)
        } else {
            match guest.id.0 % 6 {
                0 => rgb(213, 77, 82),
                1 => rgb(66, 126, 188),
                2 => rgb(150, 88, 174),
                3 => rgb(242, 184, 65),
                4 => rgb(53, 154, 140),
                _ => rgb(222, 119, 161),
            }
        };
        mesh.quad(
            [
                point + Vec2::new(-3.3, 7.0),
                point + Vec2::new(3.3, 7.0),
                point + Vec2::new(3.8, 17.0),
                point + Vec2::new(-3.8, 17.0),
            ],
            depth + 0.001,
            shirt,
        );
        let skin = if guest.id.0.is_multiple_of(3) {
            rgb(174, 119, 84)
        } else {
            rgb(239, 192, 143)
        };
        for side in [-1.0, 1.0] {
            mesh.line(
                point + Vec2::new(side * 4.0, 15.3),
                point + Vec2::new(side * 4.3, 9.2 + stride * side),
                1.8,
                depth + 0.002,
                skin,
            );
        }
        mesh.ellipse(
            point + Vec2::new(0.0, 20.2),
            3.5,
            4.0,
            depth + 0.003,
            rgb(67, 53, 42),
        );
        mesh.ellipse(point + Vec2::new(0.3, 19.6), 3.1, 3.2, depth + 0.004, skin);
        if guest.state == GuestState::Lost {
            mesh.line(
                point + Vec2::new(0.0, 28.0),
                point + Vec2::new(0.0, 32.0),
                2.0,
                depth + 0.005,
                rgb(201, 68, 34),
            );
            mesh.ellipse(
                point + Vec2::new(0.0, 25.5),
                1.0,
                1.0,
                depth + 0.005,
                rgb(201, 68, 34),
            );
        }
    }
    mesh.finish()
}

fn guest_anchor(park: &Park, id: park_sim::GuestId, rotation: u8) -> Option<Vec3> {
    let position = park.guest_position(id)?;
    let (from_u, from_v) = rotate(position.from, park.width(), rotation);
    let (to_u, to_v) = rotate(position.to, park.width(), rotation);
    let fraction = f32::from(position.progress) / f32::from(position.steps.max(1));
    let u = from_u as f32 + (to_u as f32 - from_u as f32) * fraction;
    let v = from_v as f32 + (to_v as f32 - from_v as f32) * fraction;
    // Path tops are shallow decks within their two-step clearance interval.
    // The guest's base follows that visible roof, not the terrain underneath.
    Some(project(
        u,
        v,
        f32::from(position.height) + 0.22,
        park.width(),
    ))
}

fn tree(mesh: &mut Painter, base: Vec2, top: Surface, seed: u32) {
    let depth = top.depth_at(base) + 0.025;
    let scale = 0.85 + (seed % 6) as f32 * 0.055;
    mesh.surface_ellipse(
        top,
        base + Vec2::new(6.0, -1.0),
        18.0 * scale,
        7.0 * scale,
        DETAIL_BIAS,
        rgb(87, 129, 74),
    );
    mesh.line(
        base,
        base + Vec2::new(0.0, 25.0 * scale),
        5.0 * scale,
        depth,
        rgb(119, 88, 58),
    );
    mesh.line(
        base + Vec2::new(-1.0, 1.0),
        base + Vec2::new(-1.0, 24.0 * scale),
        1.3 * scale,
        depth + 0.001,
        rgb(159, 119, 73),
    );
    if seed.is_multiple_of(3) {
        for (height, width, ink) in [
            (18.0, 19.0, rgb(47, 108, 76)),
            (30.0, 16.0, rgb(55, 125, 83)),
            (41.0, 11.0, rgb(77, 143, 89)),
        ] {
            mesh.triangle(
                [
                    base + Vec2::new(-width, height) * scale,
                    base + Vec2::new(width, height) * scale,
                    base + Vec2::new(0.0, height + 24.0) * scale,
                ],
                depth + height / 10_000.0,
                ink,
            );
        }
    } else {
        mesh.ellipse(
            base + Vec2::new(1.0, 34.0) * scale,
            21.0 * scale,
            24.0 * scale,
            depth + 0.002,
            rgb(51, 113, 72),
        );
        mesh.ellipse(
            base + Vec2::new(-6.0, 40.0) * scale,
            17.0 * scale,
            20.0 * scale,
            depth + 0.003,
            rgb(69, 133, 76),
        );
        mesh.ellipse(
            base + Vec2::new(-7.0, 48.0) * scale,
            11.0 * scale,
            12.0 * scale,
            depth + 0.004,
            rgb(94, 153, 85),
        );
    }
}

fn entrance_arch(mesh: &mut Painter, base: Vec2, top: Surface) {
    let depth = top.depth_at(base) + 0.025;
    mesh.surface_ellipse(
        top,
        base + Vec2::new(4.0, -1.0),
        36.0,
        10.0,
        DETAIL_BIAS,
        rgb(91, 131, 73),
    );
    for offset in [-27.0, 27.0] {
        let foot = base + Vec2::new(offset, 0.0);
        mesh.line(
            foot,
            foot + Vec2::new(0.0, 49.0),
            7.0,
            depth,
            rgb(239, 226, 188),
        );
        mesh.line(
            foot + Vec2::new(2.0, 0.0),
            foot + Vec2::new(2.0, 49.0),
            2.0,
            depth + 0.001,
            rgb(184, 170, 133),
        );
        mesh.ellipse(
            foot + Vec2::new(0.0, 49.0),
            5.0,
            4.0,
            depth + 0.002,
            rgb(241, 192, 92),
        );
    }
    mesh.quad(
        [
            base + Vec2::new(-33.0, 34.0),
            base + Vec2::new(33.0, 34.0),
            base + Vec2::new(33.0, 47.0),
            base + Vec2::new(-33.0, 47.0),
        ],
        depth + 0.003,
        rgb(182, 82, 65),
    );
    mesh.line(
        base + Vec2::new(-30.0, 44.0),
        base + Vec2::new(30.0, 44.0),
        2.0,
        depth + 0.004,
        rgb(235, 176, 117),
    );
    // A small sun medallion is an original graphic, without borrowed branding.
    mesh.ellipse(
        base + Vec2::new(0.0, 41.0),
        6.5,
        6.5,
        depth + 0.005,
        rgb(250, 215, 141),
    );
    mesh.line(
        base + Vec2::new(0.0, 48.0),
        base + Vec2::new(0.0, 67.0),
        1.5,
        depth + 0.006,
        rgb(225, 212, 173),
    );
    mesh.triangle(
        [
            base + Vec2::new(0.0, 67.0),
            base + Vec2::new(15.0, 62.0),
            base + Vec2::new(0.0, 57.0),
        ],
        depth + 0.007,
        rgb(228, 140, 91),
    );
}

fn noise(pos: TilePos) -> u32 {
    let seed = pos
        .x
        .wrapping_mul(374_761_393)
        .wrapping_add(pos.y.wrapping_mul(668_265_263));
    (seed ^ (seed >> 13)).wrapping_mul(1_274_126_177)
}

fn rgb(r: u8, g: u8, b: u8) -> Ink {
    Color::srgb_u8(r, g, b).to_linear().to_f32_array()
}

fn varied(r: u8, g: u8, b: u8, amount: i16) -> Ink {
    rgb(
        (i16::from(r) + amount).clamp(0, 255) as u8,
        (i16::from(g) + amount).clamp(0, 255) as u8,
        (i16::from(b) + amount).clamp(0, 255) as u8,
    )
}

#[derive(Default)]
struct Painter {
    positions: Vec<[f32; 3]>,
    colors: Vec<Ink>,
    indices: Vec<u32>,
    groups: Option<Vec<PaintGroup>>,
}

/// Optional metadata for static partitioning; never becomes a mesh attribute.
/// Keep touching faces and details of one tile, tree, or object in source order.
struct PaintGroup {
    indices: std::ops::Range<usize>,
    layer: PaintLayer,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PaintLayer {
    Shadow,
    Terrain,
    Scenery,
    Object,
}

impl Painter {
    fn with_groups(record: bool) -> Self {
        Self {
            groups: record.then(Vec::new),
            ..default()
        }
    }

    fn end_group(&mut self, layer: PaintLayer) {
        let Some(groups) = &mut self.groups else {
            return;
        };
        let start = groups.last().map_or(0, |group| group.indices.end);
        if start < self.indices.len() {
            groups.push(PaintGroup {
                indices: start..self.indices.len(),
                layer,
            });
        }
    }

    fn triangle_3d(&mut self, points: [Vec3; 3], ink: Ink) {
        let first = self.positions.len() as u32;
        for point in points {
            self.positions.push(point.to_array());
            self.colors.push(ink);
        }
        self.indices.extend([first, first + 1, first + 2]);
    }

    fn triangle(&mut self, points: [Vec2; 3], z: f32, ink: Ink) {
        self.triangle_3d(points.map(|point| point.extend(z)), ink);
    }

    fn quad(&mut self, points: [Vec2; 4], z: f32, ink: Ink) {
        self.polygon(&points, z, ink);
    }

    fn polygon(&mut self, points: &[Vec2], z: f32, ink: Ink) {
        for index in 1..points.len() - 1 {
            self.triangle([points[0], points[index], points[index + 1]], z, ink);
        }
    }

    fn surface(&mut self, surface: Surface, ink: Ink) {
        let [a, b, c, d] = surface.vertices;
        self.triangle_3d([a, b, c], ink);
        self.triangle_3d([a, c, d], ink);
    }

    /// Clip details to their supporting face so a shadow or thick line
    /// cannot float off a cliff into the air above a lower neighboring tile.
    fn paint_on_surface(&mut self, surface: Surface, points: &[Vec2], bias: f32, ink: Ink) {
        let bounds = surface.vertices.map(Vec3::truncate);
        let orientation = (bounds[1] - bounds[0])
            .perp_dot(bounds[2] - bounds[1])
            .signum();
        let mut polygon = points.to_vec();
        for edge in 0..4 {
            let start = bounds[edge];
            let direction = bounds[(edge + 1) % 4] - start;
            let signed_distance = |point: Vec2| direction.perp_dot(point - start) * orientation;
            let source = std::mem::take(&mut polygon);
            if source.is_empty() {
                return;
            }
            let mut previous = *source.last().expect("source is not empty");
            let mut previous_distance = signed_distance(previous);
            for point in source {
                let distance = signed_distance(point);
                if (distance >= 0.0) != (previous_distance >= 0.0) {
                    let fraction = previous_distance / (previous_distance - distance);
                    polygon.push(previous.lerp(point, fraction));
                }
                if distance >= 0.0 {
                    polygon.push(point);
                }
                previous = point;
                previous_distance = distance;
            }
        }
        for index in 1..polygon.len().saturating_sub(1) {
            self.triangle_3d(
                [polygon[0], polygon[index], polygon[index + 1]]
                    .map(|point| surface.point(point, bias)),
                ink,
            );
        }
    }

    fn surface_line(
        &mut self,
        surface: Surface,
        start: Vec2,
        end: Vec2,
        width: f32,
        bias: f32,
        ink: Ink,
    ) {
        let delta = end - start;
        let side = Vec2::new(-delta.y, delta.x).normalize_or_zero() * width / 2.0;
        self.paint_on_surface(
            surface,
            &[start + side, start - side, end - side, end + side],
            bias,
            ink,
        );
    }

    fn surface_ellipse(
        &mut self,
        surface: Surface,
        center: Vec2,
        radius_x: f32,
        radius_y: f32,
        bias: f32,
        ink: Ink,
    ) {
        let points = ellipse_points(center, radius_x, radius_y);
        self.paint_on_surface(surface, &points, bias, ink);
    }

    fn surface_outline(&mut self, surface: Surface, width: f32, bias: f32, ink: Ink) {
        let points = surface.vertices.map(Vec3::truncate);
        // Thin deck edges need a smaller inset than a full terrain cliff.
        // Otherwise the inner corners cross and the outline fills its face.
        let shortest_edge = (0..4)
            .map(|index| points[index].distance(points[(index + 1) % 4]))
            .fold(f32::INFINITY, f32::min);
        let width = width.min(shortest_edge * 0.3);
        let center = points.iter().copied().sum::<Vec2>() / 4.0;
        let mut inner = [Vec2::ZERO; 4];
        for index in 0..4 {
            let previous = points[(index + 3) % 4];
            let corner = points[index];
            let next = points[(index + 1) % 4];
            let first = (corner - previous).normalize();
            let second = (next - corner).normalize();
            let inward = |direction: Vec2| {
                let normal = Vec2::new(-direction.y, direction.x);
                normal * normal.dot(center - corner).signum()
            };
            let a = corner + inward(first) * width;
            let b = corner + inward(second) * width;
            inner[index] = a + first * ((b - a).perp_dot(second) / first.perp_dot(second));
        }
        // The outline stays inside the face, rather than projecting a stroke
        // beyond a hidden tile and leaving a halo around its front occluder.
        for index in 0..4 {
            let next = (index + 1) % 4;
            self.paint_on_surface(
                surface,
                &[points[index], points[next], inner[next], inner[index]],
                bias,
                ink,
            );
        }
    }

    fn line(&mut self, start: Vec2, end: Vec2, width: f32, z: f32, ink: Ink) {
        let delta = end - start;
        let side = Vec2::new(-delta.y, delta.x).normalize_or_zero() * width / 2.0;
        self.quad([start + side, start - side, end - side, end + side], z, ink);
    }

    fn ellipse(&mut self, center: Vec2, radius_x: f32, radius_y: f32, z: f32, ink: Ink) {
        let points = ellipse_points(center, radius_x, radius_y);
        self.polygon(&points, z, ink);
    }

    fn finish(self) -> Mesh {
        let count = self.positions.len();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

fn ellipse_points(center: Vec2, radius_x: f32, radius_y: f32) -> [Vec2; 12] {
    std::array::from_fn(|index| {
        let angle = index as f32 * std::f32::consts::TAU / 12.0;
        center + Vec2::new(angle.cos() * radius_x, angle.sin() * radius_y)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use park_sim::{MAX_TERRAIN_HEIGHT, ObjectId, TerrainProfile};

    #[test]
    fn every_tile_round_trips_through_every_rotation() {
        for size in [1, 2, 7, 32] {
            let park = Park::new(size, size);
            for rotation in 0..4 {
                for y in 0..size {
                    for x in 0..size {
                        let pos = TilePos { x, y };
                        let center = tile_center(pos, size, rotation);
                        for offset in [Vec2::ZERO, Vec2::new(15.0, 3.0), Vec2::new(-14.0, -4.0)] {
                            assert_eq!(pick_tile(center + offset, &park, rotation), Some(pos));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn known_corners_and_clockwise_rotation_match() {
        let first = TilePos { x: 0, y: 0 };
        assert_eq!(tile_center(first, 32, 0), Vec2::new(0.0, 496.0));
        assert_eq!(tile_center(first, 32, 1), Vec2::new(992.0, 0.0));
        assert_eq!(tile_center(first, 32, 2), Vec2::new(0.0, -496.0));
        assert_eq!(tile_center(first, 32, 3), Vec2::new(-992.0, 0.0));
        assert_eq!(tile_center(first, 32, 4), tile_center(first, 32, 0));
    }

    #[test]
    fn outside_diamond_and_non_finite_points_never_select_tiles() {
        let park = Park::new(32, 32);
        for rotation in 0..4 {
            for point in [
                Vec2::new(0.0, 512.1),
                Vec2::new(0.0, -540.0),
                Vec2::new(1024.1, 0.0),
                Vec2::new(-1024.1, 0.0),
                Vec2::new(600.0, 400.0),
                Vec2::new(f32::NAN, 0.0),
                Vec2::new(0.0, f32::INFINITY),
            ] {
                assert_eq!(pick_tile(point, &park, rotation), None);
            }
        }
        assert_eq!(pick_tile(Vec2::ZERO, &Park::new(7, 3), 0), None);
    }

    #[test]
    fn shared_half_tile_edges_have_consistent_ownership() {
        let left = TilePos { x: 5, y: 6 };
        let right = TilePos { x: 6, y: 6 };
        let park = Park::new(32, 32);
        for rotation in 0..4 {
            let a = tile_center(left, 32, rotation);
            let b = tile_center(right, 32, rotation);
            let edge = (a + b) / 2.0;
            let direction = (b - a).normalize();
            assert_eq!(
                pick_tile(edge - direction * 0.01, &park, rotation),
                Some(left)
            );
            assert_eq!(
                pick_tile(edge + direction * 0.01, &park, rotation),
                Some(right)
            );
            let on_edge = pick_tile(edge, &park, rotation);
            assert!(on_edge == Some(left) || on_edge == Some(right));
        }
    }

    #[test]
    fn disabling_grid_removes_geometry_and_selection_is_separate() {
        let park = Park::new(12, 12);
        let grid = build_scene(&park, 0, true, ViewMode::Surface);
        let plain = build_scene(&park, 0, false, ViewMode::Surface);
        assert!(grid.count_vertices() > plain.count_vertices());
        assert_eq!(
            selection_mesh(None, None, &park, 0, ViewMode::Surface).count_vertices(),
            0
        );
        assert!(
            selection_mesh(
                Some(SurfaceId::Terrain(TilePos { x: 3, y: 3 })),
                None,
                &park,
                0,
                ViewMode::Surface
            )
            .count_vertices()
                > 0
        );
    }

    #[test]
    fn roofs_and_both_front_cliffs_return_their_own_column_in_every_view() {
        for rotation in 0..4 {
            let mut park = Park::new(12, 12);
            let pos = unrotate(6, 6, 12, rotation);
            park.set_terrain_height(pos, 6).unwrap();
            let center = elevated_tile_center(&park, pos, rotation).unwrap();
            assert_eq!(pick_tile(center, &park, rotation), Some(pos));
            for offset in [Vec2::new(-16.0, -56.0), Vec2::new(16.0, -56.0)] {
                assert_eq!(pick_tile(center + offset, &park, rotation), Some(pos));
            }
            let surfaces = column_surfaces(&park, pos, rotation);
            assert!(surfaces.into_iter().all(|face| face.is_some()));
        }
    }

    #[test]
    fn nearer_cliff_hides_the_low_roof_in_every_view() {
        for rotation in 0..4 {
            let mut park = Park::new(12, 12);
            let low = unrotate(5, 5, 12, rotation);
            let high = unrotate(6, 6, 12, rotation);
            park.set_terrain_height(high, 6).unwrap();
            let low_center = elevated_tile_center(&park, low, rotation).unwrap();
            for offset in [Vec2::ZERO, Vec2::new(-8.0, 0.0), Vec2::new(8.0, 0.0)] {
                assert_eq!(pick_tile(low_center + offset, &park, rotation), Some(high));
            }
        }
    }

    #[test]
    fn equal_height_neighbors_do_not_emit_internal_cliffs() {
        let mut park = Park::new(12, 12);
        for rotation in 0..4 {
            for (x, y) in [(5, 5), (6, 5), (5, 6)] {
                park.set_terrain_height(unrotate(x, y, 12, rotation), 4)
                    .unwrap();
            }
            let surfaces = column_surfaces(&park, unrotate(5, 5, 12, rotation), rotation);
            assert!(surfaces[0].is_some());
            assert!(surfaces[1].is_none());
            assert!(surfaces[2].is_none());
        }
    }

    #[test]
    fn height_limits_invalid_coordinates_and_all_rotations_remain_finite() {
        let mut park = Park::new(12, 12);
        let pos = TilePos { x: 5, y: 5 };
        park.set_terrain_height(pos, MAX_TERRAIN_HEIGHT).unwrap();
        assert!(
            park.set_terrain_height(pos, MAX_TERRAIN_HEIGHT + 1)
                .is_err()
        );
        for rotation in 0..4 {
            let center = elevated_tile_center(&park, pos, rotation).unwrap();
            assert_eq!(pick_tile(center, &park, rotation), Some(pos));
            assert_eq!(
                center.y - tile_center(pos, 12, rotation).y,
                f32::from(MAX_TERRAIN_HEIGHT) * HEIGHT_STEP
            );
            let invalid = TilePos { x: 12, y: u32::MAX };
            assert!(elevated_tile_center(&park, invalid, rotation).is_none());
            assert_eq!(
                selection_mesh(
                    Some(SurfaceId::Terrain(invalid)),
                    Some(SurfaceId::Terrain(invalid)),
                    &park,
                    rotation,
                    ViewMode::Surface
                )
                .count_vertices(),
                0
            );
            let mesh = build_scene(&park, rotation, true, ViewMode::Surface);
            for point in positions(&mesh) {
                assert!(point.iter().all(|coordinate| coordinate.is_finite()));
                assert!(point[2].abs() < 1000.0, "inside Camera2d clipping range");
            }
        }
    }

    /// Independent triangle interpolation checks the emitted mesh, rather
    /// than just calling the same surface hit-test used by the picker.
    fn mesh_depth(mesh: &Mesh, point: Vec2) -> Option<f32> {
        let points = positions(mesh);
        let indices: Vec<usize> = mesh.indices().unwrap().iter().collect();
        indices
            .chunks_exact(3)
            .filter_map(|triangle| {
                let [a, b, c] =
                    [triangle[0], triangle[1], triangle[2]].map(|index| Vec3::from(points[index]));
                let ab = b.truncate() - a.truncate();
                let ac = c.truncate() - a.truncate();
                let ap = point - a.truncate();
                let determinant = ab.perp_dot(ac);
                if determinant.abs() < 0.00001 {
                    return None;
                }
                let u = ap.perp_dot(ac) / determinant;
                let v = ab.perp_dot(ap) / determinant;
                (u >= -0.00001 && v >= -0.00001 && u + v <= 1.00001)
                    .then_some(a.z + u * (b.z - a.z) + v * (c.z - a.z))
            })
            .max_by(f32::total_cmp)
    }

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            VertexAttributeValues::Float32x3(positions) => positions,
            _ => panic!("mesh position format changed"),
        }
    }

    #[test]
    fn rendered_triangles_and_selection_obey_the_same_occlusion() {
        for rotation in 0..4 {
            let mut park = Park::new(12, 12);
            let low = unrotate(5, 5, 12, rotation);
            let high = unrotate(6, 6, 12, rotation);
            park.set_terrain_height(high, 6).unwrap();
            let low_center = elevated_tile_center(&park, low, rotation).unwrap();
            let scene = build_scene(&park, rotation, false, ViewMode::Surface);
            let low_outline = selection_mesh(
                None,
                Some(SurfaceId::Terrain(low)),
                &park,
                rotation,
                ViewMode::Surface,
            );
            // This lies inside the low roof's top border, behind the cliff.
            let hidden = low_center + Vec2::new(0.0, 14.0);
            let terrain_depth = mesh_depth(&scene, hidden).unwrap();
            let outline_depth = mesh_depth(&low_outline, hidden).unwrap();
            assert!(terrain_depth > outline_depth + 1.0);
            assert_eq!(pick_tile(hidden, &park, rotation), Some(high));

            let high_roof = column_surfaces(&park, high, rotation)[0].unwrap();
            let visible =
                elevated_tile_center(&park, high, rotation).unwrap() + Vec2::new(0.0, 14.0);
            let high_outline = selection_mesh(
                None,
                Some(SurfaceId::Terrain(high)),
                &park,
                rotation,
                ViewMode::Surface,
            );
            let outline_depth = mesh_depth(&high_outline, visible).unwrap();
            assert!((outline_depth - high_roof.depth_at(visible) - SELECTION_BIAS).abs() < 0.0001);
            assert!(outline_depth > mesh_depth(&scene, visible).unwrap());
            // No fixed global overlay depth: every emitted selection vertex
            // belongs to an actual face and differs only by the small bias.
            for point in positions(&high_outline) {
                let point = Vec3::from(*point);
                assert!(
                    column_surfaces(&park, high, rotation)
                        .into_iter()
                        .flatten()
                        .any(
                            |face| face
                                .hit_depth(point.truncate())
                                .is_some_and(
                                    |depth| (point.z - depth - SELECTION_BIAS).abs() < 0.0001
                                )
                        )
                );
            }
        }
    }

    #[test]
    fn buried_track_is_hidden_until_cutaway_and_context_has_no_phantom_hits() {
        let mut park = Park::new(12, 12);
        let pos = TilePos { x: 5, y: 5 };
        park.set_terrain_height(pos, 4).unwrap();
        let id = SurfaceId::Object(
            park.place_object(pos, ObjectKind::Track, Axis::Y, 0, 2)
                .unwrap(),
        );
        for rotation in 0..4 {
            let center = surface_center(&park, id, rotation).unwrap();
            assert_ne!(
                pick_surface(center, &park, rotation, ViewMode::Surface),
                Some(id)
            );
            assert_eq!(
                selection_mesh(None, Some(id), &park, rotation, ViewMode::Surface).count_vertices(),
                0,
            );
            assert_eq!(
                pick_surface(center, &park, rotation, ViewMode::Underground),
                Some(id),
            );
            let visible = selection_mesh(None, Some(id), &park, rotation, ViewMode::Underground);
            assert!(visible.count_vertices() > 0);
            let scene = build_scene(&park, rotation, true, ViewMode::Underground);
            let top = selectable_surfaces(&park, id, rotation, ViewMode::Underground)[0];
            assert!((mesh_depth(&scene, center).unwrap() - top.depth_at(center)).abs() < 0.01);
            let ground = tile_center(TilePos { x: 1, y: 1 }, 12, rotation);
            assert_eq!(
                pick_surface(ground, &park, rotation, ViewMode::Underground),
                None
            );
            assert_eq!(
                selection_mesh(
                    None,
                    Some(SurfaceId::Terrain(pos)),
                    &park,
                    rotation,
                    ViewMode::Underground
                )
                .count_vertices(),
                0,
            );
        }
    }

    #[test]
    fn overlapping_path_hides_track_and_its_outline_with_matching_mesh_depth() {
        for rotation in 0..4 {
            for axis in [Axis::X, Axis::Y] {
                let mut park = Park::new(12, 12);
                let track = park
                    .place_object(unrotate(4, 4, 12, rotation), ObjectKind::Track, axis, 0, 2)
                    .unwrap();
                let path = park
                    .place_object(unrotate(7, 7, 12, rotation), ObjectKind::Path, axis, 6, 2)
                    .unwrap();
                let track_object = park.object(track).unwrap();
                // Their projections overlap: six height steps cancel the
                // six-ground-step difference toward the camera.
                let center = surface_center(&park, SurfaceId::Object(track), rotation).unwrap();
                let border = object_point(
                    track_object,
                    0.0,
                    0.27,
                    object_top(track_object),
                    12,
                    rotation,
                );
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    assert_eq!(
                        pick_surface(center, &park, rotation, mode),
                        Some(SurfaceId::Object(path))
                    );
                    assert_eq!(
                        pick_surface(border, &park, rotation, mode),
                        Some(SurfaceId::Object(path))
                    );
                    let scene = build_scene(&park, rotation, false, mode);
                    let outline =
                        selection_mesh(None, Some(SurfaceId::Object(track)), &park, rotation, mode);
                    assert!(
                        mesh_depth(&scene, border).unwrap()
                            > mesh_depth(&outline, border).unwrap() + 5.0
                    );
                    let path_top = object_surfaces(&park, park.object(path).unwrap(), rotation)[0];
                    assert!(
                        (mesh_depth(&scene, center).unwrap() - path_top.depth_at(center)).abs()
                            < 0.01
                    );
                    let path_outline =
                        selection_mesh(None, Some(SurfaceId::Object(path)), &park, rotation, mode);
                    for point in positions(&path_outline) {
                        let point = Vec3::from(*point);
                        assert!(point.is_finite());
                        assert!(
                            object_surfaces(&park, park.object(path).unwrap(), rotation)
                                .into_iter()
                                .any(|face| {
                                    face.hit_depth(point.truncate()).is_some_and(|depth| {
                                        (point.z - depth - SELECTION_BIAS).abs() < 0.0001
                                    })
                                })
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn ground_track_covers_terrain_but_its_narrow_sides_leave_land_selectable() {
        let pos = TilePos { x: 5, y: 5 };
        for rotation in 0..4 {
            for axis in [Axis::X, Axis::Y] {
                let mut park = Park::new(12, 12);
                let id = park
                    .place_object(pos, ObjectKind::Track, axis, 0, 2)
                    .unwrap();
                let object = park.object(id).unwrap();
                let center = surface_center(&park, SurfaceId::Object(id), rotation).unwrap();
                assert_eq!(
                    pick_surface(center, &park, rotation, ViewMode::Surface),
                    Some(SurfaceId::Object(id)),
                );
                let outside = object_point(object, 0.0, 0.45, 0.0, 12, rotation);
                assert_eq!(
                    pick_surface(outside, &park, rotation, ViewMode::Surface),
                    Some(SurfaceId::Terrain(pos)),
                );
                let scene = build_scene(&park, rotation, false, ViewMode::Surface);
                let object_depth = object_surfaces(&park, object, rotation)[0].depth_at(center);
                assert!((mesh_depth(&scene, center).unwrap() - object_depth).abs() < 0.01);
            }
        }
    }

    #[test]
    fn adjacent_decks_keep_boundary_ownership_and_front_faces_select_their_owner() {
        for rotation in 0..4 {
            let mut park = Park::new(12, 12);
            let first = park
                .place_object(TilePos { x: 4, y: 5 }, ObjectKind::Path, Axis::X, 4, 2)
                .unwrap();
            let second = park
                .place_object(TilePos { x: 5, y: 5 }, ObjectKind::Path, Axis::X, 4, 2)
                .unwrap();
            let a = surface_center(&park, SurfaceId::Object(first), rotation).unwrap();
            let b = surface_center(&park, SurfaceId::Object(second), rotation).unwrap();
            let edge = (a + b) * 0.5;
            let direction = (b - a).normalize();
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                assert_eq!(
                    pick_surface(edge - direction * 0.01, &park, rotation, mode),
                    Some(SurfaceId::Object(first))
                );
                assert_eq!(
                    pick_surface(edge + direction * 0.01, &park, rotation, mode),
                    Some(SurfaceId::Object(second))
                );
                let at_edge = pick_surface(edge, &park, rotation, mode);
                assert!(
                    at_edge == Some(SurfaceId::Object(first))
                        || at_edge == Some(SurfaceId::Object(second))
                );
            }
            // Remove the neighbor so both front faces are actually exposed.
            park.remove_object(second).unwrap();
            let scene = build_scene(&park, rotation, false, ViewMode::Surface);
            for face in object_surfaces(&park, park.object(first).unwrap(), rotation)
                .into_iter()
                .skip(1)
            {
                let center = face
                    .vertices
                    .iter()
                    .map(|point| point.truncate())
                    .sum::<Vec2>()
                    * 0.25;
                assert_eq!(
                    pick_surface(center, &park, rotation, ViewMode::Surface),
                    Some(SurfaceId::Object(first))
                );
                assert!(
                    (mesh_depth(&scene, center).unwrap() - face.depth_at(center)).abs() < 0.0001
                );
            }
        }
    }

    #[test]
    fn fixture_identity_and_exposure_follow_presets_without_replacing_objects() {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_overlap_fixture().unwrap();
        let track = *park
            .objects()
            .iter()
            .find(|object| {
                object.kind == ObjectKind::Track && object.pos == TilePos { x: 13, y: 16 }
            })
            .unwrap();
        let id = SurfaceId::Object(track.id);
        assert!(park.is_underground(track.id));
        for rotation in 0..4 {
            assert!(selectable_surfaces(&park, id, rotation, ViewMode::Surface).is_empty());
            assert_eq!(
                selectable_surfaces(&park, id, rotation, ViewMode::Underground).len(),
                3
            );
        }
        park.apply_terrain_profile(TerrainProfile::Flat);
        assert_eq!(park.object(track.id), Some(&track));
        assert!(!park.is_underground(track.id));
        for rotation in 0..4 {
            assert_eq!(
                selectable_surfaces(&park, id, rotation, ViewMode::Surface).len(),
                3
            );
        }
    }

    #[test]
    fn missing_or_removed_objects_never_render_selectable_faces() {
        let mut park = Park::new(12, 12);
        let removed = park
            .place_object(TilePos { x: 5, y: 5 }, ObjectKind::Path, Axis::X, 6, 2)
            .unwrap();
        park.remove_object(removed).unwrap();
        for id in [
            SurfaceId::Object(removed),
            SurfaceId::Object(ObjectId(u64::MAX)),
        ] {
            for rotation in 0..4 {
                assert_eq!(surface_center(&park, id, rotation), None);
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    assert!(selectable_surfaces(&park, id, rotation, mode).is_empty());
                    assert_eq!(
                        selection_mesh(Some(id), Some(id), &park, rotation, mode).count_vertices(),
                        0
                    );
                }
            }
        }
    }

    #[test]
    fn junction_decks_and_picking_reach_only_connected_same_height_edges() {
        let mut park = Park::new(12, 12);
        let center_id = park
            .place_object(TilePos { x: 5, y: 5 }, ObjectKind::Path, Axis::X, 4, 2)
            .unwrap();
        let stacked = park
            .place_object(TilePos { x: 5, y: 6 }, ObjectKind::Path, Axis::Y, 8, 2)
            .unwrap();
        let track = park
            .place_object(TilePos { x: 5, y: 4 }, ObjectKind::Track, Axis::Y, 4, 2)
            .unwrap();
        assert_eq!(
            object_bounds(&park, park.object(center_id).unwrap()),
            (-0.5, 0.5, -0.34, 0.34),
        );
        park.remove_object(stacked).unwrap();
        park.remove_object(track).unwrap();
        let mut cross_neighbors = Vec::new();
        for pos in [
            TilePos { x: 4, y: 5 },
            TilePos { x: 6, y: 5 },
            TilePos { x: 5, y: 4 },
            TilePos { x: 5, y: 6 },
        ] {
            cross_neighbors.push(
                park.place_object(pos, ObjectKind::Path, Axis::Y, 4, 2)
                    .unwrap(),
            );
        }
        assert_eq!(park.path_neighbors(center_id).len(), 4);
        assert_eq!(
            object_bounds(&park, park.object(center_id).unwrap()),
            (-0.5, 0.5, -0.5, 0.5),
        );
        for rotation in 0..4 {
            for across in [-0.45, 0.45] {
                let center = park.object(center_id).unwrap();
                let point = object_point(center, 0.0, across, object_top(center), 12, rotation);
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    assert_eq!(
                        pick_surface(point, &park, rotation, mode),
                        Some(SurfaceId::Object(center_id)),
                    );
                    let scene = build_scene(&park, rotation, false, mode);
                    let roof = object_surfaces(&park, center, rotation)[0];
                    assert!(
                        (mesh_depth(&scene, point).unwrap() - roof.depth_at(point)).abs() < 0.01,
                    );
                }
            }
        }
        // Deleting one branch closes precisely that cross edge, without
        // changing the rest of the junction's selectable owner.
        park.remove_object(cross_neighbors[3]).unwrap();
        assert_eq!(
            object_bounds(&park, park.object(center_id).unwrap()),
            (-0.5, 0.5, -0.5, 0.34),
        );
    }

    #[test]
    fn guest_mesh_interpolates_on_path_roofs_in_every_rotation_and_view() {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(1234).unwrap();
        for _ in 0..17 {
            park.advance(park_sim::Speed::Normal);
        }
        assert!(park.guests().iter().any(|guest| guest.progress > 0));
        for rotation in 0..4 {
            for guest in park.guests() {
                let position = park.guest_position(guest.id).unwrap();
                let fraction = f32::from(position.progress) / f32::from(position.steps);
                let from = tile_center(position.from, 32, rotation);
                let to = tile_center(position.to, 32, rotation);
                let expected = from.lerp(to, fraction)
                    + Vec2::Y * ((f32::from(position.height) + 0.22) * HEIGHT_STEP);
                let base = guest_anchor(&park, guest.id, rotation).unwrap();
                assert!(base.truncate().distance(expected) < 0.0001);
                assert!(base.z < 1000.0);
            }
            let surface = guest_mesh(&park, rotation, ViewMode::Surface);
            let underground = guest_mesh(&park, rotation, ViewMode::Underground);
            assert!(surface.count_vertices() > 0);
            assert_eq!(positions(&surface), positions(&underground));
            assert!(
                positions(&surface)
                    .iter()
                    .flatten()
                    .all(|value| value.is_finite())
            );
        }
    }

    #[test]
    fn guest_billboards_use_physical_depth_and_are_hidden_by_nearer_terrain() {
        for rotation in 0..4 {
            let mut park = Park::new(32, 32);
            park.install_crowd_fixture(9).unwrap();
            let guest = park.guests()[0].clone();
            let base = guest_anchor(&park, guest.id, rotation).unwrap();
            let sample = base.truncate() + Vec2::new(0.0, 12.0);
            let people = guest_mesh(&park, rotation, ViewMode::Surface);
            let person_depth = mesh_depth(&people, sample).unwrap();
            assert!(person_depth >= base.z + 0.04);
            // Put an empty terrain column toward the camera, at an elevation
            // whose roof overlaps this person's body in the projected view.
            let (u, v) = rotate(guest.anchor, 32, rotation);
            let (distance, foreground) = (2..=10)
                .filter(|distance| u + distance < 32 && v + distance < 32)
                .map(|distance| (distance, unrotate(u + distance, v + distance, 32, rotation)))
                .find(|(_, pos)| park.objects().iter().all(|object| object.pos != *pos))
                .unwrap();
            park.set_terrain_height(foreground, guest.height + (2 * distance + 1) as u8)
                .unwrap();
            let terrain = build_scene(&park, rotation, false, ViewMode::Surface);
            assert!(mesh_depth(&terrain, sample).unwrap() > person_depth + 1.0);
        }
    }

    #[test]
    fn lost_guests_remain_visible_at_their_anchor_without_deleted_path_references() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(42).unwrap();
        let paths: Vec<_> = park
            .objects()
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .map(|object| object.id)
            .collect();
        for path in paths {
            park.remove_object(path).unwrap();
        }
        assert_eq!(park.crowd_stats().lost, 12);
        for rotation in 0..4 {
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                let mesh = guest_mesh(&park, rotation, mode);
                assert!(mesh.count_vertices() > 0);
                assert!(
                    positions(&mesh)
                        .iter()
                        .flatten()
                        .all(|value| value.is_finite())
                );
                let colors = match mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap() {
                    VertexAttributeValues::Float32x4(colors) => colors,
                    _ => panic!("mesh color format changed"),
                };
                assert!(colors.contains(&rgb(235, 112, 44)));
            }
        }
    }
}
