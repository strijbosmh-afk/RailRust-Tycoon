//! Exact pre-operating-cost finance types; no new charge or history is invented.
use super::*;
use crate::{
    FINANCE_LEDGER_CAPACITY, FinanceTotals, MAX_SERVICES, RideId, ServiceFinanceOpening,
    ServiceFinanceTotals, ServiceId,
};

#[derive(Clone, Serialize, Deserialize)]
pub(super) enum LegacyFinanceKind {
    RideRevenue,
    ServiceRevenue,
    ServiceUpkeep,
    ServiceSupplies,
    RideConstruction,
    RideDemolitionRefund,
    ServiceConstruction,
    ServiceDemolitionRefund,
    TrackConstruction,
    TrackRefund,
}
impl From<LegacyFinanceKind> for crate::FinanceKind {
    fn from(value: LegacyFinanceKind) -> Self {
        match value {
            LegacyFinanceKind::RideRevenue => Self::RideRevenue,
            LegacyFinanceKind::ServiceRevenue => Self::ServiceRevenue,
            LegacyFinanceKind::ServiceUpkeep => Self::ServiceUpkeep,
            LegacyFinanceKind::ServiceSupplies => Self::ServiceSupplies,
            LegacyFinanceKind::RideConstruction => Self::RideConstruction,
            LegacyFinanceKind::RideDemolitionRefund => Self::RideDemolitionRefund,
            LegacyFinanceKind::ServiceConstruction => Self::ServiceConstruction,
            LegacyFinanceKind::ServiceDemolitionRefund => Self::ServiceDemolitionRefund,
            LegacyFinanceKind::TrackConstruction => Self::TrackConstruction,
            LegacyFinanceKind::TrackRefund => Self::TrackRefund,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegacyRideFinanceTotals {
    pub(super) ride: RideId,
    pub(super) revenue_cents: u64,
    pub(super) construction_cents: u64,
    pub(super) demolition_refunds_cents: u64,
}
impl From<LegacyRideFinanceTotals> for crate::RideFinanceTotals {
    fn from(old: LegacyRideFinanceTotals) -> Self {
        Self {
            ride: old.ride,
            revenue_cents: old.revenue_cents,
            construction_cents: old.construction_cents,
            demolition_refunds_cents: old.demolition_refunds_cents,
            upkeep_cents: 0,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegacyFinanceEntry {
    id: u64,
    tick: u64,
    #[serde(deserialize_with = "string_enum")]
    kind: LegacyFinanceKind,
    #[serde(deserialize_with = "required_option")]
    service: Option<ServiceId>,
    #[serde(deserialize_with = "required_option")]
    ride: Option<RideId>,
    amount_cents: u64,
    balance_cents: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegacyFinanceLedger {
    opening_tick: u64,
    opening_cash_cents: u64,
    opening_ride_revenue_cents: u64,
    #[serde(deserialize_with = "bounded_objects::<_,_,9>")]
    opening_rides: Vec<LegacyRideFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_,_,MAX_SERVICES>")]
    opening_services: Vec<ServiceFinanceOpening>,
    archived_entries: u64,
    archived_through_tick: u64,
    #[serde(deserialize_with = "strict_object")]
    archived: FinanceTotals,
    #[serde(deserialize_with = "bounded_objects::<_,_,MAX_SERVICES>")]
    archived_services: Vec<ServiceFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_,_,9>")]
    archived_rides: Vec<LegacyRideFinanceTotals>,
    #[serde(deserialize_with = "bounded_objects::<_,_,FINANCE_LEDGER_CAPACITY>")]
    entries: Vec<LegacyFinanceEntry>,
    next_entry_id: u64,
}
impl From<LegacyFinanceLedger> for crate::FinanceLedger {
    fn from(old: LegacyFinanceLedger) -> Self {
        Self {
            opening_tick: old.opening_tick,
            opening_cash_cents: old.opening_cash_cents,
            opening_ride_revenue_cents: old.opening_ride_revenue_cents,
            opening_rides: old.opening_rides.into_iter().map(Into::into).collect(),
            opening_services: old.opening_services,
            archived_entries: old.archived_entries,
            archived_through_tick: old.archived_through_tick,
            archived: old.archived,
            archived_services: old.archived_services,
            archived_rides: old.archived_rides.into_iter().map(Into::into).collect(),
            entries: old
                .entries
                .into_iter()
                .map(|e| crate::FinanceEntry {
                    id: e.id,
                    tick: e.tick,
                    kind: e.kind.into(),
                    service: e.service,
                    ride: e.ride,
                    amount_cents: e.amount_cents,
                    balance_cents: e.balance_cents,
                })
                .collect(),
            next_entry_id: old.next_entry_id,
        }
    }
}
