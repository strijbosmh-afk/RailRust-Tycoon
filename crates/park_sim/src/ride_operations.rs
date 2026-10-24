//! Bounded, persistent settings for the two supported ride families.
use crate::{Park, RideId, RideKind};

pub const MIN_STATION_WAIT_TICKS: u16 = 30;
pub const MAX_STATION_WAIT_TICKS: u16 = 900;
pub const STATION_WAIT_STEP_TICKS: u16 = 30;
pub const MIN_LIFT_SPEED: u32 = 12;
pub const MAX_LIFT_SPEED: u32 = 48;
pub const MIN_BRAKE_SPEED: u32 = 8;
pub const MAX_BRAKE_SPEED: u32 = 40;
pub const EQUIPMENT_SPEED_STEP: u32 = 4;
pub const LIFT_SPEED_STEP: u32 = EQUIPMENT_SPEED_STEP;
pub const BRAKE_SPEED_STEP: u32 = EQUIPMENT_SPEED_STEP;
pub const MIN_CAROUSEL_CYCLE_TICKS: u16 = 360;
pub const MAX_CAROUSEL_CYCLE_TICKS: u16 = 1800;
pub const CAROUSEL_CYCLE_STEP_TICKS: u16 = 180;

/// Speeds use fixed distance units per simulation tick, not real-world units.
/// A ride family's unsupported fields must retain their default values.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideOperatingSettings {
    pub station_wait_ticks: u16,
    pub lift_speed: u32,
    pub brake_speed: u32,
    pub cycle_ticks: u16,
}

impl Default for RideOperatingSettings {
    fn default() -> Self {
        Self {
            station_wait_ticks: crate::STATION_DWELL_TICKS,
            lift_speed: crate::trains::LIFT_SPEED,
            brake_speed: crate::trains::BRAKE_SPEED,
            cycle_ticks: crate::CAROUSEL_RUN_TICKS,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RideOperatingSettingsRecord {
    pub(crate) ride: RideId,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    pub(crate) settings: RideOperatingSettings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideOperatingError {
    UnknownRide(RideId),
    Unavailable,
    CloseAndStop,
    Passengers,
    InvalidStationWait,
    InvalidLiftSpeed,
    InvalidBrakeSpeed,
    InvalidCycleDuration,
    UnsupportedSetting,
}

impl std::fmt::Display for RideOperatingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRide(id) => write!(f, "Ride #{} does not exist", id.0),
            Self::Unavailable => f.write_str("Restore this ride before changing its settings"),
            Self::CloseAndStop => f.write_str("Close the ride and wait for it to stop first"),
            Self::Passengers => f.write_str("Let all passengers leave before changing settings"),
            Self::InvalidStationWait => f.write_str("Station wait must be 1 to 30 whole seconds"),
            Self::InvalidLiftSpeed => f.write_str("Lift speed must be 12 to 48 in steps of 4"),
            Self::InvalidBrakeSpeed => f.write_str("Brake speed must be 8 to 40 in steps of 4"),
            Self::InvalidCycleDuration => {
                f.write_str("Carousel cycle must be 12 to 60 seconds in steps of 6")
            }
            Self::UnsupportedSetting => {
                f.write_str("This ride does not support that operating setting")
            }
        }
    }
}

impl std::error::Error for RideOperatingError {}

impl RideOperatingSettings {
    fn validate(self, kind: RideKind) -> Result<(), RideOperatingError> {
        if !(MIN_STATION_WAIT_TICKS..=MAX_STATION_WAIT_TICKS).contains(&self.station_wait_ticks)
            || !self
                .station_wait_ticks
                .is_multiple_of(STATION_WAIT_STEP_TICKS)
        {
            return Err(RideOperatingError::InvalidStationWait);
        }
        if !(MIN_LIFT_SPEED..=MAX_LIFT_SPEED).contains(&self.lift_speed)
            || !self.lift_speed.is_multiple_of(EQUIPMENT_SPEED_STEP)
        {
            return Err(RideOperatingError::InvalidLiftSpeed);
        }
        if !(MIN_BRAKE_SPEED..=MAX_BRAKE_SPEED).contains(&self.brake_speed)
            || !self.brake_speed.is_multiple_of(EQUIPMENT_SPEED_STEP)
        {
            return Err(RideOperatingError::InvalidBrakeSpeed);
        }
        if !(MIN_CAROUSEL_CYCLE_TICKS..=MAX_CAROUSEL_CYCLE_TICKS).contains(&self.cycle_ticks)
            || !self.cycle_ticks.is_multiple_of(CAROUSEL_CYCLE_STEP_TICKS)
        {
            return Err(RideOperatingError::InvalidCycleDuration);
        }
        let defaults = Self::default();
        if (kind == RideKind::Coaster && self.cycle_ticks != defaults.cycle_ticks)
            || (kind == RideKind::Carousel
                && (self.lift_speed != defaults.lift_speed
                    || self.brake_speed != defaults.brake_speed))
        {
            return Err(RideOperatingError::UnsupportedSetting);
        }
        Ok(())
    }
}

impl Park {
    pub fn ride_operating_settings(&self, id: RideId) -> Option<RideOperatingSettings> {
        self.ride_by_id(id)?;
        Some(
            self.ride_operating_settings
                .iter()
                .find(|entry| entry.ride == id)
                .map_or_else(RideOperatingSettings::default, |entry| entry.settings),
        )
    }

