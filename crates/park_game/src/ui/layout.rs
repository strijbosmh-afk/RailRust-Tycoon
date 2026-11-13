//! Shared scale-aware dialog and scroll behavior.
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use super::{PATH_PANEL_RIGHT, PATH_PANEL_TOP, PATH_PANEL_WIDTH};
use crate::{FOOTER, HEADER, RIDE_PANEL_MARGIN, SIDEBAR, ViewState};

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScrollLayer {
    Front,
    Help,
    Finance,
    Guests,
    Services,
    Rides,
    Designs,
    Catalogue,
    ParkLife,
    Management,
    Workshop,
    Navigation,
    Sidebar,
    Build,
}

pub(crate) fn dialog(width_percent: f32, max_width: f32, height: Option<f32>) -> Node {
    Node {
        width: percent(width_percent),
        max_width: px(max_width),
        height: height.map_or(Val::Auto, px),
        max_height: percent(92),
        padding: UiRect::all(px(20)),
        flex_direction: FlexDirection::Column,
        row_gap: px(5),
        align_items: AlignItems::Stretch,
        overflow: Overflow {
            x: OverflowAxis::Clip,
            y: OverflowAxis::Scroll,
        },
        ..default()
    }
}

pub(crate) fn scroll(
    mut wheel: MessageReader<MouseWheel>,
    view: Res<ViewState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    ui_scale: Option<Res<UiScale>>,
    mut panels: Query<(&ScrollLayer, &mut ScrollPosition)>,
) {
    let delta: f32 = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y * 28.0,
            MouseScrollUnit::Pixel => event.y,
        })
        .sum();
    if delta == 0.0 {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let scale = ui_scale.as_ref().map_or(1.0, |scale| scale.0);
    let layer = if view.front_modal {
        Some(ScrollLayer::Front)
    } else if view.navigation_open {
        Some(ScrollLayer::Navigation)
    } else if view.workshop_open {
        Some(ScrollLayer::Workshop)
    } else if view.management_open {
        Some(ScrollLayer::Management)
    } else if view.park_life_open {
        Some(ScrollLayer::ParkLife)
    } else if view.catalogue_open {
        Some(ScrollLayer::Catalogue)
    } else if view.designs_open {
        Some(ScrollLayer::Designs)
    } else if view.rides_open {
        Some(ScrollLayer::Rides)
    } else if view.guests_open {
        Some(ScrollLayer::Guests)
    } else if view.services_open {
        Some(ScrollLayer::Services)
    } else if view.help {
        Some(ScrollLayer::Help)
    } else if view.finance_open {
        Some(ScrollLayer::Finance)
    } else if cursor.x < SIDEBAR * scale
        && cursor.y > HEADER * scale
        && cursor.y < window.height() - FOOTER * scale
    {
        Some(ScrollLayer::Sidebar)
    } else if cursor.x > window.width() - (PATH_PANEL_RIGHT + PATH_PANEL_WIDTH) * scale
        && cursor.y > PATH_PANEL_TOP * scale
        && cursor.y < window.height() - (FOOTER + RIDE_PANEL_MARGIN) * scale
    {
        Some(ScrollLayer::Build)
    } else {
        None
    };
    if let Some(layer) = layer {
        for (kind, mut position) in &mut panels {
            if *kind == layer {
                position.0.y = (position.0.y - delta).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::window::WindowResolution;

    #[test]
    fn wheel_scrolls_the_open_design_dialog_instead_of_the_sidebar() {
        let mut app = App::new();
        app.insert_resource(ViewState {
            designs_open: true,
            ..default()
        })
        .add_message::<MouseWheel>()
        .add_systems(Update, scroll);
        let mut window = Window {
            resolution: WindowResolution::new(1024, 680),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(500.0, 400.0)));
        let window = app
            .world_mut()
            .spawn((window, bevy::window::PrimaryWindow))
            .id();
        let designs = app
            .world_mut()
            .spawn((ScrollLayer::Designs, ScrollPosition::default()))
            .id();
        let sidebar = app
            .world_mut()
            .spawn((ScrollLayer::Sidebar, ScrollPosition::default()))
            .id();
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -3.0,
            window,
        });
        app.update();
        assert!(app.world().get::<ScrollPosition>(designs).unwrap().0.y > 0.0);
        assert_eq!(app.world().get::<ScrollPosition>(sidebar).unwrap().0.y, 0.0);
    }
}
