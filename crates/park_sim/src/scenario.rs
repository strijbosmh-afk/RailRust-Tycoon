//! One authored park challenge with immutable rules and a saved milestone.
use crate::{ChargingRule, Park};

pub const MEADOW_ARRIVAL_GOAL: u64 = 20;
pub const MEADOW_RIDE_GOAL: u64 = 5;
pub const MEADOW_CASH_GOAL_CENTS: u64 = 75_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScenarioKind {
    MeadowChallenge,
}
impl ScenarioKind {
    pub fn label(self) -> &'static str {
        "Meadow Challenge"
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioState {
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub kind: ScenarioKind,
    pub started_tick: u64,
    pub baseline_arrivals: u64,
    pub baseline_rides: u64,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub completed_tick: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScenarioStatus {
    pub kind: ScenarioKind,
    pub started_tick: u64,
    pub arrivals: u64,
    pub rides: u64,
    pub cash_cents: u64,
    pub completed_tick: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioError {
    AlreadyStarted,
    RadioActive,
}
impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AlreadyStarted => "Meadow Challenge is already active",
            Self::RadioActive => {
                "Wait for the current radio campaign to finish before starting Meadow Challenge"
            }
        })
    }
}
impl std::error::Error for ScenarioError {}

impl Park {
    pub fn scenario_status(&self) -> Option<ScenarioStatus> {
        let state = self.scenario.as_ref()?;
        Some(ScenarioStatus {
            kind: state.kind,
            started_tick: state.started_tick,
            arrivals: self.visitors.spawned - state.baseline_arrivals,
            rides: self.completed_visitor_rides() - state.baseline_rides,
            cash_cents: self.cash_cents,
            completed_tick: state.completed_tick,
        })
    }

    pub fn start_meadow_challenge(&mut self) -> Result<(), ScenarioError> {
        if self.scenario.is_some() {
            return Err(ScenarioError::AlreadyStarted);
        }
        if self.visitors.radio_ticks_remaining != 0 {
            return Err(ScenarioError::RadioActive);
        }
        let baseline_arrivals = self.visitors.spawned;
        let baseline_rides = self.completed_visitor_rides();
        self.set_charging_rule(ChargingRule::FreeEntry)
            .expect("sandbox permits the authored rule");
        self.scenario = Some(ScenarioState {
            kind: ScenarioKind::MeadowChallenge,
            started_tick: self.ticks,
            baseline_arrivals,
            baseline_rides,
            completed_tick: None,
        });
        Ok(())
    }

    fn completed_visitor_rides(&self) -> u64 {
        self.guests
            .iter()
            .fold(0u64, |sum, guest| sum.saturating_add(guest.rides_completed))
            .saturating_add(
                self.visitors
                    .departed
                    .rides
                    .iter()
                    .fold(0u64, |sum, row| sum.saturating_add(row.completed)),
            )
    }

