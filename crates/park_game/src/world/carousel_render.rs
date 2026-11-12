//! Original open carousel. Reserved cells are never drawn as solid roofs.
use super::*;
use crate::path_tools::CarouselPreview;
use park_sim::{ObjectId, PATH_UNITS, PathPoint, RideId, RideKind};
type Face = service_render::Face;

pub(super) fn at(point: Vec3, size: u32, rotation: u8) -> Vec3 {
    let offset = rotated_offset(point.truncate(), rotation);
    let shift = match rotation % 4 {
        0 => Vec2::ZERO,
        1 => Vec2::new(size as f32 - 1.0, 0.0),
        2 => Vec2::splat(size as f32 - 1.0),
        _ => Vec2::new(0.0, size as f32 - 1.0),
    };
    project(offset.x + shift.x, offset.y + shift.y, point.z, size)
}

pub(super) fn face(result: &mut Vec<Face>, points: [Vec3; 4], ink: Ink, size: u32, rotation: u8) {
    let surface = Surface {
        vertices: points.map(|p| at(p, size, rotation)),
        kind: SurfaceKind::Top,
    };
    if surface.has_area() {
        result.push(Face { surface, ink });
    }
}

pub(super) fn cuboid(
    result: &mut Vec<Face>,
    min: Vec3,
    max: Vec3,
    inks: [Ink; 3],
    size: u32,
    rotation: u8,
) {
    let upper = [
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(max.x, min.y, max.z),
    ];
    let lower = upper.map(|p| Vec3::new(p.x, p.y, min.z));
    face(result, upper, inks[0], size, rotation);
    for i in 0..4 {
        let n = (i + 1) % 4;
        face(
            result,
            [upper[i], upper[n], lower[n], lower[i]],
            inks[1 + i % 2],
            size,
            rotation,
        );
    }
}

pub(super) fn center(origin: TilePos, height: u8, kind: RideKind, size: u32, rotation: u8) -> Vec2 {
    let (offset, elevation) = match kind {
        RideKind::Swing => (1.5, 2.68),
        RideKind::DropTower => (1.5, 5.60),
        _ => (1.0, 2.72),
    };
    at(
        Vec3::new(
            origin.x as f32 + offset,
            origin.y as f32 + offset,
            f32::from(height) + elevation,
        ),
        size,
        rotation,
    )
    .truncate()
}

pub(super) fn owner(park: &Park, object: ObjectId) -> Option<&park_sim::CarouselRide> {
    park.object_ride(object)
        .and_then(|id| park.carousel(id))
        .filter(|c| c.objects.contains(&object))
}

pub(super) fn object_faces(park: &Park, object: ObjectId, rotation: u8) -> Vec<Face> {
    owner(park, object)
        .filter(|c| c.objects.first() == Some(&object))
        .map_or_else(Vec::new, |c| {
            let appearance = park.ride_appearance(c.ride.id).unwrap_or_default();
            let model = park.ride_model(c.ride.id).unwrap();
            if matches!(
                model,
                park_sim::RideModel::FerrisWheel
                    | park_sim::RideModel::TeaCups
                    | park_sim::RideModel::ObservationTower
                    | park_sim::RideModel::PirateShip
            ) {
                return super::catalogue_render::paint_static(
                    super::catalogue_render::static_faces(
                        model,
                        c.origin,
                        c.height,
                        c.rotation,
                        park.width(),
                        rotation,
                    ),
                    appearance,
                );
            }
            let art_rotation = if c.ride.model.is_some() { 0 } else { rotation };
            let mut result = match c.kind {
                RideKind::Swing => swing_faces(c.origin, c.height, park.width(), art_rotation),
                RideKind::DropTower => {
                    drop_tower_faces(c.origin, c.height, park.width(), art_rotation)
                }
                _ => faces(c.origin, c.height, park.width(), art_rotation),
            };
            if c.ride.model.is_some() {
                result = super::catalogue_render::orient_legacy(
                    result,
                    c.origin,
                    model,
                    c.rotation,
                    park.width(),
                    rotation,
                );
            }
            for f in &mut result {
                for (paint, colors) in [
                    (
                        appearance.structure,
                        [rgb(77, 159, 150), rgb(43, 113, 112), rgb(32, 84, 91)],
                    ),
                    (
                        appearance.supports,
                        [rgb(246, 194, 83), rgb(193, 132, 48), rgb(143, 89, 35)],
                    ),
                    (
                        appearance.structure,
                        [rgb(235, 76, 105), rgb(94, 89, 200), rgb(54, 186, 219)],
                    ),
                    (
                        appearance.structure,
                        [rgb(235, 76, 105), rgb(177, 44, 83), rgb(127, 30, 65)],
                    ),
                    (
                        appearance.structure,
                        [rgb(54, 186, 219), rgb(38, 130, 177), rgb(28, 89, 133)],
                    ),
                    (
                        appearance.supports,
                        [rgb(246, 194, 83), rgb(193, 132, 48), rgb(143, 89, 35)],
                    ),
                ] {
                    if let Some(i) = colors.iter().position(|c| *c == f.ink) {
                        f.ink = ride_colors::inks(paint, colors)[i];
                        break;
                    }
                }
            }
            result
        })
}

