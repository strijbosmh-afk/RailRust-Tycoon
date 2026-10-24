use super::*;
use crate::TerrainProfile;

#[test]
fn demo_exit_return_path_can_be_completed_from_access_suggestions() {
    for end in 8..=17 {
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Terraced);
        park.install_crowd_fixture(42).unwrap();
        park.install_train_fixture().unwrap();
        let ride = park.ride().unwrap().id;
        park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, ride)
            .unwrap();
        for y in 12..=15 {
            park.place_queue_path(TilePos { x: 8, y }, 8, PathSlope::Flat, ride)
                .unwrap();
        }
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride)
            .unwrap();
        for x in 8..=end {
            park.place_path(TilePos { x, y: 8 }, 8).unwrap();
        }
        for step in 0..30 {
            let plan = park.ride_access_plan(ride).unwrap();
            if plan.exit.status == RideExitStatus::Ready {
                break;
            }
            let next = plan
                .suggestions
                .iter()
                .find(|s| s.kind == RideAccessBuildKind::PublicPath)
                .unwrap_or_else(|| panic!("end={end}, step={step}: {}", plan.advice));
            park.place_sloped_path(next.pos, next.height, next.slope)
                .unwrap();
            assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
        }
        assert_eq!(
            park.ride_exit_access(ride).status,
            RideExitStatus::Ready,
            "end={end}"
        );
    }
}

#[test]
fn bridge_uses_a_matching_slope_and_preserves_every_queue() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    let foreign = park.ride().unwrap().id;
    let ride = park.place_carousel(TilePos { x: 12, y: 22 }, 8).unwrap();
    park.place_ride_entrance(TilePos { x: 11, y: 23 }, 8, PathDirection::East, ride)
        .unwrap();
    park.place_queue_path(TilePos { x: 10, y: 23 }, 8, PathSlope::Flat, ride)
        .unwrap();
    park.place_path(TilePos { x: 9, y: 23 }, 8).unwrap();
    park.place_path(TilePos { x: 9, y: 22 }, 8).unwrap();
    park.place_sloped_path(TilePos { x: 9, y: 21 }, 7, PathSlope::South)
        .unwrap();
    park.place_path(TilePos { x: 9, y: 20 }, 7).unwrap();
    park.place_ride_exit(TilePos { x: 13, y: 21 }, 8, PathDirection::South, ride)
        .unwrap();
    park.place_path(TilePos { x: 13, y: 20 }, 8).unwrap();
    // A path may pass above or below this queue only with genuine clearance
    // and disjoint walking edges; joining it would invalidate both rides.
    park.place_queue_path(TilePos { x: 11, y: 19 }, 7, PathSlope::Flat, foreign)
        .unwrap();
    let before_foreign = park.ride_access(foreign);
    let mut used_slope = false;
    for _ in 0..20 {
        let before = park.clone();
        let plan = park.ride_access_plan(ride).unwrap();
        assert_eq!(park, before);
        let mut reversed = park.clone();
        reversed.objects.reverse();
        assert_eq!(reversed.ride_access_plan(ride).unwrap(), plan);
        if plan.exit.status == RideExitStatus::Ready {
            break;
        }
        let tile = plan
            .suggestions
            .iter()
            .find(|s| s.kind == RideAccessBuildKind::PublicPath)
            .unwrap();
        used_slope |= tile.slope != PathSlope::Flat;
        park.place_sloped_path(tile.pos, tile.height, tile.slope)
            .unwrap();
        assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
        assert_eq!(park.ride_access(foreign), before_foreign);
    }
    assert!(used_slope);
    assert_eq!(park.ride_exit_access(ride).status, RideExitStatus::Ready);
}

#[test]
fn enclosed_exit_has_no_misleading_partial_route() {
    let mut park = Park::new(16, 16);
    let ride = park.place_carousel(TilePos { x: 8, y: 8 }, 8).unwrap();
    park.place_ride_entrance(TilePos { x: 7, y: 9 }, 8, PathDirection::East, ride)
        .unwrap();
    park.place_queue_path(TilePos { x: 6, y: 9 }, 8, PathSlope::Flat, ride)
        .unwrap();
    park.place_path(TilePos { x: 5, y: 9 }, 8).unwrap();
    park.place_ride_exit(TilePos { x: 9, y: 7 }, 8, PathDirection::South, ride)
        .unwrap();
    park.place_path(TilePos { x: 9, y: 6 }, 8).unwrap();
    for (x, y) in [(8, 6), (10, 6), (9, 5)] {
        park.set_terrain_height(TilePos { x, y }, crate::MAX_TERRAIN_HEIGHT)
            .unwrap();
    }
    let plan = park.ride_access_plan(ride).unwrap();
    assert_eq!(plan.exit.status, RideExitStatus::PublicDisconnected);
    assert!(
        !plan
            .suggestions
            .iter()
            .any(|s| s.kind == RideAccessBuildKind::PublicPath)
    );
}
