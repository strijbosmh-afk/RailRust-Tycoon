//! Decorative surface details share terrain clipping/depth and never alter picking.
use super::*;
pub(super) fn ground_details(
    mesh: &mut Painter,
    park: &Park,
    pos: TilePos,
    center: Vec2,
    top: Surface,
) {
    let seed = noise(pos);
    // The close-view material marks are clipped to this exact roof. Diagnostic
    // maps keep the simpler material so their geometry stays bounded.
    if park.width() <= 32 {
        match park.tile(pos).unwrap().terrain {
            Terrain::Grass if seed.is_multiple_of(4) => {
                let offset = Vec2::new((seed % 13) as f32 - 6.0, ((seed >> 7) % 7) as f32 - 3.0);
                mesh.surface_ellipse(
                    top,
                    center + offset,
                    12.0,
                    4.2,
                    DETAIL_BIAS,
                    rgb(79, 184, 68),
                );
                mesh.surface_ellipse(
                    top,
                    center + offset + Vec2::new(-4.0, 1.5),
                    6.0,
                    2.0,
                    DETAIL_BIAS * 2.0,
                    rgb(111, 204, 80),
                );
            }
            Terrain::Sand if seed.is_multiple_of(3) => {
                let shift = (seed % 9) as f32 - 4.0;
                for offset in [-3.0, 1.0, 5.0] {
                    mesh.surface_line(
                        top,
                        center + Vec2::new(-9.0 + shift, offset),
                        center + Vec2::new(8.0 + shift, offset + 1.0),
                        0.7,
                        DETAIL_BIAS,
                        rgb(225, 211, 167),
                    );
                }
            }
            _ => {}
        }
    }
    match park.tile(pos).unwrap().terrain {
        Terrain::Grass => {
            // Tiny paired blades break up broad flat areas without a texture pack.
            if seed.is_multiple_of(7) {
                let p =
                    center + Vec2::new((seed % 15) as f32 - 7.0, ((seed / 17) % 7) as f32 - 3.0);
                mesh.surface_line(
                    top,
                    p,
                    p + Vec2::new(2.0, 2.3),
                    0.7,
                    DETAIL_BIAS,
                    rgb(135, 175, 106),
                );
                mesh.surface_line(
                    top,
                    p + Vec2::X * 3.0,
                    p + Vec2::new(4.0, 2.5),
                    0.7,
                    DETAIL_BIAS,
                    rgb(93, 143, 77),
                );
            }
            if seed.is_multiple_of(11) && !park.objects().iter().any(|o| o.pos == pos) {
                for (dx, dy) in [(-7.0, -1.0), (0.0, 2.0), (5.0, -2.0)] {
                    let c = center + Vec2::new(dx, dy);
                    mesh.surface_ellipse(
                        top,
                        c + Vec2::new(1.0, -1.0),
                        4.3,
                        1.9,
                        DETAIL_BIAS,
                        rgb(56, 144, 68),
                    );
                    mesh.surface_ellipse(
                        top,
                        c,
                        2.1,
                        1.5,
                        DETAIL_BIAS * 2.0,
                        if seed.is_multiple_of(2) {
                            rgb(255, 202, 63)
                        } else {
                            rgb(234, 78, 166)
                        },
                    );
                    mesh.surface_ellipse(top, c, 0.55, 0.4, DETAIL_BIAS * 3.0, rgb(255, 240, 104));
                }
            }
        }
        Terrain::Sand if seed.is_multiple_of(3) => {
            mesh.surface_line(
                top,
                center - Vec2::X * 7.0,
                center + Vec2::X * 6.0,
                0.7,
                DETAIL_BIAS,
                rgb(224, 208, 161),
            );
        }
        _ => {}
    }
}

