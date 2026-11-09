//! Cross-process award, unlock and separate-park campaign proof.
use super::*;
use crate::ui::scenarios::ScenarioCommand;
use crate::{Simulation, campaign::Campaign, ui};
use bevy::prelude::*;
use park_sim::ScenarioKind;

fn press(park: &mut Park, campaign: Campaign, kind: ScenarioKind) -> Result<String, String> {
    let mut app = App::new();
    app.insert_resource(Simulation(park.clone()))
        .insert_resource(campaign)
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
                ui::scenarios::refresh,
            )
                .chain(),
        );
    app.update();
    let button = {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &ui::Action)>();
        query.iter(world).find_map(|(entity, action)| {
            matches!(action, ui::Action::Scenario(ScenarioCommand::Start(value)) if *value == kind).then_some(entity)
        }).ok_or("campaign challenge button missing")?
    };
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    app.update();
    let notice = app
        .world()
        .resource::<ViewState>()
        .scenario_notice
        .clone()
        .ok_or("campaign action gave no notice")?;
    let after = app.world().resource::<Simulation>().0.clone();
    app.update();
    if app.world().resource::<Simulation>().0 != after {
        return Err("held campaign button repeated".into());
    }
    *park = after;
    Ok(notice)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut sandbox = Park::new(32, 32);
    let before = sandbox.clone();
    let notice = press(
        &mut sandbox,
        Campaign::load(Some(store.clone())),
        ScenarioKind::BoardwalkSprint,
    )?;
    if !notice.contains("locked") || sandbox != before {
        return Err("Boardwalk was available before Meadow award".into());
    }
    let mut meadow = super::challenge::fixture()?;
    super::challenge::replay(&mut meadow)?;
    if meadow
        .scenario_status()
        .is_none_or(|status| status.completed_tick.is_none())
    {
        return Err("Meadow award proof did not win".into());
    }
    store.save(&meadow).map_err(|e| e.to_string())?;
    let mut campaign = Campaign::load(Some(store.clone()));
    campaign.record_completed(ScenarioKind::MeadowChallenge, meadow.ticks());
    if !campaign.unlocked(ScenarioKind::BoardwalkSprint) {
        return Err("Meadow win did not unlock Boardwalk".into());
    }
    write_new(
        &store.directory().join("expected-campaign.park.json"),
        &meadow.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Meadow award and Boardwalk unlock written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let original = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let mut campaign = Campaign::load(Some(store.clone()));
    if !campaign.progress.meadow_awarded || !campaign.unlocked(ScenarioKind::BoardwalkSprint) {
        return Err("fresh process lost Meadow award or Boardwalk unlock".into());
    }
    let meadow = store.load().map_err(|e| e.to_string())?;
    if meadow != read_expected(&store.directory().join("expected-campaign.park.json"))? {
        return Err("winning park save changed".into());
    }
    let award_bytes =
        fs::read(store.directory().join("campaign.json")).map_err(|e| e.to_string())?;
    campaign.record_completed(ScenarioKind::MeadowChallenge, meadow.ticks());
    if fs::read(store.directory().join("campaign.json")).map_err(|e| e.to_string())? != award_bytes
    {
        return Err("reloading the Meadow win awarded twice".into());
    }
    let mut new_park = Park::new(32, 32);
    let notice = press(&mut new_park, campaign, ScenarioKind::BoardwalkSprint)?;
    if !notice.contains("Boardwalk Sprint started")
        || new_park
            .scenario_status()
            .is_none_or(|status| status.kind != ScenarioKind::BoardwalkSprint)
        || fs::read(store.save_path()).map_err(|e| e.to_string())? != original
    {
        return Err("unlocked Boardwalk button failed or changed the previous park".into());
    }
    write_new(&store.directory().join("verification-campaign.txt"), b"Meadow award persisted to a fresh process: PASS\nBoardwalk button unlocked in a separate new park: PASS\nWinning park primary unchanged: PASS\n")?;
    println!("Campaign award and next-park unlock verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boardwalk_button_is_gated_until_campaign_award() {
        let store = crate::save_store::SaveStore::new(std::env::temp_dir().join(format!(
                "parkbound-campaign-button-{}-{} é",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
        let mut park = Park::new(32, 32);
        let before = park.clone();
        assert!(
            press(
                &mut park,
                Campaign::load(Some(store)),
                ScenarioKind::BoardwalkSprint
            )
            .unwrap()
            .contains("locked")
        );
        assert_eq!(park, before);
        assert!(
            press(
                &mut park,
                Campaign::preview(),
                ScenarioKind::BoardwalkSprint
            )
            .unwrap()
            .contains("started")
        );
    }
}
