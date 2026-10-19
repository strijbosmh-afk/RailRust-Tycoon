//! Original, asset-free isometric scenery and the matching picking transform.

use bevy::{
    asset::RenderAssetUsages, color::ColorToComponents, mesh::Indices, prelude::*,
    render::render_resource::PrimitiveTopology,
};
use park_sim::{Park, Terrain, TilePos};

pub const TILE_WIDTH: f32 = 64.0;
pub const TILE_HEIGHT: f32 = 32.0;
const HALF_WIDTH: f32 = TILE_WIDTH / 2.0;
const HALF_HEIGHT: f32 = TILE_HEIGHT / 2.0;
type Ink = [f32; 4];

/// Project a simulation tile into the centered, clockwise-rotated map.
pub fn tile_center(pos: TilePos, size: u32, rotation: u8) -> Vec2 {
    let (x, y) = rotate(pos, size, rotation);
    Vec2::new(
        (x as f32 - y as f32) * HALF_WIDTH,
        -(x as f32 + y as f32 - (size as f32 - 1.0)) * HALF_HEIGHT,
    )
}

/// Pick the diamond under a world point. Shared edges belong to one tile only.
pub fn pick_tile(world: Vec2, size: u32, rotation: u8) -> Option<TilePos> {
    if size == 0 || !world.is_finite() {
        return None;
    }
    let column = world.x / HALF_WIDTH;
    let row = size as f32 - 1.0 - world.y / HALF_HEIGHT;
    let x = ((row + column) / 2.0 + 0.5).floor();
    let y = ((row - column) / 2.0 + 0.5).floor();
    if x < 0.0 || y < 0.0 || x >= size as f32 || y >= size as f32 {
        return None;
    }
    let (x, y) = (x as u32, y as u32);
    let last = size - 1;
    Some(match rotation % 4 {
        0 => TilePos { x, y },
        1 => TilePos { x: y, y: last - x },
        2 => TilePos {
            x: last - x,
            y: last - y,
        },
        _ => TilePos { x: last - y, y: x },
    })
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

/// Build terrain, soil edges, trees, and a decorative entrance in one mesh.
/// The material should be opaque white so per-vertex colors and depth apply.
pub fn build_scene(park: &Park, rotation: u8, grid: bool) -> Mesh {
    assert_eq!(
        park.width(),
        park.height(),
        "isometric scene expects a square park"
    );
    let size = park.width();
    let mut mesh = Painter::default();
    let half_x = size as f32 * HALF_WIDTH;
    let half_y = size as f32 * HALF_HEIGHT;
    let top = Vec2::new(0.0, half_y);
    let right = Vec2::new(half_x, 0.0);
    let bottom = Vec2::new(0.0, -half_y);
    let left = Vec2::new(-half_x, 0.0);
    let drop = Vec2::new(0.0, -27.0);

    // A low, deliberately crisp shadow makes the whole map read as a diorama.
    mesh.polygon(
        &[
            top + Vec2::new(10.0, -12.0),
            left + Vec2::new(-7.0, -20.0),
            bottom + Vec2::new(10.0, -45.0),
            right + Vec2::new(18.0, -20.0),
        ],
        10.0,
        rgb(24, 46, 48),
    );
    mesh.quad(
        [left, bottom, bottom + drop, left + drop],
        11.0,
        rgb(126, 94, 61),
    );
    mesh.quad(
        [bottom, right, right + drop, bottom + drop],
        11.0,
        rgb(102, 76, 52),
    );
    mesh.line(
        left + drop * 0.65,
        bottom + drop * 0.65,
        2.0,
        11.1,
        rgb(147, 111, 71),
    );
    mesh.line(
        bottom + drop * 0.65,
        right + drop * 0.65,
        2.0,
        11.1,
        rgb(118, 89, 58),
    );

    // All terrain sits on the same plane. Ground details occupy a slight inset
    // above it; depth-sorted scenery starts above those details.
    for y in 0..size {
        for x in 0..size {
            let pos = TilePos { x, y };
            let tile = park.tile(pos).expect("loop stays inside the park");
            let center = tile_center(pos, size, rotation);
            let variation = (noise(pos) % 5) as i16 - 2;
            let color = match tile.terrain {
                Terrain::Grass => varied(108, 153, 88, variation),
                Terrain::Water => varied(74, 157, 165, variation),
                Terrain::Sand => varied(213, 196, 145, variation),
            };
            mesh.diamond(center, 1.0, 15.0, color);

            match tile.terrain {
                Terrain::Water => {
                    if !noise(pos).is_multiple_of(3) {
                        let offset = (noise(pos) % 11) as f32 - 5.0;
                        mesh.line(
                            center + Vec2::new(-11.0, offset),
                            center + Vec2::new(8.0, offset),
                            1.1,
                            16.0,
                            rgb(133, 197, 192),
                        );
                        mesh.line(
                            center + Vec2::new(2.0, offset + 4.0),
                            center + Vec2::new(9.0, offset + 4.0),
                            0.8,
                            16.0,
                            rgb(105, 179, 181),
                        );
                    }
                }
                Terrain::Grass if noise(pos).is_multiple_of(17) => {
                    let tuft = center + Vec2::new(-3.0, -1.0);
                    mesh.line(
                        tuft,
                        tuft + Vec2::new(-2.0, 3.0),
                        0.9,
                        16.0,
                        rgb(132, 173, 105),
                    );
                    mesh.line(
                        tuft + Vec2::new(3.0, 0.0),
                        tuft + Vec2::new(4.0, 4.0),
                        0.9,
                        16.0,
                        rgb(132, 173, 105),
                    );
                }
                Terrain::Sand if noise(pos).is_multiple_of(5) => {
                    mesh.ellipse(
                        center + Vec2::new(4.0, 1.0),
                        2.4,
                        1.2,
                        16.0,
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
                mesh.line(
                    center + Vec2::Y * HALF_HEIGHT,
                    center + Vec2::X * HALF_WIDTH,
                    0.65,
                    16.5,
                    stroke,
                );
                mesh.line(
                    center + Vec2::X * HALF_WIDTH,
                    center - Vec2::Y * HALF_HEIGHT,
                    0.65,
                    16.5,
                    stroke,
                );
            }
        }
    }

    mesh.line(left, bottom, 2.0, 17.0, rgb(164, 182, 103));
    mesh.line(bottom, right, 2.0, 17.0, rgb(145, 164, 91));

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
        let base = tile_center(pos, size, rotation);
        let depth = 30.0 + (half_y - base.y) / (2.0 * half_y) * 50.0;
        tree(&mut mesh, base, depth, noise(pos));
    }

    if size >= 12 {
        let entrance = TilePos { x: 1, y: size / 2 };
        let base = tile_center(entrance, size, rotation);
        let depth = 30.0 + (half_y - base.y) / (2.0 * half_y) * 50.0;
        entrance_arch(&mut mesh, base, depth);
    }
    mesh.finish()
}

/// Build independent hover and selection outlines, always above scenery.
pub fn selection_mesh(
    hover: Option<TilePos>,
    selected: Option<TilePos>,
    size: u32,
    rotation: u8,
) -> Mesh {
    let mut mesh = Painter::default();
    if let Some(pos) = hover.filter(|pos| pos.x < size && pos.y < size) {
        mesh.outline(
            tile_center(pos, size, rotation),
            1.4,
            200.0,
            rgb(247, 246, 216),
        );
    }
    if let Some(pos) = selected.filter(|pos| pos.x < size && pos.y < size) {
        let center = tile_center(pos, size, rotation);
        mesh.outline(center, 3.0, 202.0, rgb(255, 205, 96));
        for point in diamond_points(center, 1.0) {
            mesh.ellipse(point, 2.3, 2.3, 202.1, rgb(255, 225, 148));
        }
    }
    mesh.finish()
}

fn tree(mesh: &mut Painter, base: Vec2, depth: f32, seed: u32) {
    let scale = 0.85 + (seed % 6) as f32 * 0.055;
    mesh.ellipse(
        base + Vec2::new(6.0, -1.0),
        18.0 * scale,
        7.0 * scale,
        18.0,
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
        depth + 0.1,
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
                depth + height / 100.0,
                ink,
            );
        }
    } else {
        mesh.ellipse(
            base + Vec2::new(1.0, 34.0) * scale,
            21.0 * scale,
            24.0 * scale,
            depth + 0.2,
            rgb(51, 113, 72),
        );
        mesh.ellipse(
            base + Vec2::new(-6.0, 40.0) * scale,
            17.0 * scale,
            20.0 * scale,
            depth + 0.3,
            rgb(69, 133, 76),
        );
        mesh.ellipse(
            base + Vec2::new(-7.0, 48.0) * scale,
            11.0 * scale,
            12.0 * scale,
            depth + 0.4,
            rgb(94, 153, 85),
        );
    }
}

