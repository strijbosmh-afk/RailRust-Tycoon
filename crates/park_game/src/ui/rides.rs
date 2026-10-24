//! Stable ride selection and the build entry point; no extra permanent panels.
use super::*;
use crate::access_planning::AccessCommand;
use crate::ride_operations::{OperatingField, OperationCommand};
use park_sim::{CAROUSEL_BUILD_COST_CENTS, CAROUSEL_REFUND_PERCENT, Park, RideId, RideKind};
use park_sim::{RideAccessBuildKind, RideAccessStatus, RideExitStatus};

pub(crate) const PANEL_WIDTH: f32 = 820.0;
pub(crate) const PANEL_HEIGHT: f32 = 540.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum RideTab {
    #[default]
    Operations,
    Access,
    Test,
}

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

pub(crate) fn reputation_label(profile: park_sim::RideAppealProfile) -> String {
    format!(
        "Choice reputation: {} (starts neutral)",
        guests::rating_label(Some(profile.reputation_rating_tenths))
    )
}

fn appeal_details(park: &Park, id: RideId) -> String {
    park.ride_appeal_profile(id).map_or_else(
        || "Current ride challenge: unavailable\nChoice reputation: unavailable".into(),
        |profile| {
            format!(
                "Current ride challenge: {}/100\n{}",
                profile.layout_challenge,
                reputation_label(profile)
            )
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
    TestStatus,
    TestMetrics,
    TestRatings,
    TestNotice,
}

#[derive(Component)]
pub(crate) struct OperatingRow(pub(crate) OperatingField);

#[derive(Component)]
pub(crate) struct RideSection(pub(crate) RideTab);

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
                height: px(36),
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
                    button(p, "Operations", Action::RideAccess(AccessCommand::ShowOperations), 114.0);
                    button(p, "Access", Action::RideAccess(AccessCommand::ShowAccess), 114.0);
                    button(p, "Test", Action::CoasterTest(crate::coaster_testing::TestCommand::Show), 114.0);
                });
                p.spawn((RideSection(RideTab::Operations), Node { width: percent(100), flex_direction: FlexDirection::Column, row_gap: px(4), ..default() })).with_children(|p| {
                label(p, "Operating settings", 17.0, INK);
                p.spawn((Button, Action::RideOperating(OperationCommand::ToggleOpen), Node {
                    width: percent(100), height: px(30), flex_shrink: 0.0, align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center, border: UiRect::all(px(1)), ..default()
                }, BackgroundColor(BUTTON), BorderColor::all(BORDER), BorderRadius::all(px(5))))
                .with_children(|p| operating_label(p, RideLabel::ToggleOpen, 13.0, INK));
                p.spawn(Node { height: px(30), flex_shrink: 0.0, ..default() })
                    .with_children(|p| operating_label(p, RideLabel::OperatingStatus, 12.0, MUTED));
                operating_row(p, OperatingField::StationWait);
                operating_row(p, OperatingField::LiftSpeed);
                operating_row(p, OperatingField::BrakeSpeed);
                operating_row(p, OperatingField::CycleDuration);
                operating_row(p, OperatingField::MinimumPassengers);
                operating_row(p, OperatingField::MaxExtraWait);
                button(p, "Reset operating defaults", Action::RideOperating(OperationCommand::ResetDefaults), 356.0);
                p.spawn(Node { height: px(32), flex_shrink: 0.0, ..default() })
                    .with_children(|p| operating_label(p, RideLabel::OperatingNotice, 12.0, MUTED));
                });
                p.spawn((RideSection(RideTab::Access), Node { display: Display::None, width: percent(100), flex_direction: FlexDirection::Column, row_gap: px(6), ..default() })).with_children(|p| {
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
                p.spawn((RideSection(RideTab::Test), Node { display: Display::None, width: percent(100), flex_direction: FlexDirection::Column, row_gap: px(6), ..default() })).with_children(|p| {
                    label(p, "Installed coaster test", 17.0, INK);
                    button(p, "Run one isolated lap", Action::CoasterTest(crate::coaster_testing::TestCommand::Run), 356.0);
                    p.spawn(Node { height: px(30), flex_shrink: 0.0, ..default() })
                        .with_children(|p| operating_label(p, RideLabel::TestStatus, 12.0, MUTED));
                    p.spawn(Node { height: px(140), flex_shrink: 0.0, ..default() })
                        .with_children(|p| operating_label(p, RideLabel::TestMetrics, 12.0, INK));
                    p.spawn(Node { height: px(34), flex_shrink: 0.0, ..default() })
                        .with_children(|p| operating_label(p, RideLabel::TestRatings, 12.0, INK));
                    label(p, park_sim::COASTER_TEST_LIMITATIONS, 11.0, MUTED);
                    p.spawn(Node { height: px(34), flex_shrink: 0.0, ..default() })
                        .with_children(|p| operating_label(p, RideLabel::TestNotice, 12.0, MUTED));
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
    view.coaster_testing.refresh(&sim.0, Some(id));
    // Construction is blocked while this modal is open. Opening the modal,
    // changing tabs/ride or loading clears this topology-only plan; ticking
    // guests and trains must not repeat the bounded route search every frame.
    // Commands re-plan and validate against the current park before preparing.
    if view.ride_tab == RideTab::Access
        && view
            .ride_access_plan
            .as_ref()
            .is_none_or(|plan| plan.ride != id)
    {
        view.ride_access_plan = sim.0.ride_access_plan(id).ok();
    }
    for (section, mut node) in &mut sections {
        node.display = if section.0 == view.ride_tab {
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
    let policy = sim.0.ride_dispatch_policy(id).unwrap_or_default();
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
                "{} / Ticket {}\nQueue: {}\nExit: {}\nSeats: {}/{} occupied / Completed {}\nRevenue {} / Reviews: {} ({} opinions)\n{}\n{}",
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
                appeal_details(&sim.0, id),
                construction_details(&sim.0, id)
            ),
            RideLabel::OperatingValue(field) => operating_value(*field, settings, policy),
            RideLabel::OperatingStatus => sim.0.ride_dispatch_progress(id).filter(|progress| progress.waiting_for_load).map_or_else(
                || sim.0.ride_operating_settings_status(id).map_or_else(|error| error.to_string(), |()| "Ready to tune. Open the ride when ready.".into()),
                |progress| format!("Loading {}/{} / extra wait {:.1}/{:.0} s\nClose and empty the ride before tuning.", progress.seated_passengers, progress.minimum_passengers, f64::from(progress.extra_waited_ticks) / TICKS_PER_SECOND as f64, f64::from(progress.maximum_extra_wait_ticks) / TICKS_PER_SECOND as f64)),
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
            RideLabel::TestStatus => crate::coaster_testing::status_text(&sim.0, &view.coaster_testing, Some(id)),
            RideLabel::TestMetrics => crate::coaster_testing::metrics_text(&view.coaster_testing),
            RideLabel::TestRatings => crate::coaster_testing::ratings_text(&view.coaster_testing),
            RideLabel::TestNotice => view.coaster_testing.notice.clone().unwrap_or_else(|| "Testing leaves time, guests and money unchanged.".into()),
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
    let gate_link = |exists: bool, station: bool| {
        if !exists {
            "missing"
        } else if station {
            "connected"
        } else {
            "not connected"
        }
    };
    let queue = match entrance.status {
        RideAccessStatus::Ready => "connected",
        RideAccessStatus::RideUnavailable => "ride unavailable",
        RideAccessStatus::NoEntrance => "build entrance first",
        RideAccessStatus::StationDisconnected => "reconnect entrance first",
        RideAccessStatus::NoQueue => "missing",
        RideAccessStatus::Disconnected => "join disconnected pieces",
        RideAccessStatus::Branch => "remove branch",
        RideAccessStatus::Loop => "break loop",
        RideAccessStatus::NoPublicEntry => "join public path at far end",
        RideAccessStatus::MultiplePublicEntries => "keep one public entry",
        RideAccessStatus::PublicEntryNotAtEnd => "move public entry to far end",
        RideAccessStatus::ForeignQueue => "separate from other ride",
    };
    let next = match exit.status {
        RideExitStatus::NoPublicExit => ride
            .exit
            .and_then(|gate| {
                let object = park.object(gate.object)?;
                let pos = gate.facing.opposite().neighbor(object.pos)?;
                park.tile(pos)?;
                Some(format!(
                    "Public tile {}, {} / height {}",
                    pos.x + 1,
                    pos.y + 1,
                    object.height
                ))
            })
            .unwrap_or_else(|| "Public path: no room outside gate".into()),
        RideExitStatus::PublicDisconnected => "Return route: not connected".into(),
        RideExitStatus::Ready => "Return route: connected to queue entry".into(),
        RideExitStatus::NoExit => "Return route: build exit first".into(),
        RideExitStatus::StationDisconnected => "Return route: reconnect exit first".into(),
        RideExitStatus::RideUnavailable => "Return route: restore ride first".into(),
    };
    format!(
        "{}\nEntrance to ride: {}\nQueue: {}\nExit to ride: {}\nExit path: {}\n{}",
        if ready {
            "Access ready for guests"
        } else {
            "Access needs attention"
        },
        gate_link(ride.entrance.is_some(), entrance.station.is_some()),
        queue,
        gate_link(ride.exit.is_some(), exit.station.is_some()),
        match exit.status {
            RideExitStatus::Ready | RideExitStatus::PublicDisconnected => "connected to gate",
            RideExitStatus::NoPublicExit => "missing outside gate",
            _ => "not ready",
        },
        next
    )
}

fn operating_value(
    field: OperatingField,
    settings: park_sim::RideOperatingSettings,
    policy: park_sim::RideDispatchPolicy,
) -> String {
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
        OperatingField::MinimumPassengers => format!(
            "Minimum load / {}\n0-6 passengers; 0 means no minimum",
            if policy.minimum_passengers == 0 {
                "No minimum".into()
            } else {
                format!("{} of 6", policy.minimum_passengers)
            }
        ),
        OperatingField::MaxExtraWait => format!(
            "Extra loading wait / {} s\n1-60 s; after mandatory station wait",
            seconds(policy.maximum_extra_wait_ticks)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::TilePos;

    #[test]
    fn checklist_distinguishes_station_link_landing_and_return_route() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(42).unwrap();
        park.install_train_fixture().unwrap();
        let id = park.ride().unwrap().id;
        let missing = access_checklist(&park, id);
        assert!(missing.contains("Exit to ride: missing"));
        assert!(missing.contains("Return route: build exit first"));

        park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, id)
            .unwrap();
        for y in 12..=15 {
            park.place_queue_path(TilePos { x: 8, y }, 8, PathSlope::Flat, id)
                .unwrap();
        }
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, id)
            .unwrap();
        let no_path = access_checklist(&park, id);
        assert!(no_path.contains("Entrance to ride: connected\nQueue: connected"));
        assert!(no_path.contains("Exit to ride: connected\nExit path: missing outside gate"));
        assert!(no_path.contains("Public tile 9, 9 / height 8"));
        assert_eq!(no_path.lines().count(), 6);

        park.place_path(TilePos { x: 8, y: 8 }, 8).unwrap();
        let disconnected = access_checklist(&park, id);
        assert!(disconnected.contains("Exit to ride: connected\nExit path: connected to gate"));
        assert!(disconnected.contains("Return route: not connected"));
        assert!(!disconnected.contains("Public tile"));
        for x in 9..=13 {
            park.place_path(TilePos { x, y: 8 }, 8).unwrap();
        }
        for y in 9..=11 {
            park.place_path(TilePos { x: 13, y }, 8).unwrap();
        }
        let ready = access_checklist(&park, id);
        assert!(ready.starts_with("Access ready for guests"));
        assert!(ready.contains("Return route: connected to queue entry"));
        assert_eq!(ready.lines().count(), 6);

        let station = park.ride_exit_access(id).station.unwrap();
        park.remove_object(station).unwrap();
        let unavailable = access_checklist(&park, id);
        assert!(unavailable.contains("Exit to ride: not connected"));
        assert!(unavailable.contains("Return route: restore ride first"));
        assert_eq!(
            access_checklist(&park, RideId(u64::MAX)),
            "Ride unavailable"
        );
    }
}
