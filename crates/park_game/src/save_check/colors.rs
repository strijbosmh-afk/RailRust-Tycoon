//! Real color controls, paid passengers, lifetime accounting and restart replay.
use super::*;
use crate::{Simulation, ui};
use bevy::prelude::*;
use park_sim::{RideId, RidePaint};
use ui::appearance::{Command, Part};
fn without_paint(p: &Park) -> Result<serde_json::Value, String> {
    let mut v: serde_json::Value =
        serde_json::from_slice(&p.to_save_bytes().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    v["park"]
        .as_object_mut()
        .ok_or("Missing saved park")?
        .remove("ride_paints");
    Ok(v)
}
fn click(app: &mut App, command: Command) -> Result<(), String> {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &ui::Action)>();
    let e = query
        .iter(world)
        .find_map(|(e, a)| (*a == ui::Action::RidePaint(command)).then_some(e))
        .ok_or("Missing paint control")?;
    *app.world_mut().get_mut::<Interaction>(e).unwrap() = Interaction::Pressed;
    app.update();
    let after = app.world().resource::<Simulation>().0.clone();
    app.update();
    if app.world().resource::<Simulation>().0 != after {
        return Err("Held paint button repeated".into());
    }
    *app.world_mut().get_mut::<Interaction>(e).unwrap() = Interaction::None;
    app.update();
    Ok(())
}
pub(crate) fn paint(p: &mut Park, id: RideId, steps: [u8; 3]) -> Result<(), String> {
    let before = without_paint(p)?;
    let mut app = App::new();
    app.insert_resource(Simulation(p.clone()))
        .insert_resource(ViewState {
            rides_open: true,
            selected_ride: Some(id),
            speed: Speed::Paused,
            ..Default::default()
        })
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                ui::button_actions,
                ui::appearance::apply,
                ui::rides::refresh,
                ui::appearance::refresh,
            )
                .chain(),
        );
    app.update();
    click(&mut app, Command::Show)?;
    if app.world().resource::<ViewState>().ride_tab != ui::rides::RideTab::Colors {
        return Err("Colors tab did not open".into());
    }
    for (part, steps) in [Part::Structure, Part::Supports, Part::Vehicles]
        .into_iter()
        .zip(steps)
    {
        for _ in 0..steps {
            click(&mut app, Command::Step(part, 1))?;
        }
    }
    *p = app.world().resource::<Simulation>().0.clone();
    if without_paint(p)? != before {
        return Err("Paint controls changed money, passengers, time or nonpaint state".into());
    }
    check(p)
}
pub(crate) fn check(p: &Park) -> Result<(), String> {
    super::visitors::check(p)?;
    let data = p.to_save_bytes().map_err(|e| e.to_string())?;
    let restored = Park::from_save_bytes(&data).map_err(|e| e.to_string())?;
    if *p != restored {
        return Err("Paint state changed on strict saved roundtrip".into());
    }
    Ok(())
}
pub(crate) fn install() -> Result<Park, String> {
    let mut p = super::visitors::install()?;
    super::visitors::paid_departure(&mut p)?;
    let coaster = p.ride().ok_or("Missing coaster")?.id;
    let carousel = p
        .carousels()
        .iter()
        .find(|c| !c.objects.is_empty())
        .ok_or("Missing carousel")?
        .ride
        .id;
    paint(&mut p, coaster, [1, 5, 2])?;
    paint(&mut p, carousel, [6, 3, 4])?;
    if p.ride_appearance(coaster).unwrap().structure != RidePaint::Coral
        || p.ride_appearance(carousel).unwrap().structure != RidePaint::Violet
    {
        return Err("Actual color choices differ".into());
    }
    Ok(p)
}
fn replay(p: &mut Park) -> Result<(), String> {
    for tick in 0..3600 {
        p.advance(Speed::Normal);
        if tick % 300 == 0 {
            check(p)?;
            *p = Park::from_save_bytes(&p.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
    }
    check(p)
}
pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Invalid save directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut p = install()?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut p, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &p.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    super::visitors::mature(&mut p)?;
    let id = p.ride().unwrap().id;
    paint(&mut p, id, [1, 0, 1])?;
    if !persistence::perform_action(session, &mut p, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &p.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    replay(&mut p)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &p.to_save_bytes().map_err(|e| e.to_string())?,
    )
}
pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Invalid save directory")?;
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut p = demonstration_park();
    let mut view = ViewState::default();
    for (action, name) in [
        (SaveAction::Load, "primary"),
        (SaveAction::Backup, "backup"),
    ] {
        if !persistence::perform_action(session, &mut p, &mut view, action) {
            return Err(view.save_notice);
        }
        if p != read_expected(&store.directory().join(format!("expected-{name}.park.json")))? {
            return Err("Painted primary/backup differs after restart".into());
        }
        check(&p)?;
        if name == "primary" {
            replay(&mut p)?;
            if p != read_expected(&store.directory().join("expected-after.park.json"))? {
                return Err("Painted replay diverged after restart".into());
            }
        }
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("Paint verifier changed source slots".into());
    }
    write_new(&store.directory().join("verification-colors.txt"),b"PASS: actual Colors controls, two independent ride palettes, paid passengers and unchanged nonpaint state.\nPASS: retired visitor receipts/reviews, loans, value and operating profits retained.\nPASS: primary/backup loading and exact 3600-tick future replay; original slots unchanged.\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn painted_rides_keep_paid_passengers_receipts_and_colors_after_real_restart() {
        let d = std::env::temp_dir().join(format!(
            "Parkbound colors {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["write-colors", "verify-colors"] {
            assert_eq!(
                run(vec![
                    "--save-check".into(),
                    mode.into(),
                    "--save-dir".into(),
                    d.clone().into_os_string()
                ]),
                Some(AppExit::Success)
            );
        }
        assert!(d.starts_with(std::env::temp_dir()));
        fs::remove_dir_all(d).unwrap();
    }
}
