//! A playable, paid-for park template using ordinary construction and operation.
use crate::{
    Park, PathDirection, PathSlope, RideModel, RidePaint, RidePalettePreset, SceneryKind,
    ServiceKind, Speed, StaffRole, Terrain, TerrainBrushKind, TilePos, WeatherPolicy,
};

pub const DEMO_PARK_NAME: &str = "Sunhaven Gardens";
pub const DEMO_PARK_SIZE: u32 = 48;
pub const DEMO_RIDES: [RideModel; 7] = [
    RideModel::ClassicCoaster,
    RideModel::LogFlume,
    RideModel::ParkRailway,
    RideModel::Carousel,
    RideModel::TeaCups,
    RideModel::FerrisWheel,
    RideModel::PirateShip,
];

/// A fresh independent sandbox. Construction is charged to its opening budget.
/// All gates, queues, staff and guests use the same rules as player-built parks.
pub fn build_demo_park() -> Result<Park, String> {
    let mut park = Park::new(DEMO_PARK_SIZE, DEMO_PARK_SIZE);
    park.apply_terrain_profile(crate::TerrainProfile::Flat);
    for tile in &mut park.tiles {
        tile.terrain = Terrain::Grass;
    }
    park.set_starting_budget(1_500_000)?;
    park.set_sandbox_research(true);
    park.enable_management();
    park.enable_park_life();
    park.set_weather_policy(WeatherPolicy::Sunny);

    for y in [7, 21, 31, 41] {
        for x in 5..=45 {
            path(&mut park, x, y)?;
        }
    }
    for x in [5, 32, 45] {
        for y in 8..=40 {
            path(&mut park, x, y)?;
        }
    }
    // The railway's second station returns to the east promenade.
    for x in 33..=41 {
        path(&mut park, x, 18)?;
    }

    let sites = [
        (
            RideModel::ClassicCoaster,
            TilePos { x: 6, y: 10 },
            "Sunburst Coaster",
        ),
        (
            RideModel::LogFlume,
            TilePos { x: 20, y: 10 },
            "Willow Falls",
        ),
        (
            RideModel::ParkRailway,
            TilePos { x: 34, y: 10 },
            "Sunhaven Railway",
        ),
        (
            RideModel::Carousel,
            TilePos { x: 8, y: 24 },
            "Garden Carousel",
        ),
        (RideModel::TeaCups, TilePos { x: 34, y: 24 }, "Blossom Cups"),
        (
            RideModel::FerrisWheel,
            TilePos { x: 8, y: 34 },
            "Golden Wheel",
        ),
        (
            RideModel::PirateShip,
            TilePos { x: 34, y: 34 },
            "Marigold Voyager",
        ),
    ];
    for (index, (model, origin, name)) in sites.into_iter().enumerate() {
        let id = park
            .build_catalogue_ride(model, origin, 0, 0)
            .map_err(|e| e.to_string())?;
        park.set_ride_name(id, name).map_err(|e| e.to_string())?;
        let entrance_x = origin.x + u32::from(model.tracked());
        park.place_ride_entrance(
            TilePos {
                x: entrance_x,
                y: origin.y - 1,
            },
            0,
            PathDirection::South,
            id,
        )
        .map_err(|e| e.to_string())?;
        park.build_paid_queue_path(
            TilePos {
                x: entrance_x,
                y: origin.y - 2,
            },
            0,
            PathSlope::Flat,
            id,
        )
        .map_err(|e| e.to_string())?;
        if model.transport() {
            park.place_ride_exit(
                TilePos {
                    x: origin.x + 7,
                    y: origin.y + 7,
                },
                0,
                PathDirection::North,
                id,
            )
            .map_err(|e| e.to_string())?;
        } else {
            let x = entrance_x + 2;
            park.place_ride_exit(TilePos { x, y: origin.y - 1 }, 0, PathDirection::South, id)
                .map_err(|e| e.to_string())?;
            path(&mut park, x, origin.y - 2)?;
        }
        let palette = if model == RideModel::LogFlume {
            RidePalettePreset::Seaside
        } else if model == RideModel::TeaCups {
            RidePalettePreset::Twilight
        } else {
            RidePalettePreset::Harvest
        };
        park.apply_ride_palette_preset(id, palette)
            .map_err(|e| e.to_string())?;
        if model == RideModel::ClassicCoaster {
            let mut appearance = park.ride_appearance(id).ok_or("Missing coaster paint")?;
            appearance.structure = RidePaint::Coral;
            appearance.vehicles = RidePaint::Sunflower;
            park.set_ride_appearance(appearance)
                .map_err(|e| e.to_string())?;
        }
        park.set_ride_ticket_price(id, if index < 2 { 200 } else { 100 })
            .map_err(|e| e.to_string())?;
        park.set_ride_open_checked(id, true)
            .map_err(|e| format!("{name}: {e}"))?;
    }

    for (kind, x) in ServiceKind::ALL
        .into_iter()
        .zip([8, 11, 14, 17, 30, 33, 36, 39])
    {
        let id = park
            .place_service(TilePos { x, y: 20 }, 0, PathDirection::South, kind)
            .map_err(|e| e.to_string())?;
        park.set_service_price(
            id,
            match kind {
                ServiceKind::Toilet | ServiceKind::Information | ServiceKind::FirstAid => 0,
                ServiceKind::Rest => 50,
                ServiceKind::Souvenir => 200,
                _ => 100,
            },
        )
        .map_err(|e| e.to_string())?;
        park.order_facility_stock(id, 40)?;
    }

    // A landscaped lake separates the quieter western gardens from the east rides.
    for y in 23..=28 {
        for x in 19..=28 {
            let kind = if x == 19 || x == 28 || y == 23 || y == 28 {
                TerrainBrushKind::Sand
            } else {
                TerrainBrushKind::Water
            };
            park.apply_terrain_brush(TilePos { x, y }, 0, kind)
                .map_err(|e| e.to_string())?;
        }
    }
    for y in [3, 44] {
        for x in (4..=44).step_by(3) {
            tree(&mut park, x, y)?;
        }
    }
    for x in [2, 46] {
        for y in (7..=40).step_by(3) {
            tree(&mut park, x, y)?;
        }
    }
    for (x, y) in [
        (12, 24),
        (14, 26),
        (10, 29),
        (16, 24),
        (16, 29),
        (30, 24),
        (30, 28),
        (16, 34),
        (19, 35),
        (22, 35),
        (25, 35),
        (28, 35),
        (14, 39),
        (19, 39),
        (25, 39),
        (29, 39),
        (18, 11),
        (18, 14),
        (30, 11),
        (30, 15),
        (7, 18),
        (11, 18),
        (15, 18),
        (9, 39),
        (40, 39),
    ] {
        tree(&mut park, x, y)?;
    }
    for x in 20..=27 {
        park.build_scenery(TilePos { x, y: 22 }, SceneryKind::FlowerBed)
            .map_err(|e| e.to_string())?;
        park.build_scenery(TilePos { x, y: 29 }, SceneryKind::FlowerBed)
            .map_err(|e| e.to_string())?;
    }
    for y in [6, 22, 30, 40, 42] {
        for x in [6, 12, 15, 18, 24, 27, 30, 39, 42] {
            if park
                .quote_scenery(TilePos { x, y }, SceneryKind::FlowerBed)
                .is_ok()
            {
                park.build_scenery(TilePos { x, y }, SceneryKind::FlowerBed)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    let furniture: Vec<_> = park
        .objects()
        .iter()
        .filter(|o| {
            o.is_public_path()
                && (o.pos.y == 21 || o.pos.y == 31 || o.pos.y == 41)
                && o.pos.x % 4 == 1
        })
        .map(|o| o.id)
        .collect();
    for (index, id) in furniture.into_iter().enumerate() {
        park.toggle_path_bench(id).map_err(|e| e.to_string())?;
        park.toggle_ground_bin(id).map_err(|e| e.to_string())?;
        if index % 3 == 0 {
            park.toggle_path_shelter(id)?;
        }
    }
    let entry = park
        .objects()
        .iter()
        .find(|p| p.is_public_path() && p.pos == TilePos { x: 5, y: 41 })
        .ok_or("Demo entry is missing")?
        .id;
    park.set_visitor_entry(entry).map_err(|e| e.to_string())?;
    park.set_park_entry_price(100).map_err(|e| e.to_string())?;
    park.set_visitor_arrivals(true).map_err(|e| e.to_string())?;
    park.start_flyer_campaign().map_err(|e| e.to_string())?;
    // A short opening period puts the first real guests into the park.
    park.crowd_seed = Some(0x5355_4e48_4156_454e);
    park.crowd_rng = 0x5355_4e48_4156_454e;
    for role in StaffRole::ALL {
        park.hire_park_staff(role)?;
    }
    for _ in 0..9000 {
        park.advance(Speed::Normal);
    }
    park.to_save_bytes().map_err(|e| e.to_string())?;
    Ok(park)
}

fn path(park: &mut Park, x: u32, y: u32) -> Result<(), String> {
    if !park
        .objects()
        .iter()
        .any(|p| p.is_public_path() && p.pos == TilePos { x, y })
    {
        park.build_paid_path(TilePos { x, y }, 0, PathSlope::Flat)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn tree(park: &mut Park, x: u32, y: u32) -> Result<(), String> {
    park.build_scenery(TilePos { x, y }, SceneryKind::Tree)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_is_connected_paid_and_playable_after_reload() {
        let mut park = build_demo_park().unwrap();
        assert_eq!(park.all_rides().count(), 7);
        assert_eq!(park.services().len(), 8);
        assert_eq!(park.park_staff().len(), 5);
        assert!(park.guests().len() >= 9);
        for ride in park.all_rides() {
            assert!(park.ride_is_open(ride.id));
            assert!(park.ride_opening_status(ride.id).is_ok());
        }
        assert!(park.finance_summary().construction_cents > 0);
        assert!(park.cash_cents() > 500_000);
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        for _ in 0..18_000 {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
        }
        assert_eq!(park, loaded);
        for ride in park.all_rides() {
            assert!(ride.completed > 0, "{:?} needs a completed visit", ride.id);
            assert!(ride.revenue_cents > 0, "{:?} needs a paid visit", ride.id);
        }
        assert!(park.services().iter().any(|s| s.served > 0));
        assert_eq!(park.max_terrain_height(), 0);
    }
}
