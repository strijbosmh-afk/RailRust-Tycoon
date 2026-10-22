//! Persistent CPU buffers and procedural guest templates. The original
//! `guest_mesh` remains the independently generated geometry oracle.

use std::collections::HashMap;

use bevy::mesh::VertexAttributeValues;
use park_sim::{ObjectId, WALK_TICKS};

use super::*;

const APPEARANCES: usize = 6;
const POSES: usize = WALK_TICKS as usize + 2;
// A convex twelve-point shadow clipped by four edges has at most sixteen
// vertices: 42 triangle vertices per supporting roof. A lost body has 126;
// an ordinary body plus any visit marker stays below that bound.
const MAX_GUEST_VERTICES: usize = 2 * 42 + 126;
const CLIPPED_POINTS: usize = 16;

/// Algorithm counters, not allocator instrumentation or GPU measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GuestRenderStats {
    pub updates: u64,
    pub deck_rebuilds: u64,
    /// Body-template builds; fixed visit markers are separate.
    pub template_builds: u64,
    pub buffer_growths: u64,
    pub rendered_guests: usize,
    pub vertices: usize,
}

struct BodyTemplate {
    positions: Vec<[f32; 3]>,
    colors: Vec<Ink>,
}

#[derive(Clone, Copy)]
struct Deck {
    object: SurfaceObject,
    roof: Surface,
}

/// Own one renderer alongside its mesh. Every update reads current guest
/// state; ticks, IDs and object counts are never used as load identity.
#[derive(Resource, Default)]
pub struct GuestRenderer {
    objects: Vec<SurfaceObject>,
    decks: Vec<Deck>,
    object_index: HashMap<ObjectId, usize>,
    geometry_key: Option<(u32, u32, u8)>,
    bodies: Vec<BodyTemplate>,
    markers: Vec<BodyTemplate>,
    shadow: [Vec2; 12],
    shadow_color: Ink,
    stats: GuestRenderStats,
}

impl GuestRenderer {
    pub fn empty_mesh() -> Mesh {
        Painter::default().finish()
    }

    pub fn stats(&self) -> GuestRenderStats {
        self.stats
    }

    /// Update the existing mesh's five vectors in place. There is no
    /// per-guest heap allocation, guest-ID scan, or path-neighbor query.
    pub fn update(&mut self, mesh: &mut Mesh, park: &Park, rotation: u8, mode: ViewMode) {
        self.stats.updates += 1;
        self.stats.rendered_guests = 0;
        let rotation = rotation % 4;
        let square = park.width() == park.height();
        if square {
            self.prepare(park, rotation);
        }
        // At most six reserved train seats; sample once, outside the guest loop.
        let transfers = park.ride_guest_positions();
        // Accept a fresh mesh as well as one created by empty_mesh. Once
        // initialized, attributes remain inserted and retain their allocation.
        if !matches!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION),
            Some(VertexAttributeValues::Float32x3(_))
        ) || !matches!(
            mesh.attribute(Mesh::ATTRIBUTE_COLOR),
            Some(VertexAttributeValues::Float32x4(_))
        ) || !matches!(
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
            Some(VertexAttributeValues::Float32x3(_))
        ) || !matches!(
            mesh.attribute(Mesh::ATTRIBUTE_UV_0),
            Some(VertexAttributeValues::Float32x2(_))
        ) || !matches!(mesh.indices(), Some(Indices::U32(_)))
        {
            *mesh = Self::empty_mesh();
        }
        let reserve = if square {
            park.guests().len().saturating_mul(MAX_GUEST_VERTICES)
        } else {
            0
        };
        let count = {
            let mut positions = None;
            let mut colors = None;
            let mut normals = None;
            let mut uvs = None;
            for (attribute, values) in mesh.attributes_mut() {
                if attribute.id == Mesh::ATTRIBUTE_POSITION.id {
                    if let VertexAttributeValues::Float32x3(values) = values {
                        positions = Some(values);
                    }
                } else if attribute.id == Mesh::ATTRIBUTE_COLOR.id {
                    if let VertexAttributeValues::Float32x4(values) = values {
                        colors = Some(values);
                    }
                } else if attribute.id == Mesh::ATTRIBUTE_NORMAL.id {
                    if let VertexAttributeValues::Float32x3(values) = values {
                        normals = Some(values);
                    }
                } else if attribute.id == Mesh::ATTRIBUTE_UV_0.id
                    && let VertexAttributeValues::Float32x2(values) = values
                {
                    uvs = Some(values);
                }
            }
            let positions = positions.expect("position attribute was initialized");
            let colors = colors.expect("color attribute was initialized");
            let normals = normals.expect("normal attribute was initialized");
            let uvs = uvs.expect("UV attribute was initialized");
            reserve_buffer(positions, reserve, &mut self.stats.buffer_growths);
            reserve_buffer(colors, reserve, &mut self.stats.buffer_growths);
            reserve_buffer(normals, reserve, &mut self.stats.buffer_growths);
            reserve_buffer(uvs, reserve, &mut self.stats.buffer_growths);
            positions.clear();
            colors.clear();
            if square {
                let mut writer = Writer {
                    positions: &mut *positions,
                    colors: &mut *colors,
                };
                for guest in park.guests() {
                    if guest_is_riding(guest) {
                        continue;
                    }
                    let current = guest.current.and_then(|id| self.deck(id));
                    if current.is_some_and(|deck| !visible(park, deck.object, mode)) {
                        continue;
                    }
                    let next = guest.next.and_then(|id| self.deck(id));
                    let point = if guest_is_transfer(guest) {
                        let Some(position) =
                            transfers.iter().find(|position| position.guest == guest.id)
                        else {
                            continue;
                        };
                        position.point
                    } else {
                        guest
                            .position(
                                current.as_ref().map(|deck| &deck.object),
                                next.as_ref().map(|deck| &deck.object),
                            )
                            .point
                    };
                    let base = project_path_point(point, park.width(), rotation);
                    for deck in guest_supports(guest)
                        .into_iter()
                        .flatten()
                        .filter_map(|id| self.deck(id))
                    {
                        writer.shadow(deck.roof, base.truncate(), &self.shadow, self.shadow_color);
                    }
                    let pose = guest_walk_progress(guest).map_or_else(
                        || match guest.state {
                            GuestState::Lost => usize::from(WALK_TICKS) + 1,
                            _ => usize::from(WALK_TICKS),
                        },
                        usize::from,
                    );
                    // Guest progress is validated below WALK_TICKS by the
                    // simulation and save codec, including diagnostic inputs.
                    debug_assert!(
                        guest.state != GuestState::Walking || guest.progress < WALK_TICKS
                    );
                    let appearance = (guest.id.0 % APPEARANCES as u64) as usize;
                    writer.body(&self.bodies[appearance * POSES + pose], base);
                    if let Some(phase) = guest_visit_marker(guest) {
                        writer.body(&self.markers[marker_index(phase)], base);
                    }
                    self.stats.rendered_guests += 1;
                }
            }
            let count = positions.len();
            normals.resize(count, [0.0, 0.0, 1.0]);
            uvs.resize(count, [0.0, 0.0]);
            count
        };
        let Some(Indices::U32(indices)) = mesh.indices_mut() else {
            unreachable!("indices were initialized");
        };
        reserve_buffer(indices, reserve, &mut self.stats.buffer_growths);
        if indices.len() < count {
            indices.extend(indices.len() as u32..count as u32);
        } else {
            indices.truncate(count);
        }
        self.stats.vertices = count;
    }

    fn deck(&self, id: ObjectId) -> Option<Deck> {
        self.object_index.get(&id).map(|index| self.decks[*index])
    }

    fn prepare(&mut self, park: &Park, rotation: u8) {
        if self.bodies.is_empty() {
            self.shadow = ellipse_points(Vec2::ZERO, 5.3, 2.6);
            self.shadow_color = rgb(145, 112, 77);
            self.bodies.reserve(APPEARANCES * POSES);
            for appearance in 0..APPEARANCES {
                for pose in 0..POSES {
                    self.bodies.push(body_template(appearance, pose));
                    self.stats.template_builds += 1;
                }
            }
            self.markers = [
                RideVisitPhase::Approaching,
                RideVisitPhase::Queueing,
                RideVisitPhase::Leaving,
                RideVisitPhase::Boarding,
                RideVisitPhase::Riding,
                RideVisitPhase::Exiting,
            ]
            .map(marker_template)
            .into();
        }
        let key = (park.width(), park.height(), rotation);
        if self.geometry_key == Some(key) && self.objects == park.objects() {
            return;
        }
        self.objects.clear();
        self.objects.extend_from_slice(park.objects());
        self.decks.clear();
        self.decks.reserve(park.objects().len());
        self.object_index.clear();
        self.object_index.reserve(park.objects().len());
        for object in park.objects() {
            self.object_index.insert(object.id, self.decks.len());
            self.decks.push(Deck {
                object: *object,
                roof: object_surfaces(park, object, rotation)[0],
            });
        }
        self.geometry_key = Some(key);
        self.stats.deck_rebuilds += 1;
    }
}

