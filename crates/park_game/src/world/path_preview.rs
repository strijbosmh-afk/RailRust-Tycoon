//! Level path tools use the same deck elevation for inverse projection,
//! occlusion, and their temporary ghost. The ghost never becomes a hit target.

use crate::path_tools::PathPreview;

use super::*;

fn deck_height(height: u8) -> f32 {
    f32::from(height) + PATH_TOP_OFFSET
}

fn plane_coordinates(point: Vec2, size: u32, height: u8) -> Vec2 {
    let column = point.x / HALF_WIDTH;
    let sum = size as f32 - 1.0 - (point.y - deck_height(height) * HEIGHT_STEP) / HALF_HEIGHT;
    Vec2::new((sum + column) * 0.5, (sum - column) * 0.5)
}

/// Pick a tile on the proposed path's roof plane, independent of terrain and
/// objects. `height` is its simulation base height; the roof adds the same
/// 0.22 step as a built path. Map/tile upper edges are exclusive. Snap only
/// floating-point noise at exact grid lines (under 0.007 world units at 256).
pub fn pick_path_plane(point: Vec2, park: &Park, rotation: u8, height: u8) -> Option<TilePos> {
    if !point.is_finite() || park.width() != park.height() {
        return None;
    }
    let coordinates = plane_coordinates(point, park.width(), height) + Vec2::splat(0.5);
    let cell = |coordinate: f32| {
        let nearest = coordinate.round();
        let coordinate = if (coordinate - nearest).abs() <= 0.0001 {
            nearest
        } else {
            coordinate
        };
        coordinate.floor()
    };
    let u = cell(coordinates.x);
    let v = cell(coordinates.y);
    let size = park.width() as f32;
    if u < 0.0 || v < 0.0 || u >= size || v >= size {
        return None;
    }
    Some(unrotate(u as u32, v as u32, park.width(), rotation))
}

/// Reject a level target behind a nearer visible terrain/object face. Surface
/// and underground use the normal shared hit geometry, including actual
/// sampled rail slopes. Equal-height decks and their shallow detail bias pass
/// through so placement validation can report occupied/clearance conflicts.
pub fn path_plane_occluded(
    point: Vec2,
    park: &Park,
    rotation: u8,
    height: u8,
    mode: ViewMode,
) -> bool {
    if pick_path_plane(point, park, rotation, height).is_none() {
        return true;
    }
    let coordinates = plane_coordinates(point, park.width(), height);
    let candidate_depth = BASE_DEPTH + coordinates.x + coordinates.y;
    let Some(hit) = pick_surface(point, park, rotation, mode) else {
        return false;
    };
    selectable_surfaces(park, hit, rotation, mode)
        .into_iter()
        .filter_map(|surface| surface.hit_depth(point))
        .any(|depth| depth > candidate_depth + SELECTION_BIAS * 2.0)
}

fn preview_surface(preview: &PathPreview, park: &Park, rotation: u8) -> Surface {
    if preview.demolish
        && let Some(object) = park.objects().iter().find(|object| {
            object.kind == ObjectKind::Path
                && object.pos == preview.pos
                && object.height == preview.height
        })
    {
        return object_surfaces(park, object, rotation)[0];
    }

    // New paths have X-oriented planks. Their walkable deck extends sideways
    // when a same-height north/south neighbor exists, exactly as object_bounds.
    let mut min_y = -0.34;
    let mut max_y = 0.34;
    for object in park.objects().iter().filter(|object| {
        object.kind == ObjectKind::Path
            && object.height == preview.height
            && object.pos.x == preview.pos.x
    }) {
        if object.pos.y.checked_add(1) == Some(preview.pos.y) {
            min_y = -0.5;
        } else if preview.pos.y.checked_add(1) == Some(object.pos.y) {
            max_y = 0.5;
        }
    }
    let (u, v) = rotate(preview.pos, park.width(), rotation);
    let height = deck_height(preview.height);
    let vertices = [
        Vec2::new(-0.5, min_y),
        Vec2::new(-0.5, max_y),
        Vec2::new(0.5, max_y),
        Vec2::new(0.5, min_y),
    ]
    .map(|point| {
        let point = match rotation % 4 {
            0 => point,
            1 => Vec2::new(-point.y, point.x),
            2 => -point,
            _ => Vec2::new(point.y, -point.x),
        };
        project(u as f32 + point.x, v as f32 + point.y, height, park.width())
    });
    Surface {
        vertices,
        kind: SurfaceKind::Top,
    }
}

