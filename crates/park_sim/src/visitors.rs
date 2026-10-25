//! Opt-in visitor flow; retired visitors retain bounded aggregate receipts/reviews.
use crate::{Guest, GuestId, GuestRideHistory, GuestState, ObjectId, Park, SurfaceObject};
use std::collections::VecDeque;

pub const VISITOR_ARRIVAL_TICKS: u16 = 900;
pub const VISITOR_STAY_TICKS: u64 = 18_000;
pub const VISITOR_ACTIVE_CAP: usize = 64;
pub const VISITOR_LIFETIME_CAP: u64 = 1_000_000;

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepartedVisitors {
    pub count: u64,
    pub highest_id: u64,
    pub wallet_cents: u64,
    pub spent_cents: u64,
    pub service_spent_cents: u64,
    pub services_used: u64,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,{crate::MAX_RIDE_RECORDS}>")]
    pub rides: Vec<GuestRideHistory>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VisitorStay {
    pub guest: GuestId,
    pub entered_tick: u64,
    pub leaving: bool,
}
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VisitorFlow {
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub entry: Option<ObjectId>,
    pub enabled: bool,
    pub arrival_progress: u16,
    pub spawned: u64,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub departed: DepartedVisitors,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,256>")]
    pub stays: Vec<VisitorStay>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WalkGoal {
    Explore,
    Exit(Option<ObjectId>),
}
impl VisitorFlow {
    pub(crate) fn leaving(&self, id: GuestId) -> bool {
        self.stays
            .binary_search_by_key(&id, |r| r.guest)
            .ok()
            .is_some_and(|i| self.stays[i].leaving)
    }
    pub(crate) fn walk_goal(&self, id: GuestId) -> WalkGoal {
        if self.leaving(id) {
            WalkGoal::Exit(self.entry)
        } else {
            WalkGoal::Explore
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisitorFlowStats {
    pub entry: Option<ObjectId>,
    pub enabled: bool,
    pub active: usize,
    pub tracked: usize,
    pub leaving: usize,
    pub finishing_visits: usize,
    pub spawned: u64,
    pub departed: u64,
    pub departed_spent_cents: u64,
    pub departed_wallet_cents: u64,
    pub arrival_progress: u16,
    pub entry_ready: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisitorError {
    PublicPath,
    EntryDisconnected,
    UnknownGuest,
    Capacity,
}
impl std::fmt::Display for VisitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::PublicPath => "Select an ordinary public path first",
            Self::EntryDisconnected => "Entry needs a connected public path neighbor",
            Self::UnknownGuest => "Select an active guest first",
            Self::Capacity => "Visitor tracking capacity is exhausted",
        })
    }
}
impl std::error::Error for VisitorError {}

impl Park {
    pub fn departed_visitors(&self) -> &DepartedVisitors {
        &self.visitors.departed
    }
    pub fn guest_is_leaving(&self, id: GuestId) -> bool {
        self.visitors.leaving(id)
    }
    pub fn visitor_flow_stats(&self) -> VisitorFlowStats {
        let leaving = self.guests.iter().filter(|g| self.visitors.leaving(g.id));
        VisitorFlowStats {
            entry: self.visitors.entry,
            enabled: self.visitors.enabled,
            active: self.guests.len(),
            tracked: self.visitors.stays.len(),
            leaving: leaving.clone().count(),
            finishing_visits: leaving
                .filter(|g| g.ride_visit.is_some() || g.service_visit.is_some())
                .count(),
            spawned: self.visitors.spawned,
            departed: self.visitors.departed.count,
            departed_spent_cents: self.visitors.departed.spent_cents,
            departed_wallet_cents: self.visitors.departed.wallet_cents,
            arrival_progress: self.visitors.arrival_progress,
            entry_ready: self.visitor_entry_ready(),
        }
    }
    fn visitor_entry_ready(&self) -> bool {
        self.visitors.entry.is_some_and(|id| {
            self.object(id).is_some_and(|p| p.is_public_path())
                && !self.path_neighbors(id).is_empty()
        })
    }
    pub fn set_visitor_entry(&mut self, id: ObjectId) -> Result<(), VisitorError> {
        if self.object(id).is_none_or(|p| !p.is_public_path()) {
            return Err(VisitorError::PublicPath);
        }
        self.visitors.entry = Some(id);
        Ok(())
    }
    pub fn set_visitor_arrivals(&mut self, enabled: bool) -> Result<(), VisitorError> {
        if enabled && !self.visitor_entry_ready() {
            return Err(VisitorError::EntryDisconnected);
        }
        if enabled {
            for g in &self.guests {
                if let Err(i) = self.visitors.stays.binary_search_by_key(&g.id, |r| r.guest) {
                    self.visitors.stays.insert(
                        i,
                        VisitorStay {
                            guest: g.id,
                            entered_tick: self.ticks,
                            leaving: false,
                        },
                    );
                }
            }
            if self.crowd_seed.is_none() {
                self.crowd_seed = Some(0);
            }
        }
        self.visitors.enabled = enabled;
        Ok(())
    }
    pub fn request_guest_departure(&mut self, id: GuestId) -> Result<(), VisitorError> {
        if !self.guests.iter().any(|g| g.id == id) {
            return Err(VisitorError::UnknownGuest);
        }
        match self.visitors.stays.binary_search_by_key(&id, |r| r.guest) {
            Ok(i) => self.visitors.stays[i].leaving = true,
            Err(i) => self.visitors.stays.insert(
                i,
                VisitorStay {
                    guest: id,
                    entered_tick: self.ticks,
                    leaving: true,
                },
            ),
        }
        Ok(())
    }
    pub(crate) fn step_visitor_arrivals(&mut self) {
        for stay in &mut self.visitors.stays {
            if self.ticks - stay.entered_tick >= VISITOR_STAY_TICKS {
                stay.leaving = true;
            }
        }
        if !self.visitors.enabled || !self.visitor_entry_ready() {
            return;
        }
        if self.visitors.arrival_progress < VISITOR_ARRIVAL_TICKS - 1 {
            self.visitors.arrival_progress += 1;
            return;
        }
        if self.guests.len() >= VISITOR_ACTIVE_CAP
            || self.visitors.departed.count + self.guests.len() as u64 >= VISITOR_LIFETIME_CAP
            || self.next_guest_id == u64::MAX
        {
            return;
        }
        let path = *self.object(self.visitors.entry.unwrap()).unwrap();
        let id = GuestId(self.next_guest_id);
        self.next_guest_id += 1;
        let seed = self.crowd_seed.unwrap_or(0);
        self.guests.push(new_guest(id, path, seed));
        let i = self
            .visitors
            .stays
            .binary_search_by_key(&id, |r| r.guest)
            .unwrap_err();
        self.visitors.stays.insert(
            i,
            VisitorStay {
                guest: id,
                entered_tick: self.ticks,
                leaving: false,
            },
        );
        self.visitors.spawned += 1;
        self.visitors.arrival_progress = 0;
    }
    pub(crate) fn finish_visitor_departures(&mut self) {
        let Some(entry) = self.visitors.entry else {
            return;
        };
        let ids = self
            .guests
            .iter()
            .filter(|g| {
                self.visitors.leaving(g.id)
                    && g.current == Some(entry)
                    && g.next.is_none()
                    && g.progress == 0
                    && g.ride_visit.is_none()
                    && g.service_visit.is_none()
            })
            .map(|g| g.id)
            .collect::<Vec<_>>();
        for id in ids {
            let i = self.guests.iter().position(|g| g.id == id).unwrap();
            let g = self.guests.remove(i);
            let a = &mut self.visitors.departed;
            a.count += 1;
            a.highest_id = a.highest_id.max(g.id.0);
            a.wallet_cents += u64::from(g.wallet_cents);
            a.spent_cents += g.spent_cents;
            a.service_spent_cents += g.service_spent_cents;
            a.services_used += g.services_used;
            for row in g.ride_history {
                match a.rides.binary_search_by_key(&row.ride, |r| r.ride) {
                    Ok(i) => {
                        let r = &mut a.rides[i];
                        r.spent_cents += row.spent_cents;
                        r.completed += row.completed;
                        r.rating_count += row.rating_count;
                        r.rating_total += row.rating_total;
                    }
                    Err(i) => a.rides.insert(i, row),
                }
            }
            let i = self
                .visitors
                .stays
                .binary_search_by_key(&id, |r| r.guest)
                .unwrap();
            self.visitors.stays.remove(i);
        }
    }
}
fn new_guest(id: GuestId, path: SurfaceObject, seed: u64) -> Guest {
    Guest {
        id,
        current: Some(path.id),
        next: None,
        destination: None,
        anchor: path.pos,
        height: path.height,
        anchor_slope: path.path_slope,
        progress: 0,
        state: GuestState::Waiting,
        arrivals: 0,
        rides_completed: 0,
        ride_history: Vec::new(),
        wallet_cents: crate::INITIAL_GUEST_WALLET_CENTS,
        spent_cents: 0,
        service_spent_cents: 0,
        services_used: 0,
        needs: crate::GuestNeeds::for_guest(seed, id),
        service_visit: None,
        service_cooldown: 0,
        preferences: crate::GuestPreferences::for_guest(seed, id),
        satisfaction: 70,
        thought: crate::GuestThought::Exploring,
        thought_tick: 0,
        ratings_given: 0,
        rating_total: 0,
        last_rating: None,
        ride_visit: None,
        ride_cooldown: 0,
        route: VecDeque::new(),
        wait_ticks: 0,
    }
}

pub(crate) fn validate(p: &Park) -> Result<(), String> {
    let f = &p.visitors;
    let a = &f.departed;
    if (p.crowd_seed.is_none()
        && (f.enabled || f.spawned != 0 || a.count != 0 || !f.stays.is_empty()))
        || (f.enabled && f.stays.len() != p.guests.len())
        || f.arrival_progress >= VISITOR_ARRIVAL_TICKS
        || f.spawned > VISITOR_LIFETIME_CAP
        || a.count > VISITOR_LIFETIME_CAP
        || a.count + p.guests.len() as u64 > VISITOR_LIFETIME_CAP
        || f.spawned > a.count + p.guests.len() as u64
        || (a.count == 0) != (a.highest_id == 0)
        || a.count > a.highest_id
        || a.highest_id >= p.next_guest_id
        || a.wallet_cents.checked_add(a.spent_cents)
            != a.count
                .checked_mul(u64::from(crate::INITIAL_GUEST_WALLET_CENTS))
        || a.service_spent_cents > a.spent_cents
        || f.entry
            .is_some_and(|id| p.object(id).is_none_or(|o| !o.is_public_path()))
    {
        return Err("visitor counters, entry or retired wallets are invalid".into());
    }
    let mut previous = None;
    for r in &f.stays {
        if previous.is_some_and(|id| id >= r.guest)
            || !p.guests.iter().any(|g| g.id == r.guest)
            || r.entered_tick > p.ticks
            || (!r.leaving && p.ticks - r.entered_tick >= VISITOR_STAY_TICKS)
        {
            return Err("visitor stay reference, order or age is invalid".into());
        }
        previous = Some(r.guest);
    }
    let mut previous = None;
    let mut spent = 0u64;
    let mut ratings = 0u128;
    for r in &a.rides {
        if previous.is_some_and(|id| id >= r.ride)
            || p.ride_by_id(r.ride).is_none()
            || r.rating_count > r.completed
            || r.rating_count > crate::experience::MAX_RATINGS
            || r.rating_total < r.rating_count
            || u128::from(r.rating_total) > u128::from(r.rating_count) * 5
            || (r.completed == 0 && r.spent_cents != 0)
        {
            return Err("retired ride history is invalid".into());
        }
        ratings += u128::from(r.rating_count);
        previous = Some(r.ride);
        spent = spent
            .checked_add(r.spent_cents)
            .ok_or("retired spending overflows")?;
    }
    if ratings > u128::from(crate::experience::MAX_RATINGS)
        || spent != a.spent_cents - a.service_spent_cents
        || (a.count == 0 && (!a.rides.is_empty() || a.services_used != 0))
    {
        return Err("retired receipts disagree with wallet history".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
