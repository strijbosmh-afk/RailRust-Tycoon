//! Original one-tile facilities; the same solid faces drive all presentation.
use park_sim::{ObjectId, PathDirection, ServiceKind};

use crate::path_tools::ServicePreview;

use super::*;

pub(super) struct Face {
    pub(super) surface: Surface,
    pub(super) ink: Ink,
}

fn projected(
    object: &SurfaceObject,
    facing: PathDirection,
    point: Vec3,
    size: u32,
    rotation: u8,
) -> Vec3 {
    let (dx, dy) = facing.offset();
    let front = Vec2::new(dx as f32, dy as f32);
    let side = Vec2::new(-front.y, front.x);
    let offset = rotated_offset(front * point.x + side * point.y, rotation);
    let (u, v) = rotate(object.pos, size, rotation);
    project(
        u as f32 + offset.x,
        v as f32 + offset.y,
        f32::from(object.height) + point.z,
        size,
    )
}

pub(super) fn center(
    object: &SurfaceObject,
    facing: PathDirection,
    size: u32,
    rotation: u8,
) -> Vec2 {
    projected(object, facing, Vec3::new(0.0, 0.0, 2.72), size, rotation).truncate()
}

pub(super) fn lit_counter(
    mesh: &mut Painter,
    park: &Park,
    object: &SurfaceObject,
    service: &park_sim::Service,
    rotation: u8,
    strength: f32,
) {
    if matches!(service.kind, ServiceKind::Toilet | ServiceKind::Rest) {
        return;
    }
    // Behind the product models, in front of the back panel of the serving hatch.
    let corner = |y, z| {
        projected(
            object,
            service.facing,
            Vec3::new(-0.19, y, z),
            park.width(),
            rotation,
        )
    };
    let [a, b, c, d] = [
        corner(-0.25, 1.28),
        corner(0.25, 1.28),
        corner(0.25, 1.90),
        corner(-0.25, 1.90),
    ];
    let ink = [1.0, 0.71, 0.26, 0.84 * strength];
    mesh.triangle_3d([a, b, c], ink);
    mesh.triangle_3d([a, c, d], ink);
}

