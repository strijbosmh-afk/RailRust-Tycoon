//! One coaster train, with two seats in each of its one to three compact cars.
use crate::{Park, RideId, RideKind, RideOperatingError};

pub const MIN_COASTER_CARS: u8 = 1;
pub const MAX_COASTER_CARS: u8 = 3;
pub const DEFAULT_COASTER_CARS: u8 = 3;
pub const SEATS_PER_COASTER_CAR: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrainConfigurationError {
    Operating(RideOperatingError),
    InvalidCars,
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
            Self::InvalidCars => f.write_str("Choose 1 to 3 coaster cars"),
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
        (self.ride_kind(id) == Some(RideKind::Coaster)).then_some(self.coaster_cars)
    }

    pub fn ride_capacity(&self, id: RideId) -> Option<usize> {
        match self.ride_kind(id)? {
            RideKind::Coaster => Some(usize::from(self.coaster_cars * SEATS_PER_COASTER_CAR)),
            RideKind::Carousel => Some(crate::TRAIN_SEATS),
        }
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
        if !(MIN_COASTER_CARS..=MAX_COASTER_CARS).contains(&cars) {
            return Err(TrainConfigurationError::InvalidCars);
        }
        if cars == current {
            return Ok(false);
        }
        self.ride_operating_settings_status(id)?;
        let minimum = self
            .ride_dispatch_policy(id)
            .expect("known ride")
            .minimum_passengers;
        let capacity = cars * SEATS_PER_COASTER_CAR;
        if minimum > capacity {
            return Err(TrainConfigurationError::MinimumLoad { minimum, capacity });
        }
        self.coaster_cars = cars;
        self.reset_ride_dispatch_wait(id);
        Ok(true)
    }
}

pub(crate) fn validate_saved_configuration(park: &Park) -> Result<(), String> {
    if !(MIN_COASTER_CARS..=MAX_COASTER_CARS).contains(&park.coaster_cars)
        || (park.ride.is_none() && park.coaster_cars != DEFAULT_COASTER_CARS)
    {
        return Err("coaster car configuration is invalid".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
