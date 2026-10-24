//! Production input ordering with isolated design files; no native window backend.
use super::*;
use crate::{
    design_library::{DesignCommand, DesignLibrary},
    design_store::DesignStore,
    design_transfer::{self, TransferRequest},
    design_transfer_file,
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
    // Windows reports AltGr as Ctrl+Alt. On a Polish layout AltGr+A is ą,
    // not Select All; the following ordinary character must also append.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::ControlLeft);
        keys.release(KeyCode::ControlLeft);
        keys.press(KeyCode::AltRight);
        keys.release(KeyCode::AltRight);
    }
    text(
        &mut app,
        window,
        KeyCode::KeyA,
        Key::Character("ą".into()),
        Some("ą"),
    );
    frame(&mut app);
    assert_eq!(app.world().resource::<DesignLibrary>().name, "€ą");
    type_text(&mut app, window, "x");
    assert_eq!(app.world().resource::<DesignLibrary>().name, "€ąx");
    text(&mut app, window, KeyCode::Enter, Key::Enter, None);
    frame(&mut app);
    assert!(!app.world().resource::<DesignLibrary>().name_focused);
    type_text(&mut app, window, "ignored");
    assert_eq!(app.world().resource::<DesignLibrary>().name, "€ąx");
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

fn gallery_app(scratch: &Scratch, park: Park, view: ViewState) -> (App, Entity) {
    let (mut app, window) = app(scratch, park, view);
    app.add_systems(Startup, |mut commands: Commands| {
        ui::designs::setup(&mut commands);
    })
    .add_systems(
        Update,
        ui::designs::refresh.after(persistence::apply_save_action),
    );
    (app, window)
}

fn save_gallery(scratch: &Scratch) {
    for (index, name) in ["Alpha", "Bravo", "Charlie", "Delta", "Echo"]
        .into_iter()
        .enumerate()
    {
        scratch
            .store()
            .save_new(&TrackDesign::from_draft(name, &specs(10 + index as u32 * 2)).unwrap())
            .unwrap();
    }
}

fn gallery_cards(app: &mut App) -> Vec<(usize, Entity, Display, ui::Action, BorderColor)> {
    let world = app.world_mut();
    let mut cards = world.query::<(
        Entity,
        &ui::designs::DesignThumbnail,
        &Node,
        &ui::Action,
        &BorderColor,
    )>();
    let mut result: Vec<_> = cards
        .iter(world)
        .map(|(entity, slot, node, action, border)| {
            (slot.0, entity, node.display, *action, *border)
        })
        .collect();
    result.sort_by_key(|card| card.0);
    result
}

fn gallery_names(app: &mut App) -> Vec<String> {
    let world = app.world_mut();
    let mut labels = world.query::<(&ui::designs::ThumbnailLabel, &Text)>();
    let mut values: Vec<_> = labels
        .iter(world)
        .map(|(slot, text)| (slot.0, text.0.clone()))
        .collect();
    values.sort_by_key(|(slot, _)| *slot);
    values.into_iter().map(|(_, value)| value).collect()
}

fn press_widget(app: &mut App, entity: Entity) {
    *app.world_mut().get_mut::<Interaction>(entity).unwrap() = Interaction::Pressed;
    frame(app);
    *app.world_mut().get_mut::<Interaction>(entity).unwrap() = Interaction::None;
    frame(app);
}

fn press_gallery_control(app: &mut App, command: DesignCommand) {
    let world = app.world_mut();
    let mut buttons = world.query_filtered::<(Entity, &ui::Action), With<Button>>();
    let entity = buttons
        .iter(world)
        .find_map(|(entity, action)| (*action == ui::Action::Design(command)).then_some(entity))
        .expect("gallery setup must create the real requested control");
    press_widget(app, entity);
}

