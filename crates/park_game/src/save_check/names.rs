//! Player ride names through real controls, both save slots and future replay.
use super::*;
use crate::{Simulation, ui};
use bevy::prelude::*;
use park_sim::RideId;
use ui::names::Command;

fn without_names(park: &Park) -> Result<serde_json::Value, String> {
    let mut data: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    data["park"]
        .as_object_mut()
        .ok_or("Missing saved park")?
        .remove("ride_names");
    Ok(data)
}

fn click(app: &mut App, command: Command) -> Result<(), String> {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &ui::Action)>();
    let button = query
        .iter(world)
        .find_map(|(entity, action)| (*action == ui::Action::RideName(command)).then_some(entity))
        .ok_or("Missing ride name control")?;
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    app.update();
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
    app.update();
    Ok(())
}

fn rename(park: &mut Park, id: RideId, name: &str) -> Result<(), String> {
    let before = without_names(park)?;
    let mut app = App::new();
    app.insert_resource(Simulation(park.clone()))
        .insert_resource(ViewState {
            rides_open: true,
            selected_ride: Some(id),
            speed: Speed::Paused,
            ..Default::default()
        })
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (ui::button_actions, ui::names::apply, ui::rides::refresh).chain(),
        );
    app.update();
    click(&mut app, Command::Focus)?;
    app.world_mut().resource_mut::<ViewState>().ride_name_draft = name.into();
    click(&mut app, Command::Apply)?;
    *park = app.world().resource::<Simulation>().0.clone();
    if park.ride_name(id) != Some(name) || without_names(park)? != before {
        return Err("Ride name action changed another park field or failed to rename".into());
    }
    check(park)
}

pub(super) fn check(park: &Park) -> Result<(), String> {
    super::colors::check(park)?;
    if Park::from_save_bytes(&park.to_save_bytes().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?
        != *park
    {
        return Err("Ride name changed on strict saved roundtrip".into());
    }
    Ok(())
}

pub(crate) fn install() -> Result<Park, String> {
    let mut park = super::colors::install()?;
    let coaster = park.ride().ok_or("Missing coaster")?.id;
    let carousel = park
        .carousels()
        .iter()
        .find(|ride| !ride.objects.is_empty())
        .ok_or("Missing carousel")?
        .ride
        .id;
    rename(&mut park, coaster, "Éclair Loop")?;
    rename(&mut park, carousel, "Moonlight Round")?;
    Ok(park)
}

fn replay(park: &mut Park) -> Result<(), String> {
    for tick in 0..3600 {
        park.advance(Speed::Normal);
        if tick % 300 == 0 {
            check(park)?;
            *park = Park::from_save_bytes(&park.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
    }
    check(park)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Invalid save directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = install()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    super::visitors::mature(&mut park)?;
    let coaster = park.ride().unwrap().id;
    rename(&mut park, coaster, "Morning Comet")?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    replay(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Invalid save directory")?;
    let original_primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let original_backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    for (action, slot) in [
        (SaveAction::Load, "primary"),
        (SaveAction::Backup, "backup"),
    ] {
        if !persistence::perform_action(session, &mut park, &mut view, action) {
            return Err(view.save_notice);
        }
        if park != read_expected(&store.directory().join(format!("expected-{slot}.park.json")))? {
            return Err(format!("Named {slot} differs after restart"));
        }
        check(&park)?;
        if slot == "primary" {
            replay(&mut park)?;
            if park != read_expected(&store.directory().join("expected-after.park.json"))? {
                return Err("Named future replay diverged after restart".into());
            }
        }
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != original_primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != original_backup
    {
        return Err("Name verifier changed original save slots".into());
    }
    write_new(
        &store.directory().join("verification-names.txt"),
        b"PASS: actual ride name controls; Unicode coaster and separate carousel names.\nPASS: non-name state unchanged, paid visitors retained, primary/backup exact.\nPASS: 3600-tick fresh-process replay; source slots unchanged.\n",
    )
}
