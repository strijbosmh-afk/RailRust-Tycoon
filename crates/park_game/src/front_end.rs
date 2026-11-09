//! Startup and park management. The full-screen layer owns input while open.
use std::time::{Duration, Instant};

use bevy::{input::keyboard::KeyboardInput, prelude::*};
use park_sim::{Park, ScenarioKind, Speed};

use crate::{
    MAP_SIZE, Simulation, ViewState,
    campaign::Campaign,
    new_game_park, new_game_view,
    park_library::{Entry, ParkLibrary},
    persistence::SaveSession,
    sound::SoundSettings,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Screen {
    Intro,
    Title,
    New,
    Library,
    Rename,
    SaveAs,
    Delete,
    Settings,
    Credits,
    Pause,
    ConfirmTitle,
    ConfirmQuit,
    ConfirmLoad,
    Loading,
    Error,
    Park,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LibraryPurpose {
    TitleLoad,
    ParkLoad,
}
#[derive(Component)]
pub(super) struct Overlay;
#[derive(Component, Clone, Copy)]
pub(super) enum Action {
    Skip,
    Continue,
    New,
    Campaign,
    Library,
    Settings,
    Credits,
    Quit,
    Back,
    Start,
    Mode(u8),
    Select(u64),
    Load,
    LoadBackup,
    Rename,
    Delete,
    ConfirmDelete,
    Save,
    SaveAs,
    Import,
    ImportBackup,
    ReturnTitle,
    ConfirmTitle,
    ConfirmQuit,
    ConfirmLoad,
    Resume,
    ToggleSound,
    Volume(i8),
    Page(i8),
}

#[derive(Resource)]
pub(super) struct FrontEnd {
    enabled: bool,
    screen: Screen,
    previous: Screen,
    intro_started: Instant,
    library: Option<ParkLibrary>,
    entries: Vec<Entry>,
    selected: Option<u64>,
    active: Option<u64>,
    snapshot: Option<Vec<u8>>,
    mode: u8,
    purpose: LibraryPurpose,
    page: usize,
    draft: String,
    notice: String,
    dirty_ui: bool,
    paused_speed: Speed,
    pending_load: Option<(u64, bool)>,
    load_return: Screen,
    intro_frame: u8,
    focus: usize,
}

impl FrontEnd {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn current_name(&self) -> &str {
        if self.enabled {
            &self.draft
        } else {
            "Meadow Lake"
        }
    }
    pub fn capture_scene(&mut self, name: &str) -> Result<(), String> {
        self.screen = match name {
            "intro" => Screen::Intro,
            "title" => Screen::Title,
            "new-park" => Screen::New,
            "park-library" => Screen::Library,
            "pause-menu" => Screen::Pause,
            "settings" => Screen::Settings,
            "credits" => Screen::Credits,
            _ => return Err("unknown --capture-front scene".into()),
        };
        self.enabled = true;
        self.dirty_ui = true;
        Ok(())
    }
    pub fn new(enabled: bool, session: &SaveSession) -> Self {
        let library = session
            .store
            .as_ref()
            .map(|store| ParkLibrary::new(store.directory()));
        let mut result = Self {
            enabled,
            screen: if enabled { Screen::Intro } else { Screen::Park },
            previous: Screen::Title,
            intro_started: Instant::now(),
            library,
            entries: Vec::new(),
            selected: None,
            active: None,
            snapshot: None,
            mode: 0,
            purpose: LibraryPurpose::TitleLoad,
            page: 0,
            draft: "My Park".into(),
            notice: String::new(),
            dirty_ui: true,
            paused_speed: Speed::Normal,
            pending_load: None,
            load_return: Screen::Title,
            intro_frame: 0,
            focus: 0,
        };
        result.reload();
        result
    }
    fn reload(&mut self) {
        self.entries = match &self.library {
            Some(library) => match library.entries() {
                Ok(entries) => entries,
                Err(error) => {
                    self.notice = format!("Park library: {error}");
                    Vec::new()
                }
            },
            None => {
                self.notice = "Save folder unavailable. Check launch settings.".into();
                Vec::new()
            }
        };
        if !self.entries.iter().any(|e| Some(e.id) == self.selected) {
            self.selected = self.entries.first().map(|e| e.id);
        }
        self.dirty_ui = true;
    }
    fn continue_id(&self) -> Option<u64> {
        let library = self.library.as_ref()?;
        self.entries
            .iter()
            .find(|entry| library.load(entry.id, false).is_ok())
            .map(|entry| entry.id)
    }
    fn show(&mut self, screen: Screen, view: &mut ViewState) {
        self.screen = screen;
        self.focus = 0;
        view.front_modal = screen != Screen::Park;
        if screen == Screen::Park {
            view.front_pointer_release_guard = true;
        }
        view.ui_pointer_consumed = true;
        view.dragging = false;
        view.hovered = None;
        self.dirty_ui = true;
    }
    fn open_pause(&mut self, view: &mut ViewState) {
        if self.screen != Screen::Park {
            return;
        }
        self.paused_speed = view.speed;
        view.speed = Speed::Paused;
        self.show(Screen::Pause, view);
    }
    fn resume(&mut self, view: &mut ViewState) {
        self.show(Screen::Park, view);
        view.speed = self.paused_speed;
    }
    fn unsaved(&self, park: &Park) -> bool {
        park.to_save_bytes().ok().as_ref() != self.snapshot.as_ref()
    }
    fn saved(&mut self, park: &Park) {
        self.snapshot = park.to_save_bytes().ok();
    }
    fn save(&mut self, park: &Park, view: &mut ViewState) {
        let Some(library) = &self.library else {
            self.notice = "Save folder unavailable.".into();
            self.dirty_ui = true;
            return;
        };
        let result = match self.active {
            Some(id) => library.save(id, park),
            None => library.create(&self.draft, park),
        };
        match result {
            Ok(entry) => {
                self.active = Some(entry.id);
                self.selected = Some(entry.id);
                self.draft = entry.name.clone();
                self.saved(park);
                self.notice = format!("Saved {}.", entry.name);
                view.save_notice = self.notice.clone();
                self.reload();
            }
            Err(error) => {
                self.notice = format!("Save failed: {error}");
                self.dirty_ui = true;
            }
        }
    }
    fn load(&mut self, id: u64, backup: bool, sim: &mut Simulation, view: &mut ViewState) {
        let Some(library) = &self.library else {
            self.notice = "Save folder unavailable.".into();
            return;
        };
        match library.load(id, backup) {
            Ok(park) if park.width() == MAP_SIZE && park.height() == MAP_SIZE => {
                let generation = view.park_generation.wrapping_add(1);
                let terrain_profile = if park.max_terrain_height() == 0 {
                    park_sim::TerrainProfile::Flat
                } else {
                    park_sim::TerrainProfile::Terraced
                };
                sim.0 = park;
                *view = ViewState {
                    speed: Speed::Paused,
                    resume_speed: Speed::Normal,
                    terrain_profile,
                    scene_dirty: true,
                    ui_pointer_consumed: true,
                    park_generation: generation,
                    ..new_game_view()
                };
                self.active = Some(id);
                if let Some(entry) = self.entries.iter().find(|entry| entry.id == id) {
                    self.draft = entry.name.clone();
                }
                if backup {
                    self.snapshot = None;
                } else {
                    self.saved(&sim.0);
                }
                self.paused_speed = Speed::Normal;
                self.notice = if backup {
                    "Backup loaded. Save to keep it.".into()
                } else {
                    "Park loaded.".into()
                };
                self.show(Screen::Park, view);
            }
            Ok(_) => {
                self.notice = "Load rejected: requires a 32 x 32 park.".into();
                self.show(Screen::Error, view);
            }
            Err(error) => {
                self.notice = format!("Load failed: {error}");
                self.show(Screen::Error, view);
            }
        }
    }
    fn begin_load(&mut self, id: u64, backup: bool, view: &mut ViewState) {
        if self.screen != Screen::ConfirmLoad {
            self.load_return = self.screen;
        }
        if self.screen == Screen::Park {
            self.paused_speed = view.speed;
            view.speed = Speed::Paused;
        }
        self.pending_load = Some((id, backup));
        self.show(Screen::Loading, view);
    }
    fn new_park(&mut self, sim: &mut Simulation, view: &mut ViewState, campaign: &Campaign) {
        let scenario = match self.mode {
            1 => Some(ScenarioKind::MeadowChallenge),
            2 => Some(ScenarioKind::BoardwalkSprint),
            _ => None,
        };
        if scenario.is_some_and(|kind| !campaign.unlocked(kind)) {
            self.notice = "Complete Meadow Challenge to unlock Boardwalk Sprint.".into();
            self.dirty_ui = true;
            return;
        }
        let name = self.draft.trim();
        if name.is_empty() || name.chars().count() > 40 || name.chars().any(char::is_control) {
            self.notice = "Use a park name of 1 to 40 characters.".into();
            self.dirty_ui = true;
            return;
        }
        let mut park = new_game_park();
        if let Some(kind) = scenario
            && let Err(error) = park.start_scenario(kind)
        {
            self.notice = format!("Could not start scenario: {error}");
            self.dirty_ui = true;
            return;
        }
        sim.0 = park;
        let generation = view.park_generation.wrapping_add(1);
        *view = ViewState {
            park_generation: generation,
            scene_dirty: true,
            ui_pointer_consumed: true,
            ..new_game_view()
        };
        self.active = None;
        self.snapshot = None;
        self.paused_speed = Speed::Normal;
        self.notice = "New park started. Save from the menu or press F5.".into();
        self.show(Screen::Park, view);
    }
}

pub(super) fn setup(
    mut commands: Commands,
    front: Res<FrontEnd>,
    campaign: Res<Campaign>,
    sim: Res<Simulation>,
) {
    if front.enabled {
        spawn_overlay(&mut commands, &front, None, &campaign, &sim.0);
    }
}

type ChangedButtons<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static Action), (Changed<Interaction>, With<Button>)>;
type InputResources<'w> = (
    Res<'w, Campaign>,
    Res<'w, SaveSession>,
    MessageWriter<'w, AppExit>,
    Option<ResMut<'w, SoundSettings>>,
);