#[test]
fn real_gallery_cards_follow_selection_across_pages_and_close_without_editing() {
    let scratch = Scratch::new();
    save_gallery(&scratch);
    let (park, view) = fixture();
    let (mut app, _) = gallery_app(&scratch, park.clone(), view);
    let before_history = history(&app);
    open(&mut app);
    let first = gallery_cards(&mut app);
    assert_eq!(first.len(), 4);
    for (index, card) in first.iter().enumerate() {
        assert_eq!(card.2, Display::Flex);
        assert!(card.3 == ui::Action::Design(DesignCommand::Select(index)));
    }
    assert_eq!(
        gallery_names(&mut app),
        ["Alpha", "Bravo", "Charlie", "Delta"]
    );
    let selected_border = first[0].4;
    let unselected_border = first[1].4;
    assert_ne!(selected_border, unselected_border);
    press_widget(&mut app, first[2].1);
    assert_eq!(app.world().resource::<DesignLibrary>().selected, Some(2));
    let selected = gallery_cards(&mut app);
    assert_eq!(selected[2].4, selected_border);
    assert_eq!(selected[0].4, unselected_border);

    press_gallery_control(&mut app, DesignCommand::Browse(1));
    press_gallery_control(&mut app, DesignCommand::Browse(1));
    let last = gallery_cards(&mut app);
    assert_eq!(app.world().resource::<DesignLibrary>().selected, Some(4));
    assert_eq!(last[0].2, Display::Flex);
    assert!(last[0].3 == ui::Action::Design(DesignCommand::Select(4)));
    assert_eq!(last[0].4, selected_border);
    assert_eq!(gallery_names(&mut app), ["Echo", "", "", ""]);
    for card in &last[1..] {
        assert_eq!(card.2, Display::None);
        assert!(card.3 == ui::Action::Design(DesignCommand::Select(usize::MAX)));
    }
    // Selecting the actual last-page widget must dispatch Select(4), even
    // though its physical gallery slot is zero.
    {
        let mut library = app.world_mut().resource_mut::<DesignLibrary>();
        library.name = "Unsaved typing".into();
        library.rotation = 3;
    }
    press_widget(&mut app, last[0].1);
    let library = app.world().resource::<DesignLibrary>();
    assert_eq!(library.selected, Some(4));
    assert_eq!(library.name, "Echo");
    assert_eq!(library.rotation, 0);
    assert_eq!(library.anchor, TilePos { x: 18, y: 15 });
    // Hidden cards have an invalid, disabled command even if a synthetic
    // input event bypasses normal hit testing.
    press_widget(&mut app, last[1].1);
    assert_eq!(app.world().resource::<DesignLibrary>().selected, Some(4));
    press_gallery_control(&mut app, DesignCommand::Close);
    assert!(!app.world().resource::<ViewState>().designs_open);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before_history);
}

#[test]
fn real_gallery_refreshes_actions_names_and_selection_after_confirmed_changes() {
    let scratch = Scratch::new();
    save_gallery(&scratch);
    let initial_disk = std::fs::read(scratch.store().catalog_path()).unwrap();
    let (park, view) = fixture();
    let (mut app, window) = gallery_app(&scratch, park.clone(), view);
    let before_history = history(&app);
    open(&mut app);
    let initial = gallery_cards(&mut app);
    let selected_border = initial[0].4;
    let unselected_border = initial[1].4;
    press_widget(&mut app, initial[1].1);
    press_gallery_control(&mut app, DesignCommand::DeleteSelected);
    assert_eq!(
        app.world().resource::<DesignLibrary>().confirmation,
        Some(DesignCommand::DeleteSelected)
    );
    press_widget(&mut app, initial[2].1);
    assert_eq!(app.world().resource::<DesignLibrary>().selected, Some(1));
    assert_eq!(
        std::fs::read(scratch.store().catalog_path()).unwrap(),
        initial_disk
    );
    assert_eq!(gallery_cards(&mut app)[1].4, selected_border);
    press_gallery_control(&mut app, DesignCommand::Cancel);

    press_gallery_control(&mut app, DesignCommand::NameFocus);
    type_text(&mut app, window, "Zulu renamed");
    press_gallery_control(&mut app, DesignCommand::UpdateSelected);
    press_gallery_control(&mut app, DesignCommand::Confirm);
    let renamed = gallery_cards(&mut app);
    assert_eq!(app.world().resource::<DesignLibrary>().selected, Some(4));
    assert_eq!(renamed[0].2, Display::Flex);
    assert!(renamed[0].3 == ui::Action::Design(DesignCommand::Select(4)));
    assert_eq!(renamed[0].4, selected_border);
    assert_eq!(gallery_names(&mut app), ["Zulu renamed", "", "", ""]);
    assert!(renamed[1..].iter().all(|card| card.2 == Display::None));
    assert_eq!(
        scratch.store().load("Zulu renamed").unwrap().pieces(),
        park.track_draft()
    );
    assert!(scratch.store().load("Bravo").is_err());

    press_gallery_control(&mut app, DesignCommand::DeleteSelected);
    press_gallery_control(&mut app, DesignCommand::Confirm);
    let remaining = gallery_cards(&mut app);
    assert_eq!(app.world().resource::<DesignLibrary>().selected, Some(0));
    assert_eq!(
        gallery_names(&mut app),
        ["Alpha", "Charlie", "Delta", "Echo"]
    );
    for (index, card) in remaining.iter().enumerate() {
        assert_eq!(card.2, Display::Flex);
        assert!(card.3 == ui::Action::Design(DesignCommand::Select(index)));
        assert_eq!(
            card.4,
            if index == 0 {
                selected_border
            } else {
                unselected_border
            }
        );
    }
    assert!(scratch.store().load("Zulu renamed").is_err());
    press_gallery_control(&mut app, DesignCommand::Close);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before_history);
}

