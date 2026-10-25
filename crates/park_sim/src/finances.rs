//! Bounded chronological cash ledger. All amounts are integer cents.
use std::collections::BTreeMap;

use crate::{INITIAL_PARK_CASH_CENTS, MAX_SERVICES, Park, RideId, ServiceId};

pub const SERVICE_UPKEEP_TICKS: u16 = 90;
pub const FINANCE_LEDGER_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FinanceKind {
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
impl FinanceKind {
    fn income(self) -> bool {
        matches!(
            self,
            Self::RideRevenue
                | Self::ServiceRevenue
                | Self::RideDemolitionRefund
                | Self::ServiceDemolitionRefund
                | Self::TrackRefund
        )
    }
    fn ride_source(self) -> bool {
        matches!(
            self,
            Self::RideUpkeep
                | Self::RideRevenue
                | Self::RideConstruction
                | Self::RideDemolitionRefund
                | Self::TrackConstruction
                | Self::TrackRefund
        )
    }
}
impl std::fmt::Display for FinanceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::RideUpkeep => "Ride upkeep",
            Self::RideRevenue => "Ride tickets",
            Self::ServiceRevenue => "Service sale",
            Self::ServiceUpkeep => "Service upkeep",
            Self::ServiceSupplies => "Service supplies",
            Self::RideConstruction => "Ride construction",
            Self::RideDemolitionRefund => "Demolition refund",
            Self::ServiceConstruction => "Service construction",
            Self::ServiceDemolitionRefund => "Service demolition refund",
            Self::TrackConstruction => "Track construction",
            Self::TrackRefund => "Track refund",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinanceEntry {
    pub id: u64,
    pub tick: u64,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub kind: FinanceKind,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub service: Option<ServiceId>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub ride: Option<RideId>,
    pub amount_cents: u64,
    pub balance_cents: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinanceTotals {
    pub ride_revenue_cents: u64,
    pub service_revenue_cents: u64,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
    pub construction_cents: u64,
    pub demolition_refunds_cents: u64,
}
impl FinanceTotals {
    fn add(&mut self, kind: FinanceKind, amount: u64) -> Result<(), String> {
        let target = match kind {
            FinanceKind::RideRevenue => &mut self.ride_revenue_cents,
            FinanceKind::ServiceRevenue => &mut self.service_revenue_cents,
            FinanceKind::ServiceUpkeep | FinanceKind::RideUpkeep => &mut self.upkeep_cents,
            FinanceKind::ServiceSupplies => &mut self.supplies_cents,
            FinanceKind::RideConstruction
            | FinanceKind::ServiceConstruction
            | FinanceKind::TrackConstruction => &mut self.construction_cents,
            FinanceKind::RideDemolitionRefund
            | FinanceKind::ServiceDemolitionRefund
            | FinanceKind::TrackRefund => &mut self.demolition_refunds_cents,
        };
        *target = target
            .checked_add(amount)
            .ok_or("finance totals overflow")?;
        Ok(())
    }
    fn balance(self, opening: u64) -> Option<u64> {
        opening
            .checked_add(self.ride_revenue_cents)?
            .checked_add(self.service_revenue_cents)?
            .checked_add(self.demolition_refunds_cents)?
            .checked_sub(self.upkeep_cents)?
            .checked_sub(self.supplies_cents)?
            .checked_sub(self.construction_cents)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceFinanceOpening {
    pub service: ServiceId,
    pub revenue_cents: u64,
    pub served: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideFinanceTotals {
    pub upkeep_cents: u64,
    pub ride: RideId,
    pub revenue_cents: u64,
    pub construction_cents: u64,
    pub demolition_refunds_cents: u64,
}
impl RideFinanceTotals {
    fn new(ride: RideId) -> Self {
        Self {
            upkeep_cents: 0,
            ride,
            revenue_cents: 0,
            construction_cents: 0,
            demolition_refunds_cents: 0,
        }
    }
    fn add(&mut self, kind: FinanceKind, amount: u64) -> Result<(), String> {
        let target = match kind {
            FinanceKind::RideUpkeep => &mut self.upkeep_cents,
            FinanceKind::RideRevenue => &mut self.revenue_cents,
            FinanceKind::RideConstruction | FinanceKind::TrackConstruction => {
                &mut self.construction_cents
            }
            FinanceKind::RideDemolitionRefund | FinanceKind::TrackRefund => {
                &mut self.demolition_refunds_cents
            }
            _ => return Err("service transaction has a ride owner".into()),
        };
        *target = target
            .checked_add(amount)
            .ok_or("ride finance totals overflow")?;
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceFinanceTotals {
    pub service: ServiceId,
    pub revenue_cents: u64,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
    pub construction_cents: u64,
    pub demolition_refunds_cents: u64,
}
impl ServiceFinanceTotals {
    fn new(service: ServiceId) -> Self {
        Self {
            service,
            revenue_cents: 0,
            upkeep_cents: 0,
            supplies_cents: 0,
            construction_cents: 0,
            demolition_refunds_cents: 0,
        }
    }
    fn add(&mut self, kind: FinanceKind, amount: u64) -> Result<(), String> {
        let target = match kind {
            FinanceKind::ServiceRevenue => &mut self.revenue_cents,
            FinanceKind::ServiceUpkeep => &mut self.upkeep_cents,
            FinanceKind::ServiceSupplies => &mut self.supplies_cents,
            FinanceKind::ServiceConstruction => &mut self.construction_cents,
            FinanceKind::ServiceDemolitionRefund => &mut self.demolition_refunds_cents,
            _ => return Err("ride transaction has a service source".into()),
        };
        *target = target
            .checked_add(amount)
            .ok_or("service finance totals overflow")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinanceLedger {
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
    pub archived: FinanceTotals,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, MAX_SERVICES>")]
    pub archived_services: Vec<ServiceFinanceTotals>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, 9>")]
    pub archived_rides: Vec<RideFinanceTotals>,
    /// Oldest first. Cash balances describe the state immediately after each entry.
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, FINANCE_LEDGER_CAPACITY>")]
    pub entries: Vec<FinanceEntry>,
    pub next_entry_id: u64,
}
impl Default for FinanceLedger {
    fn default() -> Self {
        Self {
            opening_tick: 0,
            opening_cash_cents: INITIAL_PARK_CASH_CENTS,
            opening_ride_revenue_cents: 0,
            opening_rides: Vec::new(),
            opening_services: Vec::new(),
            archived_entries: 0,
            archived_through_tick: 0,
            archived: FinanceTotals::default(),
            archived_services: Vec::new(),
            archived_rides: Vec::new(),
            entries: Vec::new(),
            next_entry_id: 1,
        }
    }
}
impl FinanceLedger {
    #[cfg(test)]
    pub(crate) fn opening(park: &Park) -> Self {
        let mut opening_services: Vec<_> = park
            .services
            .iter()
            .map(|service| ServiceFinanceOpening {
                service: service.id,
                revenue_cents: service.revenue_cents,
                served: service.served,
            })
            .collect();
        opening_services.sort_by_key(|entry| entry.service);
        let mut opening_rides: Vec<_> = park
            .all_rides()
            .map(|ride| RideFinanceTotals {
                upkeep_cents: 0,
                ride: ride.id,
                revenue_cents: ride.revenue_cents,
                construction_cents: 0,
                demolition_refunds_cents: 0,
            })
            .collect();
        opening_rides.sort_by_key(|entry| entry.ride);
        Self {
            opening_tick: park.ticks,
            opening_cash_cents: park.cash_cents,
            opening_ride_revenue_cents: park.all_rides().map(|ride| ride.revenue_cents).sum(),
            opening_rides,
            opening_services,
            archived_through_tick: park.ticks,
            ..Self::default()
        }
    }
    fn push(
        &mut self,
        tick: u64,
        kind: FinanceKind,
        service: Option<ServiceId>,
        ride: Option<RideId>,
        amount: u64,
        balance: u64,
    ) {
        if amount == 0 {
            return;
        }
        if self.entries.len() == FINANCE_LEDGER_CAPACITY {
            let old = self.entries.remove(0);
            self.archived
                .add(old.kind, old.amount_cents)
                .expect("bounded conserved cash history");
            if let Some(id) = old.service {
                let index = self
                    .archived_services
                    .binary_search_by_key(&id, |entry| entry.service)
                    .unwrap_or_else(|index| {
                        self.archived_services
                            .insert(index, ServiceFinanceTotals::new(id));
                        index
                    });
                self.archived_services[index]
                    .add(old.kind, old.amount_cents)
                    .expect("bounded service cash history");
            }
            if let Some(id) = old.ride {
                let index = self
                    .archived_rides
                    .binary_search_by_key(&id, |entry| entry.ride)
                    .unwrap_or_else(|index| {
                        self.archived_rides.insert(
                            index,
                            RideFinanceTotals {
                                upkeep_cents: 0,
                                ride: id,
                                revenue_cents: 0,
                                construction_cents: 0,
                                demolition_refunds_cents: 0,
                            },
                        );
                        index
                    });
                self.archived_rides[index]
                    .add(old.kind, old.amount_cents)
                    .expect("bounded ride cash history");
            }
            self.archived_entries += 1;
            self.archived_through_tick = old.tick;
        }
        self.entries.push(FinanceEntry {
            id: self.next_entry_id,
            tick,
            kind,
            service,
            ride,
            amount_cents: amount,
            balance_cents: balance,
        });
        self.next_entry_id += 1;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FinanceStats {
    pub ride_upkeep_cents: u64,
    pub service_upkeep_cents: u64,
    pub cash_cents: u64,
    pub ride_revenue_cents: u64,
    pub service_revenue_cents: u64,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
    pub construction_cents: u64,
    pub demolition_refunds_cents: u64,
    pub total_costs_cents: u64,
    pub net_profit_cents: i64,
    pub open_upkeep_per_minute_cents: u64,
    pub retained_entries: usize,
    pub archived_entries: u64,
}
impl Park {
    pub fn finance_ledger(&self) -> &FinanceLedger {
        &self.finances
    }
    pub fn finance_stats(&self) -> FinanceStats {
        let ride_revenue_cents = self.all_rides().map(|ride| ride.revenue_cents).sum();
        let service_revenue_cents = self
            .services
            .iter()
            .map(|service| service.revenue_cents)
            .sum::<u64>();
        let service_upkeep_cents = self
            .services
            .iter()
            .map(|service| service.upkeep_cents)
            .sum::<u64>();
        let ride_upkeep_cents = self.ride_costs.iter().map(|r| r.upkeep_cents).sum::<u64>();
        let upkeep_cents = service_upkeep_cents + ride_upkeep_cents;
        let supplies_cents = self
            .services
            .iter()
            .map(|service| service.supplies_cents)
            .sum::<u64>();
        let construction_cents = self
            .carousels
            .iter()
            .map(|c| c.construction_cost_cents)
            .chain(self.services.iter().map(|s| s.construction_cost_cents))
            .chain(std::iter::once(self.track_economy.construction_cents))
            .sum::<u64>();
        let demolition_refunds_cents = self
            .carousels
            .iter()
            .map(|c| c.demolition_refund_cents)
            .chain(self.services.iter().map(|s| s.demolition_refund_cents))
            .chain(std::iter::once(self.track_economy.refund_cents))
            .sum::<u64>();
        let total_costs_cents = upkeep_cents + supplies_cents + construction_cents;
        FinanceStats {
            ride_upkeep_cents,
            service_upkeep_cents,
            cash_cents: self.cash_cents,
            ride_revenue_cents,
            service_revenue_cents,
            upkeep_cents,
            supplies_cents,
            construction_cents,
            demolition_refunds_cents,
            total_costs_cents,
            net_profit_cents: ride_revenue_cents as i64
                + service_revenue_cents as i64
                + demolition_refunds_cents as i64
                - total_costs_cents as i64,
            open_upkeep_per_minute_cents: self
                .services
                .iter()
                .filter(|service| service.open && service.object.is_some())
                .map(|service| u64::from(service.kind.upkeep_cost_cents()) * 20)
                .sum::<u64>()
                + self
                    .all_rides()
                    .filter(|ride| self.ride_is_open(ride.id) && self.ride_available(ride.id))
                    .map(|ride| {
                        u64::from(crate::ride_upkeep_cost_cents(
                            self.ride_kind(ride.id).unwrap(),
                        )) * 2
                    })
                    .sum::<u64>(),
            retained_entries: self.finances.entries.len(),
            archived_entries: self.finances.archived_entries,
        }
    }
    pub(crate) fn record_cash(
        &mut self,
        kind: FinanceKind,
        source: Option<ServiceId>,
        amount: u64,
    ) {
        if amount == 0 {
            return;
        }
        self.cash_cents = if kind.income() {
            self.cash_cents.checked_add(amount)
        } else {
            self.cash_cents.checked_sub(amount)
        }
        .expect("validated conserved and affordable cash transaction");
        self.finances
            .push(self.ticks, kind, source, None, amount, self.cash_cents);
    }
    pub(crate) fn record_ride_cash(&mut self, ride: RideId, amount: u64) {
        self.record_ride_transaction(FinanceKind::RideRevenue, ride, amount);
    }
    pub(crate) fn record_ride_transaction(&mut self, kind: FinanceKind, ride: RideId, amount: u64) {
        if amount == 0 {
            return;
        }
        self.cash_cents = if kind.income() {
            self.cash_cents.checked_add(amount)
        } else {
            self.cash_cents.checked_sub(amount)
        }
        .expect("validated conserved and affordable ride transaction");
        self.finances
            .push(self.ticks, kind, None, Some(ride), amount, self.cash_cents);
    }
    pub(crate) fn step_service_upkeep(&mut self) {
        let mut order: Vec<_> = self
            .services
            .iter()
            .enumerate()
            .filter(|(_, service)| service.open && service.object.is_some())
            .map(|(index, service)| (service.id, index))
            .collect();
        order.sort_unstable();
        let mut closed = false;
        for (id, index) in order {
            let service = &mut self.services[index];
            if service.upkeep_progress + 1 < SERVICE_UPKEEP_TICKS {
                service.upkeep_progress += 1;
                continue;
            }
            let amount = u64::from(service.kind.upkeep_cost_cents());
            if self.cash_cents < amount {
                service.open = false;
                service.closed_for_funds = true;
                closed = true;
            } else {
                service.upkeep_progress = 0;
                service.upkeep_cents += amount;
                self.record_cash(FinanceKind::ServiceUpkeep, Some(id), amount);
            }
        }
        if closed {
            self.reconcile_service_visits();
        }
    }
}

pub(crate) fn validate_finances(park: &Park) -> Result<(), String> {
    crate::ride_costs::validate(park)?;
    let ledger = &park.finances;
    if ledger.opening_tick > park.ticks
        || ledger.archived_through_tick < ledger.opening_tick
        || ledger.archived_through_tick > park.ticks
        || ledger.entries.len() > FINANCE_LEDGER_CAPACITY
        || ledger.opening_services.len() > MAX_SERVICES
        || ledger.archived_services.len() > MAX_SERVICES
        || ledger.opening_rides.len() > 9
        || ledger.archived_rides.len() > 9
    {
        return Err("finance window or bounds are invalid".into());
    }
    let ride_revenue = park
        .all_rides()
        .try_fold(0_u64, |sum, ride| sum.checked_add(ride.revenue_cents))
        .ok_or("ride receipts overflow")?;
    if ledger.opening_ride_revenue_cents > ride_revenue {
        return Err("opening ride income exceeds history".into());
    }
    let mut opening = BTreeMap::new();
    let mut opening_revenue = ledger.opening_ride_revenue_cents;
    let mut ride_opening = BTreeMap::new();
    let mut ride_opening_total = 0_u64;
    let mut previous_ride = None;
    for entry in &ledger.opening_rides {
        let ride = park
            .ride_by_id(entry.ride)
            .ok_or("opening ride is unknown")?;
        if previous_ride.is_some_and(|id| id >= entry.ride)
            || entry.revenue_cents > ride.revenue_cents
            || entry.construction_cents != 0
            || entry.upkeep_cents != 0
            || entry.demolition_refunds_cents != 0
            || (entry.revenue_cents != 0
                && park
                    .carousel(entry.ride)
                    .is_some_and(|c| c.construction_cost_cents != 0))
        {
            return Err("opening ride history is inconsistent".into());
        }
        previous_ride = Some(entry.ride);
        ride_opening.insert(entry.ride, entry.revenue_cents);
        ride_opening_total = ride_opening_total
            .checked_add(entry.revenue_cents)
            .ok_or("opening ride income overflows")?;
    }
    if ride_opening_total != ledger.opening_ride_revenue_cents {
        return Err("opening ride attribution disagrees with total".into());
    }
    let mut last = None;
    for entry in &ledger.opening_services {
        let service = park
            .service(entry.service)
            .ok_or("opening service is unknown")?;
        if last.is_some_and(|id| id >= entry.service)
            || entry.served > service.served
            || entry.revenue_cents > service.revenue_cents
            || (service.construction_cost_cents != 0
                && (entry.served != 0 || entry.revenue_cents != 0))
            || u128::from(entry.revenue_cents)
                > u128::from(entry.served) * u128::from(crate::INITIAL_GUEST_WALLET_CENTS)
        {
            return Err("opening service history is inconsistent".into());
        }
        last = Some(entry.service);
        opening.insert(entry.service, entry);
        opening_revenue = opening_revenue
            .checked_add(entry.revenue_cents)
            .ok_or("opening receipts overflow")?;
    }
    if INITIAL_PARK_CASH_CENTS.checked_add(opening_revenue) != Some(ledger.opening_cash_cents) {
        return Err("opening cash is not backed by legacy receipts".into());
    }
    let mut sources = BTreeMap::new();
    let mut archived_check = FinanceTotals::default();
    let max_fare_batch = crate::TRAIN_SEATS as u128 * u128::from(crate::INITIAL_GUEST_WALLET_CENTS);
    let mut minimum_archived = 0_u128;
    let mut maximum_archived = 0_u128;
    let mut ride_sources = BTreeMap::new();
    previous_ride = None;
    for entry in &ledger.archived_rides {
        if park.ride_by_id(entry.ride).is_none()
            || previous_ride.is_some_and(|id| id >= entry.ride)
            || (entry.revenue_cents == 0
                && entry.upkeep_cents == 0
                && entry.construction_cents == 0
                && entry.demolition_refunds_cents == 0)
        {
            return Err("archived ride source is invalid".into());
        }
        previous_ride = Some(entry.ride);
        let carousel = park.carousel(entry.ride);
        let upkeep_cost = u64::from(crate::ride_upkeep_cost_cents(
            park.ride_kind(entry.ride).unwrap(),
        ));
        if !entry.upkeep_cents.is_multiple_of(upkeep_cost)
            || entry.upkeep_cents > park.ride_running_costs(entry.ride).unwrap().upkeep_cents
        {
            return Err("archived ride upkeep disagrees with its owner".into());
        }
        let operating_entries = u128::from(entry.upkeep_cents / upkeep_cost);
        minimum_archived += operating_entries;
        maximum_archived += operating_entries;
        let basis = carousel.map_or(park.track_economy.construction_cents, |c| {
            c.construction_cost_cents
        });
        let refund = carousel.map_or(park.track_economy.refund_cents, |c| {
            c.demolition_refund_cents
        });
        if carousel.is_some()
            && ((entry.construction_cents != 0 && entry.construction_cents != basis)
                || (entry.demolition_refunds_cents != 0
                    && (entry.demolition_refunds_cents != refund
                        || entry.construction_cents != basis)))
        {
            return Err("archived ride construction or refund disagrees with body history".into());
        }
        let capital_entries = u128::from(entry.construction_cents != 0)
            + u128::from(entry.demolition_refunds_cents != 0);
        let receipts = u128::from(entry.revenue_cents);
        if carousel.is_none() {
            if entry.construction_cents > basis
                || entry.construction_cents % 500 != 0
                || (entry.construction_cents != 0 && entry.construction_cents < 1_000)
                || entry.demolition_refunds_cents > refund
                || entry.demolition_refunds_cents % 250 != 0
                || (entry.demolition_refunds_cents != 0 && entry.demolition_refunds_cents < 500)
                || entry.demolition_refunds_cents > entry.construction_cents / 2
            {
                return Err("archived track basis or refund is invalid".into());
            }
            minimum_archived += receipts.div_ceil(max_fare_batch)
                + u128::from(entry.construction_cents).div_ceil(2_560_000)
                + u128::from(entry.demolition_refunds_cents).div_ceil(1_280_000);
            maximum_archived += receipts
                + u128::from(entry.construction_cents / 1_000)
                + u128::from(entry.demolition_refunds_cents / 500);
        } else {
            minimum_archived += receipts.div_ceil(max_fare_batch) + capital_entries;
            maximum_archived += receipts + capital_entries;
        }
        archived_check.add(FinanceKind::RideRevenue, entry.revenue_cents)?;
        archived_check.add(FinanceKind::RideUpkeep, entry.upkeep_cents)?;
        archived_check.add(FinanceKind::RideConstruction, entry.construction_cents)?;
        archived_check.add(
            FinanceKind::RideDemolitionRefund,
            entry.demolition_refunds_cents,
        )?;
        ride_sources.insert(entry.ride, entry.clone());
    }
    last = None;
    for entry in &ledger.archived_services {
        let service = park
            .service(entry.service)
            .ok_or("archived service source is invalid")?;
        if last.is_some_and(|id| id >= entry.service)
            || (entry.revenue_cents == 0
                && entry.upkeep_cents == 0
                && entry.supplies_cents == 0
                && entry.construction_cents == 0
                && entry.demolition_refunds_cents == 0)
        {
            return Err("archived service source is invalid".into());
        }
        let upkeep_cost = u64::from(service.kind.upkeep_cost_cents());
        let supply_cost = u64::from(service.kind.supply_cost_cents());
        if entry.upkeep_cents % upkeep_cost != 0 || entry.supplies_cents % supply_cost != 0 {
            return Err("archived service expenses are not whole fixed-cost transactions".into());
        }
        if (entry.construction_cents != 0
            && entry.construction_cents != service.construction_cost_cents)
            || (entry.demolition_refunds_cents != 0
                && (entry.demolition_refunds_cents != service.demolition_refund_cents
                    || entry.construction_cents != service.construction_cost_cents))
            || (service.construction_cost_cents != 0
                && entry.construction_cents == 0
                && (entry.revenue_cents != 0
                    || entry.upkeep_cents != 0
                    || entry.supplies_cents != 0))
        {
            return Err(
                "archived service construction or refund disagrees with facility history".into(),
            );
        }
        let expenses = u128::from(entry.upkeep_cents / upkeep_cost)
            + u128::from(entry.supplies_cents / supply_cost)
            + u128::from(entry.construction_cents != 0)
            + u128::from(entry.demolition_refunds_cents != 0);
        let receipts = u128::from(entry.revenue_cents);
        minimum_archived +=
            expenses + receipts.div_ceil(u128::from(crate::INITIAL_GUEST_WALLET_CENTS));
        maximum_archived += expenses + receipts;
        last = Some(entry.service);
        archived_check.add(FinanceKind::ServiceRevenue, entry.revenue_cents)?;
        archived_check.add(FinanceKind::ServiceUpkeep, entry.upkeep_cents)?;
        archived_check.add(FinanceKind::ServiceSupplies, entry.supplies_cents)?;
        archived_check.add(FinanceKind::ServiceConstruction, entry.construction_cents)?;
        archived_check.add(
            FinanceKind::ServiceDemolitionRefund,
            entry.demolition_refunds_cents,
        )?;
        sources.insert(entry.service, entry.clone());
    }
    if archived_check != ledger.archived
        || (ledger.archived_entries == 0
            && (ledger.archived != FinanceTotals::default()
                || ledger.archived_through_tick != ledger.opening_tick))
        || (ledger.archived_entries > 0
            && (ledger.entries.len() != FINANCE_LEDGER_CAPACITY
                || (ledger.archived_through_tick == ledger.opening_tick
                    && (ledger.archived.ride_revenue_cents != 0
                        || ledger.archived.service_revenue_cents != 0
                        || ledger.archived.upkeep_cents != 0
                        || ledger.archived.supplies_cents != 0))
                || u128::from(ledger.archived_entries) < minimum_archived
                || u128::from(ledger.archived_entries) > maximum_archived))
    {
        return Err("archived finance totals or count are invalid".into());
    }
    let mut balance = ledger
        .archived
        .balance(ledger.opening_cash_cents)
        .ok_or("archived finance has negative cash")?;
    let mut total = ledger.archived;
    let mut expected_id = ledger
        .archived_entries
        .checked_add(1)
        .ok_or("finance ID overflow")?;
    let mut tick = ledger.archived_through_tick;
    for entry in &ledger.entries {
        if entry.id != expected_id
            || entry.tick < tick
            || (entry.tick == ledger.opening_tick
                && !matches!(
                    entry.kind,
                    FinanceKind::RideConstruction
                        | FinanceKind::RideDemolitionRefund
                        | FinanceKind::ServiceConstruction
                        | FinanceKind::ServiceDemolitionRefund
                        | FinanceKind::TrackConstruction
                        | FinanceKind::TrackRefund
                ))
            || entry.tick > park.ticks
            || entry.amount_cents == 0
        {
            return Err("finance entry order, tick or amount is invalid".into());
        }
        if entry.kind.ride_source() {
            let id = entry.ride.ok_or("ride receipt owner is missing")?;
            if entry.service.is_some() || park.ride_by_id(id).is_none() {
                return Err("ride receipt source or amount is invalid".into());
            }
            let source = ride_sources
                .entry(id)
                .or_insert_with(|| RideFinanceTotals::new(id));
            match entry.kind {
                FinanceKind::RideUpkeep => {
                    if entry.amount_cents
                        != u64::from(crate::ride_upkeep_cost_cents(park.ride_kind(id).unwrap()))
                        || (park.carousel(id).is_some()
                            && (source.demolition_refunds_cents != 0
                                || source.construction_cents
                                    != park.carousel(id).unwrap().construction_cost_cents))
                    {
                        return Err("ride upkeep amount or chronology is invalid".into());
                    }
                }
                FinanceKind::RideRevenue => {
                    if entry.amount_cents
                        > crate::TRAIN_SEATS as u64 * u64::from(crate::INITIAL_GUEST_WALLET_CENTS)
                        || (park.carousel(id).is_some() && source.demolition_refunds_cents != 0)
                    {
                        return Err(
                            "ride receipt amount or construction chronology is invalid".into()
                        );
                    }
                }
                FinanceKind::RideConstruction => {
                    let carousel = park
                        .carousel(id)
                        .ok_or("construction owner is not a buildable ride")?;
                    if entry.amount_cents != carousel.construction_cost_cents
                        || source.construction_cents != 0
                        || source.revenue_cents != 0
                        || ride_opening.get(&id).copied().unwrap_or(0) != 0
                        || source.demolition_refunds_cents != 0
                    {
                        return Err(
                            "construction charge is duplicated or differs from paid basis".into(),
                        );
                    }
                }
                FinanceKind::RideDemolitionRefund => {
                    let carousel = park
                        .carousel(id)
                        .ok_or("refund owner is not a buildable ride")?;
                    if entry.amount_cents != carousel.demolition_refund_cents
                        || source.construction_cents != carousel.construction_cost_cents
                        || source.demolition_refunds_cents != 0
                    {
                        return Err(
                            "demolition refund is duplicated or has no construction charge".into(),
                        );
                    }
                }
                FinanceKind::TrackConstruction | FinanceKind::TrackRefund => {
                    if park.ride.as_ref().is_none_or(|ride| ride.id != id) {
                        return Err("track transaction owner is not the coaster".into());
                    }
                    if entry.kind == FinanceKind::TrackConstruction {
                        if entry.amount_cents < 1_000
                            || entry.amount_cents % 500 != 0
                            || entry.amount_cents > 2_560_000
                        {
                            return Err("track purchase amount is invalid".into());
                        }
                    } else if entry.amount_cents < 500
                        || entry.amount_cents % 250 != 0
                        || entry.amount_cents > 1_280_000
                        || source
                            .demolition_refunds_cents
                            .checked_add(entry.amount_cents)
                            .is_none_or(|refund| refund > source.construction_cents / 2)
                    {
                        return Err("track refund has no prior paid basis".into());
                    }
                }
                _ => unreachable!(),
            }
            source.add(entry.kind, entry.amount_cents)?;
        } else {
            if entry.ride.is_some() {
                return Err("service transaction has a ride owner".into());
            }
            let id = entry.service.ok_or("service entry source is missing")?;
            let service = park.service(id).ok_or("service entry source is unknown")?;
            let cost = match entry.kind {
                FinanceKind::ServiceUpkeep => Some(service.kind.upkeep_cost_cents()),
                FinanceKind::ServiceSupplies => Some(service.kind.supply_cost_cents()),
                _ => None,
            };
            if cost.is_some_and(|cost| entry.amount_cents != u64::from(cost)) {
                return Err("service expense has the wrong fixed cost".into());
            }
            if entry.kind == FinanceKind::ServiceRevenue
                && entry.amount_cents > u64::from(crate::INITIAL_GUEST_WALLET_CENTS)
            {
                return Err("service receipt exceeds any possible wallet".into());
            }
            let source = sources
                .entry(id)
                .or_insert_with(|| ServiceFinanceTotals::new(id));
            match entry.kind {
                FinanceKind::ServiceConstruction => {
                    if entry.amount_cents != service.construction_cost_cents
                        || source.construction_cents != 0
                        || source.demolition_refunds_cents != 0
                        || source.revenue_cents != 0
                        || source.upkeep_cents != 0
                        || source.supplies_cents != 0
                    {
                        return Err(
                            "service construction charge is duplicated or out of order".into()
                        );
                    }
                }
                FinanceKind::ServiceDemolitionRefund => {
                    if entry.amount_cents != service.demolition_refund_cents
                        || source.construction_cents != service.construction_cost_cents
                        || source.demolition_refunds_cents != 0
                    {
                        return Err(
                            "service refund is duplicated or has no construction charge".into()
                        );
                    }
                }
                _ => {
                    if source.construction_cents != service.construction_cost_cents
                        || source.demolition_refunds_cents != 0
                    {
                        return Err(
                            "service operation precedes construction or follows demolition".into(),
                        );
                    }
                }
            }
            source.add(entry.kind, entry.amount_cents)?;
        }
        balance = if entry.kind.income() {
            balance.checked_add(entry.amount_cents)
        } else {
            balance.checked_sub(entry.amount_cents)
        }
        .ok_or("finance entry cash overflows or is negative")?;
        if balance != entry.balance_cents {
            return Err("finance running balance is inconsistent".into());
        }
        total.add(entry.kind, entry.amount_cents)?;
        expected_id = expected_id.checked_add(1).ok_or("finance ID overflow")?;
        tick = entry.tick;
    }
    if ledger.next_entry_id != expected_id
        || balance != park.cash_cents
        || ledger
            .opening_ride_revenue_cents
            .checked_add(total.ride_revenue_cents)
            != Some(ride_revenue)
    {
        return Err("finance final cash, ID or ride receipts disagree".into());
    }
    for ride in park.all_rides() {
        let source = ride_sources
            .get(&ride.id)
            .cloned()
            .unwrap_or_else(|| RideFinanceTotals::new(ride.id));
        let carousel = park.carousel(ride.id);
        if ride_opening
            .get(&ride.id)
            .copied()
            .unwrap_or(0)
            .checked_add(source.revenue_cents)
            != Some(ride.revenue_cents)
            || source.construction_cents
                != carousel.map_or(park.track_economy.construction_cents, |c| {
                    c.construction_cost_cents
                })
            || source.demolition_refunds_cents
                != carousel.map_or(park.track_economy.refund_cents, |c| {
                    c.demolition_refund_cents
                })
            || source.upkeep_cents != park.ride_running_costs(ride.id).unwrap().upkeep_cents
        {
            return Err("ride income disagrees with attributed cash history".into());
        }
    }
    for service in &park.services {
        let baseline = opening.get(&service.id);
        let initial_served = baseline.map_or(0, |entry| entry.served);
        let initial_revenue = baseline.map_or(0, |entry| entry.revenue_cents);
        let source = sources
            .get(&service.id)
            .cloned()
            .unwrap_or_else(|| ServiceFinanceTotals::new(service.id));
        if initial_revenue.checked_add(source.revenue_cents) != Some(service.revenue_cents)
            || source.upkeep_cents != service.upkeep_cents
            || source.supplies_cents != service.supplies_cents
            || source.construction_cents != service.construction_cost_cents
            || source.demolition_refunds_cents != service.demolition_refund_cents
            || service.upkeep_progress >= SERVICE_UPKEEP_TICKS
            || service.upkeep_cents % u64::from(service.kind.upkeep_cost_cents()) != 0
            || u128::from(service.upkeep_cents / u64::from(service.kind.upkeep_cost_cents()))
                * u128::from(SERVICE_UPKEEP_TICKS)
                + u128::from(service.upkeep_progress)
                > u128::from(park.ticks - ledger.opening_tick)
            || u128::from(service.supplies_cents)
                != u128::from(service.served - initial_served)
                    * u128::from(service.kind.supply_cost_cents())
            || (service.open && service.closed_for_funds)
        {
            return Err("service costs, opening history or operating state disagree".into());
        }
    }
    // Finite guest funds and no debt bound every transaction total, and thus
    // the positive-amount entry allocator, far below u64 exhaustion.
    let receipts = park
        .guests
        .iter()
        .try_fold(0_u64, |sum, guest| sum.checked_add(guest.spent_cents))
        .ok_or("guest receipt sum overflows")?;
    let expenses = total
        .upkeep_cents
        .checked_add(total.supplies_cents)
        .and_then(|costs| costs.checked_add(total.construction_cents))
        .ok_or("cost sum overflows")?;
    if INITIAL_PARK_CASH_CENTS
        .checked_add(receipts)
        .and_then(|funds| funds.checked_add(total.demolition_refunds_cents))
        .and_then(|funds| funds.checked_sub(expenses))
        != Some(park.cash_cents)
    {
        return Err("finance receipts and expenses do not conserve cash".into());
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;
