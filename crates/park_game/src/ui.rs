use super::{FOOTER, HEADER, MAP_SIZE, SIDEBAR, Simulation, ViewState, world::ViewMode};
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use park_sim::{ObjectKind, Speed, SurfaceId, Terrain, TerrainProfile};

const INK: Color = Color::srgb_u8(30, 62, 53);
const MUTED: Color = Color::srgb_u8(91, 109, 95);
const PAPER: Color = Color::srgb_u8(248, 248, 237);
const BORDER: Color = Color::srgb_u8(211, 219, 205);
const GOLD: Color = Color::srgb_u8(245, 194, 89);
const BUTTON: Color = Color::srgb_u8(230, 235, 221);

#[derive(Component, Clone, Copy, PartialEq)]
pub(super) enum Action {
    Pause,
    Speed(Speed),
    RotateLeft,
    RotateRight,
    ZoomIn,
    ZoomOut,
    Home,
    Grid,
    Terrain,
    Underground,
    Help,
    CloseHelp,
}

#[derive(Component)]
pub(super) enum Label {
    Clock,
    SelectionTitle,
    SelectionDetails,
    ViewInfo,
    Status,
    Pause,
    Grid,
    Terrain,
    Underground,
}

#[derive(Component)]
pub(super) struct HelpPanel;

fn label(parent: &mut ChildSpawnerCommands, text: &str, size: f32, color: Color) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    ));
}

fn dynamic_label(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    size: f32,
    color: Color,
    kind: Label,
) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        kind,
    ));
}

fn button(parent: &mut ChildSpawnerCommands, text: &str, action: Action, width: f32) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: px(width),
                height: px(36),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(BUTTON),
            BorderColor::all(BORDER),
            BorderRadius::all(px(5)),
        ))
        .with_children(|p| match action {
            Action::Pause => dynamic_label(p, text, 13.0, INK, Label::Pause),
            Action::Grid => dynamic_label(p, text, 13.0, INK, Label::Grid),
            Action::Terrain => dynamic_label(p, text, 13.0, INK, Label::Terrain),
            Action::Underground => dynamic_label(p, text, 13.0, INK, Label::Underground),
            _ => label(p, text, 13.0, INK),
        });
}

fn row() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: px(7),
        align_items: AlignItems::Center,
        ..default()
    }
}

