//! Deterministic miniature rails from the simulation's sampled track geometry.
//! Thumbnails use signed coordinates relative to the design anchor, so a shape
//! remains visible even when its chosen rotation cannot fit a particular park.

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use park_sim::{TRACK_UNITS, TrackDesign, TrackKind};

// Bound allocation and raster work independently of caller-supplied dimensions.
const MAX_SIDE: u32 = 512;
const SUPERSAMPLE: u32 = 2;
type Ink = [u8; 3];
const RAIL: Ink = [36, 83, 63];
const SLEEPER: Ink = [121, 148, 116];
const DECK: Ink = [209, 221, 192];
const SHADOW: Ink = [213, 222, 204];
const GOLD: Ink = [181, 133, 43];
const STATION_DECK: Ink = [230, 202, 132];
const LIFT: Ink = [44, 114, 131];
const DROP: Ink = [179, 95, 56];
const BRAKE: Ink = [137, 71, 86];
const LIGHT: Ink = [255, 251, 226];

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn offset(self, x: f64, y: f64) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
        }
    }

    fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

struct Piece {
    kind: TrackKind,
    center: Vec<Point>,
    left: Vec<Point>,
    right: Vec<Point>,
    deck_left: Vec<Point>,
    deck_right: Vec<Point>,
    depth: f64,
    anchor: bool,
}

fn project(x: f64, y: f64, height: f64, turns: u8) -> Point {
    let (u, v) = match turns % 4 {
        0 => (x, y),
        1 => (-y, x),
        2 => (-x, -y),
        _ => (y, -x),
    };
    // Same ground/height proportions as the main isometric renderer; image Y
    // increases downwards, while world height rises towards the top.
    Point {
        x: u - v,
        y: (u + v - height) * 0.5,
    }
}

