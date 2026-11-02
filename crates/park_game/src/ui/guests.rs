//! Guest inspection and explicit visitor controls. List slots are not identities.
use super::*;
use park_sim::{Guest, GuestState, Park, RideVisitPhase, ServiceVisitPhase};

pub(crate) const PAGE_SIZE: usize = 8;
pub(crate) const PANEL_WIDTH: f32 = 760.0;
pub(crate) const PANEL_HEIGHT: f32 = 520.0;

#[derive(Component)]
pub(crate) struct GuestPanel;
#[derive(Component)]
pub(crate) struct GuestRow(usize);
#[derive(Component)]
pub(crate) struct GuestInspection;
#[derive(Component, Clone, Copy)]
pub(crate) enum GuestAvatar {
    Row(usize),
    Selected,
}
#[derive(Component, Clone, Copy)]
pub(crate) enum GuestMeterFill {
    Mood,
    Hunger,
    Thirst,
    Toilet,
}
#[derive(Component)]
pub(crate) enum GuestLabel {
    Summary,
    FlowView,
    Page,
    Row(usize),
    Title,
    Details,
    Thought,
    Demand,
    Meter(GuestMeterFill),
}

fn text(parent: &mut ChildSpawnerCommands, initial: &str, size: f32, kind: GuestLabel) {
    parent.spawn((
        Text::new(initial),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(INK),
        kind,
    ));
}

fn meter(parent: &mut ChildSpawnerCommands, kind: GuestMeterFill) {
    parent
        .spawn(Node {
            width: percent(100),
            height: px(19),
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..row()
        })
        .with_children(|p| {
            p.spawn(Node {
                width: px(122),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|p| text(p, "", 11.0, GuestLabel::Meter(kind)));
            p.spawn((
                Node {
                    width: px(250),
                    height: px(10),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(217, 225, 210)),
                BorderRadius::all(px(5)),
            ))
            .with_children(|p| {
                p.spawn((
                    kind,
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(GOLD),
                    BorderRadius::all(px(5)),
                ));
            });
        });
}

pub(super) fn setup(commands: &mut Commands) {
    commands
        .spawn((
            GuestPanel,
            GlobalZIndex(9),
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
                    width: px(PANEL_WIDTH),
                    height: px(PANEL_HEIGHT),
                    padding: UiRect::all(px(20)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..row()
                })
                .with_children(|p| {
                    label(p, "Guests", 23.0, INK);
                    p.spawn((
                        Button,
                        Action::GuestFlowView,
                        Node {
                            width: px(140),
                            height: px(30),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(BUTTON),
                    ))
                    .with_children(|p| text(p, "Visitor flow", 12.0, GuestLabel::FlowView));
                    button(p, "Close [L / Esc]", Action::CloseGuests, 140.0);
                });
                text(p, "Guest experience", 12.0, GuestLabel::Summary);
                super::visitor_flow::setup(p);
                p.spawn((
                    GuestInspection,
                    Node {
                        height: px(350),
                        flex_shrink: 0.0,
                        column_gap: px(18),
                        ..row()
                    },
                ))
                .with_children(|p| {
                    p.spawn(Node {
                        width: px(230),
                        height: percent(100),
                        flex_shrink: 0.0,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    })
                    .with_children(|p| {
                        for index in 0..PAGE_SIZE {
                            p.spawn((
                                Button,
                                GuestRow(index),
                                Action::SelectGuest(None),
                                Node {
                                    width: percent(100),
                                    height: px(35),
                                    flex_shrink: 0.0,
                                    padding: UiRect::axes(px(7), px(3)),
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(px(1)),
                                    ..default()
                                },
                                BackgroundColor(BUTTON),
                                BorderColor::all(BORDER),
                                BorderRadius::all(px(5)),
                            ))
                            .with_children(|p| {
                                p.spawn((
                                    GuestAvatar::Row(index),
                                    Node {
                                        width: px(17),
                                        height: px(17),
                                        margin: UiRect::right(px(7)),
                                        flex_shrink: 0.0,
                                        ..default()
                                    },
                                    BackgroundColor(BUTTON),
                                    BorderRadius::all(px(9)),
                                ));
                                text(p, "", 11.0, GuestLabel::Row(index));
                            });
                        }
                        p.spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            ..row()
                        })
                        .with_children(|p| {
                            button(p, "<", Action::GuestPage(-1), 35.0);
                            text(p, "1 / 1", 11.0, GuestLabel::Page);
                            button(p, ">", Action::GuestPage(1), 35.0);
                        });
                    });
                    p.spawn(Node {
                        width: px(472),
                        height: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..row()
                        })
                        .with_children(|p| {
                            text(p, "Select a guest", 20.0, GuestLabel::Title);
                            p.spawn((
                                GuestAvatar::Selected,
                                Node {
                                    width: px(28),
                                    height: px(28),
                                    ..default()
                                },
                                BackgroundColor(BUTTON),
                                BorderRadius::all(px(14)),
                            ));
                        });
                        p.spawn((
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(3),
                                padding: UiRect::axes(px(8), px(6)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb_u8(232, 238, 224)),
                            BorderRadius::all(px(5)),
                        ))
                        .with_children(|p| {
                            for kind in [
                                GuestMeterFill::Mood,
                                GuestMeterFill::Hunger,
                                GuestMeterFill::Thirst,
                                GuestMeterFill::Toilet,
                            ] {
                                meter(p, kind);
                            }
                        });
                        text(p, "", 11.0, GuestLabel::Details);
                        text(p, "", 11.0, GuestLabel::Thought);
                        text(p, "", 11.0, GuestLabel::Demand);
                    });
                });
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..row()
                })
                .with_children(|p| {
                    label(
                        p,
                        "Up/Down: guests  PgUp/PgDn: pages  Space: pause",
                        11.0,
                        MUTED,
                    );
                    button(p, "Pause", Action::Pause, 80.0);
                });
            });
        });
}

