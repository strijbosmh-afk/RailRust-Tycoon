//! A grouped route into the existing park tools and management panels.
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use super::{Action, BORDER, BUTTON, INK, MUTED, PAPER, button, label, layout, row};
use crate::{SaveAction, Simulation, ViewState, path_tools::PathTool};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Group {
    #[default]
    Build,
    Manage,
    Inspect,
    Park,
    Help,
}

impl Group {
    const ALL: [Self; 5] = [
        Self::Build,
        Self::Manage,
        Self::Inspect,
        Self::Park,
        Self::Help,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Manage => "Manage",
            Self::Inspect => "Inspect",
            Self::Park => "Park",
            Self::Help => "Help / Settings",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Destination {
    Path,
    Queue,
    Raise,
    Lower,
    Tree,
    Flowers,
    Rides,
    Services,
    Track,
    Designs,
    Finances,
    Staff,
    ParkLife,
    Management,
    Workshop,
    Marketing,
    Guests,
    Scenarios,
    Statistics,
    Save,
    ParkMenu,
    Settings,
    Help,
    Sound,
}

#[derive(Component)]
pub(crate) struct Panel;

#[derive(Component)]
pub(crate) struct Section(Group);

fn destination(parent: &mut ChildSpawnerCommands, name: &str, target: Destination) {
    button(parent, name, Action::NavTarget(target), 190.0);
}

pub(crate) fn setup(commands: &mut Commands) {
    commands
        .spawn((
            Panel,
            GlobalZIndex(11),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.09, 0.7)),
        ))
        .with_children(|p| {
            p.spawn((
                layout::ScrollLayer::Navigation,
                ScrollPosition::default(),
                layout::dialog(94.0, 690.0, None),
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..row()
                })
                .with_children(|p| {
                    label(p, "Explore the park", 23.0, INK);
                    button(p, "Back [F2/Esc]", Action::CloseNavigation, 150.0);
                });
                label(p, "Choose an area, then a tool or report.", 12.0, MUTED);
                p.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(6),
                    row_gap: px(6),
                    ..row()
                })
                .with_children(|p| {
                    for group in Group::ALL {
                        let caption = group.title();
                        p.spawn((
                            Button,
                            Action::NavGroup(group),
                            Node {
                                width: px(if group == Group::Help { 145 } else { 105 }),
                                height: px(32),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(px(1)),
                                ..default()
                            },
                            BackgroundColor(BUTTON),
                            BorderColor::all(BORDER),
                            BorderRadius::all(px(5)),
                        ))
                        .with_children(|p| label(p, caption, 12.0, INK));
                    }
                });
                for group in Group::ALL {
                    p.spawn((
                        Section(group),
                        Node {
                            display: if group == Group::Build {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            width: percent(100),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(8),
                            row_gap: px(8),
                            ..default()
                        },
                    ))
                    .with_children(|p| match group {
                        Group::Build => {
                            destination(p, "Paths [P]", Destination::Path);
                            destination(p, "Queue [J]", Destination::Queue);
                            destination(p, "Raise terrain [B]", Destination::Raise);
                            destination(p, "Lower terrain [V]", Destination::Lower);
                            destination(p, "Trees", Destination::Tree);
                            destination(p, "Flowers", Destination::Flowers);
                            destination(p, "Build rides [Y]", Destination::Rides);
                            destination(p, "Build services [M]", Destination::Services);
                            destination(p, "Coaster track [Z]", Destination::Track);
                            destination(p, "Track designs", Destination::Designs);
                        }
                        Group::Manage => {
                            destination(p, "Finances [N]", Destination::Finances);
                            destination(p, "Scenarios and editor", Destination::Workshop);
                            destination(p, "Management and economy", Destination::Management);
                            destination(p, "Staff and park life", Destination::ParkLife);
                            destination(p, "Grounds controls", Destination::Staff);
                            destination(p, "Visitor flow and ads", Destination::Marketing);
                            destination(p, "Ride operations", Destination::Rides);
                            destination(p, "Service operations", Destination::Services);
                        }
                        Group::Inspect => {
                            destination(p, "Guests [L]", Destination::Guests);
                            destination(p, "Rides [Y]", Destination::Rides);
                            destination(p, "Facilities [M]", Destination::Services);
                        }
                        Group::Park => {
                            destination(p, "Scenarios and goals", Destination::Scenarios);
                            destination(p, "Park statistics", Destination::Statistics);
                            destination(p, "Save park [F5]", Destination::Save);
                            destination(p, "Park menu", Destination::ParkMenu);
                        }
                        Group::Help => {
                            destination(p, "Controls [F1]", Destination::Help);
                            destination(p, "Sound [F10]", Destination::Sound);
                            destination(p, "Display settings", Destination::Settings);
                        }
                    });
                }
                label(
                    p,
                    "F2 opens this menu. 1–5 change groups; Esc returns to the park.",
                    12.0,
                    MUTED,
                );
            });
        });
}