/// Temporary construction feedback. Use a separate alpha-blended material:
/// fill is translucent, outlines are strong, and an X distinguishes demolition
/// or invalid placement without depending on color alone. Depth stays physical,
/// so foreground terrain and objects can hide the ghost normally.
pub fn path_preview_mesh(preview: Option<PathPreview>, park: &Park, rotation: u8) -> Mesh {
    let mut painter = Painter::default();
    let Some(preview) = preview else {
        return painter.finish();
    };
    if park.width() != park.height() || park.tile(preview.pos).is_none() {
        return painter.finish();
    }
    let surface = preview_surface(&preview, park, rotation);
    let mut ink = if preview.demolish {
        rgb(235, 120, 51)
    } else if preview.valid {
        rgb(47, 183, 114)
    } else {
        rgb(225, 68, 63)
    };
    ink[3] = 0.22;
    let mut fill = surface;
    for vertex in &mut fill.vertices {
        vertex.z += SELECTION_BIAS;
    }
    painter.surface(fill, ink);
    ink[3] = 0.95;
    painter.surface_outline(surface, 1.5, SELECTION_BIAS * 2.0, ink);
    if preview.demolish || !preview.valid {
        let points = surface.vertices.map(Vec3::truncate);
        for (start, end) in [(points[0], points[2]), (points[1], points[3])] {
            painter.surface_line(surface, start, end, 1.2, SELECTION_BIAS * 3.0, ink);
        }
    }
    painter.finish()
}

#[cfg(test)]
mod tests {
    use bevy::mesh::VertexAttributeValues;

    use super::*;

    #[test]
    fn level_plane_round_trips_tiles_and_edges_at_all_heights_and_rotations() {
        for size in [1, 32, 256] {
            let park = Park::new(size, size);
            for rotation in 0..4 {
                for height in [0, 8, 30] {
                    for pos in [
                        TilePos { x: 0, y: 0 },
                        TilePos { x: size - 1, y: 0 },
                        TilePos { x: 0, y: size - 1 },
                        TilePos {
                            x: size - 1,
                            y: size - 1,
                        },
                        TilePos {
                            x: size / 2,
                            y: size / 2,
                        },
                    ] {
                        let (u, v) = rotate(pos, size, rotation);
                        for (du, dv) in [(0.0, 0.0), (-0.49, -0.49), (0.49, 0.49), (-0.49, 0.49)] {
                            let point =
                                project(u as f32 + du, v as f32 + dv, deck_height(height), size);
                            assert_eq!(
                                pick_path_plane(point.truncate(), &park, rotation, height),
                                Some(pos)
                            );
                        }
                    }
                    for (u, v, expected) in [
                        (-0.5, 0.0, Some(unrotate(0, 0, size, rotation))),
                        (size as f32 - 0.5, 0.0, None),
                        (0.0, -0.501, None),
                        (-0.501, 0.0, None),
                    ] {
                        let point = project(u, v, deck_height(height), size).truncate();
                        assert_eq!(pick_path_plane(point, &park, rotation, height), expected);
                    }
                }
            }
        }
        let park = Park::new(32, 32);
        for point in [Vec2::splat(f32::NAN), Vec2::splat(f32::INFINITY)] {
            assert_eq!(pick_path_plane(point, &park, 0, 8), None);
            assert!(path_plane_occluded(point, &park, 0, 8, ViewMode::Surface));
        }
        assert_eq!(pick_path_plane(Vec2::ZERO, &Park::new(32, 16), 0, 8), None);
    }

