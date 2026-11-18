//! Deterministic landscape materials, painted on authoritative surface planes.
use super::*;

fn material_point(top: Surface, u: f32, v: f32) -> Vec2 {
    let [a, b, c, d] = top.vertices.map(Vec3::truncate);
    a.lerp(d, u).lerp(b.lerp(c, u), v)
}

fn patch(mesh: &mut Painter, top: Surface, bounds: [f32; 4], ink: Ink, bias: f32) {
    let [u, v, w, h] = bounds;
    mesh.paint_on_surface(
        top,
        &[
            material_point(top, u, v),
            material_point(top, u + w, v),
            material_point(top, u + w, v + h),
            material_point(top, u, v + h),
        ],
        bias,
        ink,
    );
}

pub(super) fn ground(mesh: &mut Painter, park: &Park, pos: TilePos, top: Surface) {
    if park.width() > 64 {
        return;
    }
    let seed = noise(pos);
    let Some(tile) = park.tile(pos) else { return };
    match tile.terrain {
        Terrain::Grass => {
            // Muted, irregular turf islands replace the repeated bright dots.
            for i in 0..3_u32 {
                let n = seed.rotate_left(i * 7).wrapping_add(i * 193);
                let u = 0.10 + (n % 53) as f32 / 100.0;
                let v = 0.09 + ((n >> 7) % 57) as f32 / 100.0;
                let w = 0.16 + ((n >> 13) % 15) as f32 / 100.0;
                let h = 0.13 + ((n >> 19) % 14) as f32 / 100.0;
                let ink = if tile.owned {
                    if i % 2 == 0 {
                        rgb(59, 150, 67)
                    } else {
                        rgb(71, 161, 72)
                    }
                } else {
                    rgb(106, 134, 94)
                };
                let p = |u, v| material_point(top, u, v);
                mesh.paint_on_surface(
                    top,
                    &[
                        p(u, v + h * 0.3),
                        p(u + w * 0.35, v),
                        p(u + w, v + h * 0.12),
                        p(u + w * 0.86, v + h),
                        p(u + w * 0.2, v + h * 0.85),
                    ],
                    DETAIL_BIAS,
                    ink,
                );
            }
            if seed.is_multiple_of(3) {
                let base = material_point(
                    top,
                    0.2 + (seed % 59) as f32 / 100.0,
                    0.24 + ((seed >> 8) % 47) as f32 / 100.0,
                );
                for (dx, h) in [(-1.9, 2.2), (0.0, 3.2), (1.8, 1.8)] {
                    mesh.surface_line(
                        top,
                        base + Vec2::X * dx,
                        base + Vec2::new(dx - 0.6, h),
                        0.65,
                        DETAIL_BIAS * 2.0,
                        rgb(119, 185, 83),
                    );
                }
            }
        }
        Terrain::Sand => {
            for i in 0..4_u32 {
                let v = 0.15 + i as f32 * 0.2;
                let shift = (seed.rotate_left(i * 3) % 11) as f32 / 100.0;
                mesh.surface_line(
                    top,
                    material_point(top, 0.08 + shift, v),
                    material_point(top, 0.66 + shift, v + 0.12),
                    0.55,
                    DETAIL_BIAS,
                    rgb(244, 219, 160),
                );
            }
            for i in 0..4_u32 {
                let n = seed.rotate_left(i * 5);
                patch(
                    mesh,
                    top,
                    [
                        0.1 + (n % 73) as f32 / 100.0,
                        0.1 + ((n >> 9) % 73) as f32 / 100.0,
                        0.018,
                        0.026,
                    ],
                    rgb(193, 166, 115),
                    DETAIL_BIAS * 2.0,
                );
            }
        }
        Terrain::Water => {}
    }
}

pub(super) fn cliff(mesh: &mut Painter, park: &Park, pos: TilePos, face: Surface) {
    if park.width() > 64 {
        return;
    }
    let [a, b, _, d] = face.vertices.map(Vec3::truncate);
    let height = a.distance(d);
    if height < 2.0 {
        return;
    }
    let grass = park.tile(pos).is_some_and(|t| t.terrain == Terrain::Grass);
    mesh.surface_line(
        face,
        a,
        b,
        2.0,
        DETAIL_BIAS,
        if grass {
            rgb(54, 118, 51)
        } else {
            rgb(211, 182, 126)
        },
    );
    let seed = noise(pos);
    for row in 0..((height / 9.0).ceil() as u32).min(56) {
        let v = (row as f32 * 9.0 + 4.0) / height;
        if v >= 0.99 {
            break;
        }
        let shift = if row % 2 == 0 { 0.08 } else { 0.26 };
        for col in 0..3 {
            let u = shift + col as f32 * 0.32;
            let n = seed.wrapping_add(row * 17 + col * 41);
            let ink = if face.kind == SurfaceKind::LeftCliff {
                varied(146, 109, 68, (n % 13) as i16 - 6)
            } else {
                varied(107, 79, 49, (n % 11) as i16 - 5)
            };
            patch(
                mesh,
                face,
                // A cliff's a->d edge runs downward, unlike a ground tile.
                [v, u, (5.0 / height).min(0.25), 0.22],
                ink,
                DETAIL_BIAS,
            );
        }
    }
}

