//! Application save commands. A validated load replaces the park at one boundary.

use std::{ffi::OsString, path::PathBuf};

use bevy::prelude::*;
use park_sim::{Park, Speed, TerrainProfile};

use crate::{MAP_SIZE, Simulation, ViewState, save_store::SaveStore};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum SaveAction {
    Save,
    Load,
    Backup,
}

#[derive(Resource)]
pub(super) struct SaveSession {
    pub(super) store: Option<SaveStore>,
    problem: Option<String>,
}

impl SaveSession {
    pub(super) fn from_arguments(arguments: impl IntoIterator<Item = OsString>) -> Self {
        let arguments: Vec<_> = arguments.into_iter().collect();
        let flags: Vec<_> = arguments
            .iter()
            .enumerate()
            .filter(|(_, value)| *value == "--save-dir")
            .map(|(index, _)| index)
            .collect();
        let directory = match flags.as_slice() {
            [] => SaveStore::user_directory().map_err(|error| error.to_string()),
            [index] => arguments
                .get(index + 1)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .ok_or_else(|| "--save-dir requires one absolute directory path".into()),
            _ => Err("--save-dir may be supplied only once".into()),
        };
        match directory {
            Ok(directory) => Self {
                store: Some(SaveStore::new(directory)),
                problem: None,
            },
            Err(problem) => Self {
                store: None,
                problem: Some(problem),
            },
        }
    }

    #[cfg(test)]
    fn at(directory: PathBuf) -> Self {
        Self {
            store: Some(SaveStore::new(directory)),
            problem: None,
        }
    }
}

pub(super) fn apply_save_action(
    session: Res<SaveSession>,
    mut sim: ResMut<Simulation>,
    mut view: ResMut<ViewState>,
) {
    let Some(action) = view.save_pending.take() else {
        return;
    };
    if view.help || view.capture_active {
        return;
    }
    perform_action(&session, &mut sim.0, &mut view, action);
}

/// Shared by normal controls and opt-in persistence diagnostics.
/// Errors update only the notice; they never replace or pause the live park.
pub(super) fn perform_action(
    session: &SaveSession,
    park: &mut Park,
    view: &mut ViewState,
    action: SaveAction,
) -> bool {
    if view.diagnostic_case.is_some() {
        view.save_notice = "Render preview only\nEditing and saves disabled".into();
        return false;
    }
    let Some(store) = &session.store else {
        warn!("Save folder unavailable: {:?}", session.problem);
        view.save_notice = "Save folder unavailable.\nCheck launch settings.".into();
        return false;
    };
    if action == SaveAction::Save {
        return match store.save(park) {
            Ok(()) => {
                let seconds = park.elapsed_seconds();
                view.save_notice = format!(
                    "Saved at {:02}:{:02}\nF9 loads this park",
                    seconds / 60,
                    seconds % 60
                );
                true
            }
            Err(error) => {
                warn!("Save failed: {error}");
                view.save_notice = format!("Save failed.\n{}", error.summary());
                false
            }
        };
    }
    let loaded = match action {
        SaveAction::Load => store.load(),
        SaveAction::Backup => store.load_backup(),
        SaveAction::Save => unreachable!(),
    };
    match loaded {
        Ok(loaded) => {
            if loaded.width() != MAP_SIZE || loaded.height() != MAP_SIZE {
                view.save_notice = "Load rejected.\nRequires a 32 x 32 park.".into();
                return false;
            }
            let (terrain_profile, terrain_custom) = terrain_style(&loaded);
            let seconds = loaded.elapsed_seconds();
            let notice = if action == SaveAction::Backup {
                "Backup loaded / paused\nSpace to continue".into()
            } else {
                format!(
                    "Loaded {:02}:{:02} / paused\nSpace to continue",
                    seconds / 60,
                    seconds % 60
                )
            };
            let capture_active = view.capture_active;
            *park = loaded;
            *view = ViewState {
                speed: Speed::Paused,
                resume_speed: Speed::Normal,
                scene_dirty: true,
                terrain_profile,
                terrain_custom,
                capture_active,
                // Loading resets selection. Never let the same button press
                // select a surface in the newly loaded park behind the UI.
                ui_pointer_consumed: true,
                save_notice: notice,
                ..default()
            };
            true
        }
        Err(error) => {
            warn!("Load failed: {error}");
            view.save_notice = format!("Load failed.\n{}", error.summary());
            false
        }
    }
}

