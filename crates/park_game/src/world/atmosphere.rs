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
            if seed.is_multiple_of(29) && !park.objects().iter().any(|o| o.pos == pos) {
                for (dx, dy) in [(-7.0, -1.0), (0.0, 2.0), (5.0, -2.0)] {
                    let c = center + Vec2::new(dx, dy);
                    mesh.surface_ellipse(
                        top,
                        c + Vec2::new(1.0, -1.0),
                        3.1,
                        1.3,
                        DETAIL_BIAS,
                        rgb(93, 140, 79),
                    );
                    mesh.surface_ellipse(
                        top,
                        c,
                        1.5,
                        1.1,
                        DETAIL_BIAS * 2.0,
                        if seed.is_multiple_of(2) {
                            rgb(243, 220, 158)
                        } else {
                            rgb(197, 172, 203)
                        },
                    );
                    mesh.surface_ellipse(top, c, 0.55, 0.4, DETAIL_BIAS * 3.0, rgb(229, 176, 87));
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
