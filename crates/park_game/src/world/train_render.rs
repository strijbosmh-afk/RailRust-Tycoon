//! Original sampled-track and car geometry. Clearance boxes never become art.

use super::*;
use park_sim::{ObjectId, TRACK_UNITS, TrackKind, TrackPiece, TrackPoint, TrainPose};

pub(super) struct Face {
    pub(super) surface: Surface,
    pub(super) ink: Ink,
}

pub(super) fn piece_for(park: &Park, id: ObjectId) -> Option<&TrackPiece> {
    park.circuit()?
        .pieces
        .iter()
        .find(|piece| piece.object == id)
}

fn ground(point: TrackPoint) -> Vec3 {
    Vec3::new(point.x as f32, point.y as f32, point.height as f32) / TRACK_UNITS as f32
}

fn projected(point: Vec3, size: u32, rotation: u8) -> Vec3 {
    let last = size.saturating_sub(1) as f32;
    let (u, v) = match rotation % 4 {
        0 => (point.x, point.y),
        1 => (last - point.y, point.x),
        2 => (last - point.x, last - point.y),
        _ => (point.y, last - point.x),
    };
    project(u, v, point.z, size)
}

fn face(faces: &mut Vec<Face>, vertices: [Vec3; 4], ink: Ink, size: u32, rotation: u8) {
    let vertices = vertices.map(|point| projected(point, size, rotation));
    let surface = Surface {
        vertices,
        kind: SurfaceKind::Top,
    };
    // Edge-on rails and posts have no area or hit target. Dropping these also
    // keeps selection's inward outline well-defined at all four rotations.
    if surface.has_area() {
        faces.push(Face { surface, ink });
    }
}

/// A slab follows both endpoint heights; no flat max-height roof is emitted.
fn slab(
    faces: &mut Vec<Face>,
    footprint: [Vec3; 4],
    bottom: f32,
    top: f32,
    colors: [Ink; 3],
    size: u32,
    rotation: u8,
) {
    let upper = footprint.map(|point| point + Vec3::Z * top);
    let lower = footprint.map(|point| point + Vec3::Z * bottom);
    face(faces, upper, colors[0], size, rotation);
    for index in 0..4 {
        let next = (index + 1) % 4;
        face(
            faces,
            [upper[index], upper[next], lower[next], lower[index]],
            colors[1 + index % 2],
            size,
            rotation,
        );
    }
}

/// Shared boundary tangents use samples on both sides of a piece join.
/// The final-to-first join uses the same rule, keeping offset rails watertight.
fn normal(park: &Park, piece: &TrackPiece, index: usize) -> Vec3 {
    let circuit = park
        .circuit()
        .expect("a rendered piece belongs to a circuit");
    let piece_index = circuit
        .pieces
        .iter()
        .position(|candidate| candidate.object == piece.object)
        .expect("a rendered piece belongs to a circuit");
    let previous = if index > 0 {
        piece.samples[index - 1]
    } else {
        let previous =
            &circuit.pieces[(piece_index + circuit.pieces.len() - 1) % circuit.pieces.len()];
        previous
            .samples
            .get(previous.samples.len().saturating_sub(2))
            .copied()
            .unwrap_or(piece.samples[index])
    };
    let next = if index + 1 < piece.samples.len() {
        piece.samples[index + 1]
    } else {
        circuit.pieces[(piece_index + 1) % circuit.pieces.len()]
            .samples
            .get(1)
            .copied()
            .unwrap_or(piece.samples[index])
    };
    let tangent = if index == 0 {
        let prior =
            &circuit.pieces[(piece_index + circuit.pieces.len() - 1) % circuit.pieces.len()];
        if piece.kind != TrackKind::Curve {
            ground(piece.samples[1]) - ground(piece.samples[0])
        } else if prior.kind != TrackKind::Curve {
            ground(piece.samples[0]) - ground(previous)
        } else {
            ground(next) - ground(previous)
        }
    } else if index + 1 == piece.samples.len() {
        let following = &circuit.pieces[(piece_index + 1) % circuit.pieces.len()];
        if piece.kind != TrackKind::Curve {
            ground(piece.samples[index]) - ground(previous)
        } else if following.kind != TrackKind::Curve {
            ground(next) - ground(piece.samples[index])
        } else {
            ground(next) - ground(previous)
        }
    } else {
        ground(next) - ground(previous)
    };
    // At a straight/curve join use the exact straight tangent. Averaging a
    // finite curve chord would twist the adjacent graded quad out of plane.
    let delta = tangent.truncate();
    let side = Vec2::new(-delta.y, delta.x).normalize_or_zero();
    side.extend(0.0)
}

