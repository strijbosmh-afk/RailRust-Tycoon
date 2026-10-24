//! A local design library. Loading is a draft edit; only Apply buys track.
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};
use park_sim::{MAX_TRACK_DESIGN_NAME_CHARS, Park, TilePos, TrackDesign};

use crate::{
    SaveAction, SaveSession, Simulation, ViewState, design_store::DesignStore, track_tools,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DesignCommand {
    Open,
    Close,
    Select(usize),
    Browse(i8),
    NameFocus,
    SaveNew,
    UpdateSelected,
    DeleteSelected,
    Confirm,
    Cancel,
    LoadDraft,
    MoveX(i8),
    MoveY(i8),
    Height(i8),
    Turn,
}

#[derive(Resource)]
pub(crate) struct DesignLibrary {
    pub entries: Vec<TrackDesign>,
    pub selected: Option<usize>,
    pub name: String,
    pub name_focused: bool,
    pub anchor: TilePos,
    pub height: u8,
    pub rotation: u8,
    pub notice: String,
    pub confirmation: Option<DesignCommand>,
    pub available: bool,
    select_all: bool,
    confirmed_target: Option<TrackDesign>,
    confirmed_replacement: Option<TrackDesign>,
}

impl Default for DesignLibrary {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            selected: None,
            name: String::new(),
            name_focused: false,
            anchor: TilePos { x: 0, y: 0 },
            height: 8,
            rotation: 0,
            notice: "Save a draft for use in any park. Loading is free.".into(),
            confirmation: None,
            available: false,
            select_all: false,
            confirmed_target: None,
            confirmed_replacement: None,
        }
    }
}

impl DesignLibrary {
    pub(crate) fn selected_design(&self) -> Option<&TrackDesign> {
        self.selected.and_then(|index| self.entries.get(index))
    }

    fn select(&mut self, index: usize) {
        let Some(design) = self.entries.get(index) else {
            return;
        };
        self.selected = Some(index);
        self.name = design.name().into();
        (self.anchor, self.height) = design.anchor();
        self.rotation = 0;
        self.name_focused = false;
        self.select_all = false;
        self.cancel_confirmation();
    }

    fn cancel_confirmation(&mut self) {
        self.confirmation = None;
        self.confirmed_target = None;
        self.confirmed_replacement = None;
    }

    fn refresh(&mut self, store: &DesignStore, preferred: Option<&str>) -> bool {
        match store.list() {
            Ok(entries) => {
                self.entries = entries;
                self.available = true;
                self.selected = None;
                if !self.entries.is_empty() {
                    let index = preferred
                        .and_then(|name| self.entries.iter().position(|d| d.name() == name))
                        .unwrap_or(0);
                    self.select(index);
                }
                true
            }
            Err(error) => {
                self.entries.clear();
                self.selected = None;
                self.available = false;
                self.notice = format!(
                    "Library unavailable: {}.\nClose and reopen to retry.",
                    error.summary()
                );
                warn!("Track design library: {error}");
                false
            }
        }
    }
}

pub(crate) fn open_shortcut(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::KeyD)
        && [KeyCode::ControlLeft, KeyCode::ControlRight]
            .into_iter()
            .any(|key| keys.pressed(key) || keys.just_pressed(key) || keys.just_released(key))
}

pub(crate) fn command_enabled(
    park: &Park,
    library: &DesignLibrary,
    command: DesignCommand,
) -> bool {
    use DesignCommand::*;
    if matches!(command, Open | Close | Cancel) {
        return true;
    }
    if library.confirmation.is_some() {
        return command == Confirm;
    }
    if !library.available {
        return false;
    }
    let selected = library.selected_design().is_some();
    match command {
        SaveNew => {
            !park.track_draft().is_empty()
                && !library.name.trim().is_empty()
                && library.entries.len() < crate::design_store::MAX_TRACK_DESIGNS
        }
        UpdateSelected => {
            selected && !park.track_draft().is_empty() && !library.name.trim().is_empty()
        }
        DeleteSelected | LoadDraft | Turn => selected,
        Browse(_) => library.entries.len() > 1,
        Select(index) => index < library.entries.len(),
        MoveX(delta) => {
            selected
                && if delta < 0 {
                    library.anchor.x > 0
                } else {
                    library.anchor.x < park.width() - 1
                }
        }
        MoveY(delta) => {
            selected
                && if delta < 0 {
                    library.anchor.y > 0
                } else {
                    library.anchor.y < park.height() - 1
                }
        }
        Height(delta) => {
            selected
                && if delta < 0 {
                    library.height > 0
                } else {
                    library.height < park_sim::MAX_TERRAIN_HEIGHT - 2
                }
        }
        Confirm => false,
        _ => true,
    }
}

