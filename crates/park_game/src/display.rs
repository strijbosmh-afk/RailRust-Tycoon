//! Player display preferences. Park saves and simulation state are untouched.
use std::{ffi::OsString, io::Read};

use bevy::{
    prelude::*,
    window::{
        Monitor, MonitorSelection, PrimaryWindow, VideoModeSelection, WindowMode, WindowPosition,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    persistence::SaveSession,
    save_store::{SaveLock, SaveStore, StagedFile},
};

const FILE_NAME: &str = "display-settings.json";
const MAX_BYTES: u64 = 4096;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScaleChoice {
    #[default]
    Auto,
    Percent100,
    Percent125,
    Percent150,
    Percent200,
}

impl ScaleChoice {
    const ALL: [Self; 5] = [
        Self::Auto,
        Self::Percent100,
        Self::Percent125,
        Self::Percent150,
        Self::Percent200,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Percent100 => "100%",
            Self::Percent125 => "125%",
            Self::Percent150 => "150%",
            Self::Percent200 => "200%",
        }
    }

    fn factor(self, width: f32) -> f32 {
        match self {
            Self::Auto if width >= 3200.0 => 2.0,
            Self::Auto if width >= 2560.0 => 1.5,
            Self::Auto if width >= 1600.0 => 1.25,
            Self::Auto => 1.0,
            Self::Percent100 => 1.0,
            Self::Percent125 => 1.25,
            Self::Percent150 => 1.5,
            Self::Percent200 => 2.0,
        }
    }

    fn cycle(self, delta: i8) -> Self {
        let index = Self::ALL
            .iter()
            .position(|choice| *choice == self)
            .unwrap_or(0);
        Self::ALL[(index as isize + delta as isize).rem_euclid(Self::ALL.len() as isize) as usize]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ModeChoice {
    #[default]
    Windowed,
    Borderless,
    Fullscreen,
}

impl ModeChoice {
    const ALL: [Self; 3] = [Self::Windowed, Self::Borderless, Self::Fullscreen];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Borderless => "Borderless",
            Self::Fullscreen => "Fullscreen",
        }
    }

    fn cycle(self, delta: i8) -> Self {
        let index = Self::ALL
            .iter()
            .position(|choice| *choice == self)
            .unwrap_or(0);
        Self::ALL[(index as isize + delta as isize).rem_euclid(Self::ALL.len() as isize) as usize]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    version: u8,
    pub(crate) scale: ScaleChoice,
    pub(crate) mode: ModeChoice,
    /// Zero selects the current display. Positive values select a connected
    /// display in stable position/name order and fall back to primary.
    pub(crate) monitor: u8,
}

impl Settings {
    fn initial() -> Self {
        Self {
            version: 1,
            scale: ScaleChoice::Auto,
            mode: ModeChoice::Windowed,
            monitor: 0,
        }
    }

    fn read(path: &std::path::Path) -> Result<Self, String> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err("Display settings exceed 4 KiB".into());
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if value.version != 1 || value.monitor > 16 {
            return Err("Unsupported or invalid display settings".into());
        }
        Ok(value)
    }

    fn write(self, store: &SaveStore) -> Result<(), String> {
        store.check_directory(true).map_err(|e| e.to_string())?;
        let _lock = SaveLock::acquire(store.directory()).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(&self).map_err(|e| e.to_string())?;
        let mut staged =
            StagedFile::write(store.directory(), "display", &bytes).map_err(|e| e.to_string())?;
        staged
            .replace(&store.directory().join(FILE_NAME))
            .map_err(|e| e.to_string())
    }
}

#[derive(Resource)]
pub(crate) struct DisplaySettings {
    pub(crate) value: Settings,
    pub(crate) notice: String,
    pub(crate) available: Vec<String>,
    pub(crate) effective_scale: f32,
    store: Option<SaveStore>,
    mode_dirty: bool,
    write_blocked: bool,
}

