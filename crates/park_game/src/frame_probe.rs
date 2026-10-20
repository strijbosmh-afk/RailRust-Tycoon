//! Opt-in wall-clock main-loop pacing, not GPU or presentation timestamps.
//! Normal launches install no resources, systems, or per-frame clock reads here.

use std::{
    ffi::OsString,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::renderer::RenderAdapterInfo,
    window::{PresentMode, PrimaryWindow, WindowOccluded},
    winit::WinitSettings,
};
use park_sim::{Speed, TICKS_PER_SECOND};
use serde_json::{Value, json};

use crate::{
    Simulation, ViewState,
    measurement::{summarize, validate_output, write_json_new},
};

const DEFAULT_SAMPLES: usize = 600;
const MIN_SAMPLES: usize = 120;
const MAX_SAMPLES: usize = 3600;
const WARMUP_FRAMES: u32 = 120;
const WARMUP_TIME: Duration = Duration::from_secs(2);
const DEADLINE: Duration = Duration::from_secs(120);
const MAX_HOOK_SAMPLES: usize = 32_768;
const REQUESTED_SIZE: [u32; 2] = [1920, 1080];

/// Optional hooks for the existing simulation and mesh update systems.
/// Callers should only read Instant when this resource exists and is_measuring.
#[derive(Resource, Default)]
pub(super) struct Probe {
    measuring: bool,
    simulation_times: Vec<Duration>,
    scene_times: Vec<Duration>,
    simulation_ticks: u64,
    simulation_calls: u64,
    scene_calls: u64,
    static_rebuild_calls: u64,
    dynamic_rebuild_calls: u64,
    omitted_simulation_timings: u64,
    omitted_scene_timings: u64,
}

impl Probe {
    pub(super) fn is_measuring(&self) -> bool {
        self.measuring
    }

    pub(super) fn record_simulation(&mut self, elapsed: Duration, ticks_advanced: u64) {
        if !self.measuring {
            return;
        }
        self.simulation_calls += 1;
        self.simulation_ticks = self.simulation_ticks.saturating_add(ticks_advanced);
        if self.simulation_times.len() < MAX_HOOK_SAMPLES {
            self.simulation_times.push(elapsed);
        } else {
            self.omitted_simulation_timings += 1;
        }
    }

    pub(super) fn record_scene(
        &mut self,
        elapsed: Duration,
        static_rebuild: bool,
        dynamic_rebuild: bool,
    ) {
        if !self.measuring {
            return;
        }
        self.scene_calls += 1;
        self.static_rebuild_calls += u64::from(static_rebuild);
        self.dynamic_rebuild_calls += u64::from(dynamic_rebuild);
        if self.scene_times.len() < MAX_HOOK_SAMPLES {
            self.scene_times.push(elapsed);
        } else {
            self.omitted_scene_timings += 1;
        }
    }

    fn start(&mut self, target_samples: usize) {
        *self = Self {
            measuring: true,
            simulation_times: Vec::with_capacity(target_samples),
            scene_times: Vec::with_capacity(target_samples),
            ..default()
        };
    }
}

#[derive(Debug)]
struct Settings {
    destination: PathBuf,
    samples: usize,
    no_vsync: bool,
}

impl Settings {
    fn parse(arguments: &[OsString]) -> Result<Option<Self>, String> {
        if !arguments
            .iter()
            .any(|argument| argument == "--profile-frames")
        {
            if arguments
                .iter()
                .any(|argument| argument == "--profile-samples" || argument == "--profile-no-vsync")
            {
                return Err(
                    "profile options require --profile-frames <absolute NEW JSON path>".into(),
                );
            }
            return Ok(None);
        }
        let mut destination = None;
        let mut samples = None;
        let mut no_vsync = false;
        let mut arguments = arguments.iter();
        while let Some(argument) = arguments.next() {
            let flag = argument
                .to_str()
                .ok_or("profile flags must be valid text")?;
            match flag {
                "--profile-frames" | "--profile-samples" => {
                    let value = arguments
                        .next()
                        .filter(|value| !value.to_string_lossy().starts_with("--"))
                        .ok_or_else(|| format!("{flag} requires a value"))?;
                    if flag == "--profile-frames" {
                        if destination.replace(PathBuf::from(value)).is_some() {
                            return Err("--profile-frames may only be supplied once".into());
                        }
                    } else {
                        let count = value
                            .to_str()
                            .and_then(|value| value.parse::<usize>().ok())
                            .filter(|count| (MIN_SAMPLES..=MAX_SAMPLES).contains(count))
                            .ok_or("--profile-samples must be an integer from 120 through 3600")?;
                        if samples.replace(count).is_some() {
                            return Err("--profile-samples may only be supplied once".into());
                        }
                    }
                }
                "--profile-no-vsync" if !no_vsync => no_vsync = true,
                "--profile-no-vsync" => {
                    return Err("--profile-no-vsync may only be supplied once".into());
                }
                _ => return Err(format!("unrecognized frame profile argument: {flag}")),
            }
        }
        let destination = destination.expect("profile mode requires its destination flag");
        validate_output(&destination)?;
        Ok(Some(Self {
            destination,
            samples: samples.unwrap_or(DEFAULT_SAMPLES),
            no_vsync,
        }))
    }
}

