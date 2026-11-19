//! Compact icon buttons retain their original actions and live labels.
use super::{Action, pictograms::Glyph, theme};
use crate::{
    ViewState,
    accessibility::{AccessibilityStyle, KeyboardFocus},
};
use bevy::{a11y::AccessibilityNode, ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

#[derive(Resource)]
struct Artwork(Vec<Handle<Image>>);

#[derive(Component)]
struct IconButton {
    caption: String,
    glyph: Glyph,
    image: Entity,
}

#[derive(Component)]
struct IconImage;
#[derive(Component)]
struct Tooltip;
#[derive(Component)]
struct TooltipText;

type LabelText<'w, 's> = Query<'w, 's, &'static Text, Without<TooltipText>>;
type NewButtons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Children,
        &'static mut Node,
        Option<&'static Action>,
    ),
    (With<Button>, Without<IconButton>),
>;
type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        &'static Children,
        &'static BackgroundColor,
        &'static Interaction,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        Option<&'static KeyboardFocus>,
        Option<&'static crate::visual_style::MenuButton>,
        Option<&'static mut AccessibilityNode>,
        Option<&'static Action>,
        &'static mut IconButton,
    ),
>;

pub(crate) fn configure(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(PostUpdate, decorate.before(bevy::ui::UiSystems::Layout))
        .add_systems(PostUpdate, update.after(AccessibilityStyle));
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(Artwork(
        Glyph::ALL
            .iter()
            .map(|glyph| images.add(glyph.image()))
            .collect(),
    ));
    commands
        .spawn((
            Tooltip,
            GlobalZIndex(100),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                max_width: px(400),
                padding: UiRect::axes(px(12), px(8)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(theme::INSET),
            BorderColor::all(theme::ACCENT),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| {
            p.spawn((
                TooltipText,
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme::TEXT),
            ));
        });
}

fn caption(
    descendants: &Children,
    children: &Query<&Children>,
    labels: &LabelText<'_, '_>,
) -> String {
    let mut pending: Vec<_> = descendants.iter().rev().collect();
    let mut parts = Vec::new();
    while let Some(entity) = pending.pop() {
        if let Ok(text) = labels.get(entity)
            && !text.0.trim().is_empty()
        {
            parts.push(text.0.trim());
        }
        if let Ok(children) = children.get(entity) {
            pending.extend(children.iter().rev());
        }
    }
    parts.join(" · ")
}

fn describe(action: Option<&Action>, label: String) -> String {
    let description = match action {
        Some(Action::TicketPrice(delta)) => {
            if *delta < 0 {
                "Decrease ride ticket price"
            } else {
                "Increase ride ticket price"
            }
        }
        Some(Action::ServicePrice(delta)) => {
            if *delta < 0 {
                "Decrease service price"
            } else {
                "Increase service price"
            }
        }
        Some(Action::PathHeight(delta)) => {
            if *delta < 0 {
                "Lower path height"
            } else {
                "Raise path height"
            }
        }
        Some(Action::TerrainBrushRadius(delta)) => {
            if *delta < 0 {
                "Smaller terrain brush"
            } else {
                "Larger terrain brush"
            }
        }
        Some(Action::BrowseRide(delta)) => {
            if *delta < 0 {
                "Previous ride"
            } else {
                "Next ride"
            }
        }
        Some(Action::BrowseService(delta)) => {
            if *delta < 0 {
                "Previous service"
            } else {
                "Next service"
            }
        }
        Some(Action::GuestPage(delta)) => {
            if *delta < 0 {
                "Previous guests"
            } else {
                "Next guests"
            }
        }
        Some(Action::FinancePage(delta)) => {
            if *delta < 0 {
                "Previous finance page"
            } else {
                "Next finance page"
            }
        }
        Some(Action::Track(crate::track_tools::TrackCommand::Height(delta))) => {
            if *delta < 0 {
                "Lower next track piece"
            } else {
                "Raise next track piece"
            }
        }
        Some(Action::RotateLeft) => "Rotate camera left",
        Some(Action::RotateRight) => "Rotate camera right",
        Some(Action::ZoomIn) => "Zoom in",
        Some(Action::ZoomOut) => "Zoom out",
        Some(Action::RidePaint(super::appearance::Command::Step(_, delta))) => {
            if *delta < 0 {
                "Previous paint color"
            } else {
                "Next paint color"
            }
        }
        _ => return label,
    };
    format!("{description} · {label}")
}

fn decorate(
    mut commands: Commands,
    artwork: Res<Artwork>,
    mut buttons: NewButtons<'_, '_>,
    mut nodes: Query<&mut Node, Without<Button>>,
    children: Query<&Children>,
    labels: LabelText<'_, '_>,
) {
    for (entity, descendants, mut node, action) in &mut buttons {
        let caption = describe(action, caption(descendants, &children, &labels));
        if caption.is_empty() {
            continue;
        }
        let glyph = Glyph::for_button(action, &caption);
        for child in descendants {
            if let Ok(mut node) = nodes.get_mut(*child) {
                node.display = Display::None;
            }
        }
        node.width = px(40);
        node.min_width = px(40);
        node.max_width = px(40);
        node.height = px(36);
        node.min_height = px(36);
        node.max_height = px(36);
        node.flex_grow = 0.0;
        node.flex_shrink = 0.0;
        node.flex_basis = Val::Auto;
        node.padding = UiRect::ZERO;
        node.align_items = AlignItems::Center;
        node.justify_content = JustifyContent::Center;
        let image = commands
            .spawn((
                IconImage,
                ImageNode::new(artwork.0[glyph as usize].clone()),
                Node {
                    width: px(28),
                    height: px(28),
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .id();
        commands.entity(entity).add_child(image).insert(IconButton {
            caption,
            glyph,
            image,
        });
    }
}

#[derive(Clone)]
struct Tip {
    caption: String,
    center: Vec2,
    size: Vec2,
    inverse_scale: f32,
}

#[derive(SystemParam)]
struct TooltipUi<'w, 's> {
    preferences: Option<Res<'w, crate::accessibility::Preferences>>,
    nodes: Query<'w, 's, &'static mut Node, With<Tooltip>>,
    text: Query<'w, 's, &'static mut Text, With<TooltipText>>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}

fn update(
    view: Res<ViewState>,
    artwork: Res<Artwork>,
    mut buttons: Buttons<'_, '_>,
    mut images: Query<&mut ImageNode, With<IconImage>>,
    children: Query<&Children>,
    labels: LabelText<'_, '_>,
    mut tooltip: TooltipUi<'_, '_>,
) {
    let mut hovered = None;
    let mut focused = None;
    for (
        descendants,
        background,
        interaction,
        node,
        position,
        focus,
        menu,
        accessible,
        action,
        mut button,
    ) in &mut buttons
    {
        let current = describe(action, caption(descendants, &children, &labels));
        if !current.is_empty() && current != button.caption {
            button.caption = current;
            button.glyph = Glyph::for_button(action, &button.caption);
        }
        if let Some(mut accessible) = accessible
            && accessible.label() != Some(button.caption.as_str())
        {
            accessible.set_label(button.caption.clone());
        }
        if let Ok(mut image) = images.get_mut(button.image) {
            let desired = artwork.0[button.glyph as usize].clone();
            if image.image != desired {
                image.image = desired;
            }
            let color = if tooltip
                .preferences
                .as_ref()
                .is_some_and(|p| p.value.high_contrast)
            {
                let rgba = background.0.to_srgba();
                if rgba.red * 0.2126 + rgba.green * 0.7152 + rgba.blue * 0.0722 > 0.5 {
                    Color::BLACK
                } else {
                    Color::WHITE
                }
            } else {
                theme::foreground(background.0)
            };
            if image.color != color {
                image.color = color;
            }
        }
        if node.is_empty() {
            continue;
        }
        let tip = Tip {
            caption: button.caption.clone(),
            center: position.translation,
            size: node.size(),
            inverse_scale: node.inverse_scale_factor,
        };
        if *interaction == Interaction::Hovered {
            hovered = Some(tip);
        } else if focus.is_some() || menu.is_some_and(|menu| menu.focused) {
            focused = Some(tip);
        }
    }
    let tip = if view.capture_active {
        None
    } else {
        hovered.or(focused)
    };
    for mut node in &mut tooltip.nodes {
        node.display = if tip.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(tip) = &tip
            && let Ok(window) = tooltip.windows.single()
        {
            let viewport = Vec2::new(
                window.physical_width() as f32,
                window.physical_height() as f32,
            ) * tip.inverse_scale;
            let center = tip.center * tip.inverse_scale;
            let size = tip.size * tip.inverse_scale;
            let width = (tip.caption.chars().count() as f32 * 7.5 + 24.0)
                .clamp(80.0, 400.0)
                .min(viewport.x - 16.0);
            let height = (tip.caption.chars().count() as f32 * 7.5 / (width - 24.0))
                .ceil()
                .max(1.0)
                * 18.0
                + 16.0;
            node.width = px(width);
            node.left =
                px((center.x - width / 2.0).clamp(8.0, (viewport.x - width - 8.0).max(8.0)));
            let below = center.y + size.y / 2.0 + 6.0;
            node.top = px(if below + height > viewport.y - 8.0 {
                (center.y - size.y / 2.0 - height - 6.0).max(8.0)
            } else {
                below
            });
        }
    }
    if let Some(tip) = tip {
        for mut text in &mut tooltip.text {
            if text.0 != tip.caption {
                text.0 = tip.caption.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_button_keeps_its_live_label_and_action_after_conversion() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>()
            .insert_resource(ViewState::default())
            .add_systems(Startup, setup)
            .add_systems(Update, (decorate, update).chain());
        let text = app
            .world_mut()
            .spawn((Text::new("Path [P]"), Node::default()))
            .id();
        let button = app
            .world_mut()
            .spawn((
                Button,
                Node::default(),
                BackgroundColor(theme::BUTTON),
                Interaction::Hovered,
                ComputedNode {
                    size: Vec2::new(40., 36.),
                    ..default()
                },
                UiGlobalTransform::default(),
                Action::SetPathTool(crate::path_tools::PathTool::Build),
            ))
            .add_child(text)
            .id();
        app.update();
        assert_eq!(
            app.world().get::<IconButton>(button).unwrap().caption,
            "Path [P]"
        );
        assert_eq!(
            app.world().get::<IconButton>(button).unwrap().glyph,
            Glyph::Path
        );
        assert_eq!(
            app.world().get::<Node>(text).unwrap().display,
            Display::None
        );
        assert!(matches!(
            app.world().get::<Action>(button),
            Some(Action::SetPathTool(crate::path_tools::PathTool::Build))
        ));
        app.world_mut().get_mut::<Text>(text).unwrap().0 = "Queue [J]".into();
        app.update();
        assert_eq!(
            app.world().get::<IconButton>(button).unwrap().caption,
            "Queue [J]"
        );
        assert_eq!(app.world().get::<Text>(text).unwrap().0, "Queue [J]");
        assert_eq!(app.world().get::<Children>(button).unwrap().len(), 2);
        let tip = app
            .world_mut()
            .query_filtered::<Entity, With<Tooltip>>()
            .single(app.world())
            .unwrap();
        assert_eq!(app.world().get::<Node>(tip).unwrap().display, Display::Flex);
        app.world_mut()
            .entity_mut(button)
            .insert((Interaction::None, Outline::new(px(1), px(0), Color::BLACK)));
        app.update();
        assert_eq!(app.world().get::<Node>(tip).unwrap().display, Display::None);
        app.world_mut().entity_mut(button).insert(KeyboardFocus);
        app.update();
        assert_eq!(app.world().get::<Node>(tip).unwrap().display, Display::Flex);
        app.world_mut().resource_mut::<ViewState>().capture_active = true;
        app.update();
        assert_eq!(app.world().get::<Node>(tip).unwrap().display, Display::None);
    }
}
