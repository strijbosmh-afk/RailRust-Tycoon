//! Production input ordering with isolated design files; no native window backend.
use super::*;
use crate::{
    design_library::{DesignCommand, DesignLibrary},
    design_store::DesignStore,
    track_tools::{TrackCommand, TrackEditor},
};
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};
use park_sim::{TrackBuildKind, TrackDesign, TrackDirection, TrackPieceSpec};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "Parkbound design input é {} {timestamp} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn session(&self) -> SaveSession {
        SaveSession::from_arguments([
            std::ffi::OsString::from("--save-dir"),
            self.0.clone().into_os_string(),
        ])
    }
    fn store(&self) -> DesignStore {
        DesignStore::new(self.0.clone())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        if let (Ok(path), Ok(root)) = (
            std::fs::canonicalize(&self.0),
            std::fs::canonicalize(std::env::temp_dir()),
        ) && path.parent() == Some(root.as_path())
            && path.file_name().is_some_and(|name| {
                name.to_string_lossy()
                    .starts_with("Parkbound design input ")
            })
        {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

fn specs(x: u32) -> Vec<TrackPieceSpec> {
    vec![
        TrackBuildKind::Station.spec(TilePos { x, y: 15 }, 8, TrackDirection::East),
        TrackBuildKind::Straight.spec(TilePos { x: x + 1, y: 15 }, 8, TrackDirection::East),
    ]
}

fn fixture() -> (Park, ViewState) {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    let mut view = ViewState {
        speed: Speed::Paused,
        track_editor: TrackEditor {
            active: true,
            ..default()
        },
        ..default()
    };
    track_tools::load_design_draft(&mut park, &mut view, &specs(15)).unwrap();
    (park, view)
}

fn app(scratch: &Scratch, park: Park, view: ViewState) -> (App, Entity) {
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(view)
        .insert_resource(scratch.session())
        .insert_resource(Time::<()>::default())
        .init_resource::<DesignLibrary>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_message::<KeyboardInput>()
        .add_message::<MouseWheel>()
        .add_systems(
            Update,
            (
                ui::button_actions,
                keyboard_input,
                design_library::keyboard_text,
                design_library::apply_commands,
                apply_terrain_profile,
                apply_path_actions,
                apply_train_actions,
                apply_price_actions,
                apply_service_actions,
                track_tools::apply_commands,
                pointer_input,
                persistence::apply_save_action,
            )
                .chain(),
        );
    let window = app
        .world_mut()
        .spawn((
            Window {
                resolution: WindowResolution::new(1280, 800),
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    (app, window)
}

fn frame(app: &mut App) {
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
}

fn settle(app: &mut App) {
    frame(app);
    frame(app);
}

fn actions(app: &mut App, actions: impl IntoIterator<Item = ui::Action>) {
    let entities: Vec<_> = actions
        .into_iter()
        .map(|action| {
            app.world_mut()
                .spawn((Button, Interaction::Pressed, action))
                .id()
        })
        .collect();
    frame(app);
    for entity in entities {
        app.world_mut().despawn(entity);
    }
}

fn command(app: &mut App, command: DesignCommand) {
    actions(app, [ui::Action::Design(command)]);
}

fn open(app: &mut App) {
    command(app, DesignCommand::Open);
    settle(app);
    assert!(app.world().resource::<ViewState>().designs_open);
}

fn text(app: &mut App, window: Entity, code: KeyCode, logical: Key, text: Option<&str>) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(code);
    app.world_mut().write_message(KeyboardInput {
        key_code: code,
        logical_key: logical,
        state: ButtonState::Pressed,
        text: text.map(Into::into),
        repeat: false,
        window,
    });
}

fn type_text(app: &mut App, window: Entity, value: &str) {
    text(
        app,
        window,
        KeyCode::KeyA,
        Key::Character(value.into()),
        Some(value),
    );
    frame(app);
}

fn history(app: &App) -> (usize, usize, u64) {
    let editor = &app.world().resource::<ViewState>().track_editor;
    (editor.undo_len(), editor.redo_len(), editor.revision)
}

fn select(app: &mut App, name: &str) {
    let index = app
        .world()
        .resource::<DesignLibrary>()
        .entries
        .iter()
        .position(|design| design.name() == name)
        .unwrap();
    command(app, DesignCommand::Select(index));
}

#[test]
fn control_d_opens_library_and_typing_panel_letters_never_triggers_park_shortcuts() {
    for control in [KeyCode::ControlLeft, KeyCode::ControlRight] {
        let scratch = Scratch::new();
        let (park, view) = fixture();
        let (mut app, window) = app(&scratch, park.clone(), view);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(control);
            keys.release(control);
        }
        text(
            &mut app,
            window,
            KeyCode::KeyD,
            Key::Character("d".into()),
            Some("d"),
        );
        frame(&mut app);
        settle(&mut app);
        assert!(app.world().resource::<ViewState>().designs_open);
        assert_eq!(app.world().resource::<DesignLibrary>().name, "My track");
        let before_history = history(&app);
        for (code, value) in [
            (KeyCode::KeyH, "H"),
            (KeyCode::KeyL, "L"),
            (KeyCode::KeyM, "M"),
            (KeyCode::KeyN, "N"),
            (KeyCode::KeyY, "Y"),
            (KeyCode::KeyZ, "Z"),
            (KeyCode::Space, " "),
        ] {
            text(
                &mut app,
                window,
                code,
                Key::Character(value.into()),
                Some(value),
            );
            frame(&mut app);
        }
        assert_eq!(app.world().resource::<DesignLibrary>().name, "HLMNYZ ");
        let view = app.world().resource::<ViewState>();
        assert!(
            view.designs_open
                && !view.help
                && !view.guests_open
                && !view.services_open
                && !view.finance_open
                && !view.rides_open
        );
        assert_eq!(view.speed, Speed::Paused);
        assert!(view.track_editor.active);
        assert_eq!(app.world().resource::<Simulation>().0, park);
        assert_eq!(history(&app), before_history);
        assert!(!scratch.store().catalog_path().exists());
    }
}

#[test]
fn text_editing_respects_modifier_edges_unicode_limit_and_backspace() {
    let scratch = Scratch::new();
    let (park, view) = fixture();
    let (mut app, window) = app(&scratch, park, view);
    open(&mut app);
    type_text(&mut app, window, "Original name");
    for control in [KeyCode::ControlLeft, KeyCode::ControlRight] {
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(control);
            keys.release(control);
        }
        text(
            &mut app,
            window,
            KeyCode::KeyA,
            Key::Character("a".into()),
            Some("a"),
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyA);
        frame(&mut app);
        type_text(&mut app, window, "É🎢");
        assert_eq!(app.world().resource::<DesignLibrary>().name, "É🎢");
    }
    text(&mut app, window, KeyCode::Backspace, Key::Backspace, None);
    frame(&mut app);
    assert_eq!(app.world().resource::<DesignLibrary>().name, "É");
    command(&mut app, DesignCommand::NameFocus);
    type_text(&mut app, window, &"🎢".repeat(40));
    assert_eq!(
        app.world().resource::<DesignLibrary>().name,
        "🎢".repeat(32)
    );
    text(&mut app, window, KeyCode::Backspace, Key::Backspace, None);
    frame(&mut app);
    type_text(&mut app, window, "é\n\t");
    assert_eq!(
        app.world().resource::<DesignLibrary>().name,
        "🎢".repeat(31) + "é"
    );
    command(&mut app, DesignCommand::NameFocus);
    text(&mut app, window, KeyCode::Backspace, Key::Backspace, None);
    frame(&mut app);
    assert!(app.world().resource::<DesignLibrary>().name.is_empty());
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::ControlRight);
        keys.release(KeyCode::ControlRight);
        keys.press(KeyCode::AltRight);
        keys.release(KeyCode::AltRight);
    }
    text(
        &mut app,
        window,
        KeyCode::KeyE,
        Key::Character("€".into()),
        Some("€"),
    );
    frame(&mut app);
    assert_eq!(app.world().resource::<DesignLibrary>().name, "€");
    text(&mut app, window, KeyCode::Enter, Key::Enter, None);
    frame(&mut app);
    assert!(!app.world().resource::<DesignLibrary>().name_focused);
    type_text(&mut app, window, "ignored");
    assert_eq!(app.world().resource::<DesignLibrary>().name, "€");
}

