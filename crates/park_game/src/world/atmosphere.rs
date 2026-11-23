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
    if park.width() == park_sim::demo_park::DEMO_PARK_SIZE
        && park
            .tile(TilePos { x: 23, y: 25 })
            .is_some_and(|tile| tile.terrain == Terrain::Water)
        && matches!((pos.x, pos.y), (21, 15) | (22, 16) | (26, 15) | (27, 17))
        && park
            .tile(pos)
            .is_some_and(|tile| tile.terrain == Terrain::Grass)
    {
        for (offset, size) in [(Vec2::new(-8.0, 0.0), 7.0), (Vec2::new(5.0, -2.0), 5.0)] {
            let base = center + offset;
            let depth = top.depth_at(base) + DETAIL_BIAS * 5.0;
            let left = base - Vec2::X * size;
            let right = base + Vec2::X * size;
            let crown = base + Vec2::new(-size * 0.25, size * 0.85);
            mesh.triangle([left, right, crown], depth, rgb(102, 114, 105));
            mesh.triangle(
                [left, crown, base + Vec2::Y * (size * 0.33)],
                depth + DETAIL_BIAS,
                rgb(159, 164, 147),
            );
        }
    }
    if park.width() <= 64
        && park
            .tile(pos)
            .is_some_and(|tile| tile.terrain == Terrain::Grass && tile.owned)
        && seed.is_multiple_of(3)
        && noise(TilePos {
            x: pos.x / 4,
            y: pos.y / 4,
        })
        .is_multiple_of(3)
        && !park.objects().iter().any(|object| object.pos == pos)
    {
        // Nearby tiles share a coarse seed, leaving clear lawns between clusters.
        for offset in [
            Vec2::new(-11.0, 1.0),
            Vec2::new(2.0, -4.0),
            Vec2::new(10.0, 4.0),
        ] {
            let base = center + offset;
            mesh.surface_ellipse(top, base, 5.5, 2.8, DETAIL_BIAS * 2.0, rgb(69, 118, 57));
            mesh.surface_ellipse(
                top,
                base + Vec2::Y,
                3.7,
                2.2,
                DETAIL_BIAS * 3.0,
                rgb(100, 151, 71),
            );
        }
    }
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

pub(super) fn structure_shadows(mesh: &mut Painter, park: &Park, rotation: u8) {
    for object in park.objects() {
        let Some(tile) = park.tile(object.pos) else {
            continue;
        };
        let (offset, radii) = match object.kind {
            ObjectKind::Path => {
                let height = object_base_at(object, Vec2::ZERO) - f32::from(tile.height);
                if height < 0.75 {
                    continue;
                }
                let reach = height.min(10.0);
                (reach, Vec2::new(11.0 + reach, 5.5))
            }
            ObjectKind::Track
                if object_visible(park, object, ViewMode::Surface)
                    && train_render::piece_for(park, object.id)
                        .is_some_and(|piece| piece.kind == park_sim::TrackKind::Station) =>
            {
                (10.0, Vec2::new(23.0, 9.0))
            }
            _ => continue,
        };
        if let (Some(base), Some(top)) = (
            elevated_tile_center(park, object.pos, rotation),
            column_surfaces(park, object.pos, rotation)[0],
        ) {
            contact_shadow(
                mesh,
                top,
                base + Vec2::new(offset, -2.0),
                radii,
                tile.terrain,
            );
            mesh.end_group(PaintLayer::Terrain);
        }
    }
}

pub(super) fn contact_shadow(
    mesh: &mut Painter,
    top: Surface,
    center: Vec2,
    radii: Vec2,
    terrain: Terrain,
) {
    let (outer, inner) = match terrain {
        Terrain::Grass => (rgb(58, 126, 63), rgb(49, 113, 58)),
        Terrain::Sand => (rgb(209, 185, 131), rgb(190, 166, 117)),
        Terrain::Water => return,
    };
    mesh.surface_ellipse(top, center, radii.x, radii.y, DETAIL_BIAS * 4.0, outer);
    mesh.surface_ellipse(
        top,
        center - Vec2::X * (radii.x * 0.12),
        radii.x * 0.72,
        radii.y * 0.68,
        DETAIL_BIAS * 5.0,
        inner,
    );
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
        if noise(neighbor).is_multiple_of(7) {
            let root = a.lerp(b, 0.35).lerp(center, 0.07);
            for (dx, height) in [(-3.0, 7.0), (0.0, 11.0), (3.0, 9.0)] {
                let tip = root + Vec2::new(dx * 1.5, height);
                mesh.surface_line(
                    top,
                    root + Vec2::X * dx,
                    tip,
                    1.2,
                    DETAIL_BIAS * 5.0,
                    rgb(85, 121, 54),
                );
                mesh.surface_line(
                    top,
                    tip - Vec2::Y * 2.5,
                    tip,
                    2.1,
                    DETAIL_BIAS * 6.0,
                    rgb(143, 114, 67),
                );
            }
        }
    }
}

