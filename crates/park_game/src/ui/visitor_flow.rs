//! Visitor management uses explicit commands; choosing the view is read-only.
use super::*;

#[derive(Component)]
pub(crate) struct VisitorBody;
#[derive(Component)]
pub(crate) enum VisitorLabel {
    Entry,
    Totals,
    Pricing,
    Marketing,
    Rules,
    Selected,
    Notice,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VisitorCommand {
    SetEntry,
    Toggle,
    TogglePark,
    EntryDown,
    EntryUp,
    CycleRule,
    RunFlyers,
    RunRadio,
    SendHome,
    Conflict,
}
pub(crate) fn queue(view: &mut ViewState, command: VisitorCommand) {
    view.visitor_pending = Some(if view.visitor_pending.is_some() {
        VisitorCommand::Conflict
    } else {
        command
    });
}
pub(crate) fn setup(p: &mut ChildSpawnerCommands) {
    p.spawn((
        VisitorBody,
        Node {
            display: Display::None,
            height: px(420),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
    ))
    .with_children(|p| {
        for kind in [
            VisitorLabel::Entry,
            VisitorLabel::Totals,
            VisitorLabel::Pricing,
            VisitorLabel::Marketing,
            VisitorLabel::Rules,
            VisitorLabel::Selected,
        ] {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(INK),
                kind,
            ));
        }
        p.spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            ..row()
        })
        .with_children(|p| {
            button(
                p,
                "Set entry",
                Action::Visitor(VisitorCommand::SetEntry),
                145.0,
            );
            button(
                p,
                "Toggle arrivals",
                Action::Visitor(VisitorCommand::Toggle),
                145.0,
            );
            button(
                p,
                "Open / close park",
                Action::Visitor(VisitorCommand::TogglePark),
                145.0,
            );
            button(
                p,
                "Send guest home",
                Action::Visitor(VisitorCommand::SendHome),
                145.0,
            );
        });
        p.spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            ..row()
        })
        .with_children(|p| {
            button(
                p,
                "Entry -$0.50",
                Action::Visitor(VisitorCommand::EntryDown),
                145.0,
            );
            button(
                p,
                "Entry +$0.50",
                Action::Visitor(VisitorCommand::EntryUp),
                145.0,
            );
            button(
                p,
                "Next charging rule",
                Action::Visitor(VisitorCommand::CycleRule),
                185.0,
            );
            button(
                p,
                "Run flyers $25",
                Action::Visitor(VisitorCommand::RunFlyers),
                145.0,
            );
        });
        p.spawn((
            Text::new(""),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(INK),
            VisitorLabel::Notice,
        ));
        p.spawn(Node { ..row() }).with_children(|p| {
            button(
                p,
                "Run radio ads $40",
                Action::Visitor(VisitorCommand::RunRadio),
                160.0,
            );
        });
    });
}
pub(crate) fn apply_commands(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    let Some(command) = view.visitor_pending.take() else {
        return;
    };
    if !view.guests_open
        || !view.guest_flow_view
        || view.guest_flow_pending
        || view.capture_active
        || view.modal_input_blocked
        || view.diagnostic_case.is_some()
    {
        return;
    }
    let result = match command {
        VisitorCommand::SetEntry => match view.selected {
            Some(SurfaceId::Object(id)) => sim
                .0
                .set_visitor_entry(id)
                .map(|_| "Visitor entry set to the selected public path.".into()),
            _ => Err(park_sim::VisitorError::PublicPath),
        },
        VisitorCommand::Toggle => {
            let enabled = !sim.0.visitor_flow_stats().enabled;
            sim.0.set_visitor_arrivals(enabled).map(|_| {
                if enabled {
                    "Arrivals enabled; current guests begin tracked visits.".into()
                } else {
                    "Arrivals stopped; tracked guests still finish and leave.".into()
                }
            })
        }
        VisitorCommand::TogglePark => {
            let open = !sim.0.visitor_flow_stats().park_open;
            sim.0.set_park_open(open);
            Ok(if open {
                "Park opened. Enabled arrivals can resume after a full interval.".into()
            } else {
                "Park closed. Current guests finish rides/services, then leave by the entry.".into()
            })
        }
        VisitorCommand::EntryDown | VisitorCommand::EntryUp => {
            let price = sim.0.visitor_flow_stats().entry_price_cents;
            let next = if command == VisitorCommand::EntryDown {
                price.saturating_sub(park_sim::PARK_ENTRY_PRICE_STEP_CENTS)
            } else {
                price
                    .saturating_add(park_sim::PARK_ENTRY_PRICE_STEP_CENTS)
                    .min(park_sim::MAX_PARK_ENTRY_PRICE_CENTS)
            };
            sim.0.set_park_entry_price(next).map(|_| {
                format!(
                    "Park entry set to {} for future arrivals.",
                    money(u64::from(next))
                )
            })
        }
        VisitorCommand::CycleRule => {
            let next = sim.0.visitor_flow_stats().charging_rule.next();
            sim.0.set_charging_rule(next);
            Ok(format!(
                "Charging rule: {}. Restricted prices reset to zero; existing paid visits finish.",
                next.label()
            ))
        }
        VisitorCommand::RunFlyers => sim.0.start_flyer_campaign().map(|_| {
            "Local flyers started for five simulation minutes; future candidates arrive twice as often."
                .into()
        }),
        VisitorCommand::RunRadio => sim.0.start_radio_campaign().map(|_| {
            "Radio ads started for five simulation minutes; future candidates accept up to $3.00 more at the gate.".into()
        }),
        VisitorCommand::SendHome => view
            .selected_guest
            .ok_or(park_sim::VisitorError::UnknownGuest)
            .and_then(|id| sim.0.request_guest_departure(id))
            .map(|_| {
                "Departure requested; the guest finishes their visit and walks to entry.".into()
            }),
        VisitorCommand::Conflict => {
            view.visitor_notice = Some("Choose one visitor command at a time.".into());
            return;
        }
    };
    view.visitor_notice = Some(result.unwrap_or_else(|e| format!("Visitors unchanged: {e}")));
}
pub(crate) fn refresh(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut labels: Query<(&VisitorLabel, &mut Text)>,
) {
    if !view.guests_open || !view.guest_flow_view {
        return;
    }
    let s = sim.0.visitor_flow_stats();
    for (label, mut text) in &mut labels {
        text.0 = match label {
            VisitorLabel::Entry => format!(
                "Entry: {} / {}\nSelect an ordinary public path in the park, then choose Set entry.",
                s.entry.and_then(|id| sim.0.object(id)).map_or_else(
                    || "not set".into(),
                    |p| format!("({}, {}) at height {}", p.pos.x, p.pos.y, p.height)
                ),
                if s.entry_ready {
                    "connected"
                } else {
                    "missing or isolated"
                }
            ),
            VisitorLabel::Totals => format!(
                "New arrivals {} / Departed {} / {} tracked visits\nDeparted visitors spent {} and left with {} of their own money.",
                s.spawned,
                s.departed,
                s.tracked,
                money(s.departed_spent_cents),
                money(s.departed_wallet_cents)
            ),
            VisitorLabel::Pricing => format!(
                "Entry price {} / Charging rule {} / {} declined entry\nLifetime park entry income {}. Price changes affect future arrivals only.",
                money(u64::from(s.entry_price_cents)),
                s.charging_rule.label(),
                s.turned_away,
                money(s.admission_revenue_cents)
            ),
            VisitorLabel::Marketing => format!(
                "Flyers: {} / {} runs / {} ad candidates. $25; faster arrivals.\nRadio: {} / {} runs / {} ad candidates / {} extra entries. $40; +$3 willingness.",
                if s.flyer_ticks_remaining == 0 {
                    "inactive".into()
                } else {
                    format!(
                        "{}:{:02} left",
                        s.flyer_ticks_remaining / 1_800,
                        (s.flyer_ticks_remaining % 1_800) / 30
                    )
                },
                s.flyer_campaigns,
                s.flyer_candidates,
                if s.radio_ticks_remaining == 0 {
                    "inactive".into()
                } else {
                    format!(
                        "{}:{:02} left",
                        s.radio_ticks_remaining / 1_800,
                        (s.radio_ticks_remaining % 1_800) / 30
                    )
                },
                s.radio_campaigns,
                s.radio_candidates,
                s.radio_admissions
            ),
            VisitorLabel::Rules => format!(
                "Park: {} / Arrivals: {}. One candidate per 30 simulation seconds, up to 64.\nFlyers: 15-second candidates. Radio: +$3 gate willingness. Campaigns may overlap.\nClosing sends guests home after visits; paid campaigns still expire.\nFree entry blocks entry fees; Free rides blocks ride prices. Space pauses activity.",
                if s.park_open { "OPEN" } else { "CLOSED" },
                if s.enabled { "enabled" } else { "disabled" }
            ),
            VisitorLabel::Selected => view.selected_guest.map_or_else(
                || "No active guest selected. Choose one in Guest list first.".into(),
                |id| {
                    format!(
                        "Selected Guest #{}: {}\n{} leaving / {} finishing ride or service visits.",
                        id.0,
                        if sim.0.guest_is_leaving(id) {
                            "leaving after current visit"
                        } else {
                            "visiting"
                        },
                        s.leaving,
                        s.finishing_visits
                    )
                },
            ),
            VisitorLabel::Notice => view.visitor_notice.clone().unwrap_or_else(|| {
                "Arrivals are opt-in. If entry is removed, departures wait for a new entry.".into()
            }),
        };
    }
}
