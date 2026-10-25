//! Deterministic single-file admission. A queue is a waiting place, not boarding.
use std::collections::{BTreeMap, VecDeque};

use crate::{
    Guest, GuestId, GuestState, GuestThought, ObjectId, ObjectKind, Park, RideAccess,
    RideAccessStatus, RideId, SurfaceObject, WALK_TICKS, paths_connect,
};

pub const QUEUE_PATIENCE_TICKS: u32 = 1800;
pub const RIDE_COOLDOWN_TICKS: u16 = 300;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RideVisitPhase {
    Approaching,
    Queueing,
    Leaving,
    Boarding,
    Riding,
    Exiting,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideVisit {
    pub ride: RideId,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub phase: RideVisitPhase,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub ticket: Option<u64>,
    pub waited_ticks: u32,
    #[serde(deserialize_with = "crate::saves::required_object_option")]
    pub passenger: Option<crate::PassengerJourney>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RideQueueStats {
    pub seeking: usize,
    pub queueing: usize,
    pub leaving: usize,
    pub stranded: usize,
    pub capacity: usize,
    pub moving: usize,
    pub waiting: usize,
    pub oldest_wait_ticks: u32,
    pub draining: bool,
    pub admitting: bool,
    pub joined: u64,
    pub turned_away: u64,
    pub abandoned: u64,
}

impl Park {
    pub fn ride_queue_stats(&self, id: RideId) -> RideQueueStats {
        let Some(ride) = self.ride_by_id(id) else {
            return RideQueueStats::default();
        };
        let access = self.ride_access(id);
        let mut stats = RideQueueStats {
            capacity: access.queue_tiles,
            draining: ride.draining,
            admitting: self.admission_eligible(id, &access)
                && !ride.draining
                && ride.next_queue_ticket < u64::MAX,
            joined: ride.joined,
            turned_away: ride.turned_away,
            abandoned: ride.abandoned,
            ..Default::default()
        };
        for guest in &self.guests {
            let Some(visit) = guest.ride_visit.as_ref().filter(|visit| visit.ride == id) else {
                continue;
            };
            match visit.phase {
                RideVisitPhase::Approaching => stats.seeking += 1,
                RideVisitPhase::Queueing => {
                    stats.queueing += 1;
                    stats.moving += usize::from(guest.next.is_some());
                    stats.waiting += usize::from(guest.state == GuestState::Waiting);
                }
                RideVisitPhase::Leaving => {
                    stats.leaving += 1;
                    stats.stranded += usize::from(guest.state == GuestState::Lost);
                }
                RideVisitPhase::Boarding | RideVisitPhase::Riding | RideVisitPhase::Exiting => {}
            }
            if !crate::journeys::passenger_phase(visit.phase) {
                stats.oldest_wait_ticks = stats.oldest_wait_ticks.max(visit.waited_ticks);
            }
        }
        stats
    }

    pub fn ride_queue_order(&self, id: RideId) -> Vec<GuestId> {
        let mut members: Vec<_> = self
            .guests
            .iter()
            .filter_map(|guest| {
                let visit = guest.ride_visit.as_ref().filter(|visit| visit.ride == id)?;
                if !matches!(
                    visit.phase,
                    RideVisitPhase::Queueing | RideVisitPhase::Leaving
                ) {
                    return None;
                }
                Some((visit.ticket?, guest.id))
            })
            .collect();
        members.sort_unstable();
        members.into_iter().map(|(_, id)| id).collect()
    }

    pub(crate) fn admission_eligible(&self, id: RideId, access: &RideAccess) -> bool {
        access.status == RideAccessStatus::Ready && self.ride_is_open(id)
    }

    /// Commands reconcile intent immediately, including while paused. A surviving
    /// in-flight edge keeps its exact progress and finishes before walking back.
    pub(crate) fn reconcile_ride_visits(&mut self) {
        if self.guests.iter().all(|guest| guest.ride_visit.is_none()) {
            return;
        }
        let mut ids: Vec<_> = self.all_rides().map(|r| r.id).collect();
        ids.sort_unstable();
        for id in ids {
            let access = self.ride_access(id);
            self.reconcile_admission(id, &access, false);
        }
    }

    fn reconcile_admission(&mut self, id: RideId, access: &RideAccess, patience: bool) {
        let eligible = self.admission_eligible(id, access);
        let Some(mut ride) = self.ride_by_id(id).cloned() else {
            return;
        };
        let cancel = !eligible || ride.draining || patience;
        for guest in &mut self.guests {
            let Some(visit) = guest.ride_visit.as_mut().filter(|v| v.ride == id) else {
                continue;
            };
            if visit.phase == RideVisitPhase::Approaching
                && (cancel || guest.current.is_none() || guest.state == GuestState::Lost)
            {
                guest.ride_visit = None;
                guest.ride_cooldown = RIDE_COOLDOWN_TICKS;
                guest.think(
                    if guest.current.is_none() || guest.state == GuestState::Lost {
                        GuestThought::CannotReachRide
                    } else {
                        GuestThought::RideCancelled
                    },
                    self.ticks,
                );
                ride.turned_away = ride.turned_away.saturating_add(1);
            } else if visit.phase == RideVisitPhase::Queueing && cancel {
                visit.phase = RideVisitPhase::Leaving;
                ride.abandoned = ride.abandoned.saturating_add(1);
                guest.route.clear();
                guest.destination = guest.next;
                guest.wait_ticks = 0;
                guest.cancel_experience(self.ticks);
            }
        }
        ride.draining = self.guests.iter().any(|guest| {
            guest.ride_visit.as_ref().is_some_and(|visit| {
                visit.ride == id && visit.ticket.is_some() && visit.phase == RideVisitPhase::Leaving
            })
        });
        *self.ride_by_id_mut(id).unwrap() = ride;
    }

    pub(crate) fn drain_queue_for_funds(&mut self, access: &RideAccess, rejected: GuestId) {
        let id = self
            .guests
            .iter()
            .find(|g| g.id == rejected)
            .and_then(|g| g.ride_visit.as_ref())
            .expect("queue guest")
            .ride;
        let ride = self.ride_by_id_mut(id).expect("queue belongs to a ride");
        ride.unaffordable_rejections = ride.unaffordable_rejections.saturating_add(1);
        // A front guest cannot pass the line behind them. Reuse the existing
        // tail-first physical drain, without charging anyone or moving poses.
        self.reconcile_admission(id, access, true);
        if let Some(guest) = self.guests.iter_mut().find(|guest| guest.id == rejected) {
            guest.think(GuestThought::NotEnoughMoney, self.ticks);
        }
    }

    pub(crate) fn drain_queue_for_value(&mut self, access: &RideAccess, rejected: GuestId) {
        let id = self
            .guests
            .iter()
            .find(|g| g.id == rejected)
            .and_then(|g| g.ride_visit.as_ref())
            .expect("queue guest")
            .ride;
        let ride = self.ride_by_id_mut(id).expect("queue belongs to a ride");
        ride.value_rejections = ride.value_rejections.saturating_add(1);
        self.reconcile_admission(id, access, true);
        if let Some(guest) = self.guests.iter_mut().find(|guest| guest.id == rejected) {
            guest.think(GuestThought::TooExpensive, self.ticks);
        }
    }

    pub(crate) fn step_admission_crowd(&mut self, graph: &crate::crowds::PathGraph) {
        let mut ids: Vec<_> = self.all_rides().map(|r| r.id).collect();
        ids.sort_unstable();
        let mut patience = std::collections::BTreeSet::new();
        if !ids.is_empty() {
            for guest in &mut self.guests {
                guest.ride_cooldown = guest.ride_cooldown.saturating_sub(1);
                if let Some(visit) = &mut guest.ride_visit
                    && visit.phase == RideVisitPhase::Queueing
                {
                    visit.waited_ticks = visit
                        .waited_ticks
                        .saturating_add(1)
                        .min(QUEUE_PATIENCE_TICKS);
                    if visit.waited_ticks == QUEUE_PATIENCE_TICKS {
                        patience.insert(visit.ride);
                    }
                    if visit.waited_ticks == guest.preferences.comfortable_queue_ticks {
                        guest.satisfaction = guest.satisfaction.saturating_sub(5);
                        guest.think(GuestThought::WaitingTooLong, self.ticks);
                    }
                }
            }
        }
        let accesses: BTreeMap<_, _> = ids.iter().map(|&id| (id, self.ride_access(id))).collect();
        for &id in &ids {
            self.reconcile_admission(id, &accesses[&id], patience.contains(&id));
        }
        let choices = self.choose_guest_rides(graph, &accesses);
        // Keep legacy public-walk RNG ordering when there is only one ride.
        let ordinary_inline = ids.len() == 1;
        let processed: std::collections::BTreeSet<_> = self
            .guests
            .iter()
            .filter(|g| ordinary_inline || g.ride_visit.is_some() || choices.contains_key(&g.id))
            .map(|g| g.id)
            .collect();
        for id in ids {
            self.step_admission_for_ride(id, graph, &accesses[&id], &choices, ordinary_inline);
        }
        for guest in &mut self.guests {
            if guest.service_visit.is_none() && !processed.contains(&guest.id) {
                graph.step_guest(
                    guest,
                    &mut self.crowd_rng,
                    self.visitors.walk_goal(guest.id),
                );
            }
        }
    }

    fn step_admission_for_ride(
        &mut self,
        id: RideId,
        graph: &crate::crowds::PathGraph,
        access: &RideAccess,
        choices: &BTreeMap<GuestId, RideId>,
        ordinary_inline: bool,
    ) {
        let mut ride = self.ride_by_id(id).unwrap().clone();
        let eligible = self.admission_eligible(id, access)
            && !ride.draining
            && ride.next_queue_ticket < u64::MAX;
        if !eligible
            && self
                .guests
                .iter()
                .all(|g| g.ride_visit.as_ref().is_none_or(|v| v.ride != id))
        {
            if ordinary_inline {
                for guest in &mut self.guests {
                    if guest.service_visit.is_none() {
                        graph.step_guest(
                            guest,
                            &mut self.crowd_rng,
                            self.visitors.walk_goal(guest.id),
                        );
                    }
                }
            }
            return;
        }
        let objects: BTreeMap<_, _> = self
            .objects
            .iter()
            .filter(|object| object.kind == ObjectKind::Path)
            .map(|object| (object.id, *object))
            .collect();
        let queue_index: BTreeMap<_, _> = access
            .ordered_queue
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index))
            .collect();
        let escape = if ride.draining {
            EscapeGraph::new(&objects, id)
        } else {
            EscapeGraph {
                next: BTreeMap::new(),
            }
        };
        let mut reserved = reservations(&self.guests, &objects);
        let mut members: Vec<_> = self
            .guests
            .iter()
            .enumerate()
            .filter_map(|(index, guest)| {
                let visit = guest.ride_visit.as_ref().filter(|v| v.ride == id)?;
                if !matches!(
                    visit.phase,
                    RideVisitPhase::Queueing | RideVisitPhase::Leaving
                ) {
                    return None;
                }
                Some((visit.ticket?, index))
            })
            .collect();
        members.sort_unstable();
        if ride.draining {
            // A repaired footprint belongs first to its anchored former
            // occupant. Reserve these before another returning guest can walk
            // into the replacement, even if edits reversed the escape route.
            // Ticket order breaks any coincident-anchor tie deterministically.
            let footprints: BTreeMap<_, _> = objects
                .values()
                .filter(|path| path.is_public_path() || path.queue_owner == Some(id))
                .map(|path| ((path.pos.x, path.pos.y, path.height), path))
                .collect();
            for &(_, index) in &members {
                let guest = &mut self.guests[index];
                if guest.current.is_none()
                    && let Some(path) =
                        footprints.get(&(guest.anchor.x, guest.anchor.y, guest.height))
                    && path.path_slope == guest.anchor_slope
                    && !reserved.contains_key(&path.id)
                {
                    anchor_on(guest, path);
                    if path.queue_owner.is_some() {
                        reserved.insert(path.id, guest.id);
                    }
                }
            }
            members.reverse();
        }
        for (_, index) in members {
            let guest = &mut self.guests[index];
            let leaving = guest.ride_visit.as_ref().unwrap().phase == RideVisitPhase::Leaving;
            if guest.next.is_some() {
                finish_edge(guest, &objects, &mut reserved);
                if leaving
                    && guest.next.is_none()
                    && guest
                        .current
                        .is_some_and(|id| objects[&id].is_public_path())
                {
                    end_visit(guest);
                }
                continue;
            }
            if leaving {
                if guest.current.is_none() {
                    let anchor = guest.anchor_path();
                    let recovered = objects
                        .values()
                        .filter(|path| {
                            guest.can_walk_path(path) && !reserved.contains_key(&path.id)
                        })
                        .filter_map(|path| {
                            let distance = path.pos.x.abs_diff(anchor.pos.x)
                                + path.pos.y.abs_diff(anchor.pos.y);
                            ((distance == 0
                                && path.height == anchor.height
                                && path.path_slope == anchor.path_slope)
                                || paths_connect(&anchor, path))
                            .then_some((distance, path.id, *path))
                        })
                        .min_by_key(|(distance, id, _)| (*distance, *id));
                    if let Some((_, _, path)) = recovered {
                        anchor_on(guest, &path);
                        if path.queue_owner.is_some() {
                            reserved.insert(path.id, guest.id);
                        }
                    }
                }
                let Some(current) = guest.current else {
                    guest.state = GuestState::Lost;
                    continue;
                };
                if objects[&current].is_public_path() {
                    end_visit(guest);
                    continue;
                }
                if let Some(next) = escape.next_out(current) {
                    if !reserved.contains_key(&next) {
                        start_edge(guest, next, &objects, &mut reserved);
                    } else {
                        guest.state = GuestState::Waiting;
                    }
                } else {
                    guest.state = GuestState::Lost;
                }
            } else if let Some(&index) = guest.current.and_then(|current| queue_index.get(&current))
            {
                guest.state = GuestState::Waiting;
                if index > 0 {
                    let next = access.ordered_queue[index - 1];
                    if !reserved.contains_key(&next) {
                        start_edge(guest, next, &objects, &mut reserved);
                    }
                }
            }
        }
        let mut applicants = Vec::new();
        for (index, guest) in self.guests.iter_mut().enumerate() {
            if guest.service_visit.is_some()
                || guest.ride_visit.as_ref().is_some_and(|v| v.ride != id)
            {
                continue;
            }
            if guest.ride_visit.is_none() && choices.get(&guest.id) != Some(&id) {
                if ordinary_inline {
                    graph.step_guest(
                        guest,
                        &mut self.crowd_rng,
                        self.visitors.walk_goal(guest.id),
                    );
                }
                continue;
            }
            if guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.ticket.is_some())
            {
                continue;
            }
            let approaching = guest.ride_visit.is_some();
            if approaching {
                if guest.next.is_some() {
                    graph.step_guest(
                        guest,
                        &mut self.crowd_rng,
                        self.visitors.walk_goal(guest.id),
                    );
                    if guest.next.is_none() && guest.current == access.public_entry {
                        guest.wait_ticks = 0;
                    }
                } else if guest.current == access.public_entry {
                    applicants.push(index);
                } else if let (Some(current), Some(entry)) = (guest.current, access.public_entry) {
                    if let Some(mut route) = graph.route(current, entry) {
                        guest.next = route.pop_front();
                        guest.route = route;
                        guest.destination = Some(entry);
                        guest.state = GuestState::Walking;
                        guest.wait_ticks = 0;
                    } else {
                        reject_applicant(
                            guest,
                            &mut ride,
                            GuestThought::CannotReachRide,
                            self.ticks,
                        );
                    }
                } else {
                    reject_applicant(guest, &mut ride, GuestThought::CannotReachRide, self.ticks);
                }
                continue;
            }
            if eligible
                && guest.ride_cooldown == 0
                && guest.next.is_none()
                && guest.wait_ticks == 0
                && let (Some(current), Some(entry)) = (guest.current, access.public_entry)
                && let Some(mut route) = graph.route(current, entry)
            {
                let ride = &mut ride;
                if guest.wallet_cents < ride.ticket_price_cents {
                    ride.unaffordable_rejections = ride.unaffordable_rejections.saturating_add(1);
                    reject_applicant(guest, ride, GuestThought::NotEnoughMoney, self.ticks);
                    graph.step_guest(
                        guest,
                        &mut self.crowd_rng,
                        self.visitors.walk_goal(guest.id),
                    );
                    continue;
                }
                if !guest.willing_to_pay(ride.ticket_price_cents) {
                    ride.value_rejections = ride.value_rejections.saturating_add(1);
                    reject_applicant(guest, ride, GuestThought::TooExpensive, self.ticks);
                    graph.step_guest(
                        guest,
                        &mut self.crowd_rng,
                        self.visitors.walk_goal(guest.id),
                    );
                    continue;
                }
                guest.think(GuestThought::HeadingToRide, self.ticks);
                guest.ride_visit = Some(RideVisit {
                    ride: id,
                    phase: RideVisitPhase::Approaching,
                    ticket: None,
                    waited_ticks: 0,
                    passenger: None,
                });
                guest.next = route.pop_front();
                guest.destination = guest.next.map(|_| entry);
                guest.route = route;
                guest.progress = 0;
                guest.state = if guest.next.is_some() {
                    GuestState::Walking
                } else {
                    GuestState::Waiting
                };
                if guest.next.is_none() {
                    applicants.push(index);
                }
                continue;
            }
        }
        applicants.sort_unstable_by_key(|index| self.guests[*index].id);
        let mut occupied = self
            .guests
            .iter()
            .filter(|guest| {
                guest.ride_visit.as_ref().is_some_and(|visit| {
                    visit.ride == id
                        && matches!(
                            visit.phase,
                            RideVisitPhase::Queueing
                                | RideVisitPhase::Leaving
                                | RideVisitPhase::Boarding
                        )
                })
            })
            .count();
        for index in applicants {
            let guest = &mut self.guests[index];
            let ride = &mut ride;
            if guest.wallet_cents < ride.ticket_price_cents {
                ride.unaffordable_rejections = ride.unaffordable_rejections.saturating_add(1);
                reject_applicant(guest, ride, GuestThought::NotEnoughMoney, self.ticks);
            } else if !guest.willing_to_pay(ride.ticket_price_cents) {
                ride.value_rejections = ride.value_rejections.saturating_add(1);
                reject_applicant(guest, ride, GuestThought::TooExpensive, self.ticks);
            } else if !eligible
                || occupied >= access.queue_tiles
                || ride.next_queue_ticket == u64::MAX
            {
                reject_applicant(
                    guest,
                    ride,
                    if occupied >= access.queue_tiles {
                        GuestThought::QueueFull
                    } else {
                        GuestThought::RideCancelled
                    },
                    self.ticks,
                );
            } else if let Some(&tail) = access.ordered_queue.last()
                && !reserved.contains_key(&tail)
            {
                guest.ride_visit = Some(RideVisit {
                    ride: id,
                    phase: RideVisitPhase::Queueing,
                    ticket: Some(ride.next_queue_ticket),
                    waited_ticks: 0,
                    passenger: None,
                });
                ride.next_queue_ticket += 1;
                ride.joined = ride.joined.saturating_add(1);
                occupied += 1;
                start_edge(guest, tail, &objects, &mut reserved);
                guest.think(GuestThought::Queueing, self.ticks);
            }
        }
        if occupied == 0 {
            ride.draining = false;
        }
        *self.ride_by_id_mut(id).unwrap() = ride;
    }
}

