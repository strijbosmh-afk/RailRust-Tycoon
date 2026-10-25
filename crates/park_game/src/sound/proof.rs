//! Production export/restart proof. Audio checks never touch player park slots.
use super::*;
use std::{ffi::OsString, fs, io::Read};
const CLIPS: [(&str, usize, Option<usize>); 5] = [
    ("parkbound-mix.wav", 20, None),
    ("meadow-theme.wav", 8, Some(1)),
    ("park-ambience.wav", 8, Some(2)),
    ("ride-effects.wav", 8, Some(3)),
    ("interface-cues.wav", 3, Some(4)),
];
fn demo(seconds: usize, bus: Option<usize>) -> Vec<u8> {
    let mut mixer = Mixer::default();
    mixer.state.coaster = 0.85;
    mixer.state.carousel = true;
    if let Some(bus) = bus {
        mixer.state.settings.levels = [0; 5];
        mixer.state.settings.levels[0] = 50;
        mixer.state.settings.levels[bus] = 70;
    }
    let mut pcm = Vec::new();
    for second in 0..seconds {
        mixer.cue([Cue::Click, Cue::Build, Cue::Remove, Cue::Sale, Cue::Alert][second % 5]);
        pcm.extend(mixer.render(synth::RATE as usize));
    }
    synth::wav(&pcm)
}
fn proof_app(directory: &Path) -> App {
    let mut app = App::new();
    app.insert_resource(Simulation(crate::demonstration_park()))
        .insert_resource(ViewState {
            help: true,
            audio_view: true,
            speed: park_sim::Speed::Paused,
            ..Default::default()
        })
        .insert_resource(SoundSettings::at(
            Some(directory.join("sound-settings.json")),
            "Offline renderer; live output disabled",
        ))
        .add_systems(Startup, |mut c: Commands| crate::ui::audio::setup(&mut c))
        .add_systems(
            Update,
            (crate::ui::audio::apply, crate::ui::audio::refresh).chain(),
        );
    app.update();
    app
}
fn press(app: &mut App, control: crate::ui::audio::Control) -> Result<(), String> {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &crate::ui::audio::Control)>();
    let e = query
        .iter(world)
        .find_map(|(e, c)| (*c == control).then_some(e))
        .ok_or("Missing actual sound control")?;
    *app.world_mut().get_mut::<Interaction>(e).unwrap() = Interaction::Pressed;
    app.update();
    let after = app.world().resource::<SoundSettings>().value;
    app.update();
    if app.world().resource::<SoundSettings>().value != after {
        return Err("Held sound control repeated".into());
    }
    *app.world_mut().get_mut::<Interaction>(e).unwrap() = Interaction::None;
    app.update();
    Ok(())
}
fn write(directory: &Path) -> Result<(), String> {
    if directory.exists() {
        return Err("Sound writer requires a new directory".into());
    }
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let mut app = proof_app(directory);
    let before = app
        .world()
        .resource::<Simulation>()
        .0
        .to_save_bytes()
        .map_err(|e| e.to_string())?;
    for (index, step) in [(0, 10), (1, 10), (2, -10), (3, 10), (4, -10)] {
        press(&mut app, crate::ui::audio::Control::Level(index, step))?;
    }
    press(&mut app, crate::ui::audio::Control::Mute)?;
    press(&mut app, crate::ui::audio::Control::Mute)?;
    if before
        != app
            .world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .map_err(|e| e.to_string())?
    {
        return Err("Sound controls changed park bytes".into());
    }
    let settings = Settings::read(&directory.join("sound-settings.json"))?;
    if settings.levels != [50, 40, 45, 75, 60] || settings.muted {
        return Err("Actual controls did not persist expected levels".into());
    }
    for (name, seconds, bus) in CLIPS {
        fs::write(directory.join(name), demo(seconds, bus)).map_err(|e| e.to_string())?;
    }
    fs::write(directory.join("verification-written.txt"),"Actual sound controls persisted independently; park bytes unchanged. Original deterministic WAVs exported.\n").map_err(|e|e.to_string())
}
fn verify(directory: &Path) -> Result<(), String> {
    let mut app = proof_app(directory);
    let settings = app.world().resource::<SoundSettings>().value;
    if settings.levels != [50, 40, 45, 75, 60] || settings.muted {
        return Err("Sound levels changed after restart".into());
    }
    let before = app
        .world()
        .resource::<Simulation>()
        .0
        .to_save_bytes()
        .map_err(|e| e.to_string())?;
    app.update();
    if before
        != app
            .world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .map_err(|e| e.to_string())?
    {
        return Err("Sound refresh changed park bytes".into());
    }
    for (name, seconds, bus) in CLIPS {
        let mut actual = Vec::new();
        fs::File::open(directory.join(name))
            .map_err(|e| e.to_string())?
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut actual)
            .map_err(|e| e.to_string())?;
        if actual != demo(seconds, bus) {
            return Err(format!("Original PCM restart differs: {name}"));
        }
    }
    fs::write(directory.join("verification-sound.txt"),"PASS: actual UI levels/mute, bounded strict settings, independent park bytes, five exact original WAV re-renders after fresh restart.\n").map_err(|e|e.to_string())
}
pub(super) fn run(args: &[OsString]) -> Option<AppExit> {
    let flags: Vec<_> = args
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            *s == "--sound-check" || *s == "--verify-sound" || *s == "--sound-device-check"
        })
        .collect();
    if flags.is_empty() {
        return None;
    }
    let result = (|| -> Result<(), String> {
        if flags.len() != 1 {
            return Err("Use one sound proof flag".into());
        }
        let (index, flag) = flags[0];
        let path = args
            .get(index + 1)
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .ok_or("Sound proof requires an absolute directory")?;
        if flag == "--sound-check" {
            write(&path)
        } else if flag == "--verify-sound" {
            verify(&path)
        } else {
            if path.exists() {
                return Err("Device proof requires a new directory".into());
            }
            fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            #[cfg(windows)]
            native::device_check()?;
            #[cfg(not(windows))]
            return Err("Native audio is Windows only".into());
            fs::write(path.join("device-output.txt"),"PASS: Windows PCM device opened; stereo block submitted, completed and released. Physical listening is not established.\n").map_err(|e|e.to_string())
        }
    })();
    Some(match result {
        Ok(()) => AppExit::Success,
        Err(e) => {
            eprintln!("Sound check failed: {e}");
            AppExit::error()
        }
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn directory() -> PathBuf {
        std::env::temp_dir().join(format!(
            "Parkbound sound {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    #[test]
    fn sound_proof_restarts_real_controls_and_recreates_every_original_pcm_sample() {
        let p = directory();
        write(&p).unwrap();
        verify(&p).unwrap();
        assert!(p.starts_with(std::env::temp_dir()));
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn strict_bounded_settings_refuse_unknown_versions_values_fields_and_preserve_existing_files_on_failed_replace()
     {
        let p = directory();
        fs::create_dir_all(&p).unwrap();
        let file = p.join("settings.json");
        for bad in [
            r#"{"version":2,"muted":false,"levels":[40,30,55,65,70]}"#.to_string(),
            r#"{"version":1,"muted":false,"levels":[101,30,55,65,70]}"#.into(),
            r#"{"version":1,"muted":false,"levels":[40,30,55,65,70],"x":1}"#.into(),
            " ".repeat(4097),
        ] {
            fs::write(&file, bad).unwrap();
            assert!(Settings::read(&file).is_err());
        }
        let target = p.join("directory-target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), b"unchanged").unwrap();
        assert!(Settings::default().save(&target).is_err());
        assert_eq!(fs::read(target.join("keep")).unwrap(), b"unchanged");
        assert_eq!(fs::read_dir(&p).unwrap().count(), 2);
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn mute_focus_pause_and_independent_bus_levels_are_applied_to_actual_pcm() {
        let mut m = Mixer::default();
        m.state.settings.muted = true;
        assert!(m.render(4096).iter().all(|v| *v == 0));
        m.state.settings.muted = false;
        m.state.focused = false;
        assert!(m.render(4096).iter().all(|v| *v == 0));
        let mut moving = Mixer::default();
        moving.state.settings.levels = [80, 0, 0, 80, 0];
        moving.state.coaster = 1.0;
        assert!(moving.render(4096).iter().any(|v| *v != 0));
        let mut paused = Mixer::default();
        paused.state = moving.state;
        paused.state.paused = true;
        assert!(paused.render(4096).iter().all(|v| *v == 0));
    }
}