/// Every box remains inside the one-tile footprint and three-step clearance.
/// Local +X faces the PUBLIC serving edge, independent of camera rotation.
pub(super) fn faces(
    park: &Park,
    object: &SurfaceObject,
    kind: ServiceKind,
    facing: PathDirection,
    rotation: u8,
) -> Vec<Face> {
    let size = park.width();
    let mut result = Vec::with_capacity(160);
    let cream = [rgb(255, 239, 182), rgb(215, 185, 121), rgb(173, 139, 88)];
    let dark = [rgb(76, 101, 89), rgb(53, 77, 66), rgb(40, 62, 53)];
    let accent = match kind {
        ServiceKind::Food => [rgb(248, 107, 52), rgb(191, 65, 39), rgb(137, 44, 34)],
        ServiceKind::Drink => [rgb(43, 196, 235), rgb(25, 129, 192), rgb(23, 81, 149)],
        ServiceKind::Toilet => [rgb(64, 195, 138), rgb(38, 137, 104), rgb(33, 92, 75)],
        ServiceKind::Rest => [rgb(185, 93, 214), rgb(119, 63, 172), rgb(76, 44, 126)],
        ServiceKind::IceCream => [rgb(255, 135, 191), rgb(203, 62, 123), rgb(141, 35, 89)],
        ServiceKind::Souvenir => [rgb(251, 192, 40), rgb(211, 122, 20), rgb(137, 66, 13)],
        ServiceKind::Information => [rgb(51, 162, 245), rgb(24, 97, 192), rgb(17, 57, 116)],
        ServiceKind::FirstAid => [rgb(239, 67, 73), rgb(178, 35, 42), rgb(119, 25, 31)],
    };
    let roof_height = if matches!(
        kind,
        ServiceKind::IceCream
            | ServiceKind::Souvenir
            | ServiceKind::Information
            | ServiceKind::FirstAid
    ) {
        2.40
    } else {
        2.72
    };
    let mut cuboid = |min: Vec3, max: Vec3, colors: [Ink; 3]| {
        let upper = [
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(max.x, min.y, max.z),
        ];
        let lower = upper.map(|p| Vec3::new(p.x, p.y, min.z));
        let mut face = |points: [Vec3; 4], ink| {
            let surface = Surface {
                vertices: points.map(|p| projected(object, facing, p, size, rotation)),
                kind: SurfaceKind::Top,
            };
            if surface.has_area() {
                result.push(Face { surface, ink });
            }
        };
        face(upper, colors[0]);
        for index in 0..4 {
            let next = (index + 1) % 4;
            face(
                [upper[index], upper[next], lower[next], lower[index]],
                colors[1 + index % 2],
            );
        }
    };
    let ground = park
        .tile(object.pos)
        .map_or(object.height, |tile| tile.height);
    let crossing_below = park.objects().iter().any(|other| {
        other.id != object.id
            && other.pos == object.pos
            && other.height < object.height
            && u16::from(other.height) + u16::from(other.clearance) > u16::from(ground)
    });
    if object.height > ground && !crossing_below {
        let bottom = f32::from(ground) - f32::from(object.height);
        let timber = [rgb(168, 126, 82), rgb(121, 84, 54), rgb(91, 62, 43)];
        for x in [-0.40, 0.30] {
            for y in [-0.40, 0.30] {
                cuboid(
                    Vec3::new(x, y, bottom),
                    Vec3::new(x + 0.10, y + 0.10, 0.0),
                    timber,
                );
            }
        }
    }
    // Neutral entrance apron reaches the serving edge without extending out of
    // the reserved tile. Public path joins remain separate navigation geometry.
    cuboid(
        Vec3::new(-0.46, -0.46, 0.0),
        Vec3::new(0.5, 0.46, PATH_TOP_OFFSET),
        cream,
    );
    if matches!(
        kind,
        ServiceKind::IceCream
            | ServiceKind::Souvenir
            | ServiceKind::Information
            | ServiceKind::FirstAid
    ) {
        cuboid(
            Vec3::new(-0.42, -0.42, 0.20),
            Vec3::new(0.28, 0.42, 2.20),
            cream,
        );
        cuboid(
            Vec3::new(-0.46, -0.46, 2.20),
            Vec3::new(0.40, 0.46, 2.40),
            accent,
        );
        cuboid(
            Vec3::new(0.28, -0.30, 1.00),
            Vec3::new(0.31, 0.30, 1.95),
            dark,
        );
        cuboid(
            Vec3::new(0.28, -0.38, 0.25),
            Vec3::new(0.43, 0.38, 1.03),
            accent,
        );
        cuboid(
            Vec3::new(0.27, -0.40, 1.00),
            Vec3::new(0.46, 0.40, 1.14),
            cream,
        );
        match kind {
            ServiceKind::IceCream => {
                for stripe in 0..6 {
                    let y = -0.42 + stripe as f32 * 0.14;
                    cuboid(
                        Vec3::new(0.27, y, 1.96),
                        Vec3::new(0.49, y + 0.14, 2.10),
                        if stripe % 2 == 0 { cream } else { accent },
                    );
                }
                for (y, color) in [
                    (-0.22, cream),
                    (-0.02, accent),
                    (0.18, [rgb(113, 214, 184); 3]),
                ] {
                    cuboid(
                        Vec3::new(0.32, y, 1.14),
                        Vec3::new(0.42, y + 0.12, 1.30),
                        color,
                    );
                }
            }
            ServiceKind::Souvenir => {
                for y in [-0.32, 0.28] {
                    cuboid(
                        Vec3::new(0.32, y, 1.14),
                        Vec3::new(0.44, y + 0.04, 1.94),
                        cream,
                    );
                }
                cuboid(
                    Vec3::new(0.33, -0.30, 1.44),
                    Vec3::new(0.43, 0.30, 1.49),
                    dark,
                );
                for (i, y) in [-0.23, 0.0, 0.23].iter().enumerate() {
                    cuboid(
                        Vec3::new(0.32, *y, 1.14),
                        Vec3::new(0.43, *y + 0.13, 1.38),
                        if i % 2 == 0 {
                            accent
                        } else {
                            [rgb(46, 168, 223); 3]
                        },
                    );
                }
                cuboid(
                    Vec3::new(-0.2, -0.2, 2.40),
                    Vec3::new(0.20, 0.20, 2.84),
                    accent,
                );
                cuboid(
                    Vec3::new(-0.21, -0.045, 2.40),
                    Vec3::new(0.21, 0.045, 2.86),
                    [rgb(234, 67, 106); 3],
                );
            }
            ServiceKind::Information => {
                cuboid(
                    Vec3::new(-0.245, -0.30, 1.22),
                    Vec3::new(-0.22, 0.30, 1.90),
                    [rgb(113, 196, 133); 3],
                );
                cuboid(
                    Vec3::new(0.28, -0.02, 1.4),
                    Vec3::new(0.32, 0.08, 1.84),
                    cream,
                );
                cuboid(
                    Vec3::new(-0.04, -0.04, 2.40),
                    Vec3::new(0.04, 0.04, 2.83),
                    cream,
                );
                cuboid(
                    Vec3::new(-0.05, -0.05, 2.88),
                    Vec3::new(0.05, 0.05, 2.98),
                    cream,
                );
            }
            ServiceKind::FirstAid => {
                cuboid(
                    Vec3::new(-0.055, -0.20, 2.42),
                    Vec3::new(0.055, 0.20, 2.97),
                    accent,
                );
                cuboid(
                    Vec3::new(-0.055, -0.32, 2.61),
                    Vec3::new(0.055, 0.32, 2.77),
                    accent,
                );
                cuboid(
                    Vec3::new(0.33, -0.07, 1.14),
                    Vec3::new(0.43, 0.07, 1.39),
                    cream,
                );
            }
            _ => unreachable!(),
        }
    } else if kind == ServiceKind::Toilet {
        cuboid(
            Vec3::new(-0.42, -0.4, PATH_TOP_OFFSET),
            Vec3::new(0.30, 0.4, 2.50),
            cream,
        );
        // A dark recessed-looking door, handle and small roof vent distinguish
        // the enclosed cabin from both open serving counters.
        cuboid(
            Vec3::new(0.30, -0.18, 0.23),
            Vec3::new(0.315, 0.18, 1.99),
            dark,
        );
        cuboid(
            Vec3::new(0.315, 0.105, 1.12),
            Vec3::new(0.335, 0.145, 1.18),
            cream,
        );
        cuboid(
            Vec3::new(-0.46, -0.45, 2.50),
            Vec3::new(0.42, 0.45, 2.72),
            accent,
        );
        cuboid(
            Vec3::new(-0.28, -0.16, 2.72),
            Vec3::new(-0.14, 0.16, 2.91),
            dark,
        );
        // Door plaque and paired vents remain inside the one-tile cabin.
        cuboid(
            Vec3::new(0.316, -0.17, 2.08),
            Vec3::new(0.335, 0.17, 2.30),
            accent,
        );
        for y in [-0.27, 0.21] {
            cuboid(
                Vec3::new(0.304, y, 1.57),
                Vec3::new(0.326, y + 0.06, 1.72),
                dark,
            );
        }
    } else if kind == ServiceKind::Rest {
        // An open pavilion and bench show the place to recover, while keeping
        // the serving edge and one-tile picking footprint clear.
        cuboid(
            Vec3::new(-0.44, -0.44, 2.48),
            Vec3::new(0.44, 0.44, 2.72),
            accent,
        );
        for x in [-0.38, 0.31] {
            for y in [-0.37, 0.30] {
                cuboid(
                    Vec3::new(x, y, 0.20),
                    Vec3::new(x + 0.07, y + 0.07, 2.48),
                    dark,
                );
            }
        }
        cuboid(
            Vec3::new(-0.28, -0.34, 0.58),
            Vec3::new(0.24, 0.29, 0.72),
            accent,
        );
        cuboid(
            Vec3::new(-0.29, -0.34, 0.72),
            Vec3::new(-0.18, 0.29, 1.48),
            cream,
        );
        for y in [-0.29, 0.20] {
            cuboid(
                Vec3::new(-0.22, y, 0.20),
                Vec3::new(0.18, y + 0.09, 0.58),
                dark,
            );
        }
        for y in [-0.26, -0.10, 0.06, 0.22] {
            cuboid(
                Vec3::new(-0.17, y, 0.72),
                Vec3::new(0.19, y + 0.045, 0.78),
                cream,
            );
        }
        cuboid(
            Vec3::new(-0.07, -0.07, 2.14),
            Vec3::new(0.07, 0.07, 2.43),
            cream,
        );
    } else {
        // Counter, rear cabinet, corner posts and canopy. The open gap between
        // counter and canopy stays genuinely unpickable.
        cuboid(
            Vec3::new(-0.40, -0.37, 0.22),
            Vec3::new(-0.25, 0.37, 2.39),
            cream,
        );
        cuboid(
            Vec3::new(-0.22, -0.38, 0.22),
            Vec3::new(0.30, 0.38, 1.08),
            accent,
        );
        cuboid(
            Vec3::new(-0.25, -0.41, 1.08),
            Vec3::new(0.35, 0.41, 1.23),
            cream,
        );
        for y in [-0.39, 0.34] {
            cuboid(
                Vec3::new(0.25, y, 1.23),
                Vec3::new(0.30, y + 0.05, 2.46),
                dark,
            );
        }
        cuboid(
            Vec3::new(-0.46, -0.46, 2.46),
            Vec3::new(0.44, 0.46, 2.72),
            accent,
        );
        if kind == ServiceKind::Food {
            cuboid(
                Vec3::new(-0.10, 0.12, 1.23),
                Vec3::new(0.21, 0.33, 1.31),
                dark,
            );
            for y in [0.16, 0.25] {
                cuboid(
                    Vec3::new(-0.05, y, 1.31),
                    Vec3::new(0.15, y + 0.04, 1.34),
                    cream,
                );
            }
            for y in [-0.34, -0.08, 0.18] {
                cuboid(
                    Vec3::new(-0.44, y, 2.72),
                    Vec3::new(0.42, y + 0.10, 2.73),
                    cream,
                );
            }
            // Two small bread-like blocks are original procedural counter art.
            cuboid(
                Vec3::new(0.04, -0.22, 1.23),
                Vec3::new(0.23, -0.04, 1.38),
                cream,
            );
            cuboid(
                Vec3::new(0.04, 0.04, 1.23),
                Vec3::new(0.23, 0.22, 1.38),
                cream,
            );
            cuboid(
                Vec3::new(-0.245, -0.24, 1.46),
                Vec3::new(-0.215, 0.24, 2.18),
                dark,
            );
            for z in [1.66, 1.86, 2.06] {
                cuboid(
                    Vec3::new(-0.214, -0.16, z),
                    Vec3::new(-0.195, 0.16, z + 0.045),
                    cream,
                );
            }
        } else {
            // A tall dispenser and two contrasting cups make the drinks stall
            // recognizable by silhouette as well as its blue palette.
            cuboid(
                Vec3::new(-0.18, -0.19, 1.23),
                Vec3::new(0.04, 0.19, 2.05),
                accent,
            );
            for y in [-0.27, 0.15] {
                cuboid(
                    Vec3::new(0.12, y, 1.23),
                    Vec3::new(0.23, y + 0.12, 1.55),
                    cream,
                );
                cuboid(
                    Vec3::new(0.16, y + 0.045, 1.55),
                    Vec3::new(0.18, y + 0.07, 1.80),
                    dark,
                );
            }
            for y in [-0.11, 0.08] {
                cuboid(
                    Vec3::new(0.041, y, 1.74),
                    Vec3::new(0.095, y + 0.045, 1.81),
                    cream,
                );
            }
        }
    }
    if matches!(kind, ServiceKind::Food | ServiceKind::Drink) {
        // Bright striped valance faces the serving path; small alternating
        // fabric panels stop short of the neighboring tile and use real depth.
        for stripe in 0..6 {
            let y = -0.43 + stripe as f32 * 0.145;
            cuboid(
                Vec3::new(0.445, y, 2.38),
                Vec3::new(0.465, y + 0.083, 2.69),
                if stripe % 2 == 0 { cream } else { accent },
            );
        }
    }
    cuboid(
        Vec3::new(-0.065, -0.065, roof_height + 0.02),
        Vec3::new(0.065, 0.065, roof_height + 0.13),
        accent,
    );
    // Layered plinths, cornices and corner pilasters make the small facades
    // readable from every view. All trim stays inside the reserved tile.
    let brass = [rgb(247, 202, 91), rgb(187, 133, 45), rgb(123, 83, 29)];
    let ivory = [rgb(255, 248, 215), rgb(227, 207, 158), rgb(179, 153, 110)];
    for y in [-0.40, 0.34] {
        cuboid(
            Vec3::new(0.27, y, 0.24),
            Vec3::new(0.35, y + 0.06, 2.23),
            ivory,
        );
        cuboid(
            Vec3::new(0.26, y - 0.01, 0.23),
            Vec3::new(0.36, y + 0.07, 0.39),
            brass,
        );
        cuboid(
            Vec3::new(0.26, y - 0.01, 2.15),
            Vec3::new(0.36, y + 0.07, 2.26),
            brass,
        );
    }
    for z in [0.30, 0.58, 0.86, 1.14, 1.42, 1.70, 1.98] {
        cuboid(
            Vec3::new(-0.425, -0.41, z),
            Vec3::new(-0.415, 0.41, z + 0.025),
            ivory,
        );
    }
    for y in [-0.425, 0.413] {
        cuboid(
            Vec3::new(-0.39, y, 0.32),
            Vec3::new(0.20, y + 0.012, 0.45),
            brass,
        );
        cuboid(
            Vec3::new(-0.36, y, 1.16),
            Vec3::new(-0.03, y + 0.012, 1.90),
            dark,
        );
        cuboid(
            Vec3::new(-0.205, y - 0.003, 1.16),
            Vec3::new(-0.185, y + 0.02, 1.90),
            ivory,
        );
        cuboid(
            Vec3::new(-0.36, y - 0.003, 1.51),
            Vec3::new(-0.03, y + 0.02, 1.55),
            ivory,
        );
    }
    cuboid(
        Vec3::new(-0.45, -0.45, 2.37),
        Vec3::new(0.39, 0.45, 2.43),
        brass,
    );
    if !matches!(kind, ServiceKind::Rest | ServiceKind::Toilet) {
        // Counter attendant, menu boards and warm little lamps remain solid
        // geometry, so their visible faces are also valid inspection targets.
        cuboid(
            Vec3::new(-0.12, -0.085, 1.10),
            Vec3::new(0.04, 0.085, 1.58),
            accent,
        );
        cuboid(
            Vec3::new(-0.11, -0.10, 1.83),
            Vec3::new(0.04, 0.10, 1.89),
            ivory,
        );
        for y in [-0.32, 0.22] {
            cuboid(
                Vec3::new(0.305, y, 1.32),
                Vec3::new(0.326, y + 0.10, 1.93),
                brass,
            );
            cuboid(
                Vec3::new(0.327, y + 0.012, 1.36),
                Vec3::new(0.333, y + 0.087, 1.89),
                dark,
            );
            for z in [1.47, 1.61, 1.75] {
                cuboid(
                    Vec3::new(0.334, y + 0.025, z),
                    Vec3::new(0.34, y + 0.075, z + 0.025),
                    ivory,
                );
            }
        }
        for y in [-0.39, 0.33] {
            cuboid(
                Vec3::new(0.35, y, 1.97),
                Vec3::new(0.42, y + 0.06, 2.14),
                brass,
            );
            cuboid(
                Vec3::new(0.36, y + 0.01, 2.01),
                Vec3::new(0.43, y + 0.05, 2.11),
                ivory,
            );
        }
    }
    // Layered teal shingles and brass edging replace the flat roof slab.
    for row in 0..4 {
        for column in 0..6 {
            let x = -0.43 + row as f32 * 0.205;
            let y = -0.43 + column as f32 * 0.143;
            cuboid(
                Vec3::new(x, y, roof_height + 0.013),
                Vec3::new(x + 0.19, y + 0.128, roof_height + 0.025),
                if (row + column) % 3 == 0 {
                    [rgb(69, 145, 125), rgb(39, 102, 89), rgb(26, 76, 70)]
                } else {
                    [rgb(48, 118, 102), rgb(31, 86, 76), rgb(20, 63, 58)]
                },
            );
        }
    }
    let mut detail = Vec::new();
    let glass = [rgb(159, 223, 224), rgb(65, 151, 169), rgb(36, 101, 130)];
    for side in [-1.0, 1.0] {
        let y = side * 0.43;
        for row in 0..3 {
            let z = 1.22 + row as f32 * 0.21;
            ride_detail::panel(
                &mut detail,
                [
                    Vec3::new(-0.34, y, z),
                    Vec3::new(-0.23, y, z),
                    Vec3::new(-0.23, y, z + 0.17),
                    Vec3::new(-0.34, y, z + 0.17),
                ],
                glass[1],
            );
            ride_detail::panel(
                &mut detail,
                [
                    Vec3::new(-0.18, y, z),
                    Vec3::new(-0.07, y, z),
                    Vec3::new(-0.07, y, z + 0.17),
                    Vec3::new(-0.18, y, z + 0.17),
                ],
                glass[0],
            );
        }
        // Narrow brass sill and a pair of leafy window-box plants.
        ride_detail::tube(
            &mut detail,
            Vec3::new(-0.36, y, 1.16),
            Vec3::new(-0.02, y, 1.16),
            0.014,
            brass,
        );
        for x in [-0.29, -0.10] {
            ride_detail::lathe(
                &mut detail,
                Vec3::new(x, side * 0.45, 0.49),
                &[(0.026, 0.0), (0.036, 0.13), (0.037, 0.16)],
                accent,
            );
            ride_detail::lathe(
                &mut detail,
                Vec3::new(x, side * 0.45, 0.65),
                &[(0.0, 0.0), (0.043, 0.035), (0.035, 0.11), (0.0, 0.15)],
                [rgb(125, 185, 75), rgb(59, 139, 63), rgb(35, 92, 51)],
            );
        }
    }
    if !matches!(kind, ServiceKind::Rest | ServiceKind::Toilet) {
        ride_detail::lathe(
            &mut detail,
            Vec3::new(-0.035, 0.0, 1.58),
            &[
                (0.045, 0.0),
                (0.076, 0.06),
                (0.073, 0.18),
                (0.04, 0.26),
                (0.0, 0.27),
            ],
            [rgb(247, 199, 150), rgb(221, 163, 114), rgb(176, 113, 79)],
        );
        for y in [-0.068, 0.055] {
            ride_detail::tube(
                &mut detail,
                Vec3::new(0.02, y, 1.48),
                Vec3::new(0.22, y, 1.25),
                0.025,
                ivory,
            );
        }
        for y in [-0.29, 0.25] {
            ride_detail::lathe(
                &mut detail,
                Vec3::new(0.395, y, 2.02),
                &[
                    (0.0, 0.0),
                    (0.045, 0.015),
                    (0.05, 0.09),
                    (0.035, 0.14),
                    (0.0, 0.14),
                ],
                ivory,
            );
        }
    }
    match kind {
        ServiceKind::Food => {
            for (z, radius, colors) in [
                (
                    2.73,
                    0.16,
                    [rgb(250, 191, 77), rgb(203, 126, 43), rgb(155, 81, 33)],
                ),
                (
                    2.80,
                    0.17,
                    [rgb(131, 184, 65), rgb(77, 133, 46), rgb(48, 96, 41)],
                ),
                (
                    2.84,
                    0.16,
                    [rgb(126, 68, 37), rgb(87, 47, 29), rgb(68, 37, 26)],
                ),
                (
                    2.88,
                    0.16,
                    [rgb(255, 205, 82), rgb(218, 146, 48), rgb(165, 91, 29)],
                ),
            ] {
                ride_detail::lathe(
                    &mut detail,
                    Vec3::new(0.0, 0.0, z),
                    &[
                        (radius, 0.0),
                        (radius, 0.035),
                        (radius * 0.7, 0.07),
                        (0.0, 0.08),
                    ],
                    colors,
                );
            }
        }
        ServiceKind::Drink => {
            ride_detail::lathe(
                &mut detail,
                Vec3::new(0.0, 0.0, 2.73),
                &[(0.08, 0.0), (0.12, 0.20), (0.13, 0.22), (0.0, 0.22)],
                accent,
            );
            ride_detail::tube(
                &mut detail,
                Vec3::new(0.03, 0.0, 2.86),
                Vec3::new(0.03, 0.0, 2.99),
                0.014,
                ivory,
            );
        }
        ServiceKind::IceCream => {
            ride_detail::lathe(
                &mut detail,
                Vec3::new(0.0, 0.0, 2.4),
                &[(0.035, 0.0), (0.12, 0.27), (0.13, 0.29)],
                [rgb(247, 195, 109), rgb(205, 142, 62), rgb(152, 92, 37)],
            );
            ride_detail::lathe(
                &mut detail,
                Vec3::new(0.0, 0.0, 2.64),
                &[
                    (0.08, 0.0),
                    (0.17, 0.06),
                    (0.17, 0.16),
                    (0.12, 0.26),
                    (0.0, 0.33),
                ],
                [rgb(255, 231, 237), rgb(255, 175, 208), rgb(219, 112, 164)],
            );
        }
        ServiceKind::Rest => {
            for y in [-0.25, -0.12, 0.01, 0.14, 0.27] {
                ride_detail::tube(
                    &mut detail,
                    Vec3::new(-0.14, y, 0.75),
                    Vec3::new(0.22, y, 0.75),
                    0.015,
                    brass,
                );
            }
        }
        ServiceKind::Toilet => {
            ride_detail::lathe(
                &mut detail,
                Vec3::new(-0.21, 0.0, 2.89),
                &[(0.11, 0.0), (0.12, 0.03), (0.08, 0.075), (0.0, 0.075)],
                ivory,
            );
        }
        ServiceKind::Souvenir | ServiceKind::Information | ServiceKind::FirstAid => {
            for y in [-0.19, 0.0, 0.19] {
                ride_detail::lathe(
                    &mut detail,
                    Vec3::new(0.375, y, 1.14),
                    &[(0.032, 0.0), (0.044, 0.08), (0.035, 0.16), (0.0, 0.19)],
                    if kind == ServiceKind::FirstAid {
                        ivory
                    } else {
                        accent
                    },
                );
            }
        }
    }
    for mut face in detail {
        face.surface.vertices = face
            .surface
            .vertices
            .map(|p| projected(object, facing, p, size, rotation));
        if face.surface.has_area() {
            result.push(face);
        }
    }
    result
}

