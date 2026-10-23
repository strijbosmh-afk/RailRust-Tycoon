use super::{
    FOOTER, HEADER, RIDE_PANEL_HEIGHT, RIDE_PANEL_MARGIN, RIDE_PANEL_WIDTH, SIDEBAR, SaveAction,
    Simulation, ViewState, world::ViewMode,
};
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use park_sim::{
    CrowdStats, ObjectKind, PathDirection, PathSlope, QUEUE_PATIENCE_TICKS, RideAccessStatus,
    RideExitStatus, RideJourneyStats, RideQueueStats, Speed, SurfaceId, TICKS_PER_SECOND, Terrain,
    TerrainProfile, TrackKind, TrainPhase,
};

use crate::path_tools::PathTool;

mod economy;
pub(crate) mod finance;
pub(super) mod guests;
pub(crate) mod rides;
pub(crate) mod services;
pub(super) use economy::money;

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
    TicketPrice(i8),
    Guests,
    CloseGuests,
    SelectGuest(Option<park_sim::GuestId>),
    GuestPage(i8),
    Services,
    CloseServices,
    BuildService(park_sim::ServiceKind),
    BrowseService(i8),
    ServicePrice(i8),
    ToggleService,
    Finance,
    CloseFinance,
    FinancePage(i8),
    Rides,
    CloseRides,
    BrowseRide(i8),
    BuildCarousel,
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
    Economy,
    TicketPrice,
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
    RideRating,
    RideName,
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
                p.spawn(row()).with_children(|p| {
                    label(p, "PARKBOUND", 25.0, PAPER);
                    p.spawn((
                        Button,
                        Action::Finance,
                        Node {
                            width: px(100),
                            height: px(24),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(BUTTON),
                        BorderRadius::all(px(5)),
                    ))
                    .with_children(|p| label(p, "Finance [N]", 12.0, INK));
                    p.spawn((
                        Button,
                        Action::Rides,
                        Node {
                            width: px(86),
                            height: px(24),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(BUTTON),
                        BorderRadius::all(px(5)),
                    ))
                    .with_children(|p| label(p, "Rides [Y]", 12.0, INK));
                });
                dynamic_label(
                    p,
                    "Cash $1000.00 / Revenue $0.00 / Wallets $20.00",
                    11.0,
                    Color::srgb_u8(175, 199, 170),
                    Label::Economy,
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
                Button,
                Action::Guests,
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
            p.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                ..row()
            })
            .with_children(|p| {
                label(p, "BUILD", 16.0, INK);
                p.spawn((
                    Button,
                    Action::Services,
                    Node {
                        width: px(126),
                        height: px(22),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(BUTTON),
                    BorderColor::all(BORDER),
                    BorderRadius::all(px(5)),
                ))
                .with_children(|p| label(p, "Services [M]", 12.0, INK));
            });
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
                row_gap: px(4),
                ..default()
            },
            GlobalZIndex(2),
            BackgroundColor(PAPER),
            BorderColor::all(BORDER),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| {
            p.spawn(row()).with_children(|p| {
                dynamic_label(p, "Meadow Flyer", 16.0, INK, Label::RideName);
                dynamic_label(p, "Unrated", 11.0, MUTED, Label::RideRating);
            });
            dynamic_label(p, "Waiting at station", 11.0, MUTED, Label::Train);
            dynamic_label(
                p,
                "Queue: no entrance\n0/0 queued / longest 0s",
                11.0,
                INK,
                Label::RideAccess,
            );
            p.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            p.spawn(row()).with_children(|p| {
                button(p, "- F6", Action::TicketPrice(-1), 47.0);
                p.spawn(Node {
                    width: px(100),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|p| {
                    dynamic_label(
                        p,
                        "Ticket $0.00\n12 interested",
                        11.0,
                        INK,
                        Label::TicketPrice,
                    );
                });
                button(p, "+ F7", Action::TicketPrice(1), 47.0);
            });
            p.spawn(row()).with_children(|p| {
                button(p, "Close", Action::ToggleTrain, 47.0);
                button(p, "Find", Action::FocusTrain, 47.0);
                button(p, "Gate", Action::SetPathTool(PathTool::Entrance), 47.0);
                button(p, "Exit", Action::SetPathTool(PathTool::Exit), 47.0);
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
                "P/J: paths  I/K: gates  X: remove  L: guests  H: help",
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
                    &format!(
                        "Board at station; queues return after {}s waiting.",
                        u64::from(QUEUE_PATIENCE_TICKS) / TICKS_PER_SECOND
                    ),
                    12.0,
                    MUTED,
                );
                for (keys, purpose) in [
                    ("L / M / N / Y", "Guests / services / finance / rides"),
                    ("P / J / I / K / X", "Path / queue / entrance / exit / remove"),
                    ("C / shape button", "Cycle slope / turn gate or service"),
                    ("B / V", "Raise / lower dry land one step"),
                    ("PgUp / PgDn", "Build height / guest or finance page"),
                    ("Drag / WASD / arrows", "Pan around the park"),
                    ("Mouse wheel / + / -", "Zoom in and out"),
                    ("Q / E / Home", "Rotate / fit the park in view"),
                    ("G", "Show or hide the grid"),
                    ("T", "Switch flat / terraced terrain"),
                    ("U", "Show / hide underground view"),
                    ("Delete", "Remove selected path or building"),
                    ("R", "Restore the demonstration paths"),
                    ("O / F", "Open/close ride or service / find ride"),
                    ("F6 / F7", "Ticket / service price - / + $0.50"),
                    ("F5 / F9", "Save / load park"),
                    ("Shift + F9", "Load the previous valid save"),
                    ("Space / 1 / 2 / 3", "Pause / 1x / 2x / 4x time"),
                    ("Esc", "Close window / return to inspection"),
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
                    "Diamond: seeking; bars: queued; arrows: boarding or exiting.\nFare locks at boarding; charged only on departure.\nNew prices affect later boarders. Close before departure: no charge.",
                    12.0,
                    MUTED,
                );
                button(p, "Back to the park", Action::CloseHelp, 180.0);
            });
        });
    guests::setup(&mut commands);
    services::setup(&mut commands);
    finance::setup(&mut commands);
    rides::setup(&mut commands);
}

