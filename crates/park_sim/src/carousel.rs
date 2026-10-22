//! A player-built round ride sharing the normal queue and passenger pipeline.
use crate::{
    Axis, Guest, ObjectId, ObjectKind, PATH_UNITS, Park, PathPoint, PathSlope, PlacementError,
    Ride, RideAccessError, RideId, SurfaceObject, TilePos,
};
use std::collections::BTreeSet;

pub const MAX_CAROUSEL_RECORDS: usize = 8;
pub const MAX_RIDE_RECORDS: usize = MAX_CAROUSEL_RECORDS + 1;
pub const CAROUSEL_SEATS: usize = 6;
pub const CAROUSEL_RUN_TICKS: u16 = 720;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RideKind {
    Coaster,
    Carousel,
}
impl std::fmt::Display for RideKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Coaster => "Coaster",
            Self::Carousel => "Carousel",
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CarouselPhase {
    Closed,
    Station,
    Running,
}
impl std::fmt::Display for CarouselPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Closed => "Closed",
            Self::Station => "At platform",
            Self::Running => "Running",
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarouselRide {
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub ride: Ride,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub origin: TilePos,
    pub height: u8,
    #[serde(deserialize_with = "crate::saves::bounded_vec::<_,_,9>")]
    pub objects: Vec<ObjectId>,
    pub open: bool,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub phase: CarouselPhase,
    pub progress: u16,
    pub dwell_ticks: u16,
    pub cycles: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CarouselError {
    Placement(PlacementError),
    ActiveCarouselExists,
    HistoryFull,
    UnknownRide(RideId),
    Removed,
    Occupied,
}
impl From<PlacementError> for CarouselError {
    fn from(value: PlacementError) -> Self {
        Self::Placement(value)
    }
}
impl std::fmt::Display for CarouselError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Placement(e) => e.fmt(f),
            Self::ActiveCarouselExists => {
                f.write_str("Remove the existing carousel before building another")
            }
            Self::HistoryFull => {
                f.write_str("The park has reached its eight carousel history records")
            }
            Self::UnknownRide(id) => write!(f, "Ride #{} is not a carousel", id.0),
            Self::Removed => f.write_str("This carousel has been removed"),
            Self::Occupied => f.write_str("Let all passengers leave before removing the carousel"),
        }
    }
}
impl std::error::Error for CarouselError {}

