//! Boardwalk selection, deadline success/failure and exact cross-process replay.
use super::*;
use crate::ui::scenarios::ScenarioCommand;
use crate::{Simulation, ui};
use bevy::prelude::*;
use park_sim::{BOARDWALK_ARRIVAL_GOAL, BOARDWALK_RIDE_GOAL, ChargingRule, ScenarioKind};

const SUCCESS_REPLAY_TICKS: usize = 4_500;
const FAILURE_REPLAY_TICKS: usize = 9_900;

fn command(park: &mut Park, command: ScenarioCommand) -> Result<String, String> {
    let mut app = App::new();
    app.insert_resource(Simulation(park.clone()))
        .insert_resource(ViewState {
            guests_open: true,
            scenario_view: true,
            speed: Speed::Paused,
            ..default()
        })
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                ui::button_actions,
                ui::scenarios::apply_commands,
                ui::guests::refresh,
                ui::scenarios::refresh,
            )
                .chain(),
        );
    app.update();
    let world = app.world_mut();
    let mut buttons = world.query::<(Entity, &ui::Action)>();
    let button = buttons
        .iter(world)
        .find_map(|(entity, action)| {
            matches!(action, ui::Action::Scenario(value) if *value == command).then_some(entity)
        })
        .ok_or("missing scenario button")?;
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    app.update();
    let notice = app
        .world()
        .resource::<ViewState>()
        .scenario_notice
        .clone()
        .ok_or("scenario command gave no result")?;
    let result = app.world().resource::<Simulation>().0.clone();
    app.update();
    if app.world().resource::<Simulation>().0 != result {
        return Err("held scenario button repeated".into());
    }
    *park = result;
    super::visitors::check(park)?;
    Ok(notice)
}

pub(crate) fn fixture() -> Result<Park, String> {
    let mut park = super::visitors::install()?;
    let notice = command(
        &mut park,
        ScenarioCommand::Start(ScenarioKind::BoardwalkSprint),
    )?;
    if !notice.contains("Boardwalk Sprint started")
        || park.visitor_flow_stats().charging_rule != ChargingRule::FreeRides
        || !park.all_rides().all(|ride| ride.ticket_price_cents == 0)
    {
        return Err("Boardwalk button did not lock free rides".into());
    }
    let before = park.clone();
    if !command(
        &mut park,
        ScenarioCommand::Start(ScenarioKind::MeadowChallenge),
    )?
    .contains("already active")
        || park != before
    {
        return Err("second challenge changed the active park".into());
    }
    if !super::visitors::command(
        &mut park,
        crate::ui::visitor_flow::VisitorCommand::CycleRule,
        None,
        None,
    )?
    .contains("requires free rides")
        || park != before
    {
        return Err("Boardwalk charging rule changed".into());
    }
    park.borrow_loan(100_000).map_err(|e| e.to_string())?;
    super::visitors::command(
        &mut park,
        crate::ui::visitor_flow::VisitorCommand::RunFlyers,
        None,
        None,
    )?;
    super::visitors::command(
        &mut park,
        crate::ui::visitor_flow::VisitorCommand::RunRadio,
        None,
        None,
    )?;
    for _ in 0..8_100 {
        park.advance(Speed::Normal);
    }
    let status = park
        .scenario_status()
        .ok_or("Boardwalk challenge disappeared")?;
    if status.kind != ScenarioKind::BoardwalkSprint
        || status.arrivals != 18
        || status.rides < BOARDWALK_RIDE_GOAL
        || status.completed_tick.is_some()
        || status.failed_tick.is_some()
        || status.deadline_tick != Some(18_000)
        || park.visitor_flow_stats().flyer_ticks_remaining != 900
        || park.visitor_flow_stats().radio_ticks_remaining != 900
    {
        return Err(format!("Boardwalk primary progress diverged: {status:?}"));
    }
    super::visitors::check(&park)?;
    Ok(park)
}

pub(crate) fn replay_success(park: &mut Park) -> Result<(), String> {
    for _ in 0..SUCCESS_REPLAY_TICKS {
        park.advance(Speed::Normal);
    }
    let status = park
        .scenario_status()
        .ok_or("Boardwalk challenge disappeared")?;
    if status.arrivals < BOARDWALK_ARRIVAL_GOAL
        || status.rides < BOARDWALK_RIDE_GOAL
        || status.completed_tick.is_none()
        || status.failed_tick.is_some()
        || status.completed_tick > status.deadline_tick
        || park.visitor_flow_stats().flyer_ticks_remaining != 0
        || park.visitor_flow_stats().radio_ticks_remaining != 0
    {
        return Err(format!("Boardwalk success diverged: {status:?}"));
    }
    super::visitors::check(park)
}

pub(crate) fn replay_failure(park: &mut Park) -> Result<(), String> {
    park.set_park_open(false);
    for _ in 0..FAILURE_REPLAY_TICKS {
        park.advance(Speed::Normal);
    }
    let status = park
        .scenario_status()
        .ok_or("Boardwalk challenge disappeared")?;
    if status.failed_tick != status.deadline_tick
        || status.completed_tick.is_some()
        || status.arrivals >= BOARDWALK_ARRIVAL_GOAL
    {
        return Err(format!("Boardwalk deadline failure diverged: {status:?}"));
    }
    super::visitors::check(park)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = fixture()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store
            .directory()
            .join("expected-scenarios-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    let mut failed = park.clone();
    replay_success(&mut park)?;
    replay_failure(&mut failed)?;
    write_new(
        &store
            .directory()
            .join("expected-scenarios-success.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    write_new(
        &store
            .directory()
            .join("expected-scenarios-failure.park.json"),
        &failed.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Boardwalk scenario outcomes written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let original = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let mut park = Park::new(32, 32);
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if view.speed != Speed::Paused
        || park
            != read_expected(
                &store
                    .directory()
                    .join("expected-scenarios-primary.park.json"),
            )?
    {
        return Err("loaded Boardwalk primary changed".into());
    }
    let mut failed = park.clone();
    replay_success(&mut park)?;
    replay_failure(&mut failed)?;
    if park
        != read_expected(
            &store
                .directory()
                .join("expected-scenarios-success.park.json"),
        )?
        || failed
            != read_expected(
                &store
                    .directory()
                    .join("expected-scenarios-failure.park.json"),
            )?
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != original
    {
        return Err("Boardwalk outcome replay diverged or rewrote primary".into());
    }
    write_new(
        &store.directory().join("verification-scenarios.txt"),
        b"Boardwalk success and timed failure resumed exactly from one saved primary: PASS\nOriginal primary slot unchanged: PASS\n",
    )?;
    println!("Boardwalk scenario outcomes verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boardwalk_buttons_and_both_saved_outcomes() {
        let park = fixture().unwrap();
        let mut success = park.clone();
        let mut failure = park;
        replay_success(&mut success).unwrap();
        replay_failure(&mut failure).unwrap();
    }
}
