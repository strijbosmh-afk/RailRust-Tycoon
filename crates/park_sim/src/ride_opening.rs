//! Guest-facing opening checks, derived from the current installed ride and access.
//! No stored certificate: construction changes are checked again on every open.
use crate::{
    Park, RideAccessError, RideAccessStatus, RideExitStatus, RideId, RideKind, TrainError,
    TrainPhase,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RideOpeningError {
    UnknownRide(RideId),
    Unavailable,
    Circuit(TrainError),
    Entrance(RideAccessStatus),
    Exit(RideExitStatus),
    Command(RideAccessError),
}

impl std::fmt::Display for RideOpeningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRide(id) => write!(f, "Ride #{} does not exist", id.0),
            Self::Unavailable => f.write_str("Restore the installed ride first"),
            Self::Circuit(_) => f.write_str("Repair the installed coaster track"),
            Self::Entrance(status) => status.fmt(f),
            Self::Exit(RideExitStatus::PublicDisconnected) => {
                f.write_str("Join exit path to the queue network")
            }
            Self::Exit(status) => status.fmt(f),
            Self::Command(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for RideOpeningError {}

impl Park {
    /// Read-only readiness for admitting guests. Mechanics testing is separate
    /// and can run before paths are built. Installed construction/save validation
    /// owns geometry/clearance; this also rechecks its current occupied envelopes.
    pub fn ride_opening_status(&self, id: RideId) -> Result<(), RideOpeningError> {
        let kind = self
            .ride_kind(id)
            .ok_or(RideOpeningError::UnknownRide(id))?;
        if !self.ride_available(id) {
            return Err(RideOpeningError::Unavailable);
        }
        if kind == RideKind::Coaster {
            self.validate_train_circuit()
                .map_err(RideOpeningError::Circuit)?;
            if self
                .train()
                .is_none_or(|train| train.phase == TrainPhase::Fault)
            {
                return Err(RideOpeningError::Unavailable);
            }
        }
        let entrance = self.ride_access(id).status;
        if entrance != RideAccessStatus::Ready {
            return Err(RideOpeningError::Entrance(entrance));
        }
        let exit = self.ride_exit_access(id).status;
        if exit != RideExitStatus::Ready {
            return Err(RideOpeningError::Exit(exit));
        }
        Ok(())
    }

    /// Player open/close command. Refusals are atomic. Closing always remains
    /// possible after access damage, so paid trips can finish and unload normally.
    /// Existing saves keep their state; loading does not retroactively close a ride.
    pub fn set_ride_open_checked(
        &mut self,
        id: RideId,
        open: bool,
    ) -> Result<(), RideOpeningError> {
        if open {
            self.ride_opening_status(id)?;
        }
        self.set_ride_open(id, open)
            .map_err(RideOpeningError::Command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TilePos};

    #[test]
    fn readiness_and_refusals_are_atomic_and_missing_track_or_access_has_a_reason() {
        let mut park = Park::new(32, 32);
        assert_eq!(
            park.ride_opening_status(RideId(99)),
            Err(RideOpeningError::UnknownRide(RideId(99)))
        );
        park.install_train_fixture().unwrap();
        let id = park.ride().unwrap().id;
        park.set_train_open(false);
        let before = park.clone();
        assert_eq!(
            park.set_ride_open_checked(id, true),
            Err(RideOpeningError::Entrance(RideAccessStatus::NoEntrance))
        );
        assert_eq!(park, before);
        assert!(park.test_coaster().is_ok());
        assert_eq!(park, before);
        let piece = park.circuit().unwrap().pieces[1].object;
        park.remove_object(piece);
        let before = park.clone();
        assert!(matches!(
            park.set_ride_open_checked(id, true),
            Err(RideOpeningError::Unavailable | RideOpeningError::Circuit(_))
        ));
        assert_eq!(park, before);
    }

    #[test]
    fn both_families_recheck_access_and_can_close_after_damage_without_changing_other_ride() {
        let (mut park, carousel) = crate::carousel::tests::fixture(true);
        let coaster = park.ride().unwrap().id;
        for id in [coaster, carousel] {
            park.set_ride_open_checked(id, false).unwrap();
            let before = park.clone();
            assert_eq!(park.ride_opening_status(id), Ok(()));
            assert_eq!(park, before);
            park.set_ride_open_checked(id, true).unwrap();
            let other = if id == coaster { carousel } else { coaster };
            let other_open = park.ride_is_open(other);
            let exit = park.ride_by_id(id).unwrap().exit.unwrap().object;
            let mut damaged = park.clone();
            damaged.remove_object(exit);
            damaged.set_ride_open_checked(id, false).unwrap();
            let before = damaged.clone();
            assert_eq!(
                damaged.set_ride_open_checked(id, true),
                Err(RideOpeningError::Exit(RideExitStatus::NoExit))
            );
            assert_eq!(damaged, before);
            assert_eq!(damaged.ride_is_open(other), other_open);
            let bytes = damaged.to_save_bytes().unwrap();
            assert_eq!(Park::from_save_bytes(&bytes).unwrap(), damaged);
        }
        for _ in 0..7200 {
            park.advance(Speed::Normal);
        }
        assert!(park.ride_by_id(coaster).unwrap().completed > 0);
        assert!(park.ride_by_id(carousel).unwrap().completed > 0);
    }

    #[test]
    fn connected_queue_with_unusable_return_path_is_not_ready_and_repair_enables_opening() {
        let (mut park, id) = crate::carousel::tests::fixture(false);
        park.set_ride_open_checked(id, false).unwrap();
        let path = park
            .objects()
            .iter()
            .find(|o| o.is_public_path() && o.pos == TilePos { x: 20, y: 20 })
            .unwrap()
            .id;
        park.remove_object(path);
        let before = park.clone();
        assert_eq!(
            park.set_ride_open_checked(id, true),
            Err(RideOpeningError::Exit(RideExitStatus::NoPublicExit))
        );
        assert_eq!(park, before);
        park.place_path(TilePos { x: 20, y: 20 }, 8).unwrap();
        park.set_ride_open_checked(id, true).unwrap();
        assert!(park.ride_is_open(id));
    }

    #[test]
    fn disconnected_owned_queue_blocks_opening_and_current_repair_is_rechecked() {
        let (mut park, id) = crate::carousel::tests::fixture(false);
        park.set_ride_open_checked(id, false).unwrap();
        let path = *park
            .objects()
            .iter()
            .find(|o| o.queue_owner == Some(id) && o.pos == TilePos { x: 17, y: 22 })
            .unwrap();
        park.remove_object(path.id);
        let status = park.ride_access(id).status;
        assert_ne!(status, RideAccessStatus::Ready);
        let before = park.clone();
        assert_eq!(
            park.set_ride_open_checked(id, true),
            Err(RideOpeningError::Entrance(status))
        );
        assert_eq!(park, before);
        park.place_queue_path(path.pos, path.height, path.path_slope, id)
            .unwrap();
        park.set_ride_open_checked(id, true).unwrap();
        assert!(park.ride_is_open(id));
    }
}
