//! Presentation-only sound. Park simulation, saves and RNG are never mutated.
use crate::{Simulation, ViewState, persistence::SaveSession, world};
use bevy::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self, SyncSender},
    },
    thread::JoinHandle,
};
#[cfg(windows)]
mod native;
mod proof;
mod synth;
pub(crate) use synth::Cue;
use synth::{MixState, Mixer};
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub version: u8,
    pub muted: bool,
    pub levels: [u8; 5],
    #[serde(default = "default_reactions")]
    pub reactions: u8,
}
fn default_reactions() -> u8 {
    60
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 2,
            muted: false,
            levels: [40, 30, 55, 65, 70],
            reactions: default_reactions(),
        }
    }
}
impl Settings {
    fn read(path: &Path) -> Result<Self, String> {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(4097)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 4096 {
            return Err("Sound settings exceed 4 KiB".into());
        }
        let mut value: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if ![1, 2].contains(&value.version)
            || value.levels.iter().any(|v| *v > 100)
            || value.reactions > 100
        {
            return Err("Unsupported or invalid sound settings".into());
        }
        value.version = 2;
        Ok(value)
    }
    fn save(self, path: &Path) -> Result<(), String> {
        let directory = path.parent().ok_or("Settings directory missing")?;
        let store = crate::save_store::SaveStore::new(directory.into());
        store.check_directory(true).map_err(|e| e.to_string())?;
        let _lock = crate::save_store::SaveLock::acquire(directory).map_err(|e| e.to_string())?;
        let mut staged = crate::save_store::StagedFile::write(
            directory,
            "sound",
            &serde_json::to_vec_pretty(&self).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        staged.replace(path).map_err(|e| e.to_string())
    }
}
#[derive(Resource)]
pub(crate) struct SoundSettings {
    pub value: Settings,
    pub notice: String,
    pub test_requested: bool,
    pub restart_requested: bool,
    path: Option<PathBuf>,
    status: Arc<Mutex<String>>,
}
impl SoundSettings {
    fn at(path: Option<PathBuf>, status: &str) -> Self {
        let (value, notice) = match path
            .as_ref()
            .filter(|p| p.exists())
            .map(|p| Settings::read(p))
        {
            Some(Ok(s)) => (s, "Settings loaded; each level is independent.".into()),
            Some(Err(e)) => (Settings::default(), format!("Using defaults: {e}")),
            None => (
                Settings::default(),
                "Original music and effects; settings save separately from the park.".into(),
            ),
        };
        Self {
            value,
            notice,
            test_requested: false,
            restart_requested: false,
            path,
            status: Arc::new(Mutex::new(status.into())),
        }
    }
    pub fn backend(&self) -> String {
        self.status
            .lock()
            .map_or_else(|_| "Audio status unavailable".into(), |s| s.clone())
    }
    pub fn persist(&mut self) {
        self.notice = match self.path.as_ref() {
            Some(p) => self.value.save(p).map_or_else(
                |e| format!("Changed for this session; could not save: {e}"),
                |_| "Sound settings saved. Park data is unchanged.".into(),
            ),
            None => "Preview only; no settings file is written.".into(),
        };
    }
}
enum Message {
    State(MixState),
    Cue(Cue),
    Quit,
    Restart,
}
#[derive(Resource)]
struct Output {
    sender: Option<SyncSender<Message>>,
    worker: Option<JoinHandle<()>>,
    last: Option<MixState>,
    cooldown: f32,
}
impl Drop for Output {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(Message::Quit);
            drop(sender);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
pub(crate) fn configure(app: &mut App, args: &[std::ffi::OsString]) {
    let quiet = args.iter().any(|s| {
        s == "--capture-frame"
            || s.to_string_lossy().starts_with("--profile-")
            || s.to_string_lossy().starts_with("--frame-")
            || s.to_string_lossy().starts_with("--render-")
    });
    let path = if quiet {
        None
    } else {
        app.world()
            .get_resource::<SaveSession>()
            .and_then(|s| s.store.as_ref())
            .map(|s| s.directory().join("sound-settings.json"))
    };
    let settings = SoundSettings::at(
        path,
        if quiet {
            "Controlled capture / audio output disabled"
        } else {
            "Opening audio device..."
        },
    );
    #[cfg(windows)]
    if !quiet {
        let (sender, receiver) = mpsc::sync_channel(32);
        let status = settings.status.clone();
        let worker = std::thread::Builder::new()
            .name("Parkbound sound".into())
            .spawn(move || native::stream(receiver, status));
        match worker {
            Ok(worker) => {
                app.insert_resource(Output {
                    sender: Some(sender),
                    worker: Some(worker),
                    last: None,
                    cooldown: 0.0,
                });
            }
            Err(e) => {
                if let Ok(mut s) = settings.status.lock() {
                    *s = format!("Audio unavailable: {e}");
                }
            }
        }
    }
    app.insert_resource(settings)
        .init_resource::<Feedback>()
        .add_systems(Startup, setup_captions)
        .add_systems(Update, update.after(crate::ui::refresh));
}
type FeedbackResources<'w, 's> = (
    ResMut<'w, Feedback>,
    Option<Res<'w, crate::accessibility::Preferences>>,
    Query<'w, 's, (&'static mut Text, &'static mut Node), With<Caption>>,
);
type DeviceInputs<'w, 's> = (
    Query<'w, 's, &'static Window, With<bevy::window::PrimaryWindow>>,
    Query<'w, 's, &'static Interaction, (Changed<Interaction>, With<Button>)>,
);
fn update(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut settings: ResMut<SoundSettings>,
    time: Res<Time>,
    inputs: DeviceInputs<'_, '_>,
    output: Option<ResMut<Output>>,
    feedback_resources: FeedbackResources<'_, '_>,
) {
    let (window, buttons) = inputs;
    let (mut feedback, preferences, mut captions) = feedback_resources;
    let test = std::mem::take(&mut settings.test_requested);
    feedback.remaining = (feedback.remaining - time.delta_secs()).max(0.0);
    let snapshot = (
        view.park_generation,
        sim.0.objects().len(),
        sim.0.all_rides().map(|r| r.completed).sum::<u64>(),
        sim.0
            .all_rides()
            .filter(|r| sim.0.ride_is_open(r.id))
            .count(),
        view.save_notice.clone(),
        view.successful_saves,
        sim.0.park_life_status().resolved,
    );
    let cue = feedback.last.as_ref().and_then(|old| {
        if old.0 != snapshot.0 {
            return None;
        }
        if old.1 < snapshot.1 {
            Some((Cue::Build, "Construction completed"))
        } else if old.1 > snapshot.1 {
            Some((Cue::Remove, "Object removed"))
        } else if old.3 > snapshot.3 {
            Some((Cue::Alert, "A ride closed. Check ride operations"))
        } else if old.4 != snapshot.4 && snapshot.4.to_lowercase().contains("failed") {
            Some((Cue::Alert, "Save failed. Check the park menu"))
        } else if old.5 != snapshot.5 {
            Some((Cue::Sale, "Park saved"))
        } else if old.6 < snapshot.6 {
            Some((Cue::Staff, "Staff resolved a park incident"))
        } else if old.2 < snapshot.2 && feedback.remaining == 0.0 {
            Some((Cue::Cheer, "Guests completed a ride"))
        } else {
            None
        }
    });
    feedback.last = Some(snapshot);
    if let Some((_, caption)) = cue {
        feedback.caption = caption.into();
        feedback.remaining = 3.0;
    }
    for (mut label, mut node) in &mut captions {
        label.0 =
            if preferences.as_ref().is_none_or(|p| p.value.captions) && feedback.remaining > 0.0 {
                feedback.caption.clone()
            } else {
                String::new()
            };
        node.display = if label.0.is_empty() {
            Display::None
        } else {
            Display::Flex
        };
    }
    let Some(mut out) = output else {
        return;
    };
    if settings.restart_requested
        && out
            .sender
            .as_ref()
            .is_some_and(|sender| sender.try_send(Message::Restart).is_ok())
    {
        settings.restart_requested = false;
    }
    if let Some((cue, _)) = cue
        && !view.front_modal
        && !view.capture_active
        && let Some(sender) = &out.sender
    {
        let _ = sender.try_send(Message::Cue(cue));
    }
    let focused = window.iter().next().is_none_or(|w| w.focused);
    let camera =
        view.pan + Vec2::Y * f32::from(sim.0.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    let spatial = |point: Option<Vec2>| {
        point.map_or((0.0, 0.0), |point| {
            let delta = (point - camera) * view.zoom;
            let pan = (delta.x / 440.0).clamp(-1.0, 1.0);
            let distance = delta.length() / 660.0;
            (pan, 1.0 / (1.0 + distance * distance))
        })
    };
    let (coaster, coaster_pan, coaster_presence, ride_model) = sim
        .0
        .coaster_ids()
        .into_iter()
        .flat_map(|id| {
            sim.0
                .coaster_trains(id)
                .into_iter()
                .map(move |train| (id, train))
        })
        .filter_map(|(id, train)| {
            if matches!(
                train.phase,
                park_sim::TrainPhase::Station
                    | park_sim::TrainPhase::Closed
                    | park_sim::TrainPhase::Fault
            ) {
                return None;
            }
            let (pan, presence) = spatial(world::leading_train_focus(
                &sim.0,
                id,
                train.number,
                view.rotation,
            ));
            Some((
                train.speed as f32 / 96.0,
                pan,
                presence,
                sim.0.ride_model(id),
            ))
        })
        .max_by(|a, b| (a.0 * a.2).total_cmp(&(b.0 * b.2)))
        .unwrap_or((0.0, 0.0, 0.0, None));
    let running_carousel = sim
        .0
        .carousels()
        .iter()
        .find(|ride| ride.phase == park_sim::CarouselPhase::Running);
    let (carousel_pan, carousel_presence) = spatial(
        running_carousel.and_then(|ride| world::ride_focus(&sim.0, ride.ride.id, view.rotation)),
    );
    let state = MixState {
        settings: settings.value,
        focused,
        paused: view.speed == park_sim::Speed::Paused,
        crowd: (sim.0.guests().len() as f32 / 64.0).min(1.0),
        coaster,
        ride_model,
        flat_model: running_carousel.and_then(|r| sim.0.ride_model(r.ride.id)),
        coaster_pan,
        coaster_presence,
        carousel: running_carousel.is_some(),
        carousel_pan,
        carousel_presence,
        rain: sim.0.park_life_status().weather == park_sim::Weather::Rain,
    };
    if out.last != Some(state)
        && out
            .sender
            .as_ref()
            .is_some_and(|s| s.try_send(Message::State(state)).is_ok())
    {
        out.last = Some(state);
    }
    out.cooldown = (out.cooldown - time.delta_secs()).max(0.0);
    if focused && !view.capture_active && cue.is_none() {
        let cue = if test {
            Some(Cue::Build)
        } else if buttons.iter().any(|i| *i == Interaction::Pressed) && out.cooldown == 0.0 {
            Some(Cue::Click)
        } else {
            None
        };
        if let Some(cue) = cue
            && let Some(sender) = &out.sender
        {
            let _ = sender.try_send(Message::Cue(cue));
            out.cooldown = 0.12;
        }
    }
}
#[derive(Resource, Default)]
struct Feedback {
    last: Option<(u64, usize, u64, usize, String, u64, u64)>,
    caption: String,
    remaining: f32,
}
#[derive(Component)]
struct Caption;
fn setup_captions(mut commands: Commands) {
    commands.spawn((
        Caption,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: percent(30),
            top: px(78),
            padding: UiRect::all(px(4)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GlobalZIndex(45),
    ));
}
pub(crate) fn run(args: &[std::ffi::OsString]) -> Option<AppExit> {
    proof::run(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captures_and_frame_diagnostics_never_start_audio_or_load_player_settings() {
        for flag in ["--capture-frame", "--profile-frames", "--render-case"] {
            let mut app = App::new();
            configure(&mut app, &[flag.into(), "isolated".into()]);
            assert!(!app.world().contains_resource::<Output>());
            let settings = app.world().resource::<SoundSettings>();
            assert!(settings.path.is_none());
            assert!(settings.backend().contains("disabled"));
        }
    }
}
