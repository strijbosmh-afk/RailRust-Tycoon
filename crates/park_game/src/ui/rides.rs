//! Stable ride selection and the build entry point; no extra permanent panels.
use super::*;
use crate::access_planning::AccessCommand;
use crate::ride_operations::{OperatingField, OperationCommand};
use park_sim::{CAROUSEL_BUILD_COST_CENTS, CAROUSEL_REFUND_PERCENT, Park, RideId, RideKind};
use park_sim::{RideAccessBuildKind, RideAccessStatus, RideExitStatus};

pub(crate) const PANEL_WIDTH: f32 = 820.0;
pub(crate) const PANEL_HEIGHT: f32 = 540.0;

pub(crate) fn active_ids(park: &Park) -> Vec<RideId> {
    let mut ids: Vec<_> = park
        .all_rides()
        .filter(|ride| {
            park.carousel(ride.id)
                .is_none_or(|carousel| !carousel.objects.is_empty())
        })
        .map(|ride| ride.id)
        .collect();
    ids.sort_unstable();
    ids
}

pub(crate) fn selected_id(park: &Park, view: &ViewState) -> Option<RideId> {
    let ids = active_ids(park);
    match view.selected_ride {
        Some(id) => ids.contains(&id).then_some(id),
        None => ids.first().copied(),
    }
}

pub(crate) fn name(park: &Park, id: RideId) -> &'static str {
    match park.ride_kind(id) {
        Some(RideKind::Carousel) => "Meadow Carousel",
        _ => "Meadow Flyer",
    }
}

pub(crate) fn construction_details(park: &Park, id: RideId) -> String {
    park.carousel(id).map_or_else(
        || {
            let paid = park.track_economy().paid_pieces.iter().map(|piece| piece.paid_cents).sum();
            format!(
                "Live track paid {} / 50% back on removal\nZ: quote replacement / select rail + Delete: remove",
                money(paid)
            )
        },
        |carousel| match park.carousel_demolition_quote(id) {
            Ok(refund) => format!(
                "Paid {} / Demolition refund {}",
                money(carousel.construction_cost_cents),
                money(refund)
            ),
            Err(_) => format!(
                "Paid {} / Demolition blocked while occupied",
                money(carousel.construction_cost_cents)
            ),
        },
    )
}

#[derive(Component)]
pub(crate) struct RidesPanel;
#[derive(Component)]
pub(crate) enum RideLabel {
    Selected,
    Details,
    OperatingValue(OperatingField),
    OperatingStatus,
    OperatingNotice,
    ToggleOpen,
    AccessChecklist,
    AccessAdvice,
    AccessNotice,
    AccessGate(bool),
}

#[derive(Component)]
pub(crate) struct OperatingRow(pub(crate) OperatingField);

#[derive(Component)]
pub(crate) struct RideSection(pub(crate) bool);

#[derive(Component)]
pub(crate) struct AccessGate(pub(crate) bool);