/// Validate membership separately from ordinary path walking. Disconnected
/// topology is a legitimate saved evacuation; ownership and reservations remain
/// strict even when a bridge, gate, station, or occupied tile was demolished.
pub(crate) fn validate_admission(
    park: &Park,
    objects: &BTreeMap<ObjectId, &SurfaceObject>,
) -> Result<(), String> {
    let fail = |message: &str| Err(message.to_owned());
    for guest in &park.guests {
        if guest.ride_cooldown > RIDE_COOLDOWN_TICKS
            || guest
                .ride_visit
                .as_ref()
                .is_some_and(|v| park.ride_by_id(v.ride).is_none())
            || (park.all_rides().next().is_none() && guest.ride_cooldown != 0)
        {
            return fail("guest ride state has no allocated owner or invalid cooldown");
        }
    }
    for ride in park.all_rides() {
        validate_ride_admission(park, objects, ride)?;
    }
    Ok(())
}

fn validate_ride_admission(
    park: &Park,
    objects: &BTreeMap<ObjectId, &SurfaceObject>,
    ride: &crate::Ride,
) -> Result<(), String> {
    let fail = |message: &str| Err(message.to_owned());
    if ride.next_queue_ticket == 0
        || ride.joined != ride.next_queue_ticket - 1
        || ride.abandoned > ride.joined
    {
        return fail("ride admission counters or ticket allocator are inconsistent");
    }
    let access = park.ride_access(ride.id);
    let eligible = park.admission_eligible(ride.id, &access);
    let indices: BTreeMap<_, _> = access
        .ordered_queue
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index))
        .collect();
    let mut tickets = std::collections::BTreeSet::new();
    let mut reserved = BTreeMap::new();
    let mut ranks = BTreeMap::new();
    let mut queueing = 0;
    let mut boarding = 0;
    let mut leaving = 0;
    for guest in &park.guests {
        if guest.ride_cooldown > RIDE_COOLDOWN_TICKS {
            return fail("guest ride cooldown is out of range");
        }
        let Some(visit) = guest.ride_visit.as_ref().filter(|v| v.ride == ride.id) else {
            continue;
        };
        if guest.ride_cooldown != 0 {
            return fail("guest visit has foreign ride or active cooldown");
        }
        if visit.phase == RideVisitPhase::Approaching {
            if visit.ticket.is_some()
                || visit.waited_ticks != 0
                || guest.wait_ticks != 0
                || !eligible
                || ride.draining
                || guest.state == GuestState::Lost
                || (guest.next.is_some() && guest.destination != access.public_entry)
                || (guest.next.is_none() && guest.current != access.public_entry)
            {
                return fail("approaching visit is inconsistent with its public entry");
            }
            continue;
        }
        let Some(ticket) = visit.ticket else {
            return fail("admitted guest lacks a ticket");
        };
        if ticket == 0 || ticket >= ride.next_queue_ticket || !tickets.insert(ticket) {
            return fail("queue tickets must be unique allocated identities");
        }
        if crate::journeys::passenger_phase(visit.phase) {
            if visit.phase == RideVisitPhase::Boarding {
                boarding += 1;
                if let Some(transfer) = visit
                    .passenger
                    .as_ref()
                    .and_then(|passenger| passenger.transfer.as_ref())
                    && reserved
                        .insert(transfer.path, guest.id)
                        .is_some_and(|other| other != guest.id)
                {
                    return fail("two guests reserve a boarding queue tile");
                }
            }
            if visit.waited_ticks > QUEUE_PATIENCE_TICKS {
                return fail("passenger queue patience is out of range");
            }
            continue;
        }
        if visit.waited_ticks > QUEUE_PATIENCE_TICKS
            || guest.wait_ticks != 0
            || !guest.route.is_empty()
            || guest.destination != guest.next
        {
            return fail("queue movement or patience state is inconsistent");
        }
        for id in [guest.current, guest.next].into_iter().flatten() {
            let path = objects
                .get(&id)
                .ok_or("queue guest references a missing object")?;
            if path.queue_owner.is_some()
                && reserved
                    .insert(id, guest.id)
                    .is_some_and(|other| other != guest.id)
            {
                return fail("two guests reserve the same queue tile");
            }
        }
        match visit.phase {
            RideVisitPhase::Queueing => {
                queueing += 1;
                if !eligible
                    || ride.draining
                    || visit.waited_ticks >= QUEUE_PATIENCE_TICKS
                    || guest.state == GuestState::Lost
                {
                    return fail(
                        "active queue member belongs to unavailable or draining admission",
                    );
                }
                let current = guest.current.ok_or("queue member lacks current path")?;
                let index = if Some(current) == access.public_entry
                    && guest.next == access.ordered_queue.last().copied()
                {
                    access.queue_tiles
                } else {
                    *indices
                        .get(&current)
                        .ok_or("queue member is outside its ordered queue")?
                };
                if let Some(next) = guest.next
                    && (index == 0 || indices.get(&next).copied() != Some(index - 1))
                {
                    return fail("queue member walks against its admission order");
                }
                ranks.insert(
                    ticket,
                    index as u64 * u64::from(WALK_TICKS) - u64::from(guest.progress),
                );
            }
            RideVisitPhase::Leaving => {
                leaving += 1;
                if !ride.draining {
                    return fail("leaving member has no batch drain");
                }
                if let (Some(current), Some(next)) = (guest.current, guest.next)
                    && objects[&current].is_public_path()
                    && objects[&next].is_public_path()
                {
                    return fail("leaving member walks between ordinary paths");
                }
            }
            RideVisitPhase::Approaching
            | RideVisitPhase::Boarding
            | RideVisitPhase::Riding
            | RideVisitPhase::Exiting => unreachable!(),
        }
    }
    if ride.draining != (leaving > 0)
        || queueing + boarding > access.queue_tiles
        || ride.boarded > ride.joined - ride.abandoned
        || (queueing + boarding) as u64 != ride.joined - ride.abandoned - ride.boarded
        || leaving as u64 > ride.abandoned
    {
        return fail("ride queue counts disagree with drain or cumulative counters");
    }
    let ranks: Vec<_> = ranks.values().copied().collect();
    if ranks.windows(2).any(|pair| pair[0] >= pair[1]) {
        return fail("queue ticket order overtakes an earlier member");
    }
    Ok(())
}