/// Open A-frame swing with a striped apron, tall columns and overhead beam.
/// The passenger gondola is dynamic and stays inside this three-step envelope.
fn swing_faces(origin: TilePos, height: u8, size: u32, rotation: u8) -> Vec<Face> {
    let mut result = Vec::with_capacity(120);
    let center = Vec3::new(
        origin.x as f32 + 1.5,
        origin.y as f32 + 1.5,
        f32::from(height),
    );
    let red = [rgb(235, 76, 105), rgb(177, 44, 83), rgb(127, 30, 65)];
    let gold = [rgb(246, 194, 83), rgb(193, 132, 48), rgb(143, 89, 35)];
    let blue = [rgb(54, 186, 219), rgb(38, 130, 177), rgb(28, 89, 133)];
    let cream = [rgb(247, 230, 186), rgb(184, 159, 125), rgb(140, 113, 82)];
    cuboid(
        &mut result,
        center + Vec3::new(-1.5, -1.5, 0.0),
        center + Vec3::new(1.5, 1.5, 0.12),
        cream,
        size,
        rotation,
    );
    for lane in 0..10 {
        let y = -1.4 + lane as f32 * 0.28;
        cuboid(
            &mut result,
            center + Vec3::new(-1.38, y, 0.12),
            center + Vec3::new(1.38, y + 0.08, 0.135),
            if lane % 2 == 0 { red } else { blue },
            size,
            rotation,
        );
    }
    for x in [-1.15, 1.15] {
        for y in [-1.22, 1.22] {
            cuboid(
                &mut result,
                center + Vec3::new(x - 0.105, y - 0.105, 0.12),
                center + Vec3::new(x + 0.105, y + 0.105, 2.53),
                red,
                size,
                rotation,
            );
            cuboid(
                &mut result,
                center + Vec3::new(x - 0.18, y - 0.18, 0.12),
                center + Vec3::new(x + 0.18, y + 0.18, 0.24),
                gold,
                size,
                rotation,
            );
            cuboid(
                &mut result,
                center + Vec3::new(x - 0.15, y - 0.15, 2.47),
                center + Vec3::new(x + 0.15, y + 0.15, 2.65),
                gold,
                size,
                rotation,
            );
        }
    }
    for y in [-1.22, 1.22] {
        cuboid(
            &mut result,
            center + Vec3::new(-1.28, y - 0.09, 2.48),
            center + Vec3::new(1.28, y + 0.09, 2.62),
            blue,
            size,
            rotation,
        );
    }
    cuboid(
        &mut result,
        center + Vec3::new(-0.12, -1.35, 2.54),
        center + Vec3::new(0.12, 1.35, 2.68),
        gold,
        size,
        rotation,
    );
    result
}