pub(super) fn setup(mut commands: Commands) {
    // Fixed chrome reserves an unobstructed map viewport; all controls have
    // keyboard equivalents, listed in the help panel.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: px(HEADER),
                padding: UiRect::axes(px(24), px(13)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(INK),
        ))
        .with_children(|p| {
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                ..default()
            })
            .with_children(|p| {
                label(p, "PARKBOUND", 25.0, PAPER);
                label(
                    p,
                    "A little land. Endless possibilities.",
                    11.0,
                    Color::srgb_u8(175, 199, 170),
                );
            });
            p.spawn(row()).with_children(|p| {
                dynamic_label(p, "00:00  /  1x", 13.0, PAPER, Label::Clock);
                button(p, "Pause", Action::Pause, 70.0);
                button(p, "1x", Action::Speed(Speed::Normal), 39.0);
                button(p, "2x", Action::Speed(Speed::Fast), 39.0);
                button(p, "4x", Action::Speed(Speed::Fastest), 39.0);
                button(p, "Help", Action::Help, 60.0);
            });
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(HEADER),
                bottom: px(FOOTER),
                width: px(SIDEBAR),
                padding: UiRect::all(px(18)),
                border: UiRect::right(px(1)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            BackgroundColor(PAPER),
            BorderColor::all(BORDER),
        ))
        .with_children(|p| {
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                ..default()
            })
            .with_children(|p| {
                label(p, "YOUR FIRST PARK", 11.0, MUTED);
                label(p, "Meadow Lake", 22.0, INK);
                label(p, "32 x 32 tiles / woodland", 11.0, MUTED);
            });
            p.spawn((
                Node {
                    height: px(1),
                    ..default()
                },
                BackgroundColor(BORDER),
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            })
            .with_children(|p| {
                label(p, "CAMERA", 11.0, MUTED);
                p.spawn(row()).with_children(|p| {
                    button(p, "< Q", Action::RotateLeft, 65.0);
                    button(p, "E >", Action::RotateRight, 65.0);
                    button(p, "Fit", Action::Home, 60.0);
                });
                p.spawn(row()).with_children(|p| {
                    button(p, "-", Action::ZoomOut, 43.0);
                    button(p, "+", Action::ZoomIn, 43.0);
                    button(p, "Grid: on", Action::Grid, 104.0);
                });
                button(p, "Terrain: hills [T]", Action::Terrain, 204.0);
                button(p, "Underground: off [U]", Action::Underground, 204.0);
                dynamic_label(p, "View NE / Zoom 100%", 11.0, MUTED, Label::ViewInfo);
            });
            p.spawn((
                Node {
                    height: px(1),
                    ..default()
                },
                BackgroundColor(BORDER),
            ));
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                min_height: px(104),
                ..default()
            })
            .with_children(|p| {
                label(p, "PARK INSPECTOR", 11.0, MUTED);
                dynamic_label(p, "Pick a surface", 18.0, INK, Label::SelectionTitle);
                dynamic_label(
                    p,
                    "Click land, a path or track\nto inspect its height.",
                    12.0,
                    MUTED,
                    Label::SelectionDetails,
                );
            });
            p.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            p.spawn((
                Node {
                    padding: UiRect::all(px(10)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(234, 237, 221)),
                BorderRadius::all(px(5)),
            ))
            .with_children(|p| {
                label(
                    p,
                    "U reveals the buried track.\nT compares hills and flat land.",
                    11.0,
                    MUTED,
                );
            });
            label(
                p,
                concat!("v", env!("CARGO_PKG_VERSION"), " / layered park study"),
                10.0,
                MUTED,
            );
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(0),
                width: percent(100),
                height: px(FOOTER),
                padding: UiRect::axes(px(20), px(0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::top(px(1)),
                ..default()
            },
            BackgroundColor(PAPER),
            BorderColor::all(BORDER),
        ))
        .with_children(|p| {
            dynamic_label(p, "Ready to explore", 11.0, INK, Label::Status);
            label(
                p,
                "Q/E: rotate   T: terrain   U: underground   H: help",
                11.0,
                MUTED,
            );
        });

    commands
        .spawn((
            HelpPanel,
            GlobalZIndex(10),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.09, 0.65)),
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    width: px(530),
                    padding: UiRect::all(px(32)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(14),
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                label(p, "Make yourself at home", 23.0, INK);
                label(p, "Explore Meadow Lake with the mouse or keyboard.", 12.0, MUTED);
                for (keys, purpose) in [
                    ("Left click", "Inspect land, path or track"),
                    ("Right / middle drag", "Pan around the park"),
                    ("WASD / arrow keys", "Pan with the keyboard"),
                    ("Mouse wheel / + / -", "Zoom in and out"),
                    ("Q / E", "Rotate the view"),
                    ("Home", "Fit the park in view"),
                    ("G", "Show or hide the grid"),
                    ("T", "Switch flat / terraced terrain"),
                    ("U", "Show / hide underground view"),
                    ("Space / 1 / 2 / 3", "Pause / 1x / 2x / 4x time"),
                    ("Esc", "Close help or clear selection"),
                ] {
                    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..row() })
                        .with_children(|p| {
                            label(p, keys, 12.0, INK);
                            label(p, purpose, 11.0, MUTED);
                        });
                }
                label(p, "The bridge crosses above the hill and buried track.\nBuilding tools, guests, rides and saves arrive later.", 12.0, MUTED);
                button(p, "Back to the park", Action::CloseHelp, 180.0);
            });
        });
}

type ChangedButtons<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static Action), (Changed<Interaction>, With<Button>)>;

pub(super) fn button_actions(interactions: ChangedButtons<'_, '_>, mut view: ResMut<ViewState>) {
    // A button may close an overlay before pointer_input runs this frame.
    // Consume that press so it cannot also select the landscape beneath it.
    view.ui_pointer_consumed = false;
    if view.capture_active {
        return;
    }
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        view.ui_pointer_consumed = true;
        if view.help && !matches!(action, Action::CloseHelp | Action::Help) {
            continue;
        }
        match action {
            Action::Pause => view.pause(),
            Action::Speed(speed) => view.set_speed(*speed),
            Action::RotateLeft => view.rotate(3),
            Action::RotateRight => view.rotate(1),
            Action::ZoomIn => view.zoom_by(1.2),
            Action::ZoomOut => view.zoom_by(1.0 / 1.2),
            Action::Home => view.reset_camera(),
            Action::Grid => {
                view.grid = !view.grid;
                view.scene_dirty = true;
            }
            Action::Terrain => view.toggle_terrain_profile(),
            Action::Underground => view.toggle_underground(),
            Action::Help => view.help = !view.help,
            Action::CloseHelp => view.help = false,
        }
    }
}