fn entrance_arch(mesh: &mut Painter, base: Vec2, depth: f32) {
    mesh.ellipse(
        base + Vec2::new(4.0, -1.0),
        36.0,
        10.0,
        18.0,
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
            depth + 0.1,
            rgb(184, 170, 133),
        );
        mesh.ellipse(
            foot + Vec2::new(0.0, 49.0),
            5.0,
            4.0,
            depth + 0.2,
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
        depth + 0.3,
        rgb(182, 82, 65),
    );
    mesh.line(
        base + Vec2::new(-30.0, 44.0),
        base + Vec2::new(30.0, 44.0),
        2.0,
        depth + 0.4,
        rgb(235, 176, 117),
    );
    // A small sun medallion is an original graphic, without borrowed branding.
    mesh.ellipse(
        base + Vec2::new(0.0, 41.0),
        6.5,
        6.5,
        depth + 0.5,
        rgb(250, 215, 141),
    );
    mesh.line(
        base + Vec2::new(0.0, 48.0),
        base + Vec2::new(0.0, 67.0),
        1.5,
        depth + 0.6,
        rgb(225, 212, 173),
    );
    mesh.triangle(
        [
            base + Vec2::new(0.0, 67.0),
            base + Vec2::new(15.0, 62.0),
            base + Vec2::new(0.0, 57.0),
        ],
        depth + 0.7,
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

fn diamond_points(center: Vec2, scale: f32) -> [Vec2; 4] {
    [
        center + Vec2::new(0.0, HALF_HEIGHT) * scale,
        center + Vec2::new(-HALF_WIDTH, 0.0) * scale,
        center + Vec2::new(0.0, -HALF_HEIGHT) * scale,
        center + Vec2::new(HALF_WIDTH, 0.0) * scale,
    ]
}

#[derive(Default)]
struct Painter {
    positions: Vec<[f32; 3]>,
    colors: Vec<Ink>,
    indices: Vec<u32>,
}

impl Painter {
    fn triangle(&mut self, points: [Vec2; 3], z: f32, ink: Ink) {
        let first = self.positions.len() as u32;
        for point in points {
            self.positions.push([point.x, point.y, z]);
            self.colors.push(ink);
        }
        self.indices.extend([first, first + 1, first + 2]);
    }

    fn quad(&mut self, points: [Vec2; 4], z: f32, ink: Ink) {
        self.polygon(&points, z, ink);
    }

    fn polygon(&mut self, points: &[Vec2], z: f32, ink: Ink) {
        for index in 1..points.len() - 1 {
            self.triangle([points[0], points[index], points[index + 1]], z, ink);
        }
    }

    fn diamond(&mut self, center: Vec2, scale: f32, z: f32, ink: Ink) {
        self.quad(diamond_points(center, scale), z, ink);
    }

    fn line(&mut self, start: Vec2, end: Vec2, width: f32, z: f32, ink: Ink) {
        let delta = end - start;
        let side = Vec2::new(-delta.y, delta.x).normalize_or_zero() * width / 2.0;
        self.quad([start + side, start - side, end - side, end + side], z, ink);
    }

    fn ellipse(&mut self, center: Vec2, radius_x: f32, radius_y: f32, z: f32, ink: Ink) {
        let points: Vec<Vec2> = (0..12)
            .map(|index| {
                let angle = index as f32 * std::f32::consts::TAU / 12.0;
                center + Vec2::new(angle.cos() * radius_x, angle.sin() * radius_y)
            })
            .collect();
        self.polygon(&points, z, ink);
    }

    fn outline(&mut self, center: Vec2, width: f32, z: f32, ink: Ink) {
        let points = diamond_points(center, 1.0);
        for index in 0..4 {
            self.line(points[index], points[(index + 1) % 4], width, z, ink);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tile_round_trips_through_every_rotation() {
        for size in [1, 2, 7, 32] {
            for rotation in 0..4 {
                for y in 0..size {
                    for x in 0..size {
                        let pos = TilePos { x, y };
                        let center = tile_center(pos, size, rotation);
                        for offset in [Vec2::ZERO, Vec2::new(15.0, 3.0), Vec2::new(-14.0, -4.0)] {
                            assert_eq!(pick_tile(center + offset, size, rotation), Some(pos));
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
        for rotation in 0..4 {
            for point in [
                Vec2::new(0.0, 512.1),
                Vec2::new(0.0, -512.1),
                Vec2::new(1024.1, 0.0),
                Vec2::new(-1024.1, 0.0),
                Vec2::new(600.0, 400.0),
                Vec2::new(f32::NAN, 0.0),
                Vec2::new(0.0, f32::INFINITY),
            ] {
                assert_eq!(pick_tile(point, 32, rotation), None);
            }
        }
        assert_eq!(pick_tile(Vec2::ZERO, 0, 0), None);
    }

    #[test]
    fn shared_half_tile_edges_have_consistent_ownership() {
        let left = TilePos { x: 5, y: 6 };
        let right = TilePos { x: 6, y: 6 };
        for rotation in 0..4 {
            let a = tile_center(left, 32, rotation);
            let b = tile_center(right, 32, rotation);
            let edge = (a + b) / 2.0;
            let direction = (b - a).normalize();
            assert_eq!(pick_tile(edge - direction * 0.01, 32, rotation), Some(left));
            assert_eq!(
                pick_tile(edge + direction * 0.01, 32, rotation),
                Some(right)
            );
            let on_edge = pick_tile(edge, 32, rotation);
            assert!(on_edge == Some(left) || on_edge == Some(right));
        }
    }

    #[test]
    fn disabling_grid_removes_geometry_and_selection_is_separate() {
        let park = Park::new(12, 12);
        let grid = build_scene(&park, 0, true);
        let plain = build_scene(&park, 0, false);
        assert!(grid.count_vertices() > plain.count_vertices());
        assert_eq!(selection_mesh(None, None, 12, 0).count_vertices(), 0);
        assert!(selection_mesh(Some(TilePos { x: 3, y: 3 }), None, 12, 0).count_vertices() > 0);
    }
}
