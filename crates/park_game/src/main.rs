#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod benchmark;
mod capture;
mod frame_probe;
#[cfg(test)]
mod input_tests;
mod measurement;
mod persistence;
mod save_check;
mod save_store;
mod ui;
mod world;

use bevy::{
    camera::visibility::NoFrustumCulling,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::{PresentMode, PrimaryWindow, WindowResolution},
};
#[cfg(test)]
use park_sim::TilePos;
use park_sim::{ObjectKind, Park, Speed, SurfaceId, TICKS_PER_SECOND, TerrainProfile};
use persistence::{SaveAction, SaveSession};

const MAP_SIZE: u32 = 32;
const SIDEBAR: f32 = 248.0;
const HEADER: f32 = 76.0;
const FOOTER: f32 = 36.0;
const CROWD_SEED: u64 = 20261020;
const RIDE_PANEL_WIDTH: f32 = 236.0;
const RIDE_PANEL_HEIGHT: f32 = 152.0;
const RIDE_PANEL_MARGIN: f32 = 12.0;

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
    selected: Option<SurfaceId>,
    hovered: Option<SurfaceId>,
    view_mode: world::ViewMode,
    help: bool,
    dragging: bool,
    ui_pointer_consumed: bool,
    scene_dirty: bool,
    terrain_profile: TerrainProfile,
    terrain_custom: bool,
    terrain_change_pending: bool,
    terrain_error: Option<String>,
    capture_active: bool,
    remove_path_pending: bool,
    restore_paths_pending: bool,
    path_notice: Option<String>,
    toggle_train_pending: bool,
    focus_train_pending: bool,
    save_pending: Option<SaveAction>,
    save_notice: String,
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
            view_mode: world::ViewMode::Surface,
            help: false,
            dragging: false,
            ui_pointer_consumed: false,
            scene_dirty: false,
            terrain_profile: TerrainProfile::Terraced,
            terrain_custom: false,
            terrain_change_pending: false,
            terrain_error: None,
            capture_active: false,
            remove_path_pending: false,
            restore_paths_pending: false,
            path_notice: None,
            toggle_train_pending: false,
            focus_train_pending: false,
            save_pending: None,
            save_notice: "F5 save / F9 load\nShift+F9 loads backup".into(),
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

    fn toggle_terrain_profile(&mut self) {
        self.terrain_profile = match self.terrain_profile {
            TerrainProfile::Flat => TerrainProfile::Terraced,
            TerrainProfile::Terraced => TerrainProfile::Flat,
        };
        self.terrain_change_pending = true;
    }

    fn toggle_underground(&mut self) {
        self.view_mode = match self.view_mode {
            world::ViewMode::Surface => world::ViewMode::Underground,
            world::ViewMode::Underground => world::ViewMode::Surface,
        };
        self.hovered = None;
        self.scene_dirty = true;
    }

    fn request_save(&mut self, action: SaveAction) {
        // Loading wins over saving if multiple inputs arrive in one frame.
        // A combined press must never overwrite the file it intends to load.
        if self.save_pending.is_none_or(|pending| action >= pending) {
            self.save_pending = Some(action);
        }
    }
}

#[derive(Resource)]
struct SceneMeshes {
    landscape: Handle<Mesh>,
    selection: Handle<Mesh>,
    guests: Handle<Mesh>,
    train: Handle<Mesh>,
}

fn main() -> AppExit {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if let Err(error) = measurement::validate_modes(&arguments) {
        eprintln!("{error}");
        return AppExit::error();
    }
    if let Some(result) = benchmark::run(&arguments) {
        return result;
    }
    if let Some(result) = save_check::run(arguments) {
        return result;
    }
    let park = demonstration_park();
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb_u8(227, 235, 225)))
        .insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND as f64))
        .insert_resource(Simulation(park))
        .init_resource::<ViewState>()
        .insert_resource(SaveSession::from_arguments(std::env::args_os().skip(1)))
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
                apply_terrain_profile,
                apply_path_actions,
                apply_train_actions,
                persistence::apply_save_action,
                pointer_input,
                apply_camera,
                update_scene,
                ui::refresh,
            )
                .chain(),
        );
    capture::configure(&mut app);
    if let Err(error) = frame_probe::configure(&mut app) {
        eprintln!("Frame profiling failed: {error}");
        return AppExit::error();
    }
    app.run()
}