impl DisplaySettings {
    pub(crate) fn new(session: &SaveSession, arguments: &[OsString], normal_launch: bool) -> Self {
        let store = normal_launch.then(|| session.store.clone()).flatten();
        let mut notice = String::new();
        let mut write_blocked = false;
        let mut value = store
            .as_ref()
            .map(|store| store.directory().join(FILE_NAME))
            .filter(|path| path.exists())
            .map(|path| Settings::read(&path))
            .transpose()
            .unwrap_or_else(|error| {
                notice = format!("Display settings could not load: {error}. Reset display settings to replace the file.");
                write_blocked = true;
                None
            })
            .unwrap_or_else(Settings::initial);
        if !normal_launch {
            // Captures remain deterministic unless the scale is explicitly requested.
            value = Settings::initial();
            value.scale = ScaleChoice::Percent100;
            if let Some(index) = arguments.iter().position(|arg| arg == "--capture-ui-scale") {
                value.scale = match arguments.get(index + 1).map(|arg| arg.to_string_lossy()) {
                    Some(value) if value == "auto" => ScaleChoice::Auto,
                    Some(value) if value == "100" => ScaleChoice::Percent100,
                    Some(value) if value == "125" => ScaleChoice::Percent125,
                    Some(value) if value == "150" => ScaleChoice::Percent150,
                    Some(value) if value == "200" => ScaleChoice::Percent200,
                    _ => ScaleChoice::Percent100,
                };
            }
        }
        if arguments.iter().any(|arg| arg == "--safe-windowed") {
            value.mode = ModeChoice::Windowed;
            value.monitor = 0;
            notice = "Safe windowed launch; use Settings to save a new display choice.".into();
        }
        Self {
            value,
            notice,
            available: Vec::new(),
            effective_scale: 1.0,
            store,
            mode_dirty: true,
            write_blocked,
        }
    }

    pub(crate) fn change_scale(&mut self, delta: i8) {
        self.value.scale = self.value.scale.cycle(delta);
        self.persist();
    }

    pub(crate) fn change_mode(&mut self, delta: i8) {
        self.value.mode = self.value.mode.cycle(delta);
        self.mode_dirty = true;
        self.persist();
    }

    pub(crate) fn change_monitor(&mut self, delta: i8) {
        let count = self.available.len().min(16) + 1;
        self.value.monitor =
            (self.value.monitor as isize + delta as isize).rem_euclid(count as isize) as u8;
        self.mode_dirty = true;
        self.persist();
    }

    pub(crate) fn reset(&mut self) {
        self.value = Settings::initial();
        self.mode_dirty = true;
        self.write_blocked = false;
        self.persist();
    }

    pub(crate) fn monitor_label(&self) -> &str {
        if self.value.monitor == 0 {
            "Current display"
        } else {
            self.available
                .get(self.value.monitor as usize - 1)
                .map(String::as_str)
                .unwrap_or("Unavailable — using primary")
        }
    }

    fn persist(&mut self) {
        if self.write_blocked {
            self.notice = "Changed for this session. Use Reset display settings to replace the unreadable file.".into();
            return;
        }
        self.notice = match &self.store {
            Some(store) => self.value.write(store).map_or_else(
                |error| format!("Display changed for this session; save failed: {error}"),
                |_| "Display settings saved outside park saves.".into(),
            ),
            None => "Preview setting; no player settings file changed.".into(),
        };
    }
}

pub(crate) fn effective_scale(choice: ScaleChoice, width: f32) -> f32 {
    // Reserve a useful map viewport between the left and right park chrome.
    let width_cap = ((width - 360.0) / 508.0).max(1.0);
    choice.factor(width).min(width_cap).clamp(1.0, 2.0)
}