/// Open guide tower, striped boarding deck and lantern crown. The moving
/// carriage and its six passengers use the simulation's saved seat positions.
fn drop_tower_faces(origin: TilePos, height: u8, size: u32, rotation: u8) -> Vec<Face> {
    let mut result = Vec::with_capacity(150);
    let center = Vec3::new(
        origin.x as f32 + 1.5,
        origin.y as f32 + 1.5,
        f32::from(height),
    );
    let cyan = [rgb(54, 186, 219), rgb(38, 130, 177), rgb(28, 89, 133)];
    let gold = [rgb(246, 194, 83), rgb(193, 132, 48), rgb(143, 89, 35)];
    let red = [rgb(235, 76, 105), rgb(177, 44, 83), rgb(127, 30, 65)];
    let stone = [rgb(247, 230, 186), rgb(184, 159, 125), rgb(140, 113, 82)];
    cuboid(
        &mut result,
        center + Vec3::new(-1.5, -1.5, 0.0),
        center + Vec3::new(1.5, 1.5, 0.10),
        stone,
        size,
        rotation,
    );
    for stripe in 0..8 {
        let y = -1.35 + stripe as f32 * 0.36;
        cuboid(
            &mut result,
            center + Vec3::new(-1.36, y, 0.10),
            center + Vec3::new(1.36, y + 0.08, 0.13),
            if stripe % 2 == 0 { cyan } else { gold },
            size,
            rotation,
        );
    }
    for x in [-0.23, 0.23] {
        for y in [-0.23, 0.23] {
            cuboid(
                &mut result,
                center + Vec3::new(x - 0.06, y - 0.06, 0.12),
                center + Vec3::new(x + 0.06, y + 0.06, 5.22),
                cyan,
                size,
                rotation,
            );
        }
    }
    for level in [0.32, 2.30, 4.48] {
        cuboid(
            &mut result,
            center + Vec3::new(-0.35, -0.35, level),
            center + Vec3::new(0.35, 0.35, level + 0.10),
            gold,
            size,
            rotation,
        );
    }
    cuboid(
        &mut result,
        center + Vec3::new(-0.50, -0.50, 5.18),
        center + Vec3::new(0.50, 0.50, 5.43),
        red,
        size,
        rotation,
    );
    cuboid(
        &mut result,
        center + Vec3::new(-0.22, -0.22, 5.43),
        center + Vec3::new(0.22, 0.22, 5.60),
        gold,
        size,
        rotation,
    );
    for x in [-1.15, 1.15] {
        cuboid(
            &mut result,
            center + Vec3::new(x - 0.09, -1.20, 0.13),
            center + Vec3::new(x + 0.09, -1.02, 0.72),
            red,
            size,
            rotation,
        );
        cuboid(
            &mut result,
            center + Vec3::new(x - 0.09, 1.02, 0.13),
            center + Vec3::new(x + 0.09, 1.20, 0.72),
            red,
            size,
            rotation,
        );
    }
    result
}

/// Apron, raised circular deck, mast and small cap fit the full three-step
/// envelope. Open sides keep seats and passengers visible from every angle.
fn faces(origin: TilePos, height: u8, size: u32, rotation: u8) -> Vec<Face> {
    let mut result = Vec::with_capacity(440);
    let center = Vec3::new(
        origin.x as f32 + 1.0,
        origin.y as f32 + 1.0,
        f32::from(height),
    );
    let cream = [rgb(235, 221, 174), rgb(181, 161, 118), rgb(144, 124, 84)];
    let teal = [rgb(77, 159, 150), rgb(43, 113, 112), rgb(32, 84, 91)];
    let gold = [rgb(246, 194, 83), rgb(193, 132, 48), rgb(143, 89, 35)];
    let festival = [rgb(235, 76, 105), rgb(94, 89, 200), rgb(54, 186, 219)];
    cuboid(
        &mut result,
        center + Vec3::new(-1.5, -1.5, 0.0),
        center + Vec3::new(1.5, 1.5, 0.12),
        cream,
        size,
        rotation,
    );
    // Forty-eight radial pieces keep the circular deck smooth at 400% zoom.
    // Each brass rim segment belongs to the same physical deck surface.
    for i in 0..48 {
        let a = i as f32 * std::f32::consts::TAU / 48.0;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 48.0;
        let radial =
            |angle: f32, r: f32, z: f32| center + Vec3::new(angle.cos() * r, angle.sin() * r, z);
        face(
            &mut result,
            [
                radial(a, 0.08, 0.35),
                radial(a, 1.41, 0.35),
                radial(b, 1.41, 0.35),
                radial(b, 0.08, 0.35),
            ],
            if i % 2 == 0 { teal[0] } else { cream[0] },
            size,
            rotation,
        );
        face(
            &mut result,
            [
                radial(a, 1.32, 0.36),
                radial(a, 1.41, 0.36),
                radial(b, 1.41, 0.36),
                radial(b, 1.32, 0.36),
            ],
            if i % 4 == 0 { cream[0] } else { gold[0] },
            size,
            rotation,
        );
        face(
            &mut result,
            [
                radial(a, 1.41, 0.35),
                radial(b, 1.41, 0.35),
                radial(b, 1.41, 0.12),
                radial(a, 1.41, 0.12),
            ],
            teal[1 + i % 2],
            size,
            rotation,
        );
    }
    // A faceted festival canopy leaves every third bay open so the moving
    // mounts remain visible. The physical roof and valance stay below the
    // existing three-step clearance and inside the three-by-three footprint.
    let canopy = |angle: f32, radius: f32, z: f32| {
        center + Vec3::new(angle.cos() * radius, angle.sin() * radius, z)
    };
    for i in 0..24 {
        let a = i as f32 * std::f32::consts::TAU / 24.0;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 24.0;
        if i % 3 != 2 {
            face(
                &mut result,
                [
                    canopy(a, 0.37, 2.87),
                    canopy(a, 1.45, 2.10),
                    canopy(b, 1.45, 2.10),
                    canopy(b, 0.37, 2.87),
                ],
                festival[i % 3],
                size,
                rotation,
            );
        }
        face(
            &mut result,
            [
                canopy(a, 1.43, 2.10),
                canopy(b, 1.43, 2.10),
                canopy(b, 1.43, 1.94),
                canopy(a, 1.43, 1.94),
            ],
            if i % 2 == 0 { gold[0] } else { teal[0] },
            size,
            rotation,
        );
    }
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let point = canopy(angle, 1.42, 2.12);
        cuboid(
            &mut result,
            point + Vec3::new(-0.045, -0.045, 0.0),
            point + Vec3::new(0.045, 0.045, 0.14),
            gold,
            size,
            rotation,
        );
    }
    cuboid(
        &mut result,
        center + Vec3::new(-0.13, -0.13, 0.12),
        center + Vec3::new(0.13, 0.13, 2.52),
        gold,
        size,
        rotation,
    );
    cuboid(
        &mut result,
        center + Vec3::new(-0.43, -0.43, 2.50),
        center + Vec3::new(0.43, 0.43, 2.63),
        teal,
        size,
        rotation,
    );
    cuboid(
        &mut result,
        center + Vec3::new(-0.16, -0.16, 2.63),
        center + Vec3::new(0.16, 0.16, 2.79),
        gold,
        size,
        rotation,
    );
    result
}