fn reserve_buffer<T>(values: &mut Vec<T>, desired: usize, growths: &mut u64) {
    if values.capacity() < desired {
        values.reserve(desired.saturating_sub(values.len()));
        *growths += 1;
    }
}

fn visible(park: &Park, object: SurfaceObject, mode: ViewMode) -> bool {
    // Equivalent to object_visible without looking the already-resolved ID up
    // again. Terrain changes need no roof-cache invalidation.
    mode == ViewMode::Underground
        || object.kind != ObjectKind::Track
        || park.tile(object.pos).is_none_or(|tile| {
            u16::from(object.height) + u16::from(object.clearance) > u16::from(tile.height)
        })
}

struct Writer<'a> {
    positions: &'a mut Vec<[f32; 3]>,
    colors: &'a mut Vec<Ink>,
}

impl Writer<'_> {
    fn body(&mut self, body: &BodyTemplate, base: Vec3) {
        // Keep body and feet depth biases in the same order as guest_mesh.
        let depth = base.z + 0.04;
        self.positions.extend(
            body.positions
                .iter()
                .map(|offset| [base.x + offset[0], base.y + offset[1], depth + offset[2]]),
        );
        self.colors.extend_from_slice(&body.colors);
    }

    fn shadow(&mut self, roof: Surface, center: Vec2, offsets: &[Vec2; 12], ink: Ink) {
        let bounds = roof.vertices.map(Vec3::truncate);
        let orientation = (bounds[1] - bounds[0])
            .perp_dot(bounds[2] - bounds[1])
            .signum();
        let mut source = [Vec2::ZERO; CLIPPED_POINTS];
        let mut output = [Vec2::ZERO; CLIPPED_POINTS];
        for (point, offset) in source.iter_mut().zip(offsets) {
            *point = center + *offset;
        }
        let mut count = offsets.len();
        for edge in 0..4 {
            if count == 0 {
                return;
            }
            let start = bounds[edge];
            let direction = bounds[(edge + 1) % 4] - start;
            let signed_distance = |point: Vec2| direction.perp_dot(point - start) * orientation;
            let mut previous = source[count - 1];
            let mut previous_distance = signed_distance(previous);
            let mut output_count = 0;
            for point in source.iter().take(count).copied() {
                let distance = signed_distance(point);
                if (distance >= 0.0) != (previous_distance >= 0.0) {
                    let fraction = previous_distance / (previous_distance - distance);
                    output[output_count] = previous.lerp(point, fraction);
                    output_count += 1;
                }
                if distance >= 0.0 {
                    output[output_count] = point;
                    output_count += 1;
                }
                previous = point;
                previous_distance = distance;
            }
            std::mem::swap(&mut source, &mut output);
            count = output_count;
        }
        for index in 1..count.saturating_sub(1) {
            for point in [source[0], source[index], source[index + 1]] {
                self.positions
                    .push(roof.point(point, DETAIL_BIAS * 3.0).to_array());
                self.colors.push(ink);
            }
        }
    }
}