pub(crate) fn rating_label(tenths: Option<u8>) -> String {
    tenths.map_or_else(
        || "Unrated".into(),
        |rating| format!("{}.{}/5", rating / 10, rating % 10),
    )
}

fn activity(guest: &Guest) -> &'static str {
    if guest.state == GuestState::Lost {
        return "Lost / needs a path";
    }
    if let Some(visit) = &guest.service_visit {
        return match visit.phase {
            ServiceVisitPhase::Approaching => "Seeking a service",
            ServiceVisitPhase::Waiting => "Waiting for a service",
            ServiceVisitPhase::Using => "Using a service",
        };
    }
    match guest.ride_visit.as_ref().map(|visit| visit.phase) {
        Some(RideVisitPhase::Approaching) => "Heading to ride",
        Some(RideVisitPhase::Queueing) => "In the queue",
        Some(RideVisitPhase::Leaving) => "Returning to paths",
        Some(RideVisitPhase::Boarding) => "Boarding the ride",
        Some(RideVisitPhase::Riding) => "On the ride",
        Some(RideVisitPhase::Exiting) => "Leaving the ride",
        None if guest.state == GuestState::Walking => "Walking in the park",
        None => "Waiting on a path",
    }
}

fn details(guest: &Guest, park: &Park) -> String {
    let review = guest
        .last_rating
        .map_or_else(|| "none".into(), |rating| format!("{rating}/5"));
    let service = guest
        .service_visit
        .as_ref()
        .map_or_else(String::new, |visit| match visit.phase {
            ServiceVisitPhase::Using => format!(
                " / service #{}: {}s left",
                visit.service.0,
                u64::from(park_sim::SERVICE_USE_TICKS.saturating_sub(visit.progress))
                    .div_ceil(TICKS_PER_SECOND)
            ),
            ServiceVisitPhase::Waiting => format!(" / service #{}: waiting", visit.service.0),
            ServiceVisitPhase::Approaching => format!(" / service #{}", visit.service.0),
        });
    let activity = guest.ride_visit.as_ref().map_or_else(
        || format!("{}{}", activity(guest), service),
        |visit| format!("{} / ride #{}", activity(guest), visit.ride.0),
    );
    format!(
        "Now: {activity}\nWallet {} / Spent {} (services {})\nTicket comfort {} / Queue comfort {}s / Wait {}s\nPrefers {} / Challenge {}\nTrips {} / Services {} / Reviews {} (latest {review})",
        money(u64::from(guest.wallet_cents)),
        money(guest.spent_cents),
        money(guest.service_spent_cents),
        money(u64::from(guest.preferences.max_ticket_price_cents)),
        u64::from(guest.preferences.comfortable_queue_ticks) / TICKS_PER_SECOND,
        u64::from(
            guest
                .ride_visit
                .as_ref()
                .map_or(0, |visit| visit.waited_ticks)
        ) / TICKS_PER_SECOND,
        park.guest_preferred_ride_kind(guest.id)
            .map_or_else(|| "unknown".into(), |kind| kind.to_string()),
        park.guest_preferred_challenge(guest.id).map_or_else(
            || "unavailable".into(),
            |challenge| format!("{challenge}/100")
        ),
        super::compact_count(guest.rides_completed),
        super::compact_count(guest.services_used),
        super::compact_count(guest.ratings_given),
    )
}