pub fn service_preview_mesh(preview: Option<ServicePreview>, park: &Park, rotation: u8) -> Mesh {
    let mut painter = Painter::default();
    let Some(preview) = preview else {
        return painter.finish();
    };
    if park.width() != park.height() || park.tile(preview.pos).is_none() || preview.height > 29 {
        return painter.finish();
    }
    let object = SurfaceObject {
        id: ObjectId(0),
        pos: preview.pos,
        kind: ObjectKind::Service,
        axis: Axis::X,
        height: preview.height,
        clearance: 3,
        path_slope: PathSlope::Flat,
        queue_owner: None,
    };
    let faces = faces(park, &object, preview.kind, preview.facing, rotation);
    let mut ink = if !preview.valid {
        rgb(225, 68, 63)
    } else if preview.demolish {
        rgb(235, 120, 51)
    } else {
        rgb(47, 183, 114)
    };
    for face in &faces {
        let mut fill = face.surface;
        for vertex in &mut fill.vertices {
            vertex.z += SELECTION_BIAS;
        }
        ink[3] = 0.22;
        painter.surface(fill, ink);
        ink[3] = 0.95;
        painter.surface_outline(face.surface, 1.2, SELECTION_BIAS * 2.0, ink);
    }
    if (!preview.valid || preview.demolish)
        && let Some(platform) = faces.first()
    {
        let corners = platform.surface.vertices.map(Vec3::truncate);
        for (a, b) in [(corners[0], corners[2]), (corners[1], corners[3])] {
            painter.surface_line(platform.surface, a, b, 1.5, SELECTION_BIAS * 3.0, ink);
        }
    }
    painter.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
    }

    #[test]
    fn facility_faces_picking_selection_and_ghosts_share_physical_geometry() {
        for kind in ServiceKind::ALL {
            for facing in PathDirection::ALL {
                let mut park = Park::new(12, 12);
                let id = park
                    .place_service(TilePos { x: 5, y: 5 }, 8, facing, kind)
                    .unwrap();
                let object = park
                    .object(park.service(id).unwrap().object.unwrap())
                    .unwrap();
                for rotation in 0..4 {
                    let faces = faces(&park, object, kind, facing, rotation);
                    assert!(!faces.is_empty());
                    let target = SurfaceId::Object(object.id);
                    for mode in [ViewMode::Surface, ViewMode::Underground] {
                        let selectable = selectable_surfaces(&park, target, rotation, mode);
                        assert_eq!(selectable.len(), faces.len());
                        for (actual, expected) in selectable.iter().zip(&faces) {
                            assert_eq!(actual.vertices, expected.surface.vertices);
                        }
                        let center = surface_center(&park, target, rotation).unwrap();
                        assert_eq!(pick_surface(center, &park, rotation, mode), Some(target));
                        assert!(
                            !positions(&selection_mesh(None, Some(target), &park, rotation, mode))
                                .is_empty()
                        );
                    }
                    let before = park.clone();
                    let preview = service_preview_mesh(
                        Some(ServicePreview {
                            pos: object.pos,
                            height: 8,
                            kind,
                            facing,
                            valid: true,
                            demolish: false,
                        }),
                        &park,
                        rotation,
                    );
                    assert!(
                        positions(&preview)
                            .iter()
                            .flatten()
                            .all(|value| value.is_finite())
                    );
                    for face in &faces {
                        for point in face.surface.vertices {
                            assert!(positions(&preview).iter().any(|candidate| {
                                (candidate[0] - point.x).abs() < 0.001
                                    && (candidate[1] - point.y).abs() < 0.001
                                    && (candidate[2] - point.z - SELECTION_BIAS).abs() < 0.0001
                            }));
                        }
                    }
                    let Some(VertexAttributeValues::Float32x4(colors)) =
                        preview.attribute(Mesh::ATTRIBUTE_COLOR)
                    else {
                        panic!("colors")
                    };
                    assert!(colors.iter().any(|color| color[3] == 0.22));
                    assert!(colors.iter().any(|color| color[3] == 0.95));
                    assert_eq!(park, before);
                }
            }
        }
    }

    #[test]
    fn service_art_and_foundations_are_distinct_with_finite_vertices_at_height_limits() {
        for height in [0, 29] {
            let mut kind_meshes = Vec::new();
            for kind in ServiceKind::ALL {
                let mut park = Park::new(12, 12);
                let id = park
                    .place_service(TilePos { x: 5, y: 5 }, height, PathDirection::South, kind)
                    .unwrap();
                let object = park
                    .object(park.service(id).unwrap().object.unwrap())
                    .unwrap();
                let mut painter = Painter::default();
                for face in faces(&park, object, kind, PathDirection::South, 0) {
                    painter.surface(face.surface, face.ink);
                }
                let mesh = painter.finish();
                for &[x, y, z] in positions(&mesh) {
                    // Invert projection using depth to recover actual world height.
                    let sum = z - BASE_DEPTH;
                    let difference = x / HALF_WIDTH;
                    let u = (sum + difference) * 0.5;
                    let v = (sum - difference) * 0.5;
                    let h = (y + (sum - 11.0) * HALF_HEIGHT) / HEIGHT_STEP;
                    assert!((4.499..=5.501).contains(&u) && (4.499..=5.501).contains(&v));
                    assert!(
                        (-0.001..f32::from(height) + 3.0).contains(&h),
                        "{kind:?} at {height}: ({x}, {y}, {z}), height {h}"
                    );
                }
                kind_meshes.push(positions(&mesh).to_vec());
            }
            assert_ne!(kind_meshes[0], kind_meshes[1]);
            assert_ne!(kind_meshes[1], kind_meshes[2]);
            assert_ne!(kind_meshes[2], kind_meshes[3]);
        }
    }

    #[test]
    fn raised_facility_posts_end_at_land_and_avoid_a_lower_crossing() {
        let pos = TilePos { x: 5, y: 5 };
        let mut raised = Park::new(12, 12);
        raised.set_terrain_height(pos, 3).unwrap();
        let id = raised
            .place_service(pos, 8, PathDirection::North, ServiceKind::Rest)
            .unwrap();
        let object = raised
            .object(raised.service(id).unwrap().object.unwrap())
            .unwrap();
        let supported = faces(&raised, object, ServiceKind::Rest, PathDirection::North, 0);

        let mut flush = Park::new(12, 12);
        flush.set_terrain_height(pos, 3).unwrap();
        let flush_id = flush
            .place_service(pos, 3, PathDirection::North, ServiceKind::Rest)
            .unwrap();
        let flush_object = flush
            .object(flush.service(flush_id).unwrap().object.unwrap())
            .unwrap();
        let plain = faces(
            &flush,
            flush_object,
            ServiceKind::Rest,
            PathDirection::North,
            0,
        );
        assert_eq!(supported.len(), plain.len() + 20);

        let mut crossing = Park::new(12, 12);
        crossing.set_terrain_height(pos, 3).unwrap();
        crossing.place_path(pos, 3).unwrap();
        let crossing_id = crossing
            .place_service(pos, 8, PathDirection::North, ServiceKind::Rest)
            .unwrap();
        let crossing_object = crossing
            .object(crossing.service(crossing_id).unwrap().object.unwrap())
            .unwrap();
        assert_eq!(
            faces(
                &crossing,
                crossing_object,
                ServiceKind::Rest,
                PathDirection::North,
                0
            )
            .len(),
            plain.len()
        );
    }

    #[test]
    fn service_ghosts_do_not_bypass_foreground_occlusion_and_empty_preview_is_empty() {
        for rotation in 0..4 {
            let mut park = Park::new(12, 12);
            let pos = unrotate(5, 5, 12, rotation);
            let id = park
                .place_service(pos, 0, PathDirection::North, ServiceKind::Food)
                .unwrap();
            let object = park.service(id).unwrap().object.unwrap();
            let center = surface_center(&park, SurfaceId::Object(object), rotation).unwrap();
            let front = unrotate(8, 8, 12, rotation);
            park.set_terrain_height(front, 12).unwrap();
            assert_eq!(
                pick_surface(center, &park, rotation, ViewMode::Surface),
                Some(SurfaceId::Terrain(front))
            );
            assert_eq!(
                pick_surface(center, &park, rotation, ViewMode::Underground),
                Some(SurfaceId::Object(object))
            );
            let occluder = selectable_surfaces(
                &park,
                SurfaceId::Terrain(front),
                rotation,
                ViewMode::Surface,
            )
            .into_iter()
            .filter_map(|face| face.hit_depth(center))
            .reduce(f32::max)
            .unwrap();
            let selection = selection_mesh(
                None,
                Some(SurfaceId::Object(object)),
                &park,
                rotation,
                ViewMode::Surface,
            );
            assert!(positions(&selection).iter().all(|p| p[2] < occluder));
            let ghost = service_preview_mesh(
                Some(ServicePreview {
                    pos,
                    height: 0,
                    kind: ServiceKind::Food,
                    facing: PathDirection::North,
                    valid: true,
                    demolish: false,
                }),
                &park,
                rotation,
            );
            assert!(positions(&ghost).iter().all(|p| p[2] < occluder));
        }
        let park = Park::new(12, 12);
        assert_eq!(service_preview_mesh(None, &park, 0).count_vertices(), 0);
        let preview = ServicePreview {
            pos: TilePos { x: 5, y: 5 },
            height: 0,
            kind: ServiceKind::Food,
            facing: PathDirection::North,
            valid: false,
            demolish: false,
        };
        let invalid = service_preview_mesh(Some(preview), &park, 0);
        let valid = service_preview_mesh(
            Some(ServicePreview {
                valid: true,
                ..preview
            }),
            &park,
            0,
        );
        assert!(
            invalid.count_vertices() > valid.count_vertices(),
            "invalid X is geometry, not color alone"
        );
    }

    #[test]
    fn service_static_groups_preserve_all_faces_in_chunks_and_missing_metadata_is_hidden() {
        let mut park = Park::new(40, 40);
        let id = park
            .place_service(
                TilePos { x: 20, y: 20 },
                29,
                PathDirection::West,
                ServiceKind::Toilet,
            )
            .unwrap();
        let object = *park
            .object(park.service(id).unwrap().object.unwrap())
            .unwrap();
        let missing = SurfaceObject {
            id: ObjectId(u64::MAX),
            ..object
        };
        assert!(rendered_object_surfaces(&park, &missing, 0).is_empty());
        for rotation in 0..4 {
            let faces = faces(
                &park,
                &object,
                ServiceKind::Toilet,
                PathDirection::West,
                rotation,
            );
            let chunks = build_static_chunks(&park, rotation, true, ViewMode::Underground);
            // An object's complete group, including the tall vent and apron,
            // must land in one chunk without dropping or clipping any corner.
            assert!(chunks.iter().any(|chunk| {
                faces
                    .iter()
                    .flat_map(|face| face.surface.vertices)
                    .all(|point| {
                        positions(&chunk.mesh)
                            .iter()
                            .any(|candidate| *candidate == point.to_array())
                    })
            }));
            let bounds = scene_bounds(&park).unwrap();
            assert!(
                faces
                    .iter()
                    .flat_map(|face| face.surface.vertices)
                    .all(|point| bounds.contains(point.truncate()))
            );
        }
    }
}