fn reject_applicant(guest: &mut Guest, ride: &mut crate::Ride, thought: GuestThought, tick: u64) {
    guest.ride_visit = None;
    guest.ride_cooldown = RIDE_COOLDOWN_TICKS;
    ride.turned_away = ride.turned_away.saturating_add(1);
    guest.think(thought, tick);
}
fn end_visit(guest: &mut Guest) {
    guest.ride_visit = None;
    guest.ride_cooldown = RIDE_COOLDOWN_TICKS;
    guest.state = GuestState::Waiting;
    guest.destination = None;
    guest.route.clear();
    guest.wait_ticks = 0;
}
fn anchor_on(guest: &mut Guest, path: &SurfaceObject) {
    guest.current = Some(path.id);
    guest.anchor = path.pos;
    guest.height = path.height;
    guest.anchor_slope = path.path_slope;
}
fn start_edge(
    guest: &mut Guest,
    next: ObjectId,
    objects: &BTreeMap<ObjectId, SurfaceObject>,
    reserved: &mut BTreeMap<ObjectId, GuestId>,
) {
    guest.next = Some(next);
    guest.destination = Some(next);
    guest.progress = 0;
    guest.route.clear();
    guest.wait_ticks = 0;
    guest.state = GuestState::Walking;
    if objects[&next].queue_owner.is_some() {
        reserved.insert(next, guest.id);
    }
}
fn finish_edge(
    guest: &mut Guest,
    objects: &BTreeMap<ObjectId, SurfaceObject>,
    reserved: &mut BTreeMap<ObjectId, GuestId>,
) {
    guest.progress += 1;
    if guest.progress == WALK_TICKS {
        if let Some(current) = guest.current
            && objects[&current].queue_owner.is_some()
        {
            reserved.remove(&current);
        }
        anchor_on(guest, &objects[&guest.next.unwrap()]);
        guest.next = None;
        guest.destination = None;
        guest.progress = 0;
        guest.state = GuestState::Waiting;
    }
}
pub(crate) fn reservations(
    guests: &[Guest],
    objects: &BTreeMap<ObjectId, SurfaceObject>,
) -> BTreeMap<ObjectId, GuestId> {
    guests
        .iter()
        .filter(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.ticket.is_some())
        })
        .flat_map(|guest| {
            let boarding = guest
                .ride_visit
                .as_ref()
                .filter(|visit| visit.phase == RideVisitPhase::Boarding)
                .and_then(|visit| {
                    visit
                        .passenger
                        .as_ref()?
                        .transfer
                        .as_ref()
                        .map(|transfer| transfer.path)
                });
            [guest.current, guest.next, boarding]
                .into_iter()
                .flatten()
                .filter(|id| {
                    objects
                        .get(id)
                        .is_some_and(|path| path.queue_owner.is_some())
                })
                .map(move |id| (id, guest.id))
        })
        .collect()
}