#[test]
fn close_and_cancel_preempt_queued_writes_and_confirmations() {
    let scratch = Scratch::new();
    let saved = TrackDesign::from_draft("Saved", &specs(10)).unwrap();
    scratch.store().save_new(&saved).unwrap();
    let disk = std::fs::read(scratch.store().catalog_path()).unwrap();
    let (park, view) = fixture();
    let (mut app, _) = app(&scratch, park.clone(), view);
    open(&mut app);
    app.world_mut().resource_mut::<DesignLibrary>().name = "New name".into();
    actions(
        &mut app,
        [
            ui::Action::Design(DesignCommand::SaveNew),
            ui::Action::Design(DesignCommand::Close),
            ui::Action::Track(TrackCommand::Clear),
        ],
    );
    assert!(!app.world().resource::<ViewState>().designs_open);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(std::fs::read(scratch.store().catalog_path()).unwrap(), disk);
    settle(&mut app);
    open(&mut app);
    actions(
        &mut app,
        [
            ui::Action::Design(DesignCommand::UpdateSelected),
            ui::Action::Design(DesignCommand::Confirm),
        ],
    );
    assert_eq!(
        app.world().resource::<DesignLibrary>().confirmation,
        Some(DesignCommand::UpdateSelected)
    );
    assert_eq!(std::fs::read(scratch.store().catalog_path()).unwrap(), disk);
    command(&mut app, DesignCommand::Cancel);
    command(&mut app, DesignCommand::UpdateSelected);
    assert_eq!(
        app.world().resource::<DesignLibrary>().confirmation,
        Some(DesignCommand::UpdateSelected)
    );
    actions(
        &mut app,
        [
            ui::Action::Design(DesignCommand::Confirm),
            ui::Action::Design(DesignCommand::Cancel),
            ui::Action::Design(DesignCommand::SaveNew),
        ],
    );
    assert!(app.world().resource::<ViewState>().designs_open);
    assert!(
        app.world()
            .resource::<DesignLibrary>()
            .confirmation
            .is_none()
    );
    assert_eq!(std::fs::read(scratch.store().catalog_path()).unwrap(), disk);
    assert_eq!(app.world().resource::<Simulation>().0, park);
}