pub(super) fn nominal(point: PathPoint) -> Vec3 {
    Vec3::new(point.x as f32, point.y as f32, point.height as f32) / PATH_UNITS as f32
        + Vec3::Z * PATH_TOP_OFFSET
}

pub(super) fn paint_dynamic(painter: &mut Painter, park: &Park, rotation: u8, mode: ViewMode) {
    if park.carousels().iter().all(|c| c.objects.is_empty()) {
        return;
    }
    let passengers = park.ride_guest_positions();
    for carousel in park.carousels().iter().filter(|c| !c.objects.is_empty()) {
        if carousel.objects.iter().any(|id| park.object(*id).is_none()) {
            continue;
        }
        if mode == ViewMode::Surface && carousel.objects.iter().any(|id| park.is_underground(*id)) {
            continue;
        }
        if matches!(
            park.ride_model(carousel.ride.id),
            Some(
                park_sim::RideModel::FerrisWheel
                    | park_sim::RideModel::TeaCups
                    | park_sim::RideModel::ObservationTower
                    | park_sim::RideModel::PirateShip
            )
        ) {
            for face in super::catalogue_render::dynamic_faces(park, carousel, rotation) {
                painter.surface(face.surface, face.ink);
            }
            continue;
        }
        for (seat, point) in park
            .carousel_seat_positions(carousel.ride.id)
            .into_iter()
            .enumerate()
        {
            let p = nominal(point);
            let mut surfaces = Vec::new();
            let appearance = park.ride_appearance(carousel.ride.id).unwrap_or_default();
            let gold = ride_colors::inks(
                appearance.supports,
                [rgb(240, 187, 72), rgb(179, 121, 44), rgb(127, 82, 33)],
            );
            let cream = ride_colors::inks(
                appearance.vehicles,
                if carousel.kind == RideKind::DropTower {
                    [rgb(235, 76, 105), rgb(177, 44, 83), rgb(127, 30, 65)]
                } else if carousel.kind == RideKind::Swing {
                    [rgb(54, 186, 219), rgb(38, 130, 177), rgb(28, 89, 133)]
                } else {
                    [rgb(246, 231, 185), rgb(199, 173, 128), rgb(157, 129, 91)]
                },
            );
            let teal = ride_colors::inks(
                appearance.structure,
                if carousel.kind == RideKind::DropTower {
                    [rgb(54, 186, 219), rgb(38, 130, 177), rgb(28, 89, 133)]
                } else if carousel.kind == RideKind::Swing {
                    [rgb(235, 76, 105), rgb(177, 44, 83), rgb(127, 30, 65)]
                } else {
                    [rgb(63, 142, 143), rgb(40, 101, 113), rgb(27, 73, 86)]
                },
            );
            if carousel.kind == RideKind::DropTower {
                // A narrow hanging chair and colored restraint move with the
                // authoritative guest seat through the lift and drop.
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.17, -0.16, -0.20),
                    p + Vec3::new(0.17, 0.16, -0.03),
                    cream,
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.17, 0.11, -0.03),
                    p + Vec3::new(0.17, 0.16, 0.30),
                    teal,
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.16, -0.15, 0.28),
                    p + Vec3::new(0.16, 0.14, 0.34),
                    gold,
                    park.width(),
                    rotation,
                );
            } else if carousel.kind == RideKind::Swing {
                let pivot = Vec3::new(
                    carousel.origin.x as f32
                        + if carousel.ride.model.is_some() {
                            1.0
                        } else {
                            1.5
                        },
                    p.y,
                    f32::from(carousel.height) + 2.50,
                );
                for y in [-0.09, 0.09] {
                    face(
                        &mut surfaces,
                        [
                            pivot + Vec3::new(-0.025, y, 0.0),
                            pivot + Vec3::new(0.025, y, 0.0),
                            p + Vec3::new(0.025, y, 0.04),
                            p + Vec3::new(-0.025, y, 0.04),
                        ],
                        gold[0],
                        park.width(),
                        rotation,
                    );
                }
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.18, -0.13, -0.19),
                    p + Vec3::new(0.18, 0.13, -0.02),
                    cream,
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.17, 0.10, -0.03),
                    p + Vec3::new(0.17, 0.14, 0.28),
                    teal,
                    park.width(),
                    rotation,
                );
            } else {
                // Compact horse-like bench: legs, body, saddle and raised head.
                for x in [-0.13, 0.10] {
                    cuboid(
                        &mut surfaces,
                        p + Vec3::new(x, -0.065, -0.45),
                        p + Vec3::new(x + 0.04, 0.065, -0.15),
                        gold,
                        park.width(),
                        rotation,
                    );
                }
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.20, -0.12, -0.24),
                    p + Vec3::new(0.20, 0.12, -0.04),
                    cream,
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.07, -0.13, -0.04),
                    p + Vec3::new(0.08, 0.13, 0.0),
                    teal,
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(0.13, -0.075, -0.12),
                    p + Vec3::new(0.24, 0.075, 0.20),
                    cream,
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.28, -0.035, -0.12),
                    p + Vec3::new(-0.19, 0.035, 0.02),
                    gold,
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(0.17, -0.055, 0.10),
                    p + Vec3::new(0.27, 0.055, 0.24),
                    gold,
                    park.width(),
                    rotation,
                );
                for y in [-0.077, 0.068] {
                    cuboid(
                        &mut surfaces,
                        p + Vec3::new(0.235, y, 0.11),
                        p + Vec3::new(0.257, y + 0.012, 0.14),
                        teal,
                        park.width(),
                        rotation,
                    );
                }
            }
            if let Some(passenger) = passengers.iter().find(|p| {
                p.ride == carousel.ride.id
                    && usize::from(p.seat) == seat
                    && p.phase == RideVisitPhase::Riding
            }) {
                // Use the same authoritative point as transfers; no rank or
                // independently calculated animation can move a passenger.
                let p = nominal(passenger.point);
                let shirt = match passenger.guest.0 % 3 {
                    0 => rgb(213, 77, 82),
                    1 => rgb(66, 126, 188),
                    _ => rgb(150, 88, 174),
                };
                let skin = if passenger.guest.0.is_multiple_of(3) {
                    rgb(174, 119, 84)
                } else {
                    rgb(239, 192, 143)
                };
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.05, -0.045, 0.02),
                    p + Vec3::new(0.05, 0.045, 0.36),
                    [shirt; 3],
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.043, -0.038, 0.36),
                    p + Vec3::new(0.043, 0.038, 0.65),
                    [skin; 3],
                    park.width(),
                    rotation,
                );
                cuboid(
                    &mut surfaces,
                    p + Vec3::new(-0.045, -0.04, 0.63),
                    p + Vec3::new(0.045, 0.04, 0.70),
                    [rgb(67, 53, 42); 3],
                    park.width(),
                    rotation,
                );
            }
            for face in surfaces {
                painter.surface(face.surface, face.ink);
            }
        }
    }
}

