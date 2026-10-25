//! Presentation-only sound. Park simulation, saves and RNG are never mutated.
use crate::{Simulation, ViewState, persistence::SaveSession};
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
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            muted: false,
            levels: [40, 30, 55, 65, 70],
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
        let value: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if value.version != 1 || value.levels.iter().any(|v| *v > 100) {
            return Err("Unsupported or invalid sound settings".into());
        }
        Ok(value)
    }
    fn save(self, path: &Path) -> Result<(), String> {
        use std::io::Write;
        let directory = path.parent().ok_or("Settings directory missing")?;
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let temp = directory.join(format!(
            "sound-settings-{}-{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let result = (|| {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .map_err(|e| e.to_string())?;
            file.write_all(&serde_json::to_vec_pretty(&self).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            drop(file);
            std::fs::rename(&temp, path).map_err(|e| e.to_string())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(temp);
        }
        result
    }
}
#[derive(Resource)]
pub(crate) struct SoundSettings {
    pub value: Settings,
    pub notice: String,
    pub test_requested: bool,
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
}
#[derive(Resource)]
struct Output {
    sender: Option<SyncSender<Message>>,
    worker: Option<JoinHandle<()>>,
    last: Option<MixState>,
    observed: Option<(u64, usize, u64, usize, String)>,
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
                    observed: None,
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
        .add_systems(Update, update.after(crate::ui::refresh));
}
fn update(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut settings: ResMut<SoundSettings>,
    time: Res<Time>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    output: Option<ResMut<Output>>,
) {
    let test = std::mem::take(&mut settings.test_requested);
    let Some(mut out) = output else {
        return;
    };
    let focused = window.iter().next().is_none_or(|w| w.focused);
    let coaster = sim
        .0
        .train()
        .filter(|t| {
            !matches!(
                t.phase,
                park_sim::TrainPhase::Station
                    | park_sim::TrainPhase::Closed
                    | park_sim::TrainPhase::Fault
            )
        })
        .map_or(0.0, |t| t.speed as f32 / 96.0);
    let state = MixState {
        settings: settings.value,
        focused,
        paused: view.speed == park_sim::Speed::Paused,
        crowd: (sim.0.guests().len() as f32 / 64.0).min(1.0),
        coaster,
        carousel: sim
            .0
            .carousels()
            .iter()
            .any(|r| r.phase == park_sim::CarouselPhase::Running),
    };
    if out.last != Some(state)
        && out
            .sender
            .as_ref()
            .is_some_and(|s| s.try_send(Message::State(state)).is_ok())
    {
        out.last = Some(state);
    }
    let snapshot = (
        sim.0.ticks(),
        sim.0.objects().len(),
        sim.0.all_rides().map(|r| r.revenue_cents).sum::<u64>()
            + sim
                .0
                .services()
                .iter()
                .map(|s| s.revenue_cents)
                .sum::<u64>(),
        sim.0
            .all_rides()
            .filter(|r| sim.0.ride_is_open(r.id))
            .count(),
        view.save_notice.clone(),
    );
    out.cooldown = (out.cooldown - time.delta_secs()).max(0.0);
    if focused && !view.capture_active {
        let mut cue = buttons
            .iter()
            .any(|i| *i == Interaction::Pressed)
            .then_some(Cue::Click);
        if test {
            cue = Some(Cue::Build);
        }
        if let Some(old) = &out.observed {
            if snapshot.0 >= old.0 {
                if snapshot.1 > old.1 {
                    cue = Some(Cue::Build);
                } else if snapshot.1 < old.1 {
                    cue = Some(Cue::Remove);
                } else if snapshot.3 < old.3 {
                    cue = Some(Cue::Alert);
                } else if snapshot.2 > old.2 && out.cooldown == 0.0 {
                    cue = Some(Cue::Sale);
                }
            }
            if snapshot.4 != old.4 && snapshot.4.to_lowercase().contains("failed") {
                cue = Some(Cue::Alert);
            }
        }
        if let Some(c) = cue {
            if let Some(s) = &out.sender {
                let _ = s.try_send(Message::Cue(c));
            }
            if c == Cue::Sale {
                out.cooldown = 0.4;
            }
        }
    }
    out.observed = Some(snapshot);
}
pub(crate) fn run(args: &[std::ffi::OsString]) -> Option<AppExit> {
    proof::run(args)
}
