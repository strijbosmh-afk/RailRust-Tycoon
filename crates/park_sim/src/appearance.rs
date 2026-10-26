//! Cosmetic ride-owned paint, independent of operation, money and guest history.
use crate::{MAX_RIDE_RECORDS, Park, RideId};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RidePaint {
    #[default]
    Classic,
    Coral,
    Lagoon,
    Sunflower,
    Meadow,
    Sky,
    Violet,
    Charcoal,
    Cream,
}
impl RidePaint {
    pub const ALL: [Self; 9] = [
        Self::Classic,
        Self::Coral,
        Self::Lagoon,
        Self::Sunflower,
        Self::Meadow,
        Self::Sky,
        Self::Violet,
        Self::Charcoal,
        Self::Cream,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Coral => "Coral",
            Self::Lagoon => "Lagoon",
            Self::Sunflower => "Sunflower",
            Self::Meadow => "Meadow",
            Self::Sky => "Sky",
            Self::Violet => "Violet",
            Self::Charcoal => "Charcoal",
            Self::Cream => "Cream",
        }
    }
    pub fn rgb(self) -> Option<[u8; 3]> {
        Some(match self {
            Self::Classic => return None,
            Self::Coral => [225, 91, 77],
            Self::Lagoon => [57, 173, 164],
            Self::Sunflower => [244, 191, 63],
            Self::Meadow => [95, 153, 85],
            Self::Sky => [91, 157, 209],
            Self::Violet => [160, 112, 190],
            Self::Charcoal => [83, 98, 110],
            Self::Cream => [237, 224, 184],
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideAppearance {
    pub ride: RideId,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub structure: RidePaint,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub supports: RidePaint,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub vehicles: RidePaint,
}
impl Default for RideAppearance {
    fn default() -> Self {
        Self {
            ride: RideId(0),
            structure: RidePaint::Classic,
            supports: RidePaint::Classic,
            vehicles: RidePaint::Classic,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RidePaintError {
    UnknownRide,
    NotInstalled,
}
impl std::fmt::Display for RidePaintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UnknownRide => "Choose a known ride",
            Self::NotInstalled => "The ride is no longer installed",
        })
    }
}
impl std::error::Error for RidePaintError {}
impl Park {
    pub fn ride_appearance(&self, ride: RideId) -> Option<RideAppearance> {
        self.ride_by_id(ride)?;
        Some(
            self.ride_paints
                .binary_search_by_key(&ride, |p| p.ride)
                .map_or(
                    RideAppearance {
                        ride,
                        ..Default::default()
                    },
                    |i| self.ride_paints[i],
                ),
        )
    }
    pub fn set_ride_appearance(&mut self, value: RideAppearance) -> Result<(), RidePaintError> {
        self.ride_by_id(value.ride)
            .ok_or(RidePaintError::UnknownRide)?;
        if self
            .carousel(value.ride)
            .is_some_and(|c| c.objects.is_empty())
            || (self.carousel(value.ride).is_none()
                && !self
                    .objects
                    .iter()
                    .any(|o| o.kind == crate::ObjectKind::Track))
        {
            return Err(RidePaintError::NotInstalled);
        }
        let classic = value
            == RideAppearance {
                ride: value.ride,
                ..Default::default()
            };
        match self
            .ride_paints
            .binary_search_by_key(&value.ride, |p| p.ride)
        {
            Ok(i) => {
                if classic {
                    self.ride_paints.remove(i);
                } else {
                    self.ride_paints[i] = value;
                }
            }
            Err(i) => {
                if !classic {
                    self.ride_paints.insert(i, value);
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn validate(park: &Park) -> Result<(), String> {
    if park.ride_paints.len() > MAX_RIDE_RECORDS {
        return Err("Too many ride appearances".into());
    }
    let mut previous = None;
    for row in &park.ride_paints {
        if previous.is_some_and(|id| id >= row.ride)
            || park.ride_by_id(row.ride).is_none()
            || *row
                == (RideAppearance {
                    ride: row.ride,
                    ..Default::default()
                })
        {
            return Err("Ride appearance owner, order or canonical defaults are invalid".into());
        }
        previous = Some(row.ride);
    }
    Ok(())
}
#[cfg(test)]
mod tests;
