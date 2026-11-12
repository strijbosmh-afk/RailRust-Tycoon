//! Original catalogue models. All surfaces participate in physical depth and picking.
use super::carousel_render::{at, cuboid, face, nominal};
use super::*;
use park_sim::{CarouselRide, RideModel};
type Face = service_render::Face;
const GOLD: [Ink; 3] = [
    [1.0, 0.79, 0.22, 1.0],
    [0.77, 0.45, 0.09, 1.0],
    [0.55, 0.26, 0.05, 1.0],
];
const RED: [Ink; 3] = [
    [0.96, 0.20, 0.28, 1.0],
    [0.69, 0.07, 0.16, 1.0],
    [0.44, 0.04, 0.10, 1.0],
];
const BLUE: [Ink; 3] = [
    [0.15, 0.76, 0.95, 1.0],
    [0.06, 0.44, 0.72, 1.0],
    [0.04, 0.25, 0.47, 1.0],
];
const WOOD: [Ink; 3] = [
    [0.76, 0.44, 0.18, 1.0],
    [0.50, 0.24, 0.08, 1.0],
    [0.32, 0.13, 0.04, 1.0],
];
const GLASS: [Ink; 3] = [
    [0.66, 0.94, 0.98, 1.0],
    [0.18, 0.68, 0.80, 1.0],
    [0.10, 0.40, 0.57, 1.0],
];

fn cylinder(
    faces: &mut Vec<Face>,
    center: Vec3,
    radius: f32,
    height: f32,
    colors: [Ink; 3],
    size: u32,
    rotation: u8,
) {
    for i in 0..24 {
        let a = i as f32 * std::f32::consts::TAU / 24.0;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 24.0;
        let p = center + Vec3::new(a.cos() * radius, a.sin() * radius, 0.0);
        let q = center + Vec3::new(b.cos() * radius, b.sin() * radius, 0.0);
        face(
            faces,
            [p, q, q + Vec3::Z * height, p + Vec3::Z * height],
            colors[1 + i % 2],
            size,
            rotation,
        );
        face(
            faces,
            [
                center + Vec3::Z * height,
                p + Vec3::Z * height,
                q + Vec3::Z * height,
                center + Vec3::Z * height,
            ],
            colors[0],
            size,
            rotation,
        );
    }
}
fn beam(faces: &mut Vec<Face>, a: Vec3, b: Vec3, width: f32, ink: Ink, size: u32, rotation: u8) {
    face(
        faces,
        [
            a - Vec3::X * width,
            b - Vec3::X * width,
            b + Vec3::X * width,
            a + Vec3::X * width,
        ],
        ink,
        size,
        rotation,
    );
    face(
        faces,
        [
            a - Vec3::Y * width,
            b - Vec3::Y * width,
            b + Vec3::Y * width,
            a + Vec3::Y * width,
        ],
        ink,
        size,
        rotation,
    );
}
fn transform_faces(
    mut faces: Vec<Face>,
    origin: TilePos,
    orientation: u8,
    model: RideModel,
    size: u32,
    camera: u8,
) -> Vec<Face> {
    // Geometry is built in local coordinates, transformed before projection.
    // Projection is performed here once so every view shares the same footprint.
    let (w, d) = model.dimensions();
    for f in &mut faces {
        for p in &mut f.surface.vertices {
            let (x, y) = match orientation % 4 {
                0 => (p.x, p.y),
                1 => (d as f32 - 1.0 - p.y, p.x),
                2 => (w as f32 - 1.0 - p.x, d as f32 - 1.0 - p.y),
                _ => (p.y, w as f32 - 1.0 - p.x),
            };
            *p = at(
                Vec3::new(origin.x as f32 + x, origin.y as f32 + y, p.z),
                size,
                camera,
            );
        }
    }
    faces.retain(|f| f.surface.has_area());
    faces
}
// Identity projection lets the existing solid primitive helpers generate local faces.
fn local_box(out: &mut Vec<Face>, a: Vec3, b: Vec3, inks: [Ink; 3]) {
    let upper = [
        Vec3::new(a.x, a.y, b.z),
        Vec3::new(a.x, b.y, b.z),
        Vec3::new(b.x, b.y, b.z),
        Vec3::new(b.x, a.y, b.z),
    ];
    let lower = upper.map(|p| Vec3::new(p.x, p.y, a.z));
    out.push(Face {
        surface: Surface {
            vertices: upper,
            kind: SurfaceKind::Top,
        },
        ink: inks[0],
    });
    for i in 0..4 {
        let n = (i + 1) % 4;
        out.push(Face {
            surface: Surface {
                vertices: [upper[i], upper[n], lower[n], lower[i]],
                kind: SurfaceKind::Top,
            },
            ink: inks[1 + i % 2],
        });
    }
}
fn local_panel(out: &mut Vec<Face>, points: [Vec3; 4], ink: Ink) {
    out.push(Face {
        surface: Surface {
            vertices: points,
            kind: SurfaceKind::Top,
        },
        ink,
    });
}

