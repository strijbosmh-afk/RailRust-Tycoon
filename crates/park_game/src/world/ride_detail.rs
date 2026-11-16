//! Small solid primitives in unprojected park coordinates. Static, moving and
//! preview artwork share these faces, including their physical depth.
use super::*;

pub(super) type Face = service_render::Face;

pub(super) fn panel(out: &mut Vec<Face>, vertices: [Vec3; 4], ink: Ink) {
    out.push(Face {
        surface: Surface {
            vertices,
            kind: SurfaceKind::Top,
        },
        ink,
    });
}

/// A rounded horizontal shell. The profile runs from bottom to top and can
/// return inward to form a genuinely open bowl rather than a painted lid.
pub(super) fn lathe(out: &mut Vec<Face>, center: Vec3, profile: &[(f32, f32)], inks: [Ink; 3]) {
    const SEGMENTS: usize = 32;
    for pair in profile.windows(2) {
        for i in 0..SEGMENTS {
            let a = i as f32 * std::f32::consts::TAU / SEGMENTS as f32;
            let b = (i + 1) as f32 * std::f32::consts::TAU / SEGMENTS as f32;
            let point = |angle: f32, (radius, height): (f32, f32)| {
                center + Vec3::new(angle.cos() * radius, angle.sin() * radius, height)
            };
            // Three deliberate light bands keep the hand-painted look without
            // alternating dark stripes on each polygon of a round surface.
            let light = (a + b) * 0.5;
            let shade = if light.cos() + light.sin() < -0.35 {
                0
            } else if light.cos() + light.sin() > 0.75 {
                2
            } else {
                1
            };
            panel(
                out,
                [
                    point(a, pair[0]),
                    point(b, pair[0]),
                    point(b, pair[1]),
                    point(a, pair[1]),
                ],
                inks[shade],
            );
        }
    }
}

/// Eight-sided body with bevelled corners; profiles allow hulls and roofs to
/// taper. Width/depth are half-extents, bevel is the corner inset.
pub(super) fn shell(
    out: &mut Vec<Face>,
    center: Vec3,
    profile: &[(f32, f32, f32)],
    bevel: f32,
    inks: [Ink; 3],
) {
    let ring = |(x, y, z): (f32, f32, f32)| {
        let b = bevel.min(x).min(y);
        [
            (-x + b, -y),
            (x - b, -y),
            (x, -y + b),
            (x, y - b),
            (x - b, y),
            (-x + b, y),
            (-x, y - b),
            (-x, -y + b),
        ]
        .map(|(x, y)| center + Vec3::new(x, y, z))
    };
    for pair in profile.windows(2) {
        let lo = ring(pair[0]);
        let hi = ring(pair[1]);
        for i in 0..8 {
            let n = (i + 1) % 8;
            panel(
                out,
                [lo[i], lo[n], hi[n], hi[i]],
                inks[if i < 3 { 1 } else { 2 }],
            );
        }
    }
    if let Some(&last) = profile.last() {
        let rim = ring(last);
        for i in 0..8 {
            panel(
                out,
                [
                    center + Vec3::Z * last.2,
                    rim[i],
                    rim[(i + 1) % 8],
                    center + Vec3::Z * last.2,
                ],
                inks[0],
            );
        }
    }
}

/// Circular beam, with vertical units corrected to the park's height scale.
pub(super) fn tube(out: &mut Vec<Face>, a: Vec3, b: Vec3, radius: f32, inks: [Ink; 3]) {
    let metric = Vec3::new(1.0, 1.0, 0.25);
    let direction = ((b - a) * metric).normalize_or_zero();
    if direction == Vec3::ZERO {
        return;
    }
    let reference = if direction.z.abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::Z
    };
    let side = direction.cross(reference).normalize();
    let up = direction.cross(side);
    for i in 0..8 {
        let offset = |step: usize| {
            let angle = step as f32 * std::f32::consts::TAU / 8.0;
            (side * angle.cos() + up * angle.sin()) * radius / metric
        };
        let p = offset(i);
        let q = offset(i + 1);
        panel(
            out,
            [a + p, b + p, b + q, a + q],
            inks[if i < 3 {
                0
            } else if i < 6 {
                1
            } else {
                2
            }],
        );
    }
}

pub(super) fn project(mut faces: Vec<Face>, size: u32, rotation: u8) -> Vec<Face> {
    for face in &mut faces {
        face.surface.vertices = face
            .surface
            .vertices
            .map(|p| carousel_render::at(p, size, rotation));
    }
    faces.retain(|f| f.surface.has_area());
    faces
}

