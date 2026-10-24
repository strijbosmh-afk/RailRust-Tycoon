//! Explicit pre-construction DTOs. Old files cannot smuggle in capital costs,
//! refunds or paid bases, and existing balances/ledger order stay unchanged.
use super::*;

#[derive(Clone, Copy, Deserialize)]
pub(super) enum LegacyFinanceKind {
    RideRevenue,
    ServiceRevenue,
    ServiceUpkeep,
    ServiceSupplies,
}
impl From<LegacyFinanceKind> for crate::FinanceKind {
    fn from(value: LegacyFinanceKind) -> Self {
        match value {
            LegacyFinanceKind::RideRevenue => Self::RideRevenue,
            LegacyFinanceKind::ServiceRevenue => Self::ServiceRevenue,
            LegacyFinanceKind::ServiceUpkeep => Self::ServiceUpkeep,
            LegacyFinanceKind::ServiceSupplies => Self::ServiceSupplies,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VersionTenFinanceTotals {
    pub(super) ride_revenue_cents: u64,
    service_revenue_cents: u64,
    upkeep_cents: u64,
    supplies_cents: u64,
}
impl From<VersionTenFinanceTotals> for crate::FinanceTotals {
    fn from(old: VersionTenFinanceTotals) -> Self {
        Self {
            ride_revenue_cents: old.ride_revenue_cents,
            service_revenue_cents: old.service_revenue_cents,
            upkeep_cents: old.upkeep_cents,
            supplies_cents: old.supplies_cents,
            construction_cents: 0,
            demolition_refunds_cents: 0,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionTenRideFinanceTotals {
    ride: crate::RideId,
    revenue_cents: u64,
}
impl From<VersionTenRideFinanceTotals> for crate::RideFinanceTotals {
    fn from(old: VersionTenRideFinanceTotals) -> Self {
        Self {
            ride: old.ride,
            revenue_cents: old.revenue_cents,
            construction_cents: 0,
            demolition_refunds_cents: 0,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionTenFinanceEntry {
    id: u64,
    tick: u64,
    #[serde(deserialize_with = "string_enum")]
    kind: LegacyFinanceKind,
    #[serde(deserialize_with = "required_option")]
    service: Option<crate::ServiceId>,
    #[serde(deserialize_with = "required_option")]
    ride: Option<crate::RideId>,
    amount_cents: u64,
    balance_cents: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionTenFinanceLedger {
    opening_tick: u64,
    opening_cash_cents: u64,
    opening_ride_revenue_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_RIDE_RECORDS }>")]
    opening_rides: Vec<VersionTenRideFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    opening_services: Vec<crate::ServiceFinanceOpening>,
    archived_entries: u64,
    archived_through_tick: u64,
    #[serde(deserialize_with = "strict_object")]
    archived: VersionTenFinanceTotals,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    archived_services: Vec<VersionElevenServiceFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_RIDE_RECORDS }>")]
    archived_rides: Vec<VersionTenRideFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::FINANCE_LEDGER_CAPACITY }>")]
    entries: Vec<VersionTenFinanceEntry>,
    next_entry_id: u64,
}
impl From<VersionTenFinanceLedger> for crate::FinanceLedger {
    fn from(old: VersionTenFinanceLedger) -> Self {
        Self {
            opening_tick: old.opening_tick,
            opening_cash_cents: old.opening_cash_cents,
            opening_ride_revenue_cents: old.opening_ride_revenue_cents,
            opening_rides: old.opening_rides.into_iter().map(Into::into).collect(),
            opening_services: old.opening_services,
            archived_entries: old.archived_entries,
            archived_through_tick: old.archived_through_tick,
            archived: old.archived.into(),
            archived_services: old.archived_services.into_iter().map(Into::into).collect(),
            archived_rides: old.archived_rides.into_iter().map(Into::into).collect(),
            entries: old
                .entries
                .into_iter()
                .map(|entry| crate::FinanceEntry {
                    id: entry.id,
                    tick: entry.tick,
                    kind: entry.kind.into(),
                    service: entry.service,
                    ride: entry.ride,
                    amount_cents: entry.amount_cents,
                    balance_cents: entry.balance_cents,
                })
                .collect(),
            next_entry_id: old.next_entry_id,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionTenCarousel {
    #[serde(deserialize_with = "strict_object")]
    ride: crate::Ride,
    #[serde(deserialize_with = "strict_object")]
    origin: crate::TilePos,
    height: u8,
    #[serde(deserialize_with = "bounded_vec::<_, _, 9>")]
    objects: Vec<ObjectId>,
    open: bool,
    #[serde(deserialize_with = "string_enum")]
    phase: crate::CarouselPhase,
    progress: u16,
    dwell_ticks: u16,
    cycles: u64,
}
impl From<VersionTenCarousel> for crate::CarouselRide {
    fn from(old: VersionTenCarousel) -> Self {
        Self {
            ride: old.ride,
            origin: old.origin,
            height: old.height,
            objects: old.objects,
            open: old.open,
            phase: old.phase,
            progress: old.progress,
            dwell_ticks: old.dwell_ticks,
            cycles: old.cycles,
            construction_cost_cents: 0,
            demolition_refund_cents: 0,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VersionTenParkData {
    width: u32,
    height: u32,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_TILES>")]
    tiles: Vec<Tile>,
    ticks: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_OBJECTS>")]
    objects: Vec<SurfaceObject>,
    next_object_id: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, MAX_GUESTS>")]
    guests: Vec<Guest>,
    next_guest_id: u64,
    #[serde(deserialize_with = "required_option")]
    crowd_seed: Option<u64>,
    crowd_rng: u64,
    #[serde(deserialize_with = "required_object_option")]
    circuit: Option<CircuitData>,
    #[serde(deserialize_with = "required_object_option")]
    train: Option<Train>,
    #[serde(deserialize_with = "required_object_option")]
    ride: Option<crate::Ride>,
    next_ride_id: u64,
    cash_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    services: Vec<VersionElevenService>,
    next_service_id: u64,
    #[serde(deserialize_with = "strict_object")]
    finances: VersionTenFinanceLedger,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_CAROUSEL_RECORDS }>")]
    carousels: Vec<VersionTenCarousel>,
}
impl VersionTenParkData {
    pub(super) fn migrate(self) -> ParkData {
        ParkData {
            ride_dispatch: Vec::new(),
            ride_operating_settings: Vec::new(),
            track_economy: crate::TrackEconomy::default(),
            track_draft: Vec::new(),
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
            services: self.services.into_iter().map(Into::into).collect(),
            next_service_id: self.next_service_id,
            finances: self.finances.into(),
            carousels: self.carousels.into_iter().map(Into::into).collect(),
        }
    }
}
