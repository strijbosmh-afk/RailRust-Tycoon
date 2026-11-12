//! One coaster train, with two seats in each of its one to three compact cars.
use crate::{Park, RideId, RideKind, RideOperatingError};

pub const MIN_COASTER_CARS: u8 = 1;
pub const MAX_COASTER_CARS: u8 = 3;
pub const DEFAULT_COASTER_CARS: u8 = 3;
pub const SEATS_PER_COASTER_CAR: u8 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrainConfigurationError {
    Operating(RideOperatingError),
    InvalidCars,
    InvalidTrains,
    Blocks(crate::TrainError),
    MinimumLoad { minimum: u8, capacity: u8 },
}
impl From<RideOperatingError> for TrainConfigurationError {
    fn from(value: RideOperatingError) -> Self {
        Self::Operating(value)
    }
}
impl std::fmt::Display for TrainConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Operating(error) => error.fmt(f),
            Self::InvalidCars => f.write_str("Choose a car count supported by this ride"),
            Self::InvalidTrains => f.write_str("Choose a train count supported by this ride"),
            Self::Blocks(error) => error.fmt(f),
            Self::MinimumLoad { minimum, capacity } => write!(
                f,
                "Lower minimum load to {capacity} before removing cars (now {minimum})"
            ),
        }
    }
}
impl std::error::Error for TrainConfigurationError {}

impl Park {
    pub fn coaster_car_count(&self, id: RideId) -> Option<u8> {
        if self.ride.as_ref().is_some_and(|r| r.id == id) {
            Some(self.coaster_cars)
        } else {
            self.other_coasters
                .iter()
                .find(|c| c.ride.as_ref().is_some_and(|r| r.id == id))
                .map(|c| c.cars)
        }
    }

    pub fn ride_capacity(&self, id: RideId) -> Option<usize> {
        match self.ride_kind(id)? {
            RideKind::Coaster => Some(usize::from(
                self.coaster_car_count(id)?
                    * self.ride_model(id)?.seats_per_car()
                    * self.coaster_train_count(id)?,
            )),
            RideKind::Carousel | RideKind::Swing | RideKind::DropTower => {
                Some(self.ride_model(id)?.flat_capacity())
            }
        }
    }

    pub fn coaster_seats_per_train(&self, id: RideId) -> Option<u8> {
        Some(self.coaster_car_count(id)? * self.ride_model(id)?.seats_per_car())
    }

    pub fn set_coaster_train_count(
        &mut self,
        id: RideId,
        count: u8,
    ) -> Result<bool, TrainConfigurationError> {
        let current = self
            .coaster_train_count(id)
            .ok_or(RideOperatingError::UnsupportedSetting)?;
        if !(1..=self.ride_model(id).unwrap().max_trains()).contains(&count) {
            return Err(TrainConfigurationError::InvalidTrains);
        }
        if count == current {
            return Ok(false);
        }
        if let Some(circuit) = self.coaster_circuit(id) {
            self.ride_operating_settings_status(id)?;
            circuit
                .validate_blocks(count, self.coaster_car_count(id).unwrap())
                .map_err(TrainConfigurationError::Blocks)?;
        } else if count >= current
            || self
                .coaster_trains(id)
                .iter()
                .any(|t| t.phase != crate::TrainPhase::Fault)
        {
            return Err(RideOperatingError::Unavailable.into());
        }
        self.with_coaster(id, |park| {
            let mut primary = park.train.as_ref().unwrap().clone();
            primary.number = 0;
            primary.in_depot = false;
            primary.blocked = false;
            primary.laps = park
                .train
                .iter()
                .chain(&park.extra_trains)
                .map(|t| t.laps)
                .fold(0u64, u64::saturating_add);
            park.extra_trains.clear();
            for number in 1..count {
                let mut train = primary.clone();
                train.number = number;
                train.laps = 0;
                train.in_depot = true;
                park.extra_trains.push(train);
            }
            park.train = Some(primary);
        });
        self.reset_ride_dispatch_wait(id);
        Ok(true)
    }

    pub fn set_coaster_car_count(
        &mut self,
        id: RideId,
        cars: u8,
    ) -> Result<bool, TrainConfigurationError> {
        let current = self.coaster_car_count(id).ok_or_else(|| {
            if self.ride_by_id(id).is_none() {
                RideOperatingError::UnknownRide(id)
            } else {
                RideOperatingError::UnsupportedSetting
            }
        })?;
        if !(MIN_COASTER_CARS..=self.ride_model(id).unwrap().max_cars()).contains(&cars) {
            return Err(TrainConfigurationError::InvalidCars);
        }
        if cars == current {
            return Ok(false);
        }
        self.ride_operating_settings_status(id)?;
        if let Some(circuit) = self.coaster_circuit(id) {
            circuit
                .validate_blocks(self.coaster_train_count(id).unwrap(), cars)
                .map_err(TrainConfigurationError::Blocks)?;
        }
        let minimum = self
            .ride_dispatch_policy(id)
            .expect("known ride")
            .minimum_passengers;
        let capacity = cars * self.ride_model(id).unwrap().seats_per_car();
        if minimum > capacity {
            return Err(TrainConfigurationError::MinimumLoad { minimum, capacity });
        }
        self.with_coaster(id, |park| park.coaster_cars = cars);
        self.reset_ride_dispatch_wait(id);
        Ok(true)
    }
}

pub(crate) fn validate_saved_configuration(park: &Park) -> Result<(), String> {
    if park
        .ride
        .as_ref()
        .is_some_and(|r| r.model.is_some_and(|m| !m.tracked()))
        || park.extra_trains.len() + 1 > usize::from(park.selected_ride_model().max_trains())
    {
        return Err("incompatible tracked ride model or train count".into());
    }

    if !(MIN_COASTER_CARS..=park.selected_ride_model().max_cars()).contains(&park.coaster_cars)
        || (park.ride.is_none() && park.coaster_cars != DEFAULT_COASTER_CARS)
    {
        return Err("coaster car configuration is invalid".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