#[derive(Clone, Copy, Debug)]
struct WindowSnapshot {
    physical: [u32; 2],
    logical: [f32; 2],
    scale_factor: f32,
    backend_scale_factor: f32,
    present_mode: PresentMode,
    focused: bool,
    visible: bool,
}

impl WindowSnapshot {
    fn read(window: &Window) -> Self {
        Self {
            physical: [window.physical_width(), window.physical_height()],
            logical: [window.width(), window.height()],
            scale_factor: window.scale_factor(),
            backend_scale_factor: window.resolution.base_scale_factor(),
            present_mode: window.present_mode,
            focused: window.focused,
            visible: window.visible,
        }
    }

    fn json(self) -> Value {
        json!({
            "physical_pixels": self.physical,
            "logical_pixels": self.logical,
            "scale_factor": self.scale_factor,
            "backend_scale_factor": self.backend_scale_factor,
            "present_mode": format!("{:?}", self.present_mode),
            "focused": self.focused,
            "visible": self.visible,
        })
    }
}

#[derive(Resource)]
struct FrameRun {
    settings: Settings,
    started: Instant,
    frames: u32,
    warmup_frames: u32,
    warmup_elapsed: Duration,
    measurement_started: Option<Instant>,
    previous_frame: Option<Instant>,
    intervals: Vec<Duration>,
    first_window: Option<WindowSnapshot>,
    last_window: Option<WindowSnapshot>,
    occluded: Option<bool>,
    unfocused_intervals: u32,
    hidden_intervals: u32,
    occluded_intervals: u32,
    unknown_occlusion_intervals: u32,
    non_requested_size_intervals: u32,
    changed_window_intervals: u32,
    start_ticks: u64,
    start_real_elapsed: Duration,
    start_virtual_elapsed: Duration,
    start_fixed_overstep: Duration,
    real_intervals_above_cap: u32,
    real_time_above_cap: Duration,
    finished: bool,
}

impl FrameRun {
    fn new(settings: Settings) -> Self {
        let capacity = settings.samples;
        Self {
            settings,
            started: Instant::now(),
            frames: 0,
            warmup_frames: 0,
            warmup_elapsed: Duration::ZERO,
            measurement_started: None,
            previous_frame: None,
            intervals: Vec::with_capacity(capacity),
            first_window: None,
            last_window: None,
            occluded: None,
            unfocused_intervals: 0,
            hidden_intervals: 0,
            occluded_intervals: 0,
            unknown_occlusion_intervals: 0,
            non_requested_size_intervals: 0,
            changed_window_intervals: 0,
            start_ticks: 0,
            start_real_elapsed: Duration::ZERO,
            start_virtual_elapsed: Duration::ZERO,
            start_fixed_overstep: Duration::ZERO,
            real_intervals_above_cap: 0,
            real_time_above_cap: Duration::ZERO,
            finished: false,
        }
    }
}

/// Configure before app.run(), after the normal window/resources exist.
pub(super) fn configure(app: &mut App) -> Result<(), String> {
    configure_arguments(app, &std::env::args_os().skip(1).collect::<Vec<_>>())
}