fn demand(guest: &Guest, park: &Park, selected: Option<park_sim::RideId>) -> String {
    let Some(ride) = selected.and_then(|id| park.ride_by_id(id)) else {
        return "RIDE CHOICE  No ride is available to consider.".into();
    };
    let affordable = guest.wallet_cents >= ride.ticket_price_cents;
    let willing = guest.willing_to_pay(ride.ticket_price_cents);
    let yes_no = |value| if value { "yes" } else { "no" };
    let appeal = park.guest_ride_appeal(guest.id, ride.id).map_or_else(
        || "Challenge unavailable / fit unavailable / reputation unavailable".into(),
        |appeal| {
            format!(
                "Challenge {}/100 / {} preferred level\n{}",
                appeal.profile.layout_challenge,
                if appeal.profile.layout_challenge <= appeal.preferred_challenge {
                    "within"
                } else {
                    "above"
                },
                super::rides::reputation_label(appeal.profile),
            )
        },
    );
    format!(
        "RIDE CHOICE  {} / ticket {}\nCan pay: {} / Willing: {} / Trips here: {}\n{appeal}",
        super::rides::name(park, ride.id),
        money(u64::from(ride.ticket_price_cents)),
        yes_no(affordable),
        yes_no(willing),
        super::compact_count(
            guest
                .ride_history
                .iter()
                .find(|history| history.ride == ride.id)
                .map_or(0, |history| history.completed)
        ),
    )
}

fn meter_value(guest: &Guest, kind: GuestMeterFill) -> (&'static str, u8) {
    match kind {
        GuestMeterFill::Mood => ("Satisfaction", guest.satisfaction),
        GuestMeterFill::Hunger => ("Hunger", guest.needs.hunger),
        GuestMeterFill::Thirst => ("Thirst", guest.needs.thirst),
        GuestMeterFill::Toilet => ("Toilet", guest.needs.toilet),
    }
}

fn meter_color(kind: GuestMeterFill, value: u8) -> Color {
    let warning = match kind {
        GuestMeterFill::Mood => value <= 40,
        _ => value >= 70,
    };
    let caution = match kind {
        GuestMeterFill::Mood => value <= 65,
        _ => value >= 40,
    };
    if warning {
        Color::srgb_u8(196, 75, 61)
    } else if caution {
        Color::srgb_u8(215, 153, 52)
    } else {
        Color::srgb_u8(70, 148, 108)
    }
}

