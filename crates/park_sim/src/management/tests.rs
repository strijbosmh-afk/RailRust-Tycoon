use super::*;
use crate::{PathDirection, Speed, TilePos};

fn advance(park: &mut Park, ticks: usize) {
    for _ in 0..ticks {
        park.advance(Speed::Normal);
    }
}
fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(&loaded, park);
    assert_eq!(loaded.to_save_bytes().unwrap(), bytes);
    loaded
}
fn stocked_park() -> (Park, ServiceId) {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    let id = park
        .place_service(
            TilePos { x: 15, y: 21 },
            8,
            PathDirection::North,
            ServiceKind::Drink,
        )
        .unwrap();
    park.enable_management();
    park.set_automatic_stock(false);
    (park, id)
}
#[test]
fn research_spends_exact_receipts_unlocks_and_preserves_paused_progress() {
    let mut park = Park::new(32, 32);
    park.enable_management();
    assert!(park.ride_researched(RideModel::Carousel));
    assert!(!park.ride_researched(RideModel::Swing));
    let before = park.cash_cents();
    park.set_research(ResearchPriority::Thrill, ResearchFunding::High)
        .unwrap();
    advance(&mut park, 3600);
    assert_eq!(park.finance_summary().research_cents, 2000);
    park.set_research(ResearchPriority::Thrill, ResearchFunding::Paused)
        .unwrap();
    advance(&mut park, 1800);
    assert_eq!(park.finance_summary().research_cents, 2000);
    let mut park = roundtrip(&park);
    park.set_research(ResearchPriority::Thrill, ResearchFunding::High)
        .unwrap();
    advance(&mut park, 3600);
    assert!(park.ride_researched(RideModel::Swing));
    assert!(!park.ride_researched(RideModel::DropTower));
    assert_eq!(before - park.cash_cents(), RESEARCH_PROJECT_CENTS);
    roundtrip(&park);
}
#[test]
fn empty_stock_prevents_sales_and_prepaid_stock_is_consumed_once() {
    let (mut park, id) = stocked_park();
    advance(&mut park, 1800);
    assert_eq!(park.service(id).unwrap().served, 0);
    let cash = park.cash_cents();
    park.order_facility_stock(id, 20).unwrap();
    assert_eq!(cash - park.cash_cents(), 500);
    let supplies = park.service(id).unwrap().supplies_cents;
    advance(&mut park, 7200);
    let served = park.service(id).unwrap().served;
    assert!(served > 0);
    assert_eq!(park.facility_stocks()[0].remaining as u64, 20 - served);
    assert_eq!(park.service(id).unwrap().supplies_cents, supplies);
    roundtrip(&park);
}
#[test]
fn automatic_ordering_saves_exact_future_with_ledger_rollover() {
    let (mut park, _) = stocked_park();
    park.set_automatic_stock(true);
    park.set_research(ResearchPriority::Facilities, ResearchFunding::Low)
        .unwrap();
    advance(&mut park, 4000);
    let mut restored = roundtrip(&park);
    advance(&mut park, 4000);
    advance(&mut restored, 4000);
    assert_eq!(park, restored);
    assert!(park.finance_ledger().archived_entries > 0);
    assert!(park.management_reports().len() >= 4);
    roundtrip(&park);
}
#[test]
fn declined_orders_are_atomic_and_demolition_keeps_paid_stock_history() {
    let (mut park, id) = stocked_park();
    let before = park.clone();
    assert!(park.order_facility_stock(id, 101).is_err());
    assert_eq!(park, before);
    park.order_facility_stock(id, 20).unwrap();
    let object = park.service(id).unwrap().object.unwrap();
    park.remove_object(object).unwrap();
    assert!(park.order_facility_stock(id, 1).is_err());
    roundtrip(&park);
}
#[test]
fn old_parks_remain_compatible_and_management_rejects_old_headers() {
    let park = Park::new(32, 32);
    let mut value: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    value["schema_version"] = 49.into();
    value["content_version"] = 49.into();
    let mut old = Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(!old.management_status().enabled);
    assert!(old.ride_researched(RideModel::InvertedCoaster));
    old.enable_management();
    roundtrip(&old);
    let mut value: serde_json::Value =
        serde_json::from_slice(&old.to_save_bytes().unwrap()).unwrap();
    value["schema_version"] = 49.into();
    value["content_version"] = 49.into();
    assert!(Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn stock_and_research_tampering_are_rejected() {
    let (mut park, id) = stocked_park();
    park.order_facility_stock(id, 20).unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    for (field, value) in [("remaining", 21), ("purchased", 21), ("baseline_served", 1)] {
        let mut modified = original.clone();
        modified["park"]["management"]["stocks"][0][field] = value.into();
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&modified).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut modified = original;
    modified["park"]["management"]["research_spent"][0] = 4000.into();
    assert!(Park::from_save_bytes(&serde_json::to_vec(&modified).unwrap()).is_err());
}
#[test]
fn meadow_enforces_research_and_marketing_limits() {
    let mut park = Park::new(32, 32);
    park.start_scenario(crate::ScenarioKind::MeadowChallenge)
        .unwrap();
    assert!(
        park.set_research(ResearchPriority::Water, ResearchFunding::High)
            .is_err()
    );
    assert!(park.start_promotion(Promotion::Family).is_err());
    assert!(
        park.set_research(ResearchPriority::Water, ResearchFunding::Medium)
            .is_ok()
    );
    roundtrip(&park);
}
#[test]
fn research_locks_are_enforced_at_normal_construction_entry_points() {
    let mut park = Park::new(32, 32);
    park.enable_management();
    let before = park.clone();
    assert!(
        park.build_catalogue_ride(RideModel::Swing, TilePos { x: 2, y: 2 }, 0, 0)
            .unwrap_err()
            .to_string()
            .contains("Research")
    );
    assert_eq!(park, before);
    assert_eq!(
        park.place_service(
            TilePos { x: 2, y: 2 },
            0,
            PathDirection::North,
            ServiceKind::Souvenir
        ),
        Err(crate::PlacementError::ResearchRequired)
    );
    assert_eq!(park, before);
}
#[test]
fn staff_budget_suspends_and_resumes_paid_work() {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    park.hire_park_staff(crate::StaffRole::Handyman).unwrap();
    park.set_staff_budget(0).unwrap();
    advance(&mut park, 900);
    assert!(!park.park_staff()[0].working);
    park.set_staff_budget(100).unwrap();
    advance(&mut park, 1);
    assert!(park.park_staff()[0].working);
    assert_eq!(park.park_staff()[0].wages_paid_cents, 100);
    roundtrip(&park);
}

#[test]
fn a_loss_can_be_diagnosed_and_recovered_with_normal_price_and_funding_controls() {
    let (mut park, id) = stocked_park();
    park.set_automatic_stock(true);
    park.set_service_price(id, 0).unwrap();
    park.set_research(ResearchPriority::Water, ResearchFunding::High)
        .unwrap();
    let opening = park.management_report();
    advance(&mut park, 1800);
    let loss = park.management_report();
    assert!(loss.operating_profit_cents() < opening.operating_profit_cents());
    assert!(
        park.management_notices()
            .iter()
            .any(|n| n.contains("operating loss"))
    );
    park.set_research(ResearchPriority::Water, ResearchFunding::Paused)
        .unwrap();
    park.set_service_price(id, 500).unwrap();
    advance(&mut park, 7200);
    let recovered = park.management_report();
    assert!(recovered.operating_profit_cents() > loss.operating_profit_cents());
    assert!(park.service(id).unwrap().revenue_cents > 0);
    roundtrip(&park);
}

#[test]
fn historical_report_pages_remain_bounded() {
    let mut park = Park::new(32, 32);
    park.enable_management();
    advance(&mut park, 61 * 1800);
    assert_eq!(park.management_reports().len(), 60);
    let mut loaded = roundtrip(&park);
    advance(&mut park, 1800);
    advance(&mut loaded, 1800);
    assert_eq!(park, loaded);
}