#[test]
fn library_modal_blocks_underlying_buttons_hotkeys_pointer_and_park_saves() {
    let scratch = Scratch::new();
    let (park, view) = fixture();
    let (mut app, window) = app(&scratch, park.clone(), view);
    open(&mut app);
    let before_history = history(&app);
    let point = track_tools::build_center(&park, TilePos { x: 14, y: 17 }, 8, 0);
    let cursor = {
        let window = app.world().get::<Window>(window).unwrap();
        let view = app.world().resource::<ViewState>();
        let (scale, center) = camera_layout(window, view, &park);
        let relative = (point - center) / scale;
        Vec2::new(
            window.width() * 0.5 + relative.x,
            window.height() * 0.5 - relative.y,
        )
    };
    assert!(is_map_position(
        cursor,
        app.world().get::<Window>(window).unwrap(),
        false
    ));
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(cursor));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    for key in [
        KeyCode::ControlLeft,
        KeyCode::KeyZ,
        KeyCode::KeyY,
        KeyCode::KeyO,
        KeyCode::KeyT,
        KeyCode::Delete,
        KeyCode::Space,
        KeyCode::F5,
        KeyCode::F9,
    ] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }
    actions(
        &mut app,
        [
            ui::Action::Track(TrackCommand::Undo),
            ui::Action::Track(TrackCommand::Clear),
            ui::Action::ToggleTrain,
            ui::Action::Terrain,
            ui::Action::Save,
            ui::Action::Load,
            ui::Action::LoadBackup,
        ],
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before_history);
    let view = app.world().resource::<ViewState>();
    assert!(view.designs_open && view.selected.is_none() && view.hovered.is_none());
    assert_eq!(view.speed, Speed::Paused);
    assert!(view.save_pending.is_none());
    assert!(
        !save_store::SaveStore::new(scratch.0.clone())
            .save_path()
            .exists()
    );
    assert!(!scratch.store().catalog_path().exists());
}