pub(super) fn static_faces(
    model: RideModel,
    origin: TilePos,
    height: u8,
    orientation: u8,
    size: u32,
    camera: u8,
) -> Vec<Face> {
    let mut out = Vec::new();
    let (w, d) = model.dimensions();
    let z = f32::from(height);
    let center = Vec3::new((w - 1) as f32 / 2.0, (d - 1) as f32 / 2.0, z);
    local_box(
        &mut out,
        Vec3::new(-0.45, -0.45, z),
        Vec3::new(w as f32 - 0.55, d as f32 - 0.55, z + 0.15),
        GOLD,
    );
    for x in 0..w {
        local_box(
            &mut out,
            Vec3::new(x as f32 - 0.35, -0.4, z + 0.15),
            Vec3::new(x as f32 + 0.35, -0.15, z + 0.17),
            RED,
        );
    }
    match model {
        RideModel::FerrisWheel => {
            for y in [0.45, 1.55] {
                for x in [0.65, 3.35] {
                    local_panel(
                        &mut out,
                        [
                            Vec3::new(x - 0.10, y, z + 0.2),
                            Vec3::new(x + 0.10, y, z + 0.2),
                            Vec3::new(2.1, y, z + 5.6),
                            Vec3::new(1.9, y, z + 5.6),
                        ],
                        RED[1],
                    );
                }
                local_box(
                    &mut out,
                    Vec3::new(1.85, y - 0.08, z + 5.40),
                    Vec3::new(2.15, y + 0.08, z + 5.75),
                    GOLD,
                );
            }
            for y in [0.78, 1.22] {
                for i in 0..40 {
                    let a = i as f32 * std::f32::consts::TAU / 40.0;
                    let b = (i + 1) as f32 * std::f32::consts::TAU / 40.0;
                    local_panel(
                        &mut out,
                        [
                            center + Vec3::new(a.sin() * 1.32, y - 1.0, 5.55 + a.cos() * 5.2),
                            center + Vec3::new(b.sin() * 1.32, y - 1.0, 5.55 + b.cos() * 5.2),
                            center + Vec3::new(b.sin() * 1.27, y - 1.0, 5.55 + b.cos() * 5.0),
                            center + Vec3::new(a.sin() * 1.27, y - 1.0, 5.55 + a.cos() * 5.0),
                        ],
                        GOLD[0],
                    );
                }
            }
        }
        RideModel::TeaCups => {
            for i in 0..16 {
                let a = i as f32 * std::f32::consts::TAU / 16.0;
                let b = (i + 1) as f32 * std::f32::consts::TAU / 16.0;
                local_panel(
                    &mut out,
                    [
                        center + Vec3::Z * 0.18,
                        center + Vec3::new(a.cos() * 2.2, a.sin() * 2.2, 0.18),
                        center + Vec3::new(b.cos() * 2.2, b.sin() * 2.2, 0.18),
                        center + Vec3::Z * 0.18,
                    ],
                    if i % 2 == 0 { RED[0] } else { BLUE[0] },
                );
            }
            local_box(
                &mut out,
                center + Vec3::new(-0.28, -0.28, 0.2),
                center + Vec3::new(0.28, 0.28, 1.3),
                GOLD,
            );
        }
        RideModel::ObservationTower => {
            local_box(
                &mut out,
                center + Vec3::new(-0.16, -0.16, 0.2),
                center + Vec3::new(0.16, 0.16, 10.0),
                BLUE,
            );
            for h in 1..10 {
                local_box(
                    &mut out,
                    center + Vec3::new(-0.2, -0.2, h as f32),
                    center + Vec3::new(0.2, 0.2, h as f32 + 0.12),
                    GOLD,
                );
            }
            local_box(
                &mut out,
                center + Vec3::new(-0.6, -0.6, 10.0),
                center + Vec3::new(0.6, 0.6, 10.3),
                RED,
            );
            local_box(
                &mut out,
                center + Vec3::new(-0.05, -0.05, 10.3),
                center + Vec3::new(0.05, 0.05, 11.65),
                GOLD,
            );
        }
        RideModel::PirateShip => {
            for x in [0.35, 3.65] {
                for y in [-0.2, 2.2] {
                    local_panel(
                        &mut out,
                        [
                            Vec3::new(x - 0.08, y, z + 0.15),
                            Vec3::new(x + 0.08, y, z + 0.15),
                            Vec3::new(x + 0.08, 1.0, z + 5.35),
                            Vec3::new(x - 0.08, 1.0, z + 5.35),
                        ],
                        WOOD[1],
                    );
                }
            }
            local_box(
                &mut out,
                Vec3::new(0.25, 0.9, z + 5.3),
                Vec3::new(3.75, 1.1, z + 5.5),
                GOLD,
            );
        }
        _ => {}
    }
    transform_faces(out, origin, orientation, model, size, camera)
}