fn strip(a: Vec3, b: Vec3, na: Vec3, nb: Vec3, left: f32, right: f32) -> [Vec3; 4] {
    [a + na * left, b + nb * left, b + nb * right, a + na * right]
}

fn support_is_clear(park: &Park, id: ObjectId, point: Vec3) -> Option<f32> {
    if point.x < -0.5 || point.y < -0.5 {
        return None;
    }
    let pos = TilePos {
        x: (point.x + 0.5).floor() as u32,
        y: (point.y + 0.5).floor() as u32,
    };
    let bottom = f32::from(park.tile(pos)?.height);
    if bottom + 0.5 >= point.z {
        return None;
    }
    let occupied = park.objects().iter().any(|object| {
        object.id != id
            && object.pos == pos
            && f32::from(object.height) < point.z
            && f32::from(object.height) + f32::from(object.clearance) > bottom
    });
    (!occupied).then_some(bottom)
}

fn supports(faces: &mut Vec<Face>, park: &Park, piece: &TrackPiece, rotation: u8) {
    let middle = piece.samples.len() / 2;
    let a = ground(piece.samples[middle.saturating_sub(1)]);
    let b = ground(piece.samples[middle]);
    let center = if piece.samples.len() == 2 {
        a.lerp(b, 0.5)
    } else {
        b
    };
    let side = normal(park, piece, middle);
    let colors = [rgb(134, 164, 157), rgb(80, 113, 111), rgb(56, 85, 87)];
    for offset in [-0.23, 0.23] {
        let point = center + side * offset;
        let Some(bottom) = support_is_clear(park, piece.object, point) else {
            continue;
        };
        let ground_point = Vec3::new(point.x, point.y, bottom);
        let footprint = [
            ground_point + Vec3::new(-0.035, -0.035, 0.0),
            ground_point + Vec3::new(-0.035, 0.035, 0.0),
            ground_point + Vec3::new(0.035, 0.035, 0.0),
            ground_point + Vec3::new(0.035, -0.035, 0.0),
        ];
        slab(
            faces,
            footprint,
            0.0,
            point.z - bottom,
            colors,
            park.width(),
            rotation,
        );
        slab(
            faces,
            footprint.map(|corner| ground_point + (corner - ground_point) * 2.0),
            0.0,
            0.12,
            [rgb(162, 170, 155), rgb(105, 120, 110), rgb(86, 102, 95)],
            park.width(),
            rotation,
        );
    }
}

