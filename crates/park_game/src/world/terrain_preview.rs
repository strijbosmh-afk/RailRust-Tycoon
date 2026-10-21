//! Terrain ghosts use the current and proposed physical column surfaces.
//! The caller supplies only a visible Surface-view target; ghosts are not hits.

use crate::terrain_tools::TerrainPreview;
use park_sim::MAX_TERRAIN_HEIGHT;

use super::*;

fn marker(painter: &mut Painter, surface: Surface, preview: TerrainPreview, ink: Ink) {
    let center = surface
        .vertices
        .map(Vec3::truncate)
        .into_iter()
        .sum::<Vec2>()
        / 4.0;
    let bias = SELECTION_BIAS * 3.0;
    // The screen-facing symbol remains legible through every park rotation.
    // Painting on the face clips its strokes and interpolates physical depth.
    if !preview.valid {
        for direction in [Vec2::new(4.5, 4.5), Vec2::new(4.5, -4.5)] {
            painter.surface_line(
                surface,
                center - direction,
                center + direction,
                1.8,
                bias,
                ink,
            );
        }
    } else {
        painter.surface_line(
            surface,
            center - Vec2::X * 6.0,
            center + Vec2::X * 6.0,
            1.8,
            bias,
            ink,
        );
        if preview.raising {
            painter.surface_line(
                surface,
                center - Vec2::Y * 6.0,
                center + Vec2::Y * 6.0,
                1.8,
                bias,
                ink,
            );
        }
    }
}

/// Render a temporary one-step terrain edit without changing simulation state.
/// Current exposed cliffs also carry a marker, so a cliff-only pick has feedback.
/// The proposed outline keeps its true elevation: a lowered roof can be buried
/// by current terrain while the current footprint and minus remain visible.
/// Use the shared alpha-blended ghost material and suppress this mesh in
/// Underground view. Depth bias is only the small normal selection offset;
/// nearer terrain, paths, trains, and scenery still occlude the preview.
pub fn terrain_preview_mesh(preview: Option<TerrainPreview>, park: &Park, rotation: u8) -> Mesh {
    let mut painter = Painter::default();
    let Some(preview) = preview else {
        return painter.finish();
    };
    let Some(tile) = park.tile(preview.pos) else {
        return painter.finish();
    };
    let expected_target = if preview.raising {
        preview.height.saturating_add(1).min(MAX_TERRAIN_HEIGHT)
    } else {
        preview.height.saturating_sub(1)
    };
    if park.width() != park.height()
        || preview.height != tile.height
        || preview.height > MAX_TERRAIN_HEIGHT
        || preview.target_height != expected_target
    {
        return painter.finish();
    }

    let surfaces = column_surfaces(park, preview.pos, rotation);
    let top = surfaces[0].expect("in-bounds terrain has a top surface");
    let mut ink = if !preview.valid {
        rgb(225, 68, 63)
    } else if preview.raising {
        rgb(47, 183, 114)
    } else {
        rgb(61, 136, 215)
    };
    ink[3] = 0.16;
    let mut fill = top;
    for vertex in &mut fill.vertices {
        vertex.z += SELECTION_BIAS;
    }
    painter.surface(fill, ink);

    for surface in surfaces
        .into_iter()
        .flatten()
        .filter(|surface| surface.has_area())
    {
        ink[3] = if surface.kind == SurfaceKind::Top {
            0.95
        } else {
            0.65
        };
        painter.surface_outline(surface, 1.5, SELECTION_BIAS * 2.0, ink);
        marker(&mut painter, surface, preview, ink);
    }

    if preview.target_height != preview.height {
        let mut proposed = top;
        let delta = f32::from(preview.target_height) - f32::from(preview.height);
        for vertex in &mut proposed.vertices {
            vertex.y += delta * HEIGHT_STEP;
        }
        ink[3] = 0.85;
        painter.surface_outline(proposed, 1.5, SELECTION_BIAS * 2.0, ink);
    }
    painter.finish()
}

#[cfg(test)]
mod tests {
    use bevy::mesh::VertexAttributeValues;