#[test]
fn library_loading_is_one_undoable_edit_and_identical_loading_retains_redo() {
    let scratch = Scratch::new();
    let a = TrackDesign::from_draft("A", &specs(15)).unwrap();
    let b = TrackDesign::from_draft("B", &specs(20)).unwrap();
    scratch.store().save_new(&a).unwrap();
    scratch.store().save_new(&b).unwrap();
    let (park, view) = fixture();
    let (mut app, _) = app(&scratch, park.clone(), view);
    let before = history(&app);
    open(&mut app);
    select(&mut app, "B");
    command(&mut app, DesignCommand::LoadDraft);
    let mut expected = park.clone();
    expected.replace_track_draft(b.pieces()).unwrap();
    assert_eq!(app.world().resource::<Simulation>().0, expected);
    assert_eq!(history(&app).0, before.0 + 1);
    settle(&mut app);
    actions(&mut app, [ui::Action::Track(TrackCommand::Undo)]);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    let undone = history(&app);
    assert_eq!(undone.1, 1);
    open(&mut app);
    select(&mut app, "A");
    command(&mut app, DesignCommand::LoadDraft);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), undone);
    settle(&mut app);
    actions(&mut app, [ui::Action::Track(TrackCommand::Redo)]);
    assert_eq!(app.world().resource::<Simulation>().0, expected);
}

#[test]
fn invalid_placement_and_unreadable_design_leave_park_and_history_unchanged() {
    let scratch = Scratch::new();
    let saved = TrackDesign::from_draft("Saved", &specs(10)).unwrap();
    scratch.store().save_new(&saved).unwrap();
    let (mut park, mut view) = fixture();
    track_tools::load_design_draft(&mut park, &mut view, &specs(20)).unwrap();
    track_tools::apply_command(&mut park, &mut view, TrackCommand::Undo);
    let (mut app, _) = app(&scratch, park.clone(), view);
    open(&mut app);
    let before = history(&app);
    {
        let mut library = app.world_mut().resource_mut::<DesignLibrary>();
        library.anchor = TilePos { x: 31, y: 15 };
    }
    command(&mut app, DesignCommand::LoadDraft);
    assert!(
        app.world()
            .resource::<DesignLibrary>()
            .notice
            .starts_with("Design does not fit")
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before);
    {
        let mut library = app.world_mut().resource_mut::<DesignLibrary>();
        library.anchor = TilePos { x: 0, y: 15 };
        library.rotation = 2;
    }
    command(&mut app, DesignCommand::LoadDraft);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before);
    std::fs::write(scratch.store().catalog_path(), b"{broken").unwrap();
    command(&mut app, DesignCommand::LoadDraft);
    assert!(
        app.world()
            .resource::<DesignLibrary>()
            .notice
            .starts_with("Design not loaded")
    );
    assert!(app.world().resource::<ViewState>().designs_open);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before);
    assert_eq!(
        std::fs::read(scratch.store().catalog_path()).unwrap(),
        b"{broken"
    );
}