pub(super) fn piece_faces(park: &Park, piece: &TrackPiece, rotation: u8) -> Vec<Face> {
    let mut faces = Vec::new();
    if park.width() != park.height()
        || park.object(piece.object).is_none()
        || piece.samples.len() < 2
    {
        return faces;
    }
    let size = park.width();
    supports(&mut faces, park, piece, rotation);
    for index in 0..piece.samples.len() - 1 {
        let a = ground(piece.samples[index]);
        let b = ground(piece.samples[index + 1]);
        let na = normal(park, piece, index);
        let nb = normal(park, piece, index + 1);
        let bed = strip(a, b, na, nb, -0.27, 0.27);
        slab(
            &mut faces,
            bed,
            0.0,
            0.10,
            [rgb(48, 70, 77), rgb(35, 51, 59), rgb(29, 43, 50)],
            size,
            rotation,
        );
        // All details are physical shallow solids and inherit the slope,
        // including ties, chain, rail heads, brakes, and platform edging.
        let length = a.truncate().distance(b.truncate());
        let tie_count = (length / 0.22).ceil().max(1.0) as usize;
        for tie in 0..tie_count {
            let center = (tie as f32 + 0.5) / tie_count as f32;
            let half = (0.035 / length.max(0.001)).min(0.3 / tie_count as f32);
            let lo = center - half;
            let hi = center + half;
            slab(
                &mut faces,
                strip(
                    a.lerp(b, lo),
                    a.lerp(b, hi),
                    na.lerp(nb, lo),
                    na.lerp(nb, hi),
                    -0.255,
                    0.255,
                ),
                0.10,
                0.15,
                [rgb(174, 126, 75), rgb(120, 81, 53), rgb(90, 62, 47)],
                size,
                rotation,
            );
        }
        for offset in [-0.17, 0.17] {
            slab(
                &mut faces,
                strip(a, b, na, nb, offset - 0.025, offset + 0.025),
                0.15,
                0.25,
                [rgb(228, 235, 212), rgb(148, 174, 166), rgb(108, 140, 139)],
                size,
                rotation,
            );
        }
        match piece.kind {
            TrackKind::Station => {
                for (lo, hi) in [(-0.49, -0.29), (0.29, 0.49)] {
                    slab(
                        &mut faces,
                        strip(a, b, na, nb, lo, hi),
                        0.0,
                        0.24,
                        [rgb(222, 206, 161), rgb(152, 137, 104), rgb(121, 113, 92)],
                        size,
                        rotation,
                    );
                    let edge = if lo < 0.0 { hi - 0.02 } else { lo + 0.02 };
                    slab(
                        &mut faces,
                        strip(a, b, na, nb, edge - 0.014, edge + 0.014),
                        0.24,
                        0.27,
                        [rgb(240, 181, 64); 3],
                        size,
                        rotation,
                    );
                }
            }
            TrackKind::Lift => {
                slab(
                    &mut faces,
                    strip(a, b, na, nb, -0.035, 0.035),
                    0.13,
                    0.23,
                    [rgb(237, 182, 66), rgb(161, 115, 41), rgb(126, 92, 40)],
                    size,
                    rotation,
                );
                for link in 0..tie_count {
                    let at = (link as f32 + 0.5) / tie_count as f32;
                    let half = (0.018 / length.max(0.001)).min(0.2 / tie_count as f32);
                    slab(
                        &mut faces,
                        strip(
                            a.lerp(b, at - half),
                            a.lerp(b, at + half),
                            na.lerp(nb, at - half),
                            na.lerp(nb, at + half),
                            -0.055,
                            0.055,
                        ),
                        0.23,
                        0.27,
                        [rgb(106, 78, 42); 3],
                        size,
                        rotation,
                    );
                }
            }
            TrackKind::Brake => {
                for offset in [-0.085, 0.085] {
                    slab(
                        &mut faces,
                        strip(a, b, na, nb, offset - 0.025, offset + 0.025),
                        0.13,
                        0.26,
                        [rgb(216, 110, 69), rgb(138, 63, 45), rgb(103, 49, 40)],
                        size,
                        rotation,
                    );
                }
            }
            TrackKind::Drop | TrackKind::Curve | TrackKind::Straight => {}
        }
    }
    faces
}

pub(super) fn piece_center(park: &Park, piece: &TrackPiece, rotation: u8) -> Option<Vec2> {
    park.object(piece.object)?;
    let middle = piece.samples.len() / 2;
    let point = if piece.samples.len() == 2 {
        ground(piece.samples[0]).lerp(ground(piece.samples[1]), 0.5)
    } else {
        ground(*piece.samples.get(middle)?)
    };
    Some(projected(point + Vec3::Z * 0.25, park.width(), rotation).truncate())
}

pub(super) fn train_focus(park: &Park, rotation: u8) -> Option<Vec2> {
    if park.width() != park.height() {
        return None;
    }
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for piece in &park.circuit()?.pieces {
        if park.object(piece.object).is_none() {
            continue;
        }
        for point in &piece.samples {
            let point = projected(ground(*point), park.width(), rotation).truncate();
            min = min.min(point);
            max = max.max(point);
        }
    }
    (min.is_finite() && max.is_finite()).then_some((min + max) * 0.5)
}

