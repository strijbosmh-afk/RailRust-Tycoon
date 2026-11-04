//! One park-wide mechanic, paid from cash before ride operating costs.
use crate::{FinanceKind, Park};

pub const MECHANIC_WAGE_TICKS: u16 = 900;
pub const MECHANIC_WAGE_CENTS: u64 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MechanicStatus {
    pub hired: bool,
    pub wage_progress: u16,
    pub wages_paid_cents: u64,
}

impl Park {
    pub fn mechanic_status(&self) -> MechanicStatus {
        MechanicStatus {
            hired: self.mechanic_hired,
            wage_progress: self.mechanic_wage_progress,
            wages_paid_cents: self.finances.archived.wages_cents
                + self
                    .finances
                    .entries
                    .iter()
                    .filter(|e| e.kind == FinanceKind::MechanicWage)
                    .map(|e| e.amount_cents)
                    .sum::<u64>(),
        }
    }

    pub fn set_mechanic_hired(&mut self, hired: bool) -> bool {
        if self.mechanic_hired == hired {
            return false;
        }
        self.mechanic_hired = hired;
        self.mechanic_wage_progress = 0;
        true
    }

    pub(crate) fn step_mechanic_wages(&mut self) {
        if !self.mechanic_hired {
            return;
        }
        if self.mechanic_wage_progress + 1 < MECHANIC_WAGE_TICKS {
            self.mechanic_wage_progress += 1;
        } else if self.cash_cents < MECHANIC_WAGE_CENTS {
            self.mechanic_hired = false;
            self.mechanic_wage_progress = 0;
        } else {
            self.mechanic_wage_progress = 0;
            self.record_cash(FinanceKind::MechanicWage, None, MECHANIC_WAGE_CENTS);
        }
    }

    pub(crate) fn step_mechanic_service(&mut self) {
        if !self.mechanic_hired {
            return;
        }
        let due: Vec<_> = self
            .all_rides()
            .filter_map(|ride| {
                let status = self.ride_maintenance_status(ride.id)?;
                (status.inspection_due && self.can_service_ride(ride.id)).then_some(ride.id)
            })
            .collect();
        for id in due {
            self.service_ride(id).expect("checked stopped ride");
        }
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    if park.mechanic_wage_progress >= MECHANIC_WAGE_TICKS
        || (!park.mechanic_hired && park.mechanic_wage_progress != 0)
    {
        return Err("mechanic wage clock is invalid".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RideMaintenanceStatus, Speed};

    #[test]
    fn wage_charge_is_exact_and_survives_archive_and_restart() {
        let mut park = Park::new(32, 32);
        assert!(park.set_mechanic_hired(true));
        for _ in 0..899 {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.mechanic_status().wage_progress, 899);
        assert_eq!(park.cash_cents(), crate::INITIAL_PARK_CASH_CENTS);
        park.advance(Speed::Normal);
        assert_eq!(park.mechanic_status().wage_progress, 0);
        assert_eq!(park.mechanic_status().wages_paid_cents, MECHANIC_WAGE_CENTS);
        assert_eq!(
            park.cash_cents(),
            crate::INITIAL_PARK_CASH_CENTS - MECHANIC_WAGE_CENTS
        );
        assert_eq!(park.finance_stats().wages_cents, MECHANIC_WAGE_CENTS);
        assert_eq!(
            park.finance_summary().operating_profit_cents,
            -i128::from(MECHANIC_WAGE_CENTS)
        );
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
        for _ in 0..(130_u32 * u32::from(MECHANIC_WAGE_TICKS)) {
            park.advance(Speed::Normal);
        }
        assert!(park.finance_ledger().archived.wages_cents > 0);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn insufficient_cash_dismisses_mechanic_without_negative_money() {
        let mut park = Park::new(32, 32);
        park.cash_cents = MECHANIC_WAGE_CENTS - 1;
        park.set_mechanic_hired(true);
        for _ in 0..MECHANIC_WAGE_TICKS {
            park.advance(Speed::Normal);
        }
        assert!(!park.mechanic_status().hired);
        assert_eq!(park.mechanic_status().wage_progress, 0);
        assert_eq!(park.cash_cents(), MECHANIC_WAGE_CENTS - 1);
        assert_eq!(park.mechanic_status().wages_paid_cents, 0);
    }

    #[test]
    fn mechanic_services_due_closed_ride_and_schema_26_migrates() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        park.advance(Speed::Normal);
        let id = park.ride().unwrap().id;
        let current: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        let mut old = current.clone();
        old["schema_version"] = 26.into();
        old["content_version"] = 26.into();
        let body = old["park"].as_object_mut().unwrap();
        body.remove("mechanic_hired");
        body.remove("mechanic_wage_progress");
        body["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove("wages_cents");
        let migrated = Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).unwrap();
        assert_eq!(migrated, park);
        let mut smuggled = current.clone();
        smuggled["schema_version"] = 26.into();
        smuggled["content_version"] = 26.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&smuggled).unwrap()).is_err());
        let mut missing = current;
        missing["park"]
            .as_object_mut()
            .unwrap()
            .remove("mechanic_hired");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&missing).unwrap()).is_err());

        park.set_train_open(false);
        park.set_mechanic_hired(true);
        park.ride.as_mut().unwrap().completed += crate::MAINTENANCE_DUE_VISITS;
        park.advance(Speed::Normal);
        assert_eq!(
            park.ride_maintenance_status(id)
                .map(|s: RideMaintenanceStatus| s.service_count),
            Some(1)
        );
        assert_eq!(
            park.ride_maintenance_status(id)
                .unwrap()
                .visits_since_service,
            0
        );
    }
}