fn fountain(mesh: &mut Painter, park: &Park, rotation: u8, phase: u8) {
    // The ornamental basin occupies the four central tiles of Sunhaven's
    // pond. Edits that displace its water or add a crossing remove the prop.
    let center = TilePos { x: 23, y: 25 };
    if park.width() != 48
        || !(24..=27).all(|y| {
            (20..=27).all(|x| {
                park.tile(TilePos { x, y })
                    .is_some_and(|tile| tile.terrain == Terrain::Water && tile.height == 0)
            })
        })
        || park
            .objects()
            .iter()
            .any(|object| (22..=25).contains(&object.pos.x) && (24..=27).contains(&object.pos.y))
    {
        return;
    }
    let opposite = TilePos { x: 24, y: 26 };
    let (x, y) = rotate(center, park.width(), rotation);
    let (other_x, other_y) = rotate(opposite, park.width(), rotation);
    let base = project(
        (x + other_x) as f32 * 0.5,
        (y + other_y) as f32 * 0.5,
        0.0,
        park.width(),
    );
    let p = base.truncate();
    let z = base.z + DETAIL_BIAS * 6.0;
    mesh.ellipse(p, 22.0, 10.5, z, rgb(21, 103, 132));
    mesh.ellipse(
        p + Vec2::Y * 2.0,
        17.0,
        8.0,
        z + DETAIL_BIAS,
        rgb(167, 151, 110),
    );
    mesh.ellipse(
        p + Vec2::Y * 4.0,
        14.0,
        6.0,
        z + DETAIL_BIAS * 2.0,
        rgb(104, 199, 202),
    );
    mesh.line(
        p + Vec2::Y * 4.0,
        p + Vec2::Y * 13.0,
        6.0,
        z + DETAIL_BIAS * 3.0,
        rgb(191, 173, 123),
    );
    let pulse = (f32::from(phase) * std::f32::consts::TAU / 16.0).sin();
    let spray = p + Vec2::Y * 13.0;
    let crest = p + Vec2::new(pulse * 2.0, 53.0 + pulse * 3.0);
    mesh.line(spray, crest, 3.0, z + DETAIL_BIAS * 4.0, rgb(126, 222, 230));
    mesh.line(
        spray + Vec2::X,
        crest + Vec2::X,
        1.1,
        z + DETAIL_BIAS * 5.0,
        rgb(236, 252, 238),
    );
    for sign in [-1.0, 1.0] {
        let mut previous = spray;
        for step in 1..=5 {
            let t = step as f32 / 5.0;
            let point =
                spray + Vec2::new(sign * (15.0 + pulse) * t, 72.0 * t * (1.0 - t) - 9.0 * t);
            mesh.line(
                previous,
                point,
                1.3,
                z + DETAIL_BIAS * 4.0,
                rgb(165, 232, 225),
            );
            previous = point;
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
            let shift = (wave / 16.0 * std::f32::consts::TAU).sin() * 3.5;
            let width = 3.0 + (wave / 16.0 * std::f32::consts::PI).sin() * 8.0;
            mesh.surface_line(
                top,
                center + Vec2::new(-width, shift),
                center + Vec2::new(width, shift),
                1.05,
                DETAIL_BIAS * 3.0,
                rgb(147, 205, 197),
            );
        }
    }
    fountain(&mut mesh, park, rotation, phase);
    mesh.finish()
}
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn demo_fountain_tracks_the_pond_and_moves_without_editing_the_park() {
        let park = park_sim::demo_park::build_demo_park().unwrap();
        let before = park.to_save_bytes().unwrap();
        for rotation in 0..4 {
            let mut still = Painter::default();
            let mut moving = Painter::default();
            fountain(&mut still, &park, rotation, 0);
            fountain(&mut moving, &park, rotation, 4);
            assert!(!still.positions.is_empty());
            assert_eq!(still.positions.len(), moving.positions.len());
            assert_ne!(still.positions, moving.positions);
            assert!(
                moving
                    .positions
                    .iter()
                    .flatten()
                    .all(|value| value.is_finite())
            );
        }
        assert_eq!(before, park.to_save_bytes().unwrap());
        let mut changed = park;
        changed
            .apply_terrain_brush(
                TilePos { x: 23, y: 25 },
                0,
                park_sim::TerrainBrushKind::Sand,
            )
            .unwrap();
        let mut drained = Painter::default();
        fountain(&mut drained, &changed, 0, 0);
        assert!(drained.positions.is_empty());
    }

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