fn demonstration_park() -> Park {
    let mut park = Park::new(MAP_SIZE, MAP_SIZE);
    park.apply_terrain_profile(TerrainProfile::Terraced);
    park.install_crowd_fixture(CROWD_SEED)
        .expect("the demonstration fixtures must fit the initial park");
    park.install_train_fixture()
        .expect("the demonstration train circuit must fit the initial park");
    park
}

fn setup_scene(
    mut commands: Commands,
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    let landscape = meshes.add(world::build_scene(
        &sim.0,
        view.rotation,
        view.grid,
        view.view_mode,
    ));
    let selection = meshes.add(world::selection_mesh(
        None,
        None,
        &sim.0,
        view.rotation,
        view.view_mode,
    ));
    let guests = meshes.add(world::guest_mesh(&sim.0, view.rotation, view.view_mode));
    let train = meshes.add(world::train_mesh(&sim.0, view.rotation, view.view_mode));
    let material = materials.add(ColorMaterial::from(Color::WHITE));
    commands.spawn((
        Mesh2d(landscape.clone()),
        MeshMaterial2d(material.clone()),
        NoFrustumCulling,
    ));
    // Its vertices move to different tiles; a cached initial bounding box
    // would incorrectly hide the highlight after panning away from that tile.
    commands.spawn((
        Mesh2d(selection.clone()),
        MeshMaterial2d(material.clone()),
        NoFrustumCulling,
    ));
    commands.spawn((
        Mesh2d(guests.clone()),
        MeshMaterial2d(material.clone()),
        NoFrustumCulling,
    ));
    commands.spawn((
        Mesh2d(train.clone()),
        MeshMaterial2d(material),
        NoFrustumCulling,
    ));
    commands.insert_resource(SceneMeshes {
        landscape,
        selection,
        guests,
        train,
    });
}

fn tick_simulation(
    mut sim: ResMut<Simulation>,
    view: Res<ViewState>,
    mut probe: Option<ResMut<frame_probe::Probe>>,
) {
    let timer = probe
        .as_ref()
        .filter(|p| p.is_measuring())
        .map(|_| std::time::Instant::now());
    let before = sim.0.ticks();
    if view.speed != Speed::Paused {
        sim.0.advance(view.speed);
    }
    if let (Some(start), Some(probe)) = (timer, probe.as_mut()) {
        probe.record_simulation(start.elapsed(), sim.0.ticks() - before);
    }
}

fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut view: ResMut<ViewState>) {
    if view.capture_active {
        return;
    }
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
    if keys.just_pressed(KeyCode::KeyT) {
        view.toggle_terrain_profile();
    }
    if keys.just_pressed(KeyCode::KeyU) {
        view.toggle_underground();
    }
    if keys.just_pressed(KeyCode::Delete) {
        view.remove_path_pending = true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        view.restore_paths_pending = true;
    }
    if keys.just_pressed(KeyCode::KeyO) {
        view.toggle_train_pending = true;
    }
    if keys.just_pressed(KeyCode::KeyF) {
        view.focus_train_pending = true;
    }
    if keys.just_pressed(KeyCode::F5) {
        view.request_save(SaveAction::Save);
    }
    if keys.just_pressed(KeyCode::F9) {
        let backup = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        view.request_save(if backup {
            SaveAction::Backup
        } else {
            SaveAction::Load
        });
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

fn apply_terrain_profile(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    if view.terrain_change_pending {
        if let Err(error) = sim.0.try_apply_terrain_profile(view.terrain_profile) {
            view.terrain_error = Some(format!("Terrain unchanged: {error}"));
            view.terrain_profile = match view.terrain_profile {
                TerrainProfile::Flat => TerrainProfile::Terraced,
                TerrainProfile::Terraced => TerrainProfile::Flat,
            };
        } else {
            view.terrain_error = None;
            view.terrain_custom = false;
        }
        view.terrain_change_pending = false;
        view.hovered = None;
        view.scene_dirty = true;
    }
}

fn apply_path_actions(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    if view.remove_path_pending {
        view.remove_path_pending = false;
        let selected_path = match view.selected {
            Some(SurfaceId::Object(id))
                if sim.0.object(id).is_some_and(|o| o.kind == ObjectKind::Path) =>
            {
                Some(id)
            }
            _ => None,
        };
        if let Some(id) = selected_path {
            sim.0.remove_object(id);
            view.selected = None;
            view.hovered = None;
            view.scene_dirty = true;
            view.path_notice = Some("Path removed. R restores the walkway.".into());
        } else {
            view.path_notice = Some("Select a path to remove it.".into());
        }
    }
    if view.restore_paths_pending {
        view.restore_paths_pending = false;
        match sim.0.restore_crowd_paths() {
            Ok(()) => {
                view.scene_dirty = true;
                view.hovered = None;
                view.path_notice = Some("Walkway restored. Guests can find new routes.".into());
            }
            Err(error) => {
                warn!("Cannot restore walkway: {error}");
                view.path_notice = Some("Cannot restore: a path space is occupied.".into());
            }
        }
    }
}

fn focus_train(park: &Park, view: &mut ViewState) {
    if let Some(center) = world::train_focus(park, view.rotation) {
        view.zoom = 1.5;
        view.pan =
            center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
        view.hovered = None;
    }
}

fn apply_train_actions(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    if view.toggle_train_pending {
        view.toggle_train_pending = false;
        if let Some(train) = sim.0.train() {
            let open = !train.open;
            sim.0.set_train_open(open);
        }
    }
    if view.focus_train_pending {
        view.focus_train_pending = false;
        focus_train(&sim.0, &mut view);
    }
}

/// This formula uses logical pixels, just like cursor_position and the UI.
fn camera_layout(window: &Window, view: &ViewState, park: &Park) -> (f32, Vec2) {
    let available_width = (window.width() - SIDEBAR - 72.0).max(1.0);
    let available_height = (window.height() - HEADER - FOOTER - 100.0).max(1.0);
    let elevation = f32::from(park.max_surface_height()) * world::HEIGHT_STEP;
    let fit = (park.width() as f32 * world::TILE_WIDTH / available_width)
        .max((park.height() as f32 * world::TILE_HEIGHT + elevation + 70.0) / available_height);
    let scale = fit / view.zoom;
    let center = view.pan
        + Vec2::Y * elevation * 0.5
        + Vec2::new(-SIDEBAR * 0.5, (HEADER - FOOTER) * 0.5) * scale;
    (scale, center)
}

fn is_map_position(cursor: Vec2, window: &Window, help: bool) -> bool {
    let over_ride_panel = cursor.x >= window.width() - RIDE_PANEL_MARGIN - RIDE_PANEL_WIDTH
        && cursor.x <= window.width() - RIDE_PANEL_MARGIN
        && cursor.y >= HEADER + RIDE_PANEL_MARGIN
        && cursor.y <= HEADER + RIDE_PANEL_MARGIN + RIDE_PANEL_HEIGHT;
    !help
        && !over_ride_panel
        && cursor.x > SIDEBAR
        && cursor.x < window.width()
        && cursor.y > HEADER
        && cursor.y < window.height() - FOOTER
}

fn pointer_input(
    sim: Res<Simulation>,
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
    if view.capture_active {
        view.hovered = None;
        *previous_cursor = None;
        return;
    }
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
        let (scale, _) = camera_layout(window, &view, &sim.0);
        view.pan += Vec2::new(-movement.x, movement.y) * scale;
        view.pan = view.pan.clamp(Vec2::splat(-1800.0), Vec2::splat(1800.0));
    }
    if over_map && scroll != 0.0 {
        view.zoom_by((scroll.clamp(-5.0, 5.0) * 0.12).exp());
    }
    view.hovered = if over_map {
        let (scale, center) = camera_layout(window, &view, &sim.0);
        let world = center
            + Vec2::new(
                cursor.x - window.width() * 0.5,
                window.height() * 0.5 - cursor.y,
            ) * scale;
        world::pick_surface(world, &sim.0, view.rotation, view.view_mode)
    } else {
        None
    };
    if over_map && buttons.just_pressed(MouseButton::Left) {
        view.selected = view.hovered;
    }
}

fn apply_camera(
    sim: Res<Simulation>,
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
    let (scale, center) = camera_layout(window, &view, &sim.0);
    transform.translation.x = center.x;
    transform.translation.y = center.y;
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = scale;
    }
}