fn gate_button(parent: &mut ChildSpawnerCommands, exit: bool) {
    parent
        .spawn((
            Button,
            AccessGate(exit),
            Action::RideAccess(AccessCommand::Prepare(if exit {
                RideAccessBuildKind::Exit
            } else {
                RideAccessBuildKind::Entrance
            })),
            Node {
                width: px(356),
                height: px(30),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(BUTTON),
            BorderColor::all(BORDER),
            BorderRadius::all(px(5)),
        ))
        .with_children(|p| operating_label(p, RideLabel::AccessGate(exit), 13.0, INK));
}

fn operating_label(parent: &mut ChildSpawnerCommands, kind: RideLabel, size: f32, color: Color) {
    parent.spawn((
        Text::new(""),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        kind,
    ));
}

fn operating_row(parent: &mut ChildSpawnerCommands, field: OperatingField) {
    parent
        .spawn((
            OperatingRow(field),
            Node {
                width: percent(100),
                height: px(44),
                flex_shrink: 0.0,
                ..row()
            },
        ))
        .with_children(|p| {
            button(
                p,
                "-",
                Action::RideOperating(OperationCommand::Adjust(field, -1)),
                32.0,
            );
            p.spawn(Node {
                width: px(278),
                flex_shrink: 1.0,
                ..default()
            })
            .with_children(|p| operating_label(p, RideLabel::OperatingValue(field), 12.0, INK));
            button(
                p,
                "+",
                Action::RideOperating(OperationCommand::Adjust(field, 1)),
                32.0,
            );
        });
}

pub(super) fn setup(commands: &mut Commands) {
    commands.spawn((RidesPanel, GlobalZIndex(9), Node {
        display: Display::None, position_type: PositionType::Absolute,
        width: percent(100), height: percent(100), align_items: AlignItems::Center,
        justify_content: JustifyContent::Center, ..default()
    }, BackgroundColor(Color::srgba(0.04,0.12,0.09,0.65))))
    .with_children(|p| { p.spawn((Node { width:px(PANEL_WIDTH),height:px(PANEL_HEIGHT),
        padding:UiRect::all(px(20)),flex_direction:FlexDirection::Column,row_gap:px(10),..default()
    },BackgroundColor(PAPER),BorderRadius::all(px(10))))
    .with_children(|p| {
        p.spawn(Node {justify_content:JustifyContent::SpaceBetween,..row()}).with_children(|p| {
            label(p,"Rides",23.0,INK);
            button(p,"Use ride [Y/Esc]",Action::CloseRides,150.0);
        });
        p.spawn(Node {justify_content:JustifyContent::SpaceBetween,..row()}).with_children(|p| {
            button(p,"< Up",Action::BrowseRide(-1),65.0);
            p.spawn((Text::new("No ride"),TextFont{font_size:17.0,..default()},TextColor(INK),RideLabel::Selected));
            button(p,"Down >",Action::BrowseRide(1),65.0);
        });
        p.spawn(Node { column_gap: px(16), align_items: AlignItems::Start, flex_grow: 1.0, ..row() }).with_children(|p| {
            p.spawn(Node { width: px(408), flex_shrink: 0.0, flex_direction: FlexDirection::Column, row_gap: px(10), ..default() }).with_children(|p| {
                p.spawn(Node{height:px(170),flex_shrink:0.0,..default()}).with_children(|p| {
                    operating_label(p, RideLabel::Details, 13.0, INK);
                });
                button(p,&format!("Build carousel / {}",money(CAROUSEL_BUILD_COST_CENTS)),Action::BuildCarousel,310.0);
                button(p,"Edit coaster track [Z]",Action::EditTrack,310.0);
                label(p,&format!("Six seats; 3 x 3 tiles. Add a queue and gates.\nDemolition returns {}% of the amount paid.\nOne carousel at a time. Set height in Build Tools.",CAROUSEL_REFUND_PERCENT),12.0,MUTED);
                label(p,"After closing this panel: O open/close / F find\nF6-F7 ticket / J queue / I entrance / K exit\nSpace pauses from this panel or the park.",12.0,MUTED);
            });
            p.spawn(Node { width: px(356), flex_shrink: 0.0, flex_direction: FlexDirection::Column, row_gap: px(6), ..default() }).with_children(|p| {
                p.spawn(row()).with_children(|p| {
                    button(p, "Operations", Action::RideAccess(AccessCommand::ShowOperations), 174.0);
                    button(p, "Access", Action::RideAccess(AccessCommand::ShowAccess), 175.0);
                });
                p.spawn((RideSection(false), Node { width: percent(100), flex_direction: FlexDirection::Column, row_gap: px(6), ..default() })).with_children(|p| {
                label(p, "Operating settings", 17.0, INK);
                p.spawn((Button, Action::RideOperating(OperationCommand::ToggleOpen), Node {
                    width: percent(100), height: px(30), flex_shrink: 0.0, align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center, border: UiRect::all(px(1)), ..default()
                }, BackgroundColor(BUTTON), BorderColor::all(BORDER), BorderRadius::all(px(5))))
                .with_children(|p| operating_label(p, RideLabel::ToggleOpen, 13.0, INK));
                p.spawn(Node { height: px(42), flex_shrink: 0.0, ..default() })
                    .with_children(|p| operating_label(p, RideLabel::OperatingStatus, 12.0, MUTED));
                operating_row(p, OperatingField::StationWait);
                operating_row(p, OperatingField::LiftSpeed);
                operating_row(p, OperatingField::BrakeSpeed);
                operating_row(p, OperatingField::CycleDuration);
                button(p, "Reset operating defaults", Action::RideOperating(OperationCommand::ResetDefaults), 356.0);
                p.spawn(Node { height: px(44), flex_shrink: 0.0, ..default() })
                    .with_children(|p| operating_label(p, RideLabel::OperatingNotice, 12.0, MUTED));
                });
                p.spawn((RideSection(true), Node { display: Display::None, width: percent(100), flex_direction: FlexDirection::Column, row_gap: px(6), ..default() })).with_children(|p| {
                    label(p, "Station and paths", 17.0, INK);
                    p.spawn(Node { height: px(105), flex_shrink: 0.0, ..default() })
                        .with_children(|p| operating_label(p, RideLabel::AccessChecklist, 13.0, INK));
                    gate_button(p, false);
                    gate_button(p, true);
                    button(p, "Prepare suggested queue tile", Action::RideAccess(AccessCommand::Prepare(RideAccessBuildKind::Queue)), 356.0);
                    button(p, "Prepare suggested public path", Action::RideAccess(AccessCommand::Prepare(RideAccessBuildKind::PublicPath)), 356.0);
                    p.spawn(Node { height: px(60), flex_shrink: 0.0, ..default() })
                        .with_children(|p| operating_label(p, RideLabel::AccessAdvice, 12.0, MUTED));
                    p.spawn(Node { height: px(36), flex_shrink: 0.0, ..default() })
                        .with_children(|p| operating_label(p, RideLabel::AccessNotice, 12.0, MUTED));
                });
            });
        });
    }); });
}

type OperatingRows<'w, 's> = Query<
    'w,
    's,
    (&'static OperatingRow, &'static mut Node),
    (Without<RidesPanel>, Without<RideSection>),
>;

type RideSections<'w, 's> = Query<
    'w,
    's,
    (&'static RideSection, &'static mut Node),
    (Without<RidesPanel>, Without<OperatingRow>),
>;

pub(crate) fn refresh(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    mut panels: Query<&mut Node, With<RidesPanel>>,
    mut operating_rows: OperatingRows<'_, '_>,
    mut sections: RideSections<'_, '_>,
    mut gates: Query<(&AccessGate, &mut Action)>,
    mut labels: Query<(&RideLabel, &mut Text)>,
) {
    for mut node in &mut panels {
        node.display = if view.rides_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    let ids = active_ids(&sim.0);
    let delta = std::mem::take(&mut view.ride_browse_pending);
    if ids.is_empty() {
        view.selected_ride = None;
        return;
    }
    let selected = selected_id(&sim.0, &view).unwrap_or(ids[0]);
    let index = ids.iter().position(|id| *id == selected).unwrap_or(0);
    let index = if view.rides_open {
        (index as i64 + i64::from(delta)).rem_euclid(ids.len() as i64) as usize
    } else {
        index
    };
    if view.rides_open || view.selected_ride.is_some() {
        view.select_ride(ids[index]);
    }
    if !view.rides_open {
        return;
    }
    let id = ids[index];
    if view.ride_access_open
        && (sim.is_changed()
            || view
                .ride_access_plan
                .as_ref()
                .is_none_or(|plan| plan.ride != id))
    {
        view.ride_access_plan = sim.0.ride_access_plan(id).ok();
    }
    for (section, mut node) in &mut sections {
        node.display = if section.0 == view.ride_access_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (exit, mut action) in &mut gates {
        let exists = sim.0.ride_by_id(id).is_some_and(|ride| {
            if exit.0 {
                ride.exit.is_some()
            } else {
                ride.entrance.is_some()
            }
        });
        *action = Action::RideAccess(if exists {
            if exit.0 {
                AccessCommand::FindExit
            } else {
                AccessCommand::FindEntrance
            }
        } else {
            AccessCommand::Prepare(if exit.0 {
                RideAccessBuildKind::Exit
            } else {
                RideAccessBuildKind::Entrance
            })
        });
    }
    let journey = sim.0.ride_journey_stats(id);
    let access = sim.0.ride_access(id);
    let economy = sim.0.ride_economy_stats(id);
    let experience = sim.0.ride_experience_stats(id);
    let ride_kind = sim.0.ride_kind(id).unwrap_or(RideKind::Coaster);
    let settings = sim.0.ride_operating_settings(id).unwrap_or_default();
    for (field, mut node) in &mut operating_rows {
        node.display = if field.0.supported(ride_kind) {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (kind, mut text) in &mut labels {
        let value = match kind {
            RideLabel::Selected => format!("{} / #{}", name(&sim.0, id), id.0),
            RideLabel::Details => format!(
                "{} / Ticket {}\nQueue: {}\nExit: {}\nSeats: {}/{} occupied / Completed {}\nRevenue {} / Reviews: {} ({} opinions)\n{}",
                if sim.0.ride_is_open(id) {
                    "Open"
                } else {
                    "Closed"
                },
                money(u64::from(economy.ticket_price_cents)),
                access.status,
                journey.exit_status,
                journey.boarding + journey.riding + journey.exiting,
                journey.capacity,
                compact_count(journey.completed),
                money(economy.revenue_cents),
                guests::rating_label(experience.average_rating_tenths),
                compact_count(experience.rating_count),
                construction_details(&sim.0, id)
            ),
            RideLabel::OperatingValue(field) => operating_value(*field, settings),
            RideLabel::OperatingStatus => sim.0.ride_operating_settings_status(id).map_or_else(
                |error| error.to_string(),
                |()| "Ready to tune. Open the ride when ready.".into(),
            ),
            RideLabel::OperatingNotice => view
                .ride_operating_notice
                .clone()
                .unwrap_or_else(|| "Changes are included when you save the park.".into()),
            RideLabel::ToggleOpen => if sim.0.ride_is_open(id) {
                "Close ride to tune"
            } else {
                "Open ride"
            }
            .into(),
            RideLabel::AccessChecklist => access_checklist(&sim.0, id),
            RideLabel::AccessAdvice => view.ride_access_plan.as_ref().filter(|plan| plan.ride == id).map_or("Choose Access to inspect this ride.".into(), |plan| plan.advice.into()),
            RideLabel::AccessNotice => view.ride_access_notice.clone().unwrap_or_else(|| "Prepare selects a tool and shows a ghost.\nMove to it and click to build one tile.".into()),
            RideLabel::AccessGate(exit) => {
                let exists = sim.0.ride_by_id(id).is_some_and(|ride| if *exit { ride.exit.is_some() } else { ride.entrance.is_some() });
                format!("{} {}", if exists { "Find existing" } else { "Prepare suggested" }, if *exit { "exit" } else { "entrance" })
            }
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

fn access_checklist(park: &Park, id: RideId) -> String {
    let Some(ride) = park.ride_by_id(id) else {
        return "Ride unavailable".into();
    };
    let entrance = park.ride_access(id);
    let exit = park.ride_exit_access(id);
    let ready = entrance.status == RideAccessStatus::Ready && exit.status == RideExitStatus::Ready;
    format!(
        "{}\nEntrance gate: {} / Exit gate: {}\nQueue: {}\nExit path: {}",
        if ready {
            "Access ready for guests"
        } else {
            "Access needs attention"
        },
        if ride.entrance.is_some() {
            "built"
        } else {
            "missing"
        },
        if ride.exit.is_some() {
            "built"
        } else {
            "missing"
        },
        entrance.status,
        exit.status
    )
}

fn operating_value(field: OperatingField, settings: park_sim::RideOperatingSettings) -> String {
    let seconds = |ticks| u64::from(ticks) / TICKS_PER_SECOND;
    // Track positions use 1,024 distance units per tile. Show game distance,
    // not physical metres or the simulation's fixed-point representation.
    let speed =
        |value| f64::from(value) * TICKS_PER_SECOND as f64 / f64::from(park_sim::TRACK_UNITS);
    match field {
        OperatingField::StationWait => format!(
            "Station wait / {} s\n1-30 s; step 1 s",
            seconds(settings.station_wait_ticks)
        ),
        OperatingField::LiftSpeed => format!(
            "Lift speed / {:.2} tiles/s\n{:.2}-{:.2} tiles/s",
            speed(settings.lift_speed),
            speed(park_sim::MIN_LIFT_SPEED),
            speed(park_sim::MAX_LIFT_SPEED)
        ),
        OperatingField::BrakeSpeed => format!(
            "Brake speed / {:.2} tiles/s\n{:.2}-{:.2} tiles/s",
            speed(settings.brake_speed),
            speed(park_sim::MIN_BRAKE_SPEED),
            speed(park_sim::MAX_BRAKE_SPEED)
        ),
        OperatingField::CycleDuration => format!(
            "Cycle duration / {} s\n12-60 s; step 6 s",
            seconds(settings.cycle_ticks)
        ),
    }
}
