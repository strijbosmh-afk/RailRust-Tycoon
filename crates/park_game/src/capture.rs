//! Opt-in, in-engine render verification. Normal launches install no systems.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};
use park_sim::{
    ObjectKind, Park, Speed, SurfaceId, TRACK_UNITS, TerrainProfile, TilePos, TrainPhase,
};

use crate::{
    MAP_SIZE, Simulation, ViewState,
    path_tools::{self, PathTool, ToolState},
    persistence::{self, SaveAction, SaveSession},
    world::{self, ViewMode},
};

const WARMUP_FRAMES: u32 = 90;
const WARMUP_TIME: Duration = Duration::from_secs(2);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy)]
enum Scene {
    Overview,
    Help,
    Selection,
    Path,
    Track,
    Crowd,
    Lost,
    Recovered,
    Train,
    TrainLift,
    TrainDrop,
    TrainBrake,
    TrainReturn,
    TrainClosed,
    Saved,
    Loaded,
    Backup,
    LoadError,
    PathValid,
    PathBlocked,
    PathBuilt,
    PathDemolish,
    PathRemoved,
    PathRecovered,
}

impl Scene {
    fn is_path_tool(self) -> bool {
        matches!(
            self,
            Self::PathValid
                | Self::PathBlocked
                | Self::PathBuilt
                | Self::PathDemolish
                | Self::PathRemoved
                | Self::PathRecovered
        )
    }

    fn is_train(self) -> bool {
        matches!(
            self,
            Self::Train
                | Self::TrainLift
                | Self::TrainDrop
                | Self::TrainBrake
                | Self::TrainReturn
                | Self::TrainClosed
        )
    }
}

#[derive(Resource)]
struct Capture {
    destination: PathBuf,
    temporary: PathBuf,
    scene: Scene,
    rotation: Option<u8>,
    terrain_profile: TerrainProfile,
    view_mode: ViewMode,
    small_window: bool,
    ticks: u32,
    started: Instant,
    frames: u32,
    requested: bool,
    captured_size: Option<(u32, u32)>,
    finished: bool,
}

