use crate::*;

fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(bytes, loaded.to_save_bytes().unwrap());
    loaded
}

#[test]
fn large_owned_park_build_operate_reload_and_demolish_preserves_guests_and_money() {
    let mut park = Park::new(128, 128);
    assert!(park.prepare_large_sandbox_land());
    let purchased = TilePos { x: 80, y: 70 };
    assert!(!park.owns_land(purchased));
    let before = park.clone();
    assert!(park.build_scenery(purchased, SceneryKind::Tree).is_err());
    assert_eq!(park, before);
    park.buy_land(purchased).unwrap();
    park.apply_terrain_brush(purchased, 0, TerrainBrushKind::Grass)
        .unwrap();
    let tree = park.build_scenery(purchased, SceneryKind::Tree).unwrap();
    for x in 52..=56 {
        park.apply_terrain_brush(TilePos { x, y: 64 }, 0, TerrainBrushKind::Raise)
            .unwrap();
    }
    let mut paths = Vec::new();
    for x in 0..=65 {
        let (height, slope) = match x {
            51 => (0, PathSlope::East),
            52..=56 => (1, PathSlope::Flat),
            57 => (0, PathSlope::West),
            _ => (0, PathSlope::Flat),
        };
        paths.push(
            park.build_paid_path(TilePos { x, y: 64 }, height, slope)
                .unwrap(),
        );
    }
    for y in 65..=67 {
        park.build_paid_path(TilePos { x: 60, y }, 0, PathSlope::Flat)
            .unwrap();
    }
    let ride = park.place_carousel(TilePos { x: 64, y: 66 }, 0).unwrap();
    let entrance = park
        .place_ride_entrance(TilePos { x: 63, y: 67 }, 0, PathDirection::East, ride)
        .unwrap();
    for x in 61..=62 {
        park.build_paid_queue_path(TilePos { x, y: 67 }, 0, PathSlope::Flat, ride)
            .unwrap();
    }
    park.place_ride_exit(TilePos { x: 65, y: 65 }, 0, PathDirection::South, ride)
        .unwrap();
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
    assert_eq!(park.ride_exit_access(ride).status, RideExitStatus::Ready);
    assert_eq!(park.path_neighbors(paths[51]), vec![paths[50], paths[52]]);

    let decoration_path = park
        .build_paid_path(TilePos { x: 76, y: 76 }, 0, PathSlope::Flat)
        .unwrap();
    for kind in [
        PathAdditionKind::Bench,
        PathAdditionKind::Sign,
        PathAdditionKind::Fence,
    ] {
        park.toggle_path_addition(decoration_path, kind).unwrap();
    }
    park.set_visitor_entry(paths[0]).unwrap();
    park.set_visitor_arrivals(true).unwrap();
    park.set_ride_ticket_price(ride, 100).unwrap();
    park.set_ride_open(ride, true).unwrap();

    for _ in 0..24_000 {
        park.advance(Speed::Normal);
        if park.ride_by_id(ride).unwrap().completed > 0 {
            break;
        }
    }
    assert!(park.visitor_flow_stats().spawned > 0);
    assert!(park.ride_by_id(ride).unwrap().completed > 0);
    assert!(park.ride_by_id(ride).unwrap().revenue_cents > 0);
    let mut loaded = roundtrip(&park);
    for _ in 0..600 {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
    }
    assert_eq!(park, loaded);
    let cash = loaded.cash_cents();
    let refund = loaded.path_demolition_refund_cents(decoration_path)
        + loaded.decoration_refund_quote(decoration_path).unwrap();
    loaded.remove_object(decoration_path).unwrap();
    assert_eq!(loaded.cash_cents(), cash + refund);
    assert_eq!(loaded.remove_object(decoration_path), None);
    let cash = loaded.cash_cents();
    let refund = loaded.decoration_refund_quote(tree).unwrap();
    loaded.remove_object(tree).unwrap();
    assert_eq!(loaded.cash_cents(), cash + refund);
    loaded.set_visitor_arrivals(false).unwrap();
    loaded.set_ride_open(ride, false).unwrap();
    for _ in 0..6_000 {
        loaded.advance(Speed::Normal);
        if loaded.carousel_demolition_quote(ride).is_ok() {
            break;
        }
    }
    let cash = loaded.cash_cents();
    let refund = loaded.carousel_demolition_quote(ride).unwrap();
    loaded.remove_carousel(ride).unwrap();
    assert_eq!(loaded.cash_cents(), cash + refund);
    assert!(loaded.object(entrance).is_none());
    assert!(!loaded.ride_available(ride));
    roundtrip(&loaded);
}
