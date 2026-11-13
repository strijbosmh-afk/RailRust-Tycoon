use super::*;
use crate::{PathDirection, Speed, TilePos};

fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let restored = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(&restored, park);
    restored
}
#[test]
fn sandbox_budgets_reconcile_before_and_after_construction_and_reports() {
    let mut park = Park::new(32, 32);
    park.set_starting_budget(500_000).unwrap();
    park.enable_management();
    park.place_service(
        TilePos { x: 2, y: 2 },
        0,
        PathDirection::North,
        ServiceKind::Food,
    )
    .unwrap();
    for _ in 0..1900 {
        park.advance(Speed::Normal);
    }
    park.set_starting_budget(100_000).unwrap();
    assert_eq!(
        park.cash_cents(),
        88_000
            - park.service(crate::ServiceId(1)).unwrap().upkeep_cents
            - park.service(crate::ServiceId(1)).unwrap().supplies_cents
    );
    assert_eq!(park.finance_summary().starting_cash_cents, 100_000);
    roundtrip(&park);
    let before = park.clone();
    assert!(park.set_starting_budget(10_000).is_err());
    assert_eq!(before, park);
}
#[test]
fn research_and_content_rules_block_normal_construction_and_loans() {
    let mut park = Park::new(32, 32);
    park.enable_management();
    park.set_sandbox_research(true);
    assert!(park.ride_researched(RideModel::InvertedCoaster));
    park.set_content_available(Some(RideModel::Swing), None, false);
    let before = park.clone();
    assert!(
        park.build_catalogue_ride(RideModel::Swing, TilePos { x: 2, y: 2 }, 0, 0)
            .is_err()
    );
    assert_eq!(before, park);
    park.set_loans_allowed(false);
    assert_eq!(park.loan_stats().available_cents, 0);
    assert!(park.borrow_loan(10_000).is_err());
    roundtrip(&park);
}
#[test]
fn operating_profit_goals_exclude_borrowed_funds_and_construction() {
    let mut park = Park::new(32, 32);
    park.start_custom_challenge(ChallengeDefinition {
        goals: vec![Objective {
            metric: ObjectiveMetric::OperatingProfit,
            target: 1,
        }],
        deadline_ticks: Some(900),
        ..Default::default()
    })
    .unwrap();
    park.borrow_loan(10_000).unwrap();
    park.place_service(
        TilePos { x: 2, y: 2 },
        0,
        PathDirection::North,
        ServiceKind::Food,
    )
    .unwrap();
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    let status = park.challenge_status().unwrap();
    assert_eq!(status.values, vec![0]);
    assert_eq!(status.failed_tick, Some(900));
    assert_eq!(status.completed_tick, None);
    roundtrip(&park);
}
#[test]
fn portable_scenario_preserves_land_rides_rules_and_exact_future() {
    let mut park = Park::new(32, 32);
    park.install_ground_starter_crowd(42).unwrap();
    park.install_ground_starter_coaster().unwrap();
    park.set_starting_budget(200_000).unwrap();
    park.set_sandbox_research(true);
    let doc = ScenarioDocument::from_park(&park, ChallengeDefinition::default()).unwrap();
    let loaded = ScenarioDocument::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert_eq!(loaded.template().unwrap(), park);
    let mut a = loaded.play().unwrap();
    for _ in 0..1300 {
        a.advance(Speed::Normal);
    }
    let mut b = roundtrip(&a);
    for _ in 0..1800 {
        a.advance(Speed::Normal);
        b.advance(Speed::Normal);
    }
    assert_eq!(a, b);
}
#[test]
fn goals_win_and_deadlines_fail_through_normal_ticks() {
    let mut park = Park::new(32, 32);
    let definition = ChallengeDefinition {
        goals: vec![Objective {
            metric: ObjectiveMetric::Cash,
            target: 50_000,
        }],
        ..Default::default()
    };
    park.start_custom_challenge(definition).unwrap();
    park.advance(Speed::Normal);
    assert_eq!(park.challenge_status().unwrap().completed_tick, Some(1));
    roundtrip(&park);
    park.clear_challenge();
    let definition = ChallengeDefinition {
        goals: vec![Objective {
            metric: ObjectiveMetric::Arrivals,
            target: 100,
        }],
        deadline_ticks: Some(900),
        ..Default::default()
    };
    park.start_custom_challenge(definition).unwrap();
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    assert!(park.challenge_status().unwrap().failed_tick.is_some());
    roundtrip(&park);
}
#[test]
fn malformed_scenarios_and_forged_campaign_awards_are_rejected() {
    let park = Park::new(32, 32);
    let doc = ScenarioDocument::from_park(&park, ChallengeDefinition::default()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&doc.to_bytes().unwrap()).unwrap();
    for (field, value) in [
        ("version", serde_json::json!(2)),
        ("goals", serde_json::json!([])),
    ] {
        let mut v = original.clone();
        if field == "goals" {
            v["definition"][field] = value;
        } else {
            v[field] = value;
        }
        assert!(ScenarioDocument::from_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    assert!(
        ScenarioDocument::from_park(&park, CampaignScenario::GrandFestival.definition()).is_err()
    );
}
#[test]
fn version_fifty_parks_keep_original_budget_and_new_rules_require_fifty_one() {
    let park = Park::new(32, 32);
    let mut v: serde_json::Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    v["schema_version"] = 50.into();
    v["content_version"] = 50.into();
    let mut park = Park::from_save_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(park.starting_cash_cents(), 100_000);
    park.set_starting_budget(200_000).unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    v["schema_version"] = 50.into();
    v["content_version"] = 50.into();
    assert!(Park::from_save_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
}