/// Install diagnostics only when `--capture-frame <absolute PNG path>` is set.
/// Scene and tick presets make crowd movement and recovery reproducible.
pub(super) fn configure(app: &mut App) {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    let Some(index) = arguments
        .iter()
        .position(|value| value == "--capture-frame")
    else {
        return;
    };
    let destination = arguments
        .get(index + 1)
        .map(PathBuf::from)
        .expect("--capture-frame requires an absolute PNG output path");
    assert!(
        destination.is_absolute(),
        "--capture-frame requires an absolute output path"
    );
    assert!(
        destination
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("png")),
        "--capture-frame output must have a .png extension"
    );
    let scene = arguments
        .iter()
        .position(|value| value == "--capture-scene")
        .map(
            |index| match arguments.get(index + 1).and_then(|value| value.to_str()) {
                Some("overview") => Scene::Overview,
                Some("help") => Scene::Help,
                Some("selection") => Scene::Selection,
                Some("path") => Scene::Path,
                Some("track") => Scene::Track,
                Some("crowd") => Scene::Crowd,
                Some("lost") => Scene::Lost,
                Some("recovered") => Scene::Recovered,
                Some("train") => Scene::Train,
                Some("train-lift") => Scene::TrainLift,
                Some("train-drop") => Scene::TrainDrop,
                Some("train-brake") => Scene::TrainBrake,
                Some("train-return") => Scene::TrainReturn,
                Some("train-closed") => Scene::TrainClosed,
                Some("saved") => Scene::Saved,
                Some("loaded") => Scene::Loaded,
                Some("backup") => Scene::Backup,
                Some("load-error") => Scene::LoadError,
                Some("path-valid") => Scene::PathValid,
                Some("path-blocked") => Scene::PathBlocked,
                Some("path-built") => Scene::PathBuilt,
                Some("path-demolish") => Scene::PathDemolish,
                Some("path-removed") => Scene::PathRemoved,
                Some("path-recovered") => Scene::PathRecovered,
                _ => panic!(
                    "unknown --capture-scene; see the capture scenes in RESUME_DEVELOPMENT.md"
                ),
            },
        )
        .unwrap_or(Scene::Overview);
    if matches!(
        scene,
        Scene::Saved | Scene::Loaded | Scene::Backup | Scene::LoadError
    ) {
        assert!(
            arguments.iter().any(|value| value == "--save-dir"),
            "persistence captures require an explicit isolated --save-dir"
        );
    }
    let rotation = arguments
        .iter()
        .position(|value| value == "--capture-rotation")
        .map(|index| {
            arguments
                .get(index + 1)
                .and_then(|value| value.to_str())
                .and_then(|value| value.parse::<u8>().ok())
                .filter(|value| *value < 4)
                .expect("--capture-rotation must be 0, 1, 2, or 3")
        });
    let terrain_profile = arguments
        .iter()
        .position(|value| value == "--capture-map")
        .map(
            |index| match arguments.get(index + 1).and_then(|value| value.to_str()) {
                Some("flat") => TerrainProfile::Flat,
                Some("terraced") => TerrainProfile::Terraced,
                _ => panic!("--capture-map must be flat or terraced"),
            },
        )
        .unwrap_or(TerrainProfile::Terraced);
    let view_mode = arguments
        .iter()
        .position(|value| value == "--capture-view")
        .map(
            |index| match arguments.get(index + 1).and_then(|value| value.to_str()) {
                Some("surface") => ViewMode::Surface,
                Some("underground") => ViewMode::Underground,
                _ => panic!("--capture-view must be surface or underground"),
            },
        )
        .unwrap_or(ViewMode::Surface);
    let small_window = arguments
        .iter()
        .any(|value| value == "--capture-small-window");
    let ticks = arguments
        .iter()
        .position(|value| value == "--capture-ticks")
        .map(|index| {
            arguments
                .get(index + 1)
                .and_then(|value| value.to_str())
                .and_then(|value| value.parse::<u32>().ok())
                .filter(|value| *value <= 36_000)
                .expect("--capture-ticks must be an integer between 0 and 36000")
        })
        .unwrap_or(600);

    let directory = destination
        .parent()
        .expect("absolute PNG path has a parent");
    fs::create_dir_all(directory).expect("cannot create capture output directory");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after 1970")
        .as_nanos();
    let temporary = directory.join(format!(
        ".parkbound-capture-{}-{nonce}.png",
        std::process::id()
    ));

    app.insert_resource(Capture {
        destination,
        temporary,
        scene,
        rotation,
        terrain_profile,
        view_mode,
        small_window,
        ticks,
        started: Instant::now(),
        frames: 0,
        requested: false,
        captured_size: None,
        finished: false,
    })
    .add_systems(PostStartup, apply_preset)
    .add_systems(Last, capture_frame);
}

