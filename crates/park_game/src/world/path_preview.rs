//! Path tools use the same level/sloped deck for inverse projection,
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

fn candidate(pos: TilePos, height: u8, slope: PathSlope) -> SurfaceObject {
    SurfaceObject {
        id: park_sim::ObjectId(0),
        pos,
        kind: ObjectKind::Path,
        axis: match slope {
            PathSlope::North | PathSlope::South => Axis::Y,
            _ => Axis::X,
        },
        height,
        clearance: if slope == PathSlope::Flat { 2 } else { 3 },
        path_slope: slope,
    }
}

/// Center of the actual proposed deck, including its half-step slope rise.
pub fn path_build_center(
    pos: TilePos,
    park: &Park,
    rotation: u8,
    height: u8,
    slope: PathSlope,
) -> Option<Vec2> {
    if park.width() != park.height() || park.tile(pos).is_none() {
        return None;
    }
    let object = candidate(pos, height, slope);
    Some(object_point(
        &object,
        0.0,
        0.0,
        object_top(&object),
        park.width(),
        rotation,
    ))
}

/// Pick the actual ramp roof, resolving overlapping candidate projections by
/// physical depth. At most nine tiles can overlap the midpoint-height plane.
/// Level paths retain the established full-tile plane target behavior.
pub fn pick_sloped_path_plane(
    point: Vec2,
    park: &Park,
    rotation: u8,
    height: u8,
    slope: PathSlope,
) -> Option<TilePos> {
    if slope == PathSlope::Flat {
        return pick_path_plane(point, park, rotation, height);
    }
    if !point.is_finite() || park.width() != park.height() {
        return None;
    }
    let size = park.width();
    let around = plane_coordinates(point - Vec2::Y * (HEIGHT_STEP * 0.5), size, height);
    if around.min_element() < -1.5 || around.max_element() > size as f32 + 0.5 {
        return None;
    }
    let cell = (around + Vec2::splat(0.5)).floor().as_ivec2();
    let mut closest: Option<(f32, (u32, u32), TilePos)> = None;
    for v in (cell.y - 1).max(0)..=(cell.y + 1).min(size as i32 - 1) {
        for u in (cell.x - 1).max(0)..=(cell.x + 1).min(size as i32 - 1) {
            let pos = unrotate(u as u32, v as u32, size, rotation);
            let roof = object_surfaces(park, &candidate(pos, height, slope), rotation)[0];
            if let Some(depth) = roof.hit_depth(point) {
                let tie = ((u + v) as u32, u as u32);
                if closest.is_none_or(|(old, old_tie, _)| {
                    depth > old + 0.0001 || ((depth - old).abs() <= 0.0001 && tie > old_tie)
                }) {
                    closest = Some((depth, tie, pos));
                }
            }
        }
    }
    closest.map(|(_, _, pos)| pos)
}

/// Test occlusion against the candidate's true inclined plane, never the
/// low-height floor or its reserved clearance envelope.
pub fn sloped_path_plane_occluded(
    point: Vec2,
    park: &Park,
    rotation: u8,
    height: u8,
    slope: PathSlope,
    mode: ViewMode,
) -> bool {
    if slope == PathSlope::Flat {
        return path_plane_occluded(point, park, rotation, height, mode);
    }
    let Some(pos) = pick_sloped_path_plane(point, park, rotation, height, slope) else {
        return true;
    };
    let roof = object_surfaces(park, &candidate(pos, height, slope), rotation)[0];
    let depth = roof.depth_at(point);
    pick_surface(point, park, rotation, mode).is_some_and(|hit| {
        selectable_surfaces(park, hit, rotation, mode)
            .into_iter()
            .filter_map(|surface| surface.hit_depth(point))
            .any(|hit_depth| hit_depth > depth + SELECTION_BIAS * 2.0)
    })
}