fn avatar_color(guest: &Guest) -> Color {
    let [r, g, b] = if guest.state == GuestState::Lost {
        [235, 112, 44]
    } else {
        crate::visitor_look::look(guest.id).shirt
    };
    Color::srgb_u8(r, g, b)
}

/// Stable-ID order is independent of storage order in a valid loaded save.
fn resolve_selection(guests: &[&Guest], view: &mut ViewState) {
    let browse = std::mem::take(&mut view.guest_browse_pending);
    let pages = std::mem::take(&mut view.guest_page_pending);
    if guests.is_empty() {
        view.selected_guest = None;
        view.guest_page = 0;
        return;
    }
    let mut index = view
        .selected_guest
        .and_then(|id| guests.iter().position(|guest| guest.id == id))
        .unwrap_or(0);
    if pages != 0 {
        let page_count = guests.len().div_ceil(PAGE_SIZE);
        let page = (index as i64 / PAGE_SIZE as i64 + i64::from(pages))
            .rem_euclid(page_count as i64) as usize;
        index = (page * PAGE_SIZE + index % PAGE_SIZE).min(guests.len() - 1);
    }
    index = (index as i64 + i64::from(browse)).rem_euclid(guests.len() as i64) as usize;
    view.selected_guest = Some(guests[index].id);
    view.guest_page = index / PAGE_SIZE;
}

type Rows<'w, 's> = Query<
    'w,
    's,
    (
        &'static GuestRow,
        &'static Interaction,
        &'static mut Action,
        &'static mut BackgroundColor,
    ),
    (
        Without<GuestPanel>,
        Without<GuestAvatar>,
        Without<GuestMeterFill>,
    ),
>;
type Avatars<'w, 's> = Query<
    'w,
    's,
    (&'static GuestAvatar, &'static mut BackgroundColor),
    (Without<GuestRow>, Without<GuestMeterFill>),
>;
type Meters<'w, 's> = Query<
    'w,
    's,
    (
        &'static GuestMeterFill,
        &'static mut Node,
        &'static mut BackgroundColor,
    ),
    (
        Without<GuestRow>,
        Without<GuestAvatar>,
        Without<GuestPanel>,
        Without<GuestInspection>,
        Without<super::visitor_flow::VisitorBody>,
    ),
>;

type GuestPanels<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<GuestPanel>,
        Without<GuestInspection>,
        Without<super::visitor_flow::VisitorBody>,
        Without<GuestMeterFill>,
    ),