fn complete_test_transfer(
    app: &mut App,
    scratch: &Scratch,
    choice: Result<Option<PathBuf>, String>,
) -> bool {
    let result = {
        let mut library = app.world_mut().resource_mut::<DesignLibrary>();
        let request = library
            .transfer_request
            .take()
            .expect("UI must queue a file request");
        assert!(library.transfer_pending);
        design_transfer::complete_transfer(&scratch.store(), &mut library, request, choice)
    };
    frame(app);
    let library = app.world().resource::<DesignLibrary>();
    assert!(!library.transfer_pending && library.transfer_request.is_none());
    result
}

fn assert_no_transfer(app: &App) {
    let library = app.world().resource::<DesignLibrary>();
    assert!(!library.transfer_pending);
    assert!(library.transfer_request.is_none());
}

#[test]
fn real_transfer_buttons_export_saved_geometry_and_select_a_numbered_import() {
    let scratch = Scratch::new();
    let saved = TrackDesign::from_draft("Shared coaster é", &specs(10)).unwrap();
    scratch.store().save_new(&saved).unwrap();
    let initial_catalog = std::fs::read(scratch.store().catalog_path()).unwrap();
    let (park, view) = fixture();
    assert_ne!(park.track_draft(), saved.pieces());
    let (mut app, window) = gallery_app(&scratch, park.clone(), view);
    let before_history = history(&app);
    open(&mut app);
    let original_border = gallery_cards(&mut app)[0].4;
    press_gallery_control(&mut app, DesignCommand::NameFocus);
    type_text(&mut app, window, "Unsaved draft name");
    for command in [
        DesignCommand::Turn,
        DesignCommand::MoveX(1),
        DesignCommand::Height(1),
    ] {
        press_gallery_control(&mut app, command);
    }
    press_gallery_control(&mut app, DesignCommand::Export);
    let request = app
        .world()
        .resource::<DesignLibrary>()
        .transfer_request
        .clone();
    assert_eq!(request, Some(TransferRequest::Export(saved.clone())));
    assert!(app.world().resource::<DesignLibrary>().transfer_pending);
    for command in [
        DesignCommand::Import,
        DesignCommand::LoadDraft,
        DesignCommand::UpdateSelected,
    ] {
        press_gallery_control(&mut app, command);
        assert_eq!(
            app.world().resource::<DesignLibrary>().transfer_request,
            request
        );
    }
    let exported = scratch.0.join("Shared coaster é.parktrack.json");
    assert!(complete_test_transfer(
        &mut app,
        &scratch,
        Ok(Some(exported.clone()))
    ));
    assert_eq!(design_transfer_file::read_design(&exported).unwrap(), saved);
    assert_eq!(
        std::fs::read(scratch.store().catalog_path()).unwrap(),
        initial_catalog
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before_history);

    press_gallery_control(&mut app, DesignCommand::Import);
    assert_eq!(
        app.world().resource::<DesignLibrary>().transfer_request,
        Some(TransferRequest::Import)
    );
    assert!(complete_test_transfer(
        &mut app,
        &scratch,
        Ok(Some(exported.clone()))
    ));
    let imported = scratch.store().load("Shared coaster é (2)").unwrap();
    assert_eq!(imported.pieces(), saved.pieces());
    assert_eq!(scratch.store().load(saved.name()).unwrap(), saved);
    assert_eq!(
        app.world().resource::<DesignLibrary>().selected_design(),
        Some(&imported)
    );
    assert_eq!(
        gallery_names(&mut app),
        ["Shared coaster é", "Shared coaster é (2)", "", ""]
    );
    let cards = gallery_cards(&mut app);
    assert_eq!(cards[1].2, Display::Flex);
    assert!(cards[1].3 == ui::Action::Design(DesignCommand::Select(1)));
    assert_eq!(cards[1].4, original_border);
    assert_ne!(cards[0].4, original_border);
    assert_eq!(design_transfer_file::read_design(&exported).unwrap(), saved);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before_history);
}

