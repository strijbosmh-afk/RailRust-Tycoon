use super::{
    FOOTER, HEADER, RIDE_PANEL_HEIGHT, RIDE_PANEL_MARGIN, RIDE_PANEL_WIDTH, SIDEBAR, SaveAction,
    Simulation, ViewState, world::ViewMode,
};
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use park_sim::{
    ObjectKind, PathDirection, PathSlope, RideAccessStatus, Speed, SurfaceId, Terrain,
    TerrainProfile, TrackKind, TrainPhase,
};

use crate::path_tools::PathTool;

pub(super) const PATH_PANEL_TOP: f32 = HEADER + RIDE_PANEL_HEIGHT + 2.0 * RIDE_PANEL_MARGIN;
pub(super) const PATH_PANEL_RIGHT: f32 = RIDE_PANEL_MARGIN;
pub(super) const PATH_PANEL_WIDTH: f32 = RIDE_PANEL_WIDTH;
pub(super) const PATH_PANEL_HEIGHT: f32 = 358.0;

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
    RemovePath,
    RestorePaths,
    ToggleTrain,
    FocusTrain,
    Save,
    Load,
    LoadBackup,
    SetPathTool(PathTool),
    PathHeight(i8),
    CyclePathShape,
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
    Crowd,
    Train,
    TrainToggle,
    RideAccess,
    SaveStatus,
    ParkInfo,
    PathHeight,
    PathShape,
    PathHint,
    PathResult,
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
                height: px(30),
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
            Action::ToggleTrain => dynamic_label(p, text, 13.0, INK, Label::TrainToggle),
            Action::CyclePathShape => dynamic_label(p, text, 13.0, INK, Label::PathShape),
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
                padding: UiRect::all(px(14)),
                border: UiRect::right(px(1)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
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
                label(p, "Meadow Lake", 22.0, INK);
                dynamic_label(p, "32 x 32 tiles / woodland", 11.0, MUTED, Label::ParkInfo);
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
            p.spawn(row()).with_children(|p| {
                button(p, "Remove [Del]", Action::RemovePath, 102.0);
                button(p, "Restore demo", Action::RestorePaths, 102.0);
            });
            p.spawn((
                Node {
                    padding: UiRect::all(px(8)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(234, 237, 221)),
                BorderRadius::all(px(5)),
            ))
            .with_children(|p| {
                dynamic_label(p, "12 guests / finding routes", 11.0, INK, Label::Crowd);
            });
            p.spawn(row()).with_children(|p| {
                button(p, "Save", Action::Save, 63.0);
                button(p, "Load", Action::Load, 63.0);
                button(p, "Backup", Action::LoadBackup, 70.0);
            });
            dynamic_label(
                p,
                "F5 save / F9 load\nShift+F9 loads backup",
                11.0,
                INK,
                Label::SaveStatus,
            );
            p.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            label(
                p,
                concat!("v", env!("CARGO_PKG_VERSION"), " / park memories"),
                10.0,
                MUTED,
            );
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(PATH_PANEL_TOP),
                right: px(PATH_PANEL_RIGHT),
                width: px(PATH_PANEL_WIDTH),
                height: px(PATH_PANEL_HEIGHT),
                padding: UiRect::all(px(12)),
                border: UiRect::all(px(1)),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
            GlobalZIndex(2),
            BackgroundColor(PAPER),
            BorderColor::all(BORDER),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| {
            label(p, "BUILD TOOLS", 16.0, INK);
            p.spawn(row()).with_children(|p| {
                button(p, "Inspect", Action::SetPathTool(PathTool::Inspect), 100.0);
                button(p, "Path [P]", Action::SetPathTool(PathTool::Build), 100.0);
            });
            p.spawn(row()).with_children(|p| {
                button(
                    p,
                    "Queue [J]",
                    Action::SetPathTool(PathTool::BuildQueue),
                    100.0,
                );
                button(
                    p,
                    "Demolish [X]",
                    Action::SetPathTool(PathTool::Demolish),
                    100.0,
                );
            });
            p.spawn(row()).with_children(|p| {
                button(
                    p,
                    "Raise [B]",
                    Action::SetPathTool(PathTool::RaiseTerrain),
                    100.0,
                );
                button(
                    p,
                    "Lower [V]",
                    Action::SetPathTool(PathTool::LowerTerrain),
                    100.0,
                );
            });
            button(p, "Level path [C]", Action::CyclePathShape, 207.0);
            dynamic_label(p, "Path height: 8", 11.0, INK, Label::PathHeight);
            p.spawn(row()).with_children(|p| {
                button(p, "- [PgDn]", Action::PathHeight(-1), 100.0);
                button(p, "+ [PgUp]", Action::PathHeight(1), 100.0);
            });
            for (initial, kind, color) in [
                ("Choose a build tool.", Label::PathHint, INK),
                ("Last: no build changes yet.", Label::PathResult, MUTED),
            ] {
                p.spawn(Node {
                    width: percent(100),
                    height: px(44),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|p| {
                    dynamic_label(p, initial, 11.0, color, kind);
                });
            }
            label(p, "Esc: return to inspection", 11.0, MUTED);
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(HEADER + RIDE_PANEL_MARGIN),
                right: px(RIDE_PANEL_MARGIN),
                width: px(RIDE_PANEL_WIDTH),
                height: px(RIDE_PANEL_HEIGHT),
                padding: UiRect::all(px(12)),
                border: UiRect::all(px(1)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            GlobalZIndex(2),
            BackgroundColor(PAPER),
            BorderColor::all(BORDER),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| {
            label(p, "Meadow Flyer", 18.0, INK);
            dynamic_label(p, "Waiting at station", 11.0, MUTED, Label::Train);
            dynamic_label(p, "Queue: no entrance", 11.0, INK, Label::RideAccess);
            p.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            p.spawn(row()).with_children(|p| {
                button(p, "Close", Action::ToggleTrain, 64.0);
                button(p, "Find [F]", Action::FocusTrain, 64.0);
                button(p, "Gate [I]", Action::SetPathTool(PathTool::Entrance), 64.0);
            });
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
                "P/J: paths  I: gate  X: demolish  B/V: land  H: help",
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
                    padding: UiRect::all(px(24)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                label(p, "Make yourself at home", 23.0, INK);
                label(
                    p,
                    "Explore Meadow Lake with the mouse or keyboard.",
                    12.0,
                    MUTED,
                );
                for (keys, purpose) in [
                    ("Left click", "Inspect / edit one tile"),
                    ("P / J / I / X", "Public path / queue / gate / demolish"),
                    ("C / shape button", "Cycle slope / gate faces station"),
                    ("B / V", "Raise / lower dry land one step"),
                    ("PgUp / PgDn", "Raise / lower path build height"),
                    ("Right / middle drag", "Pan around the park"),
                    ("WASD / arrow keys", "Pan with the keyboard"),
                    ("Mouse wheel / + / -", "Zoom in and out"),
                    ("Q / E", "Rotate the view"),
                    ("Home", "Fit the park in view"),
                    ("G", "Show or hide the grid"),
                    ("T", "Switch flat / terraced terrain"),
                    ("U", "Show / hide underground view"),
                    ("Delete", "Remove the selected path or gate"),
                    ("R", "Restore the demonstration paths"),
                    ("O / F", "Open or close ride / find circuit"),
                    ("F5 / F9", "Save / load park"),
                    ("Shift + F9", "Load the previous valid save"),
                    ("Space / 1 / 2 / 3", "Pause / 1x / 2x / 4x time"),
                    ("Esc", "Close help / return to inspection"),
                ] {
                    p.spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        ..row()
                    })
                    .with_children(|p| {
                        label(p, keys, 12.0, INK);
                        label(p, purpose, 11.0, MUTED);
                    });
                }
                label(
                    p,
                    "Load replaces the current park and starts paused.\nSave keeps the previous valid park as a backup.",
                    12.0,
                    MUTED,
                );
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
        if view.diagnostic_case.is_some()
            && matches!(
                action,
                Action::Terrain
                    | Action::RemovePath
                    | Action::RestorePaths
                    | Action::ToggleTrain
                    | Action::Save
                    | Action::Load
                    | Action::LoadBackup
                    | Action::SetPathTool(_)
                    | Action::PathHeight(_)
                    | Action::CyclePathShape
            )
        {
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
            Action::RemovePath => view.remove_path_pending = true,
            Action::RestorePaths => view.restore_paths_pending = true,
            Action::ToggleTrain => view.toggle_train_pending = true,
            Action::FocusTrain => view.focus_train_pending = true,
            Action::Save => view.request_save(SaveAction::Save),
            Action::Load => view.request_save(SaveAction::Load),
            Action::LoadBackup => view.request_save(SaveAction::Backup),
            Action::SetPathTool(mode) => view.set_path_tool(*mode),
            Action::PathHeight(delta) => view.adjust_path_height(*delta),
            Action::CyclePathShape => view.cycle_path_shape(),
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
    let access = sim.0.ride().map(|ride| sim.0.ride_access(ride.id));
    for (kind, mut text) in &mut labels {
        let value = match kind {
            Label::ParkInfo => format!(
                "{} x {} tiles / {}",
                sim.0.width(),
                sim.0.height(),
                if view.diagnostic_case.is_some() {
                    "render preview"
                } else {
                    "woodland"
                }
            ),
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
                        let name = if object.queue_owner.is_some() {
                            "Queue path"
                        } else {
                            object_name(object.kind)
                        };
                        format!("{name} #{}", id.0)
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
                Some(SurfaceId::Object(id)) => {
                    sim.0.object(id).map_or(String::new(), |object| {
                        let buried = sim.0.is_underground(id);
                        let state = if buried && view.view_mode == ViewMode::Surface {
                            "Buried / U to reveal"
                        } else if buried {
                            "Underground"
                        } else {
                            "At / above ground"
                        };
                        if let Some(owner) = object.queue_owner {
                            let status = access
                                .as_ref()
                                .map_or("Ride unavailable", |access| access_reason(access.status));
                            return format!(
                                "Tile {}, {} / Ride #{}\n{} / low {}\n{status}\nGuest admission comes later.",
                                object.pos.x + 1,
                                object.pos.y + 1,
                                owner.0,
                                crate::path_tools::shape_name(object.path_slope),
                                object.height,
                            );
                        }
                        if object.kind == ObjectKind::RideEntrance {
                            return sim.0.ride().and_then(|ride| {
                                let entrance = ride.entrance.as_ref()?;
                                (entrance.object == id).then(|| format!(
                                    "Tile {}, {} / Ride #{}\nHeight: {} / faces {}\n{}\nGuest admission comes later.",
                                    object.pos.x + 1,
                                    object.pos.y + 1,
                                    ride.id.0,
                                    object.height,
                                    direction_name(entrance.facing),
                                    access.as_ref().map_or("Ride unavailable", |access| access_reason(access.status)),
                                ))
                            }).unwrap_or_else(|| "Entrance metadata unavailable".into());
                        }
                        if let Some(piece) = sim.0.circuit().and_then(|circuit| {
                            circuit.pieces.iter().find(|piece| piece.object == id)
                        }) {
                            let low = piece.samples.iter().map(|p| p.height).min().unwrap_or(0);
                            let high = piece.samples.iter().map(|p| p.height).max().unwrap_or(0);
                            return format!(
                                "{} / Tile {}, {}\nRail height: {} - {} steps\n{state}",
                                track_name(piece.kind),
                                object.pos.x + 1,
                                object.pos.y + 1,
                                low / park_sim::TRACK_UNITS,
                                high / park_sim::TRACK_UNITS,
                            );
                        }
                        if object.kind == ObjectKind::Path && object.path_slope != PathSlope::Flat {
                            return format!(
                                "Tile {}, {}\n{} / height {} -> {}\nEnds connect; sides closed.",
                                object.pos.x + 1,
                                object.pos.y + 1,
                                crate::path_tools::shape_name(object.path_slope),
                                object.height,
                                object.height + 1
                            );
                        }
                        format!(
                            "Tile {}, {}\nHeight: {} steps\nClearance: {} steps\n{state}",
                            object.pos.x + 1,
                            object.pos.y + 1,
                            object.height,
                            object.clearance,
                        )
                    })
                }
                None => "Click land, a path, gate or track\nto inspect its height.".into(),
            },
            Label::ViewInfo => format!(
                "View {} / Zoom {:.0}%",
                directions[view.rotation as usize],
                view.zoom * 100.0
            ),
            Label::Status if view.terrain_error.is_some() => view.terrain_error.clone().unwrap(),
            Label::Status if view.path_tool.mode == PathTool::Build => format!(
                "Path tool / height {} / click to place",
                view.path_tool.height
            ),
            Label::Status if view.path_tool.mode == PathTool::BuildQueue => {
                "Build a blue queue. Details in Build Tools.".into()
            }
            Label::Status if view.path_tool.mode == PathTool::Entrance => {
                "Place a gate facing the station. C changes direction.".into()
            }
            Label::Status if view.path_tool.mode == PathTool::Demolish => {
                "Demolish tool / click a visible path or gate".into()
            }
            Label::Status if view.path_tool.mode == PathTool::RaiseTerrain => {
                "Raise terrain / click dry land / +1 step".into()
            }
            Label::Status if view.path_tool.mode == PathTool::LowerTerrain => {
                "Lower terrain / click dry land / -1 step".into()
            }
            Label::Status if view.path_notice.is_some() && view.hovered.is_none() => {
                "Edit result: see Build tools".into()
            }
            Label::Status => view.hovered.map_or(
                format!(
                    "{}  /  {} x {} tiles",
                    if view.diagnostic_case.is_some() {
                        "Render preview"
                    } else {
                        "Meadow Lake"
                    },
                    sim.0.width(),
                    sim.0.height()
                ),
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
            Label::Terrain if view.terrain_custom => "Terrain: custom [T]".into(),
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
            Label::Crowd => {
                let stats = sim.0.crowd_stats();
                format!(
                    "{} guests / {} walking\n{} waiting / {} lost\nDestinations reached: {}",
                    stats.total, stats.walking, stats.waiting, stats.lost, stats.arrivals
                )
            }
            Label::Train => sim
                .0
                .train()
                .map_or("No circuit installed".into(), |train| {
                    let phase = match train.phase {
                        TrainPhase::Station if train.speed > 0 => "Station track",
                        TrainPhase::Station => "Station wait",
                        TrainPhase::Lift => "Chain lift",
                        TrainPhase::Drop => "Descending",
                        TrainPhase::Brake => "Braking",
                        TrainPhase::Cruising => "Running",
                        TrainPhase::Closed => "Parked at station",
                        TrainPhase::Fault => "Track incomplete",
                    };
                    let operation = if train.phase == TrainPhase::Fault {
                        "Unavailable"
                    } else if train.open {
                        "Open"
                    } else if train.phase == TrainPhase::Closed {
                        "Closed"
                    } else {
                        "Closing after this lap"
                    };
                    let speed = train.speed as f32 * park_sim::TICKS_PER_SECOND as f32
                        / park_sim::TRACK_UNITS as f32;
                    format!(
                        "{operation} / {phase}\n{speed:.2} tiles/s / Laps: {} / 3 cars",
                        train.laps
                    )
                }),
            Label::RideAccess => access.as_ref().map_or("Queue: no ride".into(), |access| {
                if access.status == RideAccessStatus::Ready {
                    format!("{} queue tiles linked / no boarding", access.queue_tiles)
                } else {
                    format!("Queue: {}", access_reason(access.status))
                }
            }),
            Label::TrainToggle => if sim.0.train().is_some_and(|train| train.open) {
                "Close"
            } else {
                "Open"
            }
            .into(),
            Label::SaveStatus => view.save_notice.clone(),
            Label::PathShape if view.path_tool.mode == PathTool::Entrance => format!(
                "Face {} [C]",
                direction_name(view.path_tool.entrance_facing),
            ),
            Label::PathShape => format!(
                "{} [C]",
                crate::path_tools::shape_name(view.path_tool.slope)
            ),
            Label::PathHeight => {
                if view.path_tool.mode == PathTool::Entrance {
                    format!("Entrance height: {}", view.path_tool.height)
                } else if view.path_tool.slope == PathSlope::Flat {
                    format!("Path height: {}", view.path_tool.height)
                } else {
                    format!(
                        "Low end: {} / high end: {}",
                        view.path_tool.height,
                        view.path_tool.height + 1
                    )
                }
            }
            Label::PathHint if view.diagnostic_case.is_some() => {
                "Read-only render preview.\nBuild tools are disabled.".into()
            }
            Label::PathHint => view.path_tool.hint.clone(),
            Label::PathResult => format!(
                "Last: {}",
                view.path_notice
                    .as_deref()
                    .unwrap_or("no build changes yet.")
            ),
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
            Action::ToggleTrain => sim.0.train().is_some_and(|train| !train.open),
            Action::SetPathTool(mode) => view.path_tool.mode == *mode,
            _ => false,
        };
        let tool_disabled = matches!(
            action,
            Action::SetPathTool(_) | Action::PathHeight(_) | Action::CyclePathShape
        ) && (view.capture_active || view.diagnostic_case.is_some());
        let color = if tool_disabled {
            Color::srgb_u8(239, 240, 232)
        } else if (!view.capture_active && *interaction == Interaction::Pressed) || active {
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

fn track_name(kind: TrackKind) -> &'static str {
    match kind {
        TrackKind::Station => "Station",
        TrackKind::Lift => "Chain lift",
        TrackKind::Drop => "Drop",
        TrackKind::Brake => "Brakes",
        TrackKind::Curve => "Curve",
        TrackKind::Straight => "Straight",
    }
}

fn object_name(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Path => "Path",
        ObjectKind::Track => "Track",
        ObjectKind::RideEntrance => "Ride entrance",
    }
}

fn direction_name(direction: PathDirection) -> &'static str {
    match direction {
        PathDirection::North => "north",
        PathDirection::East => "east",
        PathDirection::South => "south",
        PathDirection::West => "west",
    }
}

fn access_reason(status: RideAccessStatus) -> &'static str {
    match status {
        RideAccessStatus::Ready => "Linked / no boarding yet",
        RideAccessStatus::RideUnavailable => "ride unavailable",
        RideAccessStatus::NoEntrance => "place an entrance",
        RideAccessStatus::StationDisconnected => "gate needs a station",
        RideAccessStatus::NoQueue => "connect blue paths",
        RideAccessStatus::Disconnected => "disconnected tiles",
        RideAccessStatus::Branch => "branch not allowed",
        RideAccessStatus::Loop => "loop not allowed",
        RideAccessStatus::NoPublicEntry => "no public entry",
        RideAccessStatus::MultiplePublicEntries => "multiple public entries",
        RideAccessStatus::PublicEntryNotAtEnd => "entry must be at the end",
        RideAccessStatus::ForeignQueue => "mixed ride owners",
    }
}