    #[test]
    fn foreground_cliffs_and_stacked_paths_occlude_but_matching_decks_do_not() {
        for rotation in 0..4 {
            let mut park = Park::new(32, 32);
            let target = unrotate(10, 10, 32, rotation);
            let foreground = unrotate(11, 11, 32, rotation);
            let point = project(10.0, 10.0, deck_height(8), 32).truncate();
            assert!(!path_plane_occluded(
                point,
                &park,
                rotation,
                8,
                ViewMode::Surface
            ));
            park.place_object(target, ObjectKind::Path, Axis::X, 8, 2)
                .unwrap();
            assert!(!path_plane_occluded(
                point,
                &park,
                rotation,
                8,
                ViewMode::Surface
            ));
            park.set_terrain_height(foreground, 12).unwrap();
            assert!(path_plane_occluded(
                point,
                &park,
                rotation,
                8,
                ViewMode::Surface
            ));
            assert!(!path_plane_occluded(
                point,
                &park,
                rotation,
                8,
                ViewMode::Underground
            ));
            park.place_object(foreground, ObjectKind::Path, Axis::Y, 12, 2)
                .unwrap();
            let upper_point = project(11.0, 11.0, deck_height(12), 32).truncate();
            assert!(path_plane_occluded(
                upper_point,
                &park,
                rotation,
                8,
                ViewMode::Underground
            ));
        }
    }

    #[test]
    fn preview_matches_built_deck_bounds_depth_and_all_rotations() {
        let mut park = Park::new(32, 32);
        let pos = TilePos { x: 10, y: 10 };
        for y in [9, 11] {
            park.place_object(TilePos { x: 10, y }, ObjectKind::Path, Axis::Y, 8, 2)
                .unwrap();
        }
        let preview = PathPreview {
            pos,
            height: 8,
            valid: true,
            demolish: false,
        };
        let id = park
            .place_object(pos, ObjectKind::Path, Axis::X, 8, 2)
            .unwrap();
        for rotation in 0..4 {
            let expected = object_surfaces(&park, park.object(id).unwrap(), rotation)[0];
            let ghost = preview_surface(&preview, &park, rotation);
            let mut actual = ghost.vertices.map(|point| point.to_array());
            let mut expected_vertices = expected.vertices.map(|point| point.to_array());
            actual.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
            expected_vertices.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
            assert_eq!(actual, expected_vertices);
            for preview in [
                preview,
                PathPreview {
                    valid: false,
                    ..preview
                },
                PathPreview {
                    demolish: true,
                    ..preview
                },
            ] {
                let mesh = path_preview_mesh(Some(preview), &park, rotation);
                let Some(VertexAttributeValues::Float32x3(positions)) =
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    panic!("missing positions");
                };
                for &[x, y, z] in positions {
                    assert!(Vec3::new(x, y, z).is_finite());
                    assert!(expected.hit_depth(Vec2::new(x, y)).is_some());
                    assert!(z >= expected.depth_at(Vec2::new(x, y)));
                    assert!(z - expected.depth_at(Vec2::new(x, y)) <= SELECTION_BIAS * 3.0 + 0.001);
                }
                let Some(VertexAttributeValues::Float32x4(colors)) =
                    mesh.attribute(Mesh::ATTRIBUTE_COLOR)
                else {
                    panic!("missing colors");
                };
                assert!(colors.iter().all(|color| color[3] > 0.0 && color[3] < 1.0));
            }
        }
        assert_eq!(path_preview_mesh(None, &park, 0).count_vertices(), 0);
        assert_eq!(
            path_preview_mesh(
                Some(PathPreview {
                    pos: TilePos { x: 32, y: 0 },
                    ..preview
                }),
                &park,
                0
            )
            .count_vertices(),
            0
        );
    }
}