#[test]
fn transfer_cancellation_and_errors_preserve_catalog_park_history_and_existing_files() {
    let scratch = Scratch::new();
    let saved = TrackDesign::from_draft("Saved", &specs(10)).unwrap();
    scratch.store().save_new(&saved).unwrap();
    let catalog = std::fs::read(scratch.store().catalog_path()).unwrap();
    let (park, view) = fixture();
    let (mut app, _) = gallery_app(&scratch, park.clone(), view);
    let before_history = history(&app);
    open(&mut app);
    let entries = app.world().resource::<DesignLibrary>().entries.clone();
    for command in [DesignCommand::Import, DesignCommand::Export] {
        for choice in [
            Ok(None),
            Err("private path: C:\\private\\choice.json".into()),
        ] {
            press_gallery_control(&mut app, command);
            assert!(!complete_test_transfer(&mut app, &scratch, choice));
            let library = app.world().resource::<DesignLibrary>();
            assert_eq!(library.entries, entries);
            assert!(!library.notice.contains("C:\\private"));
            assert_eq!(
                std::fs::read(scratch.store().catalog_path()).unwrap(),
                catalog
            );
            assert_eq!(app.world().resource::<Simulation>().0, park);
            assert_eq!(history(&app), before_history);
        }
    }
    let malformed = scratch.0.join("Invalid.parktrack.json");
    std::fs::write(&malformed, b"{not a design").unwrap();
    press_gallery_control(&mut app, DesignCommand::Import);
    assert!(!complete_test_transfer(
        &mut app,
        &scratch,
        Ok(Some(malformed.clone()))
    ));
    assert!(
        app.world()
            .resource::<DesignLibrary>()
            .notice
            .starts_with("Import refused:")
    );
    assert_eq!(std::fs::read(&malformed).unwrap(), b"{not a design");
    let wrong_extension = scratch.0.join("Existing park.park.json");
    std::fs::write(&wrong_extension, b"existing unrelated bytes").unwrap();
    press_gallery_control(&mut app, DesignCommand::Export);
    assert!(!complete_test_transfer(
        &mut app,
        &scratch,
        Ok(Some(wrong_extension.clone()))
    ));
    assert!(
        app.world()
            .resource::<DesignLibrary>()
            .notice
            .starts_with("Export failed:")
    );
    assert_eq!(
        std::fs::read(&wrong_extension).unwrap(),
        b"existing unrelated bytes"
    );
    assert_eq!(app.world().resource::<DesignLibrary>().entries, entries);
    assert_eq!(
        std::fs::read(scratch.store().catalog_path()).unwrap(),
        catalog
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before_history);
}