fn car_faces(pose: TrainPose, size: u32, rotation: u8) -> Vec<Face> {
    let mut faces = Vec::new();
    let rear = ground(pose.rear);
    let front = ground(pose.front);
    let direction = (front - rear).truncate().normalize_or_zero();
    let normal = Vec2::new(-direction.y, direction.x).extend(0.0);
    let footprint = |start: f32, end: f32, left: f32, right: f32| {
        strip(
            rear.lerp(front, start),
            rear.lerp(front, end),
            normal,
            normal,
            left,
            right,
        )
    };
    // Four solid black wheel blocks sit exactly on the rail heads.
    for (start, end) in [(0.05, 0.25), (0.75, 0.95)] {
        for (lo, hi) in [(-0.205, -0.135), (0.135, 0.205)] {
            slab(
                &mut faces,
                footprint(start, end, lo, hi),
                0.25,
                0.39,
                [rgb(46, 53, 54), rgb(26, 34, 39), rgb(20, 28, 32)],
                size,
                rotation,
            );
        }
    }
    slab(
        &mut faces,
        footprint(0.0, 1.0, -0.215, 0.215),
        0.36,
        0.77,
        [rgb(218, 67, 57), rgb(160, 40, 45), rgb(114, 32, 40)],
        size,
        rotation,
    );
    // Gold nose, dark seat cushion, and gold side rims form the original livery.
    slab(
        &mut faces,
        footprint(0.83, 1.0, -0.215, 0.215),
        0.77,
        0.84,
        [rgb(248, 190, 70), rgb(184, 116, 43), rgb(145, 86, 36)],
        size,
        rotation,
    );
    slab(
        &mut faces,
        footprint(0.16, 0.72, -0.15, 0.15),
        0.77,
        0.80,
        [rgb(40, 57, 66), rgb(28, 42, 53), rgb(22, 32, 46)],
        size,
        rotation,
    );
    for (lo, hi) in [(-0.215, -0.18), (0.18, 0.215)] {
        slab(
            &mut faces,
            footprint(0.0, 0.83, lo, hi),
            0.77,
            0.83,
            [rgb(236, 173, 64), rgb(168, 104, 39), rgb(134, 78, 33)],
            size,
            rotation,
        );
    }
    slab(
        &mut faces,
        footprint(0.06, 0.19, -0.15, 0.15),
        0.80,
        0.99,
        [rgb(196, 66, 54), rgb(142, 43, 44), rgb(101, 35, 42)],
        size,
        rotation,
    );
    faces
}

/// Seated figures use the authoritative seat point and the physical car's
/// tangent. Their depth belongs to each small face, so rims, other cars and
/// foreground terrain occlude them without an always-on-top passenger layer.
fn passenger_faces(
    passenger: park_sim::RideGuestPosition,
    pose: TrainPose,
    size: u32,
    rotation: u8,
) -> Vec<Face> {
    let mut faces = Vec::new();
    let point = passenger.point;
    let center = Vec3::new(point.x as f32, point.y as f32, point.height as f32)
        / park_sim::PATH_UNITS as f32
        + Vec3::Z * PATH_TOP_OFFSET;
    let tangent = (ground(pose.front) - ground(pose.rear))
        .truncate()
        .normalize_or_zero();
    let along = tangent.extend(0.0);
    let side = Vec3::new(-tangent.y, tangent.x, 0.0);
    let footprint = |length: f32, width: f32| {
        [
            center - along * length - side * width,
            center - along * length + side * width,
            center + along * length + side * width,
            center + along * length - side * width,
        ]
    };
    let shirt = match passenger.guest.0 % 6 {
        0 => rgb(213, 77, 82),
        1 => rgb(66, 126, 188),
        2 => rgb(150, 88, 174),
        3 => rgb(242, 184, 65),
        4 => rgb(53, 154, 140),
        _ => rgb(222, 119, 161),
    };
    let skin = if passenger.guest.0.is_multiple_of(3) {
        rgb(174, 119, 84)
    } else {
        rgb(239, 192, 143)
    };
    slab(
        &mut faces,
        footprint(0.045, 0.038),
        0.02,
        0.36,
        [shirt; 3],
        size,
        rotation,
    );
    slab(
        &mut faces,
        footprint(0.040, 0.034),
        0.36,
        0.65,
        [skin; 3],
        size,
        rotation,
    );
    slab(
        &mut faces,
        footprint(0.042, 0.036),
        0.63,
        0.70,
        [rgb(67, 53, 42); 3],
        size,
        rotation,
    );
    faces
}