pub(super) fn input(
    mut front: ResMut<FrontEnd>,
    mut view: ResMut<ViewState>,
    mut sim: ResMut<Simulation>,
    resources: InputResources<'_>,
    keys: Res<ButtonInput<KeyCode>>,
    mut text: MessageReader<KeyboardInput>,
    interactions: ChangedButtons<'_, '_>,
) {
    let (campaign, session, mut exit, mut sound) = resources;
    if !front.enabled {
        return;
    }
    if front.screen == Screen::Loading {
        if let Some((id, backup)) = front.pending_load.take() {
            front.load(id, backup, &mut sim, &mut view);
        }
        return;
    }
    if front.screen == Screen::Intro && front.intro_started.elapsed() >= Duration::from_secs(3) {
        front.show(Screen::Title, &mut view);
    }
    if front.screen == Screen::Park {
        if view.front_request_menu
            || (keys.just_pressed(KeyCode::Escape)
                && !view.help
                && !view.guests_open
                && !view.services_open
                && !view.finance_open
                && !view.rides_open
                && !view.designs_open
                && !view.track_editor.active
                && view.selected.is_none()
                && view.path_tool.mode == crate::path_tools::PathTool::Inspect)
        {
            view.front_request_menu = false;
            front.open_pause(&mut view);
        }
        if view.front_modal {
            view.save_pending = None;
            return;
        }
        if let Some(action) = view.save_pending.take() {
            if view.help
                || view.guests_open
                || view.services_open
                || view.finance_open
                || view.rides_open
                || view.designs_open
                || view.modal_input_blocked
                || view.capture_active
            {
                return;
            }
            match action {
                crate::persistence::SaveAction::Save => front.save(&sim.0, &mut view),
                crate::persistence::SaveAction::Load | crate::persistence::SaveAction::Backup => {
                    if let Some(id) = front.active {
                        let backup = action == crate::persistence::SaveAction::Backup;
                        if front.unsaved(&sim.0) {
                            front.pending_load = Some((id, backup));
                            front.load_return = Screen::Park;
                            front.paused_speed = view.speed;
                            view.speed = Speed::Paused;
                            front.show(Screen::ConfirmLoad, &mut view);
                        } else {
                            front.begin_load(id, backup, &mut view);
                        }
                    } else {
                        front.notice = "Save this park first, or load one from the menu.".into();
                        view.save_notice = front.notice.clone();
                    }
                }
            }
        }
        return;
    }
    if matches!(front.screen, Screen::New | Screen::Rename | Screen::SaveAs) {
        use bevy::input::ButtonState;
        use bevy::input::keyboard::Key;
        for event in text.read() {
            if event.state != ButtonState::Pressed {
                continue;
            }
            match event.logical_key {
                Key::Backspace => {
                    front.draft.pop();
                    front.dirty_ui = true;
                }
                Key::Enter => {}
                _ => {
                    if let Some(value) = event.text.as_deref() {
                        for ch in value.chars().filter(|ch| !ch.is_control()) {
                            if front.draft.chars().count() < 40 {
                                front.draft.push(ch);
                                front.dirty_ui = true;
                            }
                        }
                    }
                }
            }
        }
    }
    let actions = keyboard_actions(&front);
    if !actions.is_empty() {
        if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::ArrowDown) {
            front.focus = (front.focus + 1) % actions.len();
            front.dirty_ui = true;
        }
        if keys.just_pressed(KeyCode::ArrowUp) {
            front.focus = (front.focus + actions.len() - 1) % actions.len();
            front.dirty_ui = true;
        }
    }
    let mut pressed = interactions
        .iter()
        .filter(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, a)| *a)
        .next();
    if pressed.is_none() && keys.just_pressed(KeyCode::Enter) {
        pressed = actions.get(front.focus).copied();
    }
    if front.screen == Screen::Intro
        && (keys.just_pressed(KeyCode::Space)
            || keys.just_pressed(KeyCode::Escape)
            || keys.just_pressed(KeyCode::Enter))
    {
        pressed = Some(Action::Skip);
    }
    if keys.just_pressed(KeyCode::Escape) && pressed.is_none() {
        pressed = Some(Action::Back);
    }
    let Some(action) = pressed else {
        return;
    };
    match action {
        Action::Skip => front.show(Screen::Title, &mut view),
        Action::Continue => {
            if let Some(id) = front.continue_id() {
                front.begin_load(id, false, &mut view);
            }
        }
        Action::New | Action::Campaign => {
            front.mode = if matches!(action, Action::Campaign) {
                1
            } else {
                0
            };
            front.draft = "My Park".into();
            front.show(Screen::New, &mut view);
        }
        Action::Library => {
            front.purpose = if front.screen == Screen::Pause {
                LibraryPurpose::ParkLoad
            } else {
                LibraryPurpose::TitleLoad
            };
            front.previous = front.screen;
            front.reload();
            front.show(Screen::Library, &mut view);
        }
        Action::Settings => {
            front.previous = front.screen;
            front.show(Screen::Settings, &mut view);
        }
        Action::Credits => front.show(Screen::Credits, &mut view),
        Action::Quit => {
            if front.screen == Screen::Pause && front.unsaved(&sim.0) {
                front.show(Screen::ConfirmQuit, &mut view);
            } else {
                exit.write(AppExit::Success);
            }
        }
        Action::Back => {
            let target = match front.screen {
                Screen::Pause => Screen::Park,
                Screen::Settings | Screen::Library => front.previous,
                Screen::Rename | Screen::Delete => Screen::Library,
                Screen::SaveAs => Screen::Pause,
                Screen::ConfirmTitle | Screen::ConfirmQuit => Screen::Pause,
                Screen::ConfirmLoad => front.load_return,
                Screen::Error => front.load_return,
                _ => Screen::Title,
            };
            if target == Screen::Park {
                front.resume(&mut view);
            } else {
                front.show(target, &mut view);
            }
        }
        Action::Start => front.new_park(&mut sim, &mut view, &campaign),
        Action::Mode(mode) => {
            front.mode = mode;
            front.dirty_ui = true;
        }
        Action::Select(id) => {
            front.selected = Some(id);
            front.dirty_ui = true;
        }
        Action::Load | Action::LoadBackup => {
            if let Some(id) = front.selected {
                let backup = matches!(action, Action::LoadBackup);
                if front.purpose == LibraryPurpose::ParkLoad && front.unsaved(&sim.0) {
                    front.pending_load = Some((id, backup));
                    front.load_return = Screen::Library;
                    front.show(Screen::ConfirmLoad, &mut view);
                } else {
                    front.begin_load(id, backup, &mut view);
                }
            }
        }
        Action::Rename => {
            if let Some(entry) = front.entries.iter().find(|e| Some(e.id) == front.selected) {
                front.draft = entry.name.clone();
                front.show(Screen::Rename, &mut view);
            }
        }
        Action::Delete => front.show(Screen::Delete, &mut view),
        Action::ConfirmDelete => {
            if let (Some(lib), Some(id)) = (&front.library, front.selected) {
                match lib.archive(id) {
                    Ok(()) => {
                        if front.active == Some(id) {
                            front.active = None;
                            front.snapshot = None;
                        }
                        front.notice = "Park moved to the archive.".into();
                        front.reload();
                        front.show(Screen::Library, &mut view);
                    }
                    Err(e) => {
                        front.notice = format!("Could not archive park: {e}");
                        front.dirty_ui = true;
                    }
                }
            }
        }
        Action::Save => {
            if front.screen == Screen::SaveAs {
                let old = front.active;
                front.active = None;
                front.save(&sim.0, &mut view);
                if front.active.is_none() {
                    front.active = old;
                } else {
                    front.show(Screen::Pause, &mut view);
                }
            } else if front.screen == Screen::Rename {
                if let (Some(lib), Some(id)) = (&front.library, front.selected) {
                    match lib.rename(id, &front.draft) {
                        Ok(_) => {
                            front.reload();
                            front.show(Screen::Library, &mut view);
                        }
                        Err(e) => {
                            front.notice = e;
                            front.dirty_ui = true;
                        }
                    }
                }
            } else {
                front.save(&sim.0, &mut view);
            }
        }
        Action::SaveAs => {
            front.draft = format!("{} Copy", front.draft).chars().take(40).collect();
            front.show(Screen::SaveAs, &mut view);
        }
        Action::Import | Action::ImportBackup => {
            if let (Some(lib), Some(store)) = (&front.library, &session.store) {
                match lib.import_legacy(store, matches!(action, Action::ImportBackup)) {
                    Ok(entry) => {
                        front.notice = format!("Imported {}. Original save preserved.", entry.name);
                        front.reload();
                    }
                    Err(e) => {
                        front.notice = format!("Import failed: {e}");
                        front.dirty_ui = true;
                    }
                }
            }
        }
        Action::ReturnTitle => {
            if front.unsaved(&sim.0) {
                front.show(Screen::ConfirmTitle, &mut view);
            } else {
                front.show(Screen::Title, &mut view);
            }
        }
        Action::ConfirmTitle => {
            front.active = None;
            front.snapshot = None;
            front.reload();
            front.show(Screen::Title, &mut view);
        }
        Action::ConfirmQuit => {
            exit.write(AppExit::Success);
        }
        Action::ConfirmLoad => {
            if let Some((id, backup)) = front.pending_load {
                front.begin_load(id, backup, &mut view);
            }
        }
        Action::Resume => front.resume(&mut view),
        Action::ToggleSound => {
            if let Some(s) = sound.as_mut() {
                s.value.muted = !s.value.muted;
                s.persist();
                front.notice = s.notice.clone();
                front.dirty_ui = true;
            }
        }
        Action::Volume(delta) => {
            if let Some(s) = sound.as_mut() {
                s.value.levels[0] =
                    (i16::from(s.value.levels[0]) + i16::from(delta)).clamp(0, 100) as u8;
                s.persist();
                front.notice = s.notice.clone();
                front.dirty_ui = true;
            }
        }
        Action::Page(delta) => {
            front.page = front
                .page
                .saturating_add_signed(delta as isize)
                .min(front.entries.len().saturating_sub(1) / 4);
            front.dirty_ui = true;
        }
    }
}

