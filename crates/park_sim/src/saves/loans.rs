//! Exact schema-20 finance types, before financing transactions.
use super::*;
use crate::{
    FINANCE_LEDGER_CAPACITY, MAX_SERVICES, RideFinanceTotals, RideId, ServiceFinanceOpening,
    ServiceFinanceTotals, ServiceId,
};
#[derive(Clone, Serialize, Deserialize)]
pub(super) enum LegacyTwentyKind {
    RideUpkeep,
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
impl From<LegacyTwentyKind> for crate::FinanceKind {
    fn from(v: LegacyTwentyKind) -> Self {
        match v {
            LegacyTwentyKind::RideUpkeep => Self::RideUpkeep,
            LegacyTwentyKind::RideRevenue => Self::RideRevenue,
            LegacyTwentyKind::ServiceRevenue => Self::ServiceRevenue,
            LegacyTwentyKind::ServiceUpkeep => Self::ServiceUpkeep,
            LegacyTwentyKind::ServiceSupplies => Self::ServiceSupplies,
            LegacyTwentyKind::RideConstruction => Self::RideConstruction,
            LegacyTwentyKind::RideDemolitionRefund => Self::RideDemolitionRefund,
            LegacyTwentyKind::ServiceConstruction => Self::ServiceConstruction,
            LegacyTwentyKind::ServiceDemolitionRefund => Self::ServiceDemolitionRefund,
            LegacyTwentyKind::TrackConstruction => Self::TrackConstruction,
            LegacyTwentyKind::TrackRefund => Self::TrackRefund,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegacyFinanceTotals {
    pub ride_revenue_cents: u64,
    pub service_revenue_cents: u64,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
    pub construction_cents: u64,
    pub demolition_refunds_cents: u64,
}
impl From<LegacyFinanceTotals> for crate::FinanceTotals {
    fn from(v: LegacyFinanceTotals) -> Self {
        Self {
            ride_revenue_cents: v.ride_revenue_cents,
            service_revenue_cents: v.service_revenue_cents,
            upkeep_cents: v.upkeep_cents,
            supplies_cents: v.supplies_cents,
            construction_cents: v.construction_cents,
            demolition_refunds_cents: v.demolition_refunds_cents,
            ..Default::default()
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegacyTwentyEntry {
    pub id: u64,
    pub tick: u64,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub kind: LegacyTwentyKind,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub service: Option<ServiceId>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub ride: Option<RideId>,
    pub amount_cents: u64,
    pub balance_cents: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegacyTwentyLedger {
    /// No historical transaction is fabricated when importing an older save.
    pub opening_tick: u64,
    pub opening_cash_cents: u64,
    pub opening_ride_revenue_cents: u64,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, 9>")]
    pub opening_rides: Vec<RideFinanceTotals>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, MAX_SERVICES>")]
    pub opening_services: Vec<ServiceFinanceOpening>,
    pub archived_entries: u64,
    pub archived_through_tick: u64,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub archived: LegacyFinanceTotals,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, MAX_SERVICES>")]
    pub archived_services: Vec<ServiceFinanceTotals>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, 9>")]
    pub archived_rides: Vec<RideFinanceTotals>,
    /// Oldest first. Cash balances describe the state immediately after each entry.
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, FINANCE_LEDGER_CAPACITY>")]
    pub entries: Vec<LegacyTwentyEntry>,
    pub next_entry_id: u64,
}
impl From<LegacyTwentyLedger> for crate::FinanceLedger {
    fn from(v: LegacyTwentyLedger) -> Self {
        Self {
            opening_tick: v.opening_tick,
            opening_cash_cents: v.opening_cash_cents,
            opening_ride_revenue_cents: v.opening_ride_revenue_cents,
            opening_rides: v.opening_rides,
            opening_services: v.opening_services,
            archived_entries: v.archived_entries,
            archived_through_tick: v.archived_through_tick,
            archived: v.archived.into(),
            archived_services: v.archived_services,
            archived_rides: v.archived_rides,
            entries: v
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
            next_entry_id: v.next_entry_id,
        }
    }
}