impl Park {
    pub fn carousels(&self) -> &[CarouselRide] {
        &self.carousels
    }
    pub fn carousel(&self, id: RideId) -> Option<&CarouselRide> {
        self.carousels.iter().find(|c| c.ride.id == id)
    }
    pub fn all_rides(&self) -> impl Iterator<Item = &Ride> {
        self.ride
            .iter()
            .chain(self.carousels.iter().map(|c| &c.ride))
    }
    pub(crate) fn all_rides_mut(&mut self) -> impl Iterator<Item = &mut Ride> {
        self.ride
            .iter_mut()
            .chain(self.carousels.iter_mut().map(|c| &mut c.ride))
    }
    pub fn ride_by_id(&self, id: RideId) -> Option<&Ride> {
        self.all_rides().find(|r| r.id == id)
    }
    pub(crate) fn ride_by_id_mut(&mut self, id: RideId) -> Option<&mut Ride> {
        self.all_rides_mut().find(|r| r.id == id)
    }
    pub(crate) fn ride_and_guests_mut(&mut self, id: RideId) -> Option<(&mut Ride, &mut [Guest])> {
        let ride = self
            .ride
            .iter_mut()
            .chain(self.carousels.iter_mut().map(|c| &mut c.ride))
            .find(|r| r.id == id)?;
        Some((ride, &mut self.guests))
    }
    pub fn ride_kind(&self, id: RideId) -> Option<RideKind> {
        if self.ride.as_ref().is_some_and(|r| r.id == id) {
            Some(RideKind::Coaster)
        } else {
            self.carousel(id).map(|_| RideKind::Carousel)
        }
    }
    pub fn object_ride(&self, id: ObjectId) -> Option<RideId> {
        let object = self.object(id)?;
        if let Some(owner) = object.queue_owner {
            return Some(owner);
        }
        if let Some(ride) = self.all_rides().find(|ride| {
            ride.entrance.is_some_and(|g| g.object == id)
                || ride.exit.is_some_and(|g| g.object == id)
        }) {
            return Some(ride.id);
        }
        if let Some(c) = self.carousels.iter().find(|c| c.objects.contains(&id)) {
            return Some(c.ride.id);
        }
        if self
            .circuit
            .as_ref()
            .is_some_and(|c| c.pieces.iter().any(|p| p.object == id))
        {
            return self.ride.as_ref().map(|r| r.id);
        }
        None
    }
    pub(crate) fn ride_available(&self, id: RideId) -> bool {
        match self.ride_kind(id) {
            Some(RideKind::Coaster) => self.circuit.is_some(),
            Some(RideKind::Carousel) => self.carousel(id).is_some_and(|c| !c.objects.is_empty()),
            None => false,
        }
    }
    pub fn ride_is_open(&self, id: RideId) -> bool {
        match self.ride_kind(id) {
            Some(RideKind::Coaster) => self.train.as_ref().is_some_and(|t| t.open),
            Some(RideKind::Carousel) => self
                .carousel(id)
                .is_some_and(|c| c.open && !c.objects.is_empty()),
            None => false,
        }
    }
    pub fn ride_is_running(&self, id: RideId) -> bool {
        match self.ride_kind(id) {
            Some(RideKind::Coaster) => self.train.as_ref().is_some_and(|t| t.departed),
            Some(RideKind::Carousel) => self
                .carousel(id)
                .is_some_and(|c| c.phase == CarouselPhase::Running),
            None => false,
        }
    }
    pub(crate) fn ride_stationary(&self, id: RideId) -> bool {
        self.ride_available(id)
            && !self.ride_is_running(id)
            && (self.ride_kind(id) != Some(RideKind::Coaster)
                || self.train.as_ref().is_some_and(|t| t.speed == 0))
    }
    pub fn set_ride_open(&mut self, id: RideId, open: bool) -> Result<(), RideAccessError> {
        match self.ride_kind(id).ok_or(RideAccessError::UnknownRide(id))? {
            RideKind::Coaster => self.set_train_open(open),
            RideKind::Carousel => {
                let c = self.carousels.iter_mut().find(|c| c.ride.id == id).unwrap();
                if c.objects.is_empty() {
                    return Err(RideAccessError::CircuitUnavailable);
                }
                if c.open == open {
                    return Ok(());
                }
                c.open = open;
                if c.phase != CarouselPhase::Running {
                    c.phase = if open {
                        CarouselPhase::Station
                    } else {
                        CarouselPhase::Closed
                    };
                    c.dwell_ticks = if open { crate::STATION_DWELL_TICKS } else { 0 };
                }
                self.reconcile_ride_visits();
            }
        }
        Ok(())
    }
    pub(crate) fn reset_ride_dwell(&mut self, id: RideId) {
        match self.ride_kind(id) {
            Some(RideKind::Coaster) => {
                if let Some(t) = &mut self.train
                    && t.open
                {
                    t.dwell_ticks = crate::STATION_DWELL_TICKS;
                }
            }
            Some(RideKind::Carousel) => {
                if let Some(c) = self.carousels.iter_mut().find(|c| c.ride.id == id)
                    && c.open
                {
                    c.dwell_ticks = crate::STATION_DWELL_TICKS;
                }
            }
            None => {}
        }
    }
    pub fn validate_carousel(&self, origin: TilePos, height: u8) -> Result<(), CarouselError> {
        if self.carousels.iter().any(|c| !c.objects.is_empty()) {
            return Err(CarouselError::ActiveCarouselExists);
        }
        if self.carousels.len() >= MAX_CAROUSEL_RECORDS {
            return Err(CarouselError::HistoryFull);
        }
        if self.objects.len() + 9 > crate::saves::MAX_OBJECTS {
            return Err(PlacementError::CapacityExceeded {
                resource: "objects",
                max: crate::saves::MAX_OBJECTS,
            }
            .into());
        }
        self.next_ride_id
            .checked_add(1)
            .ok_or(PlacementError::IdExhausted)?;
        self.next_object_id
            .checked_add(9)
            .ok_or(PlacementError::IdExhausted)?;
        for y in 0..3 {
            for x in 0..3 {
                let pos = TilePos {
                    x: origin
                        .x
                        .checked_add(x)
                        .ok_or(PlacementError::OutOfBounds { pos: origin })?,
                    y: origin
                        .y
                        .checked_add(y)
                        .ok_or(PlacementError::OutOfBounds { pos: origin })?,
                };
                self.validate_object_placement(pos, ObjectKind::Carousel, height, 3)?;
            }
        }
        Ok(())
    }
    pub fn place_carousel(&mut self, origin: TilePos, height: u8) -> Result<RideId, CarouselError> {
        self.validate_carousel(origin, height)?;
        let id = RideId(self.next_ride_id);
        self.next_ride_id += 1;
        let mut objects = Vec::with_capacity(9);
        for y in 0..3 {
            for x in 0..3 {
                objects.push(self.insert_surface_object(
                    SurfaceObject {
                        id: ObjectId(0),
                        pos: TilePos {
                            x: origin.x + x,
                            y: origin.y + y,
                        },
                        kind: ObjectKind::Carousel,
                        axis: Axis::X,
                        path_slope: PathSlope::Flat,
                        queue_owner: None,
                        height,
                        clearance: 3,
                    },
                    self.next_object_id + 1,
                ));
            }
        }
        self.carousels.push(CarouselRide {
            ride: Ride::new(id),
            origin,
            height,
            objects,
            open: false,
            phase: CarouselPhase::Closed,
            progress: 0,
            dwell_ticks: 0,
            cycles: 0,
        });
        Ok(id)
    }
    pub fn remove_carousel(&mut self, id: RideId) -> Result<(), CarouselError> {
        let c = self.carousel(id).ok_or(CarouselError::UnknownRide(id))?;
        if c.objects.is_empty() {
            return Err(CarouselError::Removed);
        }
        if self.guests.iter().any(|g| {
            g.ride_visit
                .as_ref()
                .is_some_and(|v| v.ride == id && v.passenger.is_some())
        }) {
            return Err(CarouselError::Occupied);
        }
        let c = self.carousels.iter_mut().find(|c| c.ride.id == id).unwrap();
        let mut footprint = std::mem::take(&mut c.objects);
        if let Some(gate) = c.ride.entrance.take() {
            footprint.push(gate.object);
        }
        if let Some(gate) = c.ride.exit.take() {
            footprint.push(gate.object);
        }
        c.open = false;
        c.phase = CarouselPhase::Closed;
        c.progress = 0;
        c.dwell_ticks = 0;
        self.objects.retain(|o| !footprint.contains(&o.id));
        self.reconcile_ride_visits();
        Ok(())
    }
    pub fn carousel_seat_positions(&self, id: RideId) -> Vec<PathPoint> {
        let Some(c) = self.carousel(id).filter(|c| !c.objects.is_empty()) else {
            return Vec::new();
        };
        // Six seats follow a symmetric 16-edge circle approximation. Integer
        // interpolation avoids platform-specific trigonometry and returns to
        // precisely the same boarding positions after 720 complete ticks.
        const RING: [(i32, i32); 16] = [
            (60, 0),
            (55, 23),
            (42, 42),
            (23, 55),
            (0, 60),
            (-23, 55),
            (-42, 42),
            (-55, 23),
            (-60, 0),
            (-55, -23),
            (-42, -42),
            (-23, -55),
            (0, -60),
            (23, -55),
            (42, -42),
            (55, -23),
        ];
        (0..CAROUSEL_SEATS)
            .map(|seat| {
                let phase = (usize::from(c.progress)
                    + seat * usize::from(CAROUSEL_RUN_TICKS) / CAROUSEL_SEATS)
                    % usize::from(CAROUSEL_RUN_TICKS);
                let index = phase / 45;
                let t = (phase % 45) as i32;
                let a = RING[index];
                let b = RING[(index + 1) % RING.len()];
                PathPoint {
                    x: (c.origin.x + 1) as i32 * PATH_UNITS + a.0 + (b.0 - a.0) * t / 45,
                    y: (c.origin.y + 1) as i32 * PATH_UNITS + a.1 + (b.1 - a.1) * t / 45,
                    height: i32::from(c.height) * PATH_UNITS + 35,
                }
            })
            .collect()
    }
    pub(crate) fn step_carousels(&mut self) {
        let mut ids: Vec<_> = self
            .carousels
            .iter()
            .filter(|c| !c.objects.is_empty())
            .map(|c| c.ride.id)
            .collect();
        ids.sort_unstable();
        for id in ids {
            let hold = self.ride_holds(id);
            let c = self.carousels.iter_mut().find(|c| c.ride.id == id).unwrap();
            if c.phase == CarouselPhase::Running {
                c.progress += 1;
                if c.progress == CAROUSEL_RUN_TICKS {
                    c.progress = 0;
                    c.cycles += 1;
                    c.phase = if c.open {
                        CarouselPhase::Station
                    } else {
                        CarouselPhase::Closed
                    };
                    c.dwell_ticks = if c.open {
                        crate::STATION_DWELL_TICKS
                    } else {
                        0
                    };
                }
            } else if c.open && !hold && c.cycles < u64::MAX {
                c.dwell_ticks = c.dwell_ticks.saturating_sub(1);
                if c.dwell_ticks == 0 {
                    c.phase = CarouselPhase::Running;
                    c.progress = 0;
                }
            }
        }
    }
}

