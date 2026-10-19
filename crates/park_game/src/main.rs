#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod ui;
mod world;

use bevy::{
    camera::visibility::NoFrustumCulling,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::{PresentMode, PrimaryWindow, WindowResolution},
};
use park_sim::{Park, Speed, TICKS_PER_SECOND, TilePos};

const MAP_SIZE: u32 = 32;
const SIDEBAR: f32 = 248.0;
const HEADER: f32 = 76.0;
const FOOTER: f32 = 36.0;

#[derive(Resource)]
struct Simulation(Park);

#[derive(Resource)]
struct ViewState {
    rotation: u8,
    zoom: f32,
    pan: Vec2,
    grid: bool,
    speed: Speed,
    resume_speed: Speed,
    selected: Option<TilePos>,
    hovered: Option<TilePos>,
    help: bool,
    dragging: bool,
    ui_pointer_consumed: bool,
    scene_dirty: bool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            rotation: 0,
            zoom: 1.0,
            pan: Vec2::ZERO,
            grid: true,
            speed: Speed::Normal,
            resume_speed: Speed::Normal,
            selected: None,
            hovered: None,
            help: false,
            dragging: false,
            ui_pointer_consumed: false,
            scene_dirty: false,
        }
    }
}

impl ViewState {
    fn rotate(&mut self, delta: u8) {
        self.rotation = (self.rotation + delta) % 4;
        self.scene_dirty = true;
    }

    fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
        if speed != Speed::Paused {
            self.resume_speed = speed;
        }
    }

    fn pause(&mut self) {
        let speed = if self.speed == Speed::Paused {
            self.resume_speed
        } else {
            Speed::Paused
        };
        self.set_speed(speed);
    }

    fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.6, 3.5);
    }

    fn reset_camera(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
    }
}

#[derive(Resource)]
struct SceneMeshes {
    landscape: Handle<Mesh>,
    selection: Handle<Mesh>,
}

fn main() -> AppExit {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb_u8(227, 235, 225)))
        .insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND as f64))
        .insert_resource(Simulation(Park::new(MAP_SIZE, MAP_SIZE)))
        .init_resource::<ViewState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Parkbound - Meadow Lake".into(),
                resolution: WindowResolution::new(1280, 800),
                resize_constraints: bevy::window::WindowResizeConstraints {
                    min_width: 1024.0,
                    min_height: 680.0,
                    ..default()
                },
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_scene, ui::setup))
        .add_systems(FixedUpdate, tick_simulation)
        .add_systems(
            Update,
            (
                ui::button_actions,
                keyboard_input,
                pointer_input,
                apply_camera,
                update_scene,
                ui::refresh,
            )
                .chain(),
        );
    capture::configure(&mut app);
    app.run()
}

fn setup_scene(
    mut commands: Commands,
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    let landscape = meshes.add(world::build_scene(&sim.0, view.rotation, view.grid));
    let selection = meshes.add(world::selection_mesh(None, None, MAP_SIZE, view.rotation));
    let material = materials.add(ColorMaterial::from(Color::WHITE));
    commands.spawn((Mesh2d(landscape.clone()), MeshMaterial2d(material.clone())));
    // Its vertices move to different tiles; a cached initial bounding box
    // would incorrectly hide the highlight after panning away from that tile.
    commands.spawn((
        Mesh2d(selection.clone()),
        MeshMaterial2d(material),
        NoFrustumCulling,
    ));
    commands.insert_resource(SceneMeshes {
        landscape,
        selection,
    });
}

fn tick_simulation(mut sim: ResMut<Simulation>, view: Res<ViewState>) {
    sim.0.advance(view.speed);
}

fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut view: ResMut<ViewState>) {
    if keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::KeyH) {
        view.help = !view.help;
    }
    if keys.just_pressed(KeyCode::Escape) {
        if view.help {
            view.help = false;
        } else {
            view.selected = None;
        }
    }
    if view.help {
        return;
    }
    if keys.just_pressed(KeyCode::Space) {
        view.pause();
    }
    for (key, speed) in [
        (KeyCode::Digit1, Speed::Normal),
        (KeyCode::Digit2, Speed::Fast),
        (KeyCode::Digit3, Speed::Fastest),
    ] {
        if keys.just_pressed(key) {
            view.set_speed(speed);
        }
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        view.rotate(3);
    }
    if keys.just_pressed(KeyCode::KeyE) {
        view.rotate(1);
    }
    if keys.just_pressed(KeyCode::KeyG) {
        view.grid = !view.grid;
        view.scene_dirty = true;
    }
    if keys.just_pressed(KeyCode::Home) {
        view.reset_camera();
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        view.zoom_by(1.2);
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        view.zoom_by(1.0 / 1.2);
    }
    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    let zoom = view.zoom;
    view.pan += direction.normalize_or_zero() * time.delta_secs().min(0.1) * 700.0 / zoom;
    view.pan = view.pan.clamp(Vec2::splat(-1800.0), Vec2::splat(1800.0));
}

/// This formula uses logical pixels, just like cursor_position and the UI.
fn camera_layout(window: &Window, view: &ViewState) -> (f32, Vec2) {
    let available_width = (window.width() - SIDEBAR - 72.0).max(1.0);
    let available_height = (window.height() - HEADER - FOOTER - 100.0).max(1.0);
    let fit = (MAP_SIZE as f32 * world::TILE_WIDTH / available_width)
        .max((MAP_SIZE as f32 * world::TILE_HEIGHT + 70.0) / available_height);
    let scale = fit / view.zoom;
    let center = view.pan + Vec2::new(-SIDEBAR * 0.5, (HEADER - FOOTER) * 0.5) * scale;
    (scale, center)
}