#[test]
fn confirmation_close_cancel_and_same_frame_ordering_preempt_file_requests() {
    let scratch = Scratch::new();
    scratch
        .store()
        .save_new(&TrackDesign::from_draft("Saved", &specs(10)).unwrap())
        .unwrap();
    let (park, view) = fixture();
    let (mut app, _) = gallery_app(&scratch, park.clone(), view);
    let before_history = history(&app);
    open(&mut app);
    for confirmation in [DesignCommand::UpdateSelected, DesignCommand::DeleteSelected] {
        press_gallery_control(&mut app, confirmation);
        for command in [DesignCommand::Import, DesignCommand::Export] {
            press_gallery_control(&mut app, command);
            assert_no_transfer(&app);
            assert_eq!(
                app.world().resource::<DesignLibrary>().confirmation,
                Some(confirmation)
            );
        }
        press_gallery_control(&mut app, DesignCommand::Cancel);
    }
    // Import/Export were disabled when this frame began. Confirming a saved
    // update must not also activate either disabled file action in that frame.
    for transfer in [DesignCommand::Import, DesignCommand::Export] {
        for confirm_first in [true, false] {
            press_gallery_control(&mut app, DesignCommand::UpdateSelected);
            let pair = if confirm_first {
                [DesignCommand::Confirm, transfer]
            } else {
                [transfer, DesignCommand::Confirm]
            };
            actions(&mut app, pair.into_iter().map(ui::Action::Design));
            assert_no_transfer(&app);
            assert!(
                app.world()
                    .resource::<DesignLibrary>()
                    .confirmation
                    .is_none()
            );
        }
    }
    let catalog = std::fs::read(scratch.store().catalog_path()).unwrap();
    for dismiss in [DesignCommand::Close, DesignCommand::Cancel] {
        for transfer in [DesignCommand::Import, DesignCommand::Export] {
            for dismiss_first in [true, false] {
                if !app.world().resource::<ViewState>().designs_open {
                    open(&mut app);
                }
                let pair = if dismiss_first {
                    [dismiss, transfer]
                } else {
                    [transfer, dismiss]
                };
                actions(&mut app, pair.into_iter().map(ui::Action::Design));
                assert!(!app.world().resource::<ViewState>().designs_open);
                assert_no_transfer(&app);
                settle(&mut app);
            }
        }
    }
    for command in [DesignCommand::Import, DesignCommand::Export] {
        open(&mut app);
        press_gallery_control(&mut app, command);
        assert!(app.world().resource::<DesignLibrary>().transfer_pending);
        press_gallery_control(&mut app, DesignCommand::Close);
        assert_no_transfer(&app);
    }
    assert_eq!(
        std::fs::read(scratch.store().catalog_path()).unwrap(),
        catalog
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before_history);
}

#[test]
fn export_refuses_a_saved_design_changed_externally_until_the_library_reopens() {
    let scratch = Scratch::new();
    let old = TrackDesign::from_draft("Saved", &specs(10)).unwrap();
    let new = TrackDesign::from_draft("Saved", &specs(20)).unwrap();
    scratch.store().save_new(&old).unwrap();
    let (park, view) = fixture();
    let (mut app, _) = gallery_app(&scratch, park.clone(), view);
    let before_history = history(&app);
    open(&mut app);
    scratch.store().replace_exact(&old, &new).unwrap();
    let changed_catalog = std::fs::read(scratch.store().catalog_path()).unwrap();
    press_gallery_control(&mut app, DesignCommand::Export);
    assert_no_transfer(&app);
    assert!(
        app.world()
            .resource::<DesignLibrary>()
            .notice
            .starts_with("Saved design changed")
    );
    assert_eq!(
        app.world().resource::<DesignLibrary>().selected_design(),
        Some(&old)
    );
    assert_eq!(
        std::fs::read(scratch.store().catalog_path()).unwrap(),
        changed_catalog
    );
    press_gallery_control(&mut app, DesignCommand::Close);
    open(&mut app);
    press_gallery_control(&mut app, DesignCommand::Export);
    assert_eq!(
        app.world().resource::<DesignLibrary>().transfer_request,
        Some(TransferRequest::Export(new))
    );
    assert!(!complete_test_transfer(&mut app, &scratch, Ok(None)));
    assert_eq!(app.world().resource::<Simulation>().0, park);
    assert_eq!(history(&app), before_history);
}