pub(super) fn path(
    mesh: &mut Painter,
    park: &Park,
    object: &SurfaceObject,
    rotation: u8,
    top: Surface,
) {
    let (min_x, max_x, min_y, max_y) = object_bounds(park, object);
    let (lo, hi, across_lo, across_hi) = match object_axis(object) {
        Axis::X => (min_x, max_x, min_y, max_y),
        Axis::Y => (min_y, max_y, min_x, max_x),
    };
    let p = |a, b| object_point(object, a, b, object_top(object), park.width(), rotation);
    let queue = object.queue_owner.is_some();
    let ground = object.path_slope == PathSlope::Flat
        && park
            .tile(object.pos)
            .is_some_and(|t| t.height == object.height && t.terrain != Terrain::Water);
    let detail = park.width() <= 64;
    if ground && !queue {
        let seed = noise(object.pos);
        // Staggered stone courses form continuous promenades, including junctions.
        let rows = if detail { 4 } else { 2 };
        for row in 0..rows {
            let y = across_lo + (across_hi - across_lo) * row as f32 / rows as f32;
            let h = (across_hi - across_lo) / rows as f32;
            let step = 0.32;
            let shift = if row % 2 == 0 { 0.0 } else { 0.16 };
            for col in -1..5 {
                let x = lo + col as f32 * step + shift;
                let start = x.max(lo) + 0.012;
                let end = (x + step).min(hi) - 0.012;
                if start >= end {
                    continue;
                }
                let n = seed.wrapping_add((row * 31 + (col + 1) * 7) as u32);
                let ink = varied(218, 207, 172, (n % 15) as i16 - 7);
                mesh.paint_on_surface(
                    top,
                    &[
                        p(start, y + 0.012),
                        p(end, y + 0.012),
                        p(end, y + h - 0.012),
                        p(start, y + h - 0.012),
                    ],
                    DETAIL_BIAS,
                    ink,
                );
                if detail {
                    mesh.surface_line(
                        top,
                        p(start, y + 0.018),
                        p(start, y + h - 0.018),
                        0.6,
                        DETAIL_BIAS * 2.0,
                        rgb(162, 151, 123),
                    );
                    mesh.surface_line(
                        top,
                        p(start, y + h - 0.018),
                        p(end, y + h - 0.018),
                        0.55,
                        DETAIL_BIAS * 2.0,
                        rgb(243, 232, 197),
                    );
                    let grain = (start + end) * 0.5;
                    mesh.surface_line(
                        top,
                        p(grain - 0.025, y + h * 0.45),
                        p(grain + 0.035, y + h * 0.52),
                        0.45,
                        DETAIL_BIAS * 2.0,
                        varied(191, 183, 153, (n % 9) as i16),
                    );
                }
            }
        }
    } else {
        // Transverse planks follow the incline; shallow grain stays on that plane.
        for a in [-0.4, -0.2, 0.0, 0.2, 0.4] {
            mesh.surface_line(
                top,
                p(a, across_lo + 0.01),
                p(a, across_hi - 0.01),
                0.8,
                DETAIL_BIAS,
                if queue {
                    rgb(77, 151, 171)
                } else {
                    rgb(139, 95, 53)
                },
            );
            if detail && !queue {
                for b in [-0.18, 0.08] {
                    mesh.surface_line(
                        top,
                        p(a + 0.035, b),
                        p(a + 0.035, b + 0.12),
                        0.45,
                        DETAIL_BIAS * 2.0,
                        rgb(227, 193, 127),
                    );
                }
                for b in [across_lo + 0.055, across_hi - 0.055] {
                    mesh.surface_ellipse(
                        top,
                        p(a + 0.075, b),
                        0.7,
                        0.4,
                        DETAIL_BIAS * 3.0,
                        rgb(86, 83, 67),
                    );
                }
            }
        }
    }
    let border = if queue {
        rgb(230, 226, 185)
    } else if ground {
        rgb(242, 229, 187)
    } else {
        rgb(105, 73, 46)
    };
    // Only exposed edges receive curbs. Open connections remain unobstructed.
    for across in [
        (across_lo > -0.5).then_some(across_lo + 0.04),
        (across_hi < 0.5).then_some(across_hi - 0.04),
    ]
    .into_iter()
    .flatten()
    {
        mesh.surface_line(
            top,
            p(lo, across),
            p(hi, across),
            2.2,
            DETAIL_BIAS * 3.0,
            border,
        );
        if detail {
            for along in [-0.40, 0.0, 0.40] {
                mesh.surface_line(
                    top,
                    p(along, across - 0.025),
                    p(along, across + 0.025),
                    0.65,
                    DETAIL_BIAS * 4.0,
                    if ground {
                        rgb(145, 134, 103)
                    } else {
                        rgb(67, 73, 60)
                    },
                );
            }
        }
    }
    if queue {
        if detail {
            for across in [-0.16, 0.16] {
                mesh.surface_line(
                    top,
                    p(lo + 0.025, across),
                    p(hi - 0.025, across),
                    0.7,
                    DETAIL_BIAS * 2.0,
                    rgb(143, 205, 204),
                );
            }
        }
        for along in [-0.30, 0.0, 0.30] {
            mesh.surface_line(
                top,
                p(along - 0.065, 0.0),
                p(along + 0.065, 0.0),
                1.8,
                DETAIL_BIAS * 3.0,
                rgb(247, 224, 137),
            );
        }
    }
}

