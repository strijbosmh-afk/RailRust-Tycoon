//! Saved accessibility preferences and one shared keyboard path for park controls.
use crate::{
    ViewState,
    persistence::SaveSession,
    save_store::{SaveLock, SaveStore, StagedFile},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, io::Read};

const FILE: &str = "accessibility-settings.json";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    version: u8,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub captions: bool,
    pub guided_start: bool,
    pub bindings: [u8; 18],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            high_contrast: false,
            reduced_motion: false,
            captions: true,
            guided_start: true,
            bindings: std::array::from_fn(|i| i as u8),
        }
    }
}
const KEYS: [KeyCode; 22] = [
    KeyCode::Space,
    KeyCode::KeyQ,
    KeyCode::KeyE,
    KeyCode::KeyG,
    KeyCode::KeyP,
    KeyCode::KeyJ,
    KeyCode::KeyI,
    KeyCode::KeyK,
    KeyCode::KeyB,
    KeyCode::KeyV,
    KeyCode::KeyL,
    KeyCode::KeyM,
    KeyCode::KeyN,
    KeyCode::KeyY,
    KeyCode::KeyZ,
    KeyCode::F5,
    KeyCode::F9,
    KeyCode::KeyF,
    KeyCode::F3,
    KeyCode::F4,
    KeyCode::F8,
    KeyCode::F12,
];
const NAMES: [&str; 18] = [
    "Pause",
    "Rotate left",
    "Rotate right",
    "Grid",
    "Paths",
    "Queues",
    "Entrance",
    "Exit",
    "Raise terrain",
    "Lower terrain",
    "Guests",
    "Facilities",
    "Finances",
    "Rides",
    "Track editor",
    "Save",
    "Load",
    "Find train",
];
impl Settings {
    fn read(store: &SaveStore) -> Result<Self, String> {
        let mut bytes = Vec::new();
        std::fs::File::open(store.directory().join(FILE))
            .map_err(|e| e.to_string())?
            .take(8193)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 8192 {
            return Err("Accessibility settings exceed 8 KiB".into());
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if value.version != 1
            || value
                .bindings
                .iter()
                .any(|key| usize::from(*key) >= KEYS.len())
        {
            return Err("Unsupported accessibility settings".into());
        }
        let mut sorted = value.bindings;
        sorted.sort();
        if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err("Duplicate shortcut keys".into());
        }
        Ok(value)
    }
    fn write(&self, store: &SaveStore) -> Result<(), String> {
        store.check_directory(true).map_err(|e| e.to_string())?;
        let _lock = SaveLock::acquire(store.directory()).map_err(|e| e.to_string())?;
        let mut file = StagedFile::write(
            store.directory(),
            "accessibility",
            &serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        file.replace(&store.directory().join(FILE))
            .map_err(|e| e.to_string())
    }
    pub fn shortcut(&self, action: usize) -> String {
        key_name(KEYS[usize::from(self.bindings[action])])
    }
    fn cycle(&mut self, action: usize) {
        let current = usize::from(self.bindings[action]);
        let next = (1..=KEYS.len())
            .map(|step| (current + step) % KEYS.len())
            .find(|candidate| {
                !self
                    .bindings
                    .iter()
                    .enumerate()
                    .any(|(i, k)| i != action && usize::from(*k) == *candidate)
            })
            .unwrap_or(current);
        self.bindings[action] = next as u8;
    }
}
fn key_name(key: KeyCode) -> String {
    format!("{key:?}").trim_start_matches("Key").to_owned()
}
#[derive(Resource)]
pub(crate) struct Preferences {
    pub value: Settings,
    store: Option<SaveStore>,
    pub notice: String,
}
impl Preferences {
    pub(crate) fn load(store: Option<SaveStore>) -> Self {
        let mut notice = String::new();
        let value = store
            .as_ref()
            .filter(|s| s.directory().join(FILE).exists())
            .map(|s| {
                Settings::read(s).unwrap_or_else(|error| {
                    notice = format!("Using default controls. {error}");
                    Settings::default()
                })
            })
            .unwrap_or_default();
        Self {
            value,
            store,
            notice,
        }
    }
    fn persist(&mut self) {
        self.notice = self.store.as_ref().map_or_else(
            || "Preview settings for this session".into(),
            |store| {
                self.value.write(store).map_or_else(
                    |error| format!("Changed for this session. Could not save settings. {error}"),
                    |_| "Settings saved".into(),
                )
            },
        );
    }
}
#[derive(Component, Clone, Copy)]
pub(crate) enum Control {
    Contrast,
    Motion,
    Captions,
    Guide,
    Binding(usize),
    Reset,
    Close,
}
#[derive(Component)]
struct Panel;
#[derive(Component)]
struct Summary;
#[derive(Resource, Default)]
struct Navigation {
    focus: Option<Entity>,
    active: bool,
    original: Option<ButtonInput<KeyCode>>,
    outlined: Option<Entity>,
    modal: Vec<bool>,
}

#[derive(SystemSet, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct AccessibilityStyle;

#[derive(Component)]
pub(crate) struct KeyboardFocus;
#[derive(Component)]
struct OriginalText {
    base: Color,
    applied: Color,
}
#[derive(Component)]
struct OriginalBackground {
    base: Color,
    applied: Color,
}

pub(crate) fn configure(app: &mut App, args: &[OsString]) {
    let quiet = args.iter().any(|s| {
        s == "--capture-frame"
            || s.to_string_lossy().starts_with("--profile-")
            || s.to_string_lossy().starts_with("--render-")
    });
    let store = if quiet {
        None
    } else {
        app.world().resource::<SaveSession>().store.clone()
    };
    let mut prefs = Preferences::load(store);
    prefs.value.high_contrast |= args.iter().any(|s| s == "--capture-high-contrast");
    prefs.value.reduced_motion |= args.iter().any(|s| s == "--capture-reduced-motion");
    app.insert_resource(prefs)
        .init_resource::<Navigation>()
        .add_systems(Startup, setup.after(crate::ui::setup))
        .add_systems(Update, prepare.before(crate::construction_history::begin))
        .add_systems(
            Update,
            (apply, refresh, shortcut_labels)
                .chain()
                .after(crate::ui::navigation::refresh),
        )
        .add_systems(
            PostUpdate,
            (restore, style.in_set(AccessibilityStyle))
                .chain()
                .after(bevy::ui::UiSystems::Layout),
        );
}
fn setup(mut commands: Commands) {
    use crate::ui::{INK, MUTED, PAPER, label, row};
    commands.spawn((Panel, GlobalZIndex(40), Node {display:Display::None, position_type:PositionType::Absolute, width:percent(100), height:percent(100), align_items:AlignItems::Center, justify_content:JustifyContent::Center,..default()}, BackgroundColor(Color::srgba(0.02,0.07,0.05,0.92))))
        .with_children(|p| { p.spawn((crate::ui::layout::ScrollLayer::Accessibility, ScrollPosition::default(), crate::ui::layout::dialog(94.0,720.0,None),BackgroundColor(PAPER),BorderRadius::all(px(12))))
            .with_children(|p| {
                label(p,"Controls & accessibility",24.0,INK);
                label(p,"Tab / Shift+Tab selects a control. Enter activates it. Escape returns.\nClick a shortcut to cycle through unused keys. F1, F2, F10, F11 and Escape remain available.",13.0,MUTED);
                for (name,control) in [("High contrast",Control::Contrast),("Reduced motion",Control::Motion),("Sound captions",Control::Captions),("Guided new parks",Control::Guide)] { control_button(p,name,control); }
                p.spawn((Text::new(""),TextFont{font_size:13.0,..default()},TextColor(INK),Summary));
                for (i,name) in NAMES.iter().enumerate() { control_button(p,name,Control::Binding(i)); }
                p.spawn(row()).with_children(|p| {control_button(p,"Restore defaults",Control::Reset);control_button(p,"Back [Esc]",Control::Close);});

            }); });
}
fn control_button(p: &mut ChildSpawnerCommands, name: &str, control: Control) {
    p.spawn((
        Button,
        control,
        Node {
            min_height: px(32),
            flex_shrink: 0.0,
            padding: UiRect::all(px(6)),
            ..default()
        },
        BackgroundColor(crate::ui::BUTTON),
        BorderRadius::all(px(5)),
    ))
    .with_children(|p| {
        crate::ui::label(p, name, 14.0, crate::ui::INK);
    });
}
fn translated(raw: &ButtonInput<KeyCode>, settings: &Settings) -> ButtonInput<KeyCode> {
    // Text entry and Ctrl/Alt chords bypass translation in prepare. Remapping is
    // simultaneous so swapping two keys never causes a second mapping pass.
    let mut result = raw.clone();
    for (i, source) in settings.bindings.iter().enumerate() {
        if usize::from(*source) != i {
            result.reset(KEYS[i]);
            result.reset(KEYS[usize::from(*source)]);
        }
    }
    for (i, source) in settings.bindings.iter().enumerate() {
        if usize::from(*source) == i {
            continue;
        }
        let source = KEYS[usize::from(*source)];
        let target = KEYS[i];
        if raw.pressed(source) || raw.just_pressed(source) {
            result.press(target);
            if !raw.just_pressed(source) {
                result.clear_just_pressed(target);
            }
        }
        if raw.just_released(source) {
            result.press(target);
            result.clear_just_pressed(target);
            result.release(target);
        }
    }
    result
}
type FocusButtons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static mut Interaction,
    ),
    With<Button>,
