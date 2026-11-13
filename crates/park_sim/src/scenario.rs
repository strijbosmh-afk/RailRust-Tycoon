//! Authored park challenges with immutable rules and saved outcomes.
use crate::{ChargingRule, Park};

pub const MEADOW_ARRIVAL_GOAL: u64 = 20;
pub const MEADOW_RIDE_GOAL: u64 = 5;
pub const MEADOW_CASH_GOAL_CENTS: u64 = 75_000;
pub const BOARDWALK_ARRIVAL_GOAL: u64 = 24;
pub const BOARDWALK_RIDE_GOAL: u64 = 12;
pub const BOARDWALK_CASH_GOAL_CENTS: u64 = 90_000;
pub const BOARDWALK_DEADLINE_TICKS: u64 = 18_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScenarioKind {
    MeadowChallenge,
    BoardwalkSprint,
}
impl ScenarioKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::MeadowChallenge => "Meadow Challenge",
            Self::BoardwalkSprint => "Boardwalk Sprint",
        }
    }
    pub fn goals(self) -> (u64, u64, u64) {
        match self {
            Self::MeadowChallenge => (
                MEADOW_ARRIVAL_GOAL,
                MEADOW_RIDE_GOAL,
                MEADOW_CASH_GOAL_CENTS,
            ),
            Self::BoardwalkSprint => (
                BOARDWALK_ARRIVAL_GOAL,
                BOARDWALK_RIDE_GOAL,
                BOARDWALK_CASH_GOAL_CENTS,
            ),
        }
    }
    pub fn charging_rule(self) -> ChargingRule {
        match self {
            Self::MeadowChallenge => ChargingRule::FreeEntry,
            Self::BoardwalkSprint => ChargingRule::FreeRides,
        }
    }
    pub fn deadline_ticks(self) -> Option<u64> {
        match self {
            Self::MeadowChallenge => None,
            Self::BoardwalkSprint => Some(BOARDWALK_DEADLINE_TICKS),
        }
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
    #[serde(default, deserialize_with = "crate::saves::required_option")]
    pub failed_tick: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScenarioStatus {
    pub kind: ScenarioKind,
    pub started_tick: u64,
    pub arrivals: u64,
    pub rides: u64,
    pub cash_cents: u64,
    pub goal_arrivals: u64,
    pub goal_rides: u64,
    pub goal_cash_cents: u64,
    pub deadline_tick: Option<u64>,
    pub completed_tick: Option<u64>,
    pub failed_tick: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioError {
    AlreadyStarted,
    RadioActive,
}
impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AlreadyStarted => "A challenge is already active",
            Self::RadioActive => {
                "Wait for radio and themed promotions to finish before starting Meadow Challenge"
            }
        })
    }
}
impl std::error::Error for ScenarioError {}

impl Park {
    pub fn scenario_status(&self) -> Option<ScenarioStatus> {
        let state = self.scenario.as_ref()?;
        let (goal_arrivals, goal_rides, goal_cash_cents) = state.kind.goals();
        Some(ScenarioStatus {
            kind: state.kind,
            started_tick: state.started_tick,
            arrivals: self.visitors.spawned - state.baseline_arrivals,
            rides: self.completed_visitor_rides() - state.baseline_rides,
            cash_cents: self.cash_cents,
            goal_arrivals,
            goal_rides,
            goal_cash_cents,
            deadline_tick: state
                .kind
                .deadline_ticks()
                .and_then(|duration| state.started_tick.checked_add(duration)),
            completed_tick: state.completed_tick,
            failed_tick: state.failed_tick,
        })
    }

    pub fn start_meadow_challenge(&mut self) -> Result<(), ScenarioError> {
        self.start_scenario(ScenarioKind::MeadowChallenge)
    }

    pub fn start_boardwalk_sprint(&mut self) -> Result<(), ScenarioError> {
        self.start_scenario(ScenarioKind::BoardwalkSprint)
    }