fn keyboard_actions(front: &FrontEnd) -> Vec<Action> {
    match front.screen {
        Screen::Intro => vec![Action::Skip],
        Screen::Title => {
            let mut v = Vec::new();
            if front.continue_id().is_some() {
                v.push(Action::Continue);
            }
            v.extend([
                Action::New,
                Action::Campaign,
                Action::Library,
                Action::Settings,
                Action::Credits,
                Action::Quit,
            ]);
            v
        }
        Screen::New => vec![
            Action::Mode(0),
            Action::Mode(1),
            Action::Mode(2),
            Action::Start,
            Action::Back,
        ],
        Screen::Library => {
            let mut v: Vec<_> = front
                .entries
                .iter()
                .skip(front.page * 4)
                .take(4)
                .map(|e| Action::Select(e.id))
                .collect();
            if front.entries.len() > 4 {
                v.extend([Action::Page(-1), Action::Page(1)]);
            }
            if front.selected.is_some() {
                v.extend([
                    Action::Load,
                    Action::LoadBackup,
                    Action::Rename,
                    Action::Delete,
                ]);
            }
            if front.purpose == LibraryPurpose::TitleLoad {
                v.extend([Action::Import, Action::ImportBackup]);
            }
            v.push(Action::Back);
            v
        }
        Screen::Rename | Screen::SaveAs => vec![Action::Save, Action::Back],
        Screen::Delete => vec![Action::ConfirmDelete, Action::Back],
        Screen::Settings => vec![
            Action::ToggleSound,
            Action::Volume(-10),
            Action::Volume(10),
            Action::Back,
        ],
        Screen::Credits | Screen::Error => vec![Action::Back],
        Screen::Pause => vec![
            Action::Resume,
            Action::Save,
            Action::SaveAs,
            Action::Library,
            Action::Settings,
            Action::ReturnTitle,
            Action::Quit,
        ],
        Screen::ConfirmTitle => vec![Action::Save, Action::ConfirmTitle, Action::Back],
        Screen::ConfirmQuit => vec![Action::Save, Action::ConfirmQuit, Action::Back],
        Screen::ConfirmLoad => vec![Action::Save, Action::ConfirmLoad, Action::Back],
        Screen::Loading | Screen::Park => Vec::new(),
    }
}

