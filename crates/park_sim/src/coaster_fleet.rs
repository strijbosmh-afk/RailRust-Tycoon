//! Deterministic whole-train block occupancy and station transfers.
use crate::{Park, RideId, RideKind, Train, TrainCircuit, TrainError, TrainPhase};

pub const MAX_COASTER_TRAINS: u8 = 3;

impl TrainCircuit {
    pub fn block_boundaries(&self) -> Vec<u32> {
        let mut distance = 0;
        let mut result = Vec::new();
        for piece in &self.pieces {
            if matches!(
                piece.kind,
                crate::TrackKind::Station | crate::TrackKind::Brake
            ) {
                result.push(distance);
            }
            distance += piece.length;
        }
        result
    }

    pub(crate) fn validate_blocks(&self, trains: u8, cars: u8) -> Result<(), TrainError> {
        if trains == 1 {
            return Ok(());
        }
        let boundaries = self.block_boundaries();
        let span = u32::from(cars - 1) * 320 + 241;
        if boundaries.len() <= usize::from(trains)
            || boundaries.iter().enumerate().any(|(i, start)| {
                let end = boundaries[(i + 1) % boundaries.len()];
                (end + self.total_length - start) % self.total_length < span
            })
        {
            return Err(TrainError::UnsafeBlocks);
        }
        Ok(())
    }

    fn occupied_blocks(&self, train: &Train, cars: u8) -> Vec<usize> {
        if train.in_depot {
            return Vec::new();
        }
        let boundaries = self.block_boundaries();
        let block = |point: u32| {
            boundaries
                .iter()
                .rposition(|start| *start <= point)
                .unwrap_or(boundaries.len() - 1)
        };
        let distance = train.distance % self.total_length;
        let front = (distance + 120) % self.total_length;
        let tail =
            (distance + self.total_length - (u32::from(cars - 1) * 320 + 120)) % self.total_length;
        vec![block(tail), block(front)]
    }
}

impl Park {
    pub fn coaster_block_occupancy(&self, id: RideId) -> Vec<Option<u8>> {
        let Some(circuit) = self.coaster_circuit(id) else {
            return Vec::new();
        };
        let mut blocks = vec![None; circuit.block_boundaries().len()];
        for train in self.coaster_trains(id) {
            for block in circuit.occupied_blocks(train, self.coaster_car_count(id).unwrap_or(3)) {
                blocks[block] = Some(train.number);
            }
        }
        blocks
    }
    pub(crate) fn validate_draft_fleet(
        &self,
        specs: &[crate::TrackPieceSpec],
    ) -> Result<(), TrainError> {
        if self.extra_trains.is_empty() {
            return Ok(());
        }
        let pieces: Vec<_> = specs
            .iter()
            .map(|spec| {
                let samples = spec.samples();
                let length = samples
                    .windows(2)
                    .map(|p| crate::trains::point_distance(p[0], p[1]))
                    .sum();
                crate::TrackPiece {
                    object: crate::ObjectId(0),
                    kind: spec.kind,
                    samples,
                    length,
                }
            })
            .collect();
        let circuit = TrainCircuit {
            total_length: pieces.iter().map(|p| p.length).sum(),
            pieces,
            specs: specs.to_vec(),
            station_stop: 0,
        };
        circuit.validate_blocks(self.extra_trains.len() as u8 + 1, self.coaster_cars)
    }
    pub(crate) fn select_first_train(&mut self) {
        if let Some(index) = self.extra_trains.iter().position(|t| t.number == 0) {
            std::mem::swap(self.train.as_mut().unwrap(), &mut self.extra_trains[index]);
        }
    }
    pub fn coaster_trains(&self, id: RideId) -> Vec<&Train> {
        let (train, extras) = if self.ride.as_ref().is_some_and(|r| r.id == id) {
            (&self.train, &self.extra_trains)
        } else if let Some(c) = self
            .other_coasters
            .iter()
            .find(|c| c.ride.as_ref().is_some_and(|r| r.id == id))
        {
            (&c.train, &c.extra_trains)
        } else {
            return Vec::new();
        };
        let mut trains: Vec<_> = train.iter().chain(extras).collect();
        trains.sort_by_key(|train| train.number);
        trains
    }