    pub fn ride_operating_settings_status(&self, id: RideId) -> Result<(), RideOperatingError> {
        self.ride_by_id(id)
            .ok_or(RideOperatingError::UnknownRide(id))?;
        if !self.ride_available(id) {
            return Err(RideOperatingError::Unavailable);
        }
        if self.ride_is_open(id) || !self.ride_stationary(id) {
            return Err(RideOperatingError::CloseAndStop);
        }
        if self.guests.iter().any(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.ride == id && visit.passenger.is_some())
        }) {
            return Err(RideOperatingError::Passengers);
        }
        Ok(())
    }

    /// Validate before mutating. A repeated setting is a no-op even when the
    /// ride is now operating; it neither resets timers nor affects passengers.
    pub fn set_ride_operating_settings(
        &mut self,
        id: RideId,
        settings: RideOperatingSettings,
    ) -> Result<bool, RideOperatingError> {
        let kind = self
            .ride_kind(id)
            .ok_or(RideOperatingError::UnknownRide(id))?;
        settings.validate(kind)?;
        if self.ride_operating_settings(id) == Some(settings) {
            return Ok(false);
        }
        self.ride_operating_settings_status(id)?;
        self.ride_operating_settings
            .retain(|entry| entry.ride != id);
        if settings != RideOperatingSettings::default() {
            self.ride_operating_settings
                .push(RideOperatingSettingsRecord { ride: id, settings });
            self.ride_operating_settings.sort_by_key(|entry| entry.ride);
        }
        Ok(true)
    }

    pub(crate) fn coaster_operating_settings(&self) -> RideOperatingSettings {
        self.ride
            .as_ref()
            .and_then(|ride| self.ride_operating_settings(ride.id))
            .unwrap_or_default()
    }
}

pub(crate) fn validate_saved_settings(park: &Park) -> Result<(), String> {
    if park.ride_operating_settings.len() > crate::MAX_RIDE_RECORDS {
        return Err("too many ride operating settings".into());
    }
    let mut previous = None;
    for entry in &park.ride_operating_settings {
        if previous.is_some_and(|id| id >= entry.ride) {
            return Err("ride operating settings must have unique sorted owners".into());
        }
        previous = Some(entry.ride);
        let kind = park
            .ride_kind(entry.ride)
            .ok_or("operating settings have an unknown ride owner")?;
        entry
            .settings
            .validate(kind)
            .map_err(|error| error.to_string())?;
        if entry.settings == RideOperatingSettings::default() {
            return Err("default ride settings must not be stored as overrides".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
