//! World-space extents for controlled large-map framing. These helpers do not
//! change normal camera settings or imply support for rectangular playfields.

use super::*;

/// A conservative projected rectangle for every rotation and visibility mode.
///
/// Include the map's soil/shadow, original trees and entrance, and dynamic
/// people/cars. A stranded guest can remain above the highest surviving path,
/// so guest anchors also contribute to the maximum elevation. This is a
/// framing bound, not an exact mesh AABB or a selectable map rectangle.
pub fn scene_bounds(park: &Park) -> Option<Rect> {
    if park.width() != park.height() {
        return None;
    }
    let half = Vec2::new(
        park.width() as f32 * HALF_WIDTH,
        park.height() as f32 * HALF_HEIGHT,
    );
    let highest = park
        .guests()
        .iter()
        .map(|guest| guest.height)
        .fold(park.max_surface_height(), u8::max);
    // The arch extends 36 units sideways and 67 upwards; the tallest tree
    // reaches 73.125 upwards. Guest markers and train cars fit below 80.
    // The diorama shadow reaches 45 below the map; soil reaches only 27.
    Some(Rect::from_corners(
        -half - Vec2::new(36.0, 45.0),
        half + Vec2::new(36.0, f32::from(highest) * HEIGHT_STEP + 80.0),
    ))
}

/// Center of the projected active path network, including its deck heights.
/// The result is raw world space like surface_center and train_focus. Empty
/// networks and unsupported rectangular maps have no meaningful path focus.
pub fn path_focus(park: &Park, rotation: u8) -> Option<Vec2> {
    if park.width() != park.height() {
        return None;
    }
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    let mut found = false;
    for object in park
        .objects()
        .iter()
        .filter(|object| object.kind == ObjectKind::Path)
    {
        let center = object_point(object, 0.0, 0.0, object_top(object), park.width(), rotation);
        min = min.min(center);
        max = max.max(center);
        found = true;
    }
    found.then_some((min + max) * 0.5)
}

#[cfg(test)]
mod tests {
    use bevy::mesh::VertexAttributeValues;
    use park_sim::{MAX_TERRAIN_HEIGHT, Speed, diagnostics};
    use serde_json::Value;