type SelectionSnapshot = (Option<SurfaceId>, Option<SurfaceId>, u8, world::ViewMode);

fn update_scene(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    scene: Res<SceneMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut last_selection: Local<Option<SelectionSnapshot>>,
    mut last_guests: Local<Option<(u64, u8, world::ViewMode)>>,
    mut probe: Option<ResMut<frame_probe::Probe>>,
) {
    let timer = probe
        .as_ref()
        .filter(|p| p.is_measuring())
        .map(|_| std::time::Instant::now());
    // Removed objects and invalid references must never leave a stale inspector
    // or outline. Valid hidden selections persist when switching view modes.
    view.selected = view
        .selected
        .filter(|id| sim.0.surface_position(*id).is_some());
    view.hovered = view
        .hovered
        .filter(|id| sim.0.surface_position(*id).is_some());
    let terrain_changed = view.scene_dirty;
    if terrain_changed {
        if let Some(mesh) = meshes.get_mut(&scene.landscape) {
            *mesh = world::build_scene(&sim.0, view.rotation, view.grid, view.view_mode);
        }
        view.scene_dirty = false;
    }
    let selection = (view.hovered, view.selected, view.rotation, view.view_mode);
    if terrain_changed || *last_selection != Some(selection) {
        if let Some(mesh) = meshes.get_mut(&scene.selection) {
            *mesh = world::selection_mesh(
                view.hovered,
                view.selected,
                &sim.0,
                view.rotation,
                view.view_mode,
            );
        }
        *last_selection = Some(selection);
    }
    let guest_snapshot = (sim.0.ticks(), view.rotation, view.view_mode);
    let dynamic_rebuild =
        terrain_changed || sim.is_changed() || *last_guests != Some(guest_snapshot);
    if dynamic_rebuild {
        if let Some(mesh) = meshes.get_mut(&scene.guests) {
            *mesh = world::guest_mesh(&sim.0, view.rotation, view.view_mode);
        }
        if let Some(mesh) = meshes.get_mut(&scene.train) {
            *mesh = world::train_mesh(&sim.0, view.rotation, view.view_mode);
        }
        *last_guests = Some(guest_snapshot);
    }
    if let (Some(start), Some(probe)) = (timer, probe.as_mut()) {
        probe.record_scene(start.elapsed(), terrain_changed, dynamic_rebuild);
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
            selected: Some(SurfaceId::Terrain(TilePos { x: 3, y: 7 })),
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
        assert_eq!(
            view.selected,
            Some(SurfaceId::Terrain(TilePos { x: 3, y: 7 }))
        );
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
            selected: Some(SurfaceId::Terrain(TilePos { x: 3, y: 7 })),
            ..default()
        })
        .add_systems(Update, ui::button_actions);
        app.world_mut()
            .spawn((Button, Interaction::Pressed, ui::Action::CloseHelp));
        app.update();
        let view = app.world().resource::<ViewState>();
        assert!(!view.help);
        assert!(view.ui_pointer_consumed);
        assert_eq!(
            view.selected,
            Some(SurfaceId::Terrain(TilePos { x: 3, y: 7 }))
        );
        app.update();
        assert!(!app.world().resource::<ViewState>().ui_pointer_consumed);
    }
}