fn is_map_position(cursor: Vec2, window: &Window, help: bool) -> bool {
    !help && cursor.x > SIDEBAR && cursor.y > HEADER && cursor.y < window.height() - FOOTER
}

fn pointer_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    mut view: ResMut<ViewState>,
    mut previous_cursor: Local<Option<Vec2>>,
) {
    let scroll: f32 = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 60.0,
        })
        .sum();
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused {
        view.dragging = false;
        view.hovered = None;
        *previous_cursor = None;
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        view.hovered = None;
        view.dragging = false;
        *previous_cursor = None;
        return;
    };
    // Cursor positions are logical pixels. Raw MouseMotion counts are not,
    // and would make dragging dependent on DPI and device acceleration.
    let movement = previous_cursor.map_or(Vec2::ZERO, |previous| cursor - previous);
    *previous_cursor = Some(cursor);
    let over_map = is_map_position(cursor, window, view.help || view.ui_pointer_consumed);
    let starting_drag = over_map
        && (buttons.just_pressed(MouseButton::Right) || buttons.just_pressed(MouseButton::Middle));
    if starting_drag {
        view.dragging = true;
    }
    if view.help || (!buttons.pressed(MouseButton::Right) && !buttons.pressed(MouseButton::Middle))
    {
        view.dragging = false;
    }
    if view.dragging && !starting_drag {
        let (scale, _) = camera_layout(window, &view);
        view.pan += Vec2::new(-movement.x, movement.y) * scale;
        view.pan = view.pan.clamp(Vec2::splat(-1800.0), Vec2::splat(1800.0));
    }
    if over_map && scroll != 0.0 {
        view.zoom_by((scroll.clamp(-5.0, 5.0) * 0.12).exp());
    }
    view.hovered = if over_map {
        let (scale, center) = camera_layout(window, &view);
        let world = center
            + Vec2::new(
                cursor.x - window.width() * 0.5,
                window.height() * 0.5 - cursor.y,
            ) * scale;
        world::pick_tile(world, MAP_SIZE, view.rotation)
    } else {
        None
    };
    if over_map && buttons.just_pressed(MouseButton::Left) {
        view.selected = view.hovered;
    }
}

fn apply_camera(
    windows: Query<&Window, With<PrimaryWindow>>,
    view: Res<ViewState>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut transform, mut projection)) = cameras.single_mut() else {
        return;
    };
    let (scale, center) = camera_layout(window, &view);
    transform.translation.x = center.x;
    transform.translation.y = center.y;
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = scale;
    }
}

type SelectionSnapshot = (Option<TilePos>, Option<TilePos>, u8);

fn update_scene(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    scene: Res<SceneMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut last_selection: Local<Option<SelectionSnapshot>>,
) {
    if view.scene_dirty {
        if let Some(mesh) = meshes.get_mut(&scene.landscape) {
            *mesh = world::build_scene(&sim.0, view.rotation, view.grid);
        }
        view.scene_dirty = false;
    }
    let selection = (view.hovered, view.selected, view.rotation);
    if *last_selection != Some(selection) {
        if let Some(mesh) = meshes.get_mut(&scene.selection) {
            *mesh = world::selection_mesh(view.hovered, view.selected, MAP_SIZE, view.rotation);
        }
        *last_selection = Some(selection);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_restores_the_previous_speed() {
        let mut view = ViewState::default();
        view.set_speed(Speed::Fastest);
        view.pause();
        assert_eq!(view.speed, Speed::Paused);
        view.pause();
        assert_eq!(view.speed, Speed::Fastest);
    }

    #[test]
    fn camera_controls_stay_bounded_and_reset_without_changing_selection() {
        let mut view = ViewState {
            selected: Some(TilePos { x: 3, y: 7 }),
            ..default()
        };
        view.zoom_by(1000.0);
        assert_eq!(view.zoom, 3.5);
        view.zoom_by(0.0001);
        assert_eq!(view.zoom, 0.6);
        for _ in 0..4 {
            view.rotate(1);
        }
        assert_eq!(view.rotation, 0);
        view.reset_camera();
        assert_eq!(view.zoom, 1.0);
        assert_eq!(view.selected, Some(TilePos { x: 3, y: 7 }));
    }

    #[test]
    fn toolbar_and_help_cannot_select_the_world() {
        let window = Window {
            resolution: WindowResolution::new(1280, 800),
            ..default()
        };
        assert!(!is_map_position(Vec2::new(20.0, 400.0), &window, false));
        assert!(!is_map_position(Vec2::new(600.0, 20.0), &window, false));
        assert!(!is_map_position(Vec2::new(600.0, 790.0), &window, false));
        assert!(!is_map_position(Vec2::new(600.0, 400.0), &window, true));
        assert!(is_map_position(Vec2::new(600.0, 400.0), &window, false));
    }

    #[test]
    fn closing_help_consumes_the_click_for_exactly_one_frame() {
        let mut app = App::new();
        app.insert_resource(ViewState {
            help: true,
            selected: Some(TilePos { x: 3, y: 7 }),
            ..default()
        })
        .add_systems(Update, ui::button_actions);
        app.world_mut()
            .spawn((Button, Interaction::Pressed, ui::Action::CloseHelp));
        app.update();
        let view = app.world().resource::<ViewState>();
        assert!(!view.help);
        assert!(view.ui_pointer_consumed);
        assert_eq!(view.selected, Some(TilePos { x: 3, y: 7 }));
        app.update();
        assert!(!app.world().resource::<ViewState>().ui_pointer_consumed);
    }
}