    pub fn coaster_train_count(&self, id: RideId) -> Option<u8> {
        if self.ride.as_ref().is_some_and(|r| r.id == id) {
            Some(1 + self.extra_trains.len() as u8)
        } else {
            self.other_coasters
                .iter()
                .find(|c| c.ride.as_ref().is_some_and(|r| r.id == id))
                .map(|c| 1 + c.extra_trains.len() as u8)
        }
    }

    pub fn coaster_train_by_number(&self, id: RideId, number: u8) -> Option<&Train> {
        if self.ride.as_ref().is_some_and(|r| r.id == id) {
            self.train
                .iter()
                .chain(&self.extra_trains)
                .find(|t| t.number == number)
        } else {
            let coaster = self
                .other_coasters
                .iter()
                .find(|c| c.ride.as_ref().is_some_and(|r| r.id == id))?;
            coaster
                .train
                .iter()
                .chain(&coaster.extra_trains)
                .find(|t| t.number == number)
        }
    }

    pub(crate) fn passenger_running(&self, id: RideId, seat: u8) -> bool {
        if self.ride_kind(id) != Some(RideKind::Coaster) {
            return self.ride_is_running(id);
        }
        let number = seat / self.coaster_seats_per_train(id).unwrap_or(1);
        self.coaster_train_by_number(id, number)
            .is_some_and(|t| t.departed)
    }

    pub(crate) fn station_seat(&self, id: RideId, seat: u8) -> bool {
        if self.ride_kind(id) != Some(RideKind::Coaster) {
            return true;
        }
        self.coaster_train(id).is_some_and(|t| {
            !t.in_depot
                && !t.departed
                && t.number == seat / self.coaster_seats_per_train(id).unwrap_or(1)
        })
    }

    pub(crate) fn loading_stationary(&self, id: RideId) -> bool {
        if self.ride_kind(id) != Some(RideKind::Coaster) {
            return self.ride_stationary(id);
        }
        self.coaster_train(id).is_some_and(|t| {
            !t.in_depot && !t.departed && t.speed == 0 && t.phase != TrainPhase::Fault
        })
    }

    pub(crate) fn rebuild_extra_trains(&mut self) -> Result<(), TrainError> {
        let Some(circuit) = &self.circuit else {
            return Ok(());
        };
        circuit.validate_blocks(self.extra_trains.len() as u8 + 1, self.coaster_cars)?;
        let Some(primary) = &self.train else {
            return Ok(());
        };
        for extra in &mut self.extra_trains {
            let number = extra.number;
            let laps = extra.laps;
            *extra = primary.clone();
            extra.number = number;
            extra.laps = laps;
            extra.in_depot = true;
            extra.open = primary.open;
            extra.phase = TrainPhase::Closed;
            extra.dwell_ticks = 0;
        }
        Ok(())
    }