type ChangedButtons<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static Action), (Changed<Interaction>, With<Button>)>;

pub(super) fn button_actions(
    interactions: ChangedButtons<'_, '_>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    sim: Option<Res<Simulation>>,
    mut view: ResMut<ViewState>,
) {
    // A button may close an overlay before pointer_input runs this frame.
    // Consume that press so it cannot also select the landscape beneath it.
    view.ui_pointer_consumed = false;
    view.modal_input_blocked = false;
    if view.capture_active {
        return;
    }
    // Keyboard transitions run next in the update chain. Do not apply a UI
    // command first when that same frame is about to open or close a modal.
    if keys.as_ref().is_some_and(|keys| {
        keys.just_pressed(KeyCode::KeyL)
            || keys.just_pressed(KeyCode::KeyM)
            || keys.just_pressed(KeyCode::KeyN)
            || keys.just_pressed(KeyCode::KeyY)
            || keys.just_pressed(KeyCode::KeyH)
            || keys.just_pressed(KeyCode::F1)
            || ((view.help
                || view.guests_open
                || view.services_open
                || view.finance_open
                || view.rides_open)
                && keys.just_pressed(KeyCode::Escape))
    }) {
        view.ui_pointer_consumed = true;
        return;
    }
    let ride_transition = interactions
        .iter()
        .filter_map(|(interaction, action)| {
            if *interaction != Interaction::Pressed {
                return None;
            }
            match action {
                Action::CloseRides if view.rides_open => Some(0),
                Action::BuildCarousel if view.rides_open => Some(1),
                Action::Rides
                    if !view.help
                        && !view.guests_open
                        && !view.services_open
                        && !view.finance_open =>
                {
                    Some(2)
                }
                _ => None,
            }
        })
        .min();
    if let Some(transition) = ride_transition {
        if transition == 1 {
            view.choose_carousel();
        } else {
            view.set_rides_modal(transition == 2);
        }
        view.modal_transition_pending = true;
        return;
    }
    // Resolve modal transitions before any other pressed button, regardless of
    // entity iteration order. A single click cannot close a panel and edit below.
    let transition = interactions
        .iter()
        .filter_map(|(interaction, action)| {
            if *interaction != Interaction::Pressed {
                return None;
            }
            match action {
                Action::CloseHelp if view.help => Some((0, false, false, false, false, None)),
                Action::CloseGuests if view.guests_open => {
                    Some((0, false, false, false, false, None))
                }
                Action::CloseServices if view.services_open => {
                    Some((0, false, false, false, false, None))
                }
                Action::CloseFinance if view.finance_open => {
                    Some((0, false, false, false, false, None))
                }
                Action::BuildService(kind)
                    if view.services_open
                        && !view.help
                        && !view.guests_open
                        && !view.finance_open
                        && !view.rides_open
                        && !view.capture_active
                        && view.diagnostic_case.is_none()
                        && view
                            .save_pending
                            .is_none_or(|pending| pending == SaveAction::Save) =>
                {
                    Some((1, false, false, false, false, Some(*kind)))
                }
                Action::Help
                    if !view.guests_open
                        && !view.services_open
                        && !view.finance_open
                        && !view.rides_open =>
                {
                    Some((2, !view.help, false, false, false, None))
                }
                Action::Guests
                    if !view.help
                        && !view.guests_open
                        && !view.services_open
                        && !view.finance_open
                        && !view.rides_open =>
                {
                    Some((3, false, true, false, false, None))
                }
                Action::Services
                    if !view.help
                        && !view.guests_open
                        && !view.services_open
                        && !view.finance_open
                        && !view.rides_open =>
                {
                    Some((4, false, false, true, false, None))
                }
                Action::Finance
                    if !view.help
                        && !view.guests_open
                        && !view.services_open
                        && !view.finance_open
                        && !view.rides_open =>
                {
                    Some((5, false, false, false, true, None))
                }
                _ => None,
            }
        })
        .min_by_key(|transition| transition.0);
    if let Some((_, help, guests, facilities, finances, build)) = transition {
        if let Some(kind) = build {
            view.choose_service(kind);
        } else {
            if facilities {
                view.selected_service = None;
                view.service_browse_pending = 0;
                view.service_notice = None;
            }
            if finances {
                view.finance_page = 0;
                view.finance_page_pending = 0;
            }
            view.set_modal(help, guests, facilities, finances);
        }
        view.modal_transition_pending = true;
        return;
    }
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        view.ui_pointer_consumed = true;
        if view.rides_open && !matches!(action, Action::Pause | Action::BrowseRide(_)) {
            continue;
        }
        if view.help && !matches!(action, Action::CloseHelp | Action::Help) {
            continue;
        }
        if view.guests_open
            && !matches!(
                action,
                Action::Pause | Action::SelectGuest(_) | Action::GuestPage(_)
            )
        {
            continue;
        }
        if view.services_open
            && !matches!(
                action,
                Action::Pause
                    | Action::BrowseService(_)
                    | Action::ServicePrice(_)
                    | Action::ToggleService
            )
        {
            continue;
        }
        if view.finance_open && !matches!(action, Action::Pause | Action::FinancePage(_)) {
            continue;
        }
        if view.diagnostic_case.is_some()
            && matches!(
                action,
                Action::Terrain
                    | Action::RemovePath
                    | Action::RestorePaths
                    | Action::ToggleTrain
                    | Action::TicketPrice(_)
                    | Action::Save
                    | Action::Load
                    | Action::LoadBackup
                    | Action::SetPathTool(_)
                    | Action::PathHeight(_)
                    | Action::CyclePathShape
                    | Action::BuildService(_)
                    | Action::ServicePrice(_)
                    | Action::ToggleService
            )
        {
            continue;
        }
        match action {
            Action::Pause => view.pause(),
            Action::BrowseRide(delta) if view.rides_open => {
                view.ride_browse_pending =
                    view.ride_browse_pending.saturating_add(i32::from(*delta));
            }
            Action::Rides | Action::CloseRides | Action::BuildCarousel | Action::BrowseRide(_) => {}
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
            Action::TicketPrice(delta) => view.request_price_change(*delta),
            Action::FinancePage(delta) if view.finance_open => {
                view.finance_page_pending =
                    view.finance_page_pending.saturating_add(i32::from(*delta));
            }
            Action::SelectGuest(id) if view.guests_open && id.is_some() => {
                view.selected_guest = *id;
                view.guest_browse_pending = 0;
                view.guest_page_pending = 0;
            }
            Action::GuestPage(delta) if view.guests_open => {
                view.guest_page_pending = view.guest_page_pending.saturating_add(i32::from(*delta));
            }
            Action::BrowseService(delta) if view.services_open => {
                view.service_browse_pending = view
                    .service_browse_pending
                    .saturating_add(i32::from(*delta));
            }
            Action::ServicePrice(delta) if view.services_open => {
                if let Some(service) = view.selected_service {
                    view.service_commands
                        .push(services::ServiceCommand::PriceSteps {
                            service,
                            steps: i32::from(*delta),
                        });
                }
            }
            Action::ToggleService if view.services_open => {
                if let Some(service) = view
                    .selected_service
                    .and_then(|id| sim.as_ref().and_then(|sim| sim.0.service(id)))
                {
                    view.service_commands
                        .push(services::ServiceCommand::SetOpen {
                            service: service.id,
                            open: !service.open,
                        });
                }
            }
            Action::SelectGuest(_)
            | Action::GuestPage(_)
            | Action::Guests
            | Action::CloseGuests
            | Action::Services
            | Action::CloseServices
            | Action::BuildService(_)
            | Action::BrowseService(_)
            | Action::ServicePrice(_)
            | Action::ToggleService => {}
            Action::Finance | Action::CloseFinance | Action::FinancePage(_) => {}
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
    let ride_id = rides::selected_id(&sim.0, &view);
    let access = ride_id.map(|id| sim.0.ride_access(id));
    let queue = ride_id.map(|id| sim.0.ride_queue_stats(id));
    let journey = ride_id.map(|id| sim.0.ride_journey_stats(id));
    let economy = ride_id.map(|id| sim.0.ride_economy_stats(id));
    let experience = ride_id.map(|id| sim.0.ride_experience_stats(id));
    for (kind, mut text) in &mut labels {
        let value = match kind {
            Label::RideName=>ride_id.map_or("Choose ride [Y]".into(),|id|rides::name(&sim.0,id).into()),
            Label::Train if ride_id.and_then(|id|sim.0.carousel(id)).is_some()=> {
                let carousel=sim.0.carousel(ride_id.unwrap()).unwrap();
                let phase=if sim.0.ride_is_running(carousel.ride.id){if carousel.open{"Spinning"}else{"Finishing cycle"}}else if carousel.open{"Boarding / waiting"}else{"Closed"};
                let seats=journey.as_ref().map_or(0,|s|s.boarding+s.riding+s.exiting);
                format!("{phase}\nSeats {seats}/6 / Done {}",compact_count(journey.as_ref().map_or(0,|s|s.completed)))
            },
            Label::Economy => economy::header(&sim.0, economy.as_ref()),
            Label::TicketPrice => economy::ticket(economy.as_ref(), experience.as_ref()),
            Label::RideRating => guests::rating_label(
                experience
                    .as_ref()
                    .and_then(|stats| stats.average_rating_tenths),
            ),
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
                        if let Some(service)=sim.0.service_by_object(id) {
                            return format!("{} #{}",service.kind,service.id.0);
                        }
                        if object.kind==ObjectKind::Carousel {
                            return "Meadow Carousel".into();
                        }
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
                        if let Some(service)=sim.0.service_by_object(id) {
                            let stats=sim.0.service_stats(service.id);
                            let refund=sim.0.service_demolition_quote(service.id).map_or_else(|_|"Refund unavailable".into(),|amount|format!("Refund {}",money(amount)));
                            return format!("{} / {}\nFaces {} / height {}\nPaid {} / {refund}\nM: settings / X or Delete: remove",stats.status,money(u64::from(service.price_cents)),
                                direction_name(service.facing),object.height,money(service.construction_cost_cents));
                        }
                        let buried = sim.0.is_underground(id);
                        let state = if buried && view.view_mode == ViewMode::Surface {
                            "Buried / U to reveal"
                        } else if buried {
                            "Underground"
                        } else {
                            "At / above ground"
                        };
                        if let Some(owner) = object.queue_owner {
                            let owned_access=sim.0.ride_access(owner);
                            let status=access_reason(owned_access.status);
                            return format!(
                                "Tile {}, {} / Ride #{}\n{} / low {}\n{status}\n{}",
                                object.pos.x + 1,
                                object.pos.y + 1,
                                owner.0,
                                crate::path_tools::shape_name(object.path_slope),
                                object.height,
                                queue_details(&sim.0.ride_queue_stats(owner)),
                            );
                        }
                        if matches!(object.kind, ObjectKind::RideEntrance | ObjectKind::RideExit) {
                            return sim
                                .0
                                .object_ride(id).and_then(|owner|sim.0.ride_by_id(owner))
                                .and_then(|ride| {
                                    let exit = object.kind == ObjectKind::RideExit;
                                    let gate = if exit {
                                        ride.exit.as_ref()?
                                    } else {
                                        ride.entrance.as_ref()?
                                    };
                                    (gate.object == id).then(|| {
                                        let status = if exit {
                                            sim.0.ride_exit_access(ride.id).status.to_string()
                                        } else {
                                            access_reason(sim.0.ride_access(ride.id).status).into()
                                        };
                                        format!(
                                            "Tile {}, {} / Ride #{}\nHeight: {} / faces {}\n{}\n{}",
                                            object.pos.x + 1,
                                            object.pos.y + 1,
                                            ride.id.0,
                                            object.height,
                                            direction_name(gate.facing),
                                            status,
                                            queue_details(&sim.0.ride_queue_stats(ride.id)),
                                        )
                                    })
                                })
                                .unwrap_or_else(|| "Gate metadata unavailable".into());
                        }
                        if object.kind==ObjectKind::Carousel && let Some(owner)=sim.0.object_ride(id) {
                            let refund=sim.0.carousel_demolition_quote(owner).map_or_else(|_|"Removal blocked: passengers aboard".into(),|amount|format!("Demolition refund {}",money(amount)));
                            return format!("Ride #{} / height {} / six seats\n{refund}\nX / Delete: remove body and gates.\nOwned queue stays in place.",owner.0,object.height);
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
            Label::Status if view.price_notice.is_some() => view.price_notice.clone().unwrap(),
            Label::Status if view.path_tool.mode == PathTool::Build => format!(
                "Path tool / height {} / click to place",
                view.path_tool.height
            ),
            Label::Status if view.path_tool.mode == PathTool::BuildQueue => {
                "Build a blue queue. Details in Build Tools.".into()
            }
            Label::Status if view.path_tool.mode == PathTool::Entrance => {
                "Place a gate facing the selected ride. C turns it.".into()
            }
            Label::Status if view.path_tool.mode == PathTool::Exit => {
                "Face the selected ride; link exit to public path.".into()
            }
            Label::Status if view.path_tool.mode == PathTool::Demolish => {
                "Demolish / click a visible path, gate or facility".into()
            }
            Label::Status if matches!(view.path_tool.mode,PathTool::Service(_)) => {
                let PathTool::Service(kind)=view.path_tool.mode else {unreachable!()};
                format!("Build {kind} / {} / C turns serving side",money(kind.build_cost_cents()))
            }
            Label::Status if view.path_tool.mode==PathTool::Carousel=>format!("Build carousel / {} / clear 3 x 3 footprint",money(park_sim::CAROUSEL_BUILD_COST_CENTS)),
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
                // This card describes every guest in the park, whereas the
                // right-hand panel describes only the selected ride.
                let mut total_queue=RideQueueStats::default();
                let mut total_journey=sim.0.all_rides().next().map(|r|sim.0.ride_journey_stats(r.id));
                if let Some(journey)=&mut total_journey {journey.boarding=0;journey.riding=0;journey.exiting=0;}
                for visit in sim.0.guests().iter().filter_map(|guest|guest.ride_visit.as_ref()) {
                    match visit.phase {
                        park_sim::RideVisitPhase::Approaching=>total_queue.seeking+=1,
                        park_sim::RideVisitPhase::Queueing=>total_queue.queueing+=1,
                        park_sim::RideVisitPhase::Leaving=>total_queue.leaving+=1,
                        phase=>if let Some(journey)=&mut total_journey {match phase {
                            park_sim::RideVisitPhase::Boarding=>journey.boarding+=1,
                            park_sim::RideVisitPhase::Riding=>journey.riding+=1,
                            park_sim::RideVisitPhase::Exiting=>journey.exiting+=1,
                            _=>{},
                        }},
                    }
                }
                let text = crowd_summary(sim.0.crowd_stats(),total_journey.as_ref().map(|_|&total_queue),total_journey.as_ref());
                let (_, remainder) = text.split_once('\n').unwrap_or((&text, ""));
                format!(
                    "Guests [L]: {} / {} lost\n{remainder}",
                    sim.0.guests().len(),
                    sim.0.crowd_stats().lost
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
                    let seats = journey
                        .as_ref()
                        .map_or(0, |stats| stats.boarding + stats.riding + stats.exiting);
                    let capacity = journey
                        .as_ref()
                        .map_or(park_sim::TRAIN_SEATS, |stats| stats.capacity);
                    let completed =
                        compact_count(journey.as_ref().map_or(0, |stats| stats.completed));
                    format!("{operation} / {phase}\nSeats {seats}/{capacity} / Done {completed}")
                }),
            Label::RideAccess => match (access.as_ref(), queue.as_ref()) {
                (Some(access), Some(queue)) => format!(
                    "{}\n{}",
                    journey_status(access.status, queue, journey.as_ref()),
                    queue_details(queue)
                ),
                _ => "Queue: no ride\nBuild a ride first".into(),
            },
            Label::TrainToggle => if ride_id.is_some_and(|id|sim.0.ride_is_open(id)) {
                "Close"
            } else {
                "Open"
            }
            .into(),
            Label::SaveStatus => view.save_notice.clone(),
            Label::PathShape
                if matches!(view.path_tool.mode, PathTool::Entrance | PathTool::Exit | PathTool::Service(_)) =>
            {
                format!(
                    "Face {} [C]",
                    direction_name(if matches!(view.path_tool.mode,PathTool::Service(_)) {
                        view.path_tool.service_facing
                    } else if view.path_tool.mode == PathTool::Exit {
                        view.path_tool.exit_facing
                    } else {
                        view.path_tool.entrance_facing
                    }),
                )
            }
            Label::PathShape if view.path_tool.mode==PathTool::Carousel=>"Round / no facing".into(),
            Label::PathShape => format!(
                "{} [C]",
                crate::path_tools::shape_name(view.path_tool.slope)
            ),
            Label::PathHeight => {
                if view.path_tool.mode==PathTool::Carousel {format!("Carousel height: {}",view.path_tool.height)}
                else if matches!(view.path_tool.mode,PathTool::Service(_)) {
                    format!("Facility height: {}",view.path_tool.height)
                } else if view.path_tool.mode == PathTool::Exit {
                    format!("Exit height: {}", view.path_tool.height)
                } else if view.path_tool.mode == PathTool::Entrance {
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
            Action::ToggleTrain => ride_id.is_some_and(|id| !sim.0.ride_is_open(id)),
            Action::Rides => view.rides_open,
            Action::Services => view.services_open,
            Action::Finance => view.finance_open,
            Action::SetPathTool(mode) => view.path_tool.mode == *mode,
            _ => false,
        };
        let tool_disabled = matches!(
            action,
            Action::SetPathTool(_) | Action::PathHeight(_) | Action::CyclePathShape
        ) && (view.capture_active || view.diagnostic_case.is_some());
        let price_disabled = matches!(action, Action::TicketPrice(delta) if view.edits_blocked()
        || economy.as_ref().is_none_or(|stats| {
            (*delta < 0 && stats.ticket_price_cents == 0)
                || (*delta > 0 && stats.ticket_price_cents == park_sim::MAX_TICKET_PRICE_CENTS)
        }));
        let service_disabled=matches!(action,Action::BuildService(_) | Action::ServicePrice(_) | Action::ToggleService)
            && (view.capture_active || view.diagnostic_case.is_some()
                || (!matches!(action,Action::BuildService(_)) && view.selected_service.and_then(|id|sim.0.service(id)).is_none_or(|service| {
                    service.object.is_none() || matches!(action,Action::ServicePrice(delta) if (*delta<0 && service.price_cents==0)
                        || (*delta>0 && service.price_cents==park_sim::MAX_SERVICE_PRICE_CENTS))
                })));
        let color = if tool_disabled || price_disabled || service_disabled {
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
        ObjectKind::RideExit => "Ride exit",
        ObjectKind::Service => "Facility",
        ObjectKind::Carousel => "Carousel",
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

fn queue_status(access: RideAccessStatus, queue: &RideQueueStats) -> String {
    if access != RideAccessStatus::Ready {
        return format!("Queue: {}", access_reason(access));
    }
    if queue.draining {
        return if queue.stranded > 0 {
            "Returning / repair lost paths"
        } else {
            "Returning to public paths"
        }
        .into();
    }
    if queue.capacity > 0 && queue.queueing >= queue.capacity {
        "Queue full".into()
    } else if queue.admitting {
        "Queue open".into()
    } else {
        "Queue closed".into()
    }
}

fn queue_details(queue: &RideQueueStats) -> String {
    format!(
        "{}/{} queued / longest {}s",
        queue.queueing,
        queue.capacity,
        u64::from(queue.oldest_wait_ticks) / TICKS_PER_SECOND,
    )
}

fn journey_status(
    access: RideAccessStatus,
    queue: &RideQueueStats,
    journey: Option<&RideJourneyStats>,
) -> String {
    if access != RideAccessStatus::Ready || queue.draining {
        return queue_status(access, queue);
    }
    if let Some(journey) = journey {
        if journey.exiting > 0 {
            return format!("Leaving ride: {}", journey.exiting);
        }
        if journey.boarding > 0 {
            return format!("Boarding: {}", journey.boarding);
        }
        if journey.held {
            return "Ride held / check exit".into();
        }
        match journey.exit_status {
            RideExitStatus::Ready => {}
            RideExitStatus::NoExit => return "Boarding needs an exit [K]".into(),
            RideExitStatus::NoPublicExit | RideExitStatus::PublicDisconnected => {
                return "Link exit to public paths".into();
            }
            RideExitStatus::StationDisconnected => return "Exit must face the ride".into(),
            RideExitStatus::RideUnavailable => return "Ride unavailable".into(),
        }
    }
    queue_status(access, queue)
}

fn compact_count(count: u64) -> String {
    if count > 9999 {
        "9999+".into()
    } else {
        count.to_string()
    }
}

fn crowd_summary(
    crowd: CrowdStats,
    queue: Option<&RideQueueStats>,
    journey: Option<&RideJourneyStats>,
) -> String {
    let Some(queue) = queue else {
        return format!(
            "{} guests / {} lost\n{} walking / {} resting\nNo ride installed",
            crowd.total, crowd.lost, crowd.walking, crowd.waiting
        );
    };
    // Cumulative counts may grow indefinitely in a long-running save. Keep
    // this line inside the unchanged sidebar rather than wrapping below it.
    let movement = if let Some(journey) = journey {
        format!(
            "{} riding / {} board / {} exit",
            journey.riding, journey.boarding, journey.exiting
        )
    } else {
        format!(
            "Returning {} / cancelled {}",
            queue.leaving,
            compact_count(queue.abandoned)
        )
    };
    format!(
        "{} guests / {} lost\n{} seeking / {} queued\n{movement}",
        crowd.total, crowd.lost, queue.seeking, queue.queueing,
    )
}

fn access_reason(status: RideAccessStatus) -> &'static str {
    match status {
        RideAccessStatus::Ready => "Connected",
        RideAccessStatus::RideUnavailable => "ride unavailable",
        RideAccessStatus::NoEntrance => "place an entrance",
        RideAccessStatus::StationDisconnected => "gate must face the ride",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn queue_stats() -> RideQueueStats {
        RideQueueStats {
            seeking: 2,
            queueing: 3,
            leaving: 0,
            stranded: 0,
            capacity: 4,
            moving: 1,
            waiting: 2,
            oldest_wait_ticks: 1499,
            draining: false,
            admitting: true,
            joined: 3,
            turned_away: 0,
            abandoned: 0,
        }
    }

    #[test]
    fn boarding_feedback_explains_exit_and_transfer_states_without_growing_panels() {
        let queue = queue_stats();
        let mut journey = RideJourneyStats {
            capacity: 6,
            boarding: 0,
            riding: 0,
            exiting: 0,
            boarded: 0,
            exited: 0,
            completed: 0,
            held: false,
            exit_status: RideExitStatus::NoExit,
        };
        assert_eq!(
            journey_status(RideAccessStatus::Ready, &queue, Some(&journey)),
            "Boarding needs an exit [K]"
        );
        journey.exit_status = RideExitStatus::PublicDisconnected;
        assert_eq!(
            journey_status(RideAccessStatus::Ready, &queue, Some(&journey)),
            "Link exit to public paths"
        );
        journey.exit_status = RideExitStatus::Ready;
        journey.boarding = 1;
        journey.held = true;
        assert_eq!(
            journey_status(RideAccessStatus::Ready, &queue, Some(&journey)),
            "Boarding: 1"
        );
        journey.boarding = 0;
        journey.riding = 5;
        journey.exiting = 1;
        assert_eq!(
            journey_status(RideAccessStatus::Ready, &queue, Some(&journey)),
            "Leaving ride: 1"
        );
        let text = crowd_summary(
            CrowdStats {
                total: 12,
                ..default()
            },
            Some(&queue),
            Some(&journey),
        );
        assert_eq!(
            text,
            "12 guests / 0 lost\n2 seeking / 3 queued\n5 riding / 0 board / 1 exit"
        );
        assert_eq!(text.lines().count(), 3);
        assert!(text.lines().all(|line| line.len() <= 35));
        assert_eq!(compact_count(u64::MAX), "9999+");
    }

    #[test]
    fn queue_feedback_distinguishes_admission_capacity_and_topology() {
        let mut queue = queue_stats();
        assert_eq!(queue_status(RideAccessStatus::Ready, &queue), "Queue open");
        assert_eq!(queue_details(&queue), "3/4 queued / longest 49s");
        queue.queueing = 4;
        assert_eq!(queue_status(RideAccessStatus::Ready, &queue), "Queue full");
        queue.admitting = false;
        queue.queueing = 0;
        assert_eq!(
            queue_status(RideAccessStatus::Ready, &queue),
            "Queue closed"
        );
        assert_eq!(
            queue_status(RideAccessStatus::NoPublicEntry, &queue),
            "Queue: no public entry"
        );
    }

    #[test]
    fn draining_feedback_keeps_repair_reason_and_returning_counts_truthful() {
        let mut queue = queue_stats();
        queue.draining = true;
        queue.admitting = false;
        queue.queueing = 0;
        queue.leaving = 3;
        queue.abandoned = 3;
        assert_eq!(
            queue_status(RideAccessStatus::Ready, &queue),
            "Returning to public paths"
        );
        queue.stranded = 1;
        assert_eq!(
            queue_status(RideAccessStatus::Ready, &queue),
            "Returning / repair lost paths"
        );
        assert_eq!(
            queue_status(RideAccessStatus::Disconnected, &queue),
            "Queue: disconnected tiles"
        );
        let crowd = CrowdStats {
            total: 12,
            lost: 1,
            ..default()
        };
        assert_eq!(
            crowd_summary(crowd, Some(&queue), None),
            "12 guests / 1 lost\n2 seeking / 0 queued\nReturning 3 / cancelled 3"
        );
    }

    #[test]
    fn crowd_feedback_remains_three_short_lines_at_save_limits() {
        let mut queue = queue_stats();
        queue.seeking = 256;
        queue.queueing = 256;
        queue.capacity = 1024;
        queue.leaving = 256;
        queue.abandoned = u64::MAX;
        queue.oldest_wait_ticks = QUEUE_PATIENCE_TICKS;
        let text = crowd_summary(
            CrowdStats {
                total: 256,
                lost: 256,
                ..default()
            },
            Some(&queue),
            None,
        );
        assert_eq!(text.lines().count(), 3);
        assert!(text.lines().all(|line| line.len() <= 35));
        assert!(text.ends_with("cancelled 9999+"));
        assert_eq!(queue_details(&queue), "256/1024 queued / longest 60s");
    }
}
