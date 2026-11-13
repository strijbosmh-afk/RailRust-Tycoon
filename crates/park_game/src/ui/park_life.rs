//! Staff teams, local incidents and weather share one responsive management screen.
use super::*;
use crate::world;
use park_sim::{ObjectId, StaffRole};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Command {
    Close,
    Enable,
    Weather,
    Hire(StaffRole),
    Select(usize),
    Dismiss,
    Patrol,
    ClearPatrol,
    Shelter,
    FindStaff,
    FindIncident,
    Legacy,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Pick {
    Shelter,
    Patrol(u64),
}
#[derive(Component)]
pub(crate) struct Panel;
#[derive(Component)]
pub(crate) struct Banner;
#[derive(Component)]
pub(crate) struct Summary;
#[derive(Component)]
pub(crate) struct Details;
#[derive(Component)]
pub(crate) struct StaffRow(usize);
type SummaryText<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<Summary>, Without<Details>, Without<StaffRow>)>;
type DetailText<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<Details>, Without<Summary>, Without<StaffRow>)>;

pub(crate) fn open(view: &mut ViewState) {
    view.set_modal(false, false, false, false);
    view.park_life_open = true;
    view.ui_pointer_consumed = true;
}
pub(crate) fn command(park: &mut park_sim::Park, view: &mut ViewState, action: Command) {
    if !view.park_life_open
        || view.front_modal
        || view.capture_active
        || view.diagnostic_case.is_some()
    {
        return;
    }
    view.ui_pointer_consumed = true;
    match action {
        Command::Close => view.set_modal(false, false, false, false),
        Command::Enable => {
            park.enable_park_life();
            view.park_life_notice = "Park life enabled. Hire staff and provide shelter.".into();
        }
        Command::Weather => {
            park.set_weather_policy(park.park_life_status().policy.next());
            view.park_life_notice = "Weather policy updated.".into();
        }
        Command::Hire(role) => {
            view.park_life_notice = match park.hire_park_staff(role) {
                Ok(id) => {
                    view.selected_park_staff = Some(id);
                    format!(
                        "{} hired. Wages $1 every 30 simulation seconds.",
                        role.label()
                    )
                }
                Err(e) => e,
            };
        }
        Command::Select(index) => {
            view.selected_park_staff = park.park_staff().get(index).map(|s| s.id)
        }
        Command::Dismiss => {
            if let Some(id) = view.selected_park_staff.take() {
                park.dismiss_park_staff(id);
                view.park_life_notice =
                    "Staff member dismissed. Paid wages remain in Finance.".into();
            }
        }
        Command::ClearPatrol => {
            if let Some(id) = view.selected_park_staff {
                park.clear_staff_patrol(id);
                view.park_life_notice = "This staff member covers the whole connected park.".into();
            }
        }
        Command::Patrol => {
            if let Some(id) = view.selected_park_staff {
                view.set_modal(false, false, false, false);
                view.park_life_pick = Some(Pick::Patrol(id));
                view.park_life_notice =
                    "Click a public path to add or remove a patrol tile. Esc cancels.".into();
            }
        }
        Command::Shelter => {
            view.set_modal(false, false, false, false);
            view.park_life_pick = Some(Pick::Shelter);
            view.park_life_notice =
                "Click a public path to toggle a covered bench. New bench costs $1. Esc cancels."
                    .into();
        }
        Command::FindStaff => {
            if let Some(path) = view
                .selected_park_staff
                .and_then(|id| park.park_staff().iter().find(|s| s.id == id))
                .and_then(|s| s.current)
            {
                focus(park, view, path);
            }
        }
        Command::FindIncident => {
            if let Some(incident) = park.park_incidents().first() {
                focus(park, view, incident.path);
            }
        }
        Command::Legacy => {
            view.set_modal(false, true, false, false);
            view.guest_stats_view = true;
        }
    }
    view.scene_dirty = true;
}
fn focus(park: &park_sim::Park, view: &mut ViewState, path: ObjectId) {
    if let Some(p) = park.object(path) {
        view.set_modal(false, false, false, false);
        view.pan = world::tile_center(p.pos, park.width(), view.rotation)
            + Vec2::Y * f32::from(p.height) * world::HEIGHT_STEP;
        view.selected = Some(SurfaceId::Object(path));
        view.zoom = view.zoom.max(2.0);
    }
}
pub(crate) fn pick(park: &mut park_sim::Park, view: &mut ViewState) {
    let Some(action) = view.park_life_pick.take() else {
        return;
    };
    let result = match view.hovered {
        Some(SurfaceId::Object(path)) => match action {
            Pick::Shelter => park.toggle_path_shelter(path),
            Pick::Patrol(id) => park.toggle_staff_patrol(id, path),
        },
        _ => Err("Choose a public path tile".into()),
    };
    open(view);
    view.park_life_notice = match result {
        Ok(true) => "Path added.".into(),
        Ok(false) => "Path removed from this assignment.".into(),
        Err(e) => e,
    };
    view.scene_dirty = true;
}
fn control(p: &mut ChildSpawnerCommands, text: &str, command: Command, width: f32) {
    button(p, text, Action::ParkLife(command), width);
}
pub(crate) fn setup(commands: &mut Commands) {
    commands.spawn((
        Banner,
        Text::new(""),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(INK),
        BackgroundColor(PAPER),
        Node {
            position_type: PositionType::Absolute,
            left: px(SIDEBAR + 12.0),
            top: px(HEADER + 8.0),
            max_width: px(500.0),
            padding: UiRect::all(px(7)),
            ..default()
        },
        GlobalZIndex(4),
    ));
    commands
        .spawn((
            Panel,
            GlobalZIndex(12),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.09, 0.72)),
        ))
        .with_children(|p| {
            p.spawn((
                layout::ScrollLayer::ParkLife,
                ScrollPosition::default(),
                layout::dialog(94.0, 900.0, Some(580.0)),
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..row()
                })
                .with_children(|p| {
                    label(p, "Park life and staff", 24.0, INK);
                    control(p, "Close [Esc]", Command::Close, 130.0);
                });
                p.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(INK),
                    Summary,
                ));
                p.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(6),
                    row_gap: px(5),
                    ..row()
                })
                .with_children(|p| {
                    control(p, "Enable park life", Command::Enable, 150.0);
                    control(p, "Weather policy", Command::Weather, 150.0);
                    control(p, "Cover a path", Command::Shelter, 150.0);
                    control(p, "Find first incident", Command::FindIncident, 160.0);
                    control(p, "Grounds controls", Command::Legacy, 150.0);
                });
                label(
                    p,
                    "Hire staff / $1 every 30 simulation seconds / maximum 20",
                    13.0,
                    INK,
                );
                p.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(6),
                    row_gap: px(5),
                    ..row()
                })
                .with_children(|p| {
                    for role in StaffRole::ALL {
                        control(p, role.label(), Command::Hire(role), 155.0);
                    }
                });
                p.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(6),
                    row_gap: px(5),
                    ..row()
                })
                .with_children(|p| {
                    control(p, "Find selected", Command::FindStaff, 140.0);
                    control(p, "Patrol tile", Command::Patrol, 130.0);
                    control(p, "Whole park", Command::ClearPatrol, 130.0);
                    control(p, "Dismiss selected", Command::Dismiss, 150.0);
                });
                p.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(MUTED),
                    Details,
                ));
                for index in 0..park_sim::PARK_STAFF_LIMIT {
                    p.spawn((
                        Button,
                        Action::ParkLife(Command::Select(index)),
                        StaffRow(index),
                        Node {
                            display: Display::None,
                            min_height: px(30),
                            flex_shrink: 0.0,
                            padding: UiRect::all(px(5)),
                            ..default()
                        },
                        BackgroundColor(BUTTON),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(INK),
                            StaffRow(index),
                        ));
                    });
                }
            });
        });
}

