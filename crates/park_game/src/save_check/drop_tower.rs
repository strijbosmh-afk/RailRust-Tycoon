//! Fresh-process proof of two paid, independent flat rides and a lifted drop tower.
use super::*;
use park_sim::{PathDirection, PathSlope, RideId, RideVisitPhase};

const FUTURE_TICKS: usize = 3_600;

pub(crate) fn fixture() -> Result<(Park, RideId, RideId), String> {
    let mut park = Park::new(32, 32);
    let carousel = park
        .place_carousel(TilePos { x: 5, y: 15 }, 0)
        .map_err(|e| e.to_string())?;
    let drop_tower = park
        .place_drop_tower(TilePos { x: 15, y: 15 }, 0)
        .map_err(|e| e.to_string())?;
    for x in 2..=18 {
        park.place_path(TilePos { x, y: 12 }, 0)
            .map_err(|e| e.to_string())?;
    }
    for x in [2, 12] {
        for y in 13..=16 {
            park.place_path(TilePos { x, y }, 0)
                .map_err(|e| e.to_string())?;
        }
    }
    for x in [6, 16] {
        park.place_path(TilePos { x, y: 13 }, 0)
            .map_err(|e| e.to_string())?;
    }
    for (ride, west, north) in [(carousel, 5, 6), (drop_tower, 15, 16)] {
        park.place_queue_path(TilePos { x: west - 2, y: 16 }, 0, PathSlope::Flat, ride)
            .map_err(|e| e.to_string())?;
        park.place_ride_entrance(TilePos { x: west - 1, y: 16 }, 0, PathDirection::East, ride)
            .map_err(|e| e.to_string())?;
        park.place_ride_exit(TilePos { x: north, y: 14 }, 0, PathDirection::South, ride)
            .map_err(|e| e.to_string())?;
        park.set_ride_ticket_price(ride, 100)
            .map_err(|e| e.to_string())?;
        park.set_ride_open_checked(ride, true)
            .map_err(|e| e.to_string())?;
    }
    let entry = park
        .objects()
        .iter()
        .find(|o| o.is_public_path() && o.pos == (TilePos { x: 2, y: 12 }))
        .ok_or("missing entry path")?
        .id;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.set_visitor_arrivals(true).map_err(|e| e.to_string())?;
    Ok((park, carousel, drop_tower))
}

pub(crate) fn paid_drop_tower_running(park: &Park, drop_tower: RideId) -> bool {
    park.ride_is_running(drop_tower)
        && park
            .carousel(drop_tower)
            .is_some_and(|c| (300..=450).contains(&c.progress))
        && park.guests().iter().any(|guest| {
            guest.ride_visit.as_ref().is_some_and(|visit| {
                visit.ride == drop_tower
                    && visit.phase == RideVisitPhase::Riding
                    && visit
                        .passenger
                        .as_ref()
                        .is_some_and(|seat| seat.fare_paid && seat.departed)
            })
        })
}

pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("invalid --save-dir")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let (mut park, carousel, drop_tower) = fixture()?;
    let mut found = false;
    for _ in 0..60_000 {
        if park.ride_by_id(carousel).is_some_and(|r| r.completed > 0)
            && park.ride_by_id(drop_tower).is_some_and(|r| r.completed > 0)
            && paid_drop_tower_running(&park, drop_tower)
        {
            found = true;
            break;
        }
        park.advance(Speed::Normal);
    }
    if !found {
        return Err("both rides did not complete paid visits before a drop tower departure".into());
    }
    super::ride_choice::check(&park)?;
    let mut view = ViewState::default();
    if !persistence::perform_action(session, &mut park, &mut view, SaveAction::Save) {
        return Err(view.save_notice);
    }
    write_new(
        &store
            .directory()
            .join("expected-drop-tower-primary.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    super::ride_choice::check(&park)?;
    write_new(
        &store
            .directory()
            .join("expected-drop-tower-after.park.json"),
        &park.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    println!("Drop tower continuation proof written.");
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
    let expected = read_expected(
        &store
            .directory()
            .join("expected-drop-tower-primary.park.json"),
    )?;
    if park != expected || view.speed != Speed::Paused {
        return Err("drop tower ride mid-cycle load changed state".into());
    }
    let drop_tower = park
        .carousels()
        .iter()
        .find(|c| c.kind == park_sim::RideKind::DropTower)
        .ok_or("missing drop tower record")?
        .ride
        .id;
    if !paid_drop_tower_running(&park, drop_tower) {
        return Err("loaded drop tower is not carrying a paid passenger".into());
    }
    for _ in 0..FUTURE_TICKS {
        park.advance(Speed::Normal);
    }
    super::ride_choice::check(&park)?;
    let expected = read_expected(
        &store
            .directory()
            .join("expected-drop-tower-after.park.json"),
    )?;
    if park != expected || fs::read(store.save_path()).map_err(|e| e.to_string())? != original {
        return Err("drop tower replay diverged or rewrote primary slot".into());
    }
    write_new(&store.directory().join("verification-drop-tower.txt"), b"Two paid flat rides, independent receipts and lifted drop tower resumed after exact 3600-tick replay: PASS\nOriginal primary slot unchanged: PASS\n")?;
    println!("Drop tower continuation verified.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_dispatches_both_rides() {
        let (mut park, carousel, drop_tower) = fixture().unwrap();
        for _ in 0..60_000 {
            if park
                .ride_by_id(carousel)
                .is_some_and(|ride| ride.completed > 0)
                && paid_drop_tower_running(&park, drop_tower)
            {
                super::super::ride_choice::check(&park).unwrap();
                return;
            }
            park.advance(Speed::Normal);
        }
        panic!("both flat rides must earn revenue and drop tower must depart");
    }
}