pub fn carousel_preview_mesh(preview: Option<CarouselPreview>, park: &Park, rotation: u8) -> Mesh {
    let mut painter = Painter::default();
    let Some(preview) = preview else {
        return painter.finish();
    };
    if park.width() != park.height() || park.tile(preview.origin).is_none() {
        return painter.finish();
    }
    let surfaces = if preview.model.tracked() {
        super::catalogue_render::prefab_faces(
            preview.model,
            preview.origin,
            preview.height,
            preview.rotation,
            park.width(),
            rotation,
        )
    } else if matches!(
        preview.model,
        park_sim::RideModel::FerrisWheel
            | park_sim::RideModel::TeaCups
            | park_sim::RideModel::ObservationTower
            | park_sim::RideModel::PirateShip
    ) {
        super::catalogue_render::static_faces(
            preview.model,
            preview.origin,
            preview.height,
            preview.rotation,
            park.width(),
            rotation,
        )
    } else {
        super::catalogue_render::orient_legacy(
            match preview.kind {
                RideKind::Swing => swing_faces(preview.origin, preview.height, park.width(), 0),
                RideKind::DropTower => {
                    drop_tower_faces(preview.origin, preview.height, park.width(), 0)
                }
                _ => faces(preview.origin, preview.height, park.width(), 0),
            },
            preview.origin,
            preview.model,
            preview.rotation,
            park.width(),
            rotation,
        )
    };
    let mut ink = if !preview.valid {
        rgb(225, 68, 63)
    } else if preview.demolish {
        rgb(235, 120, 51)
    } else {
        rgb(47, 183, 114)
    };
    for face in &surfaces {
        let mut surface = face.surface;
        for p in &mut surface.vertices {
            p.z += SELECTION_BIAS;
        }
        ink[3] = 0.20;
        painter.surface(surface, ink);
        ink[3] = 0.9;
        painter.surface_outline(face.surface, 1.1, SELECTION_BIAS * 2.0, ink);
    }
    if (!preview.valid || preview.demolish)
        && let Some(apron) = surfaces.first()
    {
        let v = apron.surface.vertices.map(Vec3::truncate);
        for (a, b) in [(v[0], v[2]), (v[1], v[3])] {
            painter.surface_line(apron.surface, a, b, 1.5, SELECTION_BIAS * 3.0, ink);
        }
    }
    painter.finish()
}

