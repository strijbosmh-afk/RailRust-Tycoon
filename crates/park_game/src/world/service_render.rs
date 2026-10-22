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

/// Every box remains inside the one-tile footprint and three-step clearance.
/// Local +X faces the PUBLIC serving edge, independent of camera rotation.
pub(super) fn faces(
    object: &SurfaceObject,
    kind: ServiceKind,
    facing: PathDirection,
    size: u32,
    rotation: u8,
) -> Vec<Face> {
    let mut result = Vec::with_capacity(80);
    let cream = [rgb(242, 231, 191), rgb(205, 187, 146), rgb(171, 149, 107)];
    let dark = [rgb(76, 101, 89), rgb(53, 77, 66), rgb(40, 62, 53)];
    let accent = match kind {
        ServiceKind::Food => [rgb(226, 139, 73), rgb(189, 92, 49), rgb(141, 68, 41)],
        ServiceKind::Drink => [rgb(86, 180, 210), rgb(47, 131, 169), rgb(33, 93, 135)],
        ServiceKind::Toilet => [rgb(129, 184, 161), rgb(78, 133, 112), rgb(51, 101, 83)],
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
    // Neutral entrance apron reaches the serving edge without extending out of
    // the reserved tile. Public path joins remain separate navigation geometry.
    cuboid(
        Vec3::new(-0.46, -0.46, 0.0),
        Vec3::new(0.5, 0.46, PATH_TOP_OFFSET),
        cream,
    );
    if kind == ServiceKind::Toilet {
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
            }
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
    let faces = faces(
        &object,
        preview.kind,
        preview.facing,
        park.width(),
        rotation,
    );
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
        for kind in [ServiceKind::Food, ServiceKind::Drink, ServiceKind::Toilet] {
            for facing in PathDirection::ALL {
                let mut park = Park::new(12, 12);
                let id = park
                    .place_service(TilePos { x: 5, y: 5 }, 8, facing, kind)
                    .unwrap();
                let object = park
                    .object(park.service(id).unwrap().object.unwrap())
                    .unwrap();
                for rotation in 0..4 {
                    let faces = faces(object, kind, facing, park.width(), rotation);
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
    fn service_art_is_distinct_with_finite_in_envelope_vertices_at_height_limits() {
        for height in [0, 29] {
            let mut kind_meshes = Vec::new();
            for kind in [ServiceKind::Food, ServiceKind::Drink, ServiceKind::Toilet] {
                let mut park = Park::new(12, 12);
                let id = park
                    .place_service(TilePos { x: 5, y: 5 }, height, PathDirection::South, kind)
                    .unwrap();
                let object = park
                    .object(park.service(id).unwrap().object.unwrap())
                    .unwrap();
                let mut painter = Painter::default();
                for face in faces(object, kind, PathDirection::South, 12, 0) {
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
                    assert!((f32::from(height) - 0.001..f32::from(height) + 3.0).contains(&h));
                }
                kind_meshes.push(positions(&mesh).to_vec());
            }
            assert_ne!(kind_meshes[0], kind_meshes[1]);
            assert_ne!(kind_meshes[1], kind_meshes[2]);
        }
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
                &object,
                ServiceKind::Toilet,
                PathDirection::West,
                40,
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
