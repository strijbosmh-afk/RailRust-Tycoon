//! Full campaign and portable scenario continuation through player-facing factories.
use super::*;
use crate::{campaign::Campaign, front_end, scenario_store::ScenarioStore};
use park_sim::{
    CampaignScenario, ChallengeDefinition, Objective, ObjectiveMetric, PathDirection, PathSlope,
    ServiceKind, WeatherPolicy,
};

fn facilities(park: &mut Park) -> Result<(), String> {
    for kind in [
        ServiceKind::Food,
        ServiceKind::Drink,
        ServiceKind::Toilet,
        ServiceKind::Rest,
    ] {
        let paths: Vec<_> = park
            .objects()
            .iter()
            .filter(|p| p.is_public_path())
            .copied()
            .collect();
        let mut built = false;
        'find: for path in paths {
            for (dx, dy, facing) in [
                (0, -1, PathDirection::South),
                (1, 0, PathDirection::West),
                (0, 1, PathDirection::North),
                (-1, 0, PathDirection::East),
            ] {
                let x = i64::from(path.pos.x) + dx;
                let y = i64::from(path.pos.y) + dy;
                if x < 0 || y < 0 {
                    continue;
                }
                if park
                    .place_service(
                        TilePos {
                            x: x as u32,
                            y: y as u32,
                        },
                        path.height,
                        facing,
                        kind,
                    )
                    .is_ok()
                {
                    built = true;
                    break 'find;
                }
            }
        }
        if !built {
            return Err(format!("Could not build {kind}"));
        }
    }
    Ok(())
}
fn prepare(park: &mut Park) -> Result<(), String> {
    park.borrow_loan(100_000).map_err(|e| e.to_string())?;
    let id = park.ride().ok_or("Starter coaster missing")?.id;
    park.set_ride_open(id, false).map_err(|e| e.to_string())?;
    park.place_ride_entrance(TilePos { x: 8, y: 11 }, 0, PathDirection::North, id)
        .map_err(|e| e.to_string())?;
    for y in 12..=15 {
        park.place_queue_path(TilePos { x: 8, y }, 0, PathSlope::Flat, id)
            .map_err(|e| e.to_string())?;
    }
    park.place_ride_exit(TilePos { x: 8, y: 9 }, 0, PathDirection::South, id)
        .map_err(|e| e.to_string())?;
    for _ in 0..40 {
        let plan = park.ride_access_plan(id).map_err(|e| e.to_string())?;
        if plan.exit.status == park_sim::RideExitStatus::Ready {
            break;
        }
        let next = plan
            .suggestions
            .iter()
            .find(|s| s.kind == park_sim::RideAccessBuildKind::PublicPath)
            .ok_or("Return path cannot be prepared")?;
        park.place_sloped_path(next.pos, next.height, next.slope)
            .map_err(|e| e.to_string())?;
    }
    if park.visitor_flow_stats().charging_rule != park_sim::ChargingRule::FreeRides {
        park.set_ride_ticket_price(id, 200)
            .map_err(|e| e.to_string())?;
    }
    if park.visitor_flow_stats().charging_rule != park_sim::ChargingRule::FreeEntry {
        park.set_park_entry_price(300).map_err(|e| e.to_string())?;
    }
    park.set_ride_open(id, true).map_err(|e| e.to_string())?;
    let entry = park
        .objects()
        .iter()
        .find(|p| p.is_public_path() && !park.path_neighbors(p.id).is_empty())
        .ok_or("Entry missing")?
        .id;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.set_visitor_arrivals(true).map_err(|e| e.to_string())?;
    park.set_weather_policy(WeatherPolicy::Sunny);
    facilities(park)?;
    for _ in 0..3 {
        park.hire_park_staff(park_sim::StaffRole::Entertainer)?;
    }
    Ok(())
}
pub(crate) fn editor_fixture() -> Result<Park, String> {
    let mut park = crate::new_game_park();
    park.set_starting_budget(250_000)?;
    park.enable_park_life();
    park.enable_management();
    prepare(&mut park)?;
    Ok(park)
}
pub(super) fn write(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Missing proof directory")?;
    fs::create_dir(store.directory()).map_err(|e| e.to_string())?;
    let mut campaign = Campaign::load(Some(store.clone()));
    for kind in CampaignScenario::ALL {
        let mut park = front_end::campaign_park(&campaign, kind)?;
        prepare(&mut park)?;
        if kind == CampaignScenario::ResearchRidge {
            park.set_research(
                park_sim::ResearchPriority::Thrill,
                park_sim::ResearchFunding::High,
            )?;
        }
        let mut won = false;
        for _ in 0..36_000 {
            park.advance(Speed::Normal);
            if park.management_status().research_spent.iter().sum::<u64>() >= 4_000 {
                let _ = park.set_research(
                    park_sim::ResearchPriority::Thrill,
                    park_sim::ResearchFunding::Paused,
                );
            }
            won = park
                .scenario_status()
                .is_some_and(|s| s.completed_tick.is_some())
                || park
                    .challenge_status()
                    .is_some_and(|s| s.completed_tick.is_some());
            if won {
                break;
            }
        }
        if !won {
            return Err(format!(
                "Campaign {} did not complete. {:?} {:?}",
                kind.label(),
                park.scenario_status(),
                park.challenge_status()
            ));
        }
        park.to_save_bytes().map_err(|e| e.to_string())?;
        campaign.record_scenario_completed(kind, park.ticks());
        if !campaign.progress.earned(kind) {
            return Err("Campaign award was not stored".into());
        }
        campaign = Campaign::load(Some(store.clone()));
    }
    let park = editor_fixture()?;
    let definition = ChallengeDefinition {
        name: "Lake workshop é".into(),
        goals: vec![Objective {
            metric: ObjectiveMetric::Arrivals,
            target: 9,
        }],
        deadline_ticks: Some(18_000),
        ..Default::default()
    };
    let doc = park_sim::ScenarioDocument::from_park(&park, definition)?;
    let scenarios = ScenarioStore::new(store.directory());
    let id = scenarios.save(None, &doc)?;
    crate::scenario_store::export_document(
        &store.directory().join("Lake workshop.parkscenario.json"),
        &doc,
    )?;
    let mut playable = scenarios.load(id)?.play()?;
    for _ in 0..1200 {
        playable.advance(Speed::Normal);
    }
    store.save(&playable).map_err(|e| e.to_string())?;
    write_new(
        &store.directory().join("expected-scenario-primary.json"),
        &playable.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    for _ in 0..3600 {
        playable.advance(Speed::Normal);
    }
    if playable
        .challenge_status()
        .is_none_or(|s| s.completed_tick.is_none())
    {
        return Err("Custom scenario did not win".into());
    }
    write_new(
        &store.directory().join("expected-scenario-after.json"),
        &playable.to_save_bytes().map_err(|e| e.to_string())?,
    )?;
    Ok(())
}
pub(super) fn verify(session: &SaveSession) -> Result<(), String> {
    let store = session.store.as_ref().ok_or("Missing proof directory")?;
    let original = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let campaign = Campaign::load(Some(store.clone()));
    if CampaignScenario::ALL
        .into_iter()
        .any(|s| !campaign.progress.earned(s))
    {
        return Err("Campaign awards did not survive restart".into());
    }
    let mut park = store.load().map_err(|e| e.to_string())?;
    if park != read_expected(&store.directory().join("expected-scenario-primary.json"))? {
        return Err("Scenario load changed state".into());
    }
    for _ in 0..3600 {
        park.advance(Speed::Normal);
    }
    if park != read_expected(&store.directory().join("expected-scenario-after.json"))? {
        return Err("Scenario continuation diverged".into());
    }
    let clean = store.directory().join("clean profile");
    let doc = crate::scenario_store::read_document(
        &store.directory().join("Lake workshop.parkscenario.json"),
    )?;
    let library = ScenarioStore::new(&clean);
    let id = library.save(None, &doc)?;
    library
        .load(id)?
        .play()?
        .to_save_bytes()
        .map_err(|e| e.to_string())?;
    if original != fs::read(store.save_path()).map_err(|e| e.to_string())? {
        return Err("Source save changed".into());
    }
    write_new(&store.directory().join("verification-scenario-tools.txt"),b"PASS\nAll six campaign scenarios won through the New Park factory.\nAwards survived restarts; custom scenario export/import to a clean profile passed.\nExact 3600-tick save continuation; source save preserved.\n")?;
    Ok(())
}
