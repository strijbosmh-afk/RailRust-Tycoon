//! File selection is presentation; completion can only alter the design library.
use std::{marker::PhantomData, path::PathBuf, rc::Rc, time::Instant};

use bevy::{
    input::keyboard::KeyboardInput,
    prelude::*,
    window::{PrimaryWindow, RawHandleWrapper},
};
use park_sim::TrackDesign;

use crate::{
    SaveSession, ViewState,
    design_file_dialog::{self, DialogRequest},
    design_library::DesignLibrary,
    design_store::DesignStore,
    design_transfer_file,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TransferRequest {
    Import,
    Export(TrackDesign),
}

/// NonSend makes the native modal run on the window's owning main thread.
#[derive(Default)]
pub(crate) struct DialogThread(PhantomData<Rc<()>>);

/// Discard fixed passes until Bevy's real clock has consumed a timestamp from
/// after the native modal. Pipelined rendering may deliver older timestamps first.
/// The selected game speed stays untouched throughout this temporary suppression.
#[derive(Resource, Default)]
pub(crate) struct DialogPause {
    resume_after: Option<Instant>,
}

impl DialogPause {
    pub(crate) fn suppresses_ticks(&self) -> bool {
        self.resume_after.is_some()
    }

    fn finished_at(&mut self, completed: Instant) {
        self.resume_after = Some(completed);
    }

    /// Called in Update, after every fixed pass for this real-time sample. The
    /// first sample crossing the completion watermark still contains modal time,
    /// so its entire fixed pass must be discarded before suppression is lifted.
    fn finish_update(&mut self, sampled: Option<Instant>) {
        if let (Some(completed), Some(sampled)) = (self.resume_after, sampled)
            && sampled >= completed
        {
            self.resume_after = None;
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct DialogInput<'w, 's> {
    windows: Query<'w, 's, &'static RawHandleWrapper, With<PrimaryWindow>>,
    keys: ResMut<'w, ButtonInput<KeyCode>>,
    mouse: ResMut<'w, ButtonInput<MouseButton>>,
    events: ResMut<'w, Messages<KeyboardInput>>,
    pause: ResMut<'w, DialogPause>,
    real_time: Res<'w, Time<Real>>,
}

fn owner_handle(wrapper: &RawHandleWrapper) -> Option<usize> {
    #[cfg(windows)]
    if let raw_window_handle::RawWindowHandle::Win32(handle) = wrapper.get_window_handle() {
        return Some(handle.hwnd.get() as usize);
    }
    #[cfg(not(windows))]
    let _ = wrapper;
    None
}

pub(crate) fn open_dialog(
    _main_thread: NonSend<DialogThread>,
    session: Res<SaveSession>,
    mut view: ResMut<ViewState>,
    mut library: ResMut<DesignLibrary>,
    mut input: DialogInput,
) {
    input.pause.finish_update(input.real_time.last_update());
    let Some(request) = library.transfer_request.take() else {
        return;
    };
    if !view.designs_open || view.capture_active || view.diagnostic_case.is_some() {
        library.transfer_pending = false;
        return;
    }
    let Some(save) = &session.store else {
        library.transfer_pending = false;
        library.notice = "Design folder unavailable. Check launch settings.".into();
        return;
    };
    let choice = match input.windows.single().ok().and_then(owner_handle) {
        Some(owner) => {
            let dialog = match &request {
                TransferRequest::Import => DialogRequest::Import,
                TransferRequest::Export(design) => DialogRequest::Export {
                    suggested_name: design_transfer_file::default_filename(design),
                },
            };
            design_file_dialog::choose_file(dialog, owner)
        }
        None => Err("The game window is unavailable for file selection.".into()),
    };
    complete_transfer(
        &DesignStore::new(save.directory().to_path_buf()),
        &mut library,
        request,
        choice,
    );
    input.keys.reset_all();
    input.mouse.reset_all();
    input.events.clear();
    // Include synchronous file I/O as well as the user's time in the dialog.
    input.pause.finished_at(Instant::now());
    view.modal_transition_pending = true;
    view.ui_pointer_consumed = true;
    view.dragging = false;
}

/// Used by native dialogs and isolated production proofs. Neither Park nor
/// ViewState is accepted, so transfer cannot load a draft, build or move money.
pub(crate) fn complete_transfer(
    store: &DesignStore,
    library: &mut DesignLibrary,
    request: TransferRequest,
    choice: Result<Option<PathBuf>, String>,
) -> bool {
    library.transfer_pending = false;
    library.transfer_request = None;
    let path = match choice {
        Ok(Some(path)) => path,
        Ok(None) => {
            library.notice =
                "File choice cancelled.\nSaved designs and the draft are unchanged.".into();
            return false;
        }
        Err(error) => {
            warn!("Track design file dialog: {error}");
            library.notice =
                "File selection failed.\nUse a .parktrack.json filename and try again.".into();
            return false;
        }
    };
    match request {
        TransferRequest::Import => {
            let design = match design_transfer_file::read_design(&path) {
                Ok(design) => design,
                Err(error) => {
                    warn!("Track design import: {error}");
                    library.notice = format!(
                        "Import refused: {}.\nSaved designs and the draft are unchanged.",
                        error.summary()
                    );
                    return false;
                }
            };
            match store.import_new(&design) {
                Ok(imported) => {
                    if library.refresh(store, Some(imported.name())) {
                        library.notice = format!(
                            "Imported \"{}\".\nSelect Load into draft when ready to edit.",
                            imported.name()
                        );
                    }
                    true
                }
                Err(error) => {
                    warn!(
                        "Track design import into {}: {error}",
                        store.directory().display()
                    );
                    library.notice = format!(
                        "Import refused: {}.\nSaved designs and the draft are unchanged.",
                        error.summary()
                    );
                    false
                }
            }
        }
        TransferRequest::Export(design) => match design_transfer_file::write_design(&path, &design)
        {
            Ok(()) => {
                library.notice = format!(
                    "Exported \"{}\".\nThe saved design and draft are unchanged.",
                    design.name()
                );
                true
            }
            Err(error) => {
                warn!("Track design export: {error}");
                library.notice = format!(
                    "Export failed: {}.\nThe saved design and draft are unchanged.",
                    error.summary()
                );
                false
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Simulation, tick_simulation};
    use park_sim::{Park, Speed};
    use std::{ffi::OsString, time::Duration};

    #[derive(Resource, Default)]
    struct FixedPasses(usize);

    fn count_fixed_passes(mut passes: ResMut<FixedPasses>) {
        passes.0 += 1;
    }

    #[test]
    fn queued_render_timestamps_cannot_replay_modal_time_or_change_selected_speed() {
        for speed in [Speed::Fastest, Speed::Paused] {
            let start = Instant::now();
            let step = Duration::from_millis(20);
            let completed = start + Duration::from_secs(10);
            let (sender, receiver) = bevy::time::create_time_channels();
            let park = Park::new(32, 32);
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .insert_resource(Time::<Real>::new(start))
                .insert_resource(Time::<Fixed>::from_duration(step))
                .insert_resource(receiver)
                .insert_resource(Simulation(park.clone()))
                .insert_resource(ViewState {
                    speed,
                    designs_open: true,
                    ..default()
                })
                // Deliberately invalid arguments avoid any default player directory.
                // No file request is queued and no native dialog is opened.
                .insert_resource(SaveSession::from_arguments([OsString::from("--save-dir")]))
                .init_resource::<DesignLibrary>()
                .init_resource::<DialogPause>()
                .init_resource::<FixedPasses>()
                .init_resource::<ButtonInput<KeyCode>>()
                .init_resource::<ButtonInput<MouseButton>>()
                .add_message::<KeyboardInput>()
                .insert_non_send_resource(DialogThread::default())
                .add_systems(FixedUpdate, (tick_simulation, count_fixed_passes))
                .add_systems(Update, open_dialog);

            sender.0.send(start).unwrap();
            app.update();
            app.world_mut()
                .resource_mut::<DialogPause>()
                .finished_at(completed);

            // The actual Bevy time channel can hold both an old render sample
            // and the sample containing the long modal wait. The first resumed
            // frame has no fixed pass, but must not clear suppression in Update.
            sender.0.send(start + Duration::from_millis(5)).unwrap();
            sender.0.send(completed).unwrap();
            app.update();
            assert_eq!(app.world().resource::<FixedPasses>().0, 0);
            assert!(app.world().resource::<DialogPause>().suppresses_ticks());
            assert_eq!(app.world().resource::<Simulation>().0, park);

            // Bevy consumes the later timestamp next. Even though this invokes
            // several fixed passes, none may charge money or advance the park.
            app.update();
            assert!(app.world().resource::<FixedPasses>().0 > 1);
            assert!(!app.world().resource::<DialogPause>().suppresses_ticks());
            assert_eq!(app.world().resource::<Simulation>().0, park);
            assert_eq!(app.world().resource::<ViewState>().speed, speed);

            let mut expected = park;
            expected.advance(speed);
            sender.0.send(completed + step).unwrap();
            app.update();
            assert_eq!(app.world().resource::<Simulation>().0, expected);
            assert_eq!(app.world().resource::<ViewState>().speed, speed);

            // A previously paused game also remains resumable after the modal.
            app.world_mut().resource_mut::<ViewState>().speed = Speed::Normal;
            expected.advance(Speed::Normal);
            sender.0.send(completed + step * 2).unwrap();
            app.update();
            assert_eq!(app.world().resource::<Simulation>().0, expected);
        }
    }
}