>;
type GuestBodies<'w, 's> = Query<
    'w,
    's,
    (&'static mut Node, Has<super::visitor_flow::VisitorBody>),
    (
        Or<(
            With<GuestInspection>,
            With<super::visitor_flow::VisitorBody>,
        )>,
        Without<GuestMeterFill>,
    ),
>;
pub(crate) fn refresh(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    mut panels: GuestPanels<'_, '_>,
    mut bodies: GuestBodies<'_, '_>,
    mut rows: Rows<'_, '_>,
    mut labels: Query<(&GuestLabel, &mut Text)>,
    mut visuals: ParamSet<'_, '_, (Avatars<'_, '_>, Meters<'_, '_>)>,
) {
    for mut panel in &mut panels {
        panel.display = if view.guests_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !view.guests_open {
        view.guest_browse_pending = 0;
        view.guest_page_pending = 0;
        return;
    }
    if std::mem::take(&mut view.guest_flow_pending) {
        view.guest_flow_view = !view.guest_flow_view;
        view.visitor_pending = None;
        view.visitor_notice = None;
    }
    for (mut node, is_flow) in &mut bodies {
        node.display = if is_flow == view.guest_flow_view {
            Display::Flex
        } else {
            Display::None
        };
    }
    let flow = sim.0.visitor_flow_stats();
    let mut guests: Vec<_> = sim.0.guests().iter().collect();
    guests.sort_unstable_by_key(|guest| guest.id);
    resolve_selection(&guests, &mut view);
    let selected = view
        .selected_guest
        .and_then(|id| guests.iter().copied().find(|guest| guest.id == id));
    let start = view.guest_page * PAGE_SIZE;
    for (row, interaction, mut action, mut background) in &mut rows {
        let guest = guests.get(start + row.0).copied();
        // Keep empty row slots, so pagination and detail controls never jump.
        *action = Action::SelectGuest(guest.map(|guest| guest.id));
        background.0 = if guest.is_some_and(|guest| Some(guest.id) == view.selected_guest) {
            GOLD
        } else if guest.is_some() && !view.capture_active && *interaction == Interaction::Hovered {
            Color::srgb_u8(207, 220, 197)
        } else {
            BUTTON
        };
    }
    for (avatar, mut background) in &mut visuals.p0() {
        let guest = match avatar {
            GuestAvatar::Row(index) => guests.get(start + index).copied(),
            GuestAvatar::Selected => selected,
        };
        background.0 = guest.map_or(BUTTON, avatar_color);
    }
    for (kind, mut node, mut background) in &mut visuals.p1() {
        let value = selected
            .map_or(0, |guest| meter_value(guest, *kind).1)
            .min(100);
        node.width = percent(f32::from(value));
        background.0 = meter_color(*kind, value);
    }
    let summary = sim.0.guest_experience_summary();
    let ride_id = super::rides::selected_id(&sim.0, &view);
    let experience = ride_id.map(|id| sim.0.ride_experience_stats(id));
    for (kind, mut text) in &mut labels {
        let value = match kind {
            GuestLabel::FlowView => if view.guest_flow_view {
                "Guest list"
            } else {
                "Visitor flow"
            }
            .into(),
            GuestLabel::Summary if view.guest_flow_view => format!(
                "{} active guests / {} leaving / {} finishing visits\nArrival flow {} / {} lifetime departures",
                flow.active,
                flow.leaving,
                flow.finishing_visits,
                if flow.enabled { "on" } else { "off" },
                flow.departed
            ),
            GuestLabel::Summary => format!(
                "{} guests / Average satisfaction {} / {} unhappy\n{} reviews: {} from {} opinions",
                guests.len(),
                summary
                    .average_satisfaction
                    .map_or_else(|| "none".into(), |value| format!("{value}/100")),
                summary.unhappy_guests,
                ride_id.map_or("Selected ride", |id| super::rides::name(&sim.0, id)),
                rating_label(
                    experience
                        .as_ref()
                        .and_then(|stats| stats.average_rating_tenths)
                ),
                super::compact_count(experience.as_ref().map_or(0, |stats| stats.rating_count))
            ),
            GuestLabel::Page => format!(
                "{} / {}",
                view.guest_page + 1,
                guests.len().div_ceil(PAGE_SIZE).max(1)
            ),
            GuestLabel::Row(index) => guests.get(start + index).map_or_else(
                || "".into(),
                |guest| {
                    format!(
                        "Guest #{}  {}/100\n{}",
                        guest.id.0,
                        guest.satisfaction,
                        activity(guest)
                    )
                },
            ),
            GuestLabel::Title => selected.map_or_else(
                || "No guests in this park".into(),
                |guest| format!("Guest #{}", guest.id.0),
            ),
            GuestLabel::Details => {
                selected.map_or_else(String::new, |guest| details(guest, &sim.0))
            }
            GuestLabel::Thought => selected.map_or_else(String::new, |guest| {
                format!(
                    "THOUGHT  {}s ago\n{}",
                    sim.0.ticks().saturating_sub(guest.thought_tick) / TICKS_PER_SECOND,
                    guest.thought
                )
            }),
            GuestLabel::Demand => {
                selected.map_or_else(String::new, |guest| demand(guest, &sim.0, ride_id))
            }
            GuestLabel::Meter(kind) => selected.map_or_else(String::new, |guest| {
                let (name, value) = meter_value(guest, *kind);
                format!("{name}  {value}/100")
            }),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