fn body_template(appearance: usize, pose: usize) -> BodyTemplate {
    let mut mesh = Painter::default();
    let lost = pose == usize::from(WALK_TICKS) + 1;
    let stride = if pose < usize::from(WALK_TICKS) {
        ((pose as f32 / f32::from(WALK_TICKS)) * std::f32::consts::TAU).sin() * 1.8
    } else {
        0.0
    };
    for (side, step) in [(-1.8, stride), (1.8, -stride)] {
        mesh.line(
            Vec2::new(side, 7.5),
            Vec2::new(side + step, 0.8),
            2.4,
            0.0,
            rgb(43, 62, 68),
        );
    }
    let shirt = if lost {
        rgb(235, 112, 44)
    } else {
        match appearance {
            0 => rgb(213, 77, 82),
            1 => rgb(66, 126, 188),
            2 => rgb(150, 88, 174),
            3 => rgb(242, 184, 65),
            4 => rgb(53, 154, 140),
            _ => rgb(222, 119, 161),
        }
    };
    mesh.quad(
        [
            Vec2::new(-3.3, 7.0),
            Vec2::new(3.3, 7.0),
            Vec2::new(3.8, 17.0),
            Vec2::new(-3.8, 17.0),
        ],
        0.001,
        shirt,
    );
    let skin = if appearance.is_multiple_of(3) {
        rgb(174, 119, 84)
    } else {
        rgb(239, 192, 143)
    };
    for side in [-1.0, 1.0] {
        mesh.line(
            Vec2::new(side * 4.0, 15.3),
            Vec2::new(side * 4.3, 9.2 + stride * side),
            1.8,
            0.002,
            skin,
        );
    }
    mesh.ellipse(Vec2::new(0.0, 20.2), 3.5, 4.0, 0.003, rgb(67, 53, 42));
    mesh.ellipse(Vec2::new(0.3, 19.6), 3.1, 3.2, 0.004, skin);
    if lost {
        mesh.line(
            Vec2::new(0.0, 28.0),
            Vec2::new(0.0, 32.0),
            2.0,
            0.005,
            rgb(201, 68, 34),
        );
        mesh.ellipse(Vec2::new(0.0, 25.5), 1.0, 1.0, 0.005, rgb(201, 68, 34));
    }
    BodyTemplate {
        positions: mesh.positions,
        colors: mesh.colors,
    }
}

fn marker_index(phase: RideVisitPhase) -> usize {
    match phase {
        RideVisitPhase::Approaching => 0,
        RideVisitPhase::Queueing => 1,
        RideVisitPhase::Leaving => 2,
        RideVisitPhase::Boarding => 3,
        RideVisitPhase::Riding => 4,
        RideVisitPhase::Exiting => 5,
    }
}

fn marker_template(phase: RideVisitPhase) -> BodyTemplate {
    let mut mesh = Painter::default();
    paint_guest_visit_marker(&mut mesh, phase, Vec2::ZERO, 0.0);
    BodyTemplate {
        positions: mesh.positions,
        colors: mesh.colors,
    }
}

#[cfg(test)]
mod tests {
    use park_sim::{PathDirection, RideAccessStatus, Speed, TerrainProfile, diagnostics};
    use serde_json::Value;

