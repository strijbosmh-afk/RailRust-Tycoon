//! Built scenery through primary/backup saves and continued simulation.
use super::*;
use park_sim::{ObjectKind, SceneryKind, TilePos};

fn clear_land(park: &Park, kind: SceneryKind) -> Result<TilePos, String> {
    (0..park.height())
        .flat_map(|y| (0..park.width()).map(move |x| TilePos { x, y }))
        .find(|pos| park.validate_scenery(*pos, kind).is_ok())
        .ok_or_else(|| "No clear land for scenery proof".into())
}

fn check(park: &Park, flowers: bool) -> Result<(), String> {
    super::names::check(park)?;
    let count = |kind| {
        park.objects()
            .iter()
            .filter(|object| object.kind == kind)
            .count()
    };
    if count(ObjectKind::SceneryTree) != 1 || count(ObjectKind::FlowerBed) != usize::from(flowers) {
        return Err("Scenery objects changed across save or replay".into());
    }
    Ok(())
}

fn replay(park: &mut Park) -> Result<(), String> {
    for tick in 0..3600 {
        park.advance(Speed::Normal);
        if tick % 300 == 0 {
            check(park, true)?;
            *park = Park::from_save_bytes(&park.to_save_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
    }
    check(park, true)
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Invalid save directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = super::names::install()?;
    let tree = clear_land(&park, SceneryKind::Tree)?;
    park.place_scenery(tree, SceneryKind::Tree)
        .map_err(|e| e.to_string())?;
    check(&park, false)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    super::visitors::mature(&mut park)?;
    let flowers = clear_land(&park, SceneryKind::FlowerBed)?;
    park.place_scenery(flowers, SceneryKind::FlowerBed)
        .map_err(|e| e.to_string())?;
    check(&park, true)?;
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
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    for (action, slot, flowers) in [
        (SaveAction::Load, "primary", true),
        (SaveAction::Backup, "backup", false),
    ] {
        if !persistence::perform_action(session, &mut park, &mut view, action) {
            return Err(view.save_notice);
        }
        if park != read_expected(&store.directory().join(format!("expected-{slot}.park.json")))? {
            return Err(format!("Scenery {slot} differs after restart"));
        }
        check(&park, flowers)?;
        if flowers {
            replay(&mut park)?;
            if park != read_expected(&store.directory().join("expected-after.park.json"))? {
                return Err("Scenery future replay diverged after restart".into());
            }
        }
    }
    if fs::read(store.save_path()).map_err(|e| e.to_string())? != primary
        || fs::read(store.backup_path()).map_err(|e| e.to_string())? != backup
    {
        return Err("Scenery verifier changed original save slots".into());
    }
    write_new(&store.directory().join("verification-scenery.txt"), b"PASS: tree and flower bed placement, strict primary and backup saves.\nPASS: paid visitors and ride names retained during 3600-tick fresh-process replay.\nPASS: source save slots unchanged.\n")
}
