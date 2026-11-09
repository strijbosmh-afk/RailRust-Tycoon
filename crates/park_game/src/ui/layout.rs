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