pub(super) fn flowerbed(mesh: &mut Painter, base: Vec2, top: Surface, seed: u32) {
    let dark = rgb(52, 99, 45);
    mesh.surface_ellipse(top, base, 23.0, 11.5, DETAIL_BIAS, rgb(118, 101, 76));
    mesh.surface_ellipse(top, base, 20.5, 9.8, DETAIL_BIAS * 2.0, rgb(68, 63, 41));
    for i in 0..20 {
        let a = i as f32 * std::f32::consts::TAU / 20.0;
        let p = base + Vec2::new(a.cos() * 22.0, a.sin() * 10.5);
        mesh.surface_ellipse(
            top,
            p,
            2.8,
            1.8,
            DETAIL_BIAS * 3.0,
            if i % 3 == 0 {
                rgb(235, 218, 174)
            } else {
                rgb(193, 179, 139)
            },
        );
    }
    let palette = match seed % 3 {
        0 => [rgb(239, 90, 137), rgb(255, 211, 65), rgb(250, 238, 206)],
        1 => [rgb(167, 101, 206), rgb(238, 149, 199), rgb(250, 232, 152)],
        _ => [rgb(239, 108, 59), rgb(255, 194, 44), rgb(243, 225, 172)],
    };
    for i in 0..24_u32 {
        let a = i as f32 * 2.399_963;
        let r = 2.0 + (i % 6) as f32 * 1.9;
        let p = base + Vec2::new(a.cos() * r, a.sin() * r * 0.44);
        let depth = top.depth_at(p) + 0.03;
        let height = 2.5 + ((i + seed) % 4) as f32 * 0.8;
        let bloom = p + Vec2::new(a.sin() * 0.7, height);
        mesh.surface_ellipse(top, p, 3.6, 1.7, DETAIL_BIAS * 4.0, dark);
        mesh.line(p, bloom, 0.65, depth, rgb(56, 119, 43));
        for side in [-1.0, 1.0] {
            let leaf = p + Vec2::new(side * 2.7, height * 0.45);
            mesh.polygon(
                &[p + Vec2::Y, leaf, leaf + Vec2::Y * 1.2, p + Vec2::Y * 2.2],
                depth + 0.001,
                if side < 0.0 {
                    rgb(109, 176, 57)
                } else {
                    rgb(57, 131, 46)
                },
            );
        }
        let color = palette[(i % 3) as usize];
        let mut shade = color;
        for channel in &mut shade[..3] {
            *channel *= 0.72;
        }
        mesh.ellipse(bloom + Vec2::new(0.4, -0.5), 2.7, 1.9, depth + 0.002, shade);
        for petal in 0..5 {
            let angle = petal as f32 * std::f32::consts::TAU / 5.0 + a;
            let offset = Vec2::new(angle.cos() * 1.6, angle.sin() * 1.05);
            mesh.ellipse(bloom + offset, 1.5, 1.05, depth + 0.003, color);
        }
        mesh.ellipse(bloom, 0.85, 0.62, depth + 0.004, rgb(126, 79, 25));
        mesh.ellipse(
            bloom + Vec2::new(-0.2, 0.3),
            0.55,
            0.4,
            depth + 0.005,
            rgb(255, 226, 105),
        );
    }
}

