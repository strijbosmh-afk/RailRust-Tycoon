//! A dedicated challenge selection pane; sandbox is the unchanged default.
use super::*;
use park_sim::ScenarioKind;

#[derive(Component)]
pub(crate) struct ScenarioBody;
#[derive(Component)]
pub(crate) enum ScenarioLabel {
    Current,
    Notice,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScenarioCommand {
    Start(ScenarioKind),
    Conflict,
}

pub(crate) fn queue(view: &mut ViewState, command: ScenarioCommand) {
    view.scenario_pending = Some(if view.scenario_pending.is_some() {
        ScenarioCommand::Conflict
    } else {
        command
    });
}

fn card(p: &mut ChildSpawnerCommands, description: &str, kind: ScenarioKind) {
    p.spawn((
        Node {
            width: percent(100),
            height: px(100),
            padding: UiRect::all(px(10)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        BackgroundColor(Color::srgb_u8(232, 238, 224)),
        BorderRadius::all(px(5)),
    ))
    .with_children(|p| {
        label(p, description, 12.0, INK);
        button(
            p,
            &format!("Start {}", kind.label()),
            Action::Scenario(ScenarioCommand::Start(kind)),
            205.0,
        );
    });
}

pub(crate) fn setup(p: &mut ChildSpawnerCommands) {
    p.spawn((
        ScenarioBody,
        Node {
            display: Display::None,
            height: px(420),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(9),
            ..default()
        },
    ))
    .with_children(|p| {
        label(p, "Park challenges", 20.0, INK);
        label(
            p,
            "Choose once for this park. Sandbox stays active until you start a challenge.",
            12.0,
            MUTED,
        );
        card(
            p,
            "MEADOW CHALLENGE  /  No deadline\n20 new visitors + 5 rides + $750 cash. Free entry; no radio ads.",
            ScenarioKind::MeadowChallenge,
        );
        card(
            p,
            "BOARDWALK SPRINT  /  Ten simulation minutes\n24 new visitors + 12 rides + $900 cash. Free rides; ads allowed.",
            ScenarioKind::BoardwalkSprint,
        );
        for kind in [ScenarioLabel::Current, ScenarioLabel::Notice] {
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
        label(
            p,
            "Finished or failed challenges stay in this park. Load an earlier save to retry.",
            11.0,
            MUTED,
        );
    });
}

pub(crate) fn apply_commands(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    let Some(command) = view.scenario_pending.take() else {
        return;
    };
    if !view.guests_open
        || !view.scenario_view
        || view.scenario_view_pending
        || view.capture_active
        || view.modal_input_blocked
        || view.diagnostic_case.is_some()
    {
        return;
    }
    view.scenario_notice = Some(match command {
        ScenarioCommand::Start(kind) => match sim.0.start_scenario(kind) {
            Ok(()) => format!("{} started. Goals count from this moment.", kind.label()),
            Err(error) => format!("Challenge unchanged: {error}"),
        },
        ScenarioCommand::Conflict => "Choose one challenge action at a time.".into(),
    });
}

pub(crate) fn refresh(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut labels: Query<(&ScenarioLabel, &mut Text)>,
) {
    if !view.guests_open || !view.scenario_view {
        return;
    }
    for (kind, mut text) in &mut labels {
        text.0 = match kind {
            ScenarioLabel::Current => sim.0.scenario_status().map_or_else(
                || "Current park: Sandbox. No goals, deadline or locked charging rule.".into(),
                |s| {
                    let outcome = if s.completed_tick.is_some() {
                        "COMPLETED".to_string()
                    } else if s.failed_tick.is_some() {
                        "FAILED".to_string()
                    } else if let Some(limit) = s.deadline_tick {
                        let seconds = limit.saturating_sub(sim.0.ticks()) / TICKS_PER_SECOND;
                        format!("In progress / {:02}:{:02} left", seconds / 60, seconds % 60)
                    } else {
                        "In progress / no deadline".to_string()
                    };
                    format!(
                        "{}: {}\nNew visitors {} / {}  /  rides {} / {}  /  cash {} / {}",
                        s.kind.label(),
                        outcome,
                        s.arrivals,
                        s.goal_arrivals,
                        s.rides,
                        s.goal_rides,
                        money(s.cash_cents),
                        money(s.goal_cash_cents)
                    )
                },
            ),
            ScenarioLabel::Notice => view.scenario_notice.clone().unwrap_or_else(|| {
                "Set a connected public entry and open a ride; Visitor flow controls arrivals and ads."
                    .into()
            }),
        };
    }
}
