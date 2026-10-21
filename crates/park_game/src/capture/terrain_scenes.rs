//! Reproducible terrain-tool scenes dispatched through production commands.
use super::*;
use park_sim::Terrain;

/// Find an exposed top/cliff point using the production visible-surface picker.
fn visible_point(park: &Park, pos: TilePos, rotation: u8) -> Option<Vec2> {
    let center = world::elevated_tile_center(park, pos, rotation)?;
    let mut offsets = vec![Vec2::ZERO];
    for x in [-24.0, -16.0, 0.0, 16.0, 24.0] {
        for y in [-8.0, 0.0, 8.0] {
            offsets.push(Vec2::new(x, y));
        }
    }
    for down in [16.0, 32.0, 48.0, 64.0, 96.0] {
        offsets.push(Vec2::new(16.0, -8.0 - down));
        offsets.push(Vec2::new(-16.0, -8.0 - down));
    }
    offsets
        .into_iter()
        .map(|offset| center + offset)
        .find(|&point| {
            world::pick_surface(point, park, rotation, ViewMode::Surface)
                == Some(SurfaceId::Terrain(pos))
        })
}

fn fixture(park: &Park, rotation: u8, scene: Scene) -> (Park, TilePos, Vec2) {
    let mut positions: Vec<_> = (0..park.height())
        .flat_map(|y| (0..park.width()).map(move |x| TilePos { x, y }))
        .collect();
    positions.sort_by_key(|pos| (pos.x.abs_diff(15) + pos.y.abs_diff(18), pos.y, pos.x));
    for pos in positions {
        let water = park.tile(pos).unwrap().terrain == Terrain::Water;
        if water != matches!(scene, Scene::TerrainWater) {
            continue;
        }
        let mut candidate = park.clone();
        if matches!(scene, Scene::TerrainBlocked) {
            let Some(path) = park
                .objects()
                .iter()
                .find(|object| object.pos == pos && object.kind == ObjectKind::Path)
            else {
                continue;
            };
            // Put the ground exactly under a path. The next +1 must fail clearance.
            if candidate.set_terrain_height(pos, path.height).is_err() {
                continue;
            }
        } else if !water {
            if park.objects().iter().any(|object| object.pos == pos) {
                continue;
            }
            candidate.set_terrain_height(pos, 4).unwrap();
        }
        if let Some(point) = visible_point(&candidate, pos, rotation) {
            return (candidate, pos, point);
        }
    }
    panic!("terrain capture needs an exposed target");
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    assert!(scene.is_terrain_tool());
    let (candidate, pos, point) = fixture(park, view.rotation, scene);
    *park = candidate;
    let before = park.clone();
    view.terrain_custom = true;
    view.terrain_error = None;
    view.view_mode = if matches!(scene, Scene::TerrainUnderground) {
        ViewMode::Underground
    } else {
        ViewMode::Surface
    };
    view.path_tool = ToolState {
        mode: if matches!(scene, Scene::TerrainLower | Scene::TerrainLowered) {
            PathTool::LowerTerrain
        } else {
            PathTool::RaiseTerrain
        },
        ..default()
    };
    view.selected = None;
    view.hovered = world::pick_surface(point, park, view.rotation, view.view_mode);
    let click = !matches!(scene, Scene::TerrainRaise | Scene::TerrainLower);
    path_tools::point_at(park, view, point, click);
    match scene {
        Scene::TerrainRaised | Scene::TerrainLowered => {
            let expected = if matches!(scene, Scene::TerrainRaised) {
                5
            } else {
                3
            };
            assert_eq!(park.tile(pos).unwrap().height, expected);
            let mut oracle = before.clone();
            oracle.set_terrain_height(pos, expected).unwrap();
            assert_eq!(*park, oracle);
            assert_eq!(view.selected, Some(SurfaceId::Terrain(pos)));
            assert!(view.path_tool.terrain_preview.is_none());
        }
        Scene::TerrainUnderground => {
            assert_eq!(*park, before);
            assert!(view.path_tool.terrain_preview.is_none());
            assert!(view.path_tool.hint.contains("Surface"));
        }
        _ => {
            assert_eq!(*park, before);
            let preview = view
                .path_tool
                .terrain_preview
                .expect("visible terrain ghost");
            assert_eq!(preview.pos, pos);
            assert_eq!(
                preview.valid,
                matches!(scene, Scene::TerrainRaise | Scene::TerrainLower)
            );
        }
    }
    // Keep the target centered even if the committed height changes fit bounds.
    let focus = if view.selected.is_some() {
        world::elevated_tile_center(park, pos, view.rotation).unwrap()
    } else {
        point
    };
    view.zoom = 2.5;
    view.pan = focus - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_capture_scenes_validate_real_commands_across_rotations_and_presets() {
        for rotation in 0..4 {
            for profile in [TerrainProfile::Flat, TerrainProfile::Terraced] {
                let mut initial = Park::new(MAP_SIZE, MAP_SIZE);
                initial.try_apply_terrain_profile(profile).unwrap();
                initial.install_crowd_fixture(crate::CROWD_SEED).unwrap();
                initial.install_train_fixture().unwrap();
                for _ in 0..600 {
                    initial.advance(Speed::Normal);
                }
                for scene in [
                    Scene::TerrainRaise,
                    Scene::TerrainLower,
                    Scene::TerrainRaised,
                    Scene::TerrainLowered,
                    Scene::TerrainBlocked,
                    Scene::TerrainWater,
                    Scene::TerrainUnderground,
                ] {
                    let mut park = initial.clone();
                    let mut view = ViewState {
                        rotation,
                        speed: Speed::Paused,
                        ..default()
                    };
                    prepare(&mut park, &mut view, scene);
                    assert_eq!(park.ticks(), initial.ticks());
                    assert_eq!(park.objects(), initial.objects());
                    assert_eq!(park.guests(), initial.guests());
                    assert_eq!(park.train(), initial.train());
                    assert_eq!(park.circuit(), initial.circuit());
                    assert_eq!(park.path_cache_stats(), initial.path_cache_stats());
                    assert_eq!(
                        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                        park
                    );
                    let window = Window {
                        resolution: bevy::window::WindowResolution::new(1024, 680),
                        ..default()
                    };
                    let focus = view.pan
                        + Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
                    let (scale, center) = crate::camera_layout(&window, &view, &park);
                    let relative = (focus - center) / scale;
                    let cursor = Vec2::new(
                        window.width() * 0.5 + relative.x,
                        window.height() * 0.5 - relative.y,
                    );
                    assert!(crate::is_map_position(cursor, &window, false));
                }
            }
        }
    }
}