pub(super) fn dynamic_faces(park: &Park, ride: &CarouselRide, rotation: u8) -> Vec<Face> {
    let model = park.ride_model(ride.ride.id).unwrap();
    let points = park.carousel_seat_positions(ride.ride.id);
    let mut out = Vec::new();
    let size = park.width();
    let mid = |slice: &[park_sim::PathPoint]| {
        slice.iter().map(|p| nominal(*p)).sum::<Vec3>() / slice.len() as f32
    };
    let colors = ride_colors::inks(
        park.ride_appearance(ride.ride.id)
            .unwrap_or_default()
            .vehicles,
        RED,
    );
    match model {
        RideModel::TeaCups => {
            for (i, seats) in points.chunks(2).enumerate() {
                let c = mid(seats);
                cylinder(
                    &mut out,
                    c - Vec3::Z * 0.25,
                    0.40,
                    0.45,
                    if i % 2 == 0 { colors } else { BLUE },
                    size,
                    rotation,
                );
                cylinder(
                    &mut out,
                    c + Vec3::Z * 0.18,
                    0.14,
                    0.07,
                    GOLD,
                    size,
                    rotation,
                );
            }
        }
        RideModel::ObservationTower => {
            let c = mid(&points);
            cylinder(
                &mut out,
                c - Vec3::Z * 0.25,
                0.72,
                0.20,
                colors,
                size,
                rotation,
            );
            cylinder(
                &mut out,
                c + Vec3::Z * 0.15,
                0.66,
                0.85,
                GLASS,
                size,
                rotation,
            );
            cylinder(
                &mut out,
                c + Vec3::Z * 1.0,
                0.75,
                0.18,
                GOLD,
                size,
                rotation,
            );
        }
        RideModel::FerrisWheel => {
            let c = mid(&points);
            cuboid(
                &mut out,
                c - Vec3::new(0.42, 0.35, 0.25),
                c + Vec3::new(0.42, 0.35, 0.14),
                colors,
                size,
                rotation,
            );
            cuboid(
                &mut out,
                c + Vec3::new(-0.45, -0.38, 0.95),
                c + Vec3::new(0.45, 0.38, 1.1),
                GOLD,
                size,
                rotation,
            );
            let (w, d) = model.dimensions();
            let center = Vec3::new(
                ride.origin.x as f32 + (w - 1) as f32 / 2.0,
                ride.origin.y as f32 + (d - 1) as f32 / 2.0,
                ride.height as f32 + 5.55,
            );
            beam(&mut out, center, c, 0.035, GOLD[0], size, rotation);
            for i in 1..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let direction = c - center;
                let along = if ride.rotation.is_multiple_of(2) {
                    Vec3::X
                } else {
                    Vec3::Y
                };
                let signed = direction.dot(along);
                let pos = center
                    + along * (signed * a.cos() + direction.z * 0.25 * a.sin())
                    + Vec3::Z * (direction.z * a.cos() - signed * 4.0 * a.sin());
                beam(&mut out, center, pos, 0.028, GOLD[0], size, rotation);
                cuboid(
                    &mut out,
                    pos - Vec3::new(0.25, 0.25, 0.18),
                    pos + Vec3::new(0.25, 0.25, 0.3),
                    if i % 2 == 0 { BLUE } else { RED },
                    size,
                    rotation,
                );
            }
        }
        RideModel::PirateShip => {
            let c = mid(&points);
            let along = if ride.rotation.is_multiple_of(2) {
                Vec3::X
            } else {
                Vec3::Y
            };
            let side = Vec3::new(-along.y, along.x, 0.0);
            for i in 0..6 {
                let a = c + along * (i as f32 * 0.4 - 1.2);
                cuboid(
                    &mut out,
                    a - side * 0.45 - Vec3::Z * 0.32,
                    a + along * 0.32 + side * 0.45,
                    WOOD,
                    size,
                    rotation,
                );
            }
            let pivot = Vec3::new(c.x, c.y, ride.height as f32 + 5.3);
            beam(&mut out, pivot, c, 0.04, GOLD[0], size, rotation);
        }
        _ => {}
    }
    // Every seated guest remains visible at the simulation's authoritative point.
    for p in park
        .ride_guest_positions()
        .iter()
        .filter(|p| p.ride == ride.ride.id && p.phase == RideVisitPhase::Riding)
    {
        let c = nominal(p.point);
        cuboid(
            &mut out,
            c + Vec3::new(-0.06, -0.05, 0.02),
            c + Vec3::new(0.06, 0.05, 0.35),
            if p.guest.0.is_multiple_of(2) {
                RED
            } else {
                BLUE
            },
            size,
            rotation,
        );
        cuboid(
            &mut out,
            c + Vec3::new(-0.045, -0.04, 0.35),
            c + Vec3::new(0.045, 0.04, 0.65),
            [rgb(240, 192, 140); 3],
            size,
            rotation,
        );
    }
    out
}

