//! Visitor management uses explicit commands; choosing the view is read-only.
use super::*;

#[derive(Component)]
pub(crate) struct VisitorBody;
#[derive(Component)]
pub(crate) enum VisitorLabel {
    Entry,
    Totals,
    Rules,
    Selected,
    Notice,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VisitorCommand {
    SetEntry,
    Toggle,
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
            height: px(350),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(15),
            ..default()
        },
    ))
    .with_children(|p| {
        for kind in [
            VisitorLabel::Entry,
            VisitorLabel::Totals,
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
                "Use selected path",
                Action::Visitor(VisitorCommand::SetEntry),
                200.0,
            );
            button(
                p,
                "Toggle arrivals",
                Action::Visitor(VisitorCommand::Toggle),
                200.0,
            );
            button(
                p,
                "Send selected home",
                Action::Visitor(VisitorCommand::SendHome),
                200.0,
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
        text.0=match label {
            VisitorLabel::Entry=>format!("Entry: {} / {}\nSelect an ordinary public path in the park, then choose Use selected path.",s.entry.and_then(|id|sim.0.object(id)).map_or_else(||"not set".into(),|p|format!("({}, {}) at height {}",p.pos.x,p.pos.y,p.height)),if s.entry_ready{"connected"}else{"missing or isolated"}),
            VisitorLabel::Totals=>format!("New arrivals {} / Departed {} / {} tracked visits\nDeparted visitors spent {} and left with {} of their own money.",s.spawned,s.departed,s.tracked,money(s.departed_spent_cents),money(s.departed_wallet_cents)),
            VisitorLabel::Rules=>"One arrival per 30 simulation seconds, up to 64 active guests.\nTracked visits last 10 simulation minutes; existing larger crowds are preserved.\nDepartures finish current rides/services, then walk to entry. Space pauses activity.\nWallets are visitor money, not park income. Receipts and reviews stay after departure.".into(),
            VisitorLabel::Selected=>view.selected_guest.map_or_else(||"No active guest selected. Choose one in Guest list first.".into(),|id|format!("Selected Guest #{}: {}\n{} leaving / {} finishing ride or service visits.",id.0,if sim.0.guest_is_leaving(id){"leaving after current visit"}else{"visiting"},s.leaving,s.finishing_visits)),
            VisitorLabel::Notice=>view.visitor_notice.clone().unwrap_or_else(||"Arrivals are opt-in. If entry is removed, departures wait for a new entry.".into()),
        };
    }
}
