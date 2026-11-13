//! Original, finite Phase 6 catalogue. Legacy engine families retain their saves.
use crate::{
    Park, RideId, RideKind, RideOperatingSettings, TilePos, TrackBuildKind, TrackDirection,
    TrackKind, TrackPieceSpec,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideCategory {
    Gentle,
    Thrill,
    Water,
    Transport,
    Coaster,
}
impl std::fmt::Display for RideCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Gentle => "Gentle",
            Self::Thrill => "Thrill",
            Self::Water => "Water",
            Self::Transport => "Transport",
            Self::Coaster => "Coasters",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RideModel {
    Carousel,
    FerrisWheel,
    TeaCups,
    ObservationTower,
    Swing,
    DropTower,
    PirateShip,
    LogFlume,
    RiverRapids,
    ParkRailway,
    Monorail,
    ClassicCoaster,
    SteelCoaster,
    InvertedCoaster,
}

impl RideModel {
    pub const ALL: [Self; 14] = [
        Self::Carousel,
        Self::FerrisWheel,
        Self::TeaCups,
        Self::ObservationTower,
        Self::Swing,
        Self::DropTower,
        Self::PirateShip,
        Self::LogFlume,
        Self::RiverRapids,
        Self::ParkRailway,
        Self::Monorail,
        Self::ClassicCoaster,
        Self::SteelCoaster,
        Self::InvertedCoaster,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Carousel => "Festival Carousel",
            Self::FerrisWheel => "Meadow Wheel",
            Self::TeaCups => "Whirling Cups",
            Self::ObservationTower => "Skyline Tower",
            Self::Swing => "Skyward Swing",
            Self::DropTower => "Beacon Drop",
            Self::PirateShip => "Corsair Ship",
            Self::LogFlume => "Timber Falls",
            Self::RiverRapids => "River Rapids",
            Self::ParkRailway => "Park Railway",
            Self::Monorail => "Vista Monorail",
            Self::ClassicCoaster => "Classic Coaster",
            Self::SteelCoaster => "Steel Launch Coaster",
            Self::InvertedCoaster => "Inverted Coaster",
        }
    }
    pub const fn category(self) -> RideCategory {
        match self {
            Self::Carousel | Self::FerrisWheel | Self::TeaCups | Self::ObservationTower => {
                RideCategory::Gentle
            }
            Self::Swing | Self::DropTower | Self::PirateShip => RideCategory::Thrill,
            Self::LogFlume | Self::RiverRapids => RideCategory::Water,
            Self::ParkRailway | Self::Monorail => RideCategory::Transport,
            Self::ClassicCoaster | Self::SteelCoaster | Self::InvertedCoaster => {
                RideCategory::Coaster
            }
        }
    }
    pub const fn base_kind(self) -> RideKind {
        match self {
            Self::Swing | Self::PirateShip => RideKind::Swing,
            Self::DropTower => RideKind::DropTower,
            Self::Carousel | Self::FerrisWheel | Self::TeaCups | Self::ObservationTower => {
                RideKind::Carousel
            }
            _ => RideKind::Coaster,
        }
    }
    pub const fn legacy(kind: RideKind) -> Self {
        match kind {
            RideKind::Coaster => Self::ClassicCoaster,
            RideKind::Carousel => Self::Carousel,
            RideKind::Swing => Self::Swing,
            RideKind::DropTower => Self::DropTower,
        }
    }
    pub fn tracked(self) -> bool {
        self.base_kind() == RideKind::Coaster
    }
    pub fn transport(self) -> bool {
        self.category() == RideCategory::Transport
    }
    pub const fn dimensions(self) -> (u32, u32) {
        match self {
            Self::FerrisWheel | Self::PirateShip => (5, 3),
            Self::TeaCups => (5, 5),
            Self::Carousel | Self::ObservationTower | Self::Swing | Self::DropTower => (3, 3),
            _ => (10, 7),
        }
    }
    pub const fn clearance(self) -> u8 {
        match self {
            Self::FerrisWheel => 13,
            Self::ObservationTower => 12,
            Self::DropTower => crate::carousel::DROP_TOWER_CLEARANCE,
            Self::PirateShip => 6,
            _ => 3,
        }
    }
    pub const fn flat_cost(self) -> u64 {
        match self {
            Self::Carousel => 30_000,
            Self::Swing => 35_000,
            Self::DropTower => 42_000,
            Self::FerrisWheel => 65_000,
            Self::TeaCups => 48_000,
            Self::ObservationTower => 58_000,
            Self::PirateShip => 62_000,
            _ => 0,
        }
    }
    pub const fn flat_capacity(self) -> usize {
        match self {
            Self::FerrisWheel => 6,
            Self::TeaCups => 12,
            Self::ObservationTower => 8,
            Self::PirateShip => 12,
            _ => 6,
        }
    }
    pub const fn seats_per_car(self) -> u8 {
        match self {
            Self::RiverRapids => 6,
            Self::LogFlume | Self::ParkRailway | Self::Monorail => 4,
            _ => 2,
        }
    }
    pub const fn max_cars(self) -> u8 {
        match self {
            Self::LogFlume | Self::RiverRapids => 1,
            _ => 3,
        }
    }
    pub fn max_trains(self) -> u8 {
        if self.transport() { 1 } else { 3 }
    }
    pub const fn challenge(self) -> u8 {
        match self {
            Self::Carousel => 20,
            Self::FerrisWheel => 15,
            Self::TeaCups => 35,
            Self::ObservationTower => 20,
            Self::Swing => 55,
            Self::DropTower => 72,
            Self::PirateShip => 60,
            Self::LogFlume => 42,
            Self::RiverRapids => 45,
            Self::ParkRailway | Self::Monorail => 10,
            Self::ClassicCoaster => 45,
            Self::SteelCoaster => 70,
            Self::InvertedCoaster => 75,
        }
    }
    pub const fn upkeep(self) -> u32 {
        match self {
            Self::FerrisWheel => 150,
            Self::TeaCups => 130,
            Self::ObservationTower => 160,
            Self::PirateShip => 180,
            Self::LogFlume => 220,
            Self::RiverRapids => 200,
            Self::ParkRailway => 180,
            Self::Monorail => 200,
            Self::SteelCoaster | Self::InvertedCoaster => 250,
            _ => 0,
        }
    }
    pub fn settings(self) -> RideOperatingSettings {
        RideOperatingSettings {
            cycle_ticks: match self {
                Self::FerrisWheel => 1440,
                Self::ObservationTower => 1080,
                Self::PirateShip => 900,
                _ => 720,
            },
            ..RideOperatingSettings::default()
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Carousel => "Canopy carousel with six horses and a gentle circular ride.",
            Self::FerrisWheel => {
                "Six guests share an upright observation gondola on a tall rotating wheel."
            }
            Self::TeaCups => "Six spinning cups orbit a patterned platform. Seats twelve.",
            Self::ObservationTower => "Eight-seat observation cabin rises slowly above the park.",
            Self::Swing => "Six-seat pendulum ride with a rising sweep.",
            Self::DropTower => "Six seats climb, pause, drop and brake at the loading deck.",
            Self::PirateShip => "Twelve-seat boat swings between two tall timber frames.",
            Self::LogFlume => "Four-seat logs follow a water channel, lift and splashdown.",
            Self::RiverRapids => "Six-seat rafts circulate through a level rapids channel.",
            Self::ParkRailway => {
                "Steam train carries guests from the entrance station to the far station."
            }
            Self::Monorail => "Elevated beam train connects two stations with enclosed coaches.",
            Self::ClassicCoaster => {
                "Timber-supported track with configurable cars and block sections."
            }
            Self::SteelCoaster => {
                "Tubular steel track, powered launch and banked or inverted elements."
            }
            Self::InvertedCoaster => {
                "Suspended seats travel below the rail with visible hanger assemblies."
            }
        }
    }
    pub fn allows(self, kind: TrackKind) -> bool {
        if matches!(self, Self::RiverRapids | Self::ParkRailway | Self::Monorail) {
            matches!(
                kind,
                TrackKind::Station | TrackKind::Straight | TrackKind::Curve | TrackKind::Brake
            )
        } else if self == Self::LogFlume {
            matches!(
                kind,
                TrackKind::Station
                    | TrackKind::Straight
                    | TrackKind::Curve
                    | TrackKind::Brake
                    | TrackKind::Lift
                    | TrackKind::Drop
                    | TrackKind::TransitionUp
                    | TrackKind::TransitionDown
            )
        } else {
            true
        }
    }
    pub fn layout(
        self,
        origin: TilePos,
        height: u8,
        rotation: u8,
    ) -> Result<Vec<TrackPieceSpec>, CatalogueError> {
        if !self.tracked() {
            return Err(CatalogueError::WrongFamily);
        }
        use TrackBuildKind as K;
        use TrackDirection as D;
        let mut route = vec![(1, 0, K::Station, D::East)];
        for x in 2..=8 {
            route.push((x, 0, K::Straight, D::East));
        }
        route.push((9, 0, K::CurveRight, D::East));
        for y in 1..=5 {
            route.push((9, y, K::Straight, D::South));
        }
        route.push((9, 6, K::CurveRight, D::South));
        for x in (1..=8).rev() {
            route.push((x, 6, K::Straight, D::West));
        }
        route.push((0, 6, K::CurveRight, D::West));
        for y in (1..=5).rev() {
            route.push((0, y, K::Straight, D::North));
        }
        route.push((0, 0, K::CurveRight, D::North));
        let hills = matches!(
            self,
            Self::ClassicCoaster | Self::SteelCoaster | Self::InvertedCoaster | Self::LogFlume
        );
        let mut elevation = height;
        let (width, depth) = self.dimensions();
        let mut specs = Vec::new();
        for (x, y, mut kind, mut direction) in route {
            if hills {
                if y == 0 && matches!(x, 4 | 5) {
                    kind = K::Lift;
                }
                if x == 9 && y == 2 {
                    kind = K::SteepDrop;
                }
            }
            if (x, y) == (9, 4) || (x, y) == (5, 6) || (x, y) == (0, 3) {
                kind = K::Brake;
            }
            if !self.transport() && y == 0 && matches!(x, 2 | 3) {
                kind = K::Station;
            }
            if self.transport() && (x, y) == (7, 6) {
                kind = K::Station;
            }
            if self == Self::SteelCoaster {
                if (x, y) == (6, 0) {
                    kind = K::Launch;
                }
                if (x, y) == (9, 0) {
                    kind = K::BankedRight;
                }
                if (x, y) == (7, 6) {
                    kind = K::HeartlineRoll;
                }
            }
            let (rx, ry) = match rotation % 4 {
                0 => (x, y),
                1 => (depth - 1 - y, x),
                2 => (width - 1 - x, depth - 1 - y),
                _ => (y, width - 1 - x),
            };
            for _ in 0..rotation % 4 {
                direction = match direction {
                    D::North => D::East,
                    D::East => D::South,
                    D::South => D::West,
                    D::West => D::North,
                };
            }
            let pos = TilePos {
                x: origin.x.checked_add(rx).ok_or(CatalogueError::OutsideMap)?,
                y: origin.y.checked_add(ry).ok_or(CatalogueError::OutsideMap)?,
            };
            let spec = kind.spec(pos, elevation, direction);
            elevation = spec.exit.height;
            specs.push(spec);
        }
        Ok(specs)
    }
}
impl std::fmt::Display for RideModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogueError {
    WrongFamily,
    OutsideMap,
    Construction(String),
}
impl std::fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongFamily => f.write_str("Choose a compatible ride family"),
            Self::OutsideMap => f.write_str("The ride extends outside the map"),
            Self::Construction(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for CatalogueError {}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogueQuote {
    pub cost_cents: u64,
    pub width: u32,
    pub height: u32,
    pub seats: usize,
}

impl Park {
    pub fn ride_model(&self, id: RideId) -> Option<RideModel> {
        Some(
            self.ride_by_id(id)?
                .model
                .unwrap_or(RideModel::legacy(self.ride_kind(id)?)),
        )
    }
    pub(crate) fn selected_ride_model(&self) -> RideModel {
        self.ride
            .as_ref()
            .and_then(|r| r.model)
            .unwrap_or(RideModel::ClassicCoaster)
    }
    pub fn catalogue_quote(
        &self,
        model: RideModel,
        origin: TilePos,
        height: u8,
        rotation: u8,
    ) -> Result<CatalogueQuote, CatalogueError> {
        if !self.ride_researched(model) {
            return Err(CatalogueError::Construction(format!(
                "Research {} before construction",
                model.name()
            )));
        }
        let err = |e: crate::PlacementError| CatalogueError::Construction(e.to_string());
        let (mut width, mut depth) = model.dimensions();
        if rotation % 2 == 1 {
            std::mem::swap(&mut width, &mut depth);
        }
        if self.next_ride_id == u64::MAX {
            return Err(CatalogueError::Construction(
                "Ride identity capacity reached".into(),
            ));
        }
        let cost = if model.tracked() {
            if (self.ride.is_none() && !self.track_draft.is_empty())
                || self
                    .other_coasters
                    .iter()
                    .any(|c| c.ride.is_none() && !c.track_draft.is_empty())
            {
                return Err(CatalogueError::Construction(
                    "Build or clear the unfinished tracked-ride draft first".into(),
                ));
            }
            if self.coaster_ids().len() >= crate::MAX_COASTER_RECORDS {
                return Err(CatalogueError::Construction(
                    "Tracked ride capacity reached".into(),
                ));
            }
            let specs = model.layout(origin, height, rotation)?;
            crate::trains::validate_specs_for_model(self, &specs, model)
                .map_err(|e| CatalogueError::Construction(e.to_string()))?;
            if self.objects.len() + specs.len() > crate::saves::MAX_OBJECTS {
                return Err(CatalogueError::Construction(
                    "Object capacity reached".into(),
                ));
            }
            for p in &specs {
                let (_, base, clearance) = crate::trains::envelope(*p);
                self.validate_object_placement(p.pos, crate::ObjectKind::Track, base, clearance)
                    .map_err(err)?;
            }
            specs
                .iter()
                .copied()
                .map(crate::track_piece_cost_cents)
                .sum()
        } else {
            self.catalogue_flat_quote(model, origin, height, rotation)
                .map_err(|e| CatalogueError::Construction(e.to_string()))?
        };
        if self.cash_cents < cost {
            return Err(CatalogueError::Construction(format!(
                "Need ${:.2} to build this ride",
                cost as f64 / 100.0
            )));
        }
        Ok(CatalogueQuote {
            cost_cents: cost,
            width,
            height: depth,
            seats: if model.tracked() {
                usize::from(model.seats_per_car() * model.max_cars())
            } else {
                model.flat_capacity()
            },
        })
    }
    pub fn build_catalogue_ride(
        &mut self,
        model: RideModel,
        origin: TilePos,
        height: u8,
        rotation: u8,
    ) -> Result<RideId, CatalogueError> {
        self.catalogue_quote(model, origin, height, rotation)?;
        let mut candidate = self.clone();
        let id = if model.tracked() {
            candidate
                .new_coaster_draft()
                .map_err(|e| CatalogueError::Construction(e.to_string()))?;
            if !candidate.track_draft.is_empty() {
                return Err(CatalogueError::Construction(
                    "Build or clear the unfinished tracked-ride draft first".into(),
                ));
            }
            let id = RideId(candidate.next_ride_id);
            candidate.next_ride_id = candidate
                .next_ride_id
                .checked_add(1)
                .ok_or(CatalogueError::OutsideMap)?;
            let mut ride = crate::Ride::new(id);
            ride.model = Some(model);
            candidate.ride = Some(ride);
            candidate.coaster_cars = model.max_cars();
            candidate
                .replace_track_draft(&model.layout(origin, height, rotation)?)
                .map_err(|e| CatalogueError::Construction(e.to_string()))?;
            candidate
                .apply_track_draft()
                .map_err(|e| CatalogueError::Construction(e.to_string()))?;
            id
        } else {
            candidate
                .place_catalogue_flat(model, origin, height, rotation)
                .map_err(|e| CatalogueError::Construction(e.to_string()))?
        };
        candidate
            .set_ride_name(id, model.name())
            .map_err(|e| CatalogueError::Construction(e.to_string()))?;
        candidate
            .set_ride_operating_settings(id, model.settings())
            .map_err(|e| CatalogueError::Construction(e.to_string()))?;
        let palette = match model.category() {
            RideCategory::Water => crate::RidePalettePreset::Seaside,
            RideCategory::Transport => crate::RidePalettePreset::Harvest,
            _ if model == RideModel::InvertedCoaster => crate::RidePalettePreset::Twilight,
            _ => crate::RidePalettePreset::Harvest,
        };
        candidate
            .apply_ride_palette_preset(id, palette)
            .map_err(|e| CatalogueError::Construction(e.to_string()))?;
        *self = candidate;
        Ok(id)
    }
}

pub(crate) fn optional_model<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<RideModel>, D::Error> {
    use serde::{Deserialize, de::IntoDeserializer};
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .map(|name| RideModel::deserialize(name.into_deserializer()))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PathDirection, PathSlope, ServiceKind, Speed};
    #[test]
    fn all_models_build_save_and_reject_overlaps_in_every_orientation() {
        for model in RideModel::ALL {
            for rotation in 0..4 {
                let mut park = Park::new(32, 32);
                park.borrow_loan(200_000).unwrap();
                let origin = TilePos { x: 5, y: 5 };
                let quote = park.catalogue_quote(model, origin, 0, rotation).unwrap();
                let id = park
                    .build_catalogue_ride(model, origin, 0, rotation)
                    .unwrap();
                assert_eq!(park.cash_cents(), 300_000 - quote.cost_cents);
                assert_eq!(park.ride_model(id), Some(model));
                let before = park.clone();
                assert!(
                    park.build_catalogue_ride(model, origin, 0, rotation)
                        .is_err()
                );
                assert_eq!(park, before);
                let loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
                assert_eq!(loaded, park, "{model} rotation {rotation}");
            }
        }
    }
    #[test]
    fn complete_catalogue_coexists_and_continues_exactly() {
        let mut park = crate::diagnostics::build_catalogue_showcase().unwrap();
        assert_eq!(park.all_rides().count(), 14);
        assert_eq!(park.services().len(), 8);
        for _ in 0..2400 {
            park.advance(Speed::Normal);
        }
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        for _ in 0..1800 {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
        }
        assert_eq!(park, loaded);
    }
    #[test]
    fn transport_guests_arrive_at_far_station_and_survive_close_and_reload() {
        for model in [RideModel::ParkRailway, RideModel::Monorail] {
            let mut park = Park::new(32, 32);
            park.borrow_loan(200_000).unwrap();
            let id = park
                .build_catalogue_ride(model, TilePos { x: 5, y: 7 }, 0, 0)
                .unwrap();
            for x in 2..=6 {
                park.place_path(TilePos { x, y: 4 }, 0).unwrap();
            }
            park.place_queue_path(TilePos { x: 6, y: 5 }, 0, PathSlope::Flat, id)
                .unwrap();
            park.place_ride_entrance(TilePos { x: 6, y: 6 }, 0, PathDirection::South, id)
                .unwrap();
            park.place_ride_exit(TilePos { x: 12, y: 14 }, 0, PathDirection::North, id)
                .unwrap();
            for x in 2..=12 {
                park.place_path(TilePos { x, y: 15 }, 0).unwrap();
            }
            for y in 5..15 {
                park.place_path(TilePos { x: 2, y }, 0).unwrap();
            }
            let entry = park
                .objects
                .iter()
                .find(|o| o.pos == TilePos { x: 2, y: 4 })
                .unwrap()
                .id;
            park.set_visitor_entry(entry).unwrap();
            park.set_visitor_arrivals(true).unwrap();
            park.set_ride_ticket_price(id, 100).unwrap();
            park.set_ride_open_checked(id, true).unwrap();
            let mut found = false;
            for _ in 0..8000 {
                park.advance(Speed::Normal);
                if park.coaster_train(id).unwrap().at_destination
                    && park.ride_by_id(id).unwrap().revenue_cents > 0
                {
                    park.set_visitor_arrivals(false).unwrap();
                    park.set_ride_open(id, false).unwrap();
                    found = true;
                    break;
                }
            }
            assert!(found, "{model} never delivered a paid passenger");
            let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
            for _ in 0..1500 {
                park.advance(Speed::Normal);
                loaded.advance(Speed::Normal);
            }
            assert_eq!(loaded, park);
            assert!(park.ride_by_id(id).unwrap().completed > 0);
            assert!(!park.coaster_train(id).unwrap().at_destination);
            assert_eq!(
                park.coaster_train(id).unwrap().phase,
                crate::TrainPhase::Closed
            );
        }
    }
    #[test]
    fn eight_facilities_have_distinct_costs_and_new_effects() {
        let mut needs = crate::GuestNeeds {
            hunger: 90,
            thirst: 80,
            nausea: 90,
            energy: 20,
            souvenir: 80,
            information: 80,
            ..Default::default()
        };
        for kind in ServiceKind::ALL {
            assert!(kind.build_cost_cents() > 0);
            assert!(kind.upkeep_cost_cents() > 0);
            assert!(kind.supply_cost_cents() > 0);
        }
        needs.relieve(ServiceKind::IceCream);
        assert_eq!((needs.hunger, needs.thirst), (45, 35));
        needs.relieve(ServiceKind::Souvenir);
        assert_eq!(needs.souvenir, 0);
        needs.relieve(ServiceKind::Information);
        assert_eq!(needs.information, 0);
        needs.relieve(ServiceKind::FirstAid);
        assert_eq!((needs.nausea, needs.energy), (0, 65));
    }
    #[test]
    fn every_ride_completes_paid_guest_visits_through_normal_access() {
        for model in RideModel::ALL {
            let mut park = Park::new(32, 32);
            park.apply_terrain_profile(crate::TerrainProfile::Flat);
            park.borrow_loan(200_000).unwrap();
            let origin = TilePos { x: 5, y: 7 };
            let id = park.build_catalogue_ride(model, origin, 0, 0).unwrap();
            for x in 2..=10 {
                park.place_path(TilePos { x, y: 4 }, 0).unwrap();
            }
            let entry_x = 5 + u32::from(model.tracked());
            park.place_queue_path(TilePos { x: entry_x, y: 5 }, 0, PathSlope::Flat, id)
                .unwrap();
            park.place_ride_entrance(TilePos { x: entry_x, y: 6 }, 0, PathDirection::South, id)
                .unwrap();
            if model.transport() {
                park.place_ride_exit(TilePos { x: 12, y: 14 }, 0, PathDirection::North, id)
                    .unwrap();
                for x in 2..=12 {
                    park.place_path(TilePos { x, y: 15 }, 0).unwrap();
                }
                for y in 5..15 {
                    park.place_path(TilePos { x: 2, y }, 0).unwrap();
                }
            } else {
                park.place_ride_exit(
                    TilePos {
                        x: entry_x + 2,
                        y: 6,
                    },
                    0,
                    PathDirection::South,
                    id,
                )
                .unwrap();
                park.place_path(
                    TilePos {
                        x: entry_x + 2,
                        y: 5,
                    },
                    0,
                )
                .unwrap();
            }
            let entry = park
                .objects
                .iter()
                .find(|o| o.pos == TilePos { x: 2, y: 4 })
                .unwrap()
                .id;
            park.set_visitor_entry(entry).unwrap();
            park.set_visitor_arrivals(true).unwrap();
            park.set_ride_ticket_price(id, 100).unwrap();
            park.set_ride_open_checked(id, true).unwrap();
            for _ in 0..12000 {
                park.advance(Speed::Normal);
                if park.ride_by_id(id).unwrap().completed > 0 {
                    break;
                }
            }
            let ride = park.ride_by_id(id).unwrap();
            assert!(ride.completed > 0, "{model} has no completed trip");
            assert!(ride.revenue_cents > 0, "{model} has no paid departure");
            let bytes = park.to_save_bytes().unwrap();
            let mut restored = Park::from_save_bytes(&bytes).unwrap();
            for _ in 0..180 {
                park.advance(Speed::Normal);
                restored.advance(Speed::Normal);
            }
            assert_eq!(restored, park, "{model} guest continuation");
        }
    }
    #[test]
    fn every_facility_serves_and_records_a_real_purchase() {
        for kind in ServiceKind::ALL {
            let mut park = Park::new(16, 16);
            park.apply_terrain_profile(crate::TerrainProfile::Flat);
            for x in 2..=6 {
                park.place_path(TilePos { x, y: 4 }, 0).unwrap();
            }
            let id = park
                .place_service(TilePos { x: 4, y: 5 }, 0, PathDirection::North, kind)
                .unwrap();
            let entry = park
                .objects
                .iter()
                .find(|o| o.pos == TilePos { x: 2, y: 4 })
                .unwrap()
                .id;
            park.set_visitor_entry(entry).unwrap();
            park.set_visitor_arrivals(true).unwrap();
            for _ in 0..1000 {
                park.advance(Speed::Normal);
                if !park.guests.is_empty() {
                    break;
                }
            }
            park.set_visitor_arrivals(false).unwrap();
            let guest = &mut park.guests[0];
            guest.needs = crate::GuestNeeds {
                hunger: 90,
                thirst: 90,
                toilet: 90,
                energy: 10,
                nausea: 90,
                souvenir: 90,
                information: 90,
            };
            for _ in 0..1000 {
                park.advance(Speed::Normal);
                if park.service(id).unwrap().served > 0 {
                    break;
                }
            }
            let service = park.service(id).unwrap();
            assert!(service.served > 0, "{kind} did not serve");
            assert!(service.supplies_cents > 0);
            assert_eq!(service.revenue_cents, u64::from(kind.default_price_cents()));
            let restored = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
            assert_eq!(restored, park);
        }
    }
    #[test]
    fn old_headers_refuse_catalogue_content_and_wrong_family_models() {
        let mut park = Park::new(32, 32);
        let id = park
            .build_catalogue_ride(RideModel::TeaCups, TilePos { x: 5, y: 5 }, 0, 0)
            .unwrap();
        let bytes = park.to_save_bytes().unwrap();
        let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        json["schema_version"] = 47.into();
        json["content_version"] = 47.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
        park.ride_by_id_mut(id).unwrap().model = Some(RideModel::Monorail);
        assert!(park.to_save_bytes().is_err());
    }
}
