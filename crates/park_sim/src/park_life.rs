//! Opt-in park life for old saves, enabled for newly created parks.
use crate::{GuestId, ObjectId, Park, PathPoint, RideId};
use serde::{Deserialize, Serialize};
mod guests;
mod staff;
#[cfg(test)]
mod tests;
mod weather;

pub const PARK_STAFF_LIMIT: usize = 20;
pub const PARK_STAFF_WAGE_CENTS: u64 = 100;
pub const PARK_STAFF_WAGE_TICKS: u16 = 900;
const INCIDENT_LIMIT: usize = 32;
const PARTY_LIMIT: usize = 256;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Weather {
    #[default]
    Sunny,
    Hot,
    Rain,
    Storm,
}
impl Weather {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sunny => "Sunny",
            Self::Hot => "Hot",
            Self::Rain => "Rain",
            Self::Storm => "Thunderstorm",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum WeatherPolicy {
    #[default]
    Automatic,
    Sunny,
    Hot,
    Rain,
    Storm,
}
impl WeatherPolicy {
    pub fn next(self) -> Self {
        match self {
            Self::Automatic => Self::Sunny,
            Self::Sunny => Self::Hot,
            Self::Hot => Self::Rain,
            Self::Rain => Self::Storm,
            Self::Storm => Self::Automatic,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Automatic => "Changing weather",
            Self::Sunny => "Always sunny",
            Self::Hot => "Heat",
            Self::Rain => "Rain",
            Self::Storm => "Storm",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StaffRole {
    Mechanic,
    Handyman,
    Security,
    Entertainer,
    Attendant,
}
impl StaffRole {
    pub const ALL: [Self; 5] = [
        Self::Mechanic,
        Self::Handyman,
        Self::Security,
        Self::Entertainer,
        Self::Attendant,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Mechanic => "Mechanic",
            Self::Handyman => "Handyman",
            Self::Security => "Security",
            Self::Entertainer => "Entertainer",
            Self::Attendant => "Service attendant",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum StaffTask {
    #[default]
    Patrol,
    Repair,
    Clean,
    EmptyBin,
    ResolveIncident,
    Entertain,
    Assist,
}
impl StaffTask {
    pub fn label(self) -> &'static str {
        match self {
            Self::Patrol => "Patrolling",
            Self::Repair => "Inspecting ride",
            Self::Clean => "Cleaning",
            Self::EmptyBin => "Emptying bin",
            Self::ResolveIncident => "Resolving incident",
            Self::Entertain => "Entertaining guests",
            Self::Assist => "Helping guests",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaffMember {
    pub id: u64,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub role: StaffRole,
    pub working: bool,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub current: Option<ObjectId>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub next: Option<ObjectId>,
    pub progress: u16,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub destination: Option<ObjectId>,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub task: StaffTask,
    pub task_ticks: u16,
    pub wage_ticks: u16,
    pub wages_paid_cents: u64,
    pub completed: u64,
    #[serde(deserialize_with = "crate::saves::bounded_vec::<_,_,64>")]
    pub patrol: Vec<ObjectId>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IncidentKind {
    Vandalism,
    Distress,
}
impl IncidentKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Vandalism => "Vandalism",
            Self::Distress => "Guest needs assistance",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParkIncident {
    pub id: u64,
    pub path: ObjectId,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub kind: IncidentKind,
    pub tick: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PartyStyle {
    Family,
    Thrill,
    Relaxed,
}
impl PartyStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Family => "Family group",
            Self::Thrill => "Thrill seekers",
            Self::Relaxed => "Relaxed visitors",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Party {
    pub id: u64,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub style: PartyStyle,
    #[serde(deserialize_with = "crate::saves::bounded_vec::<_,_,4>")]
    pub members: Vec<GuestId>,
    pub returning: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReturnVisit {
    pub due: u64,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub style: PartyStyle,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ParkLife {
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub policy: WeatherPolicy,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub weather: Weather,
    pub weather_ticks: u16,
    pub next_staff_id: u64,
    pub next_party_id: u64,
    pub next_incident_id: u64,
    pub wages_paid_cents: u64,
    pub resolved: u64,
    pub returning_arrivals: u64,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,PARK_STAFF_LIMIT>")]
    pub staff: Vec<StaffMember>,
    #[serde(deserialize_with = "crate::saves::bounded_vec::<_,_,64>")]
    pub shelters: Vec<ObjectId>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,INCIDENT_LIMIT>")]
    pub incidents: Vec<ParkIncident>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,PARTY_LIMIT>")]
    pub parties: Vec<Party>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,32>")]
    pub returns: Vec<ReturnVisit>,
    #[serde(deserialize_with = "crate::saves::bounded_vec::<_,_,{crate::MAX_RIDE_RECORDS}>")]
    pub weather_closed: Vec<RideId>,
}
impl Default for ParkLife {
    fn default() -> Self {
        Self {
            policy: WeatherPolicy::Automatic,
            weather: Weather::Sunny,
            weather_ticks: 0,
            next_staff_id: 1,
            next_party_id: 1,
            next_incident_id: 1,
            wages_paid_cents: 0,
            resolved: 0,
            returning_arrivals: 0,
            staff: Vec::new(),
            shelters: Vec::new(),
            incidents: Vec::new(),
            parties: Vec::new(),
            returns: Vec::new(),
            weather_closed: Vec::new(),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParkLifeStatus {
    pub enabled: bool,
    pub weather: Weather,
    pub policy: WeatherPolicy,
    pub seconds_until_change: u16,
    pub staff: usize,
    pub unpaid_staff: usize,
    pub incidents: usize,
    pub resolved: u64,
    pub shelters: usize,
    pub parties: usize,
    pub returning_arrivals: u64,
}

impl Park {
    pub fn enable_park_life(&mut self) {
        self.park_life.get_or_insert_with(ParkLife::default);
        self.grounds.active = true;
    }
    pub fn park_life_status(&self) -> ParkLifeStatus {
        let empty = ParkLife::default();
        let life = self.park_life.as_ref().unwrap_or(&empty);
        ParkLifeStatus {
            enabled: self.park_life.is_some(),
            weather: life.weather,
            policy: life.policy,
            seconds_until_change: (1800 - life.weather_ticks) / 30,
            staff: life.staff.len(),
            unpaid_staff: life.staff.iter().filter(|s| !s.working).count(),
            incidents: life.incidents.len(),
            resolved: life.resolved,
            shelters: life.shelters.len(),
            parties: life.parties.len(),
            returning_arrivals: life.returning_arrivals,
        }
    }
    pub fn park_staff(&self) -> &[StaffMember] {
        self.park_life.as_ref().map_or(&[], |l| &l.staff)
    }
    pub fn park_incidents(&self) -> &[ParkIncident] {
        self.park_life.as_ref().map_or(&[], |l| &l.incidents)
    }
    pub fn path_shelters(&self) -> &[ObjectId] {
        self.park_life.as_ref().map_or(&[], |l| &l.shelters)
    }
    pub fn toggle_path_shelter(&mut self, path: ObjectId) -> Result<bool, String> {
        if self.object(path).is_none_or(|p| !p.is_public_path()) {
            return Err("Select a public path for a shelter".into());
        }
        let exists = self.path_shelters().contains(&path);
        if exists {
            self.park_life
                .as_mut()
                .unwrap()
                .shelters
                .retain(|p| *p != path);
            return Ok(false);
        }
        if self.path_shelters().len() >= 64 {
            return Err("Shelter limit reached".into());
        }
        if !self.path_benches().iter().any(|b| b.path == path) {
            self.toggle_path_bench(path).map_err(|e| e.to_string())?;
        }
        self.enable_park_life();
        self.park_life.as_mut().unwrap().shelters.push(path);
        self.park_life.as_mut().unwrap().shelters.sort_unstable();
        Ok(true)
    }
    pub fn path_is_sheltered(&self, path: ObjectId) -> bool {
        self.path_shelters().contains(&path)
            && !self
                .park_incidents()
                .iter()
                .any(|i| i.path == path && i.kind == IncidentKind::Vandalism)
    }
    pub fn set_weather_policy(&mut self, policy: WeatherPolicy) {
        self.enable_park_life();
        self.park_life.as_mut().unwrap().policy = policy;
        self.park_life.as_mut().unwrap().weather_ticks = 0;
        let weather = match policy {
            WeatherPolicy::Automatic | WeatherPolicy::Sunny => Weather::Sunny,
            WeatherPolicy::Hot => Weather::Hot,
            WeatherPolicy::Rain => Weather::Rain,
            WeatherPolicy::Storm => Weather::Storm,
        };
        self.change_life_weather(weather);
    }
    pub fn staff_position(&self, id: u64) -> Option<PathPoint> {
        staff::position(self, self.park_staff().iter().find(|s| s.id == id)?)
    }
    pub(crate) fn step_park_life(&mut self) {
        if self.park_life.is_none() {
            return;
        }
        self.reconcile_park_life();
        self.step_life_weather();
        self.step_life_guests();
        self.step_park_staff();
    }
    pub(crate) fn reconcile_park_life(&mut self) {
        let Some(mut life) = self.park_life.take() else {
            return;
        };
        let valid = |id: ObjectId| self.object(id).is_some_and(|p| p.is_public_path());
        life.shelters
            .retain(|id| valid(*id) && self.path_benches().iter().any(|b| b.path == *id));
        life.incidents.retain(|i| valid(i.path));
        life.weather_closed
            .retain(|id| self.ride_by_id(*id).is_some());
        for worker in &mut life.staff {
            worker.patrol.retain(|id| valid(*id));
            if worker.current.is_some_and(|id| !valid(id)) {
                worker.current = None;
                worker.next = None;
                worker.progress = 0;
            }
            if worker.next.is_some_and(|id| {
                !valid(id)
                    || worker
                        .current
                        .is_none_or(|c| !self.path_neighbors(c).contains(&id))
            }) {
                worker.next = None;
                worker.progress = 0;
            }
            if worker.destination.is_some_and(|id| !valid(id)) {
                worker.destination = None;
                worker.task_ticks = 0;
                worker.task = StaffTask::Patrol;
            }
        }
        self.park_life = Some(life);
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    let Some(life) = &park.park_life else {
        return Ok(());
    };
    let recorded_wages = park.finances.archived.staff_wages_cents
        + park
            .finances
            .entries
            .iter()
            .filter(|e| e.kind == crate::FinanceKind::StaffWage)
            .map(|e| e.amount_cents)
            .sum::<u64>();
    if life.wages_paid_cents != recorded_wages
        || life.staff.iter().map(|s| s.wages_paid_cents).sum::<u64>() > recorded_wages
    {
        return Err("Park staff receipts disagree with wages".into());
    }
    let valid = |id: ObjectId| park.object(id).is_some_and(|p| p.is_public_path());
    if life.weather_ticks >= 1800
        || life.staff.len() > PARK_STAFF_LIMIT
        || life.shelters.len() > 64
        || life.incidents.len() > INCIDENT_LIMIT
        || life.parties.len() > PARTY_LIMIT
        || life.returns.len() > 32
    {
        return Err("Park life collection or clock exceeds its limit".into());
    }
    let mut staff_ids = std::collections::BTreeSet::new();
    for s in &life.staff {
        if s.id == 0
            || s.id >= life.next_staff_id
            || !staff_ids.insert(s.id)
            || s.progress >= crate::WALK_TICKS
            || (s.next.is_none() && s.progress != 0)
            || s.wage_ticks >= PARK_STAFF_WAGE_TICKS
            || (!s.working && s.wage_ticks != PARK_STAFF_WAGE_TICKS - 1)
            || s.task_ticks >= 120
            || s.patrol.len() > 64
            || s.patrol.windows(2).any(|p| p[0] >= p[1])
            || s.wages_paid_cents % PARK_STAFF_WAGE_CENTS != 0
            || s.wages_paid_cents / PARK_STAFF_WAGE_CENTS > park.ticks / 900
            || s.current.is_none() && s.next.is_some()
            || s.current.is_some_and(|id| !valid(id))
            || s.next.is_some_and(|id| {
                !valid(id)
                    || s.current
                        .is_none_or(|c| !park.path_neighbors(c).contains(&id))
            })
            || s.destination.is_some_and(|id| !valid(id))
            || s.patrol.iter().any(|id| !valid(*id))
        {
            return Err("Park staff route, identity or wage is invalid".into());
        }
    }
    let mut incident_ids = std::collections::BTreeSet::new();
    for i in &life.incidents {
        if i.id == 0
            || i.id >= life.next_incident_id
            || !incident_ids.insert(i.id)
            || !valid(i.path)
            || i.tick > park.ticks
        {
            return Err("Park incident is invalid".into());
        }
    }
    if life.shelters.windows(2).any(|p| p[0] >= p[1])
        || life
            .shelters
            .iter()
            .any(|id| !valid(*id) || !park.path_benches().iter().any(|b| b.path == *id))
    {
        return Err("Shelter reference is invalid".into());
    }
    let mut member_ids = std::collections::BTreeSet::new();
    for p in &life.parties {
        if p.id == 0
            || p.id >= life.next_party_id
            || p.members.is_empty()
            || p.members.len() > 4
            || p.members
                .iter()
                .any(|id| !member_ids.insert(*id) || !park.guests.iter().any(|g| g.id == *id))
        {
            return Err("Visitor group membership is invalid".into());
        }
    }
    if life
        .weather_closed
        .iter()
        .any(|id| park.ride_by_id(*id).is_none())
    {
        return Err("Weather closure references an unknown ride".into());
    }
    Ok(())
}