/// Reverse BFS from public boundaries is built once per crowd tick. It never
/// traverses a public node to reach another queue component or a foreign owner.
struct EscapeGraph {
    next: BTreeMap<ObjectId, ObjectId>,
}
impl EscapeGraph {
    fn new(objects: &BTreeMap<ObjectId, SurfaceObject>, ride: RideId) -> Self {
        let queue: Vec<_> = objects
            .values()
            .filter(|path| path.queue_owner == Some(ride))
            .collect();
        let mut edges: BTreeMap<ObjectId, Vec<ObjectId>> =
            queue.iter().map(|path| (path.id, Vec::new())).collect();
        let mut next = BTreeMap::new();
        let mut frontier = VecDeque::new();
        for path in &queue {
            for other in objects
                .values()
                .filter(|other| other.is_public_path() || other.queue_owner == Some(ride))
            {
                if paths_connect(path, other) {
                    if other.is_public_path() {
                        if let std::collections::btree_map::Entry::Vacant(entry) =
                            next.entry(path.id)
                        {
                            entry.insert(other.id);
                            frontier.push_back(path.id);
                        }
                    } else {
                        edges.get_mut(&path.id).unwrap().push(other.id);
                    }
                }
            }
        }
        while let Some(current) = frontier.pop_front() {
            for &neighbor in &edges[&current] {
                if let std::collections::btree_map::Entry::Vacant(entry) = next.entry(neighbor) {
                    entry.insert(current);
                    frontier.push_back(neighbor);
                }
            }
        }
        Self { next }
    }
    fn next_out(&self, current: ObjectId) -> Option<ObjectId> {
        self.next.get(&current).copied()
    }
}

#[cfg(test)]
mod tests;
