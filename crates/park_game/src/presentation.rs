//! A retained, bounded water overlay. No simulation or static landscape rebuilds.
use super::*;
#[derive(Resource)]
pub(crate) struct Water {
    mesh: Handle<Mesh>,
    elapsed: f32,
    phase: u8,
    rotation: u8,
    mode: world::ViewMode,
}
pub(crate) fn setup(
    mut commands: Commands,
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(world::water_mesh(&sim.0, view.rotation, view.view_mode, 0));
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        NoFrustumCulling,
    ));
    commands.insert_resource(Water {
        mesh,
        elapsed: 0.0,
        phase: 0,
        rotation: view.rotation,
        mode: view.view_mode,
    });
}
pub(crate) fn animate(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    time: Res<Time>,
    mut water: ResMut<Water>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let old = water.phase;
    if view.speed != Speed::Paused && !view.capture_active && view.diagnostic_case.is_none() {
        water.elapsed += time.delta_secs();
        if water.elapsed >= 0.15 {
            water.phase = (water.phase + 1) % 16;
            water.elapsed = 0.0;
        }
    }
    if old != water.phase
        || water.rotation != view.rotation
        || water.mode != view.view_mode
        || view.scene_dirty
    {
        if let Some(mesh) = meshes.get_mut(&water.mesh) {
            *mesh = world::water_mesh(&sim.0, view.rotation, view.view_mode, water.phase);
        }
        water.rotation = view.rotation;
        water.mode = view.view_mode;
    }
}