/// Soft water-side bands follow the actual land/water boundary in every
/// camera rotation. The material stays on the water face and cannot change
/// hit targets or cover a higher neighboring tile.
pub(super) fn shore_details(
    mesh: &mut Painter,
    park: &Park,
    pos: TilePos,
    rotation: u8,
    top: Surface,
) {
    if park.width() > 32
        || park
            .tile(pos)
            .is_none_or(|tile| tile.terrain != Terrain::Water)
    {
        return;
    }
    let center = top.vertices.map(Vec3::truncate).into_iter().sum::<Vec2>() / 4.0;
    let (x, y) = rotate(pos, park.width(), rotation);
    let edges = [
        (x.checked_sub(1).map(|x| (x, y)), 0),
        ((y + 1 < park.width()).then_some((x, y + 1)), 1),
        ((x + 1 < park.width()).then_some((x + 1, y)), 2),
        (y.checked_sub(1).map(|y| (x, y)), 3),
    ];
    for (neighbor, edge) in edges {
        let Some((nx, ny)) = neighbor else { continue };
        let neighbor = unrotate(nx, ny, park.width(), rotation);
        if park
            .tile(neighbor)
            .is_none_or(|tile| tile.terrain == Terrain::Water)
        {
            continue;
        }
        let a = top.vertices[edge].truncate();
        let b = top.vertices[(edge + 1) % 4].truncate();
        mesh.surface_line(
            top,
            a.lerp(center, 0.08),
            b.lerp(center, 0.08),
            2.2,
            DETAIL_BIAS,
            rgb(43, 126, 141),
        );
        mesh.surface_line(
            top,
            a.lerp(center, 0.17),
            b.lerp(center, 0.17),
            0.9,
            DETAIL_BIAS * 2.0,
            rgb(177, 219, 196),
        );
    }
}
/// Water glints are cosmetic and capped to normal playable maps. Static waves
/// remain on large maps, so animation never scans the 256x256 diagnostic map.
pub(crate) fn water_mesh(park: &Park, rotation: u8, mode: ViewMode, phase: u8) -> Mesh {
    let mut mesh = Painter::default();
    if mode == ViewMode::Underground || park.width() > 32 {
        return mesh.finish();
    }
    for y in 0..park.height() {
        for x in 0..park.width() {
            let pos = TilePos { x, y };
            if park.tile(pos).unwrap().terrain != Terrain::Water || !noise(pos).is_multiple_of(3) {
                continue;
            }
            let top = column_surfaces(park, pos, rotation)[0].unwrap();
            let center = elevated_tile_center(park, pos, rotation).unwrap();
            let wave = ((u32::from(phase) + noise(pos) % 16) % 16) as f32;
            let shift = (wave / 16.0 * std::f32::consts::TAU).sin() * 2.5;
            let width = 4.0 + (wave / 16.0 * std::f32::consts::PI).sin() * 5.0;
            mesh.surface_line(
                top,
                center + Vec2::new(-width, shift),
                center + Vec2::new(width, shift),
                0.85,
                DETAIL_BIAS * 3.0,
                rgb(147, 205, 197),
            );
        }
    }
    mesh.finish()
}
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn shoreline_stays_on_water_roofs_in_every_rotation_and_preserves_park() {
        let park = Park::new(32, 32);
        let before = park.to_save_bytes().unwrap();
        let shore = (0..park.height())
            .flat_map(|y| (0..park.width()).map(move |x| TilePos { x, y }))
            .find(|pos| {
                park.tile(*pos).unwrap().terrain == Terrain::Water
                    && [(-1_i32, 0_i32), (1, 0), (0, -1), (0, 1)]
                        .iter()
                        .any(|(dx, dy)| {
                            let nx = pos.x as i32 + dx;
                            let ny = pos.y as i32 + dy;
                            nx >= 0
                                && ny >= 0
                                && (nx as u32) < park.width()
                                && (ny as u32) < park.height()
                                && park
                                    .tile(TilePos {
                                        x: nx as u32,
                                        y: ny as u32,
                                    })
                                    .unwrap()
                                    .terrain
                                    != Terrain::Water
                        })
            })
            .unwrap();
        for rotation in 0..4 {
            let top = column_surfaces(&park, shore, rotation)[0].unwrap();
            let mut painter = Painter::default();
            shore_details(&mut painter, &park, shore, rotation, top);
            let mesh = painter.finish();
            let Some(VertexAttributeValues::Float32x3(vertices)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("shore mesh requires positions");
            };
            assert!(!vertices.is_empty());
            for &[x, y, z] in vertices {
                assert!(x.is_finite() && y.is_finite() && z.is_finite());
                assert!(top.hit_depth(Vec2::new(x, y)).is_some());
            }
        }
        assert_eq!(park.to_save_bytes().unwrap(), before);
    }
    #[test]
    fn glints_obey_surface_cutaway_and_large_map_bounds_without_mutating_park() {
        let p = Park::new(32, 32);
        let before = p.to_save_bytes().unwrap();
        for r in 0..4 {
            for phase in [0, 7, 15] {
                let m = water_mesh(&p, r, ViewMode::Surface, phase);
                assert!(m.count_vertices() > 0);
            }
        }
        assert_eq!(
            water_mesh(&p, 0, ViewMode::Underground, 0).count_vertices(),
            0
        );
        assert_eq!(
            water_mesh(&Park::new(64, 64), 0, ViewMode::Surface, 0).count_vertices(),
            0
        );
        assert_eq!(p.to_save_bytes().unwrap(), before);
    }
}
