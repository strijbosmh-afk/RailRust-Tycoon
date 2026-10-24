//! Physical station transfers and stable passenger seats. No boarding shortcuts.
use crate::{
    Guest, GuestId, GuestState, ObjectId, ObjectKind, PATH_UNITS, Park, PathDirection, PathPoint,
    PathSlope, RideAccessError, RideAccessStatus, RideEntrance, RideId, RideVisitPhase,
    SurfaceObject, TRACK_UNITS, TilePos,
};
use std::collections::{BTreeMap, BTreeSet};

/// Historical maximum and fixed carousel capacity; use ride_capacity for a coaster.
pub const TRAIN_SEATS: usize = 6;
pub const RIDE_TRANSFER_TICKS: u16 = 60;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideTransfer {
    pub path: ObjectId,
    pub portal: ObjectId,
    pub progress: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassengerJourney {
    pub seat: u8,
    pub departed: bool,
    pub completed_lap: bool,
    /// Locked at boarding. No debit occurs until the first actual departure.
    pub fare_cents: u32,
    pub fare_paid: bool,
    #[serde(deserialize_with = "crate::saves::required_object_option")]
    pub transfer: Option<RideTransfer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideGuestPosition {
    pub guest: GuestId,
    pub ride: RideId,
    pub seat: u8,
    pub phase: RideVisitPhase,
    pub point: PathPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideExitStatus {
    Ready,
    RideUnavailable,
    NoExit,
    StationDisconnected,
    NoPublicExit,
    PublicDisconnected,
}
impl std::fmt::Display for RideExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Ready => "Exit connected",
            Self::RideUnavailable => "Ride circuit unavailable",
            Self::NoExit => "Build a ride exit",
            Self::StationDisconnected => "Exit does not reach station",
            Self::NoPublicExit => "Exit needs an outward public path",
            Self::PublicDisconnected => "Connect exit path to queue entry network",
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideExitAccess {
    pub status: RideExitStatus,
    pub public_exit: Option<ObjectId>,
    pub station: Option<ObjectId>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideJourneyStats {
    pub capacity: usize,
    pub boarding: usize,
    pub riding: usize,
    pub exiting: usize,
    pub boarded: u64,
    pub exited: u64,
    pub completed: u64,
    pub held: bool,
    pub exit_status: RideExitStatus,
}

impl Park {
    pub fn validate_ride_exit(
        &self,
        pos: TilePos,
        height: u8,
        facing: PathDirection,
        ride: RideId,
    ) -> Result<(), RideAccessError> {
        if self.require_ride(ride)?.exit.is_some() {
            return Err(RideAccessError::ExitAlreadyExists);
        }
        self.validate_object_placement(pos, ObjectKind::RideExit, height, crate::PATH_CLEARANCE)?;
        if self.objects.len() >= crate::saves::MAX_OBJECTS {
            return Err(crate::PlacementError::CapacityExceeded {
                resource: "objects",
                max: crate::saves::MAX_OBJECTS,
            }
            .into());
        }
        self.entrance_station(ride, pos, height, facing)?;
        Ok(())
    }
    pub fn place_ride_exit(
        &mut self,
        pos: TilePos,
        height: u8,
        facing: PathDirection,
        ride: RideId,
    ) -> Result<ObjectId, RideAccessError> {
        self.validate_ride_exit(pos, height, facing, ride)?;
        let object = self.insert_surface_object(
            SurfaceObject {
                id: ObjectId(0),
                pos,
                kind: ObjectKind::RideExit,
                axis: crate::ride_access::axis(facing),
                path_slope: PathSlope::Flat,
                queue_owner: None,
                height,
                clearance: crate::PATH_CLEARANCE,
            },
            self.next_object_id + 1,
        );
        self.ride_by_id_mut(ride).unwrap().exit = Some(RideEntrance { object, facing });
        Ok(object)
    }
    pub fn ride_exit_access(&self, id: RideId) -> RideExitAccess {
        let mut access = RideExitAccess {
            status: RideExitStatus::RideUnavailable,
            public_exit: None,
            station: None,
        };
        let Some(ride) = self.ride_by_id(id) else {
            return access;
        };
        if !self.ride_available(id) {
            return access;
        }
        let Some(exit) = ride.exit else {
            access.status = RideExitStatus::NoExit;
            return access;
        };
        let Some(gate) = self.object(exit.object) else {
            access.status = RideExitStatus::NoExit;
            return access;
        };
        let Ok(station) = self.entrance_station(id, gate.pos, gate.height, exit.facing) else {
            access.status = RideExitStatus::StationDisconnected;
            return access;
        };
        access.station = Some(station);
        access.public_exit = self
            .objects
            .iter()
            .filter(|path| self.exit_connects(gate, path))
            .map(|path| path.id)
            .min();
        let Some(landing) = access.public_exit else {
            access.status = RideExitStatus::NoPublicExit;
            return access;
        };
        let queue = self.ride_access(id);
        access.status = if queue
            .public_entry
            .is_some_and(|entry| self.public_paths_connected(landing, entry))
        {
            RideExitStatus::Ready
        } else {
            RideExitStatus::PublicDisconnected
        };
        access
    }
    pub(crate) fn exit_connects(&self, gate: &SurfaceObject, path: &SurfaceObject) -> bool {
        if gate.kind != ObjectKind::RideExit || !path.is_public_path() {
            return false;
        }
        let Some(exit) = self
            .all_rides()
            .filter_map(|ride| ride.exit)
            .find(|exit| exit.object == gate.id)
        else {
            return false;
        };
        exit.facing.opposite().neighbor(gate.pos) == Some(path.pos)
            && path.path_edge_height(exit.facing) == Some(gate.height)
    }
    pub fn object_removal_block_reason(&self, id: ObjectId) -> Option<&'static str> {
        let owner = self.object_ride(id);
        for visit in self.guests.iter().filter_map(|g| g.ride_visit.as_ref()) {
            let Some(passenger) = &visit.passenger else {
                continue;
            };
            if passenger
                .transfer
                .as_ref()
                .is_some_and(|t| t.path == id || t.portal == id)
            {
                return Some("Guests are using this boarding or exit walkway");
            }
            if owner == Some(visit.ride)
                && self
                    .object(id)
                    .is_some_and(|o| matches!(o.kind, ObjectKind::Track | ObjectKind::Carousel))
            {
                return Some("Let all passengers leave before removing this ride");
            }
        }
        None
    }
    pub fn ride_journey_stats(&self, id: RideId) -> RideJourneyStats {
        let access = self.ride_exit_access(id);
        let mut stats = RideJourneyStats {
            capacity: self.ride_capacity(id).unwrap_or(0),
            boarding: 0,
            riding: 0,
            exiting: 0,
            boarded: 0,
            exited: 0,
            completed: 0,
            held: false,
            exit_status: access.status,
        };
        let Some(ride) = self.ride_by_id(id) else {
            return stats;
        };
        stats.boarded = ride.boarded;
        stats.exited = ride.exited;
        stats.completed = ride.completed;
        stats.held = self.ride_holds(id);
        for guest in &self.guests {
            match guest
                .ride_visit
                .as_ref()
                .filter(|v| v.ride == id)
                .map(|visit| visit.phase)
            {
                Some(RideVisitPhase::Boarding) => stats.boarding += 1,
                Some(RideVisitPhase::Riding) => stats.riding += 1,
                Some(RideVisitPhase::Exiting) => stats.exiting += 1,
                _ => {}
            }
        }
        stats
    }
    pub fn ride_guest_positions(&self) -> Vec<RideGuestPosition> {
        self.guests
            .iter()
            .filter_map(|guest| {
                let visit = guest.ride_visit.as_ref()?;
                let passenger = visit.passenger.as_ref()?;
                Some(RideGuestPosition {
                    guest: guest.id,
                    ride: visit.ride,
                    seat: passenger.seat,
                    phase: visit.phase,
                    point: self.ride_guest_point(guest)?,
                })
            })
            .collect()
    }
    pub(crate) fn ride_guest_point(&self, guest: &Guest) -> Option<PathPoint> {
        let visit = guest.ride_visit.as_ref()?;
        let passenger = visit.passenger.as_ref()?;
        let seat = self.seat_point(visit.ride, passenger.seat)?;
        let Some(transfer) = &passenger.transfer else {
            return Some(seat);
        };
        let path = self.object(transfer.path)?.path_center()?;
        let gate = self.object(transfer.portal)?;
        let middle = PathPoint {
            x: gate.pos.x as i32 * PATH_UNITS,
            y: gate.pos.y as i32 * PATH_UNITS,
            height: i32::from(gate.height) * PATH_UNITS,
        };
        let half = RIDE_TRANSFER_TICKS / 2;
        let path_to_gate = |t: u16| {
            let edge = PathPoint {
                x: (path.x + middle.x) / 2,
                y: (path.y + middle.y) / 2,
                height: middle.height,
            };
            if t <= half / 2 {
                mix_points(path, edge, t, half / 2)
            } else {
                mix_points(edge, middle, t - half / 2, half / 2)
            }
        };
        Some(if visit.phase == RideVisitPhase::Boarding {
            if transfer.progress <= half {
                path_to_gate(transfer.progress)
            } else {
                mix_points(middle, seat, transfer.progress - half, half)
            }
        } else if transfer.progress <= half {
            mix_points(seat, middle, transfer.progress, half)
        } else {
            path_to_gate(RIDE_TRANSFER_TICKS - transfer.progress)
        })
    }
    fn seat_point(&self, id: RideId, seat: u8) -> Option<PathPoint> {
        if usize::from(seat) >= self.ride_capacity(id)? {
            return None;
        }
        if self.carousel(id).is_some() {
            return self
                .carousel_seat_positions(id)
                .get(usize::from(seat))
                .copied();
        }
        let pose = *self.train_cars().get(usize::from(seat) / 2)?;
        let dx = i64::from(pose.front.x - pose.rear.x);
        let dy = i64::from(pose.front.y - pose.rear.y);
        let length = ((dx * dx + dy * dy) as u64).isqrt().max(1) as i64;
        let offset = if seat.is_multiple_of(2) { -64 } else { 64 };
        let x = i64::from(pose.center.x) - dy * offset / length;
        let y = i64::from(pose.center.y) + dx * offset / length;
        Some(PathPoint {
            x: (x * i64::from(PATH_UNITS) / i64::from(TRACK_UNITS)) as i32,
            y: (y * i64::from(PATH_UNITS) / i64::from(TRACK_UNITS)) as i32,
            height: pose.center.height * PATH_UNITS / TRACK_UNITS + 35,
        })
    }
    pub(crate) fn ride_holds_train(&self) -> bool {
        self.ride.as_ref().is_some_and(|r| self.ride_holds(r.id))
    }
    pub(crate) fn ride_holds(&self, id: RideId) -> bool {
        let mut occupied = false;
        let mut completed = false;
        for passenger in self.guests.iter().filter_map(|g| {
            g.ride_visit
                .as_ref()
                .filter(|v| v.ride == id)?
                .passenger
                .as_ref()
        }) {
            occupied = true;
            completed |= passenger.completed_lap;
            if passenger.transfer.is_some() {
                return true;
            }
        }
        occupied
            && !self.ride_is_running(id)
            && (completed || self.ride_exit_access(id).status != RideExitStatus::Ready)
    }
    pub(crate) fn step_ride_journeys(&mut self) {
        let mut ids: Vec<_> = self.all_rides().map(|r| r.id).collect();
        ids.sort_unstable();
        for id in ids {
            self.step_owner_journeys(id);
        }
    }
    fn step_owner_journeys(&mut self, ride_id: RideId) {
        if self.ride_by_id(ride_id).unwrap().exit.is_none()
            && self.guests.iter().all(|g| {
                g.ride_visit
                    .as_ref()
                    .is_none_or(|v| v.ride != ride_id || v.passenger.is_none())
            })
        {
            return;
        } // At most one guest uses a station walkway at a time. A completed
        // transfer keeps its exact endpoint when switching between render roles.
        for index in 0..self.guests.len() {
            let Some(visit) = self.guests[index]
                .ride_visit
                .as_mut()
                .filter(|v| v.ride == ride_id)
            else {
                continue;
            };
            let Some(passenger) = &mut visit.passenger else {
                continue;
            };
            let Some(transfer) = &mut passenger.transfer else {
                continue;
            };
            transfer.progress += 1;
            if transfer.progress < RIDE_TRANSFER_TICKS {
                continue;
            }
            if visit.phase == RideVisitPhase::Boarding {
                visit.phase = RideVisitPhase::Riding;
                passenger.transfer = None;
                self.guests[index].state = GuestState::Waiting;
                self.ride_by_id_mut(ride_id).unwrap().boarded += 1;
                self.reset_ride_dwell(ride_id);
            } else {
                let path_id = transfer.path;
                let completed = passenger.completed_lap;
                let fare = passenger.fare_cents;
                let waited = visit.waited_ticks;
                let path = *self.object(path_id).expect("protected transfer landing");
                let tick = self.ticks;
                let (ride, guests) = self.ride_and_guests_mut(ride_id).unwrap();
                let guest = &mut guests[index];
                guest.current = Some(path.id);
                guest.anchor = path.pos;
                guest.height = path.height;
                guest.anchor_slope = path.path_slope;
                guest.state = GuestState::Waiting;
                guest.ride_visit = None;
                guest.ride_cooldown = crate::RIDE_COOLDOWN_TICKS;
                // Prevent ordinary movement from choosing a new route on the
                // same tick that a later system observes the public endpoint.
                guest.wait_ticks = 1;
                ride.exited += 1;
                if completed {
                    ride.completed += 1;
                    guest.rides_completed = guest.rides_completed.saturating_add(1);
                    guest.rate_completed_trip(ride, fare, waited, tick);
                }
            }
        }
        if !self.ride_stationary(ride_id)
            || self.guests.iter().any(|guest| {
                guest.ride_visit.as_ref().is_some_and(|visit| {
                    visit.ride == ride_id
                        && visit
                            .passenger
                            .as_ref()
                            .is_some_and(|passenger| passenger.transfer.is_some())
                })
            })
        {
            return;
        }
        let exit = self.ride_exit_access(ride_id);
        let unload = self
            .guests
            .iter()
            .enumerate()
            .filter_map(|(index, guest)| {
                let visit = guest.ride_visit.as_ref().filter(|v| v.ride == ride_id)?;
                let passenger = visit.passenger.as_ref()?;
                (visit.phase == RideVisitPhase::Riding
                    && (passenger.completed_lap || !self.ride_is_open(ride_id)))
                .then_some((passenger.seat, index))
            })
            .min();
        if let Some((_, index)) = unload {
            if let (Some(path), Some(_), Some(portal)) = (
                exit.public_exit,
                exit.station,
                self.ride_by_id(ride_id).unwrap().exit,
            ) {
                let visit = self.guests[index].ride_visit.as_mut().unwrap();
                visit.phase = RideVisitPhase::Exiting;
                visit.passenger.as_mut().unwrap().transfer = Some(RideTransfer {
                    path,
                    portal: portal.object,
                    progress: 0,
                });
                self.guests[index].state = GuestState::Walking;
                if !self.guests[index]
                    .ride_visit
                    .as_ref()
                    .unwrap()
                    .passenger
                    .as_ref()
                    .unwrap()
                    .completed_lap
                {
                    self.guests[index].cancel_experience(self.ticks);
                }
            }
            return;
        }
        if !self.ride_is_open(ride_id)
            || exit.status != RideExitStatus::Ready
            || self.ride_by_id(ride_id).unwrap().draining
        {
            return;
        }
        let used: BTreeSet<_> = self
            .guests
            .iter()
            .filter_map(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .filter(|v| v.ride == ride_id)?
                    .passenger
                    .as_ref()
                    .map(|passenger| passenger.seat)
            })
            .collect();
        let Some(seat) =
            (0..self.ride_capacity(ride_id).unwrap_or(0) as u8).find(|seat| !used.contains(seat))
        else {
            return;
        };
        let access = self.ride_access(ride_id);
        if access.status != RideAccessStatus::Ready {
            return;
        }
        let Some(front) = access.ordered_queue.first().copied() else {
            return;
        };
        let candidate = self
            .guests
            .iter()
            .enumerate()
            .filter_map(|(index, guest)| {
                let visit = guest.ride_visit.as_ref().filter(|v| v.ride == ride_id)?;
                (visit.phase == RideVisitPhase::Queueing
                    && guest.current == Some(front)
                    && guest.next.is_none())
                .then_some((visit.ticket?, index))
            })
            .min();
        if let Some((_, index)) = candidate {
            let fare_cents = self.ride_by_id(ride_id).unwrap().ticket_price_cents;
            if self.guests[index].wallet_cents < fare_cents {
                self.drain_queue_for_funds(&access, self.guests[index].id);
                return;
            }
            if !self.guests[index].willing_to_pay(fare_cents) {
                self.drain_queue_for_value(&access, self.guests[index].id);
                return;
            }
            if !self.can_reserve_rating(index) {
                return;
            }
            let portal = self.ride_by_id(ride_id).unwrap().entrance.unwrap().object;
            let guest = &mut self.guests[index];
            let visit = guest.ride_visit.as_mut().unwrap();
            visit.phase = RideVisitPhase::Boarding;
            visit.passenger = Some(PassengerJourney {
                seat,
                departed: false,
                completed_lap: false,
                fare_cents,
                fare_paid: false,
                transfer: Some(RideTransfer {
                    path: front,
                    portal,
                    progress: 0,
                }),
            });
            guest.current = None;
            guest.next = None;
            guest.destination = None;
            guest.route.clear();
            guest.progress = 0;
            guest.wait_ticks = 0;
            guest.state = GuestState::Walking;
        }
    }
    pub(crate) fn reconcile_passenger_laps(&mut self) {
        let running: BTreeMap<_, _> = self
            .all_rides()
            .map(|r| (r.id, self.ride_is_running(r.id)))
            .collect();
        for guest in &mut self.guests {
            if let Some(visit) = &mut guest.ride_visit
                && visit.phase == RideVisitPhase::Riding
                && let Some(passenger) = &mut visit.passenger
            {
                if running.get(&visit.ride) == Some(&true) {
                    passenger.departed = true;
                } else if passenger.departed {
                    passenger.completed_lap = true;
                }
            }
        }
        self.settle_departure_fares();
    }
}

pub(crate) fn passenger_phase(phase: RideVisitPhase) -> bool {
    matches!(
        phase,
        RideVisitPhase::Boarding | RideVisitPhase::Riding | RideVisitPhase::Exiting
    )
}
fn mix_points(a: PathPoint, b: PathPoint, t: u16, steps: u16) -> PathPoint {
    let mix = |a: i32, b: i32| a + (b - a) * i32::from(t) / i32::from(steps);
    PathPoint {
        x: mix(a.x, b.x),
        y: mix(a.y, b.y),
        height: mix(a.height, b.height),
    }
}

pub(crate) fn validate_journeys(
    park: &Park,
    objects: &BTreeMap<ObjectId, &SurfaceObject>,
) -> Result<(), String> {
    if park.guests.iter().any(|g| {
        g.ride_visit
            .as_ref()
            .is_some_and(|v| park.ride_by_id(v.ride).is_none())
    }) {
        return Err("guest visit has no ride".into());
    }
    for ride in park.all_rides() {
        validate_owner_journeys(park, objects, ride)?;
    }
    Ok(())
}
fn validate_owner_journeys(
    park: &Park,
    objects: &BTreeMap<ObjectId, &SurfaceObject>,
    ride: &crate::Ride,
) -> Result<(), String> {
    if ride.boarded > ride.joined.saturating_sub(ride.abandoned)
        || ride.exited > ride.boarded
        || ride.completed > ride.exited
    {
        return Err("ride journey counters are inconsistent".into());
    }
    let mut seats = BTreeSet::new();
    let mut transfers = 0;
    let mut riding = 0;
    let mut completed_total = 0_u64;
    for guest in &park.guests {
        completed_total = completed_total
            .checked_add(
                guest
                    .ride_history
                    .iter()
                    .find(|h| h.ride == ride.id)
                    .map_or(0, |h| h.completed),
            )
            .ok_or("guest completion counters overflow")?;
        let Some(visit) = guest.ride_visit.as_ref().filter(|v| v.ride == ride.id) else {
            continue;
        };
        if !passenger_phase(visit.phase) {
            if visit.passenger.is_some() {
                return Err("ordinary visit contains passenger data".into());
            }
            continue;
        }
        let passenger = visit
            .passenger
            .as_ref()
            .ok_or("passenger phase lacks seat state")?;
        let running = park.ride_is_running(ride.id);
        if !park.ride_available(ride.id)
            || usize::from(passenger.seat) >= park.ride_capacity(ride.id).unwrap_or(0)
            || !seats.insert(passenger.seat)
            || guest.current.is_some()
            || guest.next.is_some()
            || guest.destination.is_some()
            || !guest.route.is_empty()
            || guest.progress != 0
            || guest.wait_ticks != 0
        {
            return Err("passenger seat or walking fields are invalid".into());
        }
        if passenger.completed_lap && !passenger.departed {
            return Err("passenger completed a lap without departure".into());
        }
        match visit.phase {
            RideVisitPhase::Riding => {
                riding += 1;
                if guest.state != GuestState::Waiting
                    || passenger.transfer.is_some()
                    || (running && (!passenger.departed || passenger.completed_lap))
                    || (!running && passenger.departed != passenger.completed_lap)
                {
                    return Err("seated passenger disagrees with train journey".into());
                }
            }
            RideVisitPhase::Boarding | RideVisitPhase::Exiting => {
                transfers += 1;
                if !park.ride_stationary(ride.id) || guest.state != GuestState::Walking {
                    return Err("transfer requires a stopped train".into());
                }
                let transfer = passenger
                    .transfer
                    .as_ref()
                    .ok_or("walking passenger has no transfer")?;
                if transfer.progress >= RIDE_TRANSFER_TICKS {
                    return Err("transfer progress exceeds its duration".into());
                }
                let path = *objects
                    .get(&transfer.path)
                    .ok_or("transfer path is missing")?;
                let gate = *objects
                    .get(&transfer.portal)
                    .ok_or("transfer portal is missing")?;
                let portal = if visit.phase == RideVisitPhase::Boarding {
                    ride.entrance
                } else {
                    ride.exit
                }
                .ok_or("transfer portal metadata is missing")?;
                if portal.object != gate.id
                    || park
                        .entrance_station(ride.id, gate.pos, gate.height, portal.facing)
                        .is_err()
                {
                    return Err("transfer portal is not attached to station".into());
                }
                if visit.phase == RideVisitPhase::Boarding {
                    if passenger.departed
                        || passenger.completed_lap
                        || path.queue_owner != Some(ride.id)
                        || !park.geometric_paths_connect(gate, path)
                        || guest.anchor != path.pos
                        || guest.height != path.height
                        || guest.anchor_slope != path.path_slope
                    {
                        return Err("boarding transfer does not start on its queue front".into());
                    }
                } else {
                    riding += 1;
                    if !park.exit_connects(gate, path)
                        || (passenger.departed != passenger.completed_lap)
                    {
                        return Err("exit transfer has invalid landing or lap state".into());
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    if transfers > 1
        || riding as u64 != ride.boarded - ride.exited
        || completed_total != ride.completed
    {
        return Err("passenger counts or completion totals disagree".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