fn apply_preset(
    capture: Res<Capture>,
    session: Res<SaveSession>,
    mut sim: ResMut<Simulation>,
    mut view: ResMut<ViewState>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    render_scene: Option<Res<crate::render_scene::RenderScene>>,
) {
    view.capture_active = true;
    view.set_speed(Speed::Paused);
    view.terrain_profile = capture.terrain_profile;
    sim.0
        .try_apply_terrain_profile(capture.terrain_profile)
        .expect("capture terrain must preserve fixture clearance");
    view.terrain_change_pending = false;
    view.view_mode = capture.view_mode;
    view.scene_dirty = true;
    view.help = matches!(capture.scene, Scene::Help);
    for _ in 0..capture.ticks {
        sim.0.advance(Speed::Normal);
    }
    if capture.scene.is_train() {
        prepare_train_scene(&mut sim.0, capture.scene);
        view.selected = sim.0.train().and_then(|train| {
            sim.0
                .circuit()?
                .pieces
                .get(train.piece_index)
                .map(|piece| SurfaceId::Object(piece.object))
        });
    }
    let mut crowd_anchor = None;
    if matches!(capture.scene, Scene::Lost | Scene::Recovered) {
        let guest_id = sim.0.guests()[0].id;
        let current = sim.0.guests()[0].current.expect("initial crowd has a path");
        crowd_anchor = sim.0.object(current).map(|object| object.pos);
        for neighbor in sim.0.path_neighbors(current) {
            sim.0.remove_object(neighbor);
        }
        for _ in 0..60 {
            sim.0.advance(Speed::Normal);
        }
        assert!(sim.0.crowd_stats().lost > 0, "isolated guest must be lost");
        view.selected = Some(SurfaceId::Object(current));
        if matches!(capture.scene, Scene::Recovered) {
            sim.0
                .restore_crowd_paths()
                .expect("capture paths must restore");
            for _ in 0..600 {
                sim.0.advance(Speed::Normal);
            }
            let guest = sim
                .0
                .guests()
                .iter()
                .find(|guest| guest.id == guest_id)
                .unwrap();
            assert_ne!(guest.state, park_sim::GuestState::Lost);
        }
    }
    if matches!(capture.scene, Scene::Selection) {
        view.selected = Some(SurfaceId::Terrain(TilePos {
            x: MAP_SIZE * 2 / 5,
            y: MAP_SIZE / 2,
        }));
        view.rotation = 1;
        view.zoom = 1.35;
        view.scene_dirty = true;
    }
    if matches!(capture.scene, Scene::Path | Scene::Track) {
        let kind = if matches!(capture.scene, Scene::Path) {
            ObjectKind::Path
        } else {
            ObjectKind::Track
        };
        view.selected = sim
            .0
            .objects()
            .iter()
            .find(|object| object.kind == kind && object.pos == (TilePos { x: 13, y: 16 }))
            .map(|object| SurfaceId::Object(object.id));
        assert!(view.selected.is_some(), "capture fixture must exist");
        view.zoom = 1.35;
    }
    if let Some(rotation) = capture.rotation {
        view.rotation = rotation;
        view.scene_dirty = true;
    }
    if let Some(scene) = render_scene {
        // Only overview captures may use an isolated rendering fixture.
        // Reapply its camera after the ordinary preset resets rotation/view.
        scene.apply_view(&sim.0, &mut view);
    }
    if capture.scene.is_train() {
        crate::focus_train(&sim.0, &mut view);
    }
    if matches!(capture.scene, Scene::Crowd | Scene::Lost | Scene::Recovered) {
        let anchor = crowd_anchor.unwrap_or(TilePos { x: 13, y: 17 });
        view.zoom = 1.6;
        view.pan = world::tile_center(anchor, MAP_SIZE, view.rotation)
            + Vec2::Y * (8.0 - f32::from(sim.0.max_surface_height()) * 0.5) * world::HEIGHT_STEP;
    }
    if matches!(capture.scene, Scene::Path | Scene::Track) {
        let point = world::surface_center(&sim.0, view.selected.unwrap(), view.rotation)
            .expect("selected capture object must have geometry");
        view.pan =
            point - Vec2::Y * f32::from(sim.0.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    }
    if capture.scene.is_path_tool() {
        prepare_path_scene(&mut sim.0, &mut view, capture.scene);
    }
    if matches!(
        capture.scene,
        Scene::Saved | Scene::Loaded | Scene::Backup | Scene::LoadError
    ) {
        let action = match capture.scene {
            Scene::Saved => SaveAction::Save,
            Scene::Backup => SaveAction::Backup,
            _ => SaveAction::Load,
        };
        let before = sim.0.clone();
        let success = persistence::perform_action(&session, &mut sim.0, &mut view, action);
        if matches!(capture.scene, Scene::LoadError) {
            assert!(
                !success,
                "load-error capture requires a missing or invalid save"
            );
            assert_eq!(sim.0, before, "failed load must preserve the live park");
        } else {
            assert!(success, "persistence capture failed: {}", view.save_notice);
        }
        // A loaded scene intentionally retains the normal load's fitted camera
        // and paused state, instead of modifying its terrain or simulating ahead.
    }
    if capture.small_window
        && let Ok(mut window) = windows.single_mut()
    {
        window.resolution.set(1024.0, 680.0);
    }
}

/// Use the real map tool command after resolving the normal visible hit. These
/// deterministic fixtures exercise commands and rendering, not native input.
fn prepare_path_scene(park: &mut Park, view: &mut ViewState, scene: Scene) {
    assert!(scene.is_path_tool());
    let before = park.clone();
    let build = matches!(scene, Scene::PathValid | Scene::PathBuilt);
    let occupied = (!build).then(|| {
        park.guests()
            .iter()
            .find_map(|guest| {
                let object = *park.object(guest.current?)?;
                let surface = SurfaceId::Object(object.id);
                let point = world::surface_center(park, surface, view.rotation)?;
                (world::pick_surface(point, park, view.rotation, view.view_mode) == Some(surface))
                    .then_some((object, guest.id, point))
            })
            .expect("path capture needs a visible occupied path")
    });
    let (pos, height, point) = if let Some((object, _, point)) = occupied {
        (object.pos, object.height, point)
    } else {
        let pos = TilePos { x: 13, y: 21 };
        let height = 8;
        let point = world::tile_center(pos, park.width(), view.rotation)
            + Vec2::Y * (f32::from(height) + 0.22) * world::HEIGHT_STEP;
        (pos, height, point)
    };
    view.path_tool = ToolState {
        mode: if matches!(
            scene,
            Scene::PathDemolish | Scene::PathRemoved | Scene::PathRecovered
        ) {
            PathTool::Demolish
        } else {
            PathTool::Build
        },
        height,
        ..default()
    };
    view.selected = None;
    view.hovered = world::pick_surface(point, park, view.rotation, view.view_mode);
    let click = matches!(
        scene,
        Scene::PathBuilt | Scene::PathRemoved | Scene::PathRecovered
    );
    path_tools::point_at(park, view, point, click);
    match scene {
        Scene::PathValid | Scene::PathBlocked | Scene::PathDemolish => {
            assert_eq!(*park, before, "path previews must not mutate the park");
            let preview = view
                .path_tool
                .preview
                .expect("capture preview must be visible");
            assert_eq!((preview.pos, preview.height), (pos, height));
            assert_eq!(preview.valid, !matches!(scene, Scene::PathBlocked));
            assert_eq!(preview.demolish, matches!(scene, Scene::PathDemolish));
        }
        Scene::PathBuilt => {
            assert_eq!(park.objects().len(), before.objects().len() + 1);
            let Some(SurfaceId::Object(id)) = view.selected else {
                panic!("build capture must select its newly placed path");
            };
            let object = park.object(id).expect("placed path must exist");
            assert_eq!((object.pos, object.height), (pos, height));
            assert!(
                before.object(id).is_none(),
                "a placed path needs a fresh ID"
            );
        }
        Scene::PathRemoved | Scene::PathRecovered => {
            let (object, guest_id, _) = occupied.unwrap();
            assert!(park.object(object.id).is_none());
            assert_eq!(park.objects().len() + 1, before.objects().len());
            assert!(
                park.crowd_stats().lost > 0,
                "removal must strand an occupied path"
            );
            assert_eq!(park.ticks(), before.ticks(), "removal happens while paused");
            if matches!(scene, Scene::PathRecovered) {
                view.path_tool.set_mode(PathTool::Build);
                view.hovered = world::pick_surface(point, park, view.rotation, view.view_mode);
                path_tools::point_at(park, view, point, true);
                let Some(SurfaceId::Object(id)) = view.selected else {
                    panic!("recovery capture must place a replacement path");
                };
                assert_ne!(id, object.id, "demolished IDs must not be reused");
                assert_eq!(park.objects().len(), before.objects().len());
                for _ in 0..120 {
                    park.advance(Speed::Normal);
                }
                let guest = park
                    .guests()
                    .iter()
                    .find(|guest| guest.id == guest_id)
                    .unwrap();
                assert_ne!(guest.state, park_sim::GuestState::Lost);
            }
        }
        _ => unreachable!(),
    }
    view.zoom = 2.0;
    view.pan = point - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

fn prepare_train_scene(park: &mut Park, scene: Scene) {
    if matches!(scene, Scene::TrainClosed) {
        park.set_train_open(false);
    }
    let target = match scene {
        Scene::TrainLift => Some(TrainPhase::Lift),
        Scene::TrainDrop => Some(TrainPhase::Drop),
        Scene::TrainBrake => Some(TrainPhase::Brake),
        Scene::TrainReturn => Some(TrainPhase::Station),
        Scene::TrainClosed => Some(TrainPhase::Closed),
        _ => None,
    };
    if let Some(target) = target {
        // Bound diagnostic seeking: a broken circuit must fail, not hang.
        for _ in 0..36_000 {
            let train = park.train().expect("train fixture must be installed");
            if train.phase == target && train_capture_ready(park, scene) {
                return;
            }
            assert_ne!(train.phase, TrainPhase::Fault, "capture circuit is invalid");
            park.advance(Speed::Normal);
        }
        panic!("train did not reach the requested capture phase");
    }
}

/// Capture the demonstrated behavior inside a section, not the first tick at
/// its boundary before lift, gravity or brake speed has taken effect.
fn train_capture_ready(park: &Park, scene: Scene) -> bool {
    let Some(train) = park.train() else {
        return false;
    };
    match scene {
        Scene::TrainLift => {
            train.phase == TrainPhase::Lift
                && train.speed == 24
                && park.train_cars().first().is_some_and(|car| {
                    car.center.height >= 12 * TRACK_UNITS && car.center.height < 16 * TRACK_UNITS
                })
        }
        Scene::TrainDrop => {
            train.phase == TrainPhase::Drop
                && train.speed >= 60
                && park.train_cars().first().is_some_and(|car| {
                    car.center.height <= 14 * TRACK_UNITS && car.center.height > 8 * TRACK_UNITS
                })
        }
        Scene::TrainBrake => {
            train.phase == TrainPhase::Brake
                && train.speed == 20
                && park.circuit().is_some_and(|circuit| {
                    let piece = &circuit.pieces[train.piece_index];
                    park.train_cars().first().is_some_and(|car| {
                        let entry = piece.samples[0];
                        // Brakes are straight: Manhattan distance measures how
                        // far the lead car has travelled past this tile edge.
                        let distance =
                            (car.center.x - entry.x).abs() + (car.center.y - entry.y).abs();
                        distance >= TRACK_UNITS / 3
                    })
                })
        }
        Scene::TrainReturn => {
            train.phase == TrainPhase::Station && train.laps > 0 && train.speed == 0
        }
        Scene::TrainClosed => train.phase == TrainPhase::Closed && train.speed == 0,
        _ => true,
    }
}

fn capture_frame(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.finished {
        return;
    }
    capture.frames += 1;

    // Screenshot events are delivered in First. By Last, both the save observer
    // and receipt observer have returned, independent of their execution order.
    // Bevy's desktop save_to_disk writes the PNG synchronously in its observer.
    if let Some(size) = capture.captured_size {
        let outcome = verify_png(&capture.temporary, size).and_then(|()| {
            fs::rename(&capture.temporary, &capture.destination).map_err(|error| error.to_string())
        });
        match outcome {
            Ok(()) => {
                info!(
                    "Render verification saved {} ({} × {})",
                    capture.destination.display(),
                    size.0,
                    size.1
                );
                exit.write(AppExit::Success);
            }
            Err(error) => {
                error!("Render verification failed: {error}");
                let _ = fs::remove_file(&capture.temporary);
                exit.write(AppExit::error());
            }
        }
        capture.finished = true;
        return;
    }

    if capture.started.elapsed() > CAPTURE_TIMEOUT {
        error!("Render verification timed out before a completed screenshot was received");
        let _ = fs::remove_file(&capture.temporary);
        capture.finished = true;
        exit.write(AppExit::error());
        return;
    }

    if !capture.requested
        && capture.frames >= WARMUP_FRAMES
        && capture.started.elapsed() >= WARMUP_TIME
    {
        capture.requested = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(capture.temporary.clone()))
            .observe(capture_received);
    }
}

fn capture_received(event: On<ScreenshotCaptured>, mut capture: ResMut<Capture>) {
    capture.captured_size = Some((event.image.width(), event.image.height()));
}

fn verify_png(path: &Path, expected: (u32, u32)) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read captured PNG: {error}"))?;
    // An explicit final IEND chunk catches a failed/truncated image write. The
    // unique temporary name prevents an earlier capture from passing this check.
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    const END: &[u8] = b"\0\0\0\0IEND\xaeB`\x82";
    if bytes.len() < 45 || !bytes.starts_with(SIGNATURE) || !bytes.ends_with(END) {
        return Err("renderer did not produce a complete PNG".into());
    }
    let width = u32::from_be_bytes(
        bytes[16..20]
            .try_into()
            .expect("PNG width slice is four bytes"),
    );
    let height = u32::from_be_bytes(
        bytes[20..24]
            .try_into()
            .expect("PNG height slice is four bytes"),
    );
    if (width, height) != expected || width == 0 || height == 0 {
        return Err("saved PNG dimensions differ from the captured render target".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_scene::{CameraPreset, RenderScene};
    use park_sim::diagnostics::DiagnosticCase;

    #[test]
    fn large_render_capture_overrides_survive_post_startup_and_refresh_the_first_frame() {
        for camera in [CameraPreset::Crowd, CameraPreset::Ride] {
            let render_scene = RenderScene {
                case: DiagnosticCase::SaveScale,
                camera,
                rotation: 3,
                underground: true,
            };
            let park = render_scene.build().unwrap();
            assert!(park.max_terrain_height() > 0);
            let initial_train = park.train_cars();
            let mut view = ViewState::default();
            render_scene.apply_view(&park, &mut view);
            let untouched_directory =
                std::env::temp_dir().join("Parkbound capture ordering no file operations");
            let mut app = App::new();
            app.insert_resource(Simulation(park))
                .insert_resource(view)
                .insert_resource(render_scene)
                .insert_resource(SaveSession::from_arguments([
                    OsString::from("--save-dir"),
                    untouched_directory.into_os_string(),
                ]))
                .insert_resource(Capture {
                    // No capture_frame system is installed: this test covers
                    // real preset/startup ordering without a GPU or file I/O.
                    destination: PathBuf::new(),
                    temporary: PathBuf::new(),
                    scene: Scene::Overview,
                    rotation: None,
                    terrain_profile: TerrainProfile::Flat,
                    view_mode: ViewMode::Surface,
                    small_window: false,
                    ticks: 137,
                    started: Instant::now(),
                    frames: 0,
                    requested: false,
                    captured_size: None,
                    finished: false,
                })
                .insert_resource(Time::<()>::default())
                .init_resource::<ButtonInput<KeyCode>>()
                .init_resource::<Assets<Mesh>>()
                .init_resource::<Assets<ColorMaterial>>()
                .add_systems(Startup, crate::setup_scene)
                .add_systems(PostStartup, apply_preset)
                .add_systems(Update, (crate::keyboard_input, crate::update_scene).chain());
            app.world_mut()
                .spawn((Window::default(), bevy::window::PrimaryWindow));
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                for key in [
                    KeyCode::Space,
                    KeyCode::KeyE,
                    KeyCode::Home,
                    KeyCode::Delete,
                    KeyCode::KeyO,
                    KeyCode::F5,
                ] {
                    keys.press(key);
                }
            }
            app.update();
            let park = &app.world().resource::<Simulation>().0;
            let view = app.world().resource::<ViewState>();
            assert_eq!((park.width(), park.height(), park.ticks()), (256, 256, 137));
            assert_eq!(park.max_terrain_height(), 0);
            assert_ne!(park.train_cars(), initial_train);
            assert_eq!(view.terrain_profile, TerrainProfile::Flat);
            assert_eq!(view.rotation, 3);
            assert_eq!(view.view_mode, ViewMode::Underground);
            assert_eq!(view.speed, Speed::Paused);
            assert!(view.capture_active);
            assert!(
                !view.scene_dirty,
                "first Update must consume the preset rebuild"
            );
            assert!(!view.remove_path_pending && !view.toggle_train_pending);
            assert_eq!(view.save_pending, None);
            let focus = match camera {
                CameraPreset::Crowd => world::path_focus(park, 3).unwrap(),
                CameraPreset::Ride => world::train_focus(park, 3).unwrap(),
                CameraPreset::Overview => unreachable!(),
            };
            assert_eq!(
                view.zoom,
                if camera == CameraPreset::Crowd {
                    12.8
                } else {
                    12.0
                }
            );
            assert!(
                (view.pan
                    - (focus
                        - Vec2::Y
                            * f32::from(park.max_surface_height())
                            * world::HEIGHT_STEP
                            * 0.5))
                    .length()
                    < 0.001
            );
            let scene = app.world().resource::<crate::SceneMeshes>();
            let meshes = app.world().resource::<Assets<Mesh>>();
            assert_eq!(scene.guest_renderer.stats().updates, 2);
            // In the underground view, flat-vs-hills changes the floor colors.
            // This catches retaining the Startup mesh after the preset runs.
            let expected_static =
                world::build_static_chunks(park, 3, view.grid, ViewMode::Underground);
            let landscape = app
                .world()
                .resource::<crate::static_landscape::StaticLandscape>();
            assert_eq!(landscape.mesh_handles().count(), expected_static.len());
            for (handle, expected) in landscape.mesh_handles().zip(expected_static) {
                let actual = meshes.get(handle).unwrap();
                assert_eq!(
                    actual.attribute(Mesh::ATTRIBUTE_COLOR),
                    expected.mesh.attribute(Mesh::ATTRIBUTE_COLOR)
                );
                assert_eq!(
                    actual.attribute(Mesh::ATTRIBUTE_POSITION),
                    expected.mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                );
                assert_eq!(actual.indices(), expected.mesh.indices());
            }
            let mut expected_renderer = world::GuestRenderer::default();
            let mut expected_guests = world::GuestRenderer::empty_mesh();
            expected_renderer.update(&mut expected_guests, park, 3, ViewMode::Underground);
            let actual_guests = meshes.get(&scene.guests).unwrap();
            assert_eq!(
                actual_guests.attribute(Mesh::ATTRIBUTE_POSITION),
                expected_guests.attribute(Mesh::ATTRIBUTE_POSITION)
            );
            assert_eq!(
                actual_guests.attribute(Mesh::ATTRIBUTE_COLOR),
                expected_guests.attribute(Mesh::ATTRIBUTE_COLOR)
            );
            assert_eq!(
                meshes
                    .get(&scene.train)
                    .unwrap()
                    .attribute(Mesh::ATTRIBUTE_POSITION),
                world::train_mesh(park, 3, ViewMode::Underground)
                    .attribute(Mesh::ATTRIBUTE_POSITION)
            );
            app.update();
            assert_eq!(app.world().resource::<Simulation>().0.ticks(), 137);
            assert_eq!(
                app.world()
                    .resource::<crate::SceneMeshes>()
                    .guest_renderer
                    .stats()
                    .updates,
                2,
            );
        }
    }

    fn park_at(ticks: u32) -> Park {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(crate::CROWD_SEED).unwrap();
        park.install_train_fixture().unwrap();
        for _ in 0..ticks {
            park.advance(Speed::Normal);
        }
        park
    }

    #[test]
    fn path_previews_preserve_save_bytes_and_cache_at_all_rotations_and_views() {
        for terrain in [TerrainProfile::Flat, TerrainProfile::Terraced] {
            let mut original = park_at(600);
            original.try_apply_terrain_profile(terrain).unwrap();
            for rotation in 0..4 {
                for view_mode in [ViewMode::Surface, ViewMode::Underground] {
                    for scene in [Scene::PathValid, Scene::PathBlocked, Scene::PathDemolish] {
                        let mut park = original.clone();
                        let bytes = park.to_save_bytes().unwrap();
                        let cache = park.path_cache_stats();
                        let mut view = ViewState {
                            rotation,
                            view_mode,
                            speed: Speed::Paused,
                            capture_active: true,
                            ..default()
                        };
                        prepare_path_scene(&mut park, &mut view, scene);
                        assert_eq!(park, original);
                        assert_eq!(park.to_save_bytes().unwrap(), bytes);
                        assert_eq!(park.path_cache_stats(), cache);
                        assert!(view.path_tool.preview.is_some());
                        assert!(view.path_notice.is_none());
                        assert!(view.save_pending.is_none());
                        assert!(!view.remove_path_pending && !view.restore_paths_pending);
                        assert!(!view.toggle_train_pending && !view.terrain_change_pending);
                        assert_eq!(view.speed, Speed::Paused);
                    }
                }
            }
        }
    }

    #[test]
    fn path_capture_edits_only_the_target_and_recovers_with_a_fresh_id() {
        for rotation in 0..4 {
            for view_mode in [ViewMode::Surface, ViewMode::Underground] {
                for scene in [Scene::PathBuilt, Scene::PathRemoved, Scene::PathRecovered] {
                    let original = park_at(600);
                    let mut park = original.clone();
                    let mut view = ViewState {
                        rotation,
                        view_mode,
                        speed: Speed::Paused,
                        capture_active: true,
                        ..default()
                    };
                    prepare_path_scene(&mut park, &mut view, scene);
                    let removed: Vec<_> = original
                        .objects()
                        .iter()
                        .filter(|object| park.object(object.id).is_none())
                        .collect();
                    let added: Vec<_> = park
                        .objects()
                        .iter()
                        .filter(|object| original.object(object.id).is_none())
                        .collect();
                    assert_eq!(
                        removed.len(),
                        usize::from(!matches!(scene, Scene::PathBuilt))
                    );
                    assert_eq!(
                        added.len(),
                        usize::from(!matches!(scene, Scene::PathRemoved))
                    );
                    for object in original.objects() {
                        if let Some(retained) = park.object(object.id) {
                            assert_eq!(retained, object);
                        }
                    }
                    for object in removed.iter().chain(&added) {
                        assert_eq!(object.kind, ObjectKind::Path);
                    }
                    if matches!(scene, Scene::PathRecovered) {
                        assert_eq!(added[0].pos, removed[0].pos);
                        assert_eq!(added[0].height, removed[0].height);
                        assert!(added[0].id > removed[0].id);
                    }
                    for y in 0..park.height() {
                        for x in 0..park.width() {
                            assert_eq!(
                                park.tile(TilePos { x, y }),
                                original.tile(TilePos { x, y })
                            );
                        }
                    }
                    assert_eq!(
                        park.guests()
                            .iter()
                            .map(|guest| guest.id)
                            .collect::<Vec<_>>(),
                        original
                            .guests()
                            .iter()
                            .map(|guest| guest.id)
                            .collect::<Vec<_>>()
                    );
                    let mut expected_train = original.clone();
                    let recovery_ticks = if matches!(scene, Scene::PathRecovered) {
                        120
                    } else {
                        0
                    };
                    for _ in 0..recovery_ticks {
                        expected_train.advance(Speed::Normal);
                    }
                    assert_eq!(park.ticks(), original.ticks() + recovery_ticks);
                    assert_eq!(park.train(), expected_train.train());
                    assert_eq!(park.circuit(), original.circuit());
                    assert!(view.path_tool.preview.is_none());
                    assert!(view.save_pending.is_none());
                    assert_eq!(view.speed, Speed::Paused);
                }
            }
        }
    }

    #[test]
    fn path_capture_focus_stays_clear_of_controls_at_the_minimum_window() {
        for scene in [
            Scene::PathValid,
            Scene::PathBlocked,
            Scene::PathBuilt,
            Scene::PathDemolish,
            Scene::PathRemoved,
            Scene::PathRecovered,
        ] {
            let mut park = park_at(600);
            let mut view = ViewState {
                rotation: 3,
                speed: Speed::Paused,
                capture_active: true,
                ..default()
            };
            prepare_path_scene(&mut park, &mut view, scene);
            let focus = view.pan
                + Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
            for (width, height) in [(1024.0, 680.0), (1280.0, 800.0)] {
                let mut window = Window::default();
                window.resolution.set(width, height);
                let (scale, center) = crate::camera_layout(&window, &view, &park);
                let relative = (focus - center) / scale;
                let screen = Vec2::new(width * 0.5 + relative.x, height * 0.5 - relative.y);
                assert!(crate::is_map_position(screen, &window, false));
            }
        }
    }

    #[test]
    fn train_phase_presets_capture_interior_motion_and_parked_station_states() {
        for initial_ticks in [0, 600] {
            let original = park_at(initial_ticks);
            for scene in [
                Scene::TrainLift,
                Scene::TrainDrop,
                Scene::TrainBrake,
                Scene::TrainReturn,
                Scene::TrainClosed,
            ] {
                let mut park = original.clone();
                prepare_train_scene(&mut park, scene);
                assert!(train_capture_ready(&park, scene));
                assert_eq!(park.objects(), original.objects());
                assert_eq!(
                    park.guests()
                        .iter()
                        .map(|guest| guest.id)
                        .collect::<Vec<_>>(),
                    original
                        .guests()
                        .iter()
                        .map(|guest| guest.id)
                        .collect::<Vec<_>>()
                );
                let train = park.train().unwrap();
                match scene {
                    Scene::TrainLift => {
                        assert_eq!(train.phase, TrainPhase::Lift);
                        assert_eq!(train.speed, 24);
                        assert!(park.train_cars()[0].center.height >= 12 * TRACK_UNITS);
                    }
                    Scene::TrainDrop => {
                        assert_eq!(train.phase, TrainPhase::Drop);
                        assert!(train.speed >= 60);
                        assert!(park.train_cars()[0].center.height <= 14 * TRACK_UNITS);
                    }
                    Scene::TrainBrake => {
                        assert_eq!(train.phase, TrainPhase::Brake);
                        assert_eq!(train.speed, 20);
                    }
                    Scene::TrainReturn => {
                        assert_eq!(train.phase, TrainPhase::Station);
                        assert!(train.laps > 0);
                        assert_eq!(train.speed, 0);
                    }
                    Scene::TrainClosed => {
                        assert_eq!(train.phase, TrainPhase::Closed);
                        assert!(!train.open);
                        assert_eq!(train.speed, 0);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    #[test]
    fn train_capture_rejects_first_lift_drop_and_brake_boundary_ticks() {
        let mut park = park_at(0);
        for (scene, phase) in [
            (Scene::TrainLift, TrainPhase::Lift),
            (Scene::TrainDrop, TrainPhase::Drop),
            (Scene::TrainBrake, TrainPhase::Brake),
        ] {
            for _ in 0..3_600 {
                if park.train().unwrap().phase == phase {
                    break;
                }
                park.advance(Speed::Normal);
            }
            assert_eq!(park.train().unwrap().phase, phase);
            assert!(
                !train_capture_ready(&park, scene),
                "the first section-entry frame must not be used as motion evidence"
            );
            prepare_train_scene(&mut park, scene);
            assert!(train_capture_ready(&park, scene));
        }
    }
}
