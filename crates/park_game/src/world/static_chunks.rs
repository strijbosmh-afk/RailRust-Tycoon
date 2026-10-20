//! Deterministic, conservative partitions of the existing static scene.
//!
//! A chunk owns complete triangles and their original vertex attributes. Its
//! actual projected bounds, rather than its nominal bucket, govern visibility.
//! The full scene is temporary input and is not retained alongside the chunks.

use std::collections::BTreeMap;

use bevy::mesh::VertexAttributeValues;

use super::*;

const BUCKET_WIDTH: f32 = 1024.0;
const BUCKET_HEIGHT: f32 = 512.0;
const SINGLE_MESH_MAX_SIZE: u32 = 32;

pub struct StaticChunkGeometry {
    pub mesh: Mesh,
    /// Inclusive projected XY bounds of all vertices used by this chunk.
    /// Triangles can cross nominal bucket boundaries; culling must use these.
    pub bounds: Rect,
}

/// Partition the authoritative scene without changing its geometry or depth.
///
/// Normal maps retain their original single mesh. Large maps assign triangles
/// to fixed projected buckets by centroid. A triangle larger than a bucket
/// goes into a separate overflow chunk, so the full-map diorama shadow does
/// not inflate the bounds of otherwise local terrain. Return order is stable:
/// overflow first when present, then occupied buckets ordered by (x, y).
///
/// Like `build_scene`, this requires a square park.
pub fn build_static_chunks(
    park: &Park,
    rotation: u8,
    grid: bool,
    mode: ViewMode,
) -> Vec<StaticChunkGeometry> {
    partition(
        build_scene(park, rotation, grid, mode),
        park.width() > SINGLE_MESH_MAX_SIZE,
    )
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Bucket {
    Overflow,
    Cell(i32, i32),
}

fn positions(mesh: &Mesh) -> &[[f32; 3]] {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => panic!("static scene requires Float32x3 positions"),
    }
}

fn point(position: [f32; 3]) -> Vec2 {
    Vec2::new(position[0], position[1])
}

