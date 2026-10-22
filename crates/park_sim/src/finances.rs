//! Bounded chronological cash ledger. All amounts are integer cents.
use std::collections::BTreeMap;

use crate::{INITIAL_PARK_CASH_CENTS, MAX_SERVICES, Park, ServiceId};

pub const SERVICE_UPKEEP_TICKS: u16 = 90;
pub const FINANCE_LEDGER_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FinanceKind {
    RideRevenue,
    ServiceRevenue,
    ServiceUpkeep,
    ServiceSupplies,
}
impl FinanceKind {
    fn income(self) -> bool {
        matches!(self, Self::RideRevenue | Self::ServiceRevenue)
    }
}
impl std::fmt::Display for FinanceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::RideRevenue => "Ride tickets",
            Self::ServiceRevenue => "Service sale",
            Self::ServiceUpkeep => "Service upkeep",
            Self::ServiceSupplies => "Service supplies",
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
}
impl FinanceTotals {
    fn add(&mut self, kind: FinanceKind, amount: u64) -> Result<(), String> {
        let target = match kind {
            FinanceKind::RideRevenue => &mut self.ride_revenue_cents,
            FinanceKind::ServiceRevenue => &mut self.service_revenue_cents,
            FinanceKind::ServiceUpkeep => &mut self.upkeep_cents,
            FinanceKind::ServiceSupplies => &mut self.supplies_cents,
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
            .checked_sub(self.upkeep_cents)?
            .checked_sub(self.supplies_cents)
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
pub struct ServiceFinanceTotals {
    pub service: ServiceId,
    pub revenue_cents: u64,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
}
impl ServiceFinanceTotals {
    fn new(service: ServiceId) -> Self {
        Self {
            service,
            revenue_cents: 0,
            upkeep_cents: 0,
            supplies_cents: 0,
        }
    }
    fn add(&mut self, kind: FinanceKind, amount: u64) -> Result<(), String> {
        let target = match kind {
            FinanceKind::ServiceRevenue => &mut self.revenue_cents,
            FinanceKind::ServiceUpkeep => &mut self.upkeep_cents,
            FinanceKind::ServiceSupplies => &mut self.supplies_cents,
            FinanceKind::RideRevenue => return Err("ride income has a service source".into()),
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
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, MAX_SERVICES>")]
    pub opening_services: Vec<ServiceFinanceOpening>,
    pub archived_entries: u64,
    pub archived_through_tick: u64,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub archived: FinanceTotals,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, MAX_SERVICES>")]
    pub archived_services: Vec<ServiceFinanceTotals>,
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
            opening_services: Vec::new(),
            archived_entries: 0,
            archived_through_tick: 0,
            archived: FinanceTotals::default(),
            archived_services: Vec::new(),
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
        Self {
            opening_tick: park.ticks,
            opening_cash_cents: park.cash_cents,
            opening_ride_revenue_cents: park.ride.as_ref().map_or(0, |ride| ride.revenue_cents),
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
            self.archived_entries += 1;
            self.archived_through_tick = old.tick;
        }
        self.entries.push(FinanceEntry {
            id: self.next_entry_id,
            tick,
            kind,
            service,
            amount_cents: amount,
            balance_cents: balance,
        });
        self.next_entry_id += 1;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FinanceStats {
    pub cash_cents: u64,
    pub ride_revenue_cents: u64,
    pub service_revenue_cents: u64,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
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
        let ride_revenue_cents = self.ride.as_ref().map_or(0, |ride| ride.revenue_cents);
        let service_revenue_cents = self
            .services
            .iter()
            .map(|service| service.revenue_cents)
            .sum::<u64>();
        let upkeep_cents = self
            .services
            .iter()
            .map(|service| service.upkeep_cents)
            .sum::<u64>();
        let supplies_cents = self
            .services
            .iter()
            .map(|service| service.supplies_cents)
            .sum::<u64>();
        let total_costs_cents = upkeep_cents + supplies_cents;
        FinanceStats {
            cash_cents: self.cash_cents,
            ride_revenue_cents,
            service_revenue_cents,
            upkeep_cents,
            supplies_cents,
            total_costs_cents,
            net_profit_cents: ride_revenue_cents as i64 + service_revenue_cents as i64
                - total_costs_cents as i64,
            open_upkeep_per_minute_cents: self
                .services
                .iter()
                .filter(|service| service.open && service.object.is_some())
                .map(|service| u64::from(service.kind.upkeep_cost_cents()) * 20)
                .sum(),
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
            .push(self.ticks, kind, source, amount, self.cash_cents);
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
    let ledger = &park.finances;
    if ledger.opening_tick > park.ticks
        || ledger.archived_through_tick < ledger.opening_tick
        || ledger.archived_through_tick > park.ticks
        || ledger.entries.len() > FINANCE_LEDGER_CAPACITY
        || ledger.opening_services.len() > MAX_SERVICES
        || ledger.archived_services.len() > MAX_SERVICES
    {
        return Err("finance window or bounds are invalid".into());
    }
    let ride_revenue = park.ride.as_ref().map_or(0, |ride| ride.revenue_cents);
    if ledger.opening_ride_revenue_cents > ride_revenue {
        return Err("opening ride income exceeds history".into());
    }
    let mut opening = BTreeMap::new();
    let mut opening_revenue = ledger.opening_ride_revenue_cents;
    let mut last = None;
    for entry in &ledger.opening_services {
        let service = park
            .service(entry.service)
            .ok_or("opening service is unknown")?;
        if last.is_some_and(|id| id >= entry.service)
            || entry.served > service.served
            || entry.revenue_cents > service.revenue_cents
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
    let mut archived_check = FinanceTotals {
        ride_revenue_cents: ledger.archived.ride_revenue_cents,
        ..Default::default()
    };
    let ride_receipts = u128::from(ledger.archived.ride_revenue_cents);
    let max_fare_batch = crate::TRAIN_SEATS as u128 * u128::from(crate::INITIAL_GUEST_WALLET_CENTS);
    let mut minimum_archived = ride_receipts.div_ceil(max_fare_batch);
    let mut maximum_archived = ride_receipts;
    last = None;
    for entry in &ledger.archived_services {
        let service = park
            .service(entry.service)
            .ok_or("archived service source is invalid")?;
        if last.is_some_and(|id| id >= entry.service)
            || (entry.revenue_cents == 0 && entry.upkeep_cents == 0 && entry.supplies_cents == 0)
        {
            return Err("archived service source is invalid".into());
        }
        let upkeep_cost = u64::from(service.kind.upkeep_cost_cents());
        let supply_cost = u64::from(service.kind.supply_cost_cents());
        if entry.upkeep_cents % upkeep_cost != 0 || entry.supplies_cents % supply_cost != 0 {
            return Err("archived service expenses are not whole fixed-cost transactions".into());
        }
        let expenses = u128::from(entry.upkeep_cents / upkeep_cost)
            + u128::from(entry.supplies_cents / supply_cost);
        let receipts = u128::from(entry.revenue_cents);
        minimum_archived +=
            expenses + receipts.div_ceil(u128::from(crate::INITIAL_GUEST_WALLET_CENTS));
        maximum_archived += expenses + receipts;
        last = Some(entry.service);
        archived_check.add(FinanceKind::ServiceRevenue, entry.revenue_cents)?;
        archived_check.add(FinanceKind::ServiceUpkeep, entry.upkeep_cents)?;
        archived_check.add(FinanceKind::ServiceSupplies, entry.supplies_cents)?;
        sources.insert(entry.service, entry.clone());
    }
    if archived_check != ledger.archived
        || (ledger.archived_entries == 0
            && (ledger.archived != FinanceTotals::default()
                || ledger.archived_through_tick != ledger.opening_tick))
        || (ledger.archived_entries > 0
            && (ledger.entries.len() != FINANCE_LEDGER_CAPACITY
                || ledger.archived_through_tick == ledger.opening_tick
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
            || entry.tick <= ledger.opening_tick
            || entry.tick > park.ticks
            || entry.amount_cents == 0
        {
            return Err("finance entry order, tick or amount is invalid".into());
        }
        if entry.kind == FinanceKind::RideRevenue {
            if entry.service.is_some()
                || park.ride.is_none()
                || entry.amount_cents
                    > crate::TRAIN_SEATS as u64 * u64::from(crate::INITIAL_GUEST_WALLET_CENTS)
            {
                return Err("ride receipt source or amount is invalid".into());
            }
        } else {
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
            sources
                .entry(id)
                .or_insert_with(|| ServiceFinanceTotals::new(id))
                .add(entry.kind, entry.amount_cents)?;
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
        .ok_or("cost sum overflows")?;
    if INITIAL_PARK_CASH_CENTS
        .checked_add(receipts)
        .and_then(|funds| funds.checked_sub(expenses))
        != Some(park.cash_cents)
    {
        return Err("finance receipts and expenses do not conserve cash".into());
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;
