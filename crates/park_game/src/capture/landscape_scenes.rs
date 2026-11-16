//! Isolated landscape inspection, built with production terrain/path/scenery APIs.
use super::*;
use park_sim::{PathDirection, PathSlope, SceneryKind, TerrainBrushKind};

pub(super) fn fixture() -> Park {
    let mut park = Park::new(32, 32);
    park.apply_terrain_profile(TerrainProfile::Flat);
    park.set_starting_budget(200_000).unwrap();
    for y in 0..32 {
        for x in 0..32 {
            park.apply_terrain_brush(TilePos { x, y }, 0, TerrainBrushKind::Grass)
                .unwrap();
        }
    }
    for y in 3..=12 {
        for x in 3..=11 {
            park.set_terrain_height(TilePos { x, y }, if x < 7 { 3 } else { 2 })
                .unwrap();
        }
    }
    for y in 4..=13 {
        for x in 18..=27 {
            park.apply_terrain_brush(
                TilePos { x, y },
                0,
                if x == 18 || x == 27 || y == 4 || y == 13 {
                    TerrainBrushKind::Sand
                } else {
                    TerrainBrushKind::Water
                },
            )
            .unwrap();
        }
    }
    for x in 3..=28 {
        park.place_path(TilePos { x, y: 18 }, 0).unwrap();
    }
    for y in 14..18 {
        park.place_path(TilePos { x: 12, y }, 0).unwrap();
    }
    for y in [17, 19] {
        for x in 11..=13 {
            if x == 12 && y == 17 {
                continue;
            }
            park.place_path(TilePos { x, y }, 0).unwrap();
        }
    }
    for x in 17..=27 {
        let (height, slope) = match x {
            17 => (0, PathSlope::East),
            18 => (1, PathSlope::East),
            _ => (2, PathSlope::Flat),
        };
        park.place_sloped_path(TilePos { x, y: 23 }, height, slope)
            .unwrap();
    }
    for x in 4..=9 {
        if x == 7 {
            park.place_sloped_path(TilePos { x, y: 8 }, 2, PathSlope::West)
                .unwrap();
        } else {
            park.place_path(TilePos { x, y: 8 }, if x < 7 { 3 } else { 2 })
                .unwrap();
        }
    }
    for (x, y) in [
        (4, 4),
        (6, 5),
        (10, 4),
        (9, 11),
        (4, 11),
        (16, 6),
        (16, 11),
        (28, 15),
        (16, 25),
        (24, 27),
        (28, 27),
        (3, 27),
    ] {
        park.build_scenery(TilePos { x, y }, SceneryKind::Tree)
            .unwrap();
    }
    for (x, y) in [
        (5, 16),
        (7, 16),
        (9, 16),
        (15, 16),
        (20, 16),
        (24, 16),
        (25, 25),
        (27, 25),
    ] {
        park.build_scenery(TilePos { x, y }, SceneryKind::FlowerBed)
            .unwrap();
    }
    let ride = park.place_carousel(TilePos { x: 5, y: 23 }, 0).unwrap();
    park.place_ride_entrance(TilePos { x: 6, y: 22 }, 0, PathDirection::South, ride)
        .unwrap();
    for y in 19..22 {
        park.place_queue_path(TilePos { x: 6, y }, 0, PathSlope::Flat, ride)
            .unwrap();
    }
    park
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, rotation: Option<u8>) {
    *park = fixture();
    view.rotation = rotation.unwrap_or(0);
    view.set_modal(false, false, false, false);
    view.selected = None;
    view.hovered = None;
    view.path_tool.set_mode(PathTool::Inspect);
    view.terrain_custom = true;
    view.grid = false;
    view.pan = world::tile_center(TilePos { x: 16, y: 15 }, 32, view.rotation);
    view.zoom = 1.75;
    view.scene_dirty = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn landscape_inspection_roundtrips_with_real_terrain_paths_and_scenery() {
        let park = fixture();
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
        assert!(
            park.objects()
                .iter()
                .any(|o| o.path_slope != PathSlope::Flat)
        );
        assert!(park.objects().iter().any(|o| o.queue_owner.is_some()));
    }
}