fn geometry(design: &TrackDesign, turns: u8) -> Vec<Piece> {
    let (anchor, anchor_height) = design.anchor();
    let anchor_index = design
        .pieces()
        .iter()
        .position(|piece| piece.kind == TrackKind::Station)
        .unwrap_or(0);
    design
        .pieces()
        .iter()
        .enumerate()
        .map(|(piece_index, spec)| {
            let points: Vec<_> = spec
                .samples()
                .into_iter()
                .map(|point| {
                    // Subtract in fixed point before conversion, preserving identical
                    // bytes for an otherwise identical translated/elevated design.
                    (
                        f64::from(point.x - anchor.x as i32 * TRACK_UNITS) / f64::from(TRACK_UNITS),
                        f64::from(point.y - anchor.y as i32 * TRACK_UNITS) / f64::from(TRACK_UNITS),
                        f64::from(point.height - i32::from(anchor_height) * TRACK_UNITS)
                            / f64::from(TRACK_UNITS),
                    )
                })
                .collect();
            let normals: Vec<_> = (0..points.len())
                .map(|index| {
                    let a = points[index.saturating_sub(1)];
                    let b = points[(index + 1).min(points.len() - 1)];
                    let dx = b.0 - a.0;
                    let dy = b.1 - a.1;
                    let length = dx.hypot(dy).max(f64::EPSILON);
                    (-dy / length, dx / length)
                })
                .collect();
            let offset = |amount: f64| {
                points
                    .iter()
                    .zip(&normals)
                    .map(|(&(x, y, height), &(nx, ny))| {
                        project(x + nx * amount, y + ny * amount, height, turns)
                    })
                    .collect()
            };
            let center: Vec<_> = offset(0.0);
            let depth = points
                .iter()
                .map(|&(x, y, _)| project(x, y, 0.0, turns).y)
                .sum::<f64>()
                / points.len() as f64;
            let half_deck = if spec.kind == TrackKind::Station {
                0.42
            } else {
                0.25
            };
            Piece {
                kind: spec.kind,
                center,
                left: offset(-0.18),
                right: offset(0.18),
                deck_left: offset(-half_deck),
                deck_right: offset(half_deck),
                depth,
                anchor: piece_index == anchor_index,
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
struct Fit {
    center: Point,
    scale: f64,
    width: u32,
    height: u32,
}

impl Fit {
    fn new(pieces: &[Piece], width: u32, height: u32) -> Self {
        let mut min = Point {
            x: f64::INFINITY,
            y: f64::INFINITY,
        };
        let mut max = Point {
            x: f64::NEG_INFINITY,
            y: f64::NEG_INFINITY,
        };
        for point in pieces
            .iter()
            .flat_map(|piece| piece.deck_left.iter().chain(&piece.deck_right))
        {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
        }
        let padding = (f64::from(width.min(height)) * 0.12).clamp(1.0, 15.0);
        let available_width = (f64::from(width) - padding * 2.0).max(0.0);
        let available_height = (f64::from(height) - padding * 2.0).max(0.0);
        Self {
            center: Point {
                x: (min.x + max.x) * 0.5,
                y: (min.y + max.y) * 0.5,
            },
            scale: (available_width / (max.x - min.x).max(2.0))
                .min(available_height / (max.y - min.y).max(1.0)),
            width,
            height,
        }
    }

    fn point(self, point: Point) -> Point {
        Point {
            x: (point.x - self.center.x) * self.scale + f64::from(self.width) * 0.5,
            y: (point.y - self.center.y) * self.scale + f64::from(self.height) * 0.5,
        }
    }
}

struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        let width = width * SUPERSAMPLE;
        let height = height * SUPERSAMPLE;
        let mut pixels = vec![0; (width * height * 4) as usize];
        for y in 0..height {
            let blend = y * 12 / height.max(1);
            let color = [
                248 - blend as u8,
                246 - (blend / 3) as u8,
                234 - (blend / 2) as u8,
                255,
            ];
            for x in 0..width {
                let index = ((y * width + x) * 4) as usize;
                pixels[index..index + 4].copy_from_slice(&color);
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    fn pixel(&mut self, x: i32, y: i32, ink: Ink) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let index = ((y as u32 * self.width + x as u32) * 4) as usize;
        self.pixels[index..index + 3].copy_from_slice(&ink);
    }

    fn dot(&mut self, point: Point, radius: f64, ink: Ink) {
        let min_x = ((point.x - radius).floor() as i32).max(0);
        let max_x = ((point.x + radius).ceil() as i32).min(self.width as i32 - 1);
        let min_y = ((point.y - radius).floor() as i32).max(0);
        let max_y = ((point.y + radius).ceil() as i32).min(self.height as i32 - 1);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = f64::from(x) + 0.5 - point.x;
                let dy = f64::from(y) + 0.5 - point.y;
                if dx * dx + dy * dy <= radius * radius {
                    self.pixel(x, y, ink);
                }
            }
        }
    }

    fn line(&mut self, from: Point, to: Point, width: f64, ink: Ink) {
        let sample = f64::from(SUPERSAMPLE);
        let from = Point {
            x: from.x * sample,
            y: from.y * sample,
        };
        let to = Point {
            x: to.x * sample,
            y: to.y * sample,
        };
        let steps = (from.x - to.x)
            .abs()
            .max((from.y - to.y).abs())
            .ceil()
            .max(1.0) as u32;
        for step in 0..=steps {
            let fraction = f64::from(step) / f64::from(steps);
            self.dot(
                Point {
                    x: from.x + (to.x - from.x) * fraction,
                    y: from.y + (to.y - from.y) * fraction,
                },
                width * sample * 0.5,
                ink,
            );
        }
    }

    fn triangle(&mut self, vertices: [Point; 3], ink: Ink) {
        let vertices = vertices.map(|point| Point {
            x: point.x * f64::from(SUPERSAMPLE),
            y: point.y * f64::from(SUPERSAMPLE),
        });
        let cross =
            |a: Point, b: Point, p: Point| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        let area = cross(vertices[0], vertices[1], vertices[2]);
        if area.abs() < f64::EPSILON {
            return;
        }
        let min_x = (vertices
            .iter()
            .map(|point| point.x)
            .fold(f64::INFINITY, f64::min)
            .floor() as i32)
            .max(0);
        let max_x = (vertices
            .iter()
            .map(|point| point.x)
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil() as i32)
            .min(self.width as i32 - 1);
        let min_y = (vertices
            .iter()
            .map(|point| point.y)
            .fold(f64::INFINITY, f64::min)
            .floor() as i32)
            .max(0);
        let max_y = (vertices
            .iter()
            .map(|point| point.y)
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil() as i32)
            .min(self.height as i32 - 1);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let point = Point {
                    x: f64::from(x) + 0.5,
                    y: f64::from(y) + 0.5,
                };
                if (0..3).all(|index| {
                    cross(vertices[index], vertices[(index + 1) % 3], point) * area >= 0.0
                }) {
                    self.pixel(x, y, ink);
                }
            }
        }
    }

    fn finish(self) -> Vec<u8> {
        let width = self.width / SUPERSAMPLE;
        let height = self.height / SUPERSAMPLE;
        let mut pixels = vec![255; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let target = ((y * width + x) * 4) as usize;
                for channel in 0..3 {
                    let mut sum = 0_u32;
                    for dy in 0..SUPERSAMPLE {
                        for dx in 0..SUPERSAMPLE {
                            let source =
                                (((y * SUPERSAMPLE + dy) * self.width + x * SUPERSAMPLE + dx) * 4)
                                    as usize
                                    + channel;
                            sum += u32::from(self.pixels[source]);
                        }
                    }
                    pixels[target + channel] = (sum / (SUPERSAMPLE * SUPERSAMPLE)) as u8;
                }
            }
        }
        pixels
    }
}

