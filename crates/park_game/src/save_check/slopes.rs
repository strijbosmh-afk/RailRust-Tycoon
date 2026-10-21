//! Fresh-process proof with guests actually travelling between path heights.
use super::*;
use park_sim::{GuestState, PathSlope};

const SOUTH: TilePos = TilePos { x: 13, y: 21 };
const LANDING: TilePos = TilePos { x: 13, y: 22 };
const EAST: TilePos = TilePos { x: 14, y: 22 };
const HIGH: TilePos = TilePos { x: 15, y: 22 };

fn install(park: &mut Park) -> Result<(), String> {
    for (pos, height, slope) in [
        (SOUTH, 8, PathSlope::South),
        (LANDING, 9, PathSlope::Flat),
        (EAST, 9, PathSlope::East),
        (HIGH, 10, PathSlope::Flat),
    ] {
        park.place_sloped_path(pos, height, slope)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn moving_on_slope(park: &Park) -> bool {
    park.guests().iter().any(|guest| {
        guest.state == GuestState::Walking
            && guest.progress == 7
            && guest
                .current
                .and_then(|id| park.object(id))
                .is_some_and(|path| path.path_slope != PathSlope::Flat)
    })
}

fn continue_slopes(park: &mut Park) -> Result<(), String> {
    let mut removed_id = None;
    let mut observed_slope_travel = false;
    let mut observed_high_landing = false;
    for tick in 0..3_600 {
        if tick == 1_200 {
            let id = park
                .objects()
                .iter()
                .find(|o| o.pos == SOUTH && o.kind == ObjectKind::Path)
                .ok_or("slope bridge missing")?
                .id;
            park.remove_object(id).ok_or("slope removal failed")?;
            removed_id = Some(id);
        }
        if tick == 1_500 {
            let replacement = park
                .place_sloped_path(SOUTH, 8, PathSlope::South)
                .map_err(|e| e.to_string())?;
            if Some(replacement) <= removed_id {
                return Err("slope identity reused".into());
            }
        }
        if tick == 1_800 {
            park.set_train_open(false);
        }
        if tick == 2_400 {
            park.set_train_open(true);
        }
        park.advance(Speed::Normal);
        observed_slope_travel |= moving_on_slope(park);
        observed_high_landing |= park.guests().iter().any(|g| {
            g.current
                .and_then(|id| park.object(id))
                .is_some_and(|p| p.pos == HIGH)
        });
    }
    if !observed_slope_travel || !observed_high_landing {
        return Err("continuation did not exercise ramp travel and the high landing".into());
    }
    Ok(())
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut park = demonstration_park();
    install(&mut park)?;
    // This is a bounded deterministic fixture search, never a wall-clock test.
    for _ in 0..18_000 {
        if moving_on_slope(&park) {
            break;
        }
        park.advance(Speed::Normal);
    }
    if !moving_on_slope(&park) {
        return Err("proof needs a guest partway along a slope".into());
    }
    let before = park.clone();
    if park.place_sloped_path(SOUTH, 8, PathSlope::North).is_ok()
        || park
            .place_sloped_path(TilePos { x: 3, y: 24 }, 30, PathSlope::East)
            .is_ok()
        || park != before
    {
        return Err("invalid slope placement mutated the park".into());
    }
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-backup.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    park.set_terrain_height(TilePos { x: 3, y: 24 }, 1)
        .map_err(|e| e.to_string())?;
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store.directory().join("expected-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    continue_slopes(&mut park)?;
    write_new(
        &store.directory().join("expected-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Sloped-path continuation proof written.");
    Ok(())
}

pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    let mut park = demonstration_park();
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Load) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-primary.park.json"))?
        || view.speed != Speed::Paused
        || !view.terrain_custom
        || !moving_on_slope(&park)
    {
        return Err("slope primary must restore exact mid-ramp state paused".into());
    }
    continue_slopes(&mut park)?;
    if park != read_expected(&store.directory().join("expected-after.park.json"))? {
        return Err("slope continuation differs from uninterrupted state".into());
    }
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Backup) {
        return Err(view.save_notice);
    }
    if park != read_expected(&store.directory().join("expected-backup.park.json"))?
        || view.speed != Speed::Paused
        || !moving_on_slope(&park)
    {
        return Err("slope backup did not restore exact mid-ramp state".into());
    }
    write_new(&store.directory().join("verification-slopes.txt"), format!("Parkbound {}\nFresh-process mid-ramp primary equality: PASS\nSlope guest traversal and high landing: PASS\nContinuation: PASS (3600 ticks + demolition/replacement and train commands)\nFresh slope identities and rejected placements: PASS\nExplicit mid-ramp backup restoration and paused loading: PASS\n", env!("CARGO_PKG_VERSION")).as_bytes())?;
    println!("Fresh-process sloped-path continuation and backup verified.");
    Ok(())
}