fn preview_surface(preview: &PathPreview, park: &Park, rotation: u8) -> Surface {
    if preview.demolish
        && let Some(object) = park.objects().iter().find(|object| {
            object.kind == ObjectKind::Path
                && object.pos == preview.pos
                && object.height == preview.height
                && object.path_slope == preview.slope
        })
    {
        return object_surfaces(park, object, rotation)[0];
    }

    object_surfaces(
        park,
        &candidate(preview.pos, preview.height, preview.slope),
        rotation,
    )[0]
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
    if preview.slope != PathSlope::Flat && preview.valid && !preview.demolish {
        let object = candidate(preview.pos, preview.height, preview.slope);
        let direction = slope_direction(preview.slope);
        let side = Vec2::new(-direction.y, direction.x);
        let (u, v) = rotate(preview.pos, park.width(), rotation);
        let point = |offset: Vec2| {
            let rotated = rotated_offset(offset, rotation);
            project(
                u as f32 + rotated.x,
                v as f32 + rotated.y,
                object_base_at(&object, offset) + PATH_TOP_OFFSET,
                park.width(),
            )
            .truncate()
        };
        let tip = direction * 0.27;
        for (from, to) in [
            (-direction * 0.24, tip),
            (tip - direction * 0.17 + side * 0.12, tip),
            (tip - direction * 0.17 - side * 0.12, tip),
        ] {
            painter.surface_line(
                surface,
                point(from),
                point(to),
                1.5,
                SELECTION_BIAS * 3.0,
                ink,
            );
        }
    }
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

    const SLOPES: [PathSlope; 4] = [
        PathSlope::North,
        PathSlope::East,
        PathSlope::South,
        PathSlope::West,
    ];

    #[test]
    fn sloped_roofs_previews_picking_and_selection_share_physical_planes() {
        let pos = TilePos { x: 10, y: 10 };
        for slope in SLOPES {
            for height in [0, 8, 29] {
                let mut park = Park::new(32, 32);
                let id = park.place_sloped_path(pos, height, slope).unwrap();
                for rotation in 0..4 {
                    let object = park.object(id).unwrap();
                    let roof = object_surfaces(&park, object, rotation)[0];
                    let ghost = preview_surface(
                        &PathPreview {
                            pos,
                            height,
                            slope,
                            valid: true,
                            demolish: false,
                        },
                        &park,
                        rotation,
                    );
                    assert_eq!(roof.vertices, ghost.vertices);
                    let center = path_build_center(pos, &park, rotation, height, slope).unwrap();
                    assert_eq!(
                        Some(center),
                        surface_center(&park, SurfaceId::Object(id), rotation)
                    );
                    assert_eq!(
                        pick_sloped_path_plane(center, &park, rotation, height, slope),
                        Some(pos)
                    );
                    for mode in [ViewMode::Surface, ViewMode::Underground] {
                        assert_eq!(
                            pick_surface(center, &park, rotation, mode),
                            Some(SurfaceId::Object(id))
                        );
                        assert!(!sloped_path_plane_occluded(
                            center, &park, rotation, height, slope, mode
                        ));
                    }

                    // Sample the whole ramp, not an envelope at its low/high height.
                    let high = slope.direction().unwrap();
                    let (dx, dy) = high.offset();
                    let (u, v) = rotate(pos, park.width(), rotation);
                    for step in [-30, -18, 0, 18, 30] {
                        let offset = Vec2::new((dx * step) as f32, (dy * step) as f32) / 60.0;
                        let rotated = rotated_offset(offset, rotation);
                        let height = object.path_height_at(dx * step, dy * step).unwrap() as f32
                            / 60.0
                            + PATH_TOP_OFFSET;
                        let expected = project(
                            u as f32 + rotated.x,
                            v as f32 + rotated.y,
                            height,
                            park.width(),
                        );
                        assert!(
                            (roof.hit_depth(expected.truncate()).unwrap() - expected.z).abs()
                                < 0.0001
                        );
                    }
                    let preview = PathPreview {
                        pos,
                        height,
                        slope,
                        valid: true,
                        demolish: false,
                    };
                    let mesh = path_preview_mesh(Some(preview), &park, rotation);
                    let Some(VertexAttributeValues::Float32x3(points)) =
                        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                    else {
                        panic!("positions");
                    };
                    for &point in points {
                        let point = Vec3::from_array(point);
                        let physical = roof
                            .hit_depth(point.truncate())
                            .expect("preview escaped ramp");
                        let bias = point.z - physical;
                        assert!(
                            (SELECTION_BIAS - 0.0001..=SELECTION_BIAS * 3.0 + 0.0001)
                                .contains(&bias)
                        );
                    }
                    let Some(VertexAttributeValues::Float32x4(colors)) =
                        mesh.attribute(Mesh::ATTRIBUTE_COLOR)
                    else {
                        panic!("colors");
                    };
                    assert!(colors.iter().all(|color| color[3] > 0.0 && color[3] < 1.0));
                    assert!(points.len() > 30, "direction arrow must accompany outline");

                    let outline = selection_mesh(
                        None,
                        Some(SurfaceId::Object(id)),
                        &park,
                        rotation,
                        ViewMode::Surface,
                    );
                    let Some(VertexAttributeValues::Float32x3(points)) =
                        outline.attribute(Mesh::ATTRIBUTE_POSITION)
                    else {
                        panic!("positions");
                    };
                    let faces = object_surfaces(&park, object, rotation);
                    assert!(points.iter().all(|&point| {
                        let point = Vec3::from_array(point);
                        faces.into_iter().any(|face| {
                            face.hit_depth(point.truncate()).is_some_and(|depth| {
                                (point.z - depth - SELECTION_BIAS).abs() < 0.0001
                            })
                        })
                    }));
                }
            }
        }
    }

    #[test]
    fn slope_endpoints_meet_flat_decks_and_flat_previews_match_connected_junctions() {
        let center = TilePos { x: 10, y: 10 };
        for slope in SLOPES {
            let mut park = Park::new(32, 32);
            let high = slope.direction().unwrap();
            let low_pos = high.opposite().neighbor(center).unwrap();
            let high_pos = high.neighbor(center).unwrap();
            let ramp = park.place_sloped_path(center, 8, slope).unwrap();
            let low = park.place_path(low_pos, 8).unwrap();
            let high_id = park.place_path(high_pos, 9).unwrap();
            let (dx, dy) = high.offset();
            for rotation in 0..4 {
                let (u, v) = rotate(center, 32, rotation);
                for (sign, height, neighbor) in [(-1.0, 8.0, low), (1.0, 9.0, high_id)] {
                    let offset =
                        rotated_offset(Vec2::new(dx as f32, dy as f32) * sign * 0.5, rotation);
                    let point = project(
                        u as f32 + offset.x,
                        v as f32 + offset.y,
                        height + PATH_TOP_OFFSET,
                        32,
                    );
                    for object in [ramp, neighbor] {
                        let roof =
                            object_surfaces(&park, park.object(object).unwrap(), rotation)[0];
                        assert!(
                            (roof.hit_depth(point.truncate()).unwrap() - point.z).abs() < 0.0001
                        );
                    }
                }
                for (pos, height, id) in [(low_pos, 8, low), (high_pos, 9, high_id)] {
                    let preview = PathPreview {
                        pos,
                        height,
                        slope: PathSlope::Flat,
                        valid: false,
                        demolish: false,
                    };
                    assert_eq!(
                        preview_surface(&preview, &park, rotation).vertices,
                        object_surfaces(&park, park.object(id).unwrap(), rotation)[0].vertices
                    );
                }
            }
            let bounds = object_bounds(&park, park.object(ramp).unwrap());
            assert_eq!(
                bounds,
                match slope.axis() {
                    Axis::X => (-0.5, 0.5, -0.34, 0.34),
                    Axis::Y => (-0.34, 0.34, -0.5, 0.5),
                }
            );
        }
    }

    #[test]
    fn slope_decorations_follow_roof_and_foreground_terrain_occludes_proposed_plane() {
        for rotation in 0..4 {
            for slope in SLOPES {
                let mut park = Park::new(32, 32);
                let pos = unrotate(10, 10, 32, rotation);
                let id = park.place_sloped_path(pos, 8, slope).unwrap();
                let mut painter = Painter::default();
                paint_objects(&mut painter, &park, rotation, ViewMode::Surface);
                let faces = object_surfaces(&park, park.object(id).unwrap(), rotation);
                for point in painter.positions.iter().skip(18) {
                    let point = Vec3::from_array(*point);
                    let depth = faces[0]
                        .hit_depth(point.truncate())
                        .expect("decoration outside ramp");
                    assert!((0.0..=DETAIL_BIAS * 2.0 + 0.0001).contains(&(point.z - depth)));
                }
                let center = path_build_center(pos, &park, rotation, 8, slope).unwrap();
                park.set_terrain_height(unrotate(11, 11, 32, rotation), 12)
                    .unwrap();
                assert!(sloped_path_plane_occluded(
                    center,
                    &park,
                    rotation,
                    8,
                    slope,
                    ViewMode::Surface
                ));
                assert!(!sloped_path_plane_occluded(
                    center,
                    &park,
                    rotation,
                    8,
                    slope,
                    ViewMode::Underground
                ));
            }
        }
        let park = Park::new(32, 32);
        for point in [
            Vec2::splat(f32::NAN),
            Vec2::splat(f32::INFINITY),
            Vec2::splat(100_000.0),
        ] {
            assert_eq!(
                pick_sloped_path_plane(point, &park, 0, 8, PathSlope::East),
                None
            );
        }
        assert_eq!(
            pick_sloped_path_plane(Vec2::ZERO, &Park::new(32, 16), 0, 8, PathSlope::East),
            None
        );
    }

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
            slope: PathSlope::Flat,
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