pub(super) fn refresh(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut labels: Query<(&Label, &mut Text)>,
    mut buttons: Query<(&Action, &Interaction, &mut BackgroundColor)>,
    mut help: Query<&mut Node, With<HelpPanel>>,
) {
    let terrain_name = |terrain: Terrain| match terrain {
        Terrain::Grass => "Grassland",
        Terrain::Water => "Lake",
        Terrain::Sand => "Sandy shore",
    };
    let directions = ["NE", "SE", "SW", "NW"];
    for (kind, mut text) in &mut labels {
        let value = match kind {
            Label::Clock => {
                let seconds = sim.0.elapsed_seconds();
                let speed = if view.speed == Speed::Paused {
                    "Paused".into()
                } else {
                    format!("{}x", view.speed.multiplier())
                };
                format!("{:02}:{:02} / {speed}", seconds / 60, seconds % 60)
            }
            Label::SelectionTitle => match view.selected {
                Some(SurfaceId::Terrain(p)) if sim.0.tile(p).is_some() => {
                    format!("Tile {}, {}", p.x + 1, p.y + 1)
                }
                Some(SurfaceId::Object(id)) => {
                    sim.0.object(id).map_or("Pick a surface".into(), |object| {
                        format!("{} #{}", object_name(object.kind), id.0)
                    })
                }
                _ => "Pick a surface".into(),
            },
            Label::SelectionDetails => match view.selected {
                Some(SurfaceId::Terrain(p)) => sim.0.tile(p).map_or(String::new(), |tile| {
                    format!(
                        "{}\nHeight: {} steps\n{}",
                        terrain_name(tile.terrain),
                        tile.height,
                        if view.view_mode == ViewMode::Underground {
                            "Hidden in this view"
                        } else {
                            "Terrain surface"
                        }
                    )
                }),
                Some(SurfaceId::Object(id)) => sim.0.object(id).map_or(String::new(), |object| {
                    let buried = sim.0.is_underground(id);
                    let state = if buried && view.view_mode == ViewMode::Surface {
                        "Buried / U to reveal"
                    } else if buried {
                        "Underground"
                    } else {
                        "At / above ground"
                    };
                    format!(
                        "Tile {}, {}\nHeight: {} steps\nClearance: {} steps\n{state}",
                        object.pos.x + 1,
                        object.pos.y + 1,
                        object.height,
                        object.clearance,
                    )
                }),
                None => "Click land, a path or track\nto inspect its height.".into(),
            },
            Label::ViewInfo => format!(
                "View {} / Zoom {:.0}%",
                directions[view.rotation as usize],
                view.zoom * 100.0
            ),
            Label::Status if view.terrain_error.is_some() => view.terrain_error.clone().unwrap(),
            Label::Status => view.hovered.map_or(
                format!("Meadow Lake  /  {MAP_SIZE} x {MAP_SIZE} tiles"),
                |id| {
                    let Some(pos) = sim.0.surface_position(id) else {
                        return "Ready to explore".into();
                    };
                    let height = sim.0.surface_height(id).unwrap_or(0);
                    let name = match id {
                        SurfaceId::Terrain(_) => "Terrain",
                        SurfaceId::Object(id) => {
                            sim.0.object(id).map_or("Object", |o| object_name(o.kind))
                        }
                    };
                    format!("{name} {}, {} / Height {height}", pos.x + 1, pos.y + 1)
                },
            ),
            Label::Pause => if view.speed == Speed::Paused {
                "Resume"
            } else {
                "Pause"
            }
            .into(),
            Label::Grid => if view.grid { "Grid: on" } else { "Grid: off" }.into(),
            Label::Terrain => match view.terrain_profile {
                TerrainProfile::Flat => "Terrain: flat [T]",
                TerrainProfile::Terraced => "Terrain: hills [T]",
            }
            .into(),
            Label::Underground => match view.view_mode {
                ViewMode::Surface => "Underground: off [U]",
                ViewMode::Underground => "Underground: on [U]",
            }
            .into(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (action, interaction, mut background) in &mut buttons {
        let active = match action {
            Action::Pause => view.speed == Speed::Paused,
            Action::Speed(speed) => view.speed == *speed,
            Action::Grid => view.grid,
            Action::Terrain => view.terrain_profile == TerrainProfile::Terraced,
            Action::Underground => view.view_mode == ViewMode::Underground,
            _ => false,
        };
        let color = if (!view.capture_active && *interaction == Interaction::Pressed) || active {
            GOLD
        } else if !view.capture_active && *interaction == Interaction::Hovered {
            Color::srgb_u8(207, 220, 197)
        } else {
            BUTTON
        };
        if background.0 != color {
            background.0 = color;
        }
    }
    for mut node in &mut help {
        let display = if view.help {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
}

fn object_name(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Path => "Path",
        ObjectKind::Track => "Track",
    }
}