    pub(crate) fn step_coaster_fleet(&mut self) {
        if self.extra_trains.is_empty() {
            self.step_train();
            return;
        }
        if self.circuit.is_none() {
            return;
        }
        let id = self.ride.as_ref().expect("coaster identity").id;
        // A closed, empty arrival frees the station for the next returning train.
        let empty_station = self
            .train
            .as_ref()
            .is_some_and(|t| !t.open && !t.departed && !t.in_depot)
            && self.extra_trains.iter().any(|t| t.departed)
            && !self.guests.iter().any(|g| {
                g.ride_visit.as_ref().is_some_and(|v| {
                    v.ride == id
                        && v.passenger
                            .as_ref()
                            .is_some_and(|p| self.station_seat(id, p.seat))
                })
            });
        if empty_station {
            self.train.as_mut().unwrap().in_depot = true;
        }

        let count = self.extra_trains.len() as u8 + 1;
        for number in 0..count {
            let index = self.extra_trains.iter().position(|t| t.number == number);
            if let Some(index) = index {
                std::mem::swap(self.train.as_mut().unwrap(), &mut self.extra_trains[index]);
            }
            if !self.train.as_ref().unwrap().in_depot {
                let before = self.train.as_ref().unwrap().clone();
                let dispatch_before = self.ride_dispatch.clone();
                if before.blocked {
                    let speed = self.coaster_operating_settings().brake_speed;
                    self.train.as_mut().unwrap().speed = speed;
                    self.train.as_mut().unwrap().blocked = false;
                }
                self.step_train();
                let circuit = self.circuit.as_ref().unwrap();
                let occupied =
                    circuit.occupied_blocks(self.train.as_ref().unwrap(), self.coaster_cars);
                if self.extra_trains.iter().any(|other| {
                    circuit
                        .occupied_blocks(other, self.coaster_cars)
                        .iter()
                        .any(|b| occupied.contains(b))
                }) {
                    self.train = Some(before);
                    self.ride_dispatch = dispatch_before;
                    let train = self.train.as_mut().unwrap();
                    if train.departed {
                        train.speed = 0;
                        train.blocked = true;
                    }
                }
            }
            if let Some(index) = index {
                std::mem::swap(self.train.as_mut().unwrap(), &mut self.extra_trains[index]);
            }
        }
        if let Some(index) = self
            .extra_trains
            .iter()
            .position(|t| !t.in_depot && !t.departed)
        {
            std::mem::swap(self.train.as_mut().unwrap(), &mut self.extra_trains[index]);
            self.reset_ride_dispatch_wait(id);
        }
        if self.train.as_ref().unwrap().open
            && !self.loading_stationary(id)
            && let Some(index) = self.extra_trains.iter().position(|t| t.in_depot)
        {
            let mut candidate = self.extra_trains[index].clone();
            candidate.in_depot = false;
            let circuit = self.circuit.as_ref().unwrap();
            let station = circuit.occupied_blocks(&candidate, self.coaster_cars);
            let occupied = self
                .train
                .iter()
                .chain(&self.extra_trains)
                .filter(|t| t.number != candidate.number)
                .any(|t| {
                    circuit
                        .occupied_blocks(t, self.coaster_cars)
                        .iter()
                        .any(|b| station.contains(b))
                });
            if !occupied {
                candidate.phase = TrainPhase::Station;
                candidate.dwell_ticks = self.coaster_operating_settings().station_wait_ticks;
                self.extra_trains[index] = candidate;
                std::mem::swap(self.train.as_mut().unwrap(), &mut self.extra_trains[index]);
                self.reset_ride_dispatch_wait(id);
            }
        }
        self.extra_trains.sort_by_key(|t| t.number);
    }