    use super::*;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("mesh has no position attribute");
        };
        positions
    }

    fn assert_contained(mesh: &Mesh, bounds: Rect) {
        assert!(matches!(mesh.indices(), Some(Indices::U32(_))));
        for &[x, y, z] in positions(mesh) {
            assert!(x.is_finite() && y.is_finite() && z.is_finite());
            assert!(
                bounds.contains(Vec2::new(x, y)),
                "{x},{y} outside {bounds:?}"
            );
            // Bevy 0.17.3's default_2d has near=-1000 and far=1000.
            // Keep the 256-map physical depth within that camera volume.
            assert!((-1000.0..1000.0).contains(&z), "depth {z}");
        }
    }

    #[test]
    fn largest_map_edges_round_trip_and_elevated_paths_pick_in_every_rotation() {
        let mut park = Park::new(256, 256);
        let corners = [
            TilePos { x: 0, y: 0 },
            TilePos { x: 255, y: 0 },
            TilePos { x: 0, y: 255 },
            TilePos { x: 255, y: 255 },
        ];
        for corner in corners {
            park.set_terrain_height(corner, MAX_TERRAIN_HEIGHT).unwrap();
        }
        let path = park
            .place_object(TilePos { x: 255, y: 128 }, ObjectKind::Path, Axis::Y, 8, 2)
            .unwrap();
        let bounds = scene_bounds(&park).unwrap();
        for rotation in 0..4 {
            for pos in corners.into_iter().chain([
                TilePos { x: 0, y: 128 },
                TilePos { x: 128, y: 0 },
                TilePos { x: 128, y: 255 },
                TilePos { x: 127, y: 128 },
            ]) {
                let point = elevated_tile_center(&park, pos, rotation).unwrap();
                assert!(bounds.contains(point));
                assert_eq!(
                    pick_surface(point, &park, rotation, ViewMode::Surface),
                    Some(SurfaceId::Terrain(pos)),
                );
            }
            let point = surface_center(&park, SurfaceId::Object(path), rotation).unwrap();
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                assert_eq!(
                    pick_surface(point, &park, rotation, mode),
                    Some(SurfaceId::Object(path)),
                );
            }
            assert!(
                pick_surface(
                    Vec2::new(256.0 * HALF_WIDTH + 0.1, 0.0),
                    &park,
                    rotation,
                    ViewMode::Surface,
                )
                .is_none()
            );
        }
    }

    #[test]
    fn large_scene_bounds_contain_static_dynamic_and_selection_meshes_in_all_views() {
        let mut park = diagnostics::build_fixture(diagnostics::DiagnosticCase::SaveScale, 42)
            .unwrap()
            .park;
        for _ in 0..17 {
            park.advance(Speed::Normal);
        }
        for corner in [
            TilePos { x: 0, y: 0 },
            TilePos { x: 255, y: 0 },
            TilePos { x: 0, y: 255 },
            TilePos { x: 255, y: 255 },
        ] {
            park.set_terrain_height(corner, MAX_TERRAIN_HEIGHT).unwrap();
        }
        let bounds = scene_bounds(&park).unwrap();
        let mut renderer = GuestRenderer::default();
        let mut guests = GuestRenderer::empty_mesh();
        for rotation in 0..4 {
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                // Deliberately build one static mesh at a time: this checks
                // full-map geometry without retaining eight large buffers.
                assert_contained(&build_scene(&park, rotation, true, mode), bounds);
                renderer.update(&mut guests, &park, rotation, mode);
                assert_contained(&guests, bounds);
                assert_contained(&train_mesh(&park, rotation, mode), bounds);
                let selected = Some(SurfaceId::Object(park.guests()[0].current.unwrap()));
                assert_contained(
                    &selection_mesh(selected, selected, &park, rotation, mode),
                    bounds,
                );
            }
        }
    }

    #[test]
    fn path_focus_tracks_actual_projected_paths_and_rejects_empty_or_rectangular_maps() {
        let mut park = diagnostics::build_fixture(diagnostics::DiagnosticCase::SaveScale, 42)
            .unwrap()
            .park;
        for rotation in 0..4 {
            let first = tile_center(TilePos { x: 80, y: 80 }, 256, rotation);
            let last = tile_center(TilePos { x: 111, y: 111 }, 256, rotation);
            let expected = (first + last) * 0.5 + Vec2::Y * (8.22 * HEIGHT_STEP);
            assert!(path_focus(&park, rotation).unwrap().distance(expected) < 0.001);
            assert!(scene_bounds(&park).unwrap().contains(expected));
        }
        let paths: Vec<_> = park
            .objects()
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .map(|object| object.id)
            .collect();
        for path in paths {
            park.remove_object(path).unwrap();
        }
        assert_eq!(path_focus(&park, 0), None);
        assert_eq!(path_focus(&Park::new(16, 8), 0), None);
        assert_eq!(scene_bounds(&Park::new(16, 8)), None);
    }

    #[test]
    fn bounds_include_high_stranded_guests_after_their_support_is_removed() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(42).unwrap();
        let paths: Vec<_> = park
            .objects()
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .map(|object| object.id)
            .collect();
        for path in paths {
            park.remove_object(path).unwrap();
        }
        let mut value: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        for guest in value["park"]["guests"].as_array_mut().unwrap() {
            guest["height"] = 31.into();
            guest["anchor"]["x"] = 0.into();
            guest["anchor"]["y"] = 0.into();
        }
        let park = Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(park.max_surface_height() < 31);
        let bounds = scene_bounds(&park).unwrap();
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        for rotation in 0..4 {
            renderer.update(&mut mesh, &park, rotation, ViewMode::Surface);
            assert_contained(&mesh, bounds);
        }
    }

    #[test]
    fn retained_guest_renderer_moves_between_normal_and_large_map_spaces() {
        let baseline = diagnostics::build_fixture(diagnostics::DiagnosticCase::Baseline, 42)
            .unwrap()
            .park;
        let large = diagnostics::build_fixture(diagnostics::DiagnosticCase::SaveScale, 42)
            .unwrap()
            .park;
        let mut renderer = GuestRenderer::default();
        let mut mesh = GuestRenderer::empty_mesh();
        for park in [&baseline, &large, &baseline] {
            for rotation in 0..4 {
                renderer.update(&mut mesh, park, rotation, ViewMode::Underground);
                assert_contained(&mesh, scene_bounds(park).unwrap());
                let reference = guest_mesh(park, rotation, ViewMode::Underground);
                assert_eq!(mesh.indices(), reference.indices());
                assert_eq!(
                    mesh.attribute(Mesh::ATTRIBUTE_COLOR),
                    reference.attribute(Mesh::ATTRIBUTE_COLOR)
                );
                for (actual, expected) in positions(&mesh).iter().zip(positions(&reference)) {
                    assert!((actual[0] - expected[0]).abs() < 0.002);
                    assert!((actual[1] - expected[1]).abs() < 0.002);
                    assert_eq!(actual[2], expected[2]);
                }
                assert_eq!(renderer.stats().rendered_guests, park.guests().len());
            }
        }
    }
}
