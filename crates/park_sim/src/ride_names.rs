//! Bounded, owner-scoped player ride names. Defaults are derived from ride kind.
use crate::{MAX_RIDE_RECORDS, Park, RideId, RideKind};

pub const MAX_RIDE_NAME_CHARS: usize = 28;
pub const MAX_RIDE_NAME_BYTES: usize = 96;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideName {
    pub ride: RideId,
    pub name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideNameError {
    UnknownRide,
    Empty,
    ControlCharacter,
    TooLong,
    TooManyRecords,
}
impl std::fmt::Display for RideNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnknownRide => "ride is unavailable",
            Self::Empty => "enter a ride name",
            Self::ControlCharacter => "ride name contains a control character",
            Self::TooLong => "ride name is too long",
            Self::TooManyRecords => "too many named rides",
        };
        f.write_str(message)
    }
}
impl std::error::Error for RideNameError {}

pub fn default_name(kind: RideKind) -> &'static str {
    match kind {
        RideKind::Carousel => "Meadow Carousel",
        RideKind::Coaster => "Meadow Flyer",
    }
}

fn valid_name(name: &str) -> Result<&str, RideNameError> {
    if name.chars().any(char::is_control) {
        return Err(RideNameError::ControlCharacter);
    }
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(RideNameError::Empty);
    }
    if trimmed.chars().count() > MAX_RIDE_NAME_CHARS || trimmed.len() > MAX_RIDE_NAME_BYTES {
        return Err(RideNameError::TooLong);
    }
    Ok(trimmed)
}

impl Park {
    pub fn ride_name(&self, ride: RideId) -> Option<&str> {
        let kind = self.ride_kind(ride)?;
        Some(
            self.ride_names
                .binary_search_by_key(&ride, |row| row.ride)
                .map_or(default_name(kind), |index| {
                    self.ride_names[index].name.as_str()
                }),
        )
    }

    pub fn set_ride_name(&mut self, ride: RideId, name: &str) -> Result<(), RideNameError> {
        let kind = self.ride_kind(ride).ok_or(RideNameError::UnknownRide)?;
        let name = valid_name(name)?;
        let custom = name != default_name(kind);
        match self.ride_names.binary_search_by_key(&ride, |row| row.ride) {
            Ok(index) if custom => self.ride_names[index].name = name.to_owned(),
            Ok(index) => {
                self.ride_names.remove(index);
            }
            Err(index) if custom => {
                if self.ride_names.len() >= MAX_RIDE_RECORDS {
                    return Err(RideNameError::TooManyRecords);
                }
                self.ride_names.insert(
                    index,
                    RideName {
                        ride,
                        name: name.to_owned(),
                    },
                );
            }
            Err(_) => {}
        }
        Ok(())
    }

    pub fn reset_ride_name(&mut self, ride: RideId) -> Result<(), RideNameError> {
        let kind = self.ride_kind(ride).ok_or(RideNameError::UnknownRide)?;
        self.set_ride_name(ride, default_name(kind))
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    if park.ride_names.len() > MAX_RIDE_RECORDS {
        return Err("too many named rides".into());
    }
    let mut previous = None;
    for row in &park.ride_names {
        let kind = park
            .ride_kind(row.ride)
            .ok_or("named ride is unavailable")?;
        if previous.is_some_and(|id| id >= row.ride)
            || valid_name(&row.name).is_err()
            || row.name.trim() != row.name
            || row.name == default_name(kind)
        {
            return Err("invalid or noncanonical ride name record".into());
        }
        previous = Some(row.ride);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TilePos;

    #[test]
    fn names_are_bounded_owner_scoped_and_roundtrip() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let coaster = park.ride().unwrap().id;
        let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        assert_eq!(park.ride_name(coaster), Some("Meadow Flyer"));
        assert_eq!(park.ride_name(carousel), Some("Meadow Carousel"));
        park.set_ride_name(coaster, "  Éclair Loop  ").unwrap();
        park.set_ride_name(carousel, "Moonlight Round").unwrap();
        assert_eq!(park.ride_name(coaster), Some("Éclair Loop"));
        assert_eq!(park.ride_name(carousel), Some("Moonlight Round"));
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
        let before = park.clone();
        let too_long = "x".repeat(29);
        for (ride, name, error) in [
            (RideId(u64::MAX), "Name", RideNameError::UnknownRide),
            (coaster, "   ", RideNameError::Empty),
            (coaster, "Bad\nName", RideNameError::ControlCharacter),
            (coaster, too_long.as_str(), RideNameError::TooLong),
        ] {
            assert_eq!(park.set_ride_name(ride, name), Err(error));
            assert_eq!(park, before);
        }
        park.reset_ride_name(coaster).unwrap();
        assert_eq!(park.ride_name(coaster), Some("Meadow Flyer"));
        assert_eq!(park.ride_name(carousel), Some("Moonlight Round"));
    }

    #[test]
    fn exact_schema_23_migrates_without_names_and_current_reader_rejects_forged_records() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let id = park.ride().unwrap().id;
        park.apply_ride_palette_preset(id, crate::RidePalettePreset::Harvest)
            .unwrap();
        let mut old: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        old["schema_version"] = 23.into();
        old["content_version"] = 23.into();
        old["park"].as_object_mut().unwrap().remove("ride_names");
        old["park"].as_object_mut().unwrap().remove("maintenance");
        old["park"]
            .as_object_mut()
            .unwrap()
            .remove("mechanic_hired");
        old["park"]
            .as_object_mut()
            .unwrap()
            .remove("mechanic_wage_progress");
        old["park"]["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove("wages_cents");
        let migrated = Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).unwrap();
        assert_eq!(migrated, park);
        assert_eq!(migrated.ride_name(id), Some("Meadow Flyer"));
        assert_eq!(migrated.ride_appearance(id), park.ride_appearance(id));
        old["park"]["ride_names"] = serde_json::json!([]);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).is_err());

        let mut current: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        for bad in [
            serde_json::json!([{"ride": id.0, "name": "Meadow Flyer"}]),
            serde_json::json!([{"ride": id.0, "name": " Bad "}]),
            serde_json::json!([{"ride": 999999, "name": "Lost"}]),
            serde_json::json!([{"ride": id.0, "name": "A"}, {"ride": id.0, "name": "B"}]),
        ] {
            current["park"]["ride_names"] = bad;
            assert!(Park::from_save_bytes(&serde_json::to_vec(&current).unwrap()).is_err());
        }
    }
}
