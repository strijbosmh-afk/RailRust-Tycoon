//! Isolated rendering fixtures. These use production meshes and simulation,
//! but never expose editing or persistence of the player's park.

use std::ffi::OsString;

use bevy::prelude::*;
use park_sim::{
    ObjectKind, Park,
    diagnostics::{DiagnosticCase, build_fixture},
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{CROWD_SEED, MAP_SIZE, ViewState, world};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum CameraPreset {
    #[default]
    Overview,
    Crowd,
    Ride,
}

#[derive(Clone, Debug, Resource)]
pub(super) struct RenderScene {
    pub case: DiagnosticCase,
    pub camera: CameraPreset,
    pub rotation: u8,
    pub underground: bool,
}

impl RenderScene {
    /// Remove only this module's validated flags so the strict frame parser
    /// still rejects unrelated or malformed diagnostic options.
    pub fn extract(arguments: &[OsString]) -> Result<(Option<Self>, Vec<OsString>), String> {
        let mut case = None;
        let mut camera = None;
        let mut rotation = None;
        let mut underground = false;
        let mut remaining = Vec::new();
        let mut arguments = arguments.iter();
        while let Some(argument) = arguments.next() {
            let flag = argument.to_str().unwrap_or("");
            match flag {
                "--render-case" | "--render-camera" | "--render-rotation" => {
                    let value = arguments
                        .next()
                        .and_then(|value| value.to_str())
                        .filter(|value| !value.starts_with("--"))
                        .ok_or_else(|| format!("{flag} requires a value"))?;
                    let duplicate = match flag {
                        "--render-case" => case
                            .replace(match value {
                                "baseline" => DiagnosticCase::Baseline,
                                "save-scale" => DiagnosticCase::SaveScale,
                                "stress" => DiagnosticCase::Stress,
                                _ => {
                                    return Err(
                                        "--render-case must be baseline, save-scale or stress"
                                            .into(),
                                    );
                                }
                            })
                            .is_some(),
                        "--render-camera" => camera
                            .replace(match value {
                                "overview" => CameraPreset::Overview,
                                "crowd" => CameraPreset::Crowd,
                                "ride" => CameraPreset::Ride,
                                _ => {
                                    return Err(
                                        "--render-camera must be overview, crowd or ride".into()
                                    );
                                }
                            })
                            .is_some(),
                        _ => rotation
                            .replace(
                                value
                                    .parse::<u8>()
                                    .ok()
                                    .filter(|value| *value < 4)
                                    .ok_or("--render-rotation must be 0, 1, 2 or 3")?,
                            )
                            .is_some(),
                    };
                    if duplicate {
                        return Err(format!("{flag} may only be supplied once"));
                    }
                }
                "--render-underground" => {
                    if underground {
                        return Err("--render-underground may only be supplied once".into());
                    }
                    underground = true;
                }
                _ if flag.starts_with("--render-") => {
                    return Err(format!("unknown render option: {flag}"));
                }
                _ => remaining.push(argument.clone()),
            }
        }
        let Some(case) = case else {
            if camera.is_some() || rotation.is_some() || underground {
                return Err("render options require --render-case".into());
            }
            return Ok((None, remaining));
        };
        if remaining.iter().any(|v| {
            matches!(
                v.to_str(),
                Some(
                    "--save-dir"
                        | "--save-check"
                        | "--benchmark"
                        | "--benchmark-case"
                        | "--benchmark-ticks"
                )
            )
        }) {
            return Err(
                "render fixtures cannot be combined with save commands or CPU benchmarks".into(),
            );
        }
        if remaining.iter().any(|v| v == "--capture-frame") {
            validate_capture(&remaining)?;
        } else if !remaining.is_empty() && !remaining.iter().any(|v| v == "--profile-frames") {
            return Err(
                "render previews accept only render options, or one capture/frame-profile mode"
                    .into(),
            );
        }
        Ok((
            Some(Self {
                case,
                camera: camera.unwrap_or_default(),
                rotation: rotation.unwrap_or(0),
                underground,
            }),
            remaining,
        ))
    }

    pub fn build(&self) -> Result<Park, String> {
        build_fixture(self.case, CROWD_SEED)
            .map(|fixture| fixture.park)
            .map_err(|error| error.to_string())
    }

    pub fn apply_view(&self, park: &Park, view: &mut ViewState) {
        view.diagnostic_case = Some(self.case);
        view.camera_span = (park.width().max(park.height()) as f32 / MAP_SIZE as f32).max(1.0);
        if let Some(bounds) = world::scene_bounds(park) {
            view.pan_limit = bounds.min.abs().max(bounds.max.abs()) + Vec2::splat(600.0);
        }
        view.rotation = self.rotation;
        view.view_mode = if self.underground {
            world::ViewMode::Underground
        } else {
            world::ViewMode::Surface
        };
        view.reset_camera();
        match self.camera {
            CameraPreset::Overview => {}
            CameraPreset::Crowd => {
                if let Some(center) = world::path_focus(park, view.rotation) {
                    view.zoom = 1.6 * view.camera_span;
                    view.pan = center
                        - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
                }
            }
            CameraPreset::Ride => crate::focus_train(park, view),
        }
        view.scene_dirty = true;
        view.save_notice = "Render preview only\nEditing and saves disabled".into();
    }
}

/// Old demonstration scene commands assume fixed IDs and locations. Only the
/// overview capture is allowed here; shared render flags control its camera.
fn validate_capture(arguments: &[OsString]) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut args = arguments.iter();
    while let Some(flag) = args.next() {
        let flag = flag.to_str().ok_or("capture flags must be text")?;
        if !seen.insert(flag) {
            return Err(format!("repeated render capture option: {flag}"));
        }
        if flag == "--capture-small-window" {
            continue;
        }
        if !matches!(
            flag,
            "--capture-frame" | "--capture-scene" | "--capture-map" | "--capture-ticks"
        ) {
            return Err("render captures accept overview, map, ticks and small-window options; use render flags for camera/rotation/view".into());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag {
            "--capture-frame" => {
                let path = std::path::Path::new(value);
                if !path.is_absolute()
                    || path.exists()
                    || !path
                        .extension()
                        .and_then(|v| v.to_str())
                        .is_some_and(|v| v.eq_ignore_ascii_case("png"))
                {
                    return Err("render capture requires an absolute NEW .png path".into());
                }
            }
            "--capture-scene" if value != "overview" => {
                return Err("render fixtures support only the overview capture scene".into());
            }
            "--capture-map" if value != "flat" && value != "terraced" => {
                return Err("capture map must be flat or terraced".into());
            }
            "--capture-ticks" => {
                value
                    .to_str()
                    .and_then(|v| v.parse::<u32>().ok())
                    .filter(|v| *v <= 1800)
                    .ok_or("render capture ticks must be between 0 and 1800")?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn describe(park: &Park, view: &ViewState, scene: Option<&RenderScene>) -> Value {
    let case = scene.map_or(DiagnosticCase::Baseline, |s| s.case);
    json!({
        "mode": if scene.is_some() { "render_preview" } else { "normal_demo" },
        "fixture": {
            "case":case, "seed":CROWD_SEED,
            "map_width":park.width(), "map_height":park.height(),
            "tiles":u64::from(park.width()) * u64::from(park.height()),
            "guests":park.guests().len(),
            "paths":park.objects().iter().filter(|o| o.kind == ObjectKind::Path).count(),
            "objects":park.objects().len(),
            "rides":usize::from(park.circuit().is_some()), "trains":usize::from(park.train().is_some()),
            "track_pieces":park.circuit().map_or(0,|c|c.pieces.len()),
            "save_supported":case != DiagnosticCase::Stress,
        },
        "camera":scene.map_or(CameraPreset::Overview, |s|s.camera),
        "rotation":view.rotation,
        "view":if view.view_mode == world::ViewMode::Surface { "surface" } else { "underground" },
        "zoom_relative_to_map_fit":view.zoom, "pan_world":view.pan.to_array(),
        "edits_and_saves_enabled":scene.is_none() && !view.capture_active,
        "note":"Controlled rendering fixture; one circuit/train. Codec capacity is distinct from preview persistence, which is disabled. No reference PC or hardware acceptance.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn render_flags_are_strict_and_cannot_reach_save_commands() {
        assert!(RenderScene::extract(&[]).unwrap().0.is_none());
        let (scene, remaining) = RenderScene::extract(&args(&[
            "--render-case",
            "stress",
            "--render-camera",
            "crowd",
            "--render-rotation",
            "3",
            "--render-underground",
            "--profile-frames",
            "elsewhere.json",
            "--profile-gpu",
        ]))
        .unwrap();
        let scene = scene.unwrap();
        assert_eq!(scene.case, DiagnosticCase::Stress);
        assert_eq!(scene.camera, CameraPreset::Crowd);
        assert_eq!(scene.rotation, 3);
        assert!(scene.underground);
        assert_eq!(
            remaining,
            args(&["--profile-frames", "elsewhere.json", "--profile-gpu"])
        );
        for invalid in [
            vec!["--render-camera", "crowd"],
            vec!["--render-case", "typo"],
            vec!["--render-case"],
            vec!["--render-case", "stress", "--render-case", "baseline"],
            vec!["--render-case", "stress", "--render-rotation", "4"],
            vec!["--render-case", "stress", "--render-camera", "typo"],
            vec!["--render-case", "stress", "--save-dir", "ignored"],
            vec!["--render-case", "stress", "--save-check", "write"],
            vec!["--render-case", "stress", "--benchmark", "ignored"],
            vec!["--render-case", "stress", "--capture-frame", "relative.png"],
            vec!["--render-case", "stress", "--unknown"],
        ] {
            assert!(
                RenderScene::extract(&args(&invalid)).is_err(),
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn large_preview_frames_real_fixture_without_enabling_edits_or_saves() {
        let scene = RenderScene {
            case: DiagnosticCase::SaveScale,
            camera: CameraPreset::Crowd,
            rotation: 2,
            underground: true,
        };
        let park = scene.build().unwrap();
        let before = park.clone();
        let mut view = ViewState::default();
        scene.apply_view(&park, &mut view);
        assert_eq!(park, before);
        assert_eq!(view.diagnostic_case, Some(DiagnosticCase::SaveScale));
        assert_eq!(view.camera_span, 8.0);
        assert_eq!(view.zoom, 12.8);
        assert!(view.pan_limit.x > 8192.0);
        let report = describe(&park, &view, Some(&scene));
        assert_eq!(report["fixture"]["guests"], 256);
        assert_eq!(report["fixture"]["paths"], 1024);
        assert_eq!(report["fixture"]["rides"], 1);
        assert_eq!(report["edits_and_saves_enabled"], false);
        assert_eq!(report["view"], "underground");
    }

    #[test]
    fn shared_render_and_profile_flags_preserve_strict_early_validation() {
        let output = std::env::temp_dir().join(format!(
            "Parkbound combined CLI {}.json",
            std::process::id()
        ));
        let mut arguments = args(&[
            "--render-case",
            "save-scale",
            "--render-camera",
            "ride",
            "--render-rotation",
            "3",
            "--render-underground",
            "--profile-frames",
        ]);
        arguments.push(output.into_os_string());
        arguments.extend(args(&["--profile-samples", "120", "--profile-gpu"]));
        assert!(crate::measurement::validate_modes(&arguments).is_ok());
        let (scene, filtered) = RenderScene::extract(&arguments).unwrap();
        assert!(scene.is_some());
        assert!(crate::frame_probe::validate_arguments(&filtered).is_ok());
        for suffix in [
            vec!["--profile-gpu"],
            vec!["--profile-samples", "120"],
            vec!["--unknown"],
        ] {
            let mut invalid = arguments.clone();
            invalid.extend(args(&suffix));
            let (_, filtered) = RenderScene::extract(&invalid).unwrap();
            assert!(crate::frame_probe::validate_arguments(&filtered).is_err());
        }
    }
}
