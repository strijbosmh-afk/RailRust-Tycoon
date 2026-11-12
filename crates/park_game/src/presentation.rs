//! A retained, bounded water overlay. No simulation or static landscape rebuilds.
use super::*;
use crate::display::{Lighting, Weather};
#[derive(Resource)]
pub(crate) struct WorldMaterial(pub Handle<ColorMaterial>);
#[derive(Resource)]
pub(crate) struct Water {
    mesh: Handle<Mesh>,
    elapsed: f32,
    phase: u8,
    rotation: u8,
    mode: world::ViewMode,
    material: Handle<ColorMaterial>,
    ambience: Handle<Mesh>,
    lighting: Lighting,
    weather: Weather,
}
pub(crate) fn setup(
    mut commands: Commands,
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(world::water_mesh(&sim.0, view.rotation, view.view_mode, 0));
    let material = materials.add(ColorMaterial::from(Color::WHITE));
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        NoFrustumCulling,
    ));
    let ambience = meshes.add(world::ambience_mesh(
        &sim.0,
        view.rotation,
        view.view_mode,
        Lighting::Day,
        Weather::Clear,
        0,
    ));
    commands.spawn((
        Mesh2d(ambience.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::WHITE,
            alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
            ..default()
        })),
        NoFrustumCulling,
    ));
    commands.insert_resource(Water {
        mesh,
        elapsed: 0.0,
        phase: 0,
        rotation: view.rotation,
        mode: view.view_mode,
        material,
        ambience,
        lighting: Lighting::Day,
        weather: Weather::Clear,
    });
}
type PresentationResources<'w> = (
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<ColorMaterial>>,
    Option<Res<'w, WorldMaterial>>,
    Option<Res<'w, crate::display::DisplaySettings>>,
    Option<ResMut<'w, ClearColor>>,
);

pub(crate) fn animate(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    time: Res<Time>,
    mut water: ResMut<Water>,
    resources: PresentationResources<'_>,
) {
    let (mut meshes, mut materials, world_material, display, mut background) = resources;
    let lighting = display
        .as_ref()
        .map_or(Lighting::Day, |display| display.value.lighting);
    let weather = display
        .as_ref()
        .map_or(Weather::Clear, |display| display.value.weather);
    let (mut tint, sky) = match lighting {
        Lighting::Day => ([1.0, 1.0, 1.0], Color::srgb_u8(227, 235, 225)),
        Lighting::Sunset => ([1.0, 0.79, 0.62], Color::srgb_u8(99, 75, 97)),
        Lighting::Night => ([0.60, 0.68, 0.90], Color::srgb_u8(13, 24, 46)),
    };
    if weather == Weather::Rain {
        for channel in &mut tint {
            *channel *= 0.90;
        }
    }
    let tint = Color::srgb(tint[0], tint[1], tint[2]);
    for handle in world_material
        .iter()
        .map(|material| &material.0)
        .chain(std::iter::once(&water.material))
    {
        if let Some(material) = materials.get_mut(handle)
            && material.color != tint
        {
            material.color = tint;
        }
    }
    if let Some(background) = background.as_mut()
        && background.0 != sky
    {
        background.0 = sky;
    }
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
        || water.lighting != lighting
        || water.weather != weather
    {
        if let Some(mesh) = meshes.get_mut(&water.mesh) {
            *mesh = world::water_mesh(&sim.0, view.rotation, view.view_mode, water.phase);
        }
        water.rotation = view.rotation;
        water.mode = view.view_mode;
        if let Some(mesh) = meshes.get_mut(&water.ambience) {
            *mesh = world::ambience_mesh(
                &sim.0,
                view.rotation,
                view.view_mode,
                lighting,
                weather,
                water.phase,
            );
        }
        water.lighting = lighting;
        water.weather = weather;
    }
}