pub(crate) fn request_open(view: &mut ViewState) {
    if !view.track_editor.active || view.edits_blocked() {
        return;
    }
    view.set_modal(false, false, false, false);
    view.designs_open = true;
    view.modal_transition_pending = true;
    view.track_editor.pending.clear();
    view.design_commands.push(DesignCommand::Open);
}

pub(crate) fn queue(view: &mut ViewState, command: DesignCommand) {
    if !view.designs_open || view.capture_active || view.diagnostic_case.is_some() {
        return;
    }
    view.ui_pointer_consumed = true;
    if !view.design_commands.contains(&command) {
        view.design_commands.push(command);
    }
}

fn close(view: &mut ViewState, library: &mut DesignLibrary) {
    view.set_modal(false, false, false, false);
    view.modal_transition_pending = true;
    library.name_focused = false;
    library.cancel_confirmation();
}

pub(crate) fn keyboard_text(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<ViewState>,
    mut library: ResMut<DesignLibrary>,
) {
    for event in events.read() {
        if !view.designs_open
            || view.capture_active
            || view.diagnostic_case.is_some()
            || view.modal_input_blocked
            || view.modal_transition_pending
            || !library.name_focused
            || library.confirmation.is_some()
            || event.state != ButtonState::Pressed
        {
            continue;
        }
        let held = |key| keys.pressed(key) || keys.just_pressed(key) || keys.just_released(key);
        let control = held(KeyCode::ControlLeft) || held(KeyCode::ControlRight);
        let alt = held(KeyCode::AltLeft) || held(KeyCode::AltRight);
        if control && !alt && event.key_code == KeyCode::KeyA {
            library.select_all = true;
            continue;
        }
        // Ctrl shortcuts do not enter text; AltGr can produce ordinary Unicode.
        if control && !alt {
            continue;
        }
        match event.logical_key {
            Key::Backspace => {
                if library.select_all {
                    library.name.clear();
                } else {
                    library.name.pop();
                }
                library.select_all = false;
            }
            Key::Enter | Key::Tab => library.name_focused = false,
            _ => {
                if let Some(text) = event.text.as_deref() {
                    let text: String = text.chars().filter(|c| !c.is_control()).collect();
                    if !text.is_empty() {
                        if library.select_all {
                            library.name.clear();
                        }
                        library.select_all = false;
                        let remaining = MAX_TRACK_DESIGN_NAME_CHARS
                            .saturating_sub(library.name.chars().count());
                        library.name.extend(text.chars().take(remaining));
                    }
                }
            }
        }
    }
}

pub(crate) fn apply_commands(
    session: Res<SaveSession>,
    mut sim: ResMut<Simulation>,
    mut view: ResMut<ViewState>,
    mut library: ResMut<DesignLibrary>,
) {
    let commands = std::mem::take(&mut view.design_commands);
    if commands.is_empty()
        || !view.designs_open
        || view.capture_active
        || view.diagnostic_case.is_some()
        || view
            .save_pending
            .is_some_and(|action| action != SaveAction::Save)
    {
        return;
    }
    if commands.contains(&DesignCommand::Close) {
        close(&mut view, &mut library);
        return;
    }
    if commands.contains(&DesignCommand::Cancel) {
        if library.confirmation.is_some() {
            library.cancel_confirmation();
            library.notice = "Cancelled. Saved designs and the draft are unchanged.".into();
        } else {
            close(&mut view, &mut library);
        }
        return;
    }
    let store = session
        .store
        .as_ref()
        .map(|store| DesignStore::new(store.directory().to_path_buf()));
    let Some(store) = store else {
        library.available = false;
        library.notice = "Design folder unavailable. Check launch settings.".into();
        return;
    };
    let confirmation_was_visible = library.confirmation.is_some();
    for command in commands {
        if !view.designs_open {
            break;
        }
        // A hidden Confirm button cannot approve a prompt opened this frame.
        if command == DesignCommand::Confirm && !confirmation_was_visible {
            continue;
        }
        perform_command(&store, &mut sim.0, &mut view, &mut library, command);
    }
}

