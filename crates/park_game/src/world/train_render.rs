//! Original sampled-track and car geometry. Clearance boxes never become art.

use super::*;
use park_sim::{ObjectId, TRACK_UNITS, TrackKind, TrackPiece, TrackPoint, TrainPose};

const TIMBER_HIGHLIGHT: [u8; 3] = [207, 153, 99];
const CAR_BODY_TOP: [u8; 3] = [240, 96, 66];

pub(super) struct Face {
    pub(super) surface: Surface,
    pub(super) ink: Ink,
}

pub(super) fn piece_for(park: &Park, id: ObjectId) -> Option<&TrackPiece> {
    park.coaster_circuit(park.object_ride(id)?)?
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
fn track_axes(park: &Park, piece: &TrackPiece, index: usize) -> (Vec3, Vec3) {
    let circuit = park
        .coaster_circuit(park.object_ride(piece.object).expect("track owner"))
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
        if !matches!(piece.kind, TrackKind::Curve | TrackKind::BankedCurve) {
            ground(piece.samples[1]) - ground(piece.samples[0])
        } else if !matches!(prior.kind, TrackKind::Curve | TrackKind::BankedCurve) {
            ground(piece.samples[0]) - ground(previous)
        } else {
            ground(next) - ground(previous)
        }
    } else if index + 1 == piece.samples.len() {
        let following = &circuit.pieces[(piece_index + 1) % circuit.pieces.len()];
        if !matches!(piece.kind, TrackKind::Curve | TrackKind::BankedCurve) {
            ground(piece.samples[index]) - ground(previous)
        } else if !matches!(following.kind, TrackKind::Curve | TrackKind::BankedCurve) {
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
    let distance: u32 = piece
        .samples
        .windows(2)
        .take(index)
        .map(|pair| pair[0].distance_to(pair[1]))
        .sum();
    let roll = circuit.specs()[piece_index].roll_millidegrees(distance * 1000 / piece.length);
    if roll == 0 {
        (side.extend(0.0), Vec3::Z)
    } else {
        banked_axes(tangent, roll)
    }
}

fn normal(park: &Park, piece: &TrackPiece, index: usize) -> Vec3 {
    track_axes(park, piece, index).0
}

pub(super) fn banked_axes(tangent: Vec3, roll: i32) -> (Vec3, Vec3) {
    let tangent = (tangent * Vec3::new(1.0, 1.0, 0.25)).normalize_or_zero();
    let side = Vec3::new(-tangent.y, tangent.x, 0.0).normalize_or_zero();
    let up = tangent.cross(side).normalize_or_zero();
    let (sin, cos) = (roll as f32 / 1000.0).to_radians().sin_cos();
    (
        (side * cos + up * sin) * Vec3::new(1.0, 1.0, 4.0),
        (up * cos - side * sin) * Vec3::new(0.25, 0.25, 1.0),
    )
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

fn timber_brace(faces: &mut Vec<Face>, a: Vec3, b: Vec3, ink: [Ink; 3], size: u32, rotation: u8) {
    let delta = b.truncate() - a.truncate();
    let side = Vec2::new(-delta.y, delta.x).normalize_or_zero().extend(0.0);
    slab(
        faces,
        strip(a, b, side, side, -0.025, 0.025),
        0.0,
        0.055,
        ink,
        size,
        rotation,
    );
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
    let paint = park
        .object_ride(piece.object)
        .and_then(|id| park.ride_appearance(id))
        .unwrap_or_default();
    let colors = ride_colors::inks(
        paint.supports,
        [
            rgb_from(TIMBER_HIGHLIGHT),
            rgb(147, 94, 57),
            rgb(96, 61, 43),
        ],
    );
    let mut legs = Vec::with_capacity(2);
    for offset in [-0.23, 0.23] {
        let point = center + side * offset;
        let Some(bottom) = support_is_clear(park, piece.object, point) else {
            continue;
        };
        legs.push((point, bottom));
        let ground_point = Vec3::new(point.x, point.y, bottom);
        let footprint = [
            ground_point + Vec3::new(-0.044, -0.044, 0.0),
            ground_point + Vec3::new(-0.044, 0.044, 0.0),
            ground_point + Vec3::new(0.044, 0.044, 0.0),
            ground_point + Vec3::new(0.044, -0.044, 0.0),
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
    // Braces belong to a clear pair of posts. Never bridge across another
    // object under the track, even when the endpoints themselves are clear.
    if let [(left, left_ground), (right, right_ground)] = legs.as_slice()
        && support_is_clear(park, piece.object, center).is_some()
    {
        let mut level = left_ground.max(*right_ground) + 0.55;
        let roof = left.z.min(right.z) - 0.22;
        while level + 0.54 < roof {
            let low_left = Vec3::new(left.x, left.y, level);
            let low_right = Vec3::new(right.x, right.y, level);
            let high_left = low_left + Vec3::Z * 0.54;
            let high_right = low_right + Vec3::Z * 0.54;
            timber_brace(faces, low_left, low_right, colors, park.width(), rotation);
            timber_brace(faces, high_left, high_right, colors, park.width(), rotation);
            timber_brace(faces, low_left, high_right, colors, park.width(), rotation);
            timber_brace(faces, low_right, high_left, colors, park.width(), rotation);
            level += 0.75;
        }
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
    let model = park
        .ride_model(park.object_ride(piece.object).unwrap())
        .unwrap();
    if model != park_sim::RideModel::ClassicCoaster {
        return catalogue_track_faces(park, piece, rotation, model);
    }
    let size = park.width();
    let paint = park
        .object_ride(piece.object)
        .and_then(|id| park.ride_appearance(id))
        .unwrap_or_default();
    supports(&mut faces, park, piece, rotation);
    for index in 0..piece.samples.len() - 1 {
        let a = ground(piece.samples[index]);
        let b = ground(piece.samples[index + 1]);
        let (na, ua) = track_axes(park, piece, index);
        let (nb, ub) = track_axes(park, piece, index + 1);
        let slab = |faces: &mut Vec<Face>,
                    footprint: [Vec3; 4],
                    bottom: f32,
                    top: f32,
                    colors: [Ink; 3],
                    size: u32,
                    rotation: u8| {
            let up = [ua, ub, ub, ua];
            let upper = std::array::from_fn(|i| footprint[i] + up[i] * top);
            let lower: [Vec3; 4] = std::array::from_fn(|i| footprint[i] + up[i] * bottom);
            face(faces, upper, colors[0], size, rotation);
            for i in 0..4 {
                let next = (i + 1) % 4;
                face(
                    faces,
                    [upper[i], upper[next], lower[next], lower[i]],
                    colors[1 + i % 2],
                    size,
                    rotation,
                );
            }
        };
        let bed = strip(a, b, na, nb, -0.27, 0.27);
        slab(
            &mut faces,
            bed,
            0.0,
            0.10,
            if paint.structure == park_sim::RidePaint::Classic {
                [rgb(48, 70, 77), rgb(35, 51, 59), rgb(29, 43, 50)]
            } else {
                ride_colors::darken(ride_colors::inks(paint.structure, [[0.0; 4]; 3]), 0.36)
            },
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
                ride_colors::inks(
                    paint.structure,
                    [rgb(247, 114, 75), rgb(177, 57, 42), rgb(112, 37, 32)],
                ),
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
                        [rgb(197, 175, 128), rgb(152, 137, 104), rgb(121, 113, 92)],
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
                    // Platform canopies leave the running track open to view.
                    let outer = if lo < 0.0 { lo } else { hi };
                    for at in [0.08, 0.92] {
                        let p = a.lerp(b, at);
                        let n = na.lerp(nb, at);
                        slab(
                            &mut faces,
                            strip(
                                p - (b - a) * 0.025,
                                p + (b - a) * 0.025,
                                n,
                                n,
                                outer - 0.018,
                                outer + 0.018,
                            ),
                            0.24,
                            1.43,
                            [rgb(236, 207, 134), rgb(173, 133, 66), rgb(115, 88, 49)],
                            size,
                            rotation,
                        );
                    }
                    let roof = ride_colors::inks(
                        paint.structure,
                        [rgb(217, 92, 63), rgb(159, 52, 42), rgb(105, 34, 32)],
                    );
                    let ridge = (lo + hi) * 0.5;
                    let left = lo - 0.035;
                    let right = hi + 0.035;
                    let eave = Vec3::Z * 1.45;
                    let peak = Vec3::Z * 1.73;
                    for (x, y, x_height, y_height, ink) in [
                        (left, ridge, eave, peak, roof[0]),
                        (ridge, right, peak, eave, roof[1]),
                    ] {
                        let [p, q, r, s] = strip(a, b, na, nb, x, y);
                        face(
                            &mut faces,
                            [p + x_height, q + x_height, r + y_height, s + y_height],
                            ink,
                            size,
                            rotation,
                        );
                    }
                    for eave_edge in [left, right] {
                        slab(
                            &mut faces,
                            strip(a, b, na, nb, eave_edge - 0.012, eave_edge + 0.012),
                            1.42,
                            1.47,
                            [rgb(249, 210, 117), rgb(190, 145, 63), rgb(123, 88, 36)],
                            size,
                            rotation,
                        );
                    }
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
            TrackKind::Launch => {
                slab(
                    &mut faces,
                    strip(a, b, na, nb, -0.055, 0.055),
                    0.13,
                    0.25,
                    [rgb(73, 213, 230), rgb(31, 128, 161), rgb(24, 83, 112)],
                    size,
                    rotation,
                );
            }
            TrackKind::BankedCurve => {
                for offset in [-0.31, 0.31] {
                    slab(
                        &mut faces,
                        strip(a, b, na, nb, offset - 0.016, offset + 0.016),
                        0.15,
                        0.37,
                        [rgb(244, 190, 72), rgb(166, 106, 48), rgb(116, 76, 40)],
                        size,
                        rotation,
                    );
                }
            }
            TrackKind::Drop
            | TrackKind::Curve
            | TrackKind::Camelback
            | TrackKind::Straight
            | TrackKind::TransitionUp
            | TrackKind::TransitionDown
            | TrackKind::HeartlineRoll => {}
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

#[cfg(test)]
pub(super) fn train_focus(park: &Park, rotation: u8) -> Option<Vec2> {
    coaster_focus(park, park.ride()?.id, rotation)
}

pub(super) fn coaster_focus(park: &Park, id: park_sim::RideId, rotation: u8) -> Option<Vec2> {
    if park.width() != park.height() {
        return None;
    }
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for piece in &park.coaster_circuit(id)?.pieces {
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

/// The leading car's current world position for camera-relative sound.
pub(crate) fn leading_car_focus(park: &Park, id: park_sim::RideId, rotation: u8) -> Option<Vec2> {
    leading_train_focus(park, id, park.coaster_train(id)?.number, rotation)
}

pub(crate) fn leading_train_focus(
    park: &Park,
    id: park_sim::RideId,
    number: u8,
    rotation: u8,
) -> Option<Vec2> {
    if park.width() != park.height() {
        return None;
    }
    let pose = park.coaster_train_cars_for(id, number).into_iter().next()?;
    Some(projected(ground(pose.center), park.width(), rotation).truncate())
}

// Keep edge-on faces until the complete vehicle has been banked. A face that
// starts edge-on can gain visible area after rotating around the track axis.
fn vehicle_slab(
    faces: &mut Vec<Face>,
    footprint: [Vec3; 4],
    bottom: f32,
    top: f32,
    colors: [Ink; 3],
    size: u32,
    rotation: u8,
) {
    let upper = footprint.map(|p| p + Vec3::Z * top);
    let lower = footprint.map(|p| p + Vec3::Z * bottom);
    let mut emit = |vertices: [Vec3; 4], ink| {
        faces.push(Face {
            surface: Surface {
                vertices: vertices.map(|p| projected(p, size, rotation)),
                kind: SurfaceKind::Top,
            },
            ink,
        })
    };
    emit(upper, colors[0]);
    for i in 0..4 {
        let next = (i + 1) % 4;
        emit(
            [upper[i], upper[next], lower[next], lower[i]],
            colors[1 + i % 2],
        );
    }
}

fn bank_vehicle_faces(faces: &mut [Face], pose: TrainPose, center: Vec3, size: u32, rotation: u8) {
    if pose.roll_millidegrees == 0 {
        return;
    }
    let origin = projected(Vec3::ZERO, size, rotation);
    let projection = Mat3::from_cols(
        projected(Vec3::X, size, rotation) - origin,
        projected(Vec3::Y, size, rotation) - origin,
        projected(Vec3::Z, size, rotation) - origin,
    );
    let inverse = projection.inverse();
    let tangent =
        ((ground(pose.front) - ground(pose.rear)) * Vec3::new(1.0, 1.0, 0.25)).normalize_or_zero();
    let bank = Quat::from_axis_angle(
        tangent,
        (pose.roll_millidegrees as f32 / 1000.0).to_radians(),
    );
    for face in faces {
        for vertex in &mut face.surface.vertices {
            let point = inverse * (*vertex - origin);
            let offset = (point - center) * Vec3::new(1.0, 1.0, 0.25);
            *vertex = projected(
                center + (bank * offset) * Vec3::new(1.0, 1.0, 4.0),
                size,
                rotation,
            );
        }
    }
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
    // Round wheel faces and recessed hubs follow each car's banked bogies.
    for along in [0.15, 0.85] {
        for across in [-0.20, 0.20] {
            let center = rear.lerp(front, along) + normal * across + Vec3::Z * 0.32;
            for (radius, vertical, offset, ink) in [
                (0.073, 0.13, 0.0, rgb(27, 39, 45)),
                (0.037, 0.065, 0.002, rgb(163, 182, 181)),
            ] {
                let center = center + normal * across.signum() * offset;
                for segment in 0..12 {
                    let point = |i: usize| {
                        let angle = i as f32 * std::f32::consts::TAU / 12.0;
                        center
                            + direction.extend(0.0) * (angle.cos() * radius)
                            + Vec3::Z * (angle.sin() * vertical)
                    };
                    face(
                        &mut faces,
                        [center, point(segment), point(segment + 1), center],
                        ink,
                        size,
                        rotation,
                    );
                }
            }
        }
    }
    // Angled body sides meet a narrower underframe. The deck retains the
    // authoritative front/rear pose so slope and bank tests remain meaningful.
    let top = footprint(0.0, 1.0, -0.215, 0.215).map(|p| p + Vec3::Z * 0.77);
    let lower = footprint(0.04, 0.96, -0.175, 0.175).map(|p| p + Vec3::Z * 0.36);
    face(&mut faces, top, rgb_from(CAR_BODY_TOP), size, rotation);
    for i in 0..4 {
        let j = (i + 1) % 4;
        face(
            &mut faces,
            [top[i], top[j], lower[j], lower[i]],
            if i % 2 == 0 {
                rgb(160, 40, 45)
            } else {
                rgb(114, 32, 40)
            },
            size,
            rotation,
        );
    }
    // Gold nose, dark seat cushion, and gold side rims form the original livery.
    vehicle_slab(
        &mut faces,
        footprint(0.83, 1.0, -0.215, 0.215),
        0.77,
        0.84,
        [rgb(248, 190, 70), rgb(184, 116, 43), rgb(145, 86, 36)],
        size,
        rotation,
    );
    vehicle_slab(
        &mut faces,
        footprint(0.16, 0.72, -0.15, 0.15),
        0.77,
        0.80,
        [rgb(40, 57, 66), rgb(28, 42, 53), rgb(22, 32, 46)],
        size,
        rotation,
    );
    for (lo, hi) in [(-0.215, -0.18), (0.18, 0.215)] {
        vehicle_slab(
            &mut faces,
            footprint(0.0, 0.83, lo, hi),
            0.77,
            0.83,
            [rgb(236, 173, 64), rgb(168, 104, 39), rgb(134, 78, 33)],
            size,
            rotation,
        );
    }
    // The inset side panel and narrow highlight make the car shape legible at
    // close zoom within the track clearance while preserving ride paint.
    for (lo, hi) in [(-0.219, -0.207), (0.207, 0.219)] {
        vehicle_slab(
            &mut faces,
            footprint(0.18, 0.72, lo, hi),
            0.50,
            0.665,
            [rgb(219, 85, 58), rgb(171, 53, 45), rgb(111, 35, 39)],
            size,
            rotation,
        );
        vehicle_slab(
            &mut faces,
            footprint(0.20, 0.70, lo, hi),
            0.665,
            0.745,
            [rgb(255, 219, 119), rgb(216, 155, 57), rgb(155, 97, 41)],
            size,
            rotation,
        );
    }
    // Two small lamps sit on the leading lip. They share the car's physical
    // depth, so a foreground track or cliff can still hide them.
    for (lo, hi) in [(-0.16, -0.10), (0.10, 0.16)] {
        vehicle_slab(
            &mut faces,
            footprint(0.94, 0.99, lo, hi),
            0.84,
            0.90,
            [rgb(255, 235, 162), rgb(204, 160, 77), rgb(149, 109, 52)],
            size,
            rotation,
        );
    }
    vehicle_slab(
        &mut faces,
        footprint(0.06, 0.19, -0.15, 0.15),
        0.80,
        0.99,
        [rgb(196, 66, 54), rgb(142, 43, 44), rgb(101, 35, 42)],
        size,
        rotation,
    );
    // Divided upholstery, individual restraints and recessed wheel hubs.
    for (left, right) in [(-0.15, -0.018), (0.018, 0.15)] {
        vehicle_slab(
            &mut faces,
            footprint(0.14, 0.24, left, right),
            0.80,
            0.99,
            [rgb(42, 62, 74), rgb(28, 42, 53), rgb(22, 32, 46)],
            size,
            rotation,
        );
        vehicle_slab(
            &mut faces,
            footprint(0.54, 0.59, left, right),
            0.83,
            0.89,
            [rgb(210, 226, 225), rgb(123, 148, 152), rgb(70, 88, 98)],
            size,
            rotation,
        );
    }
    for (lo, hi) in [(-0.219, -0.209), (0.209, 0.219)] {
        vehicle_slab(
            &mut faces,
            footprint(0.08, 0.92, lo, hi),
            0.47,
            0.50,
            [rgb(248, 190, 70), rgb(184, 116, 43), rgb(145, 86, 36)],
            size,
            rotation,
        );
    }
    bank_vehicle_faces(&mut faces, pose, ground(pose.center), size, rotation);
    faces.retain(|f| f.surface.has_area());
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
    vehicle_slab(
        &mut faces,
        footprint(0.045, 0.038),
        0.02,
        0.36,
        [shirt; 3],
        size,
        rotation,
    );
    vehicle_slab(
        &mut faces,
        footprint(0.040, 0.034),
        0.36,
        0.65,
        [skin; 3],
        size,
        rotation,
    );
    vehicle_slab(
        &mut faces,
        footprint(0.042, 0.036),
        0.63,
        0.70,
        [rgb(67, 53, 42); 3],
        size,
        rotation,
    );
    bank_vehicle_faces(&mut faces, pose, center, size, rotation);
    faces.retain(|f| f.surface.has_area());
    faces
}

pub(super) fn train_mesh(park: &Park, rotation: u8, mode: ViewMode) -> Mesh {
    let mut painter = Painter::default();
    if park.width() != park.height() {
        return painter.finish();
    }
    super::carousel_render::paint_dynamic(&mut painter, park, rotation, mode);
    for id in park.coaster_ids() {
        paint_coaster_train(&mut painter, park, id, rotation, mode);
    }
    painter.finish()
}

fn paint_coaster_train(
    painter: &mut Painter,
    park: &Park,
    id: park_sim::RideId,
    rotation: u8,
    mode: ViewMode,
) {
    let Some(circuit) = park.coaster_circuit(id) else {
        return;
    };
    // A missing piece invalidates the circuit. Never display cars floating
    // across the deleted object while the simulation reports a fault.
    if circuit
        .pieces
        .iter()
        .any(|piece| park.object(piece.object).is_none())
    {
        return;
    }
    for train in park.coaster_trains(id) {
        paint_single_train(painter, park, id, train.number, rotation, mode);
    }
    if park.coaster_train_count(id).unwrap_or(1) > 1 {
        let blocks = park.coaster_block_occupancy(id);
        for (piece, occupied) in circuit
            .pieces
            .iter()
            .filter(|p| matches!(p.kind, TrackKind::Brake | TrackKind::Station))
            .zip(blocks)
        {
            let point = ground(piece.samples[0]) + normal(park, piece, 0) * 0.40;
            let pos = TilePos {
                x: point.x.round().max(0.0) as u32,
                y: point.y.round().max(0.0) as u32,
            };
            if mode == ViewMode::Surface
                && park
                    .tile(pos)
                    .is_some_and(|t| f32::from(t.height) > point.z + 1.0)
            {
                continue;
            }
            let mut faces = Vec::new();
            let square = |half: f32| {
                [
                    point + Vec3::new(-half, -half, 0.0),
                    point + Vec3::new(-half, half, 0.0),
                    point + Vec3::new(half, half, 0.0),
                    point + Vec3::new(half, -half, 0.0),
                ]
            };
            slab(
                &mut faces,
                square(0.025),
                0.2,
                0.8,
                [rgb(39, 53, 58); 3],
                park.width(),
                rotation,
            );
            let light = if occupied.is_some() {
                rgb(249, 79, 57)
            } else {
                rgb(82, 236, 115)
            };
            slab(
                &mut faces,
                square(0.055),
                0.72,
                0.87,
                [light; 3],
                park.width(),
                rotation,
            );
            for face in faces {
                painter.surface(face.surface, face.ink);
            }
        }
    }
}

fn paint_single_train(
    painter: &mut Painter,
    park: &Park,
    id: park_sim::RideId,
    number: u8,
    rotation: u8,
    mode: ViewMode,
) {
    let seats = park.coaster_seats_per_train(id).unwrap_or(6);
    let poses = park.coaster_train_cars_for(id, number);
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
    let model = park.ride_model(id).unwrap();
    for (car, (pose, visible)) in poses.iter().zip(&visible).enumerate() {
        if !visible {
            continue;
        }
        for face in catalogue_car_faces(*pose, park.width(), rotation, model, car) {
            let paint = park.ride_appearance(id).unwrap_or_default().vehicles;
            painter.surface(face.surface, ride_colors::vehicle(paint, face.ink));
        }
        if model == park_sim::RideModel::LogFlume {
            let front = ground(pose.front);
            let rear = ground(pose.rear);
            if rear.z - front.z > 0.08 {
                let tip = projected(front + Vec3::Z * 0.24, park.width(), rotation);
                let spread = 5.0 + (park.ticks() % 5) as f32;
                for side in [-1.0, 1.0] {
                    painter.line(
                        tip.truncate(),
                        tip.truncate() + Vec2::new(side * spread, 5.0),
                        1.1,
                        tip.z + DETAIL_BIAS,
                        rgb(208, 244, 241),
                    );
                }
            }
        }
    }
    for passenger in park.ride_guest_positions() {
        if passenger.phase != RideVisitPhase::Riding
            || id != passenger.ride
            || passenger.seat / seats != number
        {
            continue;
        }
        let car = usize::from((passenger.seat % seats) / model.seats_per_car());
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
}

fn catalogue_track_faces(
    park: &Park,
    piece: &TrackPiece,
    rotation: u8,
    model: park_sim::RideModel,
) -> Vec<Face> {
    use park_sim::RideModel as M;
    let mut out = Vec::new();
    let size = park.width();
    let paint = park
        .ride_appearance(park.object_ride(piece.object).unwrap())
        .unwrap_or_default();
    let color = ride_colors::inks(
        paint.structure,
        match model {
            M::LogFlume => [rgb(170, 117, 69), rgb(118, 78, 48), rgb(78, 52, 35)],
            M::RiverRapids => [rgb(88, 180, 147), rgb(47, 127, 105), rgb(28, 85, 79)],
            M::ParkRailway => [rgb(196, 102, 64), rgb(131, 63, 46), rgb(86, 44, 34)],
            M::Monorail => [rgb(217, 234, 234), rgb(136, 177, 193), rgb(86, 119, 146)],
            M::InvertedCoaster => [rgb(162, 117, 211), rgb(114, 72, 157), rgb(70, 47, 108)],
            _ => [rgb(243, 89, 69), rgb(163, 41, 51), rgb(97, 26, 36)],
        },
    );
    let steel = [rgb(185, 210, 218), rgb(100, 132, 151), rgb(57, 86, 108)];
    let water = [rgb(38, 193, 232), rgb(21, 116, 178), rgb(14, 66, 119)];
    for i in 0..piece.samples.len() - 1 {
        let a = ground(piece.samples[i]);
        let b = ground(piece.samples[i + 1]);
        let (na, ua) = track_axes(park, piece, i);
        let (nb, ub) = track_axes(park, piece, i + 1);
        let mut band = |left: f32, right: f32, bottom: f32, top: f32, inks: [Ink; 3]| {
            let p = strip(a, b, na, nb, left, right);
            let u = [ua, ub, ub, ua];
            let high = std::array::from_fn(|j| p[j] + u[j] * top);
            let low: [Vec3; 4] = std::array::from_fn(|j| p[j] + u[j] * bottom);
            face(&mut out, high, inks[0], size, rotation);
            for j in 0..4 {
                let n = (j + 1) % 4;
                face(
                    &mut out,
                    [high[j], high[n], low[n], low[j]],
                    inks[1 + j % 2],
                    size,
                    rotation,
                );
            }
        };
        match model {
            M::LogFlume | M::RiverRapids => {
                band(-0.46, 0.46, 0.0, 0.12, steel);
                band(-0.39, 0.39, 0.12, 0.17, water);
                for (l, r) in [(-0.47, -0.39), (0.39, 0.47)] {
                    band(l, r, 0.12, 0.65, color);
                }
            }
            M::Monorail => {
                band(-0.12, 0.12, 0.0, 0.42, steel);
                band(-0.16, 0.16, 0.42, 0.48, color);
            }
            M::ParkRailway => {
                band(-0.30, 0.30, 0.0, 0.12, [rgb(117, 102, 83); 3]);
                for (l, r) in [(-0.19, -0.15), (0.15, 0.19)] {
                    band(l, r, 0.12, 0.25, steel);
                }
            }
            M::InvertedCoaster => {
                band(-0.12, 0.12, 1.15, 1.38, color);
                for (l, r) in [(-0.23, -0.17), (0.17, 0.23)] {
                    band(l, r, 1.30, 1.42, steel);
                }
            }
            _ => {
                band(-0.075, 0.075, 0.04, 0.19, color);
                for (l, r) in [(-0.21, -0.14), (0.14, 0.21)] {
                    band(l, r, 0.20, 0.28, steel);
                }
            }
        }
        if piece.kind == TrackKind::Station {
            band(
                -0.49,
                -0.31,
                0.0,
                0.25,
                [rgb(238, 217, 156), rgb(165, 129, 79), rgb(111, 84, 53)],
            );
            band(
                0.31,
                0.49,
                0.0,
                0.25,
                [rgb(238, 217, 156), rgb(165, 129, 79), rgb(111, 84, 53)],
            );
        }
    }
    let mut detail = Vec::new();
    let timber = [rgb(177, 122, 68), rgb(115, 73, 40), rgb(73, 45, 29)];
    let brass = [rgb(248, 204, 95), rgb(174, 128, 51), rgb(107, 75, 34)];
    for pair in piece.samples.windows(2) {
        let a = ground(pair[0]);
        let b = ground(pair[1]);
        let length = a.truncate().distance(b.truncate());
        let count = (length / 0.18).ceil().max(1.0) as usize;
        for j in 0..count {
            let t = (j as f32 + 0.5) / count as f32;
            let center = a.lerp(b, t);
            let tangent = (b - a).truncate().normalize_or_zero().extend(0.0);
            let side = Vec3::new(-tangent.y, tangent.x, 0.0);
            match model {
                M::ParkRailway => {
                    ride_detail::tube(
                        &mut detail,
                        center - side * 0.28 + Vec3::Z * 0.14,
                        center + side * 0.28 + Vec3::Z * 0.14,
                        0.024,
                        timber,
                    );
                }
                M::LogFlume | M::RiverRapids => {
                    for sign in [-1.0, 1.0] {
                        let rim = center + side * sign * 0.455;
                        ride_detail::tube(
                            &mut detail,
                            rim + Vec3::Z * 0.17,
                            rim + Vec3::Z * 0.64,
                            0.012,
                            brass,
                        );
                    }
                    // Thin foam streaks sit inside the channel, below vehicles.
                    if j % 2 == 0 {
                        ride_detail::tube(
                            &mut detail,
                            center - side * 0.20 + Vec3::Z * 0.174,
                            center + side * 0.07 + Vec3::Z * 0.174,
                            0.006,
                            [rgb(172, 234, 242); 3],
                        );
                    }
                }
                M::Monorail => {}
                _ => {
                    let z = if model == M::InvertedCoaster {
                        1.30
                    } else {
                        0.19
                    };
                    ride_detail::tube(
                        &mut detail,
                        center - side * 0.21 + Vec3::Z * z,
                        center + side * 0.21 + Vec3::Z * z,
                        0.018,
                        color,
                    );
                }
            }
        }
        if piece.kind == TrackKind::Station {
            let tangent = (b - a).truncate().normalize_or_zero().extend(0.0);
            let side = Vec3::new(-tangent.y, tangent.x, 0.0);
            for sign in [-1.0, 1.0] {
                // The roof covers the platform only; the running lane stays open.
                let left = a + side * sign * 0.34;
                let right = b + side * sign * 0.34;
                for t in [0.12, 0.88] {
                    let post = a.lerp(b, t) + side * sign * 0.46;
                    ride_detail::tube(
                        &mut detail,
                        post + Vec3::Z * 0.25,
                        post + Vec3::Z * 1.48,
                        0.015,
                        brass,
                    );
                }
                let eaves = Vec3::Z * 1.50;
                let ridge = Vec3::Z * 1.70;
                for (edge, ink) in [(-0.13, color[0]), (0.13, color[1])] {
                    ride_detail::panel(
                        &mut detail,
                        [
                            left + side * edge + eaves,
                            right + side * edge + eaves,
                            right + ridge,
                            left + ridge,
                        ],
                        ink,
                    );
                }
                ride_detail::tube(
                    &mut detail,
                    left + side * sign * 0.12 + Vec3::Z * 1.50,
                    right + side * sign * 0.12 + Vec3::Z * 1.50,
                    0.012,
                    brass,
                );
            }
        }
    }
    out.extend(
        ride_detail::project(detail, size, rotation)
            .into_iter()
            .map(|f| Face {
                surface: f.surface,
                ink: f.ink,
            }),
    );
    let index = piece.samples.len() / 2;
    let point = ground(piece.samples[index]);
    let height = if model == M::InvertedCoaster {
        1.4
    } else {
        0.12
    };
    if let Some(bottom) = support_is_clear(park, piece.object, point + Vec3::Z * height) {
        let side = normal(park, piece, index);
        for offset in [-0.35, 0.35] {
            let p = point + side * offset;
            let q = Vec3::new(p.x, p.y, bottom);
            slab(
                &mut out,
                [
                    q + Vec3::new(-0.03, -0.03, 0.0),
                    q + Vec3::new(-0.03, 0.03, 0.0),
                    q + Vec3::new(0.03, 0.03, 0.0),
                    q + Vec3::new(0.03, -0.03, 0.0),
                ],
                0.0,
                point.z + height - bottom,
                steel,
                size,
                rotation,
            );
        }
    }
    out
}
fn catalogue_car_faces(
    pose: TrainPose,
    size: u32,
    rotation: u8,
    model: park_sim::RideModel,
    car: usize,
) -> Vec<Face> {
    use park_sim::RideModel as M;
    if matches!(model, M::ClassicCoaster | M::SteelCoaster) {
        return car_faces(pose, size, rotation);
    }
    if model == M::RiverRapids {
        let center = ground(pose.center);
        let mut body = Vec::new();
        let rubber = [rgb(49, 61, 74), rgb(25, 35, 47), rgb(17, 25, 36)];
        let red = [rgb(240, 63, 75), rgb(172, 35, 51), rgb(106, 22, 39)];
        let gold = [rgb(254, 198, 56), rgb(176, 116, 36), rgb(126, 75, 25)];
        ride_detail::lathe(
            &mut body,
            center,
            &[
                (0.28, 0.20),
                (0.40, 0.24),
                (0.45, 0.37),
                (0.43, 0.46),
                (0.29, 0.46),
            ],
            rubber,
        );
        ride_detail::lathe(
            &mut body,
            center,
            &[
                (0.28, 0.38),
                (0.37, 0.45),
                (0.37, 0.70),
                (0.30, 0.74),
                (0.25, 0.60),
            ],
            red,
        );
        ride_detail::lathe(
            &mut body,
            center,
            &[(0.25, 0.61), (0.36, 0.70), (0.37, 0.75), (0.26, 0.70)],
            gold,
        );
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::TAU / 6.0;
            let direction = Vec3::new(angle.cos(), angle.sin(), 0.0);
            ride_detail::tube(
                &mut body,
                center + direction * 0.30 + Vec3::Z * 0.50,
                center + direction * 0.30 + Vec3::Z * 0.75,
                0.014,
                gold,
            );
        }
        let mut faces = ride_detail::project(body, size, rotation)
            .into_iter()
            .map(|f| Face {
                surface: f.surface,
                ink: f.ink,
            })
            .collect::<Vec<_>>();
        bank_vehicle_faces(&mut faces, pose, center, size, rotation);
        return faces;
    }
    let mut out = Vec::new();
    let a = ground(pose.rear);
    let b = ground(pose.front);
    let d = (b - a).truncate().normalize_or_zero();
    let n = Vec3::new(-d.y, d.x, 0.0);
    let mut part =
        |lo: f32, hi: f32, left: f32, right: f32, bottom: f32, top: f32, color: [Ink; 3]| {
            vehicle_slab(
                &mut out,
                strip(a.lerp(b, lo), a.lerp(b, hi), n, n, left, right),
                bottom,
                top,
                color,
                size,
                rotation,
            );
        };
    let red = [rgb(240, 63, 75), rgb(172, 35, 51), rgb(106, 22, 39)];
    let gold = [rgb(254, 198, 56), rgb(176, 116, 36), rgb(126, 75, 25)];
    let dark = [rgb(51, 64, 78), rgb(28, 39, 53), rgb(18, 28, 42)];
    let glass = [rgb(163, 235, 246), rgb(70, 151, 180), rgb(35, 95, 137)];
    match model {
        M::LogFlume => {
            part(0.0, 1.0, -0.30, 0.30, 0.18, 0.38, gold);
            for (l, r) in [(-0.30, -0.25), (0.25, 0.30)] {
                part(0.0, 1.0, l, r, 0.38, 0.85, red);
            }
            part(0.90, 1.0, -0.25, 0.25, 0.38, 0.75, gold);
            for h in [0.45, 0.57, 0.69, 0.80] {
                for (lo, hi) in [(-0.305, -0.295), (0.295, 0.305)] {
                    part(0.04, 0.94, lo, hi, h, h + 0.017, dark);
                }
            }
            for at in [0.22, 0.48, 0.72] {
                part(at, at + 0.07, -0.24, 0.24, 0.39, 0.52, dark);
            }
        }
        M::InvertedCoaster => {
            part(0.05, 0.85, -0.21, 0.21, 0.36, 0.65, red);
            part(0.04, 0.15, -0.035, 0.035, 0.60, 1.43, dark);
            part(0.7, 0.8, -0.035, 0.035, 0.60, 1.43, dark);
            part(0.03, 0.16, -0.22, 0.22, 1.36, 1.46, gold);
        }
        M::ParkRailway if car == 0 => {
            part(0.0, 1.0, -0.25, 0.25, 0.25, 0.70, dark);
            part(0.45, 0.60, -0.10, 0.10, 0.7, 1.1, dark);
            part(0.0, 0.4, -0.25, 0.25, 0.70, 1.25, red);
            part(0.0, 0.4, -0.20, 0.20, 0.9, 1.15, glass);
            part(-0.04, 0.44, -0.29, 0.29, 1.25, 1.35, gold);
            part(0.81, 0.87, -0.11, 0.11, 0.70, 0.92, gold);
            part(0.76, 0.92, -0.13, 0.13, 0.90, 0.97, dark);
            part(0.45, 0.59, -0.16, 0.16, 0.67, 0.74, gold);
        }
        _ => {
            part(0.0, 1.0, -0.25, 0.25, 0.28, 0.75, red);
            part(0.0, 1.0, -0.24, 0.24, 0.75, 1.25, glass);
            part(-0.03, 1.03, -0.28, 0.28, 1.25, 1.36, gold);
            for x in [0.0, 0.45, 0.92] {
                part(x, x + 0.05, -0.26, 0.26, 0.7, 1.26, dark);
            }
        }
    }
    if model == M::ParkRailway {
        for at in [0.14, 0.42, 0.78] {
            for (left, right) in [(-0.285, -0.245), (0.245, 0.285)] {
                part(at, at + 0.14, left, right, 0.21, 0.42, dark);
                part(
                    at + 0.04,
                    at + 0.10,
                    left - 0.007,
                    right + 0.007,
                    0.28,
                    0.35,
                    gold,
                );
            }
        }
        for (left, right) in [(-0.261, -0.25), (0.25, 0.261)] {
            part(0.08, 0.92, left, right, 0.57, 0.62, gold);
        }
    }
    // Rounded end caps, side lining and real wheel silhouettes distinguish
    // transport vehicles at the same zoom as a coaster train.
    if model == M::Monorail {
        part(0.80, 1.04, -0.22, 0.22, 0.40, 0.76, gold);
        part(0.88, 1.02, -0.20, 0.20, 0.78, 1.05, glass);
        for (lo, hi) in [(-0.268, -0.256), (0.256, 0.268)] {
            part(0.04, 0.95, lo, hi, 0.59, 0.64, gold);
            for window in [0.12, 0.36, 0.60, 0.84] {
                part(window, window + 0.018, lo, hi, 0.80, 1.20, dark);
            }
        }
    }
    if model == M::ParkRailway && car == 0 {
        // Faceted cylindrical boiler follows the authoritative car tangent.
        let mut boiler = Vec::new();
        ride_detail::tube(
            &mut boiler,
            Vec3::new(0.35, 0.0, 0.75),
            Vec3::new(0.93, 0.0, 0.75),
            0.17,
            dark,
        );
        for ring in [0.40, 0.67, 0.88] {
            ride_detail::tube(
                &mut boiler,
                Vec3::new(ring, 0.0, 0.75),
                Vec3::new(ring + 0.025, 0.0, 0.75),
                0.18,
                gold,
            );
        }
        for f in &mut boiler {
            for p in &mut f.surface.vertices {
                let fraction = p.x;
                *p = a.lerp(b, fraction) + n * p.y + Vec3::Z * p.z;
            }
        }
        out.extend(
            ride_detail::project(boiler, size, rotation)
                .into_iter()
                .map(|f| Face {
                    surface: f.surface,
                    ink: f.ink,
                }),
        );
    }
    bank_vehicle_faces(&mut out, pose, ground(pose.center), size, rotation);
    out.retain(|f| f.surface.has_area());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use park_sim::{Speed, TrainPhase};

    #[test]
    fn banked_and_inverted_track_joins_and_vehicle_geometry_agree_in_every_rotation() {
        let mut source = Park::new(32, 32);
        source.install_train_fixture().unwrap();
        let mut specs = source.circuit().unwrap().specs().to_vec();
        let bank = specs
            .iter()
            .position(|s| s.kind == TrackKind::Curve)
            .unwrap();
        specs[bank].kind = TrackKind::BankedCurve;
        let roll = specs
            .iter()
            .position(|s| s.kind == TrackKind::Straight)
            .unwrap();
        specs[roll].kind = TrackKind::HeartlineRoll;
        let mut park = Park::new(32, 32);
        park.install_train_circuit(&specs).unwrap();
        let pieces = &park.circuit().unwrap().pieces;
        for (i, piece) in pieces.iter().enumerate() {
            let next = &pieces[(i + 1) % pieces.len()];
            let end = piece.samples.len() - 1;
            assert!(normal(&park, piece, end).distance(normal(&park, next, 0)) < 0.00001);
            for rotation in 0..4 {
                let a = ground(piece.samples[end]) + normal(&park, piece, end) * 0.17;
                let b = ground(next.samples[0]) + normal(&park, next, 0) * 0.17;
                assert!(projected(a, 32, rotation).distance(projected(b, 32, rotation)) < 0.001);
            }
        }
        let center = TrackPoint {
            x: 10 * TRACK_UNITS,
            y: 10 * TRACK_UNITS,
            height: 4 * TRACK_UNITS,
        };
        let pose = TrainPose {
            center,
            front: TrackPoint {
                x: center.x + 120,
                ..center
            },
            rear: TrackPoint {
                x: center.x - 120,
                ..center
            },
            roll_millidegrees: 180_000,
        };
        let diagonal = TrainPose {
            front: TrackPoint {
                x: center.x + 85,
                y: center.y + 85,
                ..center
            },
            rear: TrackPoint {
                x: center.x - 85,
                y: center.y - 85,
                ..center
            },
            roll_millidegrees: 0,
            ..pose
        };
        let banked = TrainPose {
            roll_millidegrees: 30_000,
            ..diagonal
        };
        assert!(car_faces(banked, 32, 0).len() > car_faces(diagonal, 32, 0).len());
        let seat = pose.bank_point(TrackPoint {
            height: center.height + 350,
            ..center
        });
        assert!(seat.height < center.height);
        for rotation in 0..4 {
            let origin = projected(Vec3::ZERO, 32, rotation);
            let inverse = Mat3::from_cols(
                projected(Vec3::X, 32, rotation) - origin,
                projected(Vec3::Y, 32, rotation) - origin,
                projected(Vec3::Z, 32, rotation) - origin,
            )
            .inverse();
            let faces = car_faces(pose, 32, rotation);
            assert!(!faces.is_empty());
            for face in faces {
                for point in face.surface.vertices {
                    let point = inverse * (point - origin);
                    assert!(point.is_finite());
                    assert!(point.z < 4.0 && point.z > 2.99);
                }
            }
        }
    }

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
                            .find(|face| face.ink == rgb_from(CAR_BODY_TOP))
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
                roll_millidegrees: 0,
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
                roll_millidegrees: 0,
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
                ride: park_sim::RideId(1),
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
        let clean = piece_faces(&park, &piece, 0);
        assert!(
            clean
                .iter()
                .any(|face| face.ink == rgb_from(TIMBER_HIGHLIGHT))
        );
        let pos = park.object(piece.object).unwrap().pos;
        park.place_object(pos, ObjectKind::Path, Axis::X, 3, 2)
            .unwrap();
        assert!(support_is_clear(&park, piece.object, point).is_none());
        let blocked = piece_faces(&park, &piece, 0);
        assert!(blocked.len() < clean.len());
        assert!(
            !blocked
                .iter()
                .any(|face| face.ink == rgb_from(TIMBER_HIGHLIGHT))
        );
    }
}