    pub(crate) fn validate_fleet(&self) -> Result<(), String> {
        if !(1..=crate::MAX_COASTER_CARS).contains(&self.coaster_cars) {
            return Err("invalid car count in train fleet".into());
        }
        let Some(train) = &self.train else {
            return if self.extra_trains.is_empty() {
                Ok(())
            } else {
                Err("fleet without a primary train".into())
            };
        };
        let mut numbers: Vec<_> = self
            .train
            .iter()
            .chain(&self.extra_trains)
            .map(|t| t.number)
            .collect();
        numbers.sort_unstable();
        if numbers.len() > usize::from(MAX_COASTER_TRAINS)
            || numbers != (0..numbers.len() as u8).collect::<Vec<_>>()
        {
            return Err("train numbers must be unique and consecutive".into());
        }
        if self.extra_trains.iter().any(|t| t.open != train.open) {
            return Err("fleet open states disagree".into());
        }
        if self
            .train
            .iter()
            .chain(&self.extra_trains)
            .all(|t| t.in_depot)
        {
            return Err("fleet must retain an active train".into());
        }
        if let Some(circuit) = &self.circuit {
            circuit
                .validate_blocks(numbers.len() as u8, self.coaster_cars)
                .map_err(|e| e.to_string())?;
            let trains: Vec<_> = self.train.iter().chain(&self.extra_trains).collect();
            for (i, a) in trains.iter().enumerate() {
                if trains[i + 1..].iter().any(|b| {
                    circuit
                        .occupied_blocks(a, self.coaster_cars)
                        .iter()
                        .any(|block| {
                            circuit
                                .occupied_blocks(b, self.coaster_cars)
                                .contains(block)
                        })
                }) {
                    return Err("multiple trains occupy the same block".into());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PathDirection, PathSlope, Speed, TilePos, TrackKind};

    fn park() -> (Park, RideId) {
        let mut source = Park::new(32, 32);
        source.install_train_fixture().unwrap();
        let mut specs = source.circuit().unwrap().specs().to_vec();
        for piece in &mut specs {
            if [(20, 10), (16, 4)].contains(&(piece.pos.x, piece.pos.y)) {
                piece.kind = TrackKind::Brake;
            }
        }
        let mut park = Park::new(32, 32);
        park.install_train_circuit(&specs).unwrap();
        let id = park.ride().unwrap().id;
        park.set_ride_open(id, false).unwrap();
        park.set_coaster_train_count(id, 3).unwrap();
        (park, id)
    }

    #[test]
    fn blocks_hold_whole_trains_and_continue_exactly_after_reload() {
        let (mut park, id) = park();
        park.set_ride_open(id, true).unwrap();
        let mut held = false;
        let mut all_dispatched = false;
        for tick in 0..18000 {
            park.advance(Speed::Normal);
            park.validate_fleet().unwrap();
            held |= park.coaster_trains(id).iter().any(|t| t.blocked);
            all_dispatched |= park.coaster_trains(id).iter().all(|t| !t.in_depot);
            if tick == 3000 {
                let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
                for _ in 0..600 {
                    park.advance(Speed::Normal);
                    loaded.advance(Speed::Normal);
                }
                assert_eq!(park, loaded);
            }
        }
        assert!(held && all_dispatched);
        assert!(park.coaster_trains(id).iter().all(|t| t.laps >= 2));
        park.set_ride_open(id, false).unwrap();
        for _ in 0..9000 {
            park.advance(Speed::Normal);
        }
        assert!(!park.ride_is_running(id));
        assert!(park.coaster_trains(id).iter().all(|t| !t.open));
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        park.set_coaster_train_count(id, 1).unwrap();
        assert_eq!(park.coaster_trains(id).len(), 1);
    }

    #[test]
    fn fleet_passengers_board_the_station_train_and_pay_once_per_trip() {
        let (mut park, id) = park();
        park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, id)
            .unwrap();
        for y in 12..=14 {
            park.place_queue_path(TilePos { x: 8, y }, 8, PathSlope::Flat, id)
                .unwrap();
        }
        let entry = park.place_path(TilePos { x: 8, y: 15 }, 8).unwrap();
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, id)
            .unwrap();
        for x in 8..=13 {
            park.place_path(TilePos { x, y: 8 }, 8).unwrap();
        }
        for y in 9..=15 {
            park.place_path(TilePos { x: 13, y }, 8).unwrap();
        }
        for x in 9..=12 {
            park.place_path(TilePos { x, y: 15 }, 8).unwrap();
        }
        park.set_visitor_entry(entry).unwrap();
        park.set_visitor_arrivals(true).unwrap();
        park.set_ride_ticket_price(id, 100).unwrap();
        park.set_ride_open_checked(id, true).unwrap();
        let mut occupied_numbers = std::collections::BTreeSet::new();
        for tick in 0..24000 {
            park.advance(Speed::Normal);
            for position in park.ride_guest_positions() {
                occupied_numbers.insert(position.seat / 6);
            }
            if tick % 1200 == 0 {
                Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
            }
        }
        assert_eq!(occupied_numbers.len(), 3);
        assert!(park.ride().unwrap().completed >= 12);
        park.set_ride_open(id, false).unwrap();
        for _ in 0..14000 {
            park.advance(Speed::Normal);
        }
        assert!(park.guests().iter().all(|g| {
            g.ride_visit
                .as_ref()
                .is_none_or(|v| v.ride != id || v.passenger.is_none())
        }));
        assert_eq!(
            park.ride().unwrap().revenue_cents,
            park.ride().unwrap().completed * 100
        );
        assert!(park.can_service_ride(id));
        park.service_ride(id).unwrap();
    }

    #[test]
    fn faulted_fleet_can_downsize_before_rebuilding_a_smaller_layout() {
        let (mut park, id) = park();
        let removed = park.circuit().unwrap().pieces[1].object;
        park.remove_object(removed).unwrap();
        park.set_coaster_train_count(id, 1).unwrap();
        assert_eq!(park.coaster_train_count(id), Some(1));
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        park.install_train_fixture().unwrap();
        assert_eq!(park.coaster_train_count(id), Some(1));
    }

    #[test]
    fn unsafe_fleet_configuration_and_saved_overlap_are_rejected() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let id = park.ride().unwrap().id;
        park.set_ride_open(id, false).unwrap();
        let before = park.clone();
        assert!(park.set_coaster_train_count(id, 3).is_err());
        assert_eq!(park, before);
        let (mut park, _) = self::park();
        park.extra_trains[0].in_depot = false;
        assert!(park.to_save_bytes().is_err());
    }
}