fn configure_arguments(app: &mut App, arguments: &[OsString]) -> Result<(), String> {
    let Some(settings) = Settings::parse(arguments)? else {
        return Ok(());
    };
    {
        let world = app.world_mut();
        if !world.contains_resource::<ViewState>() {
            return Err("frame profiling requires the normal game view".into());
        }
        let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        let mut window = windows
            .single_mut(world)
            .map_err(|_| "frame profiling requires exactly one primary window")?;
        window
            .resolution
            .set_physical_resolution(REQUESTED_SIZE[0], REQUESTED_SIZE[1]);
        window.present_mode = if settings.no_vsync {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        };
        window.visible = true;
    }
    {
        let mut view = app.world_mut().resource_mut::<ViewState>();
        view.capture_active = true;
        view.set_speed(Speed::Normal);
        view.help = false;
        view.selected = None;
        view.hovered = None;
    }
    app.insert_resource(Probe::default())
        .insert_resource(FrameRun::new(settings))
        .add_systems(Last, measure_frame);
    Ok(())
}

#[derive(SystemParam)]
struct Runtime<'w, 's> {
    sim: Res<'w, Simulation>,
    windows: Query<'w, 's, (Entity, &'static Window), With<PrimaryWindow>>,
    real_time: Res<'w, Time<Real>>,
    virtual_time: Res<'w, Time<Virtual>>,
    fixed_time: Res<'w, Time<Fixed>>,
    adapter: Option<Res<'w, RenderAdapterInfo>>,
    winit: Option<Res<'w, WinitSettings>>,
    occlusion_events: MessageReader<'w, 's, WindowOccluded>,
}

fn warmup_finished(frames: u32, elapsed: Duration) -> bool {
    frames >= WARMUP_FRAMES && elapsed >= WARMUP_TIME
}

fn measure_frame(
    mut run: ResMut<FrameRun>,
    mut probe: ResMut<Probe>,
    mut runtime: Runtime,
    mut exit: MessageWriter<AppExit>,
) {
    if run.finished {
        return;
    }
    let now = Instant::now();
    run.frames += 1;
    let window = runtime
        .windows
        .single()
        .ok()
        .map(|(entity, window)| (entity, WindowSnapshot::read(window)));
    for event in runtime.occlusion_events.read() {
        if window.is_some_and(|(entity, _)| entity == event.window) {
            run.occluded = Some(event.occluded);
        }
    }
    let Some((_, window)) = window else {
        finish(
            &mut run,
            &mut probe,
            &runtime,
            now,
            "window_unavailable",
            &mut exit,
        );
        return;
    };
    run.last_window = Some(window);
    let elapsed = now.duration_since(run.started);
    if run.measurement_started.is_none() {
        run.warmup_frames = run.frames;
        run.warmup_elapsed = elapsed;
        if elapsed >= DEADLINE {
            finish(
                &mut run,
                &mut probe,
                &runtime,
                now,
                "deadline_exceeded",
                &mut exit,
            );
        } else if warmup_finished(run.frames, elapsed) {
            // The warmup boundary is the first endpoint, not a measured sample.
            run.measurement_started = Some(now);
            run.previous_frame = Some(now);
            run.first_window = Some(window);
            run.start_ticks = runtime.sim.0.ticks();
            run.start_real_elapsed = runtime.real_time.elapsed();
            run.start_virtual_elapsed = runtime.virtual_time.elapsed();
            run.start_fixed_overstep = runtime.fixed_time.overstep();
            probe.start(run.settings.samples);
        }
        return;
    }
    let previous = run
        .previous_frame
        .replace(now)
        .expect("measurement has a boundary");
    run.intervals.push(now.duration_since(previous));
    run.unfocused_intervals += u32::from(!window.focused);
    run.hidden_intervals += u32::from(!window.visible);
    run.occluded_intervals += u32::from(run.occluded == Some(true));
    run.unknown_occlusion_intervals += u32::from(run.occluded.is_none());
    run.non_requested_size_intervals += u32::from(window.physical != REQUESTED_SIZE);
    if let Some(first) = run.first_window {
        run.changed_window_intervals += u32::from(
            window.physical != first.physical
                || window.scale_factor != first.scale_factor
                || window.present_mode != first.present_mode,
        );
    }
    let excess = runtime
        .real_time
        .delta()
        .saturating_sub(runtime.virtual_time.max_delta());
    run.real_intervals_above_cap += u32::from(!excess.is_zero());
    run.real_time_above_cap += excess;
    if elapsed >= DEADLINE {
        finish(
            &mut run,
            &mut probe,
            &runtime,
            now,
            "deadline_exceeded",
            &mut exit,
        );
    } else if run.intervals.len() >= run.settings.samples {
        finish(&mut run, &mut probe, &runtime, now, "completed", &mut exit);
    }
}

fn finish(
    run: &mut FrameRun,
    probe: &mut Probe,
    runtime: &Runtime,
    now: Instant,
    status: &str,
    exit: &mut MessageWriter<AppExit>,
) {
    run.finished = true;
    probe.measuring = false;
    let result = write_json_new(
        &run.settings.destination,
        &report(run, probe, runtime, now, status),
    );
    match result {
        Ok(()) => {
            info!(
                "Frame profile {status}: {}",
                run.settings.destination.display()
            );
            exit.write(if status == "completed" {
                AppExit::Success
            } else {
                AppExit::error()
            });
        }
        Err(error) => {
            error!("Could not write frame profile: {error}");
            exit.write(AppExit::error());
        }
    }
}

fn report(run: &FrameRun, probe: &Probe, runtime: &Runtime, now: Instant, status: &str) -> Value {
    let measured = run
        .measurement_started
        .map(|start| now.duration_since(start));
    let elapsed_seconds: f64 = run.intervals.iter().map(Duration::as_secs_f64).sum();
    let mean_fps = (elapsed_seconds > 0.0).then(|| run.intervals.len() as f64 / elapsed_seconds);
    let ticks = runtime.sim.0.ticks().saturating_sub(run.start_ticks);
    let real_elapsed = run.measurement_started.map(|_| {
        runtime
            .real_time
            .elapsed()
            .saturating_sub(run.start_real_elapsed)
    });
    let virtual_elapsed = run.measurement_started.map(|_| {
        runtime
            .virtual_time
            .elapsed()
            .saturating_sub(run.start_virtual_elapsed)
    });
    let adapter = runtime.adapter.as_ref().map(|adapter| {
        json!({
            "name": adapter.name,
            "vendor_id": adapter.vendor,
            "device_id": adapter.device,
            "device_type": format!("{:?}", adapter.device_type),
            "backend": format!("{:?}", adapter.backend),
            "driver": adapter.driver,
            "driver_info": adapter.driver_info,
        })
    });
    let winit = runtime.winit.as_ref().map(|settings| {
        json!({
            "focused_mode": format!("{:?}", settings.focused_mode),
            "unfocused_mode": format!("{:?}", settings.unfocused_mode),
        })
    });
    let window = json!({
        "requested_physical_pixels": REQUESTED_SIZE,
        "requested_present_mode": if run.settings.no_vsync { "AutoNoVsync" } else { "AutoVsync" },
        "first_measured": run.first_window.map(WindowSnapshot::json),
        "last_observed": run.last_window.map(WindowSnapshot::json),
        "non_requested_size_intervals": run.non_requested_size_intervals,
        "changed_configuration_intervals": run.changed_window_intervals,
        "unfocused_intervals": run.unfocused_intervals,
        "hidden_intervals": run.hidden_intervals,
        "occluded_intervals": run.occluded_intervals,
        "unknown_occlusion_intervals": run.unknown_occlusion_intervals,
        "last_occlusion_event_state": run.occluded,
        "interpretation": "Focus, visibility, occlusion, size and present mode must be considered before comparing FPS; unknown occlusion is not evidence of visibility. Auto modes may choose a backend fallback that this probe cannot inspect.",
    });
    let simulation = json!({
        "speed": "Normal",
        "ticks_per_second": TICKS_PER_SECOND,
        "ticks_at_measurement_start": run.measurement_started.map(|_| run.start_ticks),
        "ticks_at_end": runtime.sim.0.ticks(),
        "ticks_advanced": run.measurement_started.map(|_| ticks),
        "wall_time_expected_ticks_estimate": measured.map(|duration| duration.as_secs_f64() * TICKS_PER_SECOND as f64),
        "real_clock_elapsed_ms": real_elapsed.map(|duration| duration.as_secs_f64() * 1000.0),
        "virtual_clock_elapsed_ms": virtual_elapsed.map(|duration| duration.as_secs_f64() * 1000.0),
        "virtual_max_delta_ms": runtime.virtual_time.max_delta().as_secs_f64() * 1000.0,
        "virtual_relative_speed": runtime.virtual_time.relative_speed_f64(),
        "real_intervals_above_virtual_cap": run.real_intervals_above_cap,
        "real_time_above_virtual_cap_ms": run.real_time_above_cap.as_secs_f64() * 1000.0,
        "fixed_step_ms": runtime.fixed_time.timestep().as_secs_f64() * 1000.0,
        "fixed_overstep_start_ms": run.measurement_started.map(|_| run.start_fixed_overstep.as_secs_f64() * 1000.0),
        "fixed_overstep_end_ms": runtime.fixed_time.overstep().as_secs_f64() * 1000.0,
        "dropped_ticks": Value::Null,
        "note": "A wall-clock/tick shortfall alone does not count dropped simulation ticks. Virtual time caps discard long wall-clock gaps; fixed-step overstep also affects endpoint tick counts.",
    });
    let cpu_hooks = json!({
        "simulation_update_ms": summarize(&probe.simulation_times),
        "scene_update_ms": summarize(&probe.scene_times),
        "simulation_calls": probe.simulation_calls,
        "simulation_ticks_recorded": probe.simulation_ticks,
        "scene_calls": probe.scene_calls,
        "static_rebuild_calls": probe.static_rebuild_calls,
        "dynamic_rebuild_calls": probe.dynamic_rebuild_calls,
        "omitted_simulation_timing_samples": probe.omitted_simulation_timings,
        "omitted_scene_timing_samples": probe.omitted_scene_timings,
        "note": "Scene timing covers the update_scene CPU system, not render-thread uploads, GPU execution or presentation.",
    });
    json!({
        "schema_version": 1,
        "kind": "interactive_frame_profile",
        "version": env!("CARGO_PKG_VERSION"),
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "os": std::env::consts::OS,
        "architecture": std::env::consts::ARCH,
        "hardware_role": "local diagnostics only; no reference PC has been selected",
        "recorded_unix_seconds": SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|time| time.as_secs()),
        "status": status,
        "completed": status == "completed",
        "measurement": "Instant intervals between consecutive main-world Last schedules; CPU main-loop pacing, not GPU or presented-frame timestamps",
        "gpu_timing_collected": false,
        "requested_samples": run.settings.samples,
        "measured_samples": run.intervals.len(),
        "total_main_frames": run.frames,
        "warmup": {
            "minimum_frames": WARMUP_FRAMES,
            "minimum_seconds": WARMUP_TIME.as_secs(),
            "actual_frames": run.warmup_frames,
            "actual_seconds": run.warmup_elapsed.as_secs_f64(),
        },
        "deadline_seconds": DEADLINE.as_secs(),
        "total_elapsed_seconds": now.duration_since(run.started).as_secs_f64(),
        "measurement_elapsed_seconds": measured.map(|duration| duration.as_secs_f64()),
        "frame_interval_ms": summarize(&run.intervals),
        "frame_samples_ms": run.intervals.iter().map(|duration| duration.as_secs_f64() * 1000.0).collect::<Vec<_>>(),
        "mean_main_loop_fps": mean_fps,
        "window": window,
        "winit_update_policy": winit,
        "adapter": adapter,
        "simulation": simulation,
        "cpu_hooks": cpu_hooks,
        "park": {
            "width": runtime.sim.0.width(),
            "height": runtime.sim.0.height(),
            "objects": runtime.sim.0.objects().len(),
            "guests": runtime.sim.0.guests().len(),
            "track_pieces": runtime.sim.0.circuit().map(|circuit| circuit.pieces.len()).unwrap_or(0),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    fn output() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "Parkbound frame test é {} {} {}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn configured_app(destination: &std::path::Path) -> (App, Entity) {
        let mut app = App::new();
        app.insert_resource(Simulation(park_sim::Park::new(32, 32)))
            .insert_resource(ViewState::default())
            .insert_resource(Time::<Real>::default())
            .insert_resource(Time::<Virtual>::default())
            .insert_resource(Time::<Fixed>::from_hz(30.0))
            .add_message::<WindowOccluded>()
            .add_message::<AppExit>();
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        configure_arguments(
            &mut app,
            &[
                "--profile-frames".into(),
                destination.as_os_str().to_owned(),
                "--profile-samples".into(),
                "120".into(),
                "--profile-no-vsync".into(),
            ],
        )
        .unwrap();
        (app, window)
    }

    #[test]
    fn disabled_mode_installs_nothing_and_does_not_need_game_resources() {
        let mut app = App::new();
        configure_arguments(&mut app, &[]).unwrap();
        assert!(!app.world().contains_resource::<Probe>());
        assert!(!app.world().contains_resource::<FrameRun>());
        app.update();
    }

    #[test]
    fn profile_arguments_require_a_new_absolute_json_and_bounded_samples() {
        let path = output();
        let base = [
            OsString::from("--profile-frames"),
            path.clone().into_os_string(),
        ];
        let settings = Settings::parse(&base).unwrap().unwrap();
        assert_eq!(settings.samples, 600);
        assert!(!settings.no_vsync);
        for count in [120, 3600] {
            let mut arguments = base.to_vec();
            arguments.extend([
                OsString::from("--profile-samples"),
                count.to_string().into(),
                "--profile-no-vsync".into(),
            ]);
            let settings = Settings::parse(&arguments).unwrap().unwrap();
            assert_eq!(settings.samples, count);
            assert!(settings.no_vsync);
        }
        for count in ["0", "119", "3601", "many"] {
            let mut arguments = base.to_vec();
            arguments.extend([OsString::from("--profile-samples"), count.into()]);
            assert!(Settings::parse(&arguments).is_err());
        }
        for suffix in [
            vec!["--unknown"],
            vec!["unexpected"],
            vec!["--save-dir", "elsewhere"],
            vec!["--profile-no-vsync", "--profile-no-vsync"],
            vec!["--profile-samples", "120", "--profile-samples", "120"],
            vec!["--profile-samples", "--profile-no-vsync"],
        ] {
            let mut arguments = base.to_vec();
            arguments.extend(suffix.into_iter().map(OsString::from));
            assert!(Settings::parse(&arguments).is_err());
        }
        let mut repeated_mode = base.to_vec();
        repeated_mode.extend(base.clone());
        assert!(Settings::parse(&repeated_mode).is_err());
        assert!(
            Settings::parse(&[
                "--profile-frames".into(),
                path.with_extension("JSON").into_os_string()
            ])
            .is_err()
        );
        for arguments in [
            vec!["--profile-frames", "relative.json"],
            vec!["--profile-frames"],
            vec!["--profile-samples", "120"],
            vec!["--profile-no-vsync"],
        ] {
            assert!(
                Settings::parse(
                    &arguments
                        .into_iter()
                        .map(OsString::from)
                        .collect::<Vec<_>>()
                )
                .is_err()
            );
        }
        fs::write(&path, b"existing evidence").unwrap();
        assert!(Settings::parse(&base).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"existing evidence");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn warmup_needs_both_frame_and_wall_time_thresholds() {
        assert!(!warmup_finished(119, Duration::from_secs(10)));
        assert!(!warmup_finished(120, Duration::from_millis(1999)));
        assert!(warmup_finished(120, Duration::from_secs(2)));
    }

    #[test]
    fn hooks_ignore_warmup_and_bound_their_retained_samples() {
        let mut probe = Probe::default();
        probe.record_simulation(Duration::from_millis(1), 1);
        probe.record_scene(Duration::from_millis(2), true, true);
        assert_eq!(probe.simulation_calls, 0);
        assert_eq!(probe.scene_calls, 0);
        probe.start(120);
        for _ in 0..MAX_HOOK_SAMPLES + 1 {
            probe.record_simulation(Duration::from_millis(1), 1);
            probe.record_scene(Duration::from_millis(2), false, true);
        }
        assert_eq!(probe.simulation_times.len(), MAX_HOOK_SAMPLES);
        assert_eq!(probe.scene_times.len(), MAX_HOOK_SAMPLES);
        assert_eq!(probe.omitted_simulation_timings, 1);
        assert_eq!(probe.omitted_scene_timings, 1);
        assert_eq!(probe.static_rebuild_calls, 0);
        assert_eq!(probe.dynamic_rebuild_calls, (MAX_HOOK_SAMPLES + 1) as u64);
    }

    #[test]
    fn configured_probe_reports_actual_size_and_unknown_adapter_without_gpu_claims() {
        let destination = output();
        let (mut app, window) = configured_app(&destination);
        assert!(app.world().resource::<ViewState>().capture_active);
        let requested = app.world().get::<Window>(window).unwrap();
        assert_eq!(requested.resolution.physical_size(), UVec2::new(1920, 1080));
        assert_eq!(requested.present_mode, PresentMode::AutoNoVsync);
        // Model an OS-constrained window; the report must retain this real
        // component size rather than asserting that 1080p was achieved.
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set_physical_resolution(1280, 720);
        {
            let mut run = app.world_mut().resource_mut::<FrameRun>();
            run.started = Instant::now() - Duration::from_secs(3);
            run.frames = 119;
        }
        app.update();
        assert!(app.world().resource::<Probe>().is_measuring());
        assert_eq!(app.world().resource::<FrameRun>().intervals.len(), 0);
        for _ in 0..120 {
            app.update();
        }
        let report: Value = serde_json::from_slice(&fs::read(&destination).unwrap()).unwrap();
        assert_eq!(report["completed"], true);
        assert_eq!(report["measured_samples"], 120);
        assert_eq!(report["frame_samples_ms"].as_array().unwrap().len(), 120);
        assert_eq!(
            report["window"]["last_observed"]["physical_pixels"],
            json!([1280, 720])
        );
        assert_eq!(report["window"]["non_requested_size_intervals"], 120);
        assert_eq!(report["window"]["unknown_occlusion_intervals"], 120);
        assert!(report["adapter"].is_null());
        assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(report["gpu_timing_collected"], false);
        assert!(!app.world().resource::<Probe>().is_measuring());
        fs::remove_file(destination).unwrap();
    }

    #[test]
    fn deadline_before_warmup_writes_an_explicit_empty_partial_report() {
        let destination = output();
        let (mut app, _) = configured_app(&destination);
        app.world_mut().resource_mut::<FrameRun>().started =
            Instant::now() - DEADLINE - Duration::from_secs(1);
        app.update();
        let report: Value = serde_json::from_slice(&fs::read(&destination).unwrap()).unwrap();
        assert_eq!(report["status"], "deadline_exceeded");
        assert_eq!(report["completed"], false);
        assert_eq!(report["measured_samples"], 0);
        assert!(report["frame_interval_ms"].is_null());
        assert!(report["mean_main_loop_fps"].is_null());
        assert!(report["simulation"]["ticks_advanced"].is_null());
        assert!(!app.world().resource::<Probe>().is_measuring());
        assert_eq!(app.should_exit(), Some(AppExit::error()));
        fs::remove_file(destination).unwrap();
    }

    #[test]
    fn deadline_wins_over_final_sample_and_reports_observed_occlusion() {
        let destination = output();
        let (mut app, window) = configured_app(&destination);
        {
            let mut component = app.world_mut().get_mut::<Window>(window).unwrap();
            component.focused = false;
            component.visible = false;
        }
        let snapshot = WindowSnapshot::read(app.world().get::<Window>(window).unwrap());
        let now = Instant::now();
        {
            let mut run = app.world_mut().resource_mut::<FrameRun>();
            run.started = now - DEADLINE - Duration::from_secs(1);
            run.measurement_started = Some(now - Duration::from_secs(1));
            run.previous_frame = Some(now - Duration::from_millis(1));
            run.first_window = Some(snapshot);
            run.intervals = vec![Duration::from_millis(10); 119];
        }
        app.world_mut().resource_mut::<Probe>().start(120);
        app.world_mut().write_message(WindowOccluded {
            window,
            occluded: true,
        });
        app.update();
        let report: Value = serde_json::from_slice(&fs::read(&destination).unwrap()).unwrap();
        assert_eq!(report["status"], "deadline_exceeded");
        assert_eq!(report["completed"], false);
        assert_eq!(report["measured_samples"], 120);
        assert_eq!(report["window"]["unfocused_intervals"], 1);
        assert_eq!(report["window"]["hidden_intervals"], 1);
        assert_eq!(report["window"]["occluded_intervals"], 1);
        assert_eq!(report["window"]["unknown_occlusion_intervals"], 0);
        assert_eq!(report["window"]["last_occlusion_event_state"], true);
        assert_eq!(app.should_exit(), Some(AppExit::error()));
        // A completed/failed run must not collect or rewrite later frames.
        let original = fs::read(&destination).unwrap();
        app.update();
        assert_eq!(fs::read(&destination).unwrap(), original);
        assert_eq!(app.world().resource::<FrameRun>().intervals.len(), 120);
        fs::remove_file(destination).unwrap();
    }
}