pub(super) fn horse(
    point: Vec3,
    forward: Vec3,
    coat: [Ink; 3],
    brass: [Ink; 3],
    saddle: [Ink; 3],
) -> Vec<Face> {
    let mut out = Vec::new();
    shell(
        &mut out,
        Vec3::ZERO,
        &[
            (0.15, 0.09, -0.25),
            (0.21, 0.13, -0.18),
            (0.19, 0.12, -0.06),
        ],
        0.06,
        coat,
    );
    shell(
        &mut out,
        Vec3::ZERO,
        &[(0.09, 0.135, -0.10), (0.08, 0.13, 0.0)],
        0.03,
        saddle,
    );
    shell(
        &mut out,
        Vec3::new(0.16, 0.0, 0.0),
        &[(0.075, 0.07, -0.16), (0.055, 0.055, 0.19)],
        0.025,
        coat,
    );
    shell(
        &mut out,
        Vec3::new(0.23, 0.0, 0.0),
        &[(0.09, 0.055, 0.10), (0.085, 0.05, 0.22)],
        0.035,
        coat,
    );
    let dark = [rgb(52, 43, 39); 3];
    for x in [-0.135, 0.135] {
        for y in [-0.08, 0.08] {
            tube(
                &mut out,
                Vec3::new(x, y, -0.20),
                Vec3::new(x + 0.025, y, -0.43),
                0.019,
                coat,
            );
            shell(
                &mut out,
                Vec3::new(x + 0.03, y, 0.0),
                &[(0.035, 0.028, -0.46), (0.035, 0.028, -0.42)],
                0.01,
                dark,
            );
        }
    }
    for y in [-0.06, 0.06] {
        shell(
            &mut out,
            Vec3::new(0.22, y, 0.0),
            &[(0.022, 0.012, 0.15), (0.022, 0.012, 0.19)],
            0.006,
            dark,
        );
        tube(
            &mut out,
            Vec3::new(0.17, y, 0.19),
            Vec3::new(0.145, y, 0.30),
            0.012,
            coat,
        );
    }
    tube(
        &mut out,
        Vec3::new(-0.18, 0.0, -0.13),
        Vec3::new(-0.29, 0.0, -0.31),
        0.024,
        brass,
    );
    tube(
        &mut out,
        Vec3::new(0.12, 0.0, -0.02),
        Vec3::new(0.13, 0.0, 0.21),
        0.024,
        brass,
    );
    tube(
        &mut out,
        Vec3::new(0.0, 0.0, -0.42),
        Vec3::new(0.0, 0.0, 1.18),
        0.012,
        brass,
    );
    let side = Vec3::new(-forward.y, forward.x, 0.0);
    for f in &mut out {
        for p in &mut f.surface.vertices {
            *p = point + forward * p.x + side * p.y + Vec3::Z * p.z;
        }
    }
    out
}

pub(super) fn legacy_trim(
    out: &mut Vec<Face>,
    center: Vec3,
    kind: park_sim::RideKind,
    size: u32,
    rotation: u8,
) {
    use park_sim::RideKind;
    let gold = [rgb(246, 194, 83), rgb(193, 132, 48), rgb(143, 89, 35)];
    let teal = [rgb(77, 159, 150), rgb(43, 113, 112), rgb(32, 84, 91)];
    let red = [rgb(235, 76, 105), rgb(177, 44, 83), rgb(127, 30, 65)];
    let mut detail = Vec::new();
    match kind {
        RideKind::Carousel => {
            lathe(
                &mut detail,
                center,
                &[
                    (0.17, 0.37),
                    (0.23, 0.47),
                    (0.16, 0.64),
                    (0.16, 1.60),
                    (0.25, 1.71),
                ],
                teal,
            );
            lathe(
                &mut detail,
                center,
                &[(0.18, 2.77), (0.18, 2.89), (0.12, 2.98), (0.0, 2.98)],
                gold,
            );
            for i in 0..16 {
                let a = i as f32 * std::f32::consts::TAU / 16.0;
                let radial = Vec3::new(a.cos(), a.sin(), 0.0);
                tube(
                    &mut detail,
                    center + radial * 0.18 + Vec3::Z * 2.87,
                    center + radial * 1.44 + Vec3::Z * 2.11,
                    0.009,
                    gold,
                );
                let medallion = center + radial * 1.44 + Vec3::Z * 1.99;
                lathe(
                    &mut detail,
                    medallion,
                    &[(0.06, -0.045), (0.07, 0.025), (0.03, 0.075)],
                    gold,
                );
            }
        }
        RideKind::Swing => {
            for y in [-1.22, 1.22] {
                for x in [-1.15, 1.15] {
                    tube(
                        &mut detail,
                        center + Vec3::new(x, y, 0.30),
                        center + Vec3::new(x * 0.56, y, 2.35),
                        0.035,
                        gold,
                    );
                    lathe(
                        &mut detail,
                        center + Vec3::new(x, y, 2.60),
                        &[(0.13, 0.0), (0.11, 0.16), (0.0, 0.25)],
                        gold,
                    );
                }
                for x in [-0.85, -0.42, 0.0, 0.42, 0.85] {
                    tube(
                        &mut detail,
                        center + Vec3::new(x, y, 2.48),
                        center + Vec3::new(x + 0.17, y, 2.62),
                        0.009,
                        gold,
                    );
                }
            }
        }
        RideKind::DropTower => {
            for h in 0..8 {
                let z = 0.4 + h as f32 * 0.58;
                for sign in [-1.0, 1.0] {
                    tube(
                        &mut detail,
                        center + Vec3::new(-0.23, sign * 0.23, z),
                        center + Vec3::new(0.23, sign * 0.23, z + 0.5),
                        0.013,
                        gold,
                    );
                    tube(
                        &mut detail,
                        center + Vec3::new(sign * 0.23, -0.23, z),
                        center + Vec3::new(sign * 0.23, 0.23, z + 0.5),
                        0.013,
                        gold,
                    );
                }
            }
            for sign in [-1.0, 1.0] {
                tube(
                    &mut detail,
                    center + Vec3::new(sign * 0.30, 0.0, 0.2),
                    center + Vec3::new(sign * 0.30, 0.0, 5.18),
                    0.018,
                    red,
                );
            }
        }
        _ => {}
    }
    out.extend(project(detail, size, rotation));
}
