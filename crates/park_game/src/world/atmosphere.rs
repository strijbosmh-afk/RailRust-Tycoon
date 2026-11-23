//! Decorative surface details share terrain clipping/depth and never alter picking.
use super::*;
pub(super) fn ground_details(
    mesh: &mut Painter,
    park: &Park,
    pos: TilePos,
    center: Vec2,
    top: Surface,
) {
    landscape::ground(mesh, park, pos, top);
    let seed = noise(pos);
    if park.width() <= 64
        && park.tile(pos).is_some_and(|t| t.terrain == Terrain::Grass)
        && seed.is_multiple_of(19)
        && !park.objects().iter().any(|o| o.pos == pos)
    {
        for (dx, dy) in [(-5.0, 0.0), (1.0, 2.0), (5.0, -1.0)] {
            let c = center + Vec2::new(dx, dy);
            mesh.surface_ellipse(top, c, 2.8, 1.6, DETAIL_BIAS * 2.0, rgb(48, 120, 58));
            mesh.surface_ellipse(
                top,
                c + Vec2::Y * 0.4,
                1.4,
                1.0,
                DETAIL_BIAS * 3.0,
                if seed.is_multiple_of(2) {
                    rgb(247, 214, 98)
                } else {
                    rgb(229, 120, 169)
                },
            );
        }
    }
}

pub(super) fn water_ripples(mesh: &mut Painter, pos: TilePos, center: Vec2, top: Surface) {
    let seed = noise(pos);
    if seed.is_multiple_of(3) {
        return;
    }
    let origin = center + Vec2::new(((seed >> 8) % 17) as f32 - 8.0, (seed % 13) as f32 - 6.0);
    let length = 6.0 + ((seed >> 16) % 13) as f32;
    mesh.surface_line(
        top,
        origin - Vec2::X * (length * 0.5),
        origin + Vec2::X * (length * 0.5),
        0.85,
        DETAIL_BIAS,
        rgb(83, 172, 182),
    );
    let crest = origin + Vec2::new(length * 0.18, 2.0);
    mesh.surface_line(
        top,
        crest - Vec2::X * (length * 0.28),
        crest + Vec2::X * (length * 0.28),
        0.6,
        DETAIL_BIAS,
        rgb(146, 211, 199),
    );
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
    if park.width() > 64
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
        let shore_color = if park
            .tile(neighbor)
            .is_some_and(|t| t.terrain == Terrain::Sand)
        {
            rgb(196, 192, 135)
        } else {
            rgb(60, 142, 110)
        };
        mesh.paint_on_surface(
            top,
            &[a, b, b.lerp(center, 0.36), a.lerp(center, 0.36)],
            DETAIL_BIAS * 0.5,
            rgb(44, 153, 157),
        );
        mesh.surface_line(
            top,
            a.lerp(center, 0.025),
            b.lerp(center, 0.025),
            1.4,
            DETAIL_BIAS,
            shore_color,
        );
        mesh.surface_line(
            top,
            a.lerp(center, 0.08),
            b.lerp(center, 0.08),
            2.2,
            DETAIL_BIAS,
            rgb(65, 168, 161),
        );
        mesh.surface_line(
            top,
            a.lerp(center, 0.17),
            b.lerp(center, 0.17),
            0.9,
            DETAIL_BIAS * 2.0,
            rgb(131, 200, 180),
        );
        // Small water-edge stones break up the straight bank, while remaining
        // clipped to the actual water roof and below crossing paths.
        if noise(pos).is_multiple_of(3) {
            for t in [0.27, 0.67] {
                let stone = a.lerp(b, t).lerp(center, 0.08);
                mesh.surface_ellipse(top, stone, 1.8, 0.9, DETAIL_BIAS * 3.0, rgb(100, 135, 112));
                mesh.surface_ellipse(
                    top,
                    stone + Vec2::new(-0.3, 0.3),
                    1.2,
                    0.55,
                    DETAIL_BIAS * 4.0,
                    rgb(181, 194, 143),
                );
            }
        }
    }
}
/// Water glints are cosmetic and capped to normal playable maps. Static waves
/// remain on large maps, so animation never scans the 256x256 diagnostic map.
pub(crate) fn water_mesh(park: &Park, rotation: u8, mode: ViewMode, phase: u8) -> Mesh {
    let mut mesh = Painter::default();
    if mode == ViewMode::Underground || park.width() > 64 {
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
            water_mesh(&Park::new(128, 128), 0, ViewMode::Surface, 0).count_vertices(),
            0
        );
        assert!(water_mesh(&Park::new(48, 48), 0, ViewMode::Surface, 0).count_vertices() > 0);
        assert_eq!(p.to_save_bytes().unwrap(), before);
    }
}
