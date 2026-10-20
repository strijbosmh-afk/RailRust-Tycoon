//! Retained static meshes with conservative camera eligibility.
//!
//! Bounds contain actual emitted vertices, not tile footprints. This is a CPU
//! selection of whole meshes, not a count of visible pixels or physical draws.

use bevy::{camera::visibility::NoFrustumCulling, prelude::*};
use serde_json::{Value, json};

use crate::{Simulation, ViewState, camera_layout, world};

const VIEWPORT_GUARD: f32 = 2.0;

#[derive(Component)]
pub(super) struct StaticChunkEntity;

struct Chunk {
    mesh: Handle<Mesh>,
    entity: Entity,
    bounds: Rect,
    vertices: usize,
    indices: usize,
    selected: bool,
}

#[derive(Resource)]
pub(super) struct StaticLandscape {
    chunks: Vec<Chunk>,
    material: Handle<ColorMaterial>,
    partitioned: bool,
    last_viewport: Option<Option<Rect>>,
    geometry_builds: u64,
    visibility_evaluations: u64,
    visibility_changes: u64,
}

impl StaticLandscape {
    pub fn new(material: Handle<ColorMaterial>) -> Self {
        Self {
            chunks: Vec::new(),
            material,
            partitioned: false,
            last_viewport: None,
            geometry_builds: 0,
            visibility_evaluations: 0,
            visibility_changes: 0,
        }
    }

    pub fn rebuild(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        park: &park_sim::Park,
        view: &ViewState,
    ) {
        let geometry: Vec<world::StaticChunkGeometry> =
            world::build_static_chunks(park, view.rotation, view.grid, view.view_mode);
        let length = geometry.len();
        for (index, geometry) in geometry.into_iter().enumerate() {
            let vertices = geometry.mesh.count_vertices();
            let indices = geometry.mesh.indices().map_or(0, |indices| indices.len());
            if let Some(slot) = self.chunks.get_mut(index) {
                *meshes
                    .get_mut(&slot.mesh)
                    .expect("static landscape owns its mesh assets") = geometry.mesh;
                slot.bounds = geometry.bounds;
                slot.vertices = vertices;
                slot.indices = indices;
            } else {
                let mesh = meshes.add(geometry.mesh);
                let entity = commands
                    .spawn((
                        Mesh2d(mesh.clone()),
                        MeshMaterial2d(self.material.clone()),
                        StaticChunkEntity,
                        Visibility::Visible,
                        // Eligibility is computed below. Bevy's cached AABB
                        // would be stale when a retained mesh slot is rebuilt.
                        NoFrustumCulling,
                    ))
                    .id();
                self.chunks.push(Chunk {
                    mesh,
                    entity,
                    bounds: geometry.bounds,
                    vertices,
                    indices,
                    selected: true,
                });
            }
        }
        for slot in self.chunks.drain(length..) {
            commands.entity(slot.entity).despawn();
            meshes.remove(slot.mesh.id());
        }
        self.partitioned = park.width() > 32;
        self.last_viewport = None;
        self.geometry_builds += 1;
    }

    pub fn total_counts(&self) -> (usize, usize) {
        self.chunks
            .iter()
            .fold((0, 0), |(vertices, indices), chunk| {
                (vertices + chunk.vertices, indices + chunk.indices)
            })
    }

    #[cfg(test)]
    pub fn mesh_handles(&self) -> impl Iterator<Item = &Handle<Mesh>> {
        self.chunks.iter().map(|chunk| &chunk.mesh)
    }