fn backdrop(canvas: &mut Canvas, width: u32, height: u32) {
    let width = f64::from(width);
    let height = f64::from(height);
    let spacing = 22.0;
    let extent = ((height + width * 0.5) / spacing).ceil() as i32 + 1;
    for line in -extent..=extent {
        let y = f64::from(line) * spacing + height * 0.5;
        for slope in [-0.5, 0.5] {
            canvas.line(
                Point { x: 0.0, y },
                Point {
                    x: width,
                    y: y + width * slope,
                },
                0.55,
                [225, 231, 215],
            );
        }
    }
}

fn feature(kind: TrackKind) -> Ink {
    match kind {
        TrackKind::Station => GOLD,
        TrackKind::Lift => LIFT,
        TrackKind::Drop => DROP,
        TrackKind::Brake => BRAKE,
        TrackKind::Straight | TrackKind::Curve => RAIL,
    }
}

fn draw_piece(canvas: &mut Canvas, piece: &Piece, fit: Fit) {
    let count = piece.center.len();
    let ink = feature(piece.kind);
    for index in 0..count - 1 {
        let a = fit.point(piece.deck_left[index]);
        let b = fit.point(piece.deck_left[index + 1]);
        let c = fit.point(piece.deck_right[index + 1]);
        let d = fit.point(piece.deck_right[index]);
        let deck = if piece.kind == TrackKind::Station {
            STATION_DECK
        } else {
            DECK
        };
        canvas.triangle([a, b, c], deck);
        canvas.triangle([a, c, d], deck);
    }
    // Keep small or widely scattered unfinished designs legible even when the
    // physical pair of rails is narrower than an output pixel.
    for pair in piece.center.windows(2) {
        canvas.line(fit.point(pair[0]), fit.point(pair[1]), 1.6, ink);
    }
    if fit.scale >= 5.0 {
        for index in (1..count - 1).step_by(2) {
            canvas.line(
                fit.point(piece.deck_left[index]),
                fit.point(piece.deck_right[index]),
                0.8,
                SLEEPER,
            );
        }
        for rail in [&piece.left, &piece.right] {
            for pair in rail.windows(2) {
                canvas.line(fit.point(pair[0]), fit.point(pair[1]), 1.15, ink);
            }
        }
        match piece.kind {
            TrackKind::Lift => {
                for index in (0..count - 1).step_by(2) {
                    canvas.line(
                        fit.point(piece.center[index]),
                        fit.point(piece.center[index + 1]),
                        1.4,
                        LIFT,
                    );
                }
            }
            TrackKind::Brake => {
                for index in [2, 4, 6] {
                    canvas.line(
                        fit.point(piece.left[index]),
                        fit.point(piece.right[index]),
                        1.7,
                        BRAKE,
                    );
                }
            }
            TrackKind::Station => {
                for edge in [&piece.deck_left, &piece.deck_right] {
                    for pair in edge.windows(2) {
                        canvas.line(fit.point(pair[0]), fit.point(pair[1]), 1.0, GOLD);
                    }
                }
            }
            TrackKind::Drop | TrackKind::Straight | TrackKind::Curve => {}
        }
    }
}