    pub(crate) fn step_scenario(&mut self) {
        let Some(state) = self.scenario.as_ref() else {
            return;
        };
        if state.completed_tick.is_some() {
            return;
        }
        let arrivals = self.visitors.spawned - state.baseline_arrivals;
        let rides = self.completed_visitor_rides() - state.baseline_rides;
        if arrivals >= MEADOW_ARRIVAL_GOAL
            && rides >= MEADOW_RIDE_GOAL
            && self.cash_cents >= MEADOW_CASH_GOAL_CENTS
        {
            self.scenario.as_mut().unwrap().completed_tick = Some(self.ticks);
        }
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    let Some(state) = &park.scenario else {
        return Ok(());
    };
    let rides = park.completed_visitor_rides();
    if state.started_tick > park.ticks
        || state.baseline_arrivals > park.visitors.spawned
        || state.baseline_rides > rides
        || state
            .completed_tick
            .is_some_and(|tick| tick < state.started_tick || tick > park.ticks)
        || (state.completed_tick.is_some()
            && (park.visitors.spawned - state.baseline_arrivals < MEADOW_ARRIVAL_GOAL
                || rides - state.baseline_rides < MEADOW_RIDE_GOAL))
        || park.visitors.charging_rule != ChargingRule::FreeEntry
        || park.visitors.entry_price_cents != 0
        || park.visitors.radio_ticks_remaining != 0
    {
        return Err("scenario rules, progress or completion are invalid".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TilePos, VisitorError};

    #[test]
    fn challenge_locks_rules_and_completes_only_after_all_goals() {
        let (mut park, _) = crate::carousel::tests::fixture(true);
        let entry = park
            .objects()
            .iter()
            .find(|path| path.is_public_path() && !park.path_neighbors(path.id).is_empty())
            .unwrap()
            .id;
        park.set_visitor_entry(entry).unwrap();
        park.set_visitor_arrivals(true).unwrap();
        park.set_park_entry_price(300).unwrap();
        park.start_meadow_challenge().unwrap();
        assert_eq!(park.visitor_flow_stats().entry_price_cents, 0);
        assert_eq!(
            park.set_charging_rule(ChargingRule::Flexible),
            Err(VisitorError::ScenarioLocked)
        );
        assert_eq!(
            park.start_radio_campaign(),
            Err(VisitorError::RadioUnavailable)
        );
        assert_eq!(park.scenario_status().unwrap().arrivals, 0);
        park.start_flyer_campaign().unwrap();
        park.borrow_loan(100_000).unwrap();
        for _ in 0..15_000 {
            park.advance(Speed::Normal);
            if park.scenario_status().unwrap().completed_tick.is_some() {
                break;
            }
        }
        let status = park.scenario_status().unwrap();
        assert!(status.arrivals >= MEADOW_ARRIVAL_GOAL);
        assert!(status.rides >= MEADOW_RIDE_GOAL);
        assert!(status.cash_cents >= MEADOW_CASH_GOAL_CENTS);
        assert!(status.completed_tick.is_some());
        let loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(loaded, park);
        let completion = status.completed_tick;
        for _ in 0..100 {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.scenario_status().unwrap().completed_tick, completion);
    }

    #[test]
    fn challenge_waits_for_cash_goal_and_paused_time_does_not_complete() {
        let (mut park, _) = crate::carousel::tests::fixture(false);
        park.start_meadow_challenge().unwrap();
        park.visitors.spawned = MEADOW_ARRIVAL_GOAL;
        park.guests[0].rides_completed = MEADOW_RIDE_GOAL;
        park.cash_cents = MEADOW_CASH_GOAL_CENTS - 1;
        assert_eq!(park.scenario_status().unwrap().completed_tick, None);
        park.advance(Speed::Paused);
        assert_eq!(park.scenario_status().unwrap().completed_tick, None);
        park.cash_cents = MEADOW_CASH_GOAL_CENTS;
        park.advance(Speed::Normal);
        assert_eq!(park.scenario_status().unwrap().completed_tick, Some(1));
    }

    #[test]
    fn active_radio_refuses_challenge_without_changing_park() {
        let mut park = Park::new(32, 32);
        let entry = park.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
        park.place_path(TilePos { x: 6, y: 5 }, 0).unwrap();
        park.set_visitor_entry(entry).unwrap();
        park.set_visitor_arrivals(true).unwrap();
        park.start_radio_campaign().unwrap();
        let before = park.clone();
        assert_eq!(
            park.start_meadow_challenge(),
            Err(ScenarioError::RadioActive)
        );
        assert_eq!(park, before);
    }

    #[test]
    fn schema_37_migrates_sandbox_and_schema_38_requires_scenario_field() {
        let park = Park::new(32, 32);
        let mut old: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        old["schema_version"] = serde_json::json!(37);
        old["content_version"] = serde_json::json!(37);
        crate::legacy_cost_test_support::strip_scenario_field(&mut old);
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).unwrap(),
            park
        );
        let mut current = old.clone();
        current["schema_version"] = serde_json::json!(38);
        current["content_version"] = serde_json::json!(38);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&current).unwrap()).is_err());
        old["park"]["scenario"] = serde_json::Value::Null;
        assert!(Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).is_err());
    }
}
