//! Exact schema 18 body, before configurable train capacity.
use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "O: Deserialize<'de>, G: Deserialize<'de>, R: Deserialize<'de>"))]
pub(super) struct VersionEighteenParkData<O = SurfaceObject, G = Guest, R = crate::Ride> {
    #[serde(deserialize_with = "strict_object")]
    track_economy: crate::TrackEconomy,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_RIDE_RECORDS }>")]
    ride_operating_settings: Vec<crate::ride_operations::RideOperatingSettingsRecord>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_RIDE_RECORDS }>")]
    ride_dispatch: Vec<crate::ride_dispatch::RideDispatchRecord>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_TRACK_DRAFT_PIECES }>")]
    track_draft: Vec<TrackPieceSpec>,
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<O>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<G>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
    #[serde(deserialize_with = "required_object_option")]
    ride: Option<R>,
    next_ride_id: u64,
    cash_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    services: Vec<crate::Service>,
    next_service_id: u64,
    #[serde(deserialize_with = "strict_object")]
    finances: ride_costs::LegacyFinanceLedger,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_CAROUSEL_RECORDS }>")]
    carousels: Vec<crate::CarouselRide>,
}

impl VersionEighteenParkData {
    pub(super) fn migrate(self) -> ParkData {
        ParkData {
            ride_costs: Vec::new(),
            coaster_cars: crate::DEFAULT_COASTER_CARS,
            ride_dispatch: self.ride_dispatch,
            track_economy: self.track_economy,
            ride_operating_settings: self.ride_operating_settings,
            track_draft: self.track_draft,
            width: self.width,
            height: self.height,
            tiles: self.tiles,
            ticks: self.ticks,
            objects: self.objects,
            next_object_id: self.next_object_id,
            guests: self.guests,
            next_guest_id: self.next_guest_id,
            crowd_seed: self.crowd_seed,
            crowd_rng: self.crowd_rng,
            circuit: self.circuit,
            train: self.train,
            ride: self.ride,
            next_ride_id: self.next_ride_id,
            cash_cents: self.cash_cents,
            services: self.services,
            next_service_id: self.next_service_id,
            finances: self.finances.into(),
            carousels: self.carousels,
        }
    }
}