fn direction_arrow(canvas: &mut Canvas, piece: &Piece, fit: Fit) {
    let mid = piece.center.len() / 2;
    let a = fit.point(piece.center[mid - 1]);
    let b = fit.point(piece.center[mid + 1]);
    let length = a.distance(b);
    if length < f64::EPSILON {
        return;
    }
    let (dx, dy) = ((b.x - a.x) / length, (b.y - a.y) / length);
    let center = fit.point(piece.center[mid]);
    let size = (fit.scale * 0.25).clamp(2.8, 6.0);
    let tip = center.offset(dx * size, dy * size);
    let base = center.offset(-dx * size * 0.7, -dy * size * 0.7);
    let left = base.offset(-dy * size * 0.7, dx * size * 0.7);
    let right = base.offset(dy * size * 0.7, -dx * size * 0.7);
    // The dark outline remains readable on either a gold station or green rail.
    canvas.triangle([tip, left, right], LIGHT);
    canvas.line(tip, left, 0.8, RAIL);
    canvas.line(tip, right, 0.8, RAIL);
    canvas.line(left, right, 0.8, RAIL);
}

/// Render an opaque RGBA thumbnail. Dimensions are clamped to 1..=512; normal
/// cards use 156 × 70 and the selected-design preview uses 320 × 180.
/// Names, map position and absolute entry height do not influence the pixels.
pub(crate) fn render(design: &TrackDesign, rotation: u8, width: u32, height: u32) -> Image {
    let width = width.clamp(1, MAX_SIDE);
    let height = height.clamp(1, MAX_SIDE);
    let mut canvas = Canvas::new(width, height);
    backdrop(&mut canvas, width, height);
    let mut pieces = geometry(design, rotation);
    let fit = Fit::new(&pieces, width, height);
    for piece in &pieces {
        for pair in piece.center.windows(2) {
            canvas.line(
                fit.point(pair[0]).offset(1.0, 2.0),
                fit.point(pair[1]).offset(1.0, 2.0),
                3.0,
                SHADOW,
            );
        }
    }
    // Stable ordering preserves deterministic ties for disconnected drafts.
    pieces.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    for piece in &pieces {
        draw_piece(&mut canvas, piece, fit);
    }
    if let Some(anchor) = pieces.iter().find(|piece| piece.anchor) {
        direction_arrow(&mut canvas, anchor, fit);
    }
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        canvas.finish(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::{
        MAX_MAP_SIZE, MAX_TRACK_DRAFT_PIECES, Park, TilePos, TrackBuildKind, TrackDirection,
    };

    fn demo() -> TrackDesign {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        TrackDesign::from_draft("Sampled grades", park.circuit().unwrap().specs()).unwrap()
    }

    fn pixels(design: &TrackDesign, rotation: u8) -> Vec<u8> {
        render(design, rotation, 156, 70).data.unwrap()
    }

    #[test]
    fn deterministic_images_do_not_depend_on_name_location_or_absolute_height() {
        let design = demo();
        let before = design.to_json().unwrap();
        let moved = TrackDesign::from_draft(
            "A different name",
            &design
                .transformed(TilePos { x: 100, y: 110 }, 15, 0)
                .unwrap(),
        )
        .unwrap();
        for rotation in 0..4 {
            let bytes = pixels(&design, rotation);
            assert_eq!(bytes, pixels(&design, rotation));
            assert_eq!(bytes, pixels(&moved, rotation));
            assert_eq!(bytes, pixels(&design, rotation + 4));
        }
        assert_eq!(design.to_json().unwrap(), before);
    }

    #[test]
    fn all_rotations_match_actual_rotated_sample_geometry() {
        let design = demo();
        let original = pixels(&design, 0);
        for rotation in 1..4 {
            let transformed = TrackDesign::from_draft(
                "Rotated",
                &design
                    .transformed(TilePos { x: 100, y: 100 }, 8, rotation)
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(pixels(&design, rotation), pixels(&transformed, 0));
            assert_ne!(original, pixels(&design, rotation));
        }
    }

    #[test]
    fn each_supported_piece_and_travel_direction_has_a_distinct_thumbnail() {
        let kinds = [
            TrackBuildKind::Station,
            TrackBuildKind::Straight,
            TrackBuildKind::CurveLeft,
            TrackBuildKind::CurveRight,
            TrackBuildKind::Lift,
            TrackBuildKind::Drop,
            TrackBuildKind::SteepDrop,
            TrackBuildKind::Brake,
        ];
        let mut images = Vec::new();
        for kind in kinds {
            let design = TrackDesign::from_draft(
                "Piece",
                &[kind.spec(TilePos { x: 0, y: 0 }, 8, TrackDirection::East)],
            )
            .unwrap();
            let image = pixels(&design, 0);
            assert!(
                images.iter().all(|previous| previous != &image),
                "{kind:?} lost its geometry or feature"
            );
            images.push(image);
        }
        let forwards = TrackDesign::from_draft(
            "Forward",
            &[TrackBuildKind::Straight.spec(TilePos { x: 0, y: 0 }, 8, TrackDirection::East)],
        )
        .unwrap();
        let backwards = TrackDesign::from_draft(
            "Backward",
            &[TrackBuildKind::Straight.spec(TilePos { x: 0, y: 0 }, 8, TrackDirection::West)],
        )
        .unwrap();
        assert_ne!(pixels(&forwards, 0), pixels(&backwards, 0));
    }

    #[test]
    fn invalid_world_placement_still_fits_complete_signed_geometry() {
        let specs = [
            TrackBuildKind::Station.spec(TilePos { x: 0, y: 0 }, 0, TrackDirection::East),
            TrackBuildKind::Lift.spec(TilePos { x: 0, y: 1 }, 0, TrackDirection::South),
            TrackBuildKind::SteepDrop.spec(TilePos { x: 1, y: 1 }, 30, TrackDirection::West),
        ];
        let design = TrackDesign::from_draft("At the edge", &specs).unwrap();
        assert!(design.transformed(TilePos { x: 0, y: 0 }, 0, 1).is_err());
        for (width, height) in [(156, 70), (320, 180)] {
            for rotation in 0..4 {
                let pieces = geometry(&design, rotation);
                let fit = Fit::new(&pieces, width, height);
                assert!(fit.scale.is_finite());
                for point in pieces
                    .iter()
                    .flat_map(|piece| piece.deck_left.iter().chain(&piece.deck_right))
                {
                    let point = fit.point(*point);
                    assert!(point.x.is_finite() && point.y.is_finite());
                    assert!((7.0..f64::from(width) - 7.0).contains(&point.x));
                    assert!((7.0..f64::from(height) - 7.0).contains(&point.y));
                }
                assert_eq!(
                    render(&design, rotation, width, height).data.unwrap().len(),
                    (width * height * 4) as usize
                );
            }
        }
    }

    #[test]
    fn maximum_scattered_design_stays_bounded_and_preserves_extreme_grades() {
        let specs: Vec<_> = (0..MAX_TRACK_DRAFT_PIECES)
            .map(|index| {
                let kind = if index == 0 {
                    TrackBuildKind::Station
                } else if index % 2 == 0 {
                    TrackBuildKind::Lift
                } else {
                    TrackBuildKind::SteepDrop
                };
                kind.spec(
                    TilePos {
                        x: index as u32,
                        y: MAX_MAP_SIZE - 1 - index as u32,
                    },
                    if index % 2 == 0 { 0 } else { 30 },
                    TrackDirection::East,
                )
            })
            .collect();
        let design = TrackDesign::from_draft("Maximum separated pieces", &specs).unwrap();
        let before = design.clone();
        for rotation in 0..4 {
            let image = render(&design, rotation, 320, 180);
            let data = image.data.unwrap();
            assert_eq!(data.len(), 320 * 180 * 4);
            assert!(data.chunks_exact(4).all(|pixel| pixel[3] == 255));
            assert!(
                data.chunks_exact(4)
                    .any(|pixel| pixel[0] < 100 && pixel[1] < 150)
            );
        }
        assert_eq!(design, before);
    }

    #[test]
    fn caller_dimensions_cannot_create_an_empty_or_unbounded_image() {
        let design = demo();
        for (width, height, expected_width, expected_height) in [
            (0, 0, 1, 1),
            (u32::MAX, 1, MAX_SIDE, 1),
            (1, u32::MAX, 1, MAX_SIDE),
            (u32::MAX, u32::MAX, MAX_SIDE, MAX_SIDE),
        ] {
            let image = render(&design, 255, width, height);
            assert_eq!(image.texture_descriptor.size.width, expected_width);
            assert_eq!(image.texture_descriptor.size.height, expected_height);
            assert_eq!(
                image.texture_descriptor.format,
                TextureFormat::Rgba8UnormSrgb
            );
            assert_eq!(
                image.data.unwrap().len(),
                (expected_width * expected_height * 4) as usize
            );
        }
    }
}