pub(super) fn pine(mesh: &mut Painter, base: Vec2, depth: f32, scale: f32, seed: u32) {
    for level in 0..6 {
        let y = 18.0 + level as f32 * 7.0;
        let w = 21.0 - level as f32 * 2.7;
        let tip = y + 18.0;
        let color = if seed.is_multiple_of(2) {
            varied(32, 116, 78, level * 3)
        } else {
            varied(38, 125, 69, level * 3)
        };
        mesh.polygon(
            &[
                base + Vec2::new(-w, y) * scale,
                base + Vec2::new(-w * 0.68, y + 4.0) * scale,
                base + Vec2::new(0.0, tip) * scale,
                base + Vec2::new(w * 0.62, y + 3.0) * scale,
                base + Vec2::new(w, y) * scale,
                base + Vec2::new(w * 0.3, y - 2.0) * scale,
                base + Vec2::new(-w * 0.28, y - 1.0) * scale,
            ],
            depth + level as f32 * 0.001,
            color,
        );
        for branch in 0..3 {
            let by = y + branch as f32 * 3.8;
            let bw = w * (0.80 - branch as f32 * 0.22);
            mesh.line(
                base + Vec2::new(-bw, by) * scale,
                base + Vec2::new(-bw * 0.18, by + 2.5) * scale,
                1.1 * scale,
                depth + 0.007 + level as f32 * 0.001,
                rgb(101, 164, 91),
            );
            for side in [-1.0, 1.0] {
                let tip = base + Vec2::new(side * bw * 0.8, by) * scale;
                mesh.ellipse(
                    tip,
                    (3.5 + bw * 0.12) * scale,
                    2.2 * scale,
                    depth + 0.008 + level as f32 * 0.001,
                    if side < 0.0 {
                        rgb(68, 146, 77)
                    } else {
                        rgb(29, 98, 66)
                    },
                );
                mesh.line(
                    tip + Vec2::new(-1.2, 0.5) * scale,
                    tip + Vec2::new(1.0, 1.3) * scale,
                    0.55 * scale,
                    depth + 0.009 + level as f32 * 0.001,
                    rgb(129, 185, 95),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cliff_strata_and_sand_are_clipped_in_every_camera_direction() {
        let mut park = Park::new(12, 12);
        let pos = TilePos { x: 6, y: 6 };
        park.set_terrain_height(pos, 12).unwrap();
        for rotation in 0..4 {
            for surface in column_surfaces(&park, pos, rotation)
                .into_iter()
                .flatten()
                .skip(1)
            {
                let mut painter = Painter::default();
                cliff(&mut painter, &park, pos, surface);
                let mesh = painter.finish();
                assert!(mesh.count_vertices() > 0);
                for v in mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap()
                {
                    assert!(surface.hit_depth(Vec2::new(v[0], v[1])).is_some());
                }
            }
        }
        park.apply_terrain_brush(pos, 0, park_sim::TerrainBrushKind::Sand)
            .unwrap();
        let before = park.to_save_bytes().unwrap();
        for rotation in 0..4 {
            let top = column_surfaces(&park, pos, rotation)[0].unwrap();
            let mut painter = Painter::default();
            ground(&mut painter, &park, pos, top);
            let mesh = painter.finish();
            assert!(mesh.count_vertices() > 0);
            for v in mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
            {
                assert!(top.hit_depth(Vec2::new(v[0], v[1])).is_some());
            }
        }
        assert_eq!(before, park.to_save_bytes().unwrap());
    }
    #[test]
    fn material_details_stay_on_sloped_decks_and_terrain_without_changing_park() {
        let mut park = Park::new(32, 32);
        let pos = TilePos { x: 15, y: 15 };
        park.set_terrain_height(pos, 4).unwrap();
        let id = park.place_sloped_path(pos, 4, PathSlope::East).unwrap();
        let before = park.to_save_bytes().unwrap();
        for rotation in 0..4 {
            let object = park.object(id).unwrap();
            let top = object_surfaces(&park, object, rotation)[0];
            let mut painter = Painter::default();
            path(&mut painter, &park, object, rotation, top);
            let mesh = painter.finish();
            for v in mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
            {
                let p = Vec3::from(*v);
                assert!(p.is_finite());
                let depth = top.hit_depth(p.truncate()).expect("detail escaped ramp");
                assert!((p.z - depth).abs() < 0.04);
            }
            let top = column_surfaces(&park, pos, rotation)[0].unwrap();
            let mut painter = Painter::default();
            ground(&mut painter, &park, pos, top);
            for seed in 0..12 {
                flowerbed(
                    &mut painter,
                    elevated_tile_center(&park, pos, rotation).unwrap(),
                    top,
                    seed,
                );
            }
            let mesh = painter.finish();
            for v in mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
            {
                let p = Vec3::from(*v);
                assert!(p.is_finite());
                assert!(
                    top.hit_depth(p.truncate()).is_some(),
                    "detail escaped terrain"
                );
            }
        }
        assert_eq!(before, park.to_save_bytes().unwrap());
    }
}