pub(super) fn refresh(
    mut commands: Commands,
    mut front: ResMut<FrontEnd>,
    roots: Query<Entity, With<Overlay>>,
    sound: Option<Res<SoundSettings>>,
    campaign: Res<Campaign>,
    sim: Res<Simulation>,
) {
    if front.screen == Screen::Intro {
        let frame = (front.intro_started.elapsed().as_millis() / 250) as u8;
        if frame != front.intro_frame {
            front.intro_frame = frame;
            front.dirty_ui = true;
        }
    }
    if !front.enabled || !front.dirty_ui {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    if front.screen != Screen::Park {
        spawn_overlay(
            &mut commands,
            &front,
            sound.as_deref().map(|s| s.value),
            &campaign,
            &sim.0,
        );
    }
    // Campaign locks and goals are shown in New Park; changes can occur only in park.
    front.dirty_ui = false;
}

fn spawn_overlay(
    commands: &mut Commands,
    front: &FrontEnd,
    sound: Option<crate::sound::Settings>,
    campaign: &Campaign,
    park: &Park,
) {
    let gold = Color::srgb_u8(255, 207, 71);
    let cream = Color::srgb_u8(255, 249, 219);
    let ink = Color::srgb_u8(20, 61, 45);
    let panel = Color::srgb_u8(34, 104, 66);
    commands.spawn((Overlay, GlobalZIndex(30), Node { position_type: PositionType::Absolute,
        width: percent(100), height: percent(100), justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.03,0.11,0.08,0.94))))
        .with_children(|root| {
            root.spawn((Node { width: percent(70), max_width: px(690), max_height: percent(94),
                padding: UiRect::all(px(20)), flex_direction: FlexDirection::Column,
                row_gap: px(5), align_items: AlignItems::Stretch, overflow: Overflow::clip(),
                ..default() }, BackgroundColor(panel), BorderRadius::all(px(15))))
                .with_children(|p| {
                    title(p, "PARKBOUND", 39.0, gold);
                    let subtitle = match front.screen { Screen::Intro => "A world of rides, stories, and happy guests",
                        Screen::Title => "Your park starts here", Screen::New => "NEW PARK",
                        Screen::Library => "PARK LIBRARY", Screen::Rename => "RENAME PARK", Screen::SaveAs => "SAVE AS NEW PARK",
                        Screen::Delete => "ARCHIVE PARK?", Screen::Settings => "SETTINGS",
                        Screen::Credits => "CREDITS", Screen::Pause => "PARK MENU",
                        Screen::ConfirmTitle | Screen::ConfirmQuit | Screen::ConfirmLoad => "UNSAVED CHANGES",
                        Screen::Loading => "LOADING PARK", Screen::Error => "PARK COULD NOT LOAD",
                        Screen::Park => "" };
                    title(p, subtitle, 19.0, cream);
                    match front.screen {
                        Screen::Intro => { title(p, &format!("{} Build a place worth remembering {}", "✦".repeat((front.intro_frame % 4 + 1) as usize), "✦".repeat((front.intro_frame % 4 + 1) as usize)), 24.0, cream); button(p, "Skip intro", Action::Skip); },
                        Screen::Title => { if front.continue_id().is_some() { button(p, "Continue latest park", Action::Continue); }
                            else { title(p, "Continue unavailable — no healthy saved park. Use Load Park for a backup.", 14.0, cream); }
                            button(p, "New Park", Action::New); button(p, "Campaign", Action::Campaign);
                            button(p, "Load Park", Action::Library); button(p, "Settings", Action::Settings);
                            button(p, "Credits", Action::Credits); button(p, "Quit", Action::Quit); },
                        Screen::New => { title(p, &format!("Name: {}_", front.draft), 18.0, cream);
                            title(p, "Type to name your park. Choose how to play:", 14.0, cream);
                            button(p, if front.mode == 0 { "● Sandbox — open building" } else { "○ Sandbox — open building" }, Action::Mode(0));
                            button(p, if front.mode == 1 { "● Meadow Challenge — 20 arrivals, 5 rides, $750" } else { "○ Meadow Challenge — 20 arrivals, 5 rides, $750" }, Action::Mode(1));
                            button(p, if front.mode == 2 { "● Boardwalk Sprint — unlock through Meadow" } else { "○ Boardwalk Sprint — unlock through Meadow" }, Action::Mode(2));
                            let description = match front.mode { 1 => "Meadow: flat 32×32 map, $1,000 start, free entry; 20 arrivals, 5 rides, $750 cash. No deadline. Award unlocks Boardwalk.",
                                2 if campaign.unlocked(ScenarioKind::BoardwalkSprint) => "Boardwalk: flat 32×32 map, $1,000 start, free rides; 24 arrivals, 12 rides, $900 cash before 18,000 ticks. Award marks completion.",
                                2 => "Boardwalk locked: complete Meadow Challenge first.",
                                _ => "Sandbox: flat 32×32 park, starter rides and paths, $1,000 starting cash; no goals or deadline." };
                            title(p, description, 13.0, cream);
                            button(p, "Create Park", Action::Start); button(p, "Back", Action::Back); },
                        Screen::Library => { for entry in front.entries.iter().skip(front.page*4).take(4) {
                                button(p, &format!("{} {}", if front.selected == Some(entry.id) { "●" } else { "○" }, entry.name), Action::Select(entry.id));
                            }
                            if front.entries.is_empty() { title(p, "No parks saved yet.", 15.0, cream); }
                            if front.entries.len() > 4 { button(p, "Previous page", Action::Page(-1)); button(p, "Next page", Action::Page(1)); }
                            if front.selected.is_some() { button(p, "Load selected park", Action::Load); button(p, "Load selected backup", Action::LoadBackup); button(p, "Rename", Action::Rename); button(p, "Archive", Action::Delete); }
                            if front.purpose == LibraryPurpose::TitleLoad { button(p, "Import old single-slot save", Action::Import); button(p, "Import old single-slot backup", Action::ImportBackup); }
                            button(p, "Back", Action::Back); },
                        Screen::Rename => { title(p, &format!("Name: {}_", front.draft), 17.0, cream); button(p, "Save name", Action::Save); button(p, "Cancel", Action::Back); },
                        Screen::SaveAs => { title(p, &format!("Name: {}_", front.draft), 17.0, cream); button(p, "Save new park", Action::Save); button(p, "Cancel", Action::Back); },
                        Screen::Delete => { title(p, "This moves the park to the local archive. Its files remain available for manual recovery.", 14.0, cream); button(p, "Move to archive", Action::ConfirmDelete); button(p, "Keep park", Action::Back); },
                        Screen::Settings => { let sound = sound.unwrap_or_default(); button(p, if sound.muted { "Sound: Off" } else { "Sound: On" }, Action::ToggleSound);
                            title(p, &format!("Master volume: {}%", sound.levels[0]), 16.0, cream);
                            button(p, "Volume −", Action::Volume(-10)); button(p, "Volume +", Action::Volume(10));
                            button(p, "Back", Action::Back); },
                        Screen::Credits => { title(p, "An original park simulation built in Rust.", 17.0, cream);
                            title(p, "Design, code, art and sound: Parkbound project.", 15.0, cream); button(p, "Back", Action::Back); },
                        Screen::Pause => { title(p, if front.unsaved(park) { "Paused • Unsaved changes" } else { "Paused • All changes saved" }, 14.0, cream);
                            button(p, "Resume Park", Action::Resume); button(p, "Save", Action::Save);
                            button(p, "Save As New Park", Action::SaveAs); button(p, "Load Park", Action::Library);
                            button(p, "Settings", Action::Settings); button(p, "Return to Title", Action::ReturnTitle);
                            button(p, "Quit", Action::Quit); },
                        Screen::ConfirmTitle | Screen::ConfirmQuit | Screen::ConfirmLoad => { title(p, "Your park has changes since the last save.", 16.0, cream);
                            button(p, "Save first", Action::Save);
                            button(p, "Discard changes and continue", if front.screen == Screen::ConfirmTitle { Action::ConfirmTitle } else if front.screen == Screen::ConfirmQuit { Action::ConfirmQuit } else { Action::ConfirmLoad });
                            button(p, "Cancel", Action::Back); },
                        Screen::Loading => title(p, "Checking park files…", 17.0, cream),
                        Screen::Error => button(p, "Back", Action::Back),
                        Screen::Park => {},
                    }
                    if !front.notice.is_empty() { title(p, &front.notice, 13.0, gold); }
                    if let Some(action) = keyboard_actions(front).get(front.focus) {
                        title(p, &format!("↑/↓ or Tab • Enter selects {} • Esc goes back", action_name(*action)), 12.0, cream);
                    }
                });
        });
    let _ = ink;
}

