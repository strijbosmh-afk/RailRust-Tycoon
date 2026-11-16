//! Original catalogue models. All surfaces participate in physical depth and picking.
use super::carousel_render::{at, cuboid, face, nominal};
use super::ride_detail::{lathe, shell, tube};
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
    // The perimeter is assembled from slim posts and two rails, preserving
    // the open gate at the front left and every model's reserved footprint.
    let fence = [GOLD[0], GOLD[1], GOLD[2]];
    for x in 0..w {
        for y in [-0.34, d as f32 - 0.66] {
            if x == 0 && y < 0.0 {
                continue;
            }
            local_box(
                &mut out,
                Vec3::new(x as f32 - 0.04, y, z + 0.16),
                Vec3::new(x as f32 + 0.04, y + 0.06, z + 0.75),
                fence,
            );
            local_box(
                &mut out,
                Vec3::new(x as f32 - 0.07, y - 0.02, z + 0.75),
                Vec3::new(x as f32 + 0.07, y + 0.08, z + 0.84),
                fence,
            );
            if x > 0 && !(x == 1 && y < 0.0) {
                for h in [0.43, 0.69] {
                    local_box(
                        &mut out,
                        Vec3::new(x as f32 - 0.95, y, z + h),
                        Vec3::new(x as f32 - 0.05, y + 0.04, z + h + 0.035),
                        fence,
                    );
                }
            }
        }
    }
    match model {
        RideModel::FerrisWheel => {
            let pivot = center + Vec3::Z * 5.55;
            for y in [0.45, 1.55] {
                for x in [0.65, 3.35] {
                    let base = Vec3::new(x, y, z + 0.18);
                    tube(&mut out, base, Vec3::new(2.0, y, z + 5.55), 0.065, RED);
                    shell(
                        &mut out,
                        base,
                        &[(0.21, 0.18, 0.0), (0.21, 0.18, 0.16)],
                        0.04,
                        GOLD,
                    );
                }
                tube(
                    &mut out,
                    Vec3::new(0.75, y, z + 0.5),
                    Vec3::new(3.25, y, z + 0.5),
                    0.035,
                    GOLD,
                );
            }
            for y in [-0.22, 0.22] {
                let point = |angle: f32, r: f32| {
                    pivot + Vec3::new(angle.sin() * 1.30 * r, y, -angle.cos() * 5.20 * r)
                };
                for i in 0..64 {
                    let a = i as f32 * std::f32::consts::TAU / 64.0;
                    let b = (i + 1) as f32 * std::f32::consts::TAU / 64.0;
                    for r in [1.0, 0.94] {
                        tube(&mut out, point(a, r), point(b, r), 0.016, GOLD);
                    }
                    if i % 2 == 0 {
                        tube(&mut out, point(a, 0.94), point(a, 1.0), 0.009, RED);
                    }
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
            // A scalloped striped parasol crowns the central tea service.
            for i in 0..32 {
                let a = i as f32 * std::f32::consts::TAU / 32.0;
                let b = (i + 1) as f32 * std::f32::consts::TAU / 32.0;
                let p = |angle: f32, r: f32, h: f32| {
                    center + Vec3::new(angle.cos() * r, angle.sin() * r, h)
                };
                local_panel(
                    &mut out,
                    [
                        p(a, 0.04, 2.85),
                        p(a, 0.70, 2.43),
                        p(b, 0.70, 2.43),
                        p(b, 0.04, 2.85),
                    ],
                    if i % 4 < 2 {
                        GOLD[0]
                    } else {
                        rgb(255, 243, 205)
                    },
                );
                local_panel(
                    &mut out,
                    [
                        p(a, 0.70, 2.43),
                        p(b, 0.70, 2.43),
                        p(b, 0.70, 2.31),
                        p(a, 0.70, 2.31),
                    ],
                    GOLD[1],
                );
            }
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
            for x in [0.42, 3.42] {
                local_box(
                    &mut out,
                    Vec3::new(x, -0.20, z + 0.25),
                    Vec3::new(x + 0.16, 0.25, z + 1.25),
                    WOOD,
                );
                local_box(
                    &mut out,
                    Vec3::new(x - 0.03, -0.23, z + 1.25),
                    Vec3::new(x + 0.19, 0.28, z + 1.36),
                    GOLD,
                );
            }
        }
        _ => {}
    }
    // Inlaid deck border and side railings have open gaps at the entry.
    for x in [-0.34, w as f32 - 0.66] {
        for y in 0..d {
            let post = Vec3::new(x, y as f32, z + 0.18);
            tube(&mut out, post, post + Vec3::Z * 0.55, 0.018, GOLD);
            if y > 0 {
                for h in [0.39, 0.69] {
                    tube(
                        &mut out,
                        Vec3::new(x, y as f32 - 1.0, z + h),
                        Vec3::new(x, y as f32, z + h),
                        0.012,
                        GOLD,
                    );
                }
            }
        }
    }
    if model == RideModel::TeaCups {
        lathe(
            &mut out,
            center,
            &[(2.18, 0.18), (2.25, 0.21), (2.25, 0.27), (2.18, 0.27)],
            GOLD,
        );
        lathe(
            &mut out,
            center,
            &[
                (0.28, 0.2),
                (0.36, 0.4),
                (0.20, 0.62),
                (0.20, 1.98),
                (0.10, 2.20),
            ],
            BLUE,
        );
        lathe(
            &mut out,
            center,
            &[(0.0, 2.81), (0.14, 2.83), (0.08, 3.0), (0.0, 3.0)],
            GOLD,
        );
    }
    if model == RideModel::ObservationTower {
        for h in 0..10 {
            let height = h as f32 + 0.4;
            for side in [-1.0, 1.0] {
                tube(
                    &mut out,
                    center + Vec3::new(side * 0.18, -0.13, height),
                    center + Vec3::new(side * 0.18, 0.13, height + 0.45),
                    0.013,
                    GOLD,
                );
            }
        }
        lathe(
            &mut out,
            center,
            &[(0.0, 10.25), (0.65, 10.25), (0.58, 10.45), (0.15, 10.70)],
            RED,
        );
    }
    if model == RideModel::PirateShip {
        for x in [0.35, 3.65] {
            for y in [-0.2, 2.2] {
                tube(
                    &mut out,
                    Vec3::new(x, y, z + 0.15),
                    Vec3::new(x, 1.0, z + 5.35),
                    0.055,
                    WOOD,
                );
                shell(
                    &mut out,
                    Vec3::new(x, y, z + 0.18),
                    &[(0.15, 0.15, 0.0), (0.15, 0.15, 0.16)],
                    0.03,
                    GOLD,
                );
            }
            tube(
                &mut out,
                Vec3::new(x, 0.0, z + 0.7),
                Vec3::new(x, 2.0, z + 0.7),
                0.026,
                GOLD,
            );
        }
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
                let porcelain = if i % 2 == 0 { colors } else { BLUE };
                let mut body = Vec::new();
                // Saucer, tapered porcelain bowl and a recessed inner wall.
                lathe(
                    &mut body,
                    c,
                    &[(0.0, -0.25), (0.43, -0.25), (0.47, -0.19), (0.30, -0.15)],
                    GOLD,
                );
                lathe(
                    &mut body,
                    c,
                    &[
                        (0.24, -0.18),
                        (0.34, -0.08),
                        (0.42, 0.21),
                        (0.40, 0.25),
                        (0.36, 0.21),
                        (0.29, -0.06),
                    ],
                    porcelain,
                );
                lathe(
                    &mut body,
                    c,
                    &[(0.36, 0.21), (0.40, 0.25), (0.42, 0.21)],
                    GOLD,
                );
                // Handles turn with the saved seat pair instead of facing the camera.
                let axis = (nominal(seats[0]) - c)
                    .truncate()
                    .normalize_or_zero()
                    .extend(0.0);
                let axis = if axis == Vec3::ZERO { Vec3::X } else { axis };
                for sign in [-1.0, 1.0] {
                    let handle = c + axis * sign * 0.40;
                    for j in 0..16 {
                        let point = |j: usize| {
                            let angle = j as f32 * std::f32::consts::TAU / 16.0;
                            handle
                                + axis * sign * angle.cos() * 0.14
                                + Vec3::Z * (0.05 + angle.sin() * 0.19)
                        };
                        tube(&mut body, point(j), point(j + 1), 0.015, GOLD);
                    }
                }
                lathe(
                    &mut body,
                    c,
                    &[
                        (0.07, -0.05),
                        (0.07, 0.17),
                        (0.15, 0.17),
                        (0.15, 0.22),
                        (0.0, 0.22),
                    ],
                    GOLD,
                );
                out.extend(ride_detail::project(body, size, rotation));
            }
        }
        RideModel::ObservationTower => {
            let c = mid(&points);
            let mut body = Vec::new();
            lathe(
                &mut body,
                c,
                &[
                    (0.0, -0.26),
                    (0.62, -0.26),
                    (0.78, -0.12),
                    (0.78, 0.06),
                    (0.68, 0.15),
                ],
                colors,
            );
            lathe(&mut body, c, &[(0.66, 0.15), (0.66, 0.88)], GLASS);
            for i in 0..12 {
                let angle = i as f32 * std::f32::consts::TAU / 12.0;
                let offset = Vec3::new(angle.cos() * 0.665, angle.sin() * 0.665, 0.0);
                tube(
                    &mut body,
                    c + offset + Vec3::Z * 0.15,
                    c + offset + Vec3::Z * 0.9,
                    0.012,
                    GOLD,
                );
            }
            lathe(
                &mut body,
                c,
                &[
                    (0.68, 0.89),
                    (0.80, 0.91),
                    (0.80, 1.04),
                    (0.42, 1.19),
                    (0.0, 1.19),
                ],
                GOLD,
            );
            out.extend(ride_detail::project(body, size, rotation));
        }
        RideModel::FerrisWheel => {
            let c = mid(&points);
            let center = Vec3::new(
                ride.origin.x as f32
                    + if ride.rotation.is_multiple_of(2) {
                        2.0
                    } else {
                        1.0
                    },
                ride.origin.y as f32
                    + if ride.rotation.is_multiple_of(2) {
                        1.0
                    } else {
                        2.0
                    },
                ride.height as f32 + 5.55,
            );
            let along = if ride.rotation.is_multiple_of(2) {
                Vec3::X
            } else {
                Vec3::Y
            };
            let across = Vec3::new(-along.y, along.x, 0.0);
            let direction = c - center;
            let phase = (direction.dot(along) / 1.30).atan2(-direction.z / 5.20);
            let mut body = Vec::new();
            for i in 0..8 {
                let angle = phase + i as f32 * std::f32::consts::TAU / 8.0;
                let pos = if i == 0 {
                    c
                } else {
                    center + along * (angle.sin() * 1.30) - Vec3::Z * (angle.cos() * 5.20)
                };
                for sign in [-1.0, 1.0] {
                    tube(
                        &mut body,
                        center + across * sign * 0.22,
                        pos + across * sign * 0.22,
                        0.016,
                        GOLD,
                    );
                }
                let cabin = if i == 0 {
                    colors
                } else if i % 2 == 0 {
                    BLUE
                } else {
                    RED
                };
                let mut parts = Vec::new();
                shell(
                    &mut parts,
                    Vec3::ZERO,
                    &[(0.29, 0.24, -0.29), (0.38, 0.29, -0.12), (0.38, 0.29, 0.20)],
                    0.09,
                    cabin,
                );
                shell(
                    &mut parts,
                    Vec3::ZERO,
                    &[(0.40, 0.31, 0.18), (0.40, 0.31, 0.25)],
                    0.08,
                    GOLD,
                );
                // Open windows retain the six real riders in the active gondola.
                for x in [-0.30, 0.30] {
                    for y in [-0.23, 0.23] {
                        tube(
                            &mut parts,
                            Vec3::new(x, y, 0.24),
                            Vec3::new(x, y, 0.92),
                            0.013,
                            GOLD,
                        );
                    }
                }
                shell(
                    &mut parts,
                    Vec3::ZERO,
                    &[(0.43, 0.34, 0.91), (0.34, 0.27, 1.06), (0.12, 0.12, 1.17)],
                    0.09,
                    cabin,
                );
                for face in &mut parts {
                    for p in &mut face.surface.vertices {
                        *p = pos + along * p.x + across * p.y + Vec3::Z * p.z;
                    }
                }
                body.extend(parts);
            }
            tube(
                &mut body,
                center - across * 0.60,
                center + across * 0.60,
                0.09,
                GOLD,
            );
            out.extend(ride_detail::project(body, size, rotation));
        }
        RideModel::PirateShip => {
            let c = mid(&points);
            let along = if ride.rotation.is_multiple_of(2) {
                Vec3::X
            } else {
                Vec3::Y
            };
            let side = Vec3::new(-along.y, along.x, 0.0);
            let mut body = Vec::new();
            shell(
                &mut body,
                Vec3::ZERO,
                &[(0.91, 0.16, -0.32), (1.21, 0.32, -0.24), (1.44, 0.46, 0.0)],
                0.32,
                WOOD,
            );
            shell(
                &mut body,
                Vec3::ZERO,
                &[(1.46, 0.47, -0.04), (1.46, 0.47, 0.06)],
                0.30,
                GOLD,
            );
            for x in [-1.0, -0.6, -0.2, 0.2, 0.6, 1.0] {
                shell(
                    &mut body,
                    Vec3::new(x, 0.0, 0.0),
                    &[(0.055, 0.38, 0.06), (0.055, 0.38, 0.17)],
                    0.02,
                    colors,
                );
                for y in [-0.43, 0.43] {
                    tube(
                        &mut body,
                        Vec3::new(x, y, 0.06),
                        Vec3::new(x, y, 0.36),
                        0.013,
                        GOLD,
                    );
                }
            }
            for sign in [-1.0, 1.0] {
                for h in [0.36, 0.24] {
                    tube(
                        &mut body,
                        Vec3::new(-1.20, sign * 0.43, h),
                        Vec3::new(1.20, sign * 0.43, h),
                        0.018,
                        GOLD,
                    );
                }
                tube(
                    &mut body,
                    Vec3::new(sign * 1.18, 0.0, 0.03),
                    Vec3::new(sign * 1.46, 0.0, 0.66),
                    0.035,
                    WOOD,
                );
                tube(
                    &mut body,
                    Vec3::new(sign * 1.46, 0.0, 0.66),
                    Vec3::new(sign * 1.57, 0.0, 0.77),
                    0.025,
                    GOLD,
                );
                for x in [-0.82, -0.27, 0.27, 0.82] {
                    shell(
                        &mut body,
                        Vec3::new(x, sign * 0.40, -0.13),
                        &[(0.055, 0.018, -0.035), (0.055, 0.018, 0.055)],
                        0.01,
                        GOLD,
                    );
                }
            }
            for face in &mut body {
                for p in &mut face.surface.vertices {
                    *p = c + along * p.x + side * p.y + Vec3::Z * p.z;
                }
            }
            let pivot = Vec3::new(
                ride.origin.x as f32
                    + if ride.rotation.is_multiple_of(2) {
                        2.0
                    } else {
                        1.0
                    },
                ride.origin.y as f32
                    + if ride.rotation.is_multiple_of(2) {
                        1.0
                    } else {
                        2.0
                    },
                ride.height as f32 + 5.3,
            );
            for sign in [-1.0, 1.0] {
                tube(
                    &mut body,
                    pivot + along * sign * 1.15,
                    c + along * sign * 1.15,
                    0.035,
                    RED,
                );
            }
            out.extend(ride_detail::project(body, size, rotation));
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

#[cfg(test)]
mod detail_tests {
    use super::*;

    fn world_point(p: Vec3, size: u32, camera: u8) -> Vec3 {
        let origin = at(Vec3::ZERO, size, camera);
        Mat3::from_cols(
            at(Vec3::X, size, camera) - origin,
            at(Vec3::Y, size, camera) - origin,
            at(Vec3::Z, size, camera) - origin,
        )
        .inverse()
            * (p - origin)
    }

    #[test]
    fn rounded_flat_ride_art_stays_inside_reserved_volume_in_every_orientation() {
        // A lower path or neighbouring ride must not be covered by geometry
        // outside the attraction's reserved construction cells.
        for model in [
            RideModel::FerrisWheel,
            RideModel::TeaCups,
            RideModel::ObservationTower,
            RideModel::PirateShip,
        ] {
            for orientation in 0..4 {
                let mut park = Park::new(32, 32);
                let origin = TilePos { x: 10, y: 12 };
                let id = park
                    .build_catalogue_ride(model, origin, 2, orientation)
                    .unwrap();
                let ride = park.carousel(id).unwrap();
                for camera in 0..4 {
                    let mut faces = static_faces(model, origin, 2, orientation, 32, camera);
                    faces.extend(dynamic_faces(&park, ride, camera));
                    let (w, d) = model.dimensions();
                    let (w, d) = if orientation % 2 == 1 { (d, w) } else { (w, d) };
                    for p in faces.iter().flat_map(|f| f.surface.vertices) {
                        let p = world_point(p, 32, camera);
                        assert!(p.is_finite());
                        assert!(
                            p.x >= origin.x as f32 - 0.51
                                && p.x <= origin.x as f32 + w as f32 - 0.49,
                            "{model:?} {orientation} {camera} {p:?}"
                        );
                        assert!(
                            p.y >= origin.y as f32 - 0.51
                                && p.y <= origin.y as f32 + d as f32 - 0.49,
                            "{model:?} {orientation} {camera} {p:?}"
                        );
                        assert!(
                            (1.99..=2.01 + f32::from(model.clearance())).contains(&p.z),
                            "{model:?} {p:?}"
                        );
                    }
                }
            }
        }
    }
}