pub(super) fn prefab_faces(
    model: RideModel,
    origin: TilePos,
    height: u8,
    orientation: u8,
    size: u32,
    rotation: u8,
) -> Vec<Face> {
    let mut out = Vec::new();
    if let Ok(specs) = model.layout(origin, height, orientation) {
        for spec in specs {
            for pair in spec.samples().windows(2) {
                let a = Vec3::new(pair[0].x as f32, pair[0].y as f32, pair[0].height as f32)
                    / park_sim::TRACK_UNITS as f32;
                let b = Vec3::new(pair[1].x as f32, pair[1].y as f32, pair[1].height as f32)
                    / park_sim::TRACK_UNITS as f32;
                let d = (b - a).truncate().normalize_or_zero();
                let n = Vec3::new(-d.y, d.x, 0.0) * 0.28;
                face(
                    &mut out,
                    [a - n, b - n, b + n, a + n],
                    BLUE[0],
                    size,
                    rotation,
                );
            }
        }
    }
    out
}

/// Rotate established artwork through world space while preserving its depth planes.
pub(super) fn orient_legacy(
    mut faces: Vec<Face>,
    origin: TilePos,
    model: RideModel,
    orientation: u8,
    size: u32,
    camera: u8,
) -> Vec<Face> {
    for face in &mut faces {
        for p in &mut face.surface.vertices {
            let sum = p.z - BASE_DEPTH;
            let difference = p.x / HALF_WIDTH;
            let height = (p.y + (sum - (size as f32 - 1.0)) * HALF_HEIGHT) / HEIGHT_STEP;
            let shift = if model == RideModel::Carousel {
                0.0
            } else {
                0.5
            };
            let x = (sum + difference) / 2.0 - origin.x as f32 - shift;
            let y = (sum - difference) / 2.0 - origin.y as f32 - shift;
            let (x, y) = match orientation % 4 {
                0 => (x, y),
                1 => (2.0 - y, x),
                2 => (2.0 - x, 2.0 - y),
                _ => (y, 2.0 - x),
            };
            *p = at(
                Vec3::new(origin.x as f32 + x, origin.y as f32 + y, height),
                size,
                camera,
            );
        }
    }
    faces.retain(|f| f.surface.has_area());
    faces
}

pub(super) fn paint_static(
    mut faces: Vec<Face>,
    appearance: park_sim::RideAppearance,
) -> Vec<Face> {
    for face in &mut faces {
        for (source, paint) in [
            (RED, appearance.structure),
            (BLUE, appearance.structure),
            (GOLD, appearance.supports),
        ] {
            if let Some(index) = source.iter().position(|color| *color == face.ink) {
                face.ink = ride_colors::inks(paint, source)[index];
                break;
            }
        }
    }
    faces
}