pub(crate) fn refresh_banner(
    park: Res<Simulation>,
    view: Res<ViewState>,
    mut query: Query<(&mut Text, &mut Node), With<Banner>>,
) {
    for (mut text, mut node) in &mut query {
        let status = park.0.park_life_status();
        node.display = if view.park_life_pick.is_some()
            || (status.enabled && !view.front_modal && !view.park_life_open)
        {
            Display::Flex
        } else {
            Display::None
        };
        text.0 = if view.park_life_pick.is_some() {
            view.park_life_notice.clone()
        } else {
            format!(
                "{} / {} groups / {} staff / {} incidents",
                status.weather.label(),
                status.parties,
                status.staff,
                status.incidents
            )
        };
    }
}
pub(crate) fn refresh(
    park: Res<Simulation>,
    view: Res<ViewState>,
    mut panel: Query<&mut Node, (With<Panel>, Without<StaffRow>)>,
    mut summary: SummaryText,
    mut details: DetailText,
    mut rows: Query<(&StaffRow, Option<&mut Node>, Option<&mut Text>)>,
) {
    for mut node in &mut panel {
        node.display = if view.park_life_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !view.park_life_open {
        return;
    }
    let status = park.0.park_life_status();
    for mut text in &mut summary {
        *text = Text::new(format!(
            "{} / {} / {}\n{} groups / {} returning visits / {} shelters\n{} staff / {} awaiting wages / {} incidents / {} resolved",
            if status.enabled {
                "Park life active"
            } else {
                "Legacy park — enable park life to begin"
            },
            status.weather.label(),
            status.policy.label(),
            status.parties,
            status.returning_arrivals,
            status.shelters,
            status.staff,
            status.unpaid_staff,
            status.incidents,
            status.resolved
        ));
    }
    for mut text in &mut details {
        *text = Text::new(format!(
            "{}\nSelect a staff row, then assign patrol tiles or find them in the park.\n{}",
            view.park_life_notice,
            park.0
                .park_incidents()
                .iter()
                .take(4)
                .map(|i| format!("{} at path #{}", i.kind.label(), i.path.0))
                .collect::<Vec<_>>()
                .join(" / ")
        ));
    }
    for (row, node, text) in &mut rows {
        let worker = park.0.park_staff().get(row.0);
        if let Some(mut node) = node {
            node.display = if worker.is_some() {
                Display::Flex
            } else {
                Display::None
            };
        }
        if let Some(mut text) = text {
            text.0 = worker.map_or_else(String::new, |s| {
                format!(
                    "{}#{} {} — {} / {} tasks / {} patrol tiles{}",
                    if view.selected_park_staff == Some(s.id) {
                        "> "
                    } else {
                        ""
                    },
                    s.id,
                    s.role.label(),
                    if s.working {
                        s.task.label()
                    } else {
                        "Awaiting wages"
                    },
                    s.completed,
                    s.patrol.len(),
                    if s.current.is_none() {
                        " / waiting for entrance path"
                    } else {
                        ""
                    }
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roster_commands_and_path_picking_use_the_paid_simulation_apis() {
        let mut park = park_sim::Park::new(16, 16);
        let path = park
            .place_path(park_sim::TilePos { x: 5, y: 5 }, 0)
            .unwrap();
        park.set_visitor_entry(path).unwrap();
        let mut view = ViewState::default();
        open(&mut view);
        assert!(view.edits_blocked());
        command(&mut park, &mut view, Command::Hire(StaffRole::Security));
        assert_eq!(park.park_staff().len(), 1);
        command(&mut park, &mut view, Command::Patrol);
        assert!(!view.park_life_open);
        view.hovered = Some(SurfaceId::Object(path));
        pick(&mut park, &mut view);
        assert_eq!(park.park_staff()[0].patrol, vec![path]);
        let cash = park.cash_cents();
        command(&mut park, &mut view, Command::Shelter);
        view.hovered = Some(SurfaceId::Object(path));
        pick(&mut park, &mut view);
        assert!(park.path_is_sheltered(path));
        assert_eq!(park.cash_cents(), cash - park_sim::BENCH_COST_CENTS);
        command(&mut park, &mut view, Command::Close);
        let before = park.clone();
        command(&mut park, &mut view, Command::Hire(StaffRole::Mechanic));
        assert_eq!(park, before);
    }
    #[test]
    fn live_roster_refresh_has_disjoint_queries_and_shows_current_staff() {
        let mut park = park_sim::Park::new(16, 16);
        park.hire_park_staff(StaffRole::Entertainer).unwrap();
        let mut view = ViewState::default();
        open(&mut view);
        let mut app = App::new();
        app.insert_resource(Simulation(park))
            .insert_resource(view)
            .add_systems(Startup, setup)
            .add_systems(Update, (refresh, refresh_banner).chain());
        app.update();
        let world = app.world_mut();
        let mut query = world.query::<(&StaffRow, &Text)>();
        assert!(
            query
                .iter(world)
                .any(|(_, text)| text.0.contains("Entertainer"))
        );
    }
}
