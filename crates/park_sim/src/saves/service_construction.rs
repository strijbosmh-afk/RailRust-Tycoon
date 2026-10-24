//! Strict schema-11 and earlier service shapes. Construction bases and capital
//! transactions cannot be backdated by declaring an older save version.
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VersionElevenService {
    id: crate::ServiceId,
    #[serde(deserialize_with = "required_option")]
    object: Option<ObjectId>,
    #[serde(deserialize_with = "string_enum")]
    kind: crate::ServiceKind,
    #[serde(deserialize_with = "string_enum")]
    facing: crate::PathDirection,
    open: bool,
    price_cents: u32,
    served: u64,
    revenue_cents: u64,
    unaffordable_rejections: u64,
    upkeep_progress: u16,
    upkeep_cents: u64,
    supplies_cents: u64,
    closed_for_funds: bool,
}
impl From<VersionElevenService> for crate::Service {
    fn from(old: VersionElevenService) -> Self {
        Self {
            id: old.id,
            object: old.object,
            kind: old.kind,
            facing: old.facing,
            open: old.open,
            price_cents: old.price_cents,
            served: old.served,
            revenue_cents: old.revenue_cents,
            unaffordable_rejections: old.unaffordable_rejections,
            upkeep_progress: old.upkeep_progress,
            upkeep_cents: old.upkeep_cents,
            supplies_cents: old.supplies_cents,
            closed_for_funds: old.closed_for_funds,
            construction_cost_cents: 0,
            demolition_refund_cents: 0,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VersionElevenServiceFinanceTotals {
    service: crate::ServiceId,
    revenue_cents: u64,
    upkeep_cents: u64,
    supplies_cents: u64,
}
impl From<VersionElevenServiceFinanceTotals> for crate::ServiceFinanceTotals {
    fn from(old: VersionElevenServiceFinanceTotals) -> Self {
        Self {
            service: old.service,
            revenue_cents: old.revenue_cents,
            upkeep_cents: old.upkeep_cents,
            supplies_cents: old.supplies_cents,
            construction_cents: 0,
            demolition_refunds_cents: 0,
        }
    }
}
#[derive(Clone, Copy, Deserialize)]
enum VersionElevenFinanceKind {
    RideRevenue,
    ServiceRevenue,
    ServiceUpkeep,
    ServiceSupplies,
    RideConstruction,
    RideDemolitionRefund,
}
impl From<VersionElevenFinanceKind> for crate::FinanceKind {
    fn from(old: VersionElevenFinanceKind) -> Self {
        match old {
            VersionElevenFinanceKind::RideRevenue => Self::RideRevenue,
            VersionElevenFinanceKind::ServiceRevenue => Self::ServiceRevenue,
            VersionElevenFinanceKind::ServiceUpkeep => Self::ServiceUpkeep,
            VersionElevenFinanceKind::ServiceSupplies => Self::ServiceSupplies,
            VersionElevenFinanceKind::RideConstruction => Self::RideConstruction,
            VersionElevenFinanceKind::RideDemolitionRefund => Self::RideDemolitionRefund,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionElevenFinanceEntry {
    id: u64,
    tick: u64,
    #[serde(deserialize_with = "string_enum")]
    kind: VersionElevenFinanceKind,
    #[serde(deserialize_with = "required_option")]
    service: Option<crate::ServiceId>,
    #[serde(deserialize_with = "required_option")]
    ride: Option<crate::RideId>,
    amount_cents: u64,
    balance_cents: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionElevenFinanceLedger {
    opening_tick: u64,
    opening_cash_cents: u64,
    opening_ride_revenue_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_RIDE_RECORDS }>")]
    opening_rides: Vec<crate::RideFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    opening_services: Vec<crate::ServiceFinanceOpening>,
    archived_entries: u64,
    archived_through_tick: u64,
    #[serde(deserialize_with = "strict_object")]
    archived: crate::FinanceTotals,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_SERVICES }>")]
    archived_services: Vec<VersionElevenServiceFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_RIDE_RECORDS }>")]
    archived_rides: Vec<crate::RideFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::FINANCE_LEDGER_CAPACITY }>")]
    entries: Vec<VersionElevenFinanceEntry>,
    next_entry_id: u64,
}
impl From<VersionElevenFinanceLedger> for crate::FinanceLedger {
    fn from(old: VersionElevenFinanceLedger) -> Self {
        Self {
            opening_tick: old.opening_tick,
            opening_cash_cents: old.opening_cash_cents,
            opening_ride_revenue_cents: old.opening_ride_revenue_cents,
            opening_rides: old.opening_rides,
            opening_services: old.opening_services,
            archived_entries: old.archived_entries,
            archived_through_tick: old.archived_through_tick,
            archived: old.archived,
            archived_services: old.archived_services.into_iter().map(Into::into).collect(),
            archived_rides: old.archived_rides,
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
pub(super) struct VersionElevenParkData {
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
    finances: VersionElevenFinanceLedger,
    #[serde(deserialize_with = "bounded_objects::<_, _, { crate::MAX_CAROUSEL_RECORDS }>")]
    carousels: Vec<crate::CarouselRide>,
}
impl VersionElevenParkData {
    pub(super) fn migrate(self) -> ParkData {
        ParkData {
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
            carousels: self.carousels,
        }
    }
}