pub fn ride_focus(park: &Park, id: RideId, rotation: u8) -> Option<Vec2> {
    if park.width() != park.height() {
        return None;
    }
    if let Some(c) = park.carousel(id).filter(|c| !c.objects.is_empty()) {
        let (mut w, mut d) = park.ride_model(id)?.dimensions();
        if c.rotation % 2 == 1 {
            std::mem::swap(&mut w, &mut d);
        }
        Some(
            at(
                Vec3::new(
                    c.origin.x as f32 + (w - 1) as f32 / 2.0,
                    c.origin.y as f32 + (d - 1) as f32 / 2.0,
                    c.height as f32 + 2.0,
                ),
                park.width(),
                rotation,
            )
            .truncate(),
        )
    } else if park.ride_kind(id) == Some(park_sim::RideKind::Coaster) {
        super::train_render::coaster_focus(park, id, rotation)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use park_sim::{PathDirection, Speed};

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
    }

    #[test]
    fn swing_art_preview_and_selection_fit_reserved_three_by_three_tiles() {
        let mut park = Park::new(32, 32);
        let origin = TilePos { x: 15, y: 17 };
        let id = park.place_swing(origin, 8).unwrap();
        let objects = park.carousel(id).unwrap().objects.clone();
        let target = SurfaceId::Object(objects[0]);
        for rotation in 0..4 {
            let geometry = object_faces(&park, objects[0], rotation);
            assert!(!geometry.is_empty());
            let floor = &geometry[0].surface;
            assert_eq!(
                floor.vertices[0],
                at(Vec3::new(15.0, 17.0, 8.12), 32, rotation)
            );
            assert_eq!(
                floor.vertices[2],
                at(Vec3::new(18.0, 20.0, 8.12), 32, rotation)
            );
            for cell in &objects[1..] {
                assert!(object_faces(&park, *cell, rotation).is_empty());
            }
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                let center = surface_center(&park, target, rotation).unwrap();
                assert_eq!(pick_surface(center, &park, rotation, mode), Some(target));
                assert_eq!(
                    selectable_surfaces(&park, target, rotation, mode).len(),
                    geometry.len()
                );
            }
            let ghost = carousel_preview_mesh(
                Some(CarouselPreview {
                    model: park_sim::RideModel::legacy(RideKind::Swing),
                    rotation: 0,
                    kind: RideKind::Swing,
                    origin,
                    height: 8,
                    valid: true,
                    demolish: false,
                }),
                &park,
                rotation,
            );
            assert!(!positions(&ghost).is_empty());
            assert!(positions(&ghost).iter().flatten().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn drop_tower_art_preview_and_selection_share_footprint() {
        let mut park = Park::new(32, 32);
        let origin = TilePos { x: 15, y: 17 };
        let id = park.place_drop_tower(origin, 8).unwrap();
        let objects = park.carousel(id).unwrap().objects.clone();
        let target = SurfaceId::Object(objects[0]);
        for rotation in 0..4 {
            let geometry = object_faces(&park, objects[0], rotation);
            assert!(!geometry.is_empty());
            for cell in &objects[1..] {
                assert!(object_faces(&park, *cell, rotation).is_empty());
            }
            let center = surface_center(&park, target, rotation).unwrap();
            assert_eq!(
                pick_surface(center, &park, rotation, ViewMode::Surface),
                Some(target)
            );
            let ghost = carousel_preview_mesh(
                Some(CarouselPreview {
                    model: park_sim::RideModel::legacy(RideKind::DropTower),
                    rotation: 0,
                    kind: RideKind::DropTower,
                    origin,
                    height: 8,
                    valid: true,
                    demolish: false,
                }),
                &park,
                rotation,
            );
            assert!(!positions(&ghost).is_empty());
            assert!(positions(&ghost).iter().flatten().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn carousel_faces_pick_preview_and_chunk_bounds_share_full_footprint() {
        let mut park = Park::new(32, 32);
        let origin = TilePos { x: 15, y: 17 };
        let id = park.place_carousel(origin, 8).unwrap();
        let objects = park.carousel(id).unwrap().objects.clone();
        let original = park.clone();
        for rotation in 0..4 {
            let geometry = object_faces(&park, objects[0], rotation);
            assert!(!geometry.is_empty());
            for cell in &objects[1..] {
                assert!(object_faces(&park, *cell, rotation).is_empty());
            }
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                let target = SurfaceId::Object(objects[0]);
                let center = surface_center(&park, target, rotation).unwrap();
                assert_eq!(pick_surface(center, &park, rotation, mode), Some(target));
                let selected = selectable_surfaces(&park, target, rotation, mode);
                assert_eq!(selected.len(), geometry.len());
                for (a, b) in selected.iter().zip(&geometry) {
                    assert_eq!(a.vertices, b.surface.vertices);
                }
                let selection = selection_mesh(None, Some(target), &park, rotation, mode);
                assert!(!positions(&selection).is_empty());
                for chunk in build_static_chunks(&park, rotation, true, mode) {
                    assert!(
                        positions(&chunk.mesh)
                            .iter()
                            .all(|p| chunk.bounds.contains(Vec2::new(p[0], p[1])))
                    );
                }
            }
            for valid in [true, false] {
                let ghost = carousel_preview_mesh(
                    Some(CarouselPreview {
                        model: park_sim::RideModel::legacy(park_sim::RideKind::Carousel),
                        rotation: 0,
                        kind: park_sim::RideKind::Carousel,
                        origin,
                        height: 8,
                        valid,
                        demolish: false,
                    }),
                    &park,
                    rotation,
                );
                assert!(positions(&ghost).iter().flatten().all(|v| v.is_finite()));
                let Some(VertexAttributeValues::Float32x4(colors)) =
                    ghost.attribute(Mesh::ATTRIBUTE_COLOR)
                else {
                    panic!("colors")
                };
                assert!(colors.iter().any(|c| c[3] < 0.3));
                assert!(colors.iter().any(|c| c[3] > 0.8));
                assert!(
                    colors
                        .iter()
                        .all(|c| if valid { c[1] > c[0] } else { c[0] > c[1] })
                );
            }
            // The platform reaches every reserved edge, including its corners.
            let face = &geometry[0].surface;
            assert_eq!(
                face.vertices[0],
                at(
                    Vec3::new(origin.x as f32 - 0.5, origin.y as f32 - 0.5, 8.12),
                    32,
                    rotation
                )
            );
        }
        assert_eq!(
            park, original,
            "previews never allocate identities or mutate state"
        );
    }

    #[test]
    fn authoritative_seats_animate_without_a_coaster_and_disappear_after_atomic_removal() {
        let mut park = Park::new(32, 32);
        let id = park.place_carousel(TilePos { x: 10, y: 10 }, 8).unwrap();
        let initial = park.carousel_seat_positions(id);
        park.set_ride_open(id, true).unwrap();
        for _ in 0..110 {
            park.advance(Speed::Normal);
        }
        assert_ne!(park.carousel_seat_positions(id), initial);
        for rotation in 0..4 {
            let dynamic = train_mesh(&park, rotation, ViewMode::Surface);
            assert!(
                !positions(&dynamic).is_empty(),
                "carousel renders with no coaster circuit"
            );
            assert!(positions(&dynamic).iter().flatten().all(|v| v.is_finite()));
            for point in park.carousel_seat_positions(id) {
                let p = at(nominal(point), 32, rotation);
                assert!(
                    positions(&dynamic)
                        .iter()
                        .any(|v| (Vec3::from(*v) - p).length() < 8.0)
                );
            }
        }
        let cell = park.carousel(id).unwrap().objects[8];
        assert!(park.remove_object(cell).is_some());
        assert!(park.carousel(id).unwrap().objects.is_empty());
        assert!(positions(&train_mesh(&park, 0, ViewMode::Surface)).is_empty());
        assert!(ride_focus(&park, id, 0).is_none());
    }

    #[test]
    fn carousel_passengers_use_their_own_seat_and_guest_mesh_omits_only_riding_bodies() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(20261020).unwrap();
        park.install_train_fixture().unwrap();
        let id = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        park.place_ride_entrance(TilePos { x: 18, y: 23 }, 8, PathDirection::East, id)
            .unwrap();
        for y in 21..=23 {
            park.place_queue_path(TilePos { x: 17, y }, 8, PathSlope::Flat, id)
                .unwrap();
        }
        park.place_ride_exit(TilePos { x: 20, y: 21 }, 8, PathDirection::South, id)
            .unwrap();
        park.place_path(TilePos { x: 20, y: 20 }, 8).unwrap();
        park.set_ride_open(id, true).unwrap();
        for _ in 0..15_000 {
            park.advance(Speed::Normal);
            if park
                .ride_guest_positions()
                .iter()
                .any(|p| p.ride == id && p.phase == RideVisitPhase::Riding)
            {
                break;
            }
        }
        let passengers: Vec<_> = park
            .ride_guest_positions()
            .into_iter()
            .filter(|p| p.ride == id && p.phase == RideVisitPhase::Riding)
            .collect();
        assert!(
            !passengers.is_empty(),
            "production boarding should reach the carousel"
        );
        for rotation in 0..4 {
            let mut painter = Painter::default();
            paint_dynamic(&mut painter, &park, rotation, ViewMode::Surface);
            let mesh = painter.finish();
            for p in &passengers {
                assert_eq!(
                    p.point,
                    park.carousel_seat_positions(id)[usize::from(p.seat)]
                );
                let head = at(nominal(p.point) + Vec3::Z * 0.65, 32, rotation);
                assert!(
                    positions(&mesh)
                        .iter()
                        .any(|v| (Vec3::from(*v) - head).length() < 4.0)
                );
            }
            let mut renderer = GuestRenderer::default();
            let mut optimized = GuestRenderer::empty_mesh();
            renderer.update(&mut optimized, &park, rotation, ViewMode::Surface);
            let oracle = guest_mesh(&park, rotation, ViewMode::Surface);
            assert_eq!(optimized.indices(), oracle.indices());
            assert_eq!(
                optimized.attribute(Mesh::ATTRIBUTE_COLOR),
                oracle.attribute(Mesh::ATTRIBUTE_COLOR)
            );
            assert_eq!(positions(&optimized).len(), positions(&oracle).len());
            for (a, b) in positions(&optimized).iter().zip(positions(&oracle)) {
                assert!((Vec3::from(*a) - Vec3::from(*b)).abs().max_element() < 0.001);
            }
        }
    }
}
