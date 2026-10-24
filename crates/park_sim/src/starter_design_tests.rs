//! Package content exercises the ordinary codec, construction and access rules.
use crate::*;

const DESIGNS: [(&str, &str, usize, u64); 3] = [
    (
        "Pocket Oval",
        include_str!("../../../designs/pocket-oval.parktrack.json"),
        16,
        27_000,
    ),
    (
        "Meadow Winder",
        include_str!("../../../designs/meadow-winder.parktrack.json"),
        18,
        30_000,
    ),
    (
        "Meadow Climb",
        include_str!("../../../designs/meadow-climb.parktrack.json"),
        24,
        42_000,
    ),
];

#[test]
fn original_starters_are_canonical_closed_affordable_and_run_in_every_rotation() {
    for (name, json, count, cost) in DESIGNS {
        let design = TrackDesign::from_json(json).unwrap();
        assert_eq!(design.name(), name);
        assert_eq!(design.pieces().len(), count);
        assert_eq!(design.to_json().unwrap(), json);
        for rotation in 0..4 {
            let mut park = Park::new(32, 32);
            let pieces = design
                .transformed(TilePos { x: 16, y: 16 }, 8, rotation)
                .unwrap();
            let before = park.clone();
            let quote = park.track_construction_quote_for(&pieces).unwrap();
            assert_eq!(park, before);
            assert_eq!(quote.construction_cents, cost);
            park.replace_track_draft(&pieces).unwrap();
            park.apply_track_draft().unwrap();
            assert_eq!(park.cash_cents(), INITIAL_PARK_CASH_CENTS - cost);
            let report = park.test_coaster().unwrap();
            assert!(report.moving_ticks > 0);
            park.set_train_open(true);
            for _ in 0..1800 {
                park.advance(Speed::Normal);
            }
            assert!(park.train().unwrap().laps > 0);
            assert_eq!(
                Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                park
            );
        }
    }
}

#[test]
fn each_starter_can_repair_its_exit_with_the_existing_path_helper_and_carry_paid_guests() {
    for (name, json, _, _) in DESIGNS {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(42).unwrap();
        let design = TrackDesign::from_json(json).unwrap();
        park.replace_track_draft(design.pieces()).unwrap();
        park.apply_track_draft().unwrap();
        let id = park.ride().unwrap().id;
        park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, id)
            .unwrap();
        for y in 12..=15 {
            park.place_queue_path(TilePos { x: 8, y }, 8, PathSlope::Flat, id)
                .unwrap();
        }
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, id)
            .unwrap();
        for step in 0..32 {
            let plan = park.ride_access_plan(id).unwrap();
            if plan.exit.status == RideExitStatus::Ready {
                break;
            }
            let next = plan
                .suggestions
                .iter()
                .find(|s| s.kind == RideAccessBuildKind::PublicPath)
                .unwrap_or_else(|| panic!("{name} step {step}: {}", plan.advice));
            park.place_sloped_path(next.pos, next.height, next.slope)
                .unwrap();
            assert_eq!(park.ride_access(id).status, RideAccessStatus::Ready);
        }
        assert_eq!(
            park.ride_exit_access(id).status,
            RideExitStatus::Ready,
            "{name}"
        );
        park.set_ride_ticket_price(id, 100).unwrap();
        park.set_train_open(true);
        for _ in 0..7200 {
            park.advance(Speed::Normal);
        }
        assert!(park.ride().unwrap().completed > 0, "{name}");
        assert!(park.ride().unwrap().revenue_cents > 0, "{name}");
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }
}