>;
type InputResources<'w, 's> = (
    Option<ResMut<'w, crate::design_library::DesignLibrary>>,
    Option<Res<'w, ButtonInput<MouseButton>>>,
    Query<'w, 's, &'static ChildOf>,
    Query<'w, 's, &'static GlobalZIndex>,
);
fn prepare(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    prefs: Res<Preferences>,
    mut nav: ResMut<Navigation>,
    mut view: ResMut<ViewState>,
    mut buttons: FocusButtons<'_, '_>,
    resources: InputResources<'_, '_>,
) {
    let (mut library, mouse, parents, layers) = resources;
    nav.original = Some(keys.clone());
    let modal = vec![
        view.front_modal,
        view.accessibility_open,
        view.help,
        view.navigation_open,
        view.workshop_open,
        view.management_open,
        view.park_life_open,
        view.catalogue_open,
        view.designs_open,
        view.rides_open,
        view.guests_open,
        view.services_open,
        view.finance_open,
    ];
    if modal != nav.modal {
        nav.focus = None;
        nav.active = false;
        nav.modal = modal;
    }

    if view.front_modal && !view.accessibility_open {
        nav.focus = None;
        nav.active = false;
        return;
    }
    if mouse.as_ref().is_some_and(|m| {
        m.any_just_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle])
    }) {
        nav.active = false;
        nav.focus = None;
    }
    let text_focused = view.ride_name_focused
        || library
            .as_ref()
            .is_some_and(|l| l.name_focused || l.search_focused);
    if keys.just_pressed(KeyCode::Tab) {
        view.ride_name_focused = false;
        if let Some(library) = library.as_mut() {
            library.name_focused = false;
            library.search_focused = false;
        }
    }
    let typing = text_focused && !keys.just_pressed(KeyCode::Tab);
    if !typing && keys.just_pressed(KeyCode::Tab) {
        let layer_for = |mut id: Entity| {
            let mut layer = 0;
            loop {
                if let Ok(value) = layers.get(id) {
                    layer = layer.max(value.0);
                }
                match parents.get(id) {
                    Ok(parent) => id = parent.parent(),
                    Err(_) => break,
                }
            }
            layer
        };
        let top = buttons
            .iter_mut()
            .filter(|(_, node, _, _)| node.size().min_element() > 0.0)
            .map(|(id, _, _, _)| layer_for(id))
            .max()
            .unwrap_or(0);
        let mut candidates: Vec<_> = buttons
            .iter_mut()
            .filter(|(id, node, _, _)| {
                node.size().min_element() > 0.0 && (!view.edits_blocked() || layer_for(*id) == top)
            })
            .map(|(id, _, transform, _)| (id, transform.translation))
            .collect();
        candidates.sort_by(|a, b| {
            a.1.y
                .total_cmp(&b.1.y)
                .then(a.1.x.total_cmp(&b.1.x))
                .then(a.0.to_bits().cmp(&b.0.to_bits()))
        });
        if !candidates.is_empty() {
            let reverse = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            let index = candidates
                .iter()
                .position(|(id, _)| Some(*id) == nav.focus)
                .map_or(if reverse { candidates.len() - 1 } else { 0 }, |i| {
                    if reverse {
                        (i + candidates.len() - 1) % candidates.len()
                    } else {
                        (i + 1) % candidates.len()
                    }
                });
            nav.focus = Some(candidates[index].0);
            nav.active = true;
        }
        keys.clear_just_pressed(KeyCode::Tab);
    }
    if nav.active && !typing && keys.just_pressed(KeyCode::Enter) {
        if let Some(id) = nav.focus
            && let Ok((_, node, _, mut interaction)) = buttons.get_mut(id)
            && node.size().min_element() > 0.0
        {
            *interaction = Interaction::Pressed;
            keys.clear_just_pressed(KeyCode::Enter);
        } else {
            nav.active = false;
            nav.focus = None;
        }
    }
    if view.accessibility_open {
        return;
    }
    if !view.front_modal
        && !typing
        && !view.designs_open
        && !view.workshop_open
        && !keys.any_pressed([
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::AltLeft,
            KeyCode::AltRight,
        ])
    {
        *keys = translated(&keys, &prefs.value);
    }
}
fn restore(mut keys: ResMut<ButtonInput<KeyCode>>, mut nav: ResMut<Navigation>) {
    if let Some(original) = nav.original.take() {
        *keys = original;
    }
}
fn apply(
    mut prefs: ResMut<Preferences>,
    mut view: ResMut<ViewState>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Query<(&Interaction, &Control), Changed<Interaction>>,
) {
    if !view.accessibility_open {
        return;
    }
    view.ui_pointer_consumed = true;
    if keys.just_pressed(KeyCode::Escape) {
        view.accessibility_open = false;
        view.modal_transition_pending = true;
        return;
    }
    for (interaction, control) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match control {
            Control::Contrast => prefs.value.high_contrast = !prefs.value.high_contrast,
            Control::Motion => prefs.value.reduced_motion = !prefs.value.reduced_motion,
            Control::Captions => prefs.value.captions = !prefs.value.captions,
            Control::Guide => prefs.value.guided_start = !prefs.value.guided_start,
            Control::Binding(i) => prefs.value.cycle(*i),
            Control::Reset => prefs.value = Settings::default(),
            Control::Close => {
                view.accessibility_open = false;
                view.modal_transition_pending = true;
                return;
            }
        }
        prefs.persist();
        break;
    }
}
fn refresh(
    view: Res<ViewState>,
    prefs: Res<Preferences>,
    mut panels: Query<&mut Node, With<Panel>>,
    mut summary: Query<&mut Text, With<Summary>>,
    buttons: Query<(&Control, &Children)>,
    mut text: Query<&mut Text, Without<Summary>>,
) {
    for mut panel in &mut panels {
        panel.display = if view.accessibility_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut summary in &mut summary {
        summary.0 = prefs.notice.clone();
    }
    for (control, children) in &buttons {
        let caption = match control {
            Control::Contrast => format!(
                "High contrast · {}",
                if prefs.value.high_contrast {
                    "On"
                } else {
                    "Off"
                }
            ),
            Control::Motion => format!(
                "Reduced motion · {}",
                if prefs.value.reduced_motion {
                    "On"
                } else {
                    "Off"
                }
            ),
            Control::Captions => format!(
                "Sound captions · {}",
                if prefs.value.captions { "On" } else { "Off" }
            ),
            Control::Guide => format!(
                "Guided new parks · {}",
                if prefs.value.guided_start {
                    "On"
                } else {
                    "Off"
                }
            ),
            Control::Binding(i) => format!("{} · {}", NAMES[*i], prefs.value.shortcut(*i)),
            _ => continue,
        };
        for child in children {
            if let Ok(mut label) = text.get_mut(*child) {
                label.0 = caption.clone();
            }
        }
    }
}
type StyleResources<'w, 's> = (
    Query<'w, 's, &'static ChildOf>,
    Query<
        'w,
        's,
        (
            &'static mut ScrollPosition,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform)>,
);
fn style(
    mut commands: Commands,
    prefs: Res<Preferences>,
    mut nav: ResMut<Navigation>,
    mut texts: Query<(Entity, &mut TextColor, Option<&mut OriginalText>)>,
    mut backgrounds: Query<(
        Entity,
        &mut BackgroundColor,
        Option<&mut OriginalBackground>,
    )>,
    buttons: Query<Entity, With<Button>>,
    resources: StyleResources<'_, '_>,
) {
    let (parents, mut scrolls, positions) = resources;
    let background_colors: std::collections::HashMap<_, _> = backgrounds
        .iter_mut()
        .map(|(id, color, _)| (id, color.0))
        .collect();
    for (id, mut color, original) in &mut texts {
        let mut base = color.0;
        if let Some(mut original) = original {
            if color.0 != original.applied {
                original.base = color.0;
            }
            base = original.base;
            let desired = if prefs.value.high_contrast {
                text_contrast(id, base, &parents, &background_colors)
            } else {
                base
            };
            if color.0 != desired {
                color.0 = desired;
            }
            original.applied = color.0;
        } else {
            let desired = if prefs.value.high_contrast {
                text_contrast(id, base, &parents, &background_colors)
            } else {
                base
            };
            if color.0 != desired {
                color.0 = desired;
            }
            commands.entity(id).insert(OriginalText {
                base,
                applied: color.0,
            });
        }
    }
    for (id, mut color, original) in &mut backgrounds {
        let mut base = color.0;
        if let Some(mut original) = original {
            if color.0 != original.applied {
                original.base = color.0;
            }
            base = original.base;
            let desired = if prefs.value.high_contrast {
                contrast(base, false)
            } else {
                base
            };
            if color.0 != desired {
                color.0 = desired;
            }
            original.applied = color.0;
        } else {
            let desired = if prefs.value.high_contrast {
                contrast(base, false)
            } else {
                base
            };
            if color.0 != desired {
                color.0 = desired;
            }
            commands.entity(id).insert(OriginalBackground {
                base,
                applied: color.0,
            });
        }
    }
    let focused = nav.focus.filter(|_| nav.active);
    if nav.outlined != focused || prefs.is_changed() {
        for id in &buttons {
            if focused == Some(id) {
                commands.entity(id).insert(KeyboardFocus);
                commands.entity(id).insert(Outline::new(
                    px(3),
                    px(1),
                    Color::srgb_u8(30, 110, 245),
                ));
            } else if prefs.value.high_contrast {
                commands.entity(id).remove::<KeyboardFocus>();
                commands
                    .entity(id)
                    .insert(Outline::new(px(1), px(0), Color::BLACK));
            } else {
                commands.entity(id).remove::<KeyboardFocus>();
                commands.entity(id).remove::<Outline>();
            }
        }
        nav.outlined = focused;
    }
    if nav.active
        && let Some(focus) = nav.focus
        && let Ok((node, position)) = positions.get(focus)
    {
        let top = position.translation.y - node.size().y / 2.0;
        let bottom = top + node.size().y;
        let mut current = focus;
        while let Ok(parent) = parents.get(current) {
            current = parent.parent();
            if let Ok((mut scroll, container, transform)) = scrolls.get_mut(current) {
                let low = transform.translation.y - container.size().y / 2.0;
                let high = low + container.size().y;
                scroll.0.y = (scroll.0.y
                    + if bottom > high {
                        bottom - high
                    } else if top < low {
                        top - low
                    } else {
                        0.0
                    } * container.inverse_scale_factor())
                .max(0.0);
                break;
            }
        }
    }
}
fn text_contrast(
    mut id: Entity,
    base: Color,
    parents: &Query<&ChildOf>,
    backgrounds: &std::collections::HashMap<Entity, Color>,
) -> Color {
    while let Ok(parent) = parents.get(id) {
        id = parent.parent();
        if let Some(color) = backgrounds.get(&id) {
            let c = color.to_srgba();
            if c.alpha > 0.95 {
                return if 0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue > 0.5 {
                    Color::BLACK
                } else {
                    Color::WHITE
                };
            }
        }
    }
    contrast(base, true)
}
fn contrast(color: Color, text: bool) -> Color {
    let c = color.to_srgba();
    if c.alpha < 0.95 {
        return color;
    }
    let light = 0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue;
    if text {
        if light > 0.62 {
            Color::WHITE
        } else {
            Color::BLACK
        }
    } else if light > 0.5 {
        Color::WHITE
    } else if light < 0.35 {
        Color::srgb_u8(10, 27, 20)
    } else {
        color
    }
}
pub(crate) fn run_check(args: &[OsString]) -> Option<AppExit> {
    let flags: Vec<_> = args
        .iter()
        .enumerate()
        .filter(|(_, s)| *s == "--accessibility-check" || *s == "--verify-accessibility")
        .collect();
    if flags.is_empty() {
        return None;
    }
    let result = (|| -> Result<(), String> {
        if flags.len() != 1 {
            return Err("Choose one accessibility check".into());
        }
        let (index, flag) = flags[0];
        let directory = args
            .get(index + 1)
            .map(std::path::PathBuf::from)
            .filter(|p| p.is_absolute())
            .ok_or("An absolute check directory is required")?;
        let store = SaveStore::new(directory.clone());
        if flag == "--accessibility-check" {
            if directory.exists() {
                return Err("Writer requires a new directory".into());
            }
            let mut settings = Settings {
                high_contrast: true,
                reduced_motion: true,
                captions: false,
                guided_start: false,
                ..default()
            };
            settings.cycle(4);
            settings.cycle(15);
            settings.write(&store)?;
            let park = crate::new_game_park();
            store.save(&park).map_err(|e| e.to_string())?;
            std::fs::write(
                directory.join("expected-park.json"),
                park.to_save_bytes().map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        } else {
            let settings = Settings::read(&store)?;
            if !settings.high_contrast
                || !settings.reduced_motion
                || settings.captions
                || settings.guided_start
                || settings.bindings[4] == 4
                || settings.bindings[15] == 15
            {
                return Err("Preferences did not survive restart".into());
            }
            let mut raw = ButtonInput::default();
            raw.press(KEYS[usize::from(settings.bindings[4])]);
            if !translated(&raw, &settings).just_pressed(KeyCode::KeyP) {
                return Err("Restarted shortcut did not reach the path tool".into());
            }
            let bytes =
                std::fs::read(directory.join("expected-park.json")).map_err(|e| e.to_string())?;
            if store
                .load()
                .map_err(|e| e.to_string())?
                .to_save_bytes()
                .map_err(|e| e.to_string())?
                != bytes
            {
                return Err("Preferences changed the park".into());
            }
            std::fs::write(directory.join("verification-accessibility.txt"),"PASS: independent preferences persisted; remapped path input reaches its original action; park bytes preserved.\n").map_err(|e|e.to_string())?;
        }
        Ok(())
    })();
    Some(match result {
        Ok(()) => AppExit::Success,
        Err(error) => {
            eprintln!("Accessibility check failed: {error}");
            AppExit::error()
        }
    })
}

fn shortcut_labels(
    prefs: Res<Preferences>,
    buttons: Query<(&crate::ui::Action, &Children)>,
    mut texts: Query<&mut Text>,
) {
    use crate::{path_tools::PathTool, ui::Action};
    for (action, children) in &buttons {
        let index = match action {
            Action::Pause => 0,
            Action::RotateLeft => 1,
            Action::RotateRight => 2,
            Action::Grid => 3,
            Action::SetPathTool(PathTool::Build) => 4,
            Action::SetPathTool(PathTool::BuildQueue) => 5,
            Action::SetPathTool(PathTool::Entrance) => 6,
            Action::SetPathTool(PathTool::Exit) => 7,
            Action::SetPathTool(PathTool::RaiseTerrain) => 8,
            Action::SetPathTool(PathTool::LowerTerrain) => 9,
            Action::Guests => 10,
            Action::Services => 11,
            Action::Finance => 12,
            Action::Rides => 13,
            Action::EditTrack => 14,
            Action::Save => 15,
            Action::Load => 16,
            Action::FocusTrain => 17,
            Action::NavTarget(crate::ui::navigation::Destination::Path) => 4,
            Action::NavTarget(crate::ui::navigation::Destination::Queue) => 5,
            Action::NavTarget(crate::ui::navigation::Destination::Raise) => 8,
            Action::NavTarget(crate::ui::navigation::Destination::Lower) => 9,
            Action::NavTarget(crate::ui::navigation::Destination::Guests) => 10,
            Action::NavTarget(crate::ui::navigation::Destination::Services) => 11,
            Action::NavTarget(crate::ui::navigation::Destination::Finances) => 12,
            Action::NavTarget(crate::ui::navigation::Destination::Rides) => 13,
            Action::NavTarget(crate::ui::navigation::Destination::Track) => 14,
            Action::NavTarget(crate::ui::navigation::Destination::Save) => 15,
            _ => continue,
        };
        for child in children {
            if let Ok(mut text) = texts.get_mut(*child) {
                let base = text
                    .0
                    .rsplit_once(" [")
                    .map_or(text.0.as_str(), |(base, _)| base);
                let value = format!("{} [{}]", base, prefs.value.shortcut(index));
                if text.0 != value {
                    text.0 = value;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn directory() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "parkbound-access-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    #[test]
    fn keyboard_focus_stays_in_the_foreground_and_activates_once() {
        let mut app = App::new();
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Preferences::load(None))
            .insert_resource(Navigation::default())
            .insert_resource(ViewState {
                accessibility_open: true,
                ..default()
            })
            .add_systems(Update, prepare);
        let behind = app
            .world_mut()
            .spawn((
                Button,
                ComputedNode {
                    size: Vec2::splat(40.0),
                    ..default()
                },
                UiGlobalTransform::default(),
                Interaction::None,
                GlobalZIndex(1),
            ))
            .id();
        let front = app
            .world_mut()
            .spawn((
                Button,
                ComputedNode {
                    size: Vec2::splat(40.0),
                    ..default()
                },
                UiGlobalTransform::default(),
                Interaction::None,
                GlobalZIndex(40),
            ))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();
        assert_eq!(app.world().resource::<Navigation>().focus, Some(front));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();
        assert_eq!(
            *app.world().get::<Interaction>(front).unwrap(),
            Interaction::Pressed
        );
        assert_eq!(
            *app.world().get::<Interaction>(behind).unwrap(),
            Interaction::None
        );
        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::Enter)
        );
    }
    #[test]
    fn settings_round_trip_and_invalid_files_preserve_park() {
        let path = directory();
        let store = SaveStore::new(path.clone());
        let mut settings = Settings::default();
        for action in 0..NAMES.len() {
            settings.cycle(action);
        }
        settings.write(&store).unwrap();
        assert_eq!(Settings::read(&store).unwrap(), settings);
        let good = std::fs::read(path.join(FILE)).unwrap();
        for bad in [
            String::from_utf8(good.clone())
                .unwrap()
                .replace("\"version\": 1", "\"version\": 8"),
            "{}".into(),
            " ".repeat(8193),
        ] {
            std::fs::write(path.join(FILE), bad).unwrap();
            assert!(Settings::read(&store).is_err());
        }
        settings.write(&store).unwrap();
        let _lock = SaveLock::acquire(&path).unwrap();
        assert!(settings.write(&store).is_err());
        assert_eq!(std::fs::read(path.join(FILE)).unwrap(), good);
        drop(_lock);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn remapped_press_hold_release_reaches_real_keyboard_input() {
        let mut settings = Settings::default();
        settings.cycle(4);
        let replacement = KEYS[usize::from(settings.bindings[4])];
        let mut raw = ButtonInput::default();
        raw.press(replacement);
        let mapped = translated(&raw, &settings);
        assert!(mapped.just_pressed(KeyCode::KeyP));
        assert!(!mapped.pressed(replacement));
        let mut app = App::new();
        app.insert_resource(mapped)
            .insert_resource(Time::<()>::default())
            .insert_resource(ViewState::default())
            .add_systems(Update, crate::keyboard_input);
        app.update();
        assert_eq!(
            app.world().resource::<ViewState>().path_tool.mode,
            crate::path_tools::PathTool::Build
        );
        raw.clear();
        let held = translated(&raw, &settings);
        assert!(held.pressed(KeyCode::KeyP));
        assert!(!held.just_pressed(KeyCode::KeyP));
        raw.release(replacement);
        assert!(translated(&raw, &settings).just_released(KeyCode::KeyP));
    }
    #[test]
    fn duplicate_shortcuts_are_refused() {
        let path = directory();
        let store = SaveStore::new(path.clone());
        let mut s = Settings::default();
        s.bindings[0] = s.bindings[1];
        s.write(&store).unwrap();
        assert!(Settings::read(&store).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn foreground_settings_block_gameplay_keys() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyP);
        let mut app = App::new();
        app.insert_resource(keys)
            .insert_resource(Time::<()>::default())
            .insert_resource(ViewState {
                accessibility_open: true,
                ..default()
            })
            .add_systems(Update, crate::keyboard_input);
        app.update();
        assert_eq!(
            app.world().resource::<ViewState>().path_tool.mode,
            crate::path_tools::PathTool::Inspect
        );
    }
}