pub(crate) fn validate_carousels(park: &Park) -> Result<(), String> {
    if park.carousels.len() > MAX_CAROUSEL_RECORDS
        || park
            .carousels
            .iter()
            .filter(|c| !c.objects.is_empty())
            .count()
            > 1
    {
        return Err("carousel record or active capacity exceeds its limit".into());
    }
    let mut ids = BTreeSet::new();
    let mut parts = BTreeSet::new();
    for ride in park.all_rides() {
        if ride.id.0 == 0 || ride.id.0 >= park.next_ride_id || !ids.insert(ride.id) {
            return Err("ride identities are duplicated or unallocated".into());
        }
    }
    for c in &park.carousels {
        if c.origin.x.checked_add(2).is_none_or(|x| x >= park.width)
            || c.origin.y.checked_add(2).is_none_or(|y| y >= park.height)
            || c.height > crate::MAX_TERRAIN_HEIGHT - 3
        {
            return Err("carousel footprint is outside the map or height range".into());
        }
        if c.objects.is_empty() {
            if c.open
                || c.phase != CarouselPhase::Closed
                || c.progress != 0
                || c.dwell_ticks != 0
                || c.ride.entrance.is_some()
                || c.ride.exit.is_some()
            {
                return Err("removed carousel has active operation state".into());
            }
        } else {
            if c.objects.len() != 9 {
                return Err("carousel footprint needs nine cells".into());
            }
            for (index, id) in c.objects.iter().enumerate() {
                let object = park
                    .object(*id)
                    .ok_or("carousel footprint object is missing")?;
                if !parts.insert(*id)
                    || object.kind != ObjectKind::Carousel
                    || object.pos
                        != (TilePos {
                            x: c.origin.x + index as u32 % 3,
                            y: c.origin.y + index as u32 / 3,
                        })
                    || object.height != c.height
                    || object.clearance != 3
                    || object.axis != Axis::X
                    || object.path_slope != PathSlope::Flat
                    || object.queue_owner.is_some()
                {
                    return Err("carousel footprint metadata is inconsistent".into());
                }
            }
        }
        if c.progress >= CAROUSEL_RUN_TICKS
            || c.dwell_ticks > crate::STATION_DWELL_TICKS
            || (c.phase == CarouselPhase::Running && (c.dwell_ticks != 0 || c.cycles == u64::MAX))
            || (c.phase != CarouselPhase::Running && c.progress != 0)
            || (c.phase == CarouselPhase::Closed && (c.open || c.dwell_ticks != 0))
            || (c.phase == CarouselPhase::Station && (!c.open || c.dwell_ticks == 0))
            || u128::from(c.cycles) * u128::from(CAROUSEL_RUN_TICKS) + u128::from(c.progress)
                > u128::from(park.ticks)
        {
            return Err("carousel operation counters or phase are invalid".into());
        }
    }
    if park
        .objects
        .iter()
        .any(|o| o.kind == ObjectKind::Carousel && !parts.contains(&o.id))
    {
        return Err("carousel object lacks ride metadata".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