pub(super) fn train_mesh(park: &Park, rotation: u8, mode: ViewMode) -> Mesh {
    let mut painter = Painter::default();
    if park.width() != park.height() {
        return painter.finish();
    }
    let Some(circuit) = park.circuit() else {
        return painter.finish();
    };
    // A missing piece invalidates the circuit. Never display cars floating
    // across the deleted object while the simulation reports a fault.
    if circuit
        .pieces
        .iter()
        .any(|piece| park.object(piece.object).is_none())
    {
        return painter.finish();
    }
    let poses = park.train_cars();
    let visible: Vec<bool> = poses
        .iter()
        .map(|pose| {
            let center = ground(pose.center);
            let pos = TilePos {
                x: (center.x + 0.5).floor().max(0.0) as u32,
                y: (center.y + 0.5).floor().max(0.0) as u32,
            };
            !(mode == ViewMode::Surface
                && park
                    .tile(pos)
                    .is_some_and(|tile| f32::from(tile.height) > center.z + 1.0))
        })
        .collect();
    for (pose, visible) in poses.iter().zip(&visible) {
        if !visible {
            continue;
        }
        for face in car_faces(*pose, park.width(), rotation) {
            painter.surface(face.surface, face.ink);
        }
    }
    for passenger in park.ride_guest_positions() {
        if passenger.phase != RideVisitPhase::Riding {
            continue;
        }
        let car = usize::from(passenger.seat) / 2;
        if let Some(pose) = poses.get(car).filter(|_| visible.get(car) == Some(&true)) {
            for face in passenger_faces(passenger, *pose, park.width(), rotation) {
                painter.surface(face.surface, face.ink);
            }
        }
    }
    // Couplers are narrow physical beams; they share the cars' actual depth.
    for (index, pair) in poses.windows(2).enumerate() {
        if !visible[index] || !visible[index + 1] {
            continue;
        }
        let a = ground(pair[0].rear);
        let b = ground(pair[1].front);
        let delta = (b - a).truncate().normalize_or_zero();
        let side = Vec2::new(-delta.y, delta.x).extend(0.0);
        let mut faces = Vec::new();
        slab(
            &mut faces,
            strip(a, b, side, side, -0.025, 0.025),
            0.38,
            0.43,
            [rgb(88, 95, 91), rgb(53, 64, 64), rgb(40, 51, 55)],
            park.width(),
            rotation,
        );
        for face in faces {
            painter.surface(face.surface, face.ink);
        }
    }
    painter.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use park_sim::{Speed, TrainPhase};

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            VertexAttributeValues::Float32x3(points) => points,
            _ => panic!("mesh position format changed"),
        }
    }

    /// Independent GPU-style triangle interpolation, not Surface::depth_at.
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

    fn mesh(faces: Vec<Face>) -> Mesh {
        let mut painter = Painter::default();
        for face in faces {
            painter.surface(face.surface, face.ink);
        }
        painter.finish()
    }

    fn fixture() -> Park {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        park
    }

    #[test]
    fn sampled_rails_join_at_every_boundary_including_the_station_seam() {
        let park = fixture();
        let pieces = &park.circuit().unwrap().pieces;
        for (index, piece) in pieces.iter().enumerate() {
            let next = &pieces[(index + 1) % pieces.len()];
            let last = piece.samples.len() - 1;
            assert_eq!(piece.samples[last], next.samples[0]);
            assert!(normal(&park, piece, last).distance(normal(&park, next, 0)) < 0.00001);
            for offset in [-0.195, -0.145, 0.145, 0.195] {
                let a = ground(piece.samples[last]) + normal(&park, piece, last) * offset;
                let b = ground(next.samples[0]) + normal(&park, next, 0) * offset;
                for rotation in 0..4 {
                    assert!(
                        projected(a, 32, rotation).distance(projected(b, 32, rotation)) < 0.001
                    );
                }
            }
        }
    }

    #[test]
    fn sloped_and_curved_pick_depth_matches_emitted_triangles_in_all_rotations() {
        let park = fixture();
        let circuit = park.circuit().unwrap();
        for rotation in 0..4 {
            let meshes: Vec<_> = circuit
                .pieces
                .iter()
                .map(|piece| (piece.object, mesh(piece_faces(&park, piece, rotation))))
                .collect();
            let mut checked = 0;
            for kind in [
                TrackKind::Lift,
                TrackKind::Drop,
                TrackKind::Curve,
                TrackKind::Station,
                TrackKind::Brake,
            ] {
                let piece = circuit
                    .pieces
                    .iter()
                    .find(|piece| piece.kind == kind)
                    .unwrap();
                let object_mesh = &meshes.iter().find(|(id, _)| *id == piece.object).unwrap().1;
                for index in 1..piece.samples.len() - 1 {
                    let center = ground(piece.samples[index]);
                    let normal = normal(&park, piece, index);
                    for across in [-0.17, 0.17] {
                        let point =
                            projected(center + normal * across + Vec3::Z * 0.25, 32, rotation)
                                .truncate();
                        let expected = meshes
                            .iter()
                            .filter_map(|(id, mesh)| {
                                mesh_depth(mesh, point).map(|depth| (depth, *id))
                            })
                            .max_by(|a, b| a.0.total_cmp(&b.0));
                        let Some((_, expected_id)) = expected else {
                            // An edge-on rail has no selectable area.
                            continue;
                        };
                        assert_eq!(
                            pick_surface(point, &park, rotation, ViewMode::Underground),
                            Some(SurfaceId::Object(expected_id)),
                            "kind {kind:?}, rotation {rotation}, sample {index}",
                        );
                        if let Some(expected_depth) = mesh_depth(object_mesh, point) {
                            let picked_depth = piece_faces(&park, piece, rotation)
                                .iter()
                                .filter_map(|face| face.surface.hit_depth(point))
                                .max_by(f32::total_cmp)
                                .unwrap();
                            assert!(
                                (expected_depth - picked_depth).abs() < 0.002,
                                "kind {kind:?}, rotation {rotation}, sample {index}, across {across}: mesh {expected_depth}, pick {picked_depth}"
                            );
                        }
                        checked += 1;
                    }
                }
            }
            assert!(checked >= 30);
        }
    }

    #[test]
    fn lift_roof_and_surface_center_use_sampled_height_not_the_clearance_base() {
        let park = fixture();
        let piece = park
            .circuit()
            .unwrap()
            .pieces
            .iter()
            .find(|piece| piece.kind == TrackKind::Lift)
            .unwrap();
        let object = park.object(piece.object).unwrap();
        let middle = ground(piece.samples[piece.samples.len() / 2]);
        assert!(middle.z > f32::from(object.height));
        for rotation in 0..4 {
            let expected = projected(middle + Vec3::Z * 0.25, 32, rotation).truncate();
            assert_eq!(
                surface_center(&park, SurfaceId::Object(piece.object), rotation),
                Some(expected)
            );
            let faces = piece_faces(&park, piece, rotation);
            let bed_faces: Vec<_> = faces
                .iter()
                .filter(|face| face.ink == rgb(48, 70, 77))
                .collect();
            // Some grades are exactly edge-on from two views. The others must
            // carry the authoritative first and final heights on their roofs.
            for bed in bed_faces {
                let a = bed.surface.vertices[0];
                let b = bed.surface.vertices[1];
                assert!((a.z - b.z).abs() > 0.01);
                let point = (a + b + bed.surface.vertices[2] + bed.surface.vertices[3]) / 4.0;
                assert!((bed.surface.depth_at(point.truncate()) - point.z).abs() < 0.002);
            }
            let outline = selection_mesh(
                None,
                Some(SurfaceId::Object(piece.object)),
                &park,
                rotation,
                ViewMode::Surface,
            );
            assert!(!positions(&outline).is_empty());
            assert!(
                positions(&outline)
                    .iter()
                    .flatten()
                    .all(|value| value.is_finite())
            );
        }
    }

    #[test]
    fn removed_track_has_no_geometry_center_selection_or_floating_train() {
        let mut park = fixture();
        let piece = park.circuit().unwrap().pieces[0].clone();
        park.remove_object(piece.object).unwrap();
        for rotation in 0..4 {
            assert!(piece_faces(&park, &piece, rotation).is_empty());
            assert!(surface_center(&park, SurfaceId::Object(piece.object), rotation).is_none());
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                assert!(positions(&train_mesh(&park, rotation, mode)).is_empty());
                let outline = selection_mesh(
                    None,
                    Some(SurfaceId::Object(piece.object)),
                    &park,
                    rotation,
                    mode,
                );
                assert!(positions(&outline).is_empty());
            }
        }
    }

    #[test]
    fn moving_cars_follow_authoritative_slope_and_turn_poses() {
        let mut park = fixture();
        let initial = positions(&train_mesh(&park, 0, ViewMode::Surface)).to_vec();
        let mut saw_lift = false;
        let mut saw_curve = false;
        for _ in 0..4000 {
            park.advance(Speed::Normal);
            let train = park.train().unwrap();
            let kind = park.circuit().unwrap().pieces[train.piece_index].kind;
            if (train.phase == TrainPhase::Lift && !saw_lift)
                || (kind == TrackKind::Curve && !saw_curve)
            {
                for pose in park.train_cars() {
                    for rotation in 0..4 {
                        let faces = car_faces(pose, 32, rotation);
                        let body = faces
                            .iter()
                            .find(|face| face.ink == rgb(218, 67, 57))
                            .unwrap();
                        let rear = (body.surface.vertices[0] + body.surface.vertices[3]) / 2.0;
                        let front = (body.surface.vertices[1] + body.surface.vertices[2]) / 2.0;
                        assert!(
                            rear.distance(projected(
                                ground(pose.rear) + Vec3::Z * 0.77,
                                32,
                                rotation
                            )) < 0.001
                        );
                        assert!(
                            front.distance(projected(
                                ground(pose.front) + Vec3::Z * 0.77,
                                32,
                                rotation
                            )) < 0.001
                        );
                    }
                }
                assert_ne!(initial, positions(&train_mesh(&park, 0, ViewMode::Surface)));
                saw_lift |= train.phase == TrainPhase::Lift;
                saw_curve |= kind == TrackKind::Curve;
            }
            if saw_lift && saw_curve {
                break;
            }
        }
        assert!(saw_lift && saw_curve);
        for rotation in 0..4 {
            assert_eq!(
                positions(&train_mesh(&park, rotation, ViewMode::Surface)),
                positions(&train_mesh(&park, rotation, ViewMode::Underground)),
            );
        }
    }

    #[test]
    fn car_geometry_is_occluded_by_foreground_terrain_and_elevated_decks() {
        for rotation in 0..4 {
            let pos = unrotate(10, 10, 32, rotation);
            let center = TrackPoint {
                x: pos.x as i32 * TRACK_UNITS,
                y: pos.y as i32 * TRACK_UNITS,
                height: 4 * TRACK_UNITS,
            };
            let pose = TrainPose {
                center,
                rear: TrackPoint {
                    x: center.x - 120,
                    ..center
                },
                front: TrackPoint {
                    x: center.x + 120,
                    ..center
                },
            };
            let car = mesh(car_faces(pose, 32, rotation));
            let point = projected(ground(center) + Vec3::Z * 0.65, 32, rotation).truncate();
            let car_depth = mesh_depth(&car, point).expect("test point is inside the car");
            let foreground = unrotate(11, 11, 32, rotation);
            let mut terrain = Park::new(32, 32);
            terrain.set_terrain_height(foreground, 12).unwrap();
            let mut painter = Painter::default();
            for surface in column_surfaces(&terrain, foreground, rotation)
                .into_iter()
                .flatten()
            {
                painter.surface(surface, rgb(100, 100, 100));
            }
            assert!(mesh_depth(&painter.finish(), point).unwrap() > car_depth);

            let mut deck = Park::new(32, 32);
            let id = deck
                .place_object(foreground, ObjectKind::Path, Axis::X, 6, 2)
                .unwrap();
            let mut painter = Painter::default();
            for surface in object_surfaces(&deck, deck.object(id).unwrap(), rotation) {
                painter.surface(surface, rgb(100, 100, 100));
            }
            assert!(mesh_depth(&painter.finish(), point).unwrap() > car_depth);
        }
    }

    #[test]
    fn passenger_faces_follow_seat_height_orientation_and_foreground_depth_in_all_rotations() {
        for rotation in 0..4 {
            let pos = unrotate(10, 10, 32, rotation);
            let center = TrackPoint {
                x: pos.x as i32 * TRACK_UNITS,
                y: pos.y as i32 * TRACK_UNITS,
                height: 4 * TRACK_UNITS,
            };
            let pose = TrainPose {
                center,
                rear: TrackPoint {
                    x: center.x - 120,
                    height: center.height - 100,
                    ..center
                },
                front: TrackPoint {
                    x: center.x + 120,
                    height: center.height + 100,
                    ..center
                },
            };
            let passenger = park_sim::RideGuestPosition {
                guest: park_sim::GuestId(2),
                seat: 0,
                phase: RideVisitPhase::Riding,
                point: park_sim::PathPoint {
                    x: pos.x as i32 * park_sim::PATH_UNITS,
                    y: pos.y as i32 * park_sim::PATH_UNITS,
                    height: 4 * park_sim::PATH_UNITS + 35,
                },
            };
            let body = mesh(passenger_faces(passenger, pose, 32, rotation));
            assert!(!positions(&body).is_empty());
            assert!(
                positions(&body)
                    .iter()
                    .flatten()
                    .all(|coordinate| coordinate.is_finite())
            );
            let base =
                ground(center) + Vec3::Z * (35.0 / park_sim::PATH_UNITS as f32 + PATH_TOP_OFFSET);
            let point = projected(base + Vec3::Z * 0.2, 32, rotation).truncate();
            let body_depth = mesh_depth(&body, point).expect("inside passenger torso");
            let foreground = unrotate(11, 11, 32, rotation);
            let mut terrain = Park::new(32, 32);
            terrain.set_terrain_height(foreground, 12).unwrap();
            assert!(
                column_surfaces(&terrain, foreground, rotation)
                    .into_iter()
                    .flatten()
                    .filter_map(|surface| surface.hit_depth(point))
                    .any(|depth| depth > body_depth)
            );
            let mut deck = Park::new(32, 32);
            let id = deck.place_path(foreground, 7).unwrap();
            assert!(
                object_surfaces(&deck, deck.object(id).unwrap(), rotation)
                    .into_iter()
                    .filter_map(|surface| surface.hit_depth(point))
                    .any(|depth| depth > body_depth)
            );
            // Every seat figure remains inside the reserved track clearance.
            for vertex in positions(&body) {
                let sum = vertex[2] - BASE_DEPTH;
                let height = (vertex[1] + (sum - 31.0) * HALF_HEIGHT) / HEIGHT_STEP;
                assert!((4.8..5.51).contains(&height), "{height}");
            }
            let turned = TrainPose {
                rear: TrackPoint {
                    x: center.x,
                    y: center.y - 120,
                    ..pose.rear
                },
                front: TrackPoint {
                    x: center.x,
                    y: center.y + 120,
                    ..pose.front
                },
                ..pose
            };
            assert_ne!(
                positions(&body),
                positions(&mesh(passenger_faces(passenger, turned, 32, rotation)))
            );
        }
    }

    #[test]
    fn decorative_supports_skip_another_objects_occupied_column() {
        let mut park = fixture();
        let piece = park
            .circuit()
            .unwrap()
            .pieces
            .iter()
            .find(|piece| piece.kind == TrackKind::Station)
            .unwrap()
            .clone();
        let point = ground(piece.samples[piece.samples.len() / 2]);
        assert!(support_is_clear(&park, piece.object, point).is_some());
        let pos = park.object(piece.object).unwrap().pos;
        park.place_object(pos, ObjectKind::Path, Axis::X, 3, 2)
            .unwrap();
        assert!(support_is_clear(&park, piece.object, point).is_none());
    }
}