    pub fn start_scenario(&mut self, kind: ScenarioKind) -> Result<(), ScenarioError> {
        if self.scenario.is_some() || self.challenge_status().is_some() {
            return Err(ScenarioError::AlreadyStarted);
        }
        if kind == ScenarioKind::MeadowChallenge
            && (self.visitors.radio_ticks_remaining != 0 || self.promotion_active())
        {
            return Err(ScenarioError::RadioActive);
        }
        if kind == ScenarioKind::MeadowChallenge
            && let Some(m) = self.management.as_mut()
            && m.funding == crate::ResearchFunding::High
        {
            m.funding = crate::ResearchFunding::Medium;
        }
        let baseline_arrivals = self.visitors.spawned;
        let baseline_rides = self.completed_visitor_rides();
        self.set_charging_rule(kind.charging_rule())
            .expect("sandbox permits the authored rule");
        self.scenario = Some(ScenarioState {
            kind,
            started_tick: self.ticks,
            baseline_arrivals,
            baseline_rides,
            completed_tick: None,
            failed_tick: None,
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
        if state.completed_tick.is_some() || state.failed_tick.is_some() {
            return;
        }
        let arrivals = self.visitors.spawned - state.baseline_arrivals;
        let rides = self.completed_visitor_rides() - state.baseline_rides;
        let (goal_arrivals, goal_rides, goal_cash_cents) = state.kind.goals();
        if arrivals >= goal_arrivals && rides >= goal_rides && self.cash_cents >= goal_cash_cents {
            self.scenario.as_mut().unwrap().completed_tick = Some(self.ticks);
        } else if state
            .kind
            .deadline_ticks()
            .and_then(|duration| state.started_tick.checked_add(duration))
            .is_some_and(|deadline| self.ticks >= deadline)
        {
            self.scenario.as_mut().unwrap().failed_tick = Some(self.ticks);
        }
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    let Some(state) = &park.scenario else {
        return Ok(());
    };
    let rides = park.completed_visitor_rides();
    let (goal_arrivals, goal_rides, _) = state.kind.goals();
    let deadline = match state.kind.deadline_ticks() {
        Some(duration) => Some(
            state
                .started_tick
                .checked_add(duration)
                .ok_or("scenario deadline overflows")?,
        ),
        None => None,
    };
    if state.started_tick > park.ticks
        || state.baseline_arrivals > park.visitors.spawned
        || state.baseline_rides > rides
        || state
            .completed_tick
            .is_some_and(|tick| tick < state.started_tick || tick > park.ticks)
        || state
            .failed_tick
            .is_some_and(|tick| tick < state.started_tick || tick > park.ticks)
        || state.completed_tick.is_some() && state.failed_tick.is_some()
        || deadline.is_none() && state.failed_tick.is_some()
        || deadline.is_some_and(|limit| {
            state.completed_tick.is_some_and(|tick| tick > limit)
                || state.failed_tick.is_some_and(|tick| tick != limit)
                || park.ticks >= limit
                    && state.completed_tick.is_none()
                    && state.failed_tick.is_none()
        })
        || (state.completed_tick.is_some()
            && (park.visitors.spawned - state.baseline_arrivals < goal_arrivals
                || rides - state.baseline_rides < goal_rides))
        || park.visitors.charging_rule != state.kind.charging_rule()
        || state.kind == ScenarioKind::MeadowChallenge
            && (park.visitors.entry_price_cents != 0 || park.visitors.radio_ticks_remaining != 0)
        || state.kind == ScenarioKind::BoardwalkSprint
            && park.all_rides().any(|ride| ride.ticket_price_cents != 0)
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
        old["park"].as_object_mut().unwrap().remove("decorations");
        old["park"].as_object_mut().unwrap().remove("path_economy");
        old["content_version"] = serde_json::json!(37);
        crate::legacy_cost_test_support::strip_scenario_field(&mut old);
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).unwrap(),
            park
        );
        let mut current = old.clone();
        current["schema_version"] = serde_json::json!(38);
        current["park"]
            .as_object_mut()
            .unwrap()
            .remove("decorations");
        current["park"]
            .as_object_mut()
            .unwrap()
            .remove("path_economy");
        current["content_version"] = serde_json::json!(38);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&current).unwrap()).is_err());
        old["park"]["scenario"] = serde_json::Value::Null;
        assert!(Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).is_err());
    }

    #[test]
    fn boardwalk_sprint_locks_free_rides_and_can_finish_before_deadline() {
        let (mut park, _) = crate::carousel::tests::fixture(true);
        let entry = park
            .objects()
            .iter()
            .find(|path| path.is_public_path() && !park.path_neighbors(path.id).is_empty())
            .unwrap()
            .id;
        park.set_visitor_entry(entry).unwrap();
        park.set_visitor_arrivals(true).unwrap();
        park.start_boardwalk_sprint().unwrap();
        assert!(park.all_rides().all(|ride| ride.ticket_price_cents == 0));
        assert_eq!(
            park.set_charging_rule(ChargingRule::Flexible),
            Err(VisitorError::SprintRuleLocked)
        );
        park.borrow_loan(100_000).unwrap();
        park.start_flyer_campaign().unwrap();
        park.start_radio_campaign().unwrap();
        for _ in 0..BOARDWALK_DEADLINE_TICKS {
            park.advance(Speed::Normal);
            if park.scenario_status().unwrap().completed_tick.is_some() {
                break;
            }
        }
        let status = park.scenario_status().unwrap();
        assert!(status.arrivals >= BOARDWALK_ARRIVAL_GOAL);
        assert!(status.rides >= BOARDWALK_RIDE_GOAL);
        assert!(status.cash_cents >= BOARDWALK_CASH_GOAL_CENTS);
        assert!(status.completed_tick.unwrap() <= status.deadline_tick.unwrap());
        assert_eq!(status.failed_tick, None);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn boardwalk_sprint_fails_at_deadline_and_outcome_is_sticky() {
        let mut park = Park::new(32, 32);
        park.start_boardwalk_sprint().unwrap();
        park.advance(Speed::Paused);
        assert_eq!(park.scenario_status().unwrap().failed_tick, None);
        for _ in 0..BOARDWALK_DEADLINE_TICKS - 1 {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.scenario_status().unwrap().failed_tick, None);
        park.advance(Speed::Normal);
        let status = park.scenario_status().unwrap();
        assert_eq!(status.failed_tick, status.deadline_tick);
        assert_eq!(status.completed_tick, None);
        for _ in 0..100 {
            park.advance(Speed::Normal);
        }
        assert_eq!(
            park.scenario_status().unwrap().failed_tick,
            status.failed_tick
        );
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn boardwalk_sprint_can_win_on_the_final_deadline_tick() {
        let (mut park, _) = crate::carousel::tests::fixture(false);
        park.start_boardwalk_sprint().unwrap();
        park.ticks = BOARDWALK_DEADLINE_TICKS - 1;
        park.visitors.spawned = BOARDWALK_ARRIVAL_GOAL;
        park.guests[0].rides_completed = BOARDWALK_RIDE_GOAL;
        park.cash_cents = BOARDWALK_CASH_GOAL_CENTS + 1_000;
        park.advance(Speed::Normal);
        let status = park.scenario_status().unwrap();
        assert_eq!(status.completed_tick, status.deadline_tick);
        assert_eq!(status.failed_tick, None);
    }

    #[test]
    fn schema_38_active_meadow_migrates_and_39_requires_failure_state() {
        let mut park = Park::new(32, 32);
        park.start_meadow_challenge().unwrap();
        let mut old: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        old["schema_version"] = serde_json::json!(38);
        old["park"].as_object_mut().unwrap().remove("decorations");
        old["park"].as_object_mut().unwrap().remove("path_economy");
        old["content_version"] = serde_json::json!(38);
        crate::legacy_cost_test_support::strip_scenario_field(&mut old);
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).unwrap(),
            park
        );
        old["schema_version"] = serde_json::json!(39);
        old["park"].as_object_mut().unwrap().remove("decorations");
        old["park"].as_object_mut().unwrap().remove("path_economy");
        old["content_version"] = serde_json::json!(39);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).is_err());
        old["schema_version"] = serde_json::json!(38);
        old["park"].as_object_mut().unwrap().remove("decorations");
        old["park"].as_object_mut().unwrap().remove("path_economy");
        old["content_version"] = serde_json::json!(38);
        old["park"]["scenario"]["kind"] = serde_json::json!("BoardwalkSprint");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).is_err());
    }
}