pub(crate) fn update(
    mut settings: ResMut<DisplaySettings>,
    mut ui_scale: ResMut<UiScale>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    monitors: Query<(Entity, &Monitor)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::F11) {
        settings.reset();
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let mut connected: Vec<_> = monitors.iter().collect();
    connected.sort_by_key(|(_, monitor)| {
        (
            monitor.physical_position.x,
            monitor.physical_position.y,
            monitor.name.clone(),
        )
    });
    let names: Vec<_> = connected
        .iter()
        .enumerate()
        .map(|(index, (_, monitor))| {
            monitor
                .name
                .clone()
                .unwrap_or_else(|| format!("Display {}", index + 1))
        })
        .collect();
    if settings.available != names {
        settings.available = names;
        settings.mode_dirty = true;
    }
    if settings.mode_dirty {
        let selection = if settings.value.monitor == 0 {
            MonitorSelection::Current
        } else {
            connected
                .get(settings.value.monitor as usize - 1)
                .map_or(MonitorSelection::Primary, |(entity, _)| {
                    MonitorSelection::Entity(*entity)
                })
        };
        if settings.value.monitor != 0 && settings.value.monitor as usize > connected.len() {
            settings.notice = "Saved display is unavailable; using the primary display.".into();
        }
        window.mode = match settings.value.mode {
            ModeChoice::Windowed => WindowMode::Windowed,
            ModeChoice::Borderless => WindowMode::BorderlessFullscreen(selection),
            ModeChoice::Fullscreen => {
                WindowMode::Fullscreen(selection, VideoModeSelection::Current)
            }
        };
        if settings.value.mode == ModeChoice::Windowed && settings.value.monitor != 0 {
            window.position = WindowPosition::Centered(selection);
        }
        settings.mode_dirty = false;
    }
    let scale = effective_scale(settings.value.scale, window.width());
    if (ui_scale.0 - scale).abs() > 0.001 {
        ui_scale.0 = scale;
    }
    if (settings.effective_scale - scale).abs() > 0.001 {
        settings.effective_scale = scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::SaveSession;

    #[test]
    fn scale_keeps_a_map_viewport_and_responds_to_logical_width() {
        assert_eq!(effective_scale(ScaleChoice::Auto, 1024.0), 1.0);
        assert_eq!(effective_scale(ScaleChoice::Auto, 1920.0), 1.25);
        assert_eq!(effective_scale(ScaleChoice::Auto, 3840.0), 2.0);
        assert!(effective_scale(ScaleChoice::Percent200, 1024.0) < 1.5);
        assert_eq!(effective_scale(ScaleChoice::Percent200, 3840.0), 2.0);
    }

    #[test]
    fn choices_round_trip_without_touching_park_save() {
        let root = std::env::temp_dir().join(format!(
            "parkbound-display-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let session = SaveSession::from_arguments([
            OsString::from("--save-dir"),
            root.clone().into_os_string(),
        ]);
        let mut settings = DisplaySettings::new(&session, &[], true);
        settings.change_scale(2);
        settings.change_mode(1);
        assert_eq!(settings.value.scale, ScaleChoice::Percent125);
        assert_eq!(settings.value.mode, ModeChoice::Borderless);
        let loaded = DisplaySettings::new(&session, &[], true);
        assert_eq!(loaded.value.scale, settings.value.scale);
        assert_eq!(loaded.value.mode, settings.value.mode);
        let safe = DisplaySettings::new(&session, &[OsString::from("--safe-windowed")], true);
        assert_eq!(safe.value.mode, ModeChoice::Windowed);
        assert_eq!(
            DisplaySettings::new(&session, &[], true).value.mode,
            ModeChoice::Borderless
        );
        assert!(!root.join("park.park.json").exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn future_settings_are_retained_until_explicit_reset() {
        let root = std::env::temp_dir().join(format!(
            "parkbound-display-future-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join(FILE_NAME);
        let future = br#"{"version":2,"scale":"auto","mode":"windowed","monitor":0}"#;
        std::fs::write(&path, future).unwrap();
        let session = SaveSession::from_arguments([
            OsString::from("--save-dir"),
            root.clone().into_os_string(),
        ]);
        let mut settings = DisplaySettings::new(&session, &[], true);
        assert!(settings.write_blocked);
        settings.change_scale(1);
        assert_eq!(std::fs::read(&path).unwrap(), future);
        settings.reset();
        assert!(!settings.write_blocked);
        assert_eq!(Settings::read(&path).unwrap().version, 1);
        let _ = std::fs::remove_dir_all(root);
    }
}
