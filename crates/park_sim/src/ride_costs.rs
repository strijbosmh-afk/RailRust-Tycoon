//! Recurring operating costs, charged before departures with no debt.
use crate::{FinanceKind, Park, RideId, RideKind};
pub const RIDE_UPKEEP_TICKS: u16 = 900;
pub const fn ride_upkeep_cost_cents(kind: RideKind) -> u32 {
    match kind {
        RideKind::Coaster => 100,
        RideKind::Carousel => 50,
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RideCostRecord {
    pub(crate) ride: RideId,
    pub(crate) started_tick: u64,
    pub(crate) upkeep_progress: u16,
    pub(crate) upkeep_cents: u64,
    pub(crate) closed_for_funds: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideRunningCosts {
    pub cost_per_interval_cents: u32,
    pub upkeep_progress: u16,
    pub upkeep_cents: u64,
    pub closed_for_funds: bool,
}
impl Park {
    pub fn ride_running_costs(&self, id: RideId) -> Option<RideRunningCosts> {
        let kind = self.ride_kind(id)?;
        let record = self.ride_costs.iter().find(|r| r.ride == id);
        Some(RideRunningCosts {
            cost_per_interval_cents: ride_upkeep_cost_cents(kind),
            upkeep_progress: record.map_or(0, |r| r.upkeep_progress),
            upkeep_cents: record.map_or(0, |r| r.upkeep_cents),
            closed_for_funds: record.is_some_and(|r| r.closed_for_funds),
        })
    }
    pub(crate) fn clear_ride_funds_closure(&mut self, id: RideId) {
        if let Some(record) = self.ride_costs.iter_mut().find(|r| r.ride == id) {
            record.closed_for_funds = false;
        }
    }
    pub(crate) fn step_ride_upkeep(&mut self) {
        let mut ids = self
            .all_rides()
            .filter(|ride| self.ride_is_open(ride.id) && self.ride_available(ride.id))
            .map(|ride| ride.id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        for id in ids {
            let index = self
                .ride_costs
                .binary_search_by_key(&id, |r| r.ride)
                .unwrap_or_else(|index| {
                    self.ride_costs.insert(
                        index,
                        RideCostRecord {
                            ride: id,
                            started_tick: self.ticks - 1,
                            upkeep_progress: 0,
                            upkeep_cents: 0,
                            closed_for_funds: false,
                        },
                    );
                    index
                });
            let amount = u64::from(ride_upkeep_cost_cents(self.ride_kind(id).unwrap()));
            if self.ride_costs[index].upkeep_progress + 1 < RIDE_UPKEEP_TICKS {
                self.ride_costs[index].upkeep_progress += 1;
            } else if self.cash_cents < amount {
                self.set_ride_open(id, false).expect("known installed ride");
                self.ride_costs[index].closed_for_funds = true;
            } else {
                self.ride_costs[index].upkeep_progress = 0;
                self.ride_costs[index].upkeep_cents += amount;
                self.record_ride_transaction(FinanceKind::RideUpkeep, id, amount);
            }
        }
    }
}
pub(crate) fn validate(park: &Park) -> Result<(), String> {
    if park.ride_costs.len() > crate::MAX_RIDE_RECORDS {
        return Err("too many ride cost records".into());
    }
    let mut previous = None;
    for record in &park.ride_costs {
        let kind = park
            .ride_kind(record.ride)
            .ok_or("ride costs have an unknown owner")?;
        let cost = u64::from(ride_upkeep_cost_cents(kind));
        if previous.is_some_and(|id| id >= record.ride)
            || record.started_tick > park.ticks
            || record.upkeep_progress >= RIDE_UPKEEP_TICKS
            || !record.upkeep_cents.is_multiple_of(cost)
            || u128::from(record.upkeep_cents / cost) * u128::from(RIDE_UPKEEP_TICKS)
                + u128::from(record.upkeep_progress)
                > u128::from(park.ticks - record.started_tick)
            || (record.closed_for_funds
                && (park.ride_is_open(record.ride)
                    || record.upkeep_progress != RIDE_UPKEEP_TICKS - 1))
        {
            return Err("ride costs, elapsed time or funds-closure state disagree".into());
        }
        previous = Some(record.ride);
    }
    Ok(())
}
