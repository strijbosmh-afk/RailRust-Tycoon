//! Schema 15 stays strict so newer fields cannot be smuggled into legacy saves.
use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "O: Deserialize<'de>, G: Deserialize<'de>, R: Deserialize<'de>"))]
pub(super) struct VersionFifteenParkData<O = SurfaceObject, G = Guest, R = crate::Ride> {
    #[serde(deserialize_with = "strict_object")]
    track_economy: crate::TrackEconomy,
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
    finances: crate::FinanceLedger,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_CAROUSEL_RECORDS }>")]
    carousels: Vec<crate::CarouselRide>,
}

impl VersionFifteenParkData {
    pub(super) fn migrate(self) -> ParkData {
        ParkData {
            ride_dispatch: Vec::new(),
            ride_operating_settings: Vec::new(),
            track_economy: self.track_economy,
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
            finances: self.finances,
            carousels: self.carousels,
        }
    }
}
