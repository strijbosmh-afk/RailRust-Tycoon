//! Shared park-menu colors and readable button foregrounds.
use bevy::prelude::*;

pub(crate) const PANEL: Color = Color::srgb_u8(22, 57, 46);
pub(crate) const INSET: Color = Color::srgb_u8(15, 43, 35);
pub(crate) const HEADER: Color = Color::srgb_u8(10, 34, 28);
pub(crate) const TEXT: Color = Color::srgb_u8(255, 244, 212);
pub(crate) const MUTED: Color = Color::srgb_u8(184, 205, 184);
pub(crate) const BORDER: Color = Color::srgb_u8(143, 122, 67);
pub(crate) const ACCENT: Color = Color::srgb_u8(234, 183, 64);
pub(crate) const BUTTON: Color = Color::srgb_u8(33, 77, 59);
pub(crate) const HOVER: Color = Color::srgb_u8(49, 101, 75);
pub(crate) const DISABLED: Color = Color::srgb_u8(39, 57, 48);
pub(crate) const DARK_TEXT: Color = Color::srgb_u8(20, 46, 35);
pub(crate) const DANGER: Color = Color::srgb_u8(235, 142, 120);
pub(crate) const WARNING: Color = Color::srgb_u8(244, 196, 99);
pub(crate) const SUCCESS: Color = Color::srgb_u8(129, 202, 151);

#[derive(Component)]
pub(crate) struct StandardButton;

pub(crate) fn configure(app: &mut App) {
    app.add_systems(Update, button_feedback).add_systems(
        PostUpdate,
        foregrounds.before(crate::accessibility::AccessibilityStyle),
    );
}

fn button_feedback(
    view: Option<Res<crate::ViewState>>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<StandardButton>>,
) {
    for (interaction, mut color) in &mut buttons {
        let interaction = if view.as_ref().is_some_and(|view| view.capture_active) {
            Interaction::None
        } else {
            *interaction
        };
        let desired = match interaction {
            Interaction::Pressed => ACCENT,
            Interaction::Hovered => HOVER,
            Interaction::None => BUTTON,
        };
        if color.0 != desired {
            color.0 = desired;
        }
    }
}

pub(super) fn foreground(color: Color) -> Color {
    let rgba = color.to_srgba();
    let brightness = rgba.red * 0.2126 + rgba.green * 0.7152 + rgba.blue * 0.0722;
    if color == DISABLED {
        MUTED
    } else if brightness > 0.58 {
        DARK_TEXT
    } else {
        TEXT
    }
}

type ChangedButtons<'w, 's> = Query<
    'w,
    's,
    (&'static BackgroundColor, &'static Children),
    (With<Button>, Changed<BackgroundColor>),
>;

fn foregrounds(
    preferences: Option<Res<crate::accessibility::Preferences>>,
    buttons: ChangedButtons<'_, '_>,
    children: Query<&Children>,
    mut text: Query<&mut TextColor>,
) {
    if preferences.is_some_and(|prefs| prefs.value.high_contrast) {
        return;
    }
    for (background, descendants) in &buttons {
        let color = foreground(background.0);
        let mut pending: Vec<Entity> = descendants.iter().collect();
        while let Some(entity) = pending.pop() {
            if let Ok(mut label) = text.get_mut(entity)
                && label.0 != color
            {
                label.0 = color;
            }
            if let Ok(descendants) = children.get(entity) {
                pending.extend(descendants.iter());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_panels_and_gold_actions_keep_contrasting_text() {
        assert_eq!(foreground(PANEL), TEXT);
        assert_eq!(foreground(BUTTON), TEXT);
        assert_eq!(foreground(HOVER), TEXT);
        assert_eq!(foreground(ACCENT), DARK_TEXT);
        assert_eq!(foreground(DISABLED), MUTED);
        fn luminance(color: Color) -> f32 {
            let c = color.to_linear();
            c.red * 0.2126 + c.green * 0.7152 + c.blue * 0.0722
        }
        for background in [PANEL, BUTTON, HOVER, ACCENT, DISABLED] {
            let a = luminance(background);
            let b = luminance(foreground(background));
            assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
        }
    }

    #[test]
    fn button_foreground_tracks_nested_labels() {
        let mut app = App::new();
        app.add_systems(Update, foregrounds);
        let label = app.world_mut().spawn(TextColor(TEXT)).id();
        let group = app.world_mut().spawn_empty().add_child(label).id();
        let button = app
            .world_mut()
            .spawn((Button, BackgroundColor(ACCENT)))
            .add_child(group)
            .id();
        app.update();
        assert_eq!(app.world().get::<TextColor>(label).unwrap().0, DARK_TEXT);
        app.world_mut()
            .get_mut::<BackgroundColor>(button)
            .unwrap()
            .0 = HOVER;
        app.update();
        assert_eq!(app.world().get::<TextColor>(label).unwrap().0, TEXT);
    }
}