fn bounds(points: impl Iterator<Item = Vec2>) -> Rect {
    let (min, max) = points.fold(
        (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
        |(min, max), point| (min.min(point), max.max(point)),
    );
    Rect { min, max }
}

fn bucket(triangle: [[f32; 3]; 3]) -> Bucket {
    let points = triangle.map(point);
    let extent = bounds(points.into_iter()).size();
    if extent.x > BUCKET_WIDTH || extent.y > BUCKET_HEIGHT {
        return Bucket::Overflow;
    }
    let center = (points[0] + points[1] + points[2]) / 3.0;
    Bucket::Cell(
        (center.x / BUCKET_WIDTH).floor() as i32,
        (center.y / BUCKET_HEIGHT).floor() as i32,
    )
}

fn partition(mut source: Mesh, split: bool) -> Vec<StaticChunkGeometry> {
    assert_eq!(source.primitive_topology(), PrimitiveTopology::TriangleList);
    let index_count = source.indices().map_or(0, Indices::len);
    assert_eq!(index_count % 3, 0, "static scene has incomplete triangles");
    if index_count == 0 {
        return Vec::new();
    }
    if !split {
        let bounds = bounds(positions(&source).iter().copied().map(point));
        return vec![StaticChunkGeometry {
            mesh: source,
            bounds,
        }];
    }

    // Stage only source indices, not a second copy of every vertex attribute.
    // Each source triangle appears in exactly one bucket, in its original order.
    let source_indices = source.remove_indices().unwrap();
    let vertices = positions(&source);
    let mut buckets: BTreeMap<Bucket, Vec<u32>> = BTreeMap::new();
    let mut indices = source_indices.iter();
    while let Some(a) = indices.next() {
        let b = indices.next().unwrap();
        let c = indices.next().unwrap();
        buckets
            .entry(bucket([vertices[a], vertices[b], vertices[c]]))
            .or_default()
            .extend([a, b, c].map(|index| u32::try_from(index).unwrap()));
    }
    drop(indices);
    drop(source_indices);

    // Reuse one dense remapping table between chunks. Shared source vertices
    // remain shared within each chunk; only vertices used across chunks repeat.
    // Reset touched entries only, avoiding a full-scene scan for every bucket.
    let mut remap = vec![u32::MAX; vertices.len()];
    let mut chunks = Vec::with_capacity(buckets.len());
    for global_indices in buckets.into_values() {
        let mut source_vertices = Vec::new();
        let mut local_indices = Vec::with_capacity(global_indices.len());
        for global in global_indices {
            let global = global as usize;
            if remap[global] == u32::MAX {
                remap[global] = u32::try_from(source_vertices.len()).unwrap();
                source_vertices.push(global);
            }
            local_indices.push(remap[global]);
        }
        let bounds = bounds(source_vertices.iter().map(|&index| point(vertices[index])));
        let mut mesh = Mesh::new(source.primitive_topology(), source.asset_usage);
        for (attribute, values) in source.attributes() {
            mesh.insert_attribute(*attribute, select_attribute(values, &source_vertices));
        }
        mesh.insert_indices(Indices::U32(local_indices));
        for global in source_vertices {
            remap[global] = u32::MAX;
        }
        chunks.push(StaticChunkGeometry { mesh, bounds });
    }
    chunks
}

/// Retain every attribute supported by Bevy, including future additions to
/// Painter's current position/normal/UV/color schema. No attribute is rebuilt
/// from position, rounded, or silently discarded during partitioning.
fn select_attribute(values: &VertexAttributeValues, vertices: &[usize]) -> VertexAttributeValues {
    macro_rules! select {
        ($($variant:ident),+ $(,)?) => {
            match values {
                $(VertexAttributeValues::$variant(values) => {
                    VertexAttributeValues::$variant(
                        vertices.iter().map(|&index| values[index]).collect()
                    )
                }),+
            }
        };
    }
    select!(
        Float32, Sint32, Uint32, Float32x2, Sint32x2, Uint32x2, Float32x3, Sint32x3, Uint32x3,
        Float32x4, Sint32x4, Uint32x4, Sint16x2, Snorm16x2, Uint16x2, Unorm16x2, Sint16x4,
        Snorm16x4, Uint16x4, Unorm16x4, Sint8x2, Snorm8x2, Uint8x2, Unorm8x2, Sint8x4, Snorm8x4,
        Uint8x4, Unorm8x4,
    )
}

#[cfg(test)]
mod tests {
    use bevy::{mesh::MeshVertexAttribute, render::render_resource::VertexFormat};
    use park_sim::{MAX_TERRAIN_HEIGHT, diagnostics};

    use super::*;

    fn signatures<'a>(meshes: impl IntoIterator<Item = &'a Mesh>) -> Vec<[u32; 36]> {
        let mut result = Vec::new();
        for mesh in meshes {
            assert_eq!(mesh.attributes().count(), 4);
            let positions = positions(mesh);
            let Some(VertexAttributeValues::Float32x3(normals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                panic!("missing normals");
            };
            let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
            else {
                panic!("missing UVs");
            };
            let Some(VertexAttributeValues::Float32x4(colors)) =
                mesh.attribute(Mesh::ATTRIBUTE_COLOR)
            else {
                panic!("missing colors");
            };
            let mut indices = mesh.indices().unwrap().iter();
            while let Some(a) = indices.next() {
                let mut signature = [0; 36];
                for (corner, index) in [a, indices.next().unwrap(), indices.next().unwrap()]
                    .into_iter()
                    .enumerate()
                {
                    let vertex = &mut signature[corner * 12..(corner + 1) * 12];
                    vertex[0..3].copy_from_slice(&positions[index].map(f32::to_bits));
                    vertex[3..6].copy_from_slice(&normals[index].map(f32::to_bits));
                    vertex[6..8].copy_from_slice(&uvs[index].map(f32::to_bits));
                    vertex[8..12].copy_from_slice(&colors[index].map(f32::to_bits));
                }
                result.push(signature);
            }
        }
        result.sort_unstable();
        result
    }

    fn assert_bounds(chunk: &StaticChunkGeometry) {
        assert!(chunk.bounds.min.is_finite() && chunk.bounds.max.is_finite());
        assert!(matches!(chunk.mesh.indices(), Some(Indices::U32(_))));
        let actual = bounds(positions(&chunk.mesh).iter().copied().map(point));
        assert_eq!(chunk.bounds, actual);
        for &position in positions(&chunk.mesh) {
            assert!(position.into_iter().all(f32::is_finite));
            assert!(chunk.bounds.contains(point(position)));
        }
    }

    fn intersects(a: Rect, b: Rect) -> bool {
        a.min.x <= b.max.x && b.min.x <= a.max.x && a.min.y <= b.max.y && b.min.y <= a.max.y
    }

    #[test]
    fn normal_maps_keep_the_original_mesh_and_exact_attributes() {
        for size in [1, 32] {
            let park = Park::new(size, size);
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                let original = build_scene(&park, 3, true, mode);
                let chunks = build_static_chunks(&park, 3, true, mode);
                assert_eq!(chunks.len(), 1);
                let actual = &chunks[0].mesh;
                assert_eq!(actual.indices(), original.indices());
                assert_eq!(
                    actual.attributes().collect::<Vec<_>>(),
                    original.attributes().collect::<Vec<_>>()
                );
                assert_bounds(&chunks[0]);
            }
        }
    }

    #[test]
    fn all_triangle_attributes_survive_every_rotation_view_and_grid_setting() {
        let mut park = Park::new(64, 64);
        for pos in [
            TilePos { x: 0, y: 0 },
            TilePos { x: 63, y: 0 },
            TilePos { x: 0, y: 63 },
            TilePos { x: 63, y: 63 },
        ] {
            park.set_terrain_height(pos, MAX_TERRAIN_HEIGHT).unwrap();
        }
        for (x, kind, axis, height) in [
            (0, ObjectKind::Path, Axis::X, 8),
            (1, ObjectKind::Path, Axis::Y, 8),
            (63, ObjectKind::Track, Axis::Y, 24),
        ] {
            park.place_object(TilePos { x, y: 31 }, kind, axis, height, 2)
                .unwrap();
        }
        for rotation in 0..4 {
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                for grid in [false, true] {
                    let original = build_scene(&park, rotation, grid, mode);
                    let chunks = build_static_chunks(&park, rotation, grid, mode);
                    assert!(chunks.len() > 1);
                    for chunk in &chunks {
                        assert_bounds(chunk);
                    }
                    assert_eq!(
                        signatures([&original]),
                        signatures(chunks.iter().map(|chunk| &chunk.mesh)),
                        "rotation {rotation}, mode {mode:?}, grid {grid}"
                    );
                }
            }
        }
    }

    #[test]
    fn partition_is_stable_preserves_shared_indices_and_custom_attributes() {
        const CUSTOM: MeshVertexAttribute =
            MeshVertexAttribute::new("custom_data", 100, VertexFormat::Uint32x2);
        let source = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [-8.0, -8.0, 2.0],
                [-2.0, -8.0, 3.0],
                [-2.0, -2.0, 4.0],
                [-8.0, -2.0, 5.0],
                [2048.0, 512.0, 6.0],
                [2048.0, 512.0, 7.0],
                [2048.0, 512.0, 8.0],
            ],
        )
        .with_inserted_attribute(
            CUSTOM,
            vec![
                [10_u32, 0],
                [11, 1],
                [12, 2],
                [13, 3],
                [14, 4],
                [15, 5],
                [16, 6],
            ],
        )
        .with_inserted_indices(Indices::U16(vec![2, 0, 1, 2, 3, 0, 4, 5, 6]));
        let first = partition(source.clone(), true);
        let second = partition(source, true);
        assert_eq!(first.len(), 2);
        assert_eq!(
            first[0].mesh.indices(),
            Some(&Indices::U32(vec![0, 1, 2, 0, 3, 1]))
        );
        assert_eq!(
            first[0].mesh.attribute(CUSTOM),
            Some(&VertexAttributeValues::Uint32x2(vec![
                [12, 2],
                [10, 0],
                [11, 1],
                [13, 3]
            ]))
        );
        // Degenerate triangles and exact bucket edges remain geometry; the
        // zero-area bound is visible when a viewport merely touches its point.
        assert_eq!(first[1].bounds.min, Vec2::new(2048.0, 512.0));
        assert_eq!(first[1].bounds.min, first[1].bounds.max);
        assert!(intersects(
            first[1].bounds,
            Rect::from_corners(Vec2::ZERO, Vec2::new(2048.0, 512.0))
        ));
        for (first, second) in first.iter().zip(&second) {
            assert_bounds(first);
            assert_eq!(first.bounds, second.bounds);
            assert_eq!(first.mesh.indices(), second.mesh.indices());
            assert_eq!(
                first.mesh.attributes().collect::<Vec<_>>(),
                second.mesh.attributes().collect::<Vec<_>>()
            );
        }
        assert_eq!(bucket([[-0.5, -0.5, 0.0]; 3]), Bucket::Cell(-1, -1));
        assert_eq!(bucket([[0.0, 0.0, 0.0]; 3]), Bucket::Cell(0, 0));
    }

    #[test]
    fn oversized_triangles_have_an_independent_overflow_bound() {
        let mut painter = Painter::default();
        painter.triangle_3d(
            [
                [-4096.0, -2048.0, -10.0],
                [4096.0, -2048.0, -10.0],
                [0.0, 2048.0, -10.0],
            ]
            .map(Vec3::from_array),
            [0.1, 0.2, 0.3, 1.0],
        );
        painter.triangle_3d(
            [[1.0, 1.0, 20.0], [20.0, 1.0, 20.0], [1.0, 20.0, 20.0]].map(Vec3::from_array),
            [0.4, 0.5, 0.6, 1.0],
        );
        let source = painter.finish();
        let expected = signatures([&source]);
        let chunks = partition(source, true);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].bounds.min, Vec2::new(-4096.0, -2048.0));
        assert_eq!(chunks[1].bounds.max, Vec2::splat(20.0));
        assert_eq!(expected, signatures(chunks.iter().map(|chunk| &chunk.mesh)));
    }

    #[test]
    fn focused_large_map_view_excludes_chunks_without_losing_any_candidate_triangle() {
        let park = diagnostics::build_fixture(diagnostics::DiagnosticCase::SaveScale, 42)
            .unwrap()
            .park;
        let rotation = 3;
        let mode = ViewMode::Surface;
        let viewport = Rect::from_center_size(
            path_focus(&park, rotation).unwrap(),
            Vec2::new(1200.0, 700.0),
        );
        let original = build_scene(&park, rotation, true, mode);
        let chunks = build_static_chunks(&park, rotation, true, mode);
        let visible: Vec<_> = chunks
            .iter()
            .filter(|chunk| intersects(chunk.bounds, viewport))
            .collect();
        assert!(!visible.is_empty() && visible.len() < chunks.len() / 2);
        let selected_vertices: usize = visible
            .iter()
            .map(|chunk| positions(&chunk.mesh).len())
            .sum();
        assert!(selected_vertices < positions(&original).len() / 2);
        for chunk in &chunks {
            assert_bounds(chunk);
        }

        // Compare exactly the candidate triangles, including any with only a
        // single entering vertex or a crossing edge. AABB overlap deliberately
        // overestimates visibility, so it proves conservative coverage.
        let candidates = |mesh: &Mesh| {
            let positions = positions(mesh);
            let mut painter = Painter::default();
            let mut indices = mesh.indices().unwrap().iter();
            let Some(VertexAttributeValues::Float32x4(colors)) =
                mesh.attribute(Mesh::ATTRIBUTE_COLOR)
            else {
                panic!("missing colors");
            };
            while let Some(a) = indices.next() {
                let indices = [a, indices.next().unwrap(), indices.next().unwrap()];
                let triangle = indices.map(|index| positions[index]);
                if intersects(bounds(triangle.map(point).into_iter()), viewport) {
                    // Painter writes a constant color per triangle. Preserve
                    // this test's full oracle schema and reject any assumption
                    // change rather than comparing counts alone.
                    assert!(indices.iter().all(|&index| colors[index] == colors[a]));
                    painter.triangle_3d(triangle.map(Vec3::from_array), colors[a]);
                }
            }
            painter.finish()
        };
        let expected = candidates(&original);
        let actual: Vec<_> = visible
            .iter()
            .map(|chunk| candidates(&chunk.mesh))
            .collect();
        assert_eq!(signatures([&expected]), signatures(actual.iter()),);
    }

    #[test]
    fn empty_geometry_produces_no_chunks() {
        for split in [false, true] {
            assert!(partition(Painter::default().finish(), split).is_empty());
        }
    }

    #[test]
    #[should_panic(expected = "isometric scene expects a square park")]
    fn rectangular_parks_keep_the_full_scene_rejection_contract() {
        build_static_chunks(&Park::new(64, 32), 0, true, ViewMode::Surface);
    }
}
