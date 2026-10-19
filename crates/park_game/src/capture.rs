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
use park_sim::{Speed, TilePos};

use crate::{MAP_SIZE, ViewState};

const WARMUP_FRAMES: u32 = 90;
const WARMUP_TIME: Duration = Duration::from_secs(2);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy)]
enum Scene {
    Overview,
    Help,
    Selection,
}

#[derive(Resource)]
struct Capture {
    destination: PathBuf,
    temporary: PathBuf,
    scene: Scene,
    started: Instant,
    frames: u32,
    requested: bool,
    captured_size: Option<(u32, u32)>,
    finished: bool,
}

/// Install diagnostics only when `--capture-frame <absolute PNG path>` is set.
/// Optional `--capture-scene overview|help|selection` selects a stable preset.
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
                _ => panic!("--capture-scene must be overview, help, or selection"),
            },
        )
        .unwrap_or(Scene::Overview);

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
        started: Instant::now(),
        frames: 0,
        requested: false,
        captured_size: None,
        finished: false,
    })
    .add_systems(PostStartup, apply_preset)
    .add_systems(Last, capture_frame);
}

fn apply_preset(capture: Res<Capture>, mut view: ResMut<ViewState>) {
    view.set_speed(Speed::Paused);
    view.help = matches!(capture.scene, Scene::Help);
    if matches!(capture.scene, Scene::Selection) {
        view.selected = Some(TilePos {
            x: MAP_SIZE / 2,
            y: MAP_SIZE / 2,
        });
        view.rotation = 1;
        view.zoom = 1.35;
        view.scene_dirty = true;
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
