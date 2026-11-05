//! Deterministic ride wear and deliberate stopped-ride service.
use crate::{Park, RideId};

pub const MAINTENANCE_DUE_VISITS: u64 = 120;
pub const MAINTENANCE_SHUTDOWN_VISITS: u64 = 1_200;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RideMaintenanceRecord {
    pub(crate) ride: RideId,
    pub(crate) serviced_completed: u64,
    pub(crate) service_count: u32,
    pub(crate) last_service_tick: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideMaintenanceStatus {
    pub visits_since_service: u64,
    pub service_count: u32,
    pub last_service_tick: u64,
    pub inspection_due: bool,
    pub shutdown: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RideMaintenanceError {
    UnknownRide,
    RideOpenOrMoving,
}

impl std::fmt::Display for RideMaintenanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRide => f.write_str("Choose an installed ride"),
            Self::RideOpenOrMoving => f.write_str("Close and empty the ride before servicing"),
        }
    }
}

impl Park {
    pub fn can_service_ride(&self, id: RideId) -> bool {
        self.ride_operating_settings_status(id).is_ok()
    }
    pub fn ride_maintenance_status(&self, id: RideId) -> Option<RideMaintenanceStatus> {
        let ride = self.ride_by_id(id)?;
        let record = self.maintenance.iter().find(|r| r.ride == id);
        let visits = ride
            .completed
            .saturating_sub(record.map_or(ride.completed, |r| r.serviced_completed));
        Some(RideMaintenanceStatus {
            visits_since_service: visits,
            service_count: record.map_or(0, |r| r.service_count),
            last_service_tick: record.map_or(0, |r| r.last_service_tick),
            inspection_due: visits >= MAINTENANCE_DUE_VISITS,
            shutdown: visits >= MAINTENANCE_SHUTDOWN_VISITS,
        })
    }

    pub fn service_ride(&mut self, id: RideId) -> Result<(), RideMaintenanceError> {
        let completed = self
            .ride_by_id(id)
            .ok_or(RideMaintenanceError::UnknownRide)?
            .completed;
        if !self.can_service_ride(id) {
            return Err(RideMaintenanceError::RideOpenOrMoving);
        }
        self.ensure_maintenance_record(id, completed);
        let record = self.maintenance.iter_mut().find(|r| r.ride == id).unwrap();
        record.serviced_completed = completed;
        record.service_count = record.service_count.saturating_add(1);
        record.last_service_tick = self.ticks;
        Ok(())
    }

    pub(crate) fn step_maintenance(&mut self) {
        let ids: Vec<_> = self
            .all_rides()
            .filter(|r| self.ride_available(r.id))
            .map(|r| (r.id, r.completed))
            .collect();
        for (id, completed) in ids {
            self.ensure_maintenance_record(id, completed);
            if self
                .ride_maintenance_status(id)
                .is_some_and(|status| status.shutdown)
                && self.ride_is_open(id)
            {
                // Closing allows the current trip to finish and unload.
                self.set_ride_open(id, false)
                    .expect("installed ride can close");
            }
        }
    }

    fn ensure_maintenance_record(&mut self, id: RideId, completed: u64) {
        let index = self.maintenance.binary_search_by_key(&id, |r| r.ride);
        if let Err(index) = index {
            self.maintenance.insert(
                index,
                RideMaintenanceRecord {
                    ride: id,
                    serviced_completed: completed,
                    service_count: 0,
                    last_service_tick: self.ticks,
                },
            );
        }
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    if park.maintenance.len() > crate::MAX_RIDE_RECORDS {
        return Err("too many maintenance records".into());
    }
    let mut previous = None;
    for record in &park.maintenance {
        let ride = park
            .ride_by_id(record.ride)
            .ok_or("maintenance has unknown ride")?;
        if previous.is_some_and(|id| id >= record.ride)
            || record.serviced_completed > ride.completed
            || record.last_service_tick > park.ticks
            || park
                .carousel(record.ride)
                .is_some_and(|carousel| carousel.objects.is_empty())
        {
            return Err("maintenance state is inconsistent".into());
        }
        previous = Some(record.ride);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Speed;

    #[test]
    fn shutdown_requires_stopped_service_and_continues_across_save() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let id = park.ride().unwrap().id;
        park.advance(Speed::Normal);
        let ordinary = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(ordinary, park);
        let base = park.ride().unwrap().completed;
        park.ride.as_mut().unwrap().completed = base + MAINTENANCE_SHUTDOWN_VISITS;
        park.advance(Speed::Normal);
        assert!(park.ride_maintenance_status(id).unwrap().shutdown);
        assert!(!park.ride_is_open(id));
        assert!(park.ride_opening_status(id).is_err());
        assert_eq!(park.service_ride(id), Ok(()));
        assert!(!park.ride_maintenance_status(id).unwrap().inspection_due);
        assert_eq!(park.ride_maintenance_status(id).unwrap().service_count, 1);
    }

    #[test]
    fn schema_migration_and_record_validation_are_strict() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        park.advance(Speed::Normal);
        let current: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        let mut missing = current.clone();
        missing["park"]
            .as_object_mut()
            .unwrap()
            .remove("maintenance");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&missing).unwrap()).is_err());
        let mut old = missing.clone();
        old["park"]
            .as_object_mut()
            .unwrap()
            .remove("mechanic_route");
        old["park"].as_object_mut().unwrap().remove("grounds");
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
        old["park"]["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove("handyman_wages_cents");
        old["schema_version"] = 25.into();
        old["content_version"] = 25.into();
        crate::legacy_cost_test_support::strip_park_open(&mut old);
        let migrated = Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).unwrap();
        assert!(migrated.maintenance.is_empty());
        assert_eq!(
            Park::from_save_bytes(&migrated.to_save_bytes().unwrap()).unwrap(),
            migrated
        );
        let mut smuggled = current.clone();
        smuggled["schema_version"] = 25.into();
        smuggled["content_version"] = 25.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&smuggled).unwrap()).is_err());
        let mut forged = current;
        forged["park"]["maintenance"][0]["serviced_completed"] = u64::MAX.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
    }
}
