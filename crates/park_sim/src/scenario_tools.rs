//! Configurable sandbox rules and portable, original scenario definitions.
use crate::{ChargingRule, Park, RideModel, ServiceKind};

pub const MAX_SCENARIO_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CampaignScenario {
    Meadow,
    Boardwalk,
    CedarValley,
    ResearchRidge,
    Lakeside,
    GrandFestival,
}
impl CampaignScenario {
    pub const ALL: [Self; 6] = [
        Self::Meadow,
        Self::Boardwalk,
        Self::CedarValley,
        Self::ResearchRidge,
        Self::Lakeside,
        Self::GrandFestival,
    ];
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap()
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Meadow => "Meadow Challenge",
            Self::Boardwalk => "Boardwalk Sprint",
            Self::CedarValley => "Cedar Valley",
            Self::ResearchRidge => "Research Ridge",
            Self::Lakeside => "Lakeside Services",
            Self::GrandFestival => "Grand Festival",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Meadow => "Welcome visitors and earn your first award with free park entry.",
            Self::Boardwalk => {
                "Meet the visitor and cash targets before ten minutes expire. Rides are free."
            }
            Self::CedarValley => {
                "Build a happy destination. Welcome 30 visitors and sustain 65 satisfaction."
            }
            Self::ResearchRidge => {
                "Develop a new attraction through funded research while completing 20 rides."
            }
            Self::Lakeside => {
                "Provide three facilities and serve 12 customers while welcoming 30 visitors."
            }
            Self::GrandFestival => {
                "Welcome 60 visitors, earn $150 in sales and maintain 65 satisfaction."
            }
        }
    }
    pub fn legacy(self) -> Option<crate::ScenarioKind> {
        match self {
            Self::Meadow => Some(crate::ScenarioKind::MeadowChallenge),
            Self::Boardwalk => Some(crate::ScenarioKind::BoardwalkSprint),
            _ => None,
        }
    }
    pub fn starting_budget(self) -> u64 {
        match self {
            Self::Meadow | Self::Boardwalk => 100_000,
            Self::GrandFestival => 250_000,
            _ => 200_000,
        }
    }
    pub fn definition(self) -> ChallengeDefinition {
        use ObjectiveMetric::*;
        let goals = match self {
            Self::Meadow => vec![(Arrivals, 20), (Rides, 5), (Cash, 75_000)],
            Self::Boardwalk => vec![(Arrivals, 24), (Rides, 12), (Cash, 90_000)],
            Self::CedarValley => vec![(Arrivals, 30), (Satisfaction, 65)],
            Self::ResearchRidge => vec![(ResearchProjects, 1), (Rides, 20)],
            Self::Lakeside => vec![(Facilities, 3), (ServiceCustomers, 12), (Arrivals, 30)],
            Self::GrandFestival => vec![(Arrivals, 60), (Revenue, 15_000), (Satisfaction, 65)],
        };
        ChallengeDefinition {
            name: self.label().into(),
            description: self.description().into(),
            campaign: Some(self),
            goals: goals
                .into_iter()
                .map(|(metric, target)| Objective { metric, target })
                .collect(),
            deadline_ticks: match self {
                Self::Meadow => None,
                Self::Boardwalk => Some(18_000),
                _ => Some(36_000),
            },
            charging_rule: self
                .legacy()
                .map_or(ChargingRule::Flexible, |k| k.charging_rule()),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ObjectiveMetric {
    Arrivals,
    Rides,
    ServiceCustomers,
    Revenue,
    OperatingProfit,
    Cash,
    Satisfaction,
    Facilities,
    ParkValue,
    ResearchProjects,
}
impl ObjectiveMetric {
    pub const ALL: [Self; 10] = [
        Self::Arrivals,
        Self::Rides,
        Self::ServiceCustomers,
        Self::Revenue,
        Self::OperatingProfit,
        Self::Cash,
        Self::Satisfaction,
        Self::Facilities,
        Self::ParkValue,
        Self::ResearchProjects,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Arrivals => "New arrivals",
            Self::Rides => "Completed rides",
            Self::ServiceCustomers => "Customers served",
            Self::Revenue => "Sales in cents",
            Self::OperatingProfit => "Operating profit in cents",
            Self::Cash => "Cash in cents",
            Self::Satisfaction => "Guest satisfaction",
            Self::Facilities => "Installed facilities",
            Self::ParkValue => "Park value in cents",
            Self::ResearchProjects => "Research projects",
        }
    }
    fn cumulative(self) -> bool {
        matches!(
            self,
            Self::Arrivals
                | Self::Rides
                | Self::ServiceCustomers
                | Self::Revenue
                | Self::ResearchProjects
        )
    }
    pub fn step(self) -> u64 {
        match self {
            Self::Revenue | Self::OperatingProfit | Self::Cash | Self::ParkValue => 10_000,
            Self::Satisfaction => 5,
            Self::ResearchProjects | Self::Facilities => 1,
            _ => 5,
        }
    }
    fn maximum(self) -> u64 {
        match self {
            Self::Satisfaction => 100,
            Self::ResearchProjects => 12,
            Self::Facilities => 256,
            _ => 1_000_000_000,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Objective {
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub metric: ObjectiveMetric,
    pub target: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeDefinition {
    pub name: String,
    pub description: String,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub campaign: Option<CampaignScenario>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,4>")]
    pub goals: Vec<Objective>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub deadline_ticks: Option<u64>,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub charging_rule: ChargingRule,
}
impl Default for ChallengeDefinition {
    fn default() -> Self {
        Self {
            name: "My Scenario".into(),
            description: "Build and operate your own park.".into(),
            campaign: None,
            goals: vec![Objective {
                metric: ObjectiveMetric::Arrivals,
                target: 20,
            }],
            deadline_ticks: Some(18_000),
            charging_rule: ChargingRule::Flexible,
        }
    }
}
impl ChallengeDefinition {
    pub fn validate(&self) -> Result<(), String> {
        if !valid_name(&self.name)
            || self.description.chars().count() > 240
            || self.description.chars().any(char::is_control)
            || self.goals.is_empty()
            || self.goals.len() > 4
            || self
                .deadline_ticks
                .is_some_and(|t| !(900..=216_000).contains(&t))
        {
            return Err("Use a name of 1–40 characters, up to four goals and a deadline of 30 seconds to two hours.".into());
        }
        for (i, g) in self.goals.iter().enumerate() {
            if g.target == 0
                || g.target > g.metric.maximum()
                || self.goals[..i].iter().any(|old| old.metric == g.metric)
            {
                return Err("Goals need distinct metrics and valid positive targets.".into());
            }
        }
        if self.campaign.is_some_and(|c| c.definition() != *self) {
            return Err("Built-in campaign definitions cannot be altered.".into());
        }
        Ok(())
    }
}
fn valid_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name.trim() == name
        && name.chars().count() <= 40
        && !name.chars().any(char::is_control)
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChallengeState {
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub definition: ChallengeDefinition,
    pub started_tick: u64,
    #[serde(deserialize_with = "crate::saves::bounded_vec::<_,_,4>")]
    pub baselines: Vec<u64>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub completed_tick: Option<u64>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub failed_tick: Option<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlayRules {
    pub starting_cash_cents: u64,
    pub all_research: bool,
    pub loans_allowed: bool,
    pub rides: [bool; 14],
    pub facilities: [bool; 8],
    #[serde(deserialize_with = "crate::saves::required_object_option")]
    pub challenge: Option<ChallengeState>,
}
impl Default for PlayRules {
    fn default() -> Self {
        Self {
            starting_cash_cents: crate::INITIAL_PARK_CASH_CENTS,
            all_research: false,
            loans_allowed: true,
            rides: [true; 14],
            facilities: [true; 8],
            challenge: None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChallengeStatus {
    pub definition: ChallengeDefinition,
    pub values: Vec<u64>,
    pub started_tick: u64,
    pub deadline_tick: Option<u64>,
    pub completed_tick: Option<u64>,
    pub failed_tick: Option<u64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioDocument {
    format: String,
    version: u32,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub definition: ChallengeDefinition,
    park: String,
}
impl ScenarioDocument {
    pub fn from_park(park: &Park, definition: ChallengeDefinition) -> Result<Self, String> {
        if definition.campaign.is_some() {
            return Err("Export a custom definition without a campaign award.".into());
        }
        definition.validate()?;
        let mut template = park.clone();
        template.clear_challenge();
        Ok(Self {
            format: "parkbound-scenario".into(),
            version: 1,
            definition,
            park: String::from_utf8(template.to_save_bytes().map_err(|e| e.to_string())?).unwrap(),
        })
    }
    pub fn template(&self) -> Result<Park, String> {
        self.validate()?;
        let park = Park::from_save_bytes(self.park.as_bytes()).map_err(|e| e.to_string())?;
        if park.width() != park.height()
            || !(32..=crate::MAX_MAP_SIZE).contains(&park.width())
            || park.scenario_status().is_some()
            || park.challenge_status().is_some()
        {
            return Err(
                "Scenario templates require a square playable map without an active challenge."
                    .into(),
            );
        }
        Ok(park)
    }
    fn validate(&self) -> Result<(), String> {
        if self.format != "parkbound-scenario" || self.version != 1 {
            return Err("Unsupported scenario format or version.".into());
        }
        if self.park.len() > crate::MAX_SAVE_BYTES || self.definition.campaign.is_some() {
            return Err("Scenario data or campaign marker is invalid.".into());
        }
        self.definition.validate()
    }
    pub fn play(&self) -> Result<Park, String> {
        let mut park = self.template()?;
        park.start_custom_challenge(self.definition.clone())?;
        Ok(park)
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.template()?;
        let bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_SCENARIO_BYTES {
            return Err("Scenario file exceeds 32 MiB.".into());
        }
        Ok(bytes)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_SCENARIO_BYTES {
            return Err("Scenario file exceeds 32 MiB.".into());
        }
        let doc: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid scenario file. {e}"))?;
        doc.template()?;
        Ok(doc)
    }
}

impl Park {
    pub fn starting_cash_cents(&self) -> u64 {
        self.play_rules
            .as_ref()
            .map_or(crate::INITIAL_PARK_CASH_CENTS, |r| r.starting_cash_cents)
    }
    pub fn all_research_available(&self) -> bool {
        self.play_rules.as_ref().is_some_and(|r| r.all_research)
    }
    pub fn loans_allowed(&self) -> bool {
        self.play_rules.as_ref().is_none_or(|r| r.loans_allowed)
    }
    pub fn ride_content_available(&self, model: RideModel) -> bool {
        self.play_rules
            .as_ref()
            .is_none_or(|r| r.rides[RideModel::ALL.iter().position(|m| *m == model).unwrap()])
    }
    pub fn facility_content_available(&self, kind: ServiceKind) -> bool {
        self.play_rules
            .as_ref()
            .is_none_or(|r| r.facilities[ServiceKind::ALL.iter().position(|m| *m == kind).unwrap()])
    }
    pub fn set_content_available(
        &mut self,
        ride: Option<RideModel>,
        facility: Option<ServiceKind>,
        available: bool,
    ) {
        let rules = self.play_rules.get_or_insert_with(PlayRules::default);
        if let Some(model) = ride {
            rules.rides[RideModel::ALL.iter().position(|m| *m == model).unwrap()] = available;
        }
        if let Some(kind) = facility {
            rules.facilities[ServiceKind::ALL.iter().position(|m| *m == kind).unwrap()] = available;
        }
    }
    pub fn set_sandbox_research(&mut self, all: bool) {
        self.play_rules
            .get_or_insert_with(PlayRules::default)
            .all_research = all;
    }
    pub fn set_loans_allowed(&mut self, allowed: bool) {
        self.play_rules
            .get_or_insert_with(PlayRules::default)
            .loans_allowed = allowed;
    }
    /// Authoring adjusts the opening budget and all corresponding running balances.
    pub fn set_starting_budget(&mut self, budget: u64) -> Result<(), String> {
        if !(10_000..=10_000_000).contains(&budget) {
            return Err("Starting budget must be $100–$100,000.".into());
        }
        let delta = i128::from(budget) - i128::from(self.starting_cash_cents());
        let shift = |cash: u64| {
            u64::try_from(i128::from(cash) + delta)
                .map_err(|_| "Budget must cover the park's existing spending.".to_string())
        };
        let mut candidate = self.clone();
        candidate.cash_cents = shift(candidate.cash_cents)?;
        candidate.finances.opening_cash_cents = shift(candidate.finances.opening_cash_cents)?;
        for e in &mut candidate.finances.entries {
            e.balance_cents = shift(e.balance_cents)?;
        }
        if let Some(m) = candidate.management.as_mut() {
            for r in &mut m.reports {
                r.cash_cents = shift(r.cash_cents)?;
            }
        }
        candidate
            .play_rules
            .get_or_insert_with(PlayRules::default)
            .starting_cash_cents = budget;
        *self = candidate;
        Ok(())
    }
    pub fn clear_challenge(&mut self) {
        self.scenario = None;
        if let Some(rules) = self.play_rules.as_mut() {
            rules.challenge = None;
        }
    }
    pub fn start_custom_challenge(
        &mut self,
        definition: ChallengeDefinition,
    ) -> Result<(), String> {
        if self.scenario.is_some() || self.challenge_status().is_some() {
            return Err("This park already has a challenge.".into());
        }
        definition.validate()?;
        if definition
            .deadline_ticks
            .is_some_and(|duration| self.ticks.checked_add(duration).is_none())
        {
            return Err("Scenario deadline exceeds the supported timeline.".into());
        }
        self.set_charging_rule(definition.charging_rule)
            .map_err(|e| e.to_string())?;
        let baselines = definition
            .goals
            .iter()
            .map(|g| {
                if g.metric.cumulative() {
                    self.objective_value(g.metric)
                } else {
                    0
                }
            })
            .collect();
        self.play_rules
            .get_or_insert_with(PlayRules::default)
            .challenge = Some(ChallengeState {
            definition,
            started_tick: self.ticks,
            baselines,
            completed_tick: None,
            failed_tick: None,
        });
        Ok(())
    }
    pub fn challenge_status(&self) -> Option<ChallengeStatus> {
        let s = self.play_rules.as_ref()?.challenge.as_ref()?;
        Some(ChallengeStatus {
            definition: s.definition.clone(),
            values: s
                .definition
                .goals
                .iter()
                .zip(&s.baselines)
                .map(|(g, base)| self.objective_value(g.metric).saturating_sub(*base))
                .collect(),
            started_tick: s.started_tick,
            deadline_tick: s
                .definition
                .deadline_ticks
                .and_then(|d| s.started_tick.checked_add(d)),
            completed_tick: s.completed_tick,
            failed_tick: s.failed_tick,
        })
    }
    pub fn required_charging_rule(&self) -> Option<ChargingRule> {
        self.scenario
            .as_ref()
            .map(|s| s.kind.charging_rule())
            .or_else(|| {
                self.play_rules
                    .as_ref()?
                    .challenge
                    .as_ref()
                    .map(|s| s.definition.charging_rule)
            })
    }
    pub fn objective_value(&self, metric: ObjectiveMetric) -> u64 {
        match metric {
            ObjectiveMetric::Arrivals => self.visitors.spawned,
            ObjectiveMetric::Rides => {
                self.guests.iter().map(|g| g.rides_completed).sum::<u64>()
                    + self
                        .visitors
                        .departed
                        .rides
                        .iter()
                        .map(|r| r.completed)
                        .sum::<u64>()
            }
            ObjectiveMetric::ServiceCustomers => self.services.iter().map(|s| s.served).sum(),
            ObjectiveMetric::Revenue => {
                let f = self.finance_summary();
                f.rides.revenue_cents + f.services.revenue_cents + f.admission_revenue_cents
            }
            ObjectiveMetric::Cash => self.cash_cents,
            ObjectiveMetric::OperatingProfit => {
                self.finance_summary().operating_profit_cents.max(0) as u64
            }
            ObjectiveMetric::Satisfaction => {
                if self.guests.is_empty() {
                    0
                } else {
                    self.guests
                        .iter()
                        .map(|g| u64::from(g.satisfaction))
                        .sum::<u64>()
                        / self.guests.len() as u64
                }
            }
            ObjectiveMetric::Facilities => {
                self.services.iter().filter(|s| s.object.is_some()).count() as u64
            }
            ObjectiveMetric::ParkValue => self.park_value().park_cents,
            ObjectiveMetric::ResearchProjects => self
                .management_status()
                .research_spent
                .iter()
                .map(|v| v / crate::RESEARCH_PROJECT_CENTS)
                .sum(),
        }
    }
    pub(crate) fn step_custom_challenge(&mut self) {
        let Some(state) = self.play_rules.as_ref().and_then(|r| r.challenge.as_ref()) else {
            return;
        };
        if state.completed_tick.is_some() || state.failed_tick.is_some() {
            return;
        }
        let won = state
            .definition
            .goals
            .iter()
            .zip(&state.baselines)
            .all(|(g, base)| self.objective_value(g.metric).saturating_sub(*base) >= g.target);
        let expired = state
            .definition
            .deadline_ticks
            .is_some_and(|duration| self.ticks.saturating_sub(state.started_tick) >= duration);
        let state = self
            .play_rules
            .as_mut()
            .unwrap()
            .challenge
            .as_mut()
            .unwrap();
        if won {
            state.completed_tick = Some(self.ticks);
        } else if expired {
            state.failed_tick = Some(self.ticks);
        }
    }
}
pub(crate) fn validate(park: &Park) -> Result<(), String> {
    let Some(r) = &park.play_rules else {
        return Ok(());
    };
    if !(10_000..=10_000_000).contains(&r.starting_cash_cents) {
        return Err("Invalid starting budget.".into());
    }
    if let Some(s) = &r.challenge {
        s.definition.validate()?;
        if park.scenario.is_some()
            || s.started_tick > park.ticks
            || s.baselines.len() != s.definition.goals.len()
            || s.completed_tick.is_some() && s.failed_tick.is_some()
            || park.visitors.charging_rule != s.definition.charging_rule
        {
            return Err("Invalid custom challenge state.".into());
        }
        for (g, b) in s.definition.goals.iter().zip(&s.baselines) {
            if (!g.metric.cumulative() && *b != 0) || *b > park.objective_value(g.metric) {
                return Err("Invalid objective baseline.".into());
            }
        }
        let deadline = s
            .definition
            .deadline_ticks
            .and_then(|d| s.started_tick.checked_add(d));
        if s.definition.deadline_ticks.is_some() && deadline.is_none() {
            return Err("Invalid challenge deadline.".into());
        }
        for tick in [s.completed_tick, s.failed_tick].into_iter().flatten() {
            if tick < s.started_tick || tick > park.ticks {
                return Err("Invalid challenge outcome time.".into());
            }
        }
        if s.failed_tick.is_some_and(|t| Some(t) != deadline)
            || s.completed_tick
                .is_some_and(|t| deadline.is_some_and(|d| t > d))
            || (deadline.is_some_and(|d| park.ticks >= d)
                && s.completed_tick.is_none()
                && s.failed_tick.is_none())
        {
            return Err("Invalid challenge deadline.".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