fn terrain_style(park: &Park) -> (TerrainProfile, bool) {
    if park.max_terrain_height() == 0 {
        return (TerrainProfile::Flat, false);
    }
    let mut comparison = park.clone();
    let matches = comparison
        .try_apply_terrain_profile(TerrainProfile::Terraced)
        .is_ok()
        && comparison == *park;
    (TerrainProfile::Terraced, !matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    fn session() -> SaveSession {
        SaveSession::at(std::env::temp_dir().join(format!(
            "Parkbound application save test é {} {}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        )))
    }

    fn fixture() -> Park {
        let mut park = Park::new(MAP_SIZE, MAP_SIZE);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(crate::CROWD_SEED).unwrap();
        park.install_train_fixture().unwrap();
        for _ in 0..731 {
            park.advance(Speed::Normal);
        }
        park.set_train_open(false);
        park
    }

    #[test]
    fn load_restores_exact_park_paused_and_clears_stale_view_and_requests() {
        let session = session();
        let saved = fixture();
        session.store.as_ref().unwrap().save(&saved).unwrap();
        let mut park = fixture();
        for _ in 0..300 {
            park.advance(Speed::Fastest);
        }
        let mut view = ViewState {
            speed: Speed::Fastest,
            selected: Some(park_sim::SurfaceId::Object(park.objects()[0].id)),
            remove_path_pending: true,
            toggle_train_pending: true,
            terrain_change_pending: true,
            rotation: 3,
            zoom: 3.5,
            dragging: true,
            ..default()
        };
        assert!(perform_action(
            &session,
            &mut park,
            &mut view,
            SaveAction::Load
        ));
        assert_eq!(park, saved);
        assert_eq!(view.speed, Speed::Paused);
        assert_eq!(view.resume_speed, Speed::Normal);
        assert_eq!(view.selected, None);
        assert_eq!(view.rotation, 0);
        assert_eq!(view.zoom, 1.0);
        assert!(view.scene_dirty && view.ui_pointer_consumed);
        assert!(!view.remove_path_pending && !view.toggle_train_pending);
        assert!(!view.terrain_change_pending && !view.dragging && !view.terrain_custom);
        assert_eq!(view.terrain_profile, TerrainProfile::Terraced);
        let mut uninterrupted = saved;
        view.pause();
        for _ in 0..3_000 {
            park.advance(view.speed);
            uninterrupted.advance(Speed::Normal);
        }
        assert_eq!(park, uninterrupted);
    }

    #[test]
    fn failed_or_unsupported_load_preserves_live_park_and_speed() {
        let session = session();
        let mut park = fixture();
        let initial = park.clone();
        let mut view = ViewState {
            speed: Speed::Fast,
            rotation: 2,
            ..default()
        };
        assert!(!perform_action(
            &session,
            &mut park,
            &mut view,
            SaveAction::Load
        ));
        assert_eq!(park, initial);
        assert_eq!(view.speed, Speed::Fast);
        assert_eq!(view.rotation, 2);
        session
            .store
            .as_ref()
            .unwrap()
            .save(&Park::new(4, 5))
            .unwrap();
        assert!(!perform_action(
            &session,
            &mut park,
            &mut view,
            SaveAction::Load
        ));
        assert_eq!(park, initial);
        assert_eq!(view.speed, Speed::Fast);
        assert!(view.save_notice.contains("32 x 32"));
        std::fs::write(session.store.as_ref().unwrap().save_path(), b"{broken").unwrap();
        assert!(!perform_action(
            &session,
            &mut park,
            &mut view,
            SaveAction::Load
        ));
        assert_eq!(park, initial);
        assert_eq!(view.speed, Speed::Fast);
    }

    #[test]
    fn explicit_backup_load_and_custom_terrain_are_preserved() {
        let session = session();
        let mut saved = fixture();
        saved
            .set_terrain_height(park_sim::TilePos { x: 0, y: 0 }, 3)
            .unwrap();
        session.store.as_ref().unwrap().save(&saved).unwrap();
        let mut newer = saved.clone();
        newer.advance(Speed::Fastest);
        session.store.as_ref().unwrap().save(&newer).unwrap();
        let mut view = ViewState::default();
        assert!(perform_action(
            &session,
            &mut newer,
            &mut view,
            SaveAction::Backup
        ));
        assert_eq!(newer, saved);
        assert!(view.terrain_custom);
        assert!(view.save_notice.starts_with("Backup loaded"));
    }

    #[test]
    fn command_precedence_cannot_save_over_a_simultaneously_requested_load() {
        let mut view = ViewState::default();
        for action in [
            SaveAction::Load,
            SaveAction::Save,
            SaveAction::Backup,
            SaveAction::Save,
        ] {
            view.request_save(action);
        }
        assert_eq!(view.save_pending, Some(SaveAction::Backup));
        for blocked in [false, true] {
            let session = session();
            let path = session.store.as_ref().unwrap().save_path();
            let mut app = App::new();
            app.insert_resource(session)
                .insert_resource(Simulation(fixture()))
                .insert_resource(ViewState {
                    save_pending: Some(SaveAction::Save),
                    help: blocked,
                    capture_active: !blocked,
                    ..default()
                })
                .add_systems(Update, apply_save_action);
            app.update();
            assert!(!path.exists());
            assert_eq!(app.world().resource::<ViewState>().save_pending, None);
        }
    }

    #[test]
    fn save_directory_override_is_absolute_unique_and_does_not_touch_disk() {
        let path = std::env::temp_dir().join("Parkbound override é");
        let session = SaveSession::from_arguments([
            OsString::from("--save-dir"),
            path.clone().into_os_string(),
        ]);
        assert_eq!(session.store.unwrap().directory(), path);
        for args in [
            vec!["--save-dir"],
            vec!["--save-dir", "relative/path"],
            vec!["--save-dir", "--capture-frame"],
            vec!["--save-dir", "C:/one", "--save-dir", "C:/two"],
        ] {
            assert!(
                SaveSession::from_arguments(args.into_iter().map(OsString::from))
                    .store
                    .is_none()
            );
        }
    }

    #[test]
    fn rendering_preview_rejects_every_save_action_without_touching_disk() {
        let session = session();
        let directory = session.store.as_ref().unwrap().directory().to_owned();
        let mut park = fixture();
        let original = park.clone();
        let mut view = ViewState {
            diagnostic_case: Some(park_sim::diagnostics::DiagnosticCase::SaveScale),
            ..default()
        };
        for action in [SaveAction::Save, SaveAction::Load, SaveAction::Backup] {
            view.request_save(action);
            assert!(view.save_pending.is_none());
            assert!(!perform_action(&session, &mut park, &mut view, action));
            assert_eq!(park, original);
            assert!(!directory.exists());
        }
    }
}
