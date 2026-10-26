//! Original open carousel. Reserved cells are never drawn as solid roofs.
use super::*;
use crate::path_tools::CarouselPreview;
use park_sim::{ObjectId, PATH_UNITS, PathPoint, RideId};
type Face = service_render::Face;

fn at(point: Vec3, size: u32, rotation: u8) -> Vec3 {
    let offset = rotated_offset(point.truncate(), rotation);
    let shift = match rotation % 4 {
        0 => Vec2::ZERO,
        1 => Vec2::new(size as f32 - 1.0, 0.0),
        2 => Vec2::splat(size as f32 - 1.0),
        _ => Vec2::new(0.0, size as f32 - 1.0),
    };
    project(offset.x + shift.x, offset.y + shift.y, point.z, size)
}

fn face(result: &mut Vec<Face>, points: [Vec3; 4], ink: Ink, size: u32, rotation: u8) {
    let surface = Surface {
        vertices: points.map(|p| at(p, size, rotation)),
        kind: SurfaceKind::Top,
    };
    if surface.has_area() {
        result.push(Face { surface, ink });
    }
}

fn cuboid(result: &mut Vec<Face>, min: Vec3, max: Vec3, inks: [Ink; 3], size: u32, rotation: u8) {
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

pub(super) fn center(origin: TilePos, height: u8, size: u32, rotation: u8) -> Vec2 {
    at(
        Vec3::new(
            origin.x as f32 + 1.0,
            origin.y as f32 + 1.0,
            f32::from(height) + 2.72,
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
            let mut result = faces(c.origin, c.height, park.width(), rotation);
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

/// Apron, raised circular deck, mast and small cap fit the full three-step
/// envelope. Open sides keep seats and passengers visible from every angle.
fn faces(origin: TilePos, height: u8, size: u32, rotation: u8) -> Vec<Face> {
    let mut result = Vec::with_capacity(110);
    let center = Vec3::new(
        origin.x as f32 + 1.0,
        origin.y as f32 + 1.0,
        f32::from(height),
    );
    let cream = [rgb(235, 221, 174), rgb(181, 161, 118), rgb(144, 124, 84)];
    let teal = [rgb(77, 159, 150), rgb(43, 113, 112), rgb(32, 84, 91)];
    let gold = [rgb(246, 194, 83), rgb(193, 132, 48), rgb(143, 89, 35)];
    cuboid(
        &mut result,
        center + Vec3::new(-1.5, -1.5, 0.0),
        center + Vec3::new(1.5, 1.5, 0.12),
        cream,
        size,
        rotation,
    );
    for i in 0..24 {
        let a = i as f32 * std::f32::consts::TAU / 24.0;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 24.0;
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

fn nominal(point: PathPoint) -> Vec3 {
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
                [rgb(246, 231, 185), rgb(199, 173, 128), rgb(157, 129, 91)],
            );
            let teal = ride_colors::inks(
                appearance.structure,
                [rgb(63, 142, 143), rgb(40, 101, 113), rgb(27, 73, 86)],
            );
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
    let surfaces = faces(preview.origin, preview.height, park.width(), rotation);
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
        Some(center(c.origin, c.height, park.width(), rotation) - Vec2::Y * HEIGHT_STEP)
    } else if park.ride().is_some_and(|ride| ride.id == id) {
        train_focus(park, rotation)
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
