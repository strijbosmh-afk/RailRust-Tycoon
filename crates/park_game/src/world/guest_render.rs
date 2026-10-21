//! Persistent CPU buffers and procedural guest templates. The original
//! `guest_mesh` remains the independently generated geometry oracle.

use std::collections::HashMap;

use bevy::mesh::VertexAttributeValues;
use park_sim::{ObjectId, WALK_TICKS};

use super::*;

const APPEARANCES: usize = 6;
const POSES: usize = WALK_TICKS as usize + 2;
// A convex twelve-point shadow clipped by four edges has at most sixteen
// vertices: 42 triangle vertices per supporting roof. A lost body has 126.
const MAX_GUEST_VERTICES: usize = 2 * 42 + 126;
const CLIPPED_POINTS: usize = 16;

/// Algorithm counters, not allocator instrumentation or GPU measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GuestRenderStats {
    pub updates: u64,
    pub deck_rebuilds: u64,
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
                    let current = guest.current.and_then(|id| self.deck(id));
                    if current.is_some_and(|deck| !visible(park, deck.object, mode)) {
                        continue;
                    }
                    let next = guest.next.and_then(|id| self.deck(id));
                    let position = guest.position(
                        current.as_ref().map(|deck| &deck.object),
                        next.as_ref().map(|deck| &deck.object),
                    );
                    let base = project_path_point(position.point, park.width(), rotation);
                    for deck in [current, next].into_iter().flatten() {
                        writer.shadow(deck.roof, base.truncate(), &self.shadow, self.shadow_color);
                    }
                    let pose = match guest.state {
                        GuestState::Walking => usize::from(guest.progress),
                        GuestState::Waiting => usize::from(WALK_TICKS),
                        GuestState::Lost => usize::from(WALK_TICKS) + 1,
                    };
                    // Guest progress is validated below WALK_TICKS by the
                    // simulation and save codec, including diagnostic inputs.
                    debug_assert!(
                        guest.state != GuestState::Walking || guest.progress < WALK_TICKS
                    );
                    let appearance = (guest.id.0 % APPEARANCES as u64) as usize;
                    writer.body(&self.bodies[appearance * POSES + pose], base);
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

#[cfg(test)]
mod tests {
    use park_sim::{Speed, TerrainProfile, diagnostics};
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
            "wait_ticks": 0
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