/// Shared by the modal controls and fresh-process proofs.
pub(crate) fn perform_command(
    store: &DesignStore,
    park: &mut Park,
    view: &mut ViewState,
    library: &mut DesignLibrary,
    command: DesignCommand,
) {
    use DesignCommand::*;
    if !command_enabled(park, library, command) {
        return;
    }
    match command {
        Open => {
            info!("Track design library: {}", store.directory().display());
            library.cancel_confirmation();
            let preferred = library.selected_design().map(|d| d.name().to_owned());
            if library.refresh(store, preferred.as_deref()) {
                library.notice = "Save the current draft, or load a saved design to edit.\nLoading does not buy or replace live track.".into();
                if library.entries.is_empty() {
                    library.name = "My track".into();
                    library.name_focused = true;
                    library.select_all = true;
                }
            }
        }
        Close => close(view, library),
        Cancel => {
            library.cancel_confirmation();
            library.notice = "Cancelled. Saved designs are unchanged.".into();
        }
        Select(index) => library.select(index),
        Browse(delta) => {
            if !library.entries.is_empty() {
                let index = (library.selected.unwrap_or(0) as i64 + i64::from(delta))
                    .rem_euclid(library.entries.len() as i64) as usize;
                perform_command(store, park, view, library, Select(index));
            }
        }
        NameFocus => {
            library.name_focused = true;
            library.select_all = true;
            library.notice = "Type a name (up to 32 characters).\nEnter finishes typing; Ctrl+A selects the whole name.".into();
        }
        MoveX(delta) => {
            library.anchor.x = (i64::from(library.anchor.x) + i64::from(delta))
                .clamp(0, i64::from(park.width() - 1)) as u32
        }
        MoveY(delta) => {
            library.anchor.y = (i64::from(library.anchor.y) + i64::from(delta))
                .clamp(0, i64::from(park.height() - 1)) as u32
        }
        Height(delta) => {
            library.height = (i16::from(library.height) + i16::from(delta))
                .clamp(0, i16::from(park_sim::MAX_TERRAIN_HEIGHT - 2))
                as u8
        }
        Turn => library.rotation = (library.rotation + 1) % 4,
        SaveNew => {
            let result = TrackDesign::from_draft(&library.name, park.track_draft());
            match result {
                Ok(design) => match store.save_new(&design) {
                    Ok(()) => {
                        if library.refresh(store, Some(design.name())) {
                            library.notice = format!(
                                "Saved \"{}\".\nThe current draft and live park are unchanged.",
                                design.name()
                            );
                        }
                    }
                    Err(error) => {
                        warn!("Design save at {}: {error}", store.directory().display());
                        library.notice = format!("Design not saved: {}.", error.summary());
                    }
                },
                Err(error) => library.notice = format!("Design not saved: {error}"),
            }
        }
        UpdateSelected | DeleteSelected => {
            let Some(target) = library.selected_design().cloned() else {
                return;
            };
            let replacement = if command == UpdateSelected {
                match TrackDesign::from_draft(&library.name, park.track_draft()) {
                    Ok(design) => Some(design),
                    Err(error) => {
                        library.notice = format!("Design not updated: {error}");
                        return;
                    }
                }
            } else {
                None
            };
            library.notice = if command == UpdateSelected {
                format!(
                    "Replace \"{}\" with the current draft?\nThis changes the saved design; it cannot be undone.",
                    target.name()
                )
            } else {
                format!(
                    "Delete \"{}\" from your library?\nThis cannot be undone. Your draft and park stay unchanged.",
                    target.name()
                )
            };
            library.confirmed_target = Some(target);
            library.confirmed_replacement = replacement;
            library.confirmation = Some(command);
            library.name_focused = false;
        }
        Confirm => {
            let Some(target) = library.confirmed_target.take() else {
                return;
            };
            let replacement = library.confirmed_replacement.take();
            let result = match (library.confirmation.take(), replacement.as_ref()) {
                (Some(UpdateSelected), Some(replacement)) => {
                    store.replace_exact(&target, replacement)
                }
                (Some(DeleteSelected), _) => store.delete_exact(&target),
                _ => return,
            };
            match result {
                Ok(()) => {
                    if library.refresh(store, replacement.as_ref().map(|d| d.name())) {
                        library.notice = if replacement.is_some() {
                            "Saved design updated."
                        } else {
                            "Saved design deleted. Draft and park unchanged."
                        }
                        .into();
                    }
                }
                Err(error) => {
                    warn!("Design update at {}: {error}", store.directory().display());
                    library.notice = format!(
                        "Library unchanged: {}.\nClose and reopen to refresh.",
                        error.summary()
                    );
                }
            }
        }
        LoadDraft => {
            let Some(name) = library.selected_design().map(|d| d.name().to_owned()) else {
                return;
            };
            let design = match store.load(&name) {
                Ok(design) => design,
                Err(error) => {
                    warn!("Design load at {}: {error}", store.directory().display());
                    library.notice = format!("Design not loaded: {}.", error.summary());
                    return;
                }
            };
            let specs = match design.transformed(library.anchor, library.height, library.rotation) {
                Ok(specs) => specs,
                Err(error) => {
                    library.notice = format!(
                        "Design does not fit: {error}\nAdjust position, height or rotation."
                    );
                    return;
                }
            };
            match track_tools::load_design_draft(park, view, &specs) {
                Ok(changed) => {
                    close(view, library);
                    view.track_editor.active = true;
                    view.track_editor.notice = if changed {
                        format!(
                            "{} design pieces loaded.\nUndo restores the old draft.",
                            specs.len()
                        )
                    } else {
                        "Draft already matches design.\nExisting history is unchanged.".into()
                    };
                }
                Err(_) => {
                    library.notice =
                        "Design does not fit this park.\nAdjust position, height or rotation."
                            .into()
                }
            }
        }
    }
}