    use super::*;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("guest positions are missing");
        };
        positions
    }

    fn equivalent(actual: &Mesh, expected: &Mesh) {
        assert_eq!(actual.indices(), expected.indices());
        for attribute in [
            Mesh::ATTRIBUTE_COLOR,
            Mesh::ATTRIBUTE_NORMAL,
            Mesh::ATTRIBUTE_UV_0,
        ] {
            assert_eq!(
                actual.attribute(attribute.id),
                expected.attribute(attribute.id)
            );
        }
        assert_eq!(positions(actual).len(), positions(expected).len());
        for (index, (a, b)) in positions(actual)
            .iter()
            .zip(positions(expected))
            .enumerate()
        {
            // Caching local coordinates changes floating-point addition order
            // by a few ulps, below 0.002 pixels even on the 256-tile fixture.
            // Physical depth remains identical because it is added separately.
            assert!(
                (a[0] - b[0]).abs() < 0.002 && (a[1] - b[1]).abs() < 0.002,
                "vertex {index}: {a:?} vs {b:?}",
            );
            assert_eq!(a[2], b[2], "depth at vertex {index}");
        }
    }

    fn fixture() -> Park {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(42).unwrap();
        park
    }

    fn admission_fixture() -> Park {
        let mut park = fixture();
        park.install_train_fixture().unwrap();
        let ride = park.ride().unwrap().id;
        park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, ride)
            .unwrap();
        for (y, height, slope) in [
            (12, 8, PathSlope::Flat),
            (13, 8, PathSlope::South),
            (14, 9, PathSlope::Flat),
            (15, 8, PathSlope::North),
        ] {
            park.place_queue_path(TilePos { x: 8, y }, height, slope, ride)
                .unwrap();
        }
        assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
        park
    }

    fn snapshot_when(mut park: Park, ready: impl Fn(&Park) -> bool) -> Park {
        for _ in 0..1200 {
            if ready(&park) {
                return park;
            }
            park.advance(Speed::Normal);
        }
        panic!("admission fixture did not reach its expected render state");
    }

    fn full_queue() -> Park {
        snapshot_when(admission_fixture(), |park| {
            let queue = park.ride_queue_stats(park.ride().unwrap().id);
            queue.queueing == 4 && queue.waiting == 4
        })
    }

    fn journey_fixture() -> Park {
        let mut park = admission_fixture();
        let ride = park.ride().unwrap().id;
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride)
            .unwrap();
        for pos in (8..=13)
            .map(|x| TilePos { x, y: 8 })
            .chain((9..=11).map(|y| TilePos { x: 13, y }))
        {
            park.place_sloped_path(pos, 8, PathSlope::Flat).unwrap();
        }
        assert_eq!(
            park.ride_exit_access(ride).status,
            park_sim::RideExitStatus::Ready
        );
        park
    }

    #[test]
    fn boarding_riding_and_exit_use_authoritative_points_without_duplicate_guest_bodies() {
        let mut park = journey_fixture();
        let mut snapshots: [Option<Park>; 3] = [None, None, None];
        for _ in 0..18000 {
            for guest in park.guests() {
                let Some(visit) = &guest.ride_visit else {
                    continue;
                };
                let Some(passenger) = &visit.passenger else {
                    continue;
                };
                let index = match visit.phase {
                    RideVisitPhase::Boarding
                        if passenger
                            .transfer
                            .as_ref()
                            .is_some_and(|transfer| transfer.progress == 30) =>
                    {
                        0
                    }
                    RideVisitPhase::Riding if park.train().unwrap().speed > 0 => 1,
                    RideVisitPhase::Exiting
                        if passenger
                            .transfer
                            .as_ref()
                            .is_some_and(|transfer| transfer.progress == 30) =>
                    {
                        2
                    }
                    _ => continue,
                };
                if snapshots[index].is_none() {
                    snapshots[index] = Some(park.clone());
                }
            }
            if snapshots.iter().all(Option::is_some) {
                break;
            }
            park.advance(Speed::Normal);
        }
        assert!(
            snapshots.iter().all(Option::is_some),
            "production fixture must board, carry and unload guests"
        );
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        for snapshot in snapshots.into_iter().flatten() {
            for passenger in snapshot.ride_guest_positions() {
                let guest = snapshot
                    .guests()
                    .iter()
                    .find(|guest| guest.id == passenger.guest)
                    .unwrap();
                assert_eq!(
                    snapshot.guest_position(guest.id).unwrap().point,
                    passenger.point
                );
                if let Some(transfer) = guest
                    .ride_visit
                    .as_ref()
                    .unwrap()
                    .passenger
                    .as_ref()
                    .unwrap()
                    .transfer
                    .as_ref()
                    && transfer.progress == 30
                {
                    let gate = snapshot.object(transfer.portal).unwrap();
                    assert_eq!(passenger.point.x, gate.pos.x as i32 * park_sim::PATH_UNITS);
                    assert_eq!(passenger.point.y, gate.pos.y as i32 * park_sim::PATH_UNITS);
                    assert_eq!(
                        passenger.point.height,
                        i32::from(gate.height) * park_sim::PATH_UNITS
                    );
                }
            }
            for rotation in 0..4 {
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    renderer.update(&mut mesh, &snapshot, rotation, mode);
                    equivalent(&mesh, &guest_mesh(&snapshot, rotation, mode));
                    assert_eq!(
                        renderer.stats().rendered_guests,
                        snapshot
                            .guests()
                            .iter()
                            .filter(|guest| !guest_is_riding(guest))
                            .count()
                    );
                    assert!(
                        positions(&mesh)
                            .iter()
                            .flatten()
                            .all(|coordinate| coordinate.is_finite())
                    );
                    let loaded = Park::from_save_bytes(&snapshot.to_save_bytes().unwrap()).unwrap();
                    let before = renderer.stats();
                    renderer.update(&mut mesh, &loaded, rotation, mode);
                    equivalent(&mesh, &guest_mesh(&loaded, rotation, mode));
                    assert_eq!(before.template_builds, renderer.stats().template_builds);
                    assert_eq!(before.deck_rebuilds, renderer.stats().deck_rebuilds);
                    assert_eq!(before.buffer_growths, renderer.stats().buffer_growths);
                }
            }
        }
    }

    #[test]
    fn visit_markers_have_distinct_shapes_colors_and_physical_depth() {
        let base = Vec3::new(123.0, -234.0, 56.0);
        let phases = [
            RideVisitPhase::Approaching,
            RideVisitPhase::Queueing,
            RideVisitPhase::Leaving,
            RideVisitPhase::Boarding,
            RideVisitPhase::Exiting,
        ];
        let inks = [
            rgb(33, 160, 187),
            rgb(218, 155, 36),
            rgb(100, 119, 141),
            rgb(224, 137, 48),
            rgb(49, 161, 112),
        ];
        let mut shapes = Vec::new();
        for (phase, ink) in phases.into_iter().zip(inks) {
            let template = marker_template(phase);
            assert!(!template.positions.is_empty());
            assert!(template.positions.len() <= 12);
            assert!(template.colors.iter().all(|color| *color == ink));
            assert!(template.positions.iter().all(|point| {
                Vec3::from_array(*point).is_finite()
                    && point[0].abs() <= 4.0
                    && (25.0..=33.0).contains(&point[1])
                    && point[2] == 0.005
            }));
            let mut positions = Vec::new();
            let mut colors = Vec::new();
            Writer {
                positions: &mut positions,
                colors: &mut colors,
            }
            .body(&template, base);
            let mut expected = Painter::default();
            paint_guest_visit_marker(&mut expected, phase, base.truncate(), base.z + 0.04);
            assert_eq!(colors, expected.colors);
            for (actual, expected) in positions.iter().zip(&expected.positions) {
                assert!(Vec3::from_array(*actual).distance(Vec3::from_array(*expected)) < 0.0001);
                assert_eq!(actual[2], (base.z + 0.04) + 0.005);
            }
            shapes.push(template.positions);
        }
        assert_ne!(shapes[0], shapes[1]);
        assert_ne!(shapes[1], shapes[2]);
        assert_ne!(shapes[3], shapes[4]);
    }

    #[test]
    fn admitted_visitors_follow_ramp_roofs_and_reuse_templates_in_all_views() {
        let approaching = snapshot_when(admission_fixture(), |park| {
            park.guests().iter().any(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == RideVisitPhase::Approaching)
            })
        });
        let moving = snapshot_when(approaching.clone(), |park| {
            park.guests().iter().any(|guest| {
                guest.progress == WALK_TICKS / 2
                    && guest
                        .ride_visit
                        .as_ref()
                        .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
                    && guest
                        .next
                        .and_then(|id| park.object(id))
                        .is_some_and(|object| object.path_slope != PathSlope::Flat)
            })
        });
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        for rotation in 0..4 {
            renderer.update(&mut mesh, &approaching, rotation, ViewMode::Surface);
            equivalent(
                &mesh,
                &guest_mesh(&approaching, rotation, ViewMode::Surface),
            );
            let initial = renderer.stats();
            let storage = buffer_storage(&mesh);
            let mut park = moving.clone();
            for _ in 0..=WALK_TICKS * 2 {
                for guest in park.guests().iter().filter(|guest| {
                    guest
                        .ride_visit
                        .as_ref()
                        .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
                }) {
                    let current = guest.current.and_then(|id| park.object(id)).unwrap();
                    let next = guest.next.and_then(|id| park.object(id));
                    let position = guest.position(Some(current), next);
                    let base = project_path_point(position.point, park.width(), rotation);
                    let support = if guest.progress > WALK_TICKS / 2 {
                        next.unwrap_or(current)
                    } else {
                        current
                    };
                    let roof = object_surfaces(&park, support, rotation)[0];
                    assert!((roof.hit_depth(base.truncate()).unwrap() - base.z).abs() < 0.0001);
                }
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    renderer.update(&mut mesh, &park, rotation, mode);
                    equivalent(&mesh, &guest_mesh(&park, rotation, mode));
                    assert_eq!(buffer_storage(&mesh), storage);
                }
                park.advance(Speed::Normal);
            }
            assert_eq!(renderer.stats().deck_rebuilds, initial.deck_rebuilds);
            assert_eq!(renderer.stats().template_builds, initial.template_builds);
            assert_eq!(renderer.stats().buffer_growths, initial.buffer_growths);
        }
    }

    #[test]
    fn queue_cancellation_same_tick_load_and_lost_owner_anchors_refresh_without_stale_roles() {
        let full = full_queue();
        let mut closed = full.clone();
        closed.set_train_open(false);
        assert_eq!(full.ticks(), closed.ticks());
        assert!(closed.guests().iter().any(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == RideVisitPhase::Leaving)
        }));
        let loaded = Park::from_save_bytes(&closed.to_save_bytes().unwrap()).unwrap();
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        renderer.update(&mut mesh, &full, 0, ViewMode::Surface);
        let before = mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap().clone();
        let stats = renderer.stats();
        renderer.update(&mut mesh, &loaded, 0, ViewMode::Surface);
        assert_ne!(mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap(), &before);
        equivalent(&mesh, &guest_mesh(&loaded, 0, ViewMode::Surface));
        assert_eq!(renderer.stats().deck_rebuilds, stats.deck_rebuilds);
        assert_eq!(renderer.stats().template_builds, stats.template_builds);

        let mut broken = full.clone();
        let occupied = broken
            .guests()
            .iter()
            .find_map(|guest| {
                guest
                    .current
                    .and_then(|id| broken.object(id))
                    .filter(|object| object.queue_owner.is_some())
                    .copied()
            })
            .unwrap();
        broken.remove_object(occupied.id).unwrap();
        assert!(
            broken
                .guests()
                .iter()
                .any(|guest| guest.state == GuestState::Lost && guest.ride_visit.is_some())
        );
        for guest in broken
            .guests()
            .iter()
            .filter(|guest| guest.state == GuestState::Lost)
        {
            assert_eq!(
                guest_visit_marker(guest),
                None,
                "lost warning takes priority over return intent"
            );
        }
        let mut repaired = broken.clone();
        let replacement = repaired
            .place_queue_path(
                occupied.pos,
                occupied.height,
                occupied.path_slope,
                occupied.queue_owner.unwrap(),
            )
            .unwrap();
        assert_ne!(replacement, occupied.id);
        for rotation in 0..4 {
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                for park in [&full, &loaded, &broken, &repaired] {
                    renderer.update(&mut mesh, park, rotation, mode);
                    equivalent(&mesh, &guest_mesh(park, rotation, mode));
                    assert!(
                        positions(&mesh)
                            .iter()
                            .all(|point| Vec3::from_array(*point).is_finite())
                    );
                }
            }
        }
    }

    #[test]
    fn queued_visit_markers_remain_occluded_by_foreground_terrain_in_each_rotation() {
        let full = full_queue();
        let guest = full
            .guests()
            .iter()
            .find(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
            })
            .unwrap();
        let object = *full.object(guest.current.unwrap()).unwrap();
        for rotation in 0..4 {
            let mut park = full.clone();
            let base = guest_anchor(&park, guest.id, rotation).unwrap();
            let (u, v) = rotate(object.pos, park.width(), rotation);
            let (distance, foreground) = (2..=8)
                .filter(|distance| u + distance < 32 && v + distance < 32)
                .map(|distance| (distance, unrotate(u + distance, v + distance, 32, rotation)))
                .find(|(_, pos)| park.objects().iter().all(|object| object.pos != *pos))
                .unwrap();
            park.set_terrain_height(foreground, object.height + (2 * distance + 2) as u8)
                .unwrap();
            let top = column_surfaces(&park, foreground, rotation)[0].unwrap();
            let mut marker = Painter::default();
            paint_guest_visit_marker(
                &mut marker,
                RideVisitPhase::Queueing,
                base.truncate(),
                base.z + 0.04,
            );
            for point in marker.positions {
                let point = Vec3::from_array(point);
                assert!(top.hit_depth(point.truncate()).unwrap() > point.z + 1.0);
            }
        }
    }

    fn with_guest(park: &Park, current: ObjectId, next: Option<ObjectId>, progress: u16) -> Park {
        let object = park.object(current).unwrap();
        let mut encoded: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        encoded["park"]["guests"] = serde_json::json!([{
            "id": 1,
            "current": current,
            "next": next,
            "destination": next,
            "anchor": object.pos,
            "height": object.height,
            "anchor_slope": object.path_slope,
            "progress": progress,
            "state": if next.is_some() { GuestState::Walking } else { GuestState::Waiting },
            "arrivals": 0,
            "route": [],
            "wait_ticks": 0,
            "ride_visit": null,
            "rides_completed": 0,
            "wallet_cents": park_sim::INITIAL_GUEST_WALLET_CENTS,
            "spent_cents": 0,
            "preferences": park_sim::GuestPreferences::for_guest(42, park_sim::GuestId(1)),
            "satisfaction": 70,
            "thought": park_sim::GuestThought::Exploring,
            "thought_tick": 0,
            "ratings_given": 0,
            "rating_total": 0,
            "last_rating": null,
            "needs": park_sim::GuestNeeds::for_guest(42, park_sim::GuestId(1)),
            "service_visit": null,
            "service_cooldown": 0,
            "service_spent_cents": 0,
            "services_used": 0,
            "ride_history": [],
            "ride_cooldown": 0
        }]);
        encoded["park"]["next_guest_id"] = 2.into();
        encoded["park"]["crowd_seed"] = 42.into();
        encoded["park"]["crowd_rng"] = 42.into();
        Park::from_save_bytes(&serde_json::to_vec(&encoded).unwrap()).unwrap()
    }

    #[test]
    fn slope_walks_follow_each_half_edge_and_cached_geometry_matches_every_phase() {
        for slope in [
            PathSlope::North,
            PathSlope::East,
            PathSlope::South,
            PathSlope::West,
        ] {
            let mut network = Park::new(32, 32);
            let pos = TilePos { x: 10, y: 10 };
            let direction = slope.direction().unwrap();
            let low = network
                .place_path(direction.opposite().neighbor(pos).unwrap(), 8)
                .unwrap();
            let ramp = network.place_sloped_path(pos, 8, slope).unwrap();
            let high = network
                .place_path(direction.neighbor(pos).unwrap(), 9)
                .unwrap();
            let mut renderer = GuestRenderer::default();
            let mut mesh = GuestRenderer::empty_mesh();
            for rotation in 0..4 {
                for (from, to, start, crossing, end) in [
                    (low, ramp, 8.0, 8.0, 8.5),
                    (ramp, high, 8.5, 9.0, 9.0),
                    (high, ramp, 9.0, 9.0, 8.5),
                    (ramp, low, 8.5, 8.0, 8.0),
                ] {
                    for progress in 0..WALK_TICKS {
                        let park = with_guest(&network, from, Some(to), progress);
                        let guest = &park.guests()[0];
                        let position = park.guest_position(guest.id).unwrap();
                        let (a, b, time) = if progress <= WALK_TICKS / 2 {
                            (start, crossing, progress)
                        } else {
                            (crossing, end, progress - WALK_TICKS / 2)
                        };
                        let expected = a + (b - a) * f32::from(time) / f32::from(WALK_TICKS / 2);
                        assert!(
                            (position.point.height as f32 / park_sim::PATH_UNITS as f32 - expected)
                                .abs()
                                < 0.0001
                        );
                        let base = guest_anchor(&park, guest.id, rotation).unwrap();
                        let support = if progress <= WALK_TICKS / 2 { from } else { to };
                        let roof =
                            object_surfaces(&park, park.object(support).unwrap(), rotation)[0];
                        assert!(
                            (roof.hit_depth(base.truncate()).unwrap() - base.z).abs() < 0.0001,
                            "feet left the piecewise physical ramp"
                        );
                        for mode in [ViewMode::Surface, ViewMode::Underground] {
                            renderer.update(&mut mesh, &park, rotation, mode);
                            equivalent(&mesh, &guest_mesh(&park, rotation, mode));
                        }
                    }
                }
                let waiting = with_guest(&network, ramp, None, 0);
                renderer.update(&mut mesh, &waiting, rotation, ViewMode::Surface);
                equivalent(&mesh, &guest_mesh(&waiting, rotation, ViewMode::Surface));
                assert_eq!(
                    waiting
                        .guest_position(waiting.guests()[0].id)
                        .unwrap()
                        .point
                        .height,
                    8 * 60 + 30
                );
            }
            assert_eq!(
                renderer.stats().deck_rebuilds,
                4,
                "walking phases must reuse cached decks"
            );
            assert_eq!(
                renderer.stats().template_builds,
                (APPEARANCES * POSES) as u64
            );
        }
    }

    #[test]
    fn same_tick_slope_loads_rebuild_decks_and_removed_ramps_keep_lost_center_height() {
        let pos = TilePos { x: 10, y: 10 };
        let mut first = Park::new(32, 32);
        let original = first.place_sloped_path(pos, 8, PathSlope::East).unwrap();
        let first = with_guest(&first, original, None, 0);
        let mut second = Park::new(32, 32);
        let replacement = second.place_sloped_path(pos, 8, PathSlope::North).unwrap();
        let second = with_guest(&second, replacement, None, 0);
        assert_eq!(original, replacement);
        assert_eq!(first.ticks(), second.ticks());
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        renderer.update(&mut mesh, &first, 0, ViewMode::Surface);
        let before = positions(&mesh).to_vec();
        renderer.update(&mut mesh, &second, 0, ViewMode::Surface);
        assert_eq!(renderer.stats().deck_rebuilds, 2);
        assert_ne!(
            positions(&mesh),
            before,
            "sloped shadow must be rebuilt after same-ID load"
        );
        equivalent(&mesh, &guest_mesh(&second, 0, ViewMode::Surface));
        let mut lost = second.clone();
        lost.remove_object(replacement).unwrap();
        assert_eq!(lost.guests()[0].state, GuestState::Lost);
        assert_eq!(
            lost.guest_position(lost.guests()[0].id)
                .unwrap()
                .point
                .height,
            8 * 60 + 30
        );
        for rotation in 0..4 {
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                renderer.update(&mut mesh, &lost, rotation, mode);
                equivalent(&mesh, &guest_mesh(&lost, rotation, mode));
            }
        }
    }

    #[test]
    fn cached_geometry_matches_reference_for_all_walk_phases_views_and_rotations() {
        let initial = fixture();
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        for rotation in 0..4 {
            let mut park = initial.clone();
            let mut states = [false; 2];
            for _ in 0..=usize::from(WALK_TICKS) + 2 {
                for guest in park.guests() {
                    states[usize::from(guest.state == GuestState::Walking)] = true;
                }
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    renderer.update(&mut mesh, &park, rotation, mode);
                    equivalent(&mesh, &guest_mesh(&park, rotation, mode));
                }
                park.advance(Speed::Normal);
            }
            assert!(states.into_iter().all(|seen| seen));
        }
        assert_eq!(
            renderer.stats().template_builds,
            (APPEARANCES * POSES) as u64
        );
        assert_eq!(renderer.stats().deck_rebuilds, 4);
    }

    #[test]
    fn removed_paths_lost_markers_and_restored_decks_match_reference() {
        let mut park = fixture();
        for _ in 0..17 {
            park.advance(Speed::Normal);
        }
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        renderer.update(&mut mesh, &park, 0, ViewMode::Surface);
        let current = park.guests()[0].current.unwrap();
        park.remove_object(current).unwrap();
        renderer.update(&mut mesh, &park, 0, ViewMode::Surface);
        equivalent(&mesh, &guest_mesh(&park, 0, ViewMode::Surface));
        assert_eq!(renderer.stats().deck_rebuilds, 2);
        let paths: Vec<_> = park
            .objects()
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .map(|object| object.id)
            .collect();
        for path in paths {
            park.remove_object(path).unwrap();
        }
        assert_eq!(park.crowd_stats().lost, 12);
        for rotation in 0..4 {
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                renderer.update(&mut mesh, &park, rotation, mode);
                equivalent(&mesh, &guest_mesh(&park, rotation, mode));
                assert_eq!(renderer.stats().rendered_guests, 12);
            }
        }
        park.install_crowd_fixture(42).unwrap();
        park.advance(Speed::Normal);
        renderer.update(&mut mesh, &park, 3, ViewMode::Underground);
        equivalent(&mesh, &guest_mesh(&park, 3, ViewMode::Underground));
    }

    #[test]
    fn same_tick_same_ids_loaded_geometry_and_reordered_vectors_do_not_reuse_stale_decks() {
        let mut original = Park::new(32, 32);
        original.install_crowd_fixture(42).unwrap();
        for _ in 0..17 {
            original.advance(Speed::Normal);
        }
        let mut encoded: Value =
            serde_json::from_slice(&original.to_save_bytes().unwrap()).unwrap();
        let objects = encoded["park"]["objects"].as_array_mut().unwrap();
        for object in objects.iter_mut() {
            let x = object["pos"]["x"].as_u64().unwrap();
            object["pos"]["x"] = (x + 1).into();
        }
        objects.reverse();
        let guests = encoded["park"]["guests"].as_array_mut().unwrap();
        for guest in guests.iter_mut() {
            let x = guest["anchor"]["x"].as_u64().unwrap();
            guest["anchor"]["x"] = (x + 1).into();
        }
        guests.reverse();
        let loaded = Park::from_save_bytes(&serde_json::to_vec(&encoded).unwrap()).unwrap();
        assert_eq!(original.ticks(), loaded.ticks());
        assert_eq!(original.objects().len(), loaded.objects().len());
        for object in original.objects() {
            assert!(loaded.object(object.id).is_some());
        }
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        renderer.update(&mut mesh, &original, 2, ViewMode::Surface);
        let before = positions(&mesh).to_vec();
        renderer.update(&mut mesh, &loaded, 2, ViewMode::Surface);
        equivalent(&mesh, &guest_mesh(&loaded, 2, ViewMode::Surface));
        assert_ne!(positions(&mesh), before);
        assert_eq!(renderer.stats().deck_rebuilds, 2);
        assert_eq!(
            renderer.stats().template_builds,
            (APPEARANCES * POSES) as u64
        );
    }

    #[test]
    fn warmed_buffers_and_templates_are_reused_while_guests_actually_move() {
        let mut park = fixture();
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        renderer.update(&mut mesh, &park, 0, ViewMode::Surface);
        let warmed = renderer.stats();
        let storage = buffer_storage(&mesh);
        let initial = positions(&mesh).to_vec();
        for _ in 0..65 {
            park.advance(Speed::Normal);
            renderer.update(&mut mesh, &park, 0, ViewMode::Surface);
            assert_eq!(buffer_storage(&mesh), storage);
        }
        assert_ne!(positions(&mesh), initial);
        assert_eq!(renderer.stats().updates, warmed.updates + 65);
        assert_eq!(renderer.stats().buffer_growths, warmed.buffer_growths);
        assert_eq!(renderer.stats().deck_rebuilds, warmed.deck_rebuilds);
        assert_eq!(renderer.stats().template_builds, warmed.template_builds);
        equivalent(&mesh, &guest_mesh(&park, 0, ViewMode::Surface));
    }

    #[test]
    fn queue_joins_refresh_cached_public_decks_without_admitting_any_guest() {
        let mut original = fixture();
        original.install_train_fixture().unwrap();
        let owner = original.ride().unwrap().id;
        for rotation in 0..4 {
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                let mut park = original.clone();
                let mut renderer = GuestRenderer::default();
                let mut mesh = GuestRenderer::empty_mesh();
                renderer.update(&mut mesh, &park, rotation, mode);
                let before = renderer.stats();
                let queue = park
                    .place_queue_path(TilePos { x: 8, y: 15 }, 8, PathSlope::Flat, owner)
                    .unwrap();
                renderer.update(&mut mesh, &park, rotation, mode);
                equivalent(&mesh, &guest_mesh(&park, rotation, mode));
                assert_eq!(renderer.stats().deck_rebuilds, before.deck_rebuilds + 1);
                assert_eq!(renderer.stats().template_builds, before.template_builds);
                let rebuilt = renderer.stats().deck_rebuilds;
                for _ in 0..65 {
                    park.advance(Speed::Normal);
                    renderer.update(&mut mesh, &park, rotation, mode);
                    assert!(park.guests().iter().all(|guest| {
                        guest.current != Some(queue)
                            && guest.next != Some(queue)
                            && guest.destination != Some(queue)
                    }));
                }
                assert_eq!(renderer.stats().deck_rebuilds, rebuilt);
                equivalent(&mesh, &guest_mesh(&park, rotation, mode));
                park.remove_object(queue).unwrap();
                renderer.update(&mut mesh, &park, rotation, mode);
                equivalent(&mesh, &guest_mesh(&park, rotation, mode));
                assert_eq!(renderer.stats().deck_rebuilds, rebuilt + 1);
            }
        }
    }

    fn buffer_storage(mesh: &Mesh) -> Vec<(usize, usize)> {
        let mut storage: Vec<_> = mesh
            .attributes()
            .map(|(_, values)| match values {
                VertexAttributeValues::Float32x2(values) => {
                    (values.as_ptr() as usize, values.capacity())
                }
                VertexAttributeValues::Float32x3(values) => {
                    (values.as_ptr() as usize, values.capacity())
                }
                VertexAttributeValues::Float32x4(values) => {
                    (values.as_ptr() as usize, values.capacity())
                }
                _ => panic!("unexpected guest attribute"),
            })
            .collect();
        let Some(Indices::U32(indices)) = mesh.indices() else {
            panic!("guest indices are missing");
        };
        storage.push((indices.as_ptr() as usize, indices.capacity()));
        storage
    }

    #[test]
    fn large_fixture_matches_reference_and_terrain_edits_keep_projected_path_decks() {
        let mut park = diagnostics::build_fixture(diagnostics::DiagnosticCase::SaveScale, 123)
            .unwrap()
            .park;
        for _ in 0..17 {
            park.advance(Speed::Normal);
        }
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        for rotation in 0..4 {
            renderer.update(&mut mesh, &park, rotation, ViewMode::Surface);
            equivalent(&mesh, &guest_mesh(&park, rotation, ViewMode::Surface));
        }
        let before = renderer.stats();
        park.set_terrain_height(TilePos { x: 0, y: 0 }, 1).unwrap();
        renderer.update(&mut mesh, &park, 3, ViewMode::Surface);
        equivalent(&mesh, &guest_mesh(&park, 3, ViewMode::Surface));
        assert_eq!(renderer.stats().deck_rebuilds, before.deck_rebuilds);
    }

    #[test]
    fn a_reused_renderer_clears_geometry_for_empty_and_non_square_parks() {
        let park = fixture();
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        renderer.update(&mut mesh, &park, 0, ViewMode::Surface);
        assert!(mesh.count_vertices() > 0);
        for empty in [Park::new(32, 32), Park::new(16, 8)] {
            renderer.update(&mut mesh, &empty, 0, ViewMode::Surface);
            equivalent(&mesh, &guest_mesh(&empty, 0, ViewMode::Surface));
            assert_eq!(renderer.stats().rendered_guests, 0);
        }
        renderer.update(&mut mesh, &park, 0, ViewMode::Surface);
        equivalent(&mesh, &guest_mesh(&park, 0, ViewMode::Surface));
    }
}