fn title(parent: &mut ChildSpawnerCommands, value: &str, size: f32, color: Color) {
    parent.spawn((
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    ));
}
fn button(parent: &mut ChildSpawnerCommands, value: &str, action: Action) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: percent(100),
                height: px(30),
                padding: UiRect::horizontal(px(11)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(244, 209, 91)),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| title(p, value, 16.0, Color::srgb_u8(27, 59, 44)));
}
fn action_name(action: Action) -> &'static str {
    match action {
        Action::Continue => "Continue",
        Action::New => "New Park",
        Action::Campaign => "Campaign",
        Action::Library => "Load Park",
        Action::Settings => "Settings",
        Action::Credits => "Credits",
        Action::Quit => "Quit",
        Action::Back => "Back",
        Action::Start => "Create Park",
        Action::Mode(0) => "Sandbox",
        Action::Mode(1) => "Meadow Challenge",
        Action::Mode(_) => "Boardwalk Sprint",
        Action::Load => "Load",
        Action::LoadBackup => "Load Backup",
        Action::Rename => "Rename",
        Action::Delete => "Archive",
        Action::Save => "Save",
        Action::SaveAs => "Save As",
        Action::Import => "Import",
        Action::ImportBackup => "Import Backup",
        Action::ReturnTitle => "Return to Title",
        Action::Resume => "Resume",
        Action::ToggleSound => "Sound",
        Action::Volume(_) => "Volume",
        Action::Page(_) => "Page",
        Action::Select(_) => "Park",
        Action::ConfirmDelete
        | Action::ConfirmTitle
        | Action::ConfirmQuit
        | Action::ConfirmLoad => "Confirm",
        Action::Skip => "Skip",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::OsString, fs};

    #[test]
    fn new_save_and_load_clear_unsaved_state_and_pause() {
        let root = std::env::temp_dir().join(format!(
            "parkbound-front-test-{}-{}",
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
        let mut front = FrontEnd::new(true, &session);
        let mut sim = Simulation(new_game_park());
        let mut view = new_game_view();
        front.draft = "My first park".into();
        front.new_park(&mut sim, &mut view, &Campaign::preview());
        assert_eq!(front.screen, Screen::Park);
        assert!(front.unsaved(&sim.0));
        front.save(&sim.0, &mut view);
        assert!(!front.unsaved(&sim.0));
        let id = front.active.unwrap();
        let saved = sim.0.clone();
        sim.0.advance(Speed::Normal);
        assert!(front.unsaved(&sim.0));
        front.load(id, false, &mut sim, &mut view);
        assert_eq!(sim.0, saved);
        assert_eq!(view.speed, Speed::Paused);
        assert!(!front.unsaved(&sim.0));
        assert!(!view.front_modal);
        sim.0.advance(Speed::Normal);
        front.save(&sim.0, &mut view);
        front.load(id, true, &mut sim, &mut view);
        assert_eq!(sim.0, saved);
        assert!(front.unsaved(&sim.0));
        let safe = root.parent() == Some(std::env::temp_dir().as_path())
            && root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("parkbound-front-test-");
        if safe {
            fs::remove_dir_all(root).unwrap();
        }
    }
}