    use super::*;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("missing positions");
        };
        positions
    }

    fn colors(mesh: &Mesh) -> &[[f32; 4]] {
        let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!("missing colors");
        };
        colors
    }

    fn preview(pos: TilePos, height: u8, raising: bool, valid: bool) -> TerrainPreview {
        TerrainPreview {
            pos,
            height,
            target_height: if raising {
                height.saturating_add(1).min(MAX_TERRAIN_HEIGHT)
            } else {
                height.saturating_sub(1)
            },
            raising,
            valid,
        }
    }

    #[test]
    fn ghosts_match_actual_current_and_proposed_surfaces_at_limits_and_all_rotations() {
        let pos = TilePos { x: 10, y: 10 };
        for height in [0, 1, 8, 31, 32] {
            let mut park = Park::new(32, 32);
            park.set_terrain_height(pos, height).unwrap();
            let before = park.clone();
            for rotation in 0..4 {
                for raising in [false, true] {
                    for valid in [false, true] {
                        let preview = preview(pos, height, raising, valid);
                        let mesh = terrain_preview_mesh(Some(preview), &park, rotation);
                        let current = column_surfaces(&park, pos, rotation);
                        let mut proposed_park = park.clone();
                        proposed_park
                            .set_terrain_height(pos, preview.target_height)
                            .unwrap();
                        let proposed = column_surfaces(&proposed_park, pos, rotation)[0].unwrap();
                        let [a, b, c, d] = current[0].unwrap().vertices;
                        let expected_fill = [a, b, c, a, c, d].map(|mut vertex| {
                            vertex.z += SELECTION_BIAS;
                            vertex.to_array()
                        });
                        assert_eq!(&positions(&mesh)[..6], &expected_fill);
                        for &point in positions(&mesh) {
                            let point = Vec3::from_array(point);
                            assert!(point.is_finite());
                            assert!(
                                current
                                    .into_iter()
                                    .flatten()
                                    .chain([proposed])
                                    .any(|surface| {
                                        surface.hit_depth(point.truncate()).is_some_and(|depth| {
                                            let bias = point.z - depth;
                                            (SELECTION_BIAS - 0.0001
                                                ..=SELECTION_BIAS * 3.0 + 0.0001)
                                                .contains(&bias)
                                        })
                                    }),
                                "preview vertex escaped its physical supporting face: {point:?}"
                            );
                        }
                        if preview.target_height != height {
                            for corner in proposed.vertices {
                                assert!(
                                    positions(&mesh).iter().any(|&point| {
                                        let expected = corner + Vec3::Z * (SELECTION_BIAS * 2.0);
                                        Vec3::from_array(point).distance(expected) < 0.001
                                    }),
                                    "missing proposed-height corner"
                                );
                            }
                        }
                        assert!(
                            colors(&mesh)
                                .iter()
                                .all(|color| color[3] > 0.0 && color[3] < 1.0)
                        );
                        assert_eq!(park, before, "rendering mutated the authoritative park");
                    }
                }
            }
        }
    }

    #[test]
    fn direction_symbols_differ_and_lowering_retains_a_current_surface_marker() {
        let mut park = Park::new(32, 32);
        let pos = TilePos { x: 10, y: 10 };
        park.set_terrain_height(pos, 8).unwrap();
        for rotation in 0..4 {
            let raise = terrain_preview_mesh(Some(preview(pos, 8, true, true)), &park, rotation);
            let lower = terrain_preview_mesh(Some(preview(pos, 8, false, true)), &park, rotation);
            let invalid = terrain_preview_mesh(Some(preview(pos, 8, true, false)), &park, rotation);
            assert!(
                raise.count_vertices() > lower.count_vertices(),
                "plus has an extra stroke"
            );
            assert_ne!(colors(&raise)[0], colors(&lower)[0]);
            assert_ne!(colors(&raise)[0], colors(&invalid)[0]);
            assert_ne!(
                positions(&raise),
                positions(&invalid),
                "X must differ from plus"
            );

            let top = column_surfaces(&park, pos, rotation)[0].unwrap();
            let center = top.vertices.map(Vec3::truncate).into_iter().sum::<Vec2>() / 4.0;
            assert!(
                positions(&lower).iter().any(|&point| {
                    let point = Vec3::from_array(point);
                    point.truncate().distance(center) < 7.0
                        && (point.z - top.depth_at(point.truncate()) - SELECTION_BIAS * 3.0).abs()
                            < 0.0001
                }),
                "lowering lost its visible current-top minus"
            );
        }
    }

    #[test]
    fn foreground_columns_occlude_ghosts_without_depth_or_outline_leaks() {
        for rotation in 0..4 {
            let mut park = Park::new(32, 32);
            let target = unrotate(10, 10, 32, rotation);
            let foreground = unrotate(11, 11, 32, rotation);
            park.set_terrain_height(target, 8).unwrap();
            park.set_terrain_height(foreground, 12).unwrap();
            let center = project(10.0, 10.0, 8.0, 32).truncate();
            assert_eq!(
                pick_surface(center, &park, rotation, ViewMode::Surface),
                Some(SurfaceId::Terrain(foreground))
            );
            for raising in [false, true] {
                let ghost =
                    terrain_preview_mesh(Some(preview(target, 8, raising, true)), &park, rotation);
                let foreground_faces = column_surfaces(&park, foreground, rotation);
                let mut occluded_vertices = 0;
                for &point in positions(&ghost) {
                    let point = Vec3::from_array(point);
                    let nearest = foreground_faces
                        .into_iter()
                        .flatten()
                        .filter_map(|surface| surface.hit_depth(point.truncate()))
                        .max_by(f32::total_cmp);
                    if let Some(depth) = nearest {
                        assert!(
                            point.z < depth,
                            "preview jumped in front of foreground terrain"
                        );
                        occluded_vertices += 1;
                    }
                }
                assert!(
                    occluded_vertices > 6,
                    "fixture must cover real ghost geometry"
                );
            }
        }
    }

    #[test]
    fn absent_stale_outside_and_nonadjacent_targets_are_empty_but_invalid_water_is_red() {
        let park = Park::new(32, 32);
        let valid = preview(TilePos { x: 10, y: 10 }, 0, true, true);
        assert_eq!(terrain_preview_mesh(None, &park, 0).count_vertices(), 0);
        for invalid in [
            TerrainPreview {
                pos: TilePos { x: 32, y: 0 },
                ..valid
            },
            TerrainPreview {
                height: 1,
                target_height: 2,
                ..valid
            },
            TerrainPreview {
                height: 33,
                target_height: 32,
                ..valid
            },
            TerrainPreview {
                target_height: 32,
                ..valid
            },
        ] {
            assert_eq!(
                terrain_preview_mesh(Some(invalid), &park, 0).count_vertices(),
                0
            );
        }
        assert_eq!(
            terrain_preview_mesh(Some(valid), &Park::new(32, 16), 0).count_vertices(),
            0
        );
        let water = (0..park.height())
            .flat_map(|y| (0..park.width()).map(move |x| TilePos { x, y }))
            .find(|&pos| park.tile(pos).unwrap().terrain == Terrain::Water)
            .unwrap();
        let mesh = terrain_preview_mesh(Some(preview(water, 0, true, false)), &park, 0);
        assert!(!positions(&mesh).is_empty());
        let red = rgb(225, 68, 63);
        assert!(colors(&mesh).iter().all(|color| color[..3] == red[..3]));
    }
}