    fn select(
        &mut self,
        viewport: Option<Rect>,
        visibility: &mut Query<&mut Visibility, With<StaticChunkEntity>>,
    ) {
        if self.last_viewport == Some(viewport) {
            return;
        }
        self.visibility_evaluations += 1;
        for slot in &mut self.chunks {
            let selected = viewport.is_none_or(|viewport| intersects(slot.bounds, viewport));
            if slot.selected != selected {
                *visibility
                    .get_mut(slot.entity)
                    .expect("static entities are applied before camera eligibility") = if selected {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                slot.selected = selected;
                self.visibility_changes += 1;
            }
        }
        self.last_viewport = Some(viewport);
    }

    pub fn report(&self) -> Value {
        let (vertices, indices) = self.total_counts();
        let selected: Vec<_> = self.chunks.iter().filter(|chunk| chunk.selected).collect();
        let selected_vertices: usize = selected.iter().map(|chunk| chunk.vertices).sum();
        let selected_indices: usize = selected.iter().map(|chunk| chunk.indices).sum();
        let viewport = self.last_viewport.flatten();
        json!({
            "method":"conservative_world_xy_aabb",
            "chunking":if self.partitioned { "spatial_triangles" } else { "single_mesh" },
            "bucket_world_size":[1024.0,512.0],
            "selection_state":if viewport.is_some() { "camera_bounds" } else { "unavailable_all_selected" },
            "viewport_world":viewport.map(|bounds| json!({"min":bounds.min.to_array(),"max":bounds.max.to_array()})),
            "viewport_guard_logical_pixels":VIEWPORT_GUARD,
            "total_chunks":self.chunks.len(),
            "selected_chunks":selected.len(),
            "hidden_chunks":self.chunks.len()-selected.len(),
            "generated":{"vertices":vertices,"indices":indices,"logical_cpu_payload_bytes":vertices*48+indices*4},
            "selected":{"vertices":selected_vertices,"indices":selected_indices,
                "logical_cpu_payload_bytes":selected_vertices*48+selected_indices*4},
            "geometry_builds":self.geometry_builds,
            "visibility_evaluations":self.visibility_evaluations,
            "visibility_changes":self.visibility_changes,
            "scope":"Final snapshot of whole static mesh chunks selected on the CPU for render eligibility, including geometry behind UI and inside a conservative guard. Counters are cumulative since initialization. Not actual draw calls, visible pixels, occlusion, GPU allocation/upload traffic, or a performance guarantee. Dynamic guest/train/selection meshes are outside these counts.",
        })
    }
}

fn intersects(a: Rect, b: Rect) -> bool {
    a.min.x <= b.max.x && a.max.x >= b.min.x && a.min.y <= b.max.y && a.max.y >= b.min.y
}

fn viewport(window: &Window, view: &ViewState, park: &park_sim::Park) -> Option<Rect> {
    let size = Vec2::new(window.width(), window.height());
    let (scale, center) = camera_layout(window, view, park);
    if !size.is_finite()
        || size.min_element() <= 0.0
        || !scale.is_finite()
        || scale <= 0.0
        || !center.is_finite()
    {
        return None;
    }
    // Use the complete render target, including the area underneath the UI.
    // Logical pixels follow camera_layout, so physical DPI is not applied twice.
    let half = (size * 0.5 + Vec2::splat(VIEWPORT_GUARD)) * scale;
    Some(Rect::from_corners(center - half, center + half))
}

pub(super) fn update_visibility(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut landscape: ResMut<StaticLandscape>,
    mut visibility: Query<&mut Visibility, With<StaticChunkEntity>>,
) {
    let bounds = windows
        .single()
        .ok()
        .and_then(|window| viewport(window, &view, &sim.0));
    landscape.select(bounds, &mut visibility);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_scene::{CameraPreset, RenderScene};
    use park_sim::diagnostics::DiagnosticCase;

    fn app() -> (App, Entity) {
        let render = RenderScene {
            case: DiagnosticCase::SaveScale,
            camera: CameraPreset::Crowd,
            rotation: 0,
            underground: false,
        };
        let park = render.build().unwrap();
        let mut view = ViewState::default();
        render.apply_view(&park, &mut view);
        let mut app = App::new();
        app.insert_resource(Simulation(park))
            .insert_resource(view)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_systems(Startup, crate::setup_scene)
            .add_systems(
                Update,
                (crate::apply_camera, crate::update_scene, update_visibility).chain(),
            );
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: bevy::window::WindowResolution::new(1280, 800),
                    ..default()
                },
                bevy::window::PrimaryWindow,
            ))
            .id();
        app.update();
        (app, window)
    }

    fn slots(app: &App) -> Vec<(Entity, bevy::asset::AssetId<Mesh>)> {
        app.world()
            .resource::<StaticLandscape>()
            .chunks
            .iter()
            .map(|c| (c.entity, c.mesh.id()))
            .collect()
    }

    fn assert_entity_visibility(app: &App) {
        let landscape = app.world().resource::<StaticLandscape>();
        for chunk in &landscape.chunks {
            assert_eq!(
                *app.world().get::<Visibility>(chunk.entity).unwrap(),
                if chunk.selected {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }
            );
        }
        let report = landscape.report();
        assert_eq!(
            report["selected_chunks"].as_u64().unwrap() + report["hidden_chunks"].as_u64().unwrap(),
            report["total_chunks"].as_u64().unwrap()
        );
    }

    #[test]
    fn camera_motion_resize_and_dpi_select_retained_meshes_without_rebuilds() {
        let (mut app, window) = app();
        let initial = slots(&app);
        let landscape = app.world().resource::<StaticLandscape>();
        let builds = landscape.geometry_builds;
        assert_eq!(builds, 1);
        assert!(landscape.chunks.len() > 1);
        let initial_report = landscape.report();
        assert!(
            initial_report["selected"]["vertices"].as_u64().unwrap()
                < initial_report["generated"]["vertices"].as_u64().unwrap() / 2
        );
        let assets = app.world().resource::<Assets<Mesh>>().len();
        let guest_stats = app
            .world()
            .resource::<crate::SceneMeshes>()
            .guest_renderer
            .stats();
        for (pan, zoom, width, height, dpi) in [
            (Vec2::ZERO, 1.0, 1280, 800, 1.0),
            (Vec2::new(6000.0, 0.0), 12.8, 1024, 680, 1.0),
            (Vec2::new(-6000.0, 200.0), 18.0, 2048, 1360, 2.0),
            (Vec2::new(0.0, 3600.0), 10.0, 1280, 800, 1.0),
            (Vec2::new(0.0, -3600.0), 10.0, 1280, 800, 1.0),
            (Vec2::splat(1_000_000.0), 12.8, 1280, 800, 1.0),
        ] {
            {
                let mut view = app.world_mut().resource_mut::<ViewState>();
                view.pan = pan;
                view.zoom = zoom;
            }
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .resolution =
                bevy::window::WindowResolution::new(width, height).with_scale_factor_override(dpi);
            app.update();
            assert_entity_visibility(&app);
            assert_eq!(slots(&app), initial);
            assert_eq!(
                app.world().resource::<StaticLandscape>().geometry_builds,
                builds
            );
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), assets);
            assert_eq!(
                app.world()
                    .resource::<crate::SceneMeshes>()
                    .guest_renderer
                    .stats(),
                guest_stats
            );
            if zoom == 1.0 {
                let r = app.world().resource::<StaticLandscape>().report();
                assert_eq!(r["selected_chunks"], r["total_chunks"]);
            }
        }
        let report = app.world().resource::<StaticLandscape>().report();
        assert_eq!(report["selected_chunks"], 0);
        assert_eq!(report["selected"]["vertices"], 0);
        app.update();
        assert_eq!(app.world().resource::<StaticLandscape>().report(), report);
        // No usable window must restore all slots rather than retain a stale
        // hidden set. This is also the honest headless diagnostics fallback.
        app.world_mut().despawn(window);
        app.update();
        let report = app.world().resource::<StaticLandscape>().report();
        assert_eq!(report["selection_state"], "unavailable_all_selected");
        assert_eq!(report["selected_chunks"], report["total_chunks"]);
        assert_entity_visibility(&app);
    }

    #[test]
    fn replacement_maps_reuse_slots_remove_excess_assets_and_refresh_same_tick() {
        let (mut app, _) = app();
        let original = slots(&app);
        app.world_mut().resource_mut::<Simulation>().0 = crate::demonstration_park();
        *app.world_mut().resource_mut::<ViewState>() = ViewState {
            scene_dirty: true,
            ..default()
        };
        app.update();
        let small = slots(&app);
        assert_eq!(small, vec![original[0]]);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 4);
        for (entity, asset) in original.iter().skip(1) {
            assert!(app.world().get_entity(*entity).is_err());
            assert!(app.world().resource::<Assets<Mesh>>().get(*asset).is_none());
        }
        let render = RenderScene {
            case: DiagnosticCase::SaveScale,
            camera: CameraPreset::Ride,
            rotation: 3,
            underground: true,
        };
        let mut park = render.build().unwrap();
        park.set_terrain_height(park_sim::TilePos { x: 255, y: 255 }, 32)
            .unwrap();
        let expected =
            world::build_scene(&park, 3, false, world::ViewMode::Underground).count_vertices();
        let mut view = ViewState::default();
        render.apply_view(&park, &mut view);
        view.grid = false;
        app.world_mut().resource_mut::<Simulation>().0 = park;
        *app.world_mut().resource_mut::<ViewState>() = view;
        app.update();
        let large = slots(&app);
        assert!(large.len() > 1);
        assert_eq!(large[0], original[0]);
        assert_eq!(
            app.world().resource::<Assets<Mesh>>().len(),
            large.len() + 3
        );
        assert_eq!(
            app.world().resource::<StaticLandscape>().total_counts().0,
            expected
        );
        assert_entity_visibility(&app);
        assert_eq!(app.world().resource::<StaticLandscape>().geometry_builds, 3);
        let report = app.world().resource::<StaticLandscape>().report();
        assert!(
            report["selected_chunks"].as_u64().unwrap() < report["total_chunks"].as_u64().unwrap()
        );
    }

    #[test]
    fn intersections_include_touches_and_bounds_outside_nominal_tile_footprints() {
        let viewport = Rect::from_corners(Vec2::ZERO, Vec2::splat(100.0));
        assert!(intersects(
            viewport,
            Rect::from_corners(Vec2::splat(100.0), Vec2::splat(140.0))
        ));
        assert!(intersects(
            viewport,
            Rect::from_corners(Vec2::new(-20.0, 90.0), Vec2::new(20.0, 170.0))
        ));
        assert!(!intersects(
            viewport,
            Rect::from_corners(Vec2::splat(100.1), Vec2::splat(140.0))
        ));
    }

    #[test]
    fn viewport_matches_full_camera_target_with_dpi_and_guard() {
        let park = park_sim::Park::new(256, 256);
        let view = ViewState {
            zoom: 12.8,
            pan: Vec2::new(600.0, -700.0),
            ..default()
        };
        let mut previous = None;
        for dpi in [1.0, 2.0] {
            let window = Window {
                resolution: bevy::window::WindowResolution::new(
                    (1280.0 * dpi) as u32,
                    (800.0 * dpi) as u32,
                )
                .with_scale_factor_override(dpi),
                ..default()
            };
            let bounds = viewport(&window, &view, &park).unwrap();
            let (scale, center) = camera_layout(&window, &view, &park);
            assert!((bounds.center() - center).abs().max_element() < 0.001);
            assert!((bounds.width() - 1284.0 * scale).abs() < 0.01);
            assert!((bounds.height() - 804.0 * scale).abs() < 0.01);
            if let Some(old) = previous {
                assert_eq!(bounds, old);
            }
            previous = Some(bounds);
        }
        let window = Window {
            resolution: bevy::window::WindowResolution::new(0, 0),
            ..default()
        };
        assert!(viewport(&window, &view, &park).is_none());
    }
}