pub(crate) fn refresh(
    view: Res<ViewState>,
    mut panel: Query<&mut Node, (With<Panel>, Without<Section>)>,
    mut sections: Query<(&Section, &mut Node), Without<Panel>>,
    mut buttons: Query<(&Action, &mut BackgroundColor), With<Button>>,
) {
    for mut node in &mut panel {
        node.display = if view.navigation_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (section, mut node) in &mut sections {
        node.display = if section.0 == view.navigation_group {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (action, mut color) in &mut buttons {
        if let Action::NavGroup(group) = action {
            color.0 = if *group == view.navigation_group {
                super::GOLD
            } else {
                BUTTON
            };
        }
    }
}

pub(crate) fn open(view: &mut ViewState) {
    view.set_modal(false, false, false, false);
    view.navigation_open = true;
    view.modal_transition_pending = true;
}

pub(crate) fn close(view: &mut ViewState) {
    view.navigation_open = false;
    view.modal_input_blocked = false;
    view.modal_transition_pending = true;
    view.ui_pointer_consumed = true;
}

pub(crate) fn select(view: &mut ViewState, sim: Option<&mut Simulation>, target: Destination) {
    close(view);
    match target {
        Destination::Path => view.set_path_tool(PathTool::Build),
        Destination::Queue => view.set_path_tool(PathTool::BuildQueue),
        Destination::Raise => view.set_path_tool(PathTool::RaiseTerrain),
        Destination::Lower => view.set_path_tool(PathTool::LowerTerrain),
        Destination::Tree => view.set_path_tool(PathTool::Scenery(park_sim::SceneryKind::Tree)),
        Destination::Flowers => {
            view.set_path_tool(PathTool::Scenery(park_sim::SceneryKind::FlowerBed))
        }
        Destination::Track | Destination::Designs => {
            if let Some(sim) = sim {
                crate::track_tools::begin(&mut sim.0, view);
                if target == Destination::Designs {
                    // begin closes the previous modal in this same frame.
                    view.modal_input_blocked = false;
                    crate::design_library::request_open(view);
                }
            }
        }
        Destination::Rides => view.set_rides_modal(true),
        Destination::Services => {
            view.selected_service = None;
            view.service_browse_pending = 0;
            view.service_notice = None;
            view.set_modal(false, false, true, false);
        }
        Destination::Finances => {
            view.finance_page = 0;
            view.finance_page_pending = 0;
            view.set_modal(false, false, false, true);
        }
        Destination::Workshop => super::scenario_workshop::open(view),
        Destination::Management => super::management::open(view),
        Destination::ParkLife => super::park_life::open(view),
        Destination::Guests
        | Destination::Staff
        | Destination::Marketing
        | Destination::Scenarios
        | Destination::Statistics => {
            view.set_modal(false, true, false, false);
            view.guest_flow_view = target == Destination::Marketing;
            view.scenario_view = target == Destination::Scenarios;
            view.guest_stats_view = matches!(target, Destination::Staff | Destination::Statistics);
        }
        Destination::Save => view.request_save(SaveAction::Save),
        Destination::ParkMenu => view.front_request_menu = true,
        Destination::Settings => view.front_request_settings = true,
        Destination::Help | Destination::Sound => {
            view.set_modal(true, false, false, false);
            view.audio_view = target == Destination::Sound;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_blocks_world_edits_and_restores_a_build_tool() {
        let mut view = ViewState::default();
        open(&mut view);
        assert!(view.navigation_open);
        assert!(view.edits_blocked());
        select(&mut view, None, Destination::Path);
        assert!(!view.navigation_open);
        assert_eq!(view.path_tool.mode, PathTool::Build);
        assert!(view.ui_pointer_consumed);
        assert!(view.modal_transition_pending);
    }

    #[test]
    fn navigation_opens_the_correct_existing_management_view() {
        let mut view = ViewState::default();
        open(&mut view);
        select(&mut view, None, Destination::Scenarios);
        assert!(view.guests_open && view.scenario_view);
        assert!(!view.guest_stats_view && !view.guest_flow_view);
        open(&mut view);
        select(&mut view, None, Destination::Finances);
        assert!(view.finance_open);
        assert!(!view.guests_open && !view.navigation_open);
    }

    #[test]
    fn navigation_save_only_requests_the_normal_save_pipeline() {
        let mut view = ViewState::default();
        open(&mut view);
        select(&mut view, None, Destination::Save);
        assert_eq!(view.save_pending, Some(SaveAction::Save));
        assert!(!view.navigation_open);
    }

    #[test]
    fn navigation_opens_the_existing_track_design_library() {
        let mut sim = Simulation(crate::new_game_park());
        let mut view = ViewState::default();
        open(&mut view);
        select(&mut view, Some(&mut sim), Destination::Designs);
        assert!(view.track_editor.active);
        assert!(view.designs_open);
        assert!(!view.navigation_open);
    }

    #[test]
    fn navigation_requests_settings_without_requesting_a_save() {
        let mut view = ViewState::default();
        open(&mut view);
        select(&mut view, None, Destination::Settings);
        assert!(view.front_request_settings);
        assert!(view.save_pending.is_none());
        assert!(!view.navigation_open);
    }
}
