//! A scalable menu pictogram with a hover/focus label.
use super::{Action, BORDER, BUTTON, HEADER, INK, label, theme};
use crate::ViewState;
use bevy::{a11y::AccessibilityNode, ecs::hierarchy::ChildSpawnerCommands, prelude::*};

#[derive(Component)]
struct MenuButton;
#[derive(Component)]
struct Tooltip;

pub(super) fn button(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Button,
            Action::Menu,
            MenuButton,
            Name::new("Park menu"),
            Node {
                width: px(42),
                height: px(32),
                border: UiRect::all(px(1)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
            BackgroundColor(BUTTON),
            BorderColor::all(BORDER),
            BorderRadius::all(px(6)),
        ))
        .with_children(|parent| {
            for _ in 0..3 {
                parent.spawn((
                    theme::IconStroke,
                    Node {
                        width: px(18),
                        height: px(2),
                        ..default()
                    },
                    BackgroundColor(INK),
                    BorderRadius::all(px(1)),
                ));
            }
        });
}

pub(crate) fn configure(app: &mut App) {
    app.add_systems(Startup, setup.after(super::setup))
        .add_systems(
            PostUpdate,
            update.after(crate::accessibility::AccessibilityStyle),
        );
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Tooltip,
            GlobalZIndex(6),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: px(HEADER + 6.0),
                right: px(24),
                padding: UiRect::axes(px(12), px(8)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(theme::INSET),
            BorderColor::all(theme::ACCENT),
            BorderRadius::all(px(6)),
        ))
        .with_children(|parent| label(parent, "Park menu", 14.0, INK));
}

fn update(
    view: Res<ViewState>,
    buttons: Query<(&Interaction, Option<&Outline>), With<MenuButton>>,
    mut tooltip: Query<&mut Node, With<Tooltip>>,
    mut accessible: Query<&mut AccessibilityNode, With<MenuButton>>,
) {
    for mut node in &mut accessible {
        if node.label() != Some("Park menu") {
            node.set_label("Park menu");
        }
    }
    let visible = !view.front_modal
        && buttons.iter().any(|(interaction, outline)| {
            *interaction == Interaction::Hovered || outline.is_some()
        });
    for mut node in &mut tooltip {
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hovering_and_keyboard_focus_show_the_menu_label_and_modals_hide_it() {
        let mut app = App::new();
        app.insert_resource(ViewState::default())
            .add_systems(Update, update);
        let button = app.world_mut().spawn((MenuButton, Interaction::None)).id();
        let tooltip = app.world_mut().spawn((Tooltip, Node::default())).id();
        app.update();
        assert_eq!(
            app.world().get::<Node>(tooltip).unwrap().display,
            Display::None
        );
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Hovered;
        app.update();
        assert_eq!(
            app.world().get::<Node>(tooltip).unwrap().display,
            Display::Flex
        );
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
        app.world_mut()
            .entity_mut(button)
            .insert(Outline::new(px(2), px(1), theme::ACCENT));
        app.update();
        assert_eq!(
            app.world().get::<Node>(tooltip).unwrap().display,
            Display::Flex
        );
        app.world_mut().resource_mut::<ViewState>().front_modal = true;
        app.update();
        assert_eq!(
            app.world().get::<Node>(tooltip).unwrap().display,
            Display::None
        );
    }
}
