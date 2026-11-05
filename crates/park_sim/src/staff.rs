//! One park-wide mechanic, paid from cash before ride operating costs.
use crate::{
    FinanceKind, ObjectId, PATH_UNITS, Park, PathDirection, PathPoint, RideId, WALK_TICKS,
};

pub const MECHANIC_WAGE_TICKS: u16 = 900;
pub const MECHANIC_WAGE_CENTS: u64 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MechanicStatus {
    pub hired: bool,
    pub wage_progress: u16,
    pub wages_paid_cents: u64,
    pub current_path: Option<ObjectId>,
    pub next_path: Option<ObjectId>,
    pub walk_progress: u16,
    pub target_ride: Option<RideId>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MechanicRoute {
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub current: Option<ObjectId>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub next: Option<ObjectId>,
    pub progress: u16,
    #[serde(deserialize_with = "crate::saves::required_option")]
    pub target: Option<RideId>,
}

impl Park {
    pub fn mechanic_status(&self) -> MechanicStatus {
        MechanicStatus {
            hired: self.mechanic_hired,
            wage_progress: self.mechanic_wage_progress,
            current_path: self.mechanic_route.current,
            next_path: self.mechanic_route.next,
            walk_progress: self.mechanic_route.progress,
            target_ride: self.mechanic_route.target,
            wages_paid_cents: self
                .finances
                .archived
                .wages_cents
                .saturating_sub(self.finances.archived.handyman_wages_cents)
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
        self.mechanic_route = MechanicRoute::default();
        true
    }

    pub(crate) fn step_mechanic_wages(&mut self) {
        if !self.mechanic_hired {
            return;
        }
        if self.mechanic_wage_progress + 1 < MECHANIC_WAGE_TICKS {
            self.mechanic_wage_progress += 1;
        } else if self.cash_cents < MECHANIC_WAGE_CENTS {
            self.set_mechanic_hired(false);
        } else {
            self.mechanic_wage_progress = 0;
            self.record_cash(FinanceKind::MechanicWage, None, MECHANIC_WAGE_CENTS);
        }
    }

    pub(crate) fn step_mechanic_service(&mut self) {
        if !self.mechanic_hired {
            return;
        }
        let entry = self
            .visitors
            .entry
            .filter(|id| self.object(*id).is_some_and(|path| path.is_public_path()));
        if self
            .mechanic_route
            .current
            .is_none_or(|id| self.object(id).is_none_or(|path| !path.is_public_path()))
        {
            self.mechanic_route = MechanicRoute {
                current: entry,
                ..MechanicRoute::default()
            };
        }
        let Some(current) = self.mechanic_route.current else {
            return;
        };
        if let Some(next) = self.mechanic_route.next {
            if !self.path_neighbors(current).contains(&next) {
                self.mechanic_route.next = None;
                self.mechanic_route.progress = 0;
                self.mechanic_route.target = None;
            } else {
                self.mechanic_route.progress += 1;
                if self.mechanic_route.progress < WALK_TICKS {
                    return;
                }
                self.mechanic_route.current = Some(next);
                self.mechanic_route.next = None;
                self.mechanic_route.progress = 0;
            }
        }
        let Some(entry) = entry else {
            self.mechanic_route.target = None;
            return;
        };
        let current = self.mechanic_route.current.expect("retained path");
        let target = self
            .mechanic_route
            .target
            .filter(|id| self.service_landing(*id, entry, current).is_some());
        let target = target.or_else(|| {
            let mut ids: Vec<_> = self.all_rides().map(|ride| ride.id).collect();
            ids.sort_unstable();
            ids.into_iter()
                .find(|id| self.service_landing(*id, entry, current).is_some())
        });
        self.mechanic_route.target = target;
        let destination = target
            .and_then(|id| self.ride_exit_access(id).public_exit)
            .or(Some(entry));
        let Some(destination) = destination else {
            return;
        };
        if current == destination {
            if let Some(id) = target {
                self.service_ride(id).expect("eligible stopped ride");
                self.mechanic_route.target = None;
            }
            return;
        }
        if let Some(mut path) = self.public_path_route(current, destination) {
            self.mechanic_route.next = path.pop_front();
        } else {
            self.mechanic_route.target = None;
        }
    }

    fn service_landing(&self, id: RideId, entry: ObjectId, current: ObjectId) -> Option<ObjectId> {
        if !self.can_service_ride(id) || !self.ride_maintenance_status(id)?.inspection_due {
            return None;
        }
        let landing = self.ride_exit_access(id).public_exit?;
        (self.public_path_route(entry, landing).is_some()
            && self.public_path_route(current, landing).is_some())
        .then_some(landing)
    }

    pub(crate) fn reconcile_mechanic_after_removal(&mut self, removed: ObjectId) {
        if self.mechanic_route.current == Some(removed) {
            self.mechanic_route = MechanicRoute::default();
        } else if self.mechanic_route.next == Some(removed) {
            self.mechanic_route.next = None;
            self.mechanic_route.progress = 0;
            self.mechanic_route.target = None;
        }
        if self
            .mechanic_route
            .target
            .is_some_and(|id| self.ride_exit_access(id).public_exit.is_none())
        {
            self.mechanic_route.target = None;
        }
    }

    pub fn mechanic_position(&self) -> Option<PathPoint> {
        let route = self.mechanic_route;
        let current = self.object(route.current?)?;
        let from = current.path_center()?;
        let next = route.next.and_then(|id| self.object(id));
        let Some(next) = next else {
            return Some(from);
        };
        let to = next.path_center()?;
        let edge = PathDirection::ALL
            .into_iter()
            .find(|edge| edge.neighbor(current.pos) == Some(next.pos))?;
        let middle = PathPoint {
            x: (from.x + to.x) / 2,
            y: (from.y + to.y) / 2,
            height: i32::from(current.path_edge_height(edge)?) * PATH_UNITS,
        };
        let half = WALK_TICKS / 2;
        let (a, b, ticks) = if route.progress <= half {
            (from, middle, route.progress)
        } else {
            (middle, to, route.progress - half)
        };
        let mix = |a: i32, b: i32| a + (b - a) * i32::from(ticks) / i32::from(half);
        Some(PathPoint {
            x: mix(a.x, b.x),
            y: mix(a.y, b.y),
            height: mix(a.height, b.height),
        })
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    if park.mechanic_wage_progress >= MECHANIC_WAGE_TICKS
        || (!park.mechanic_hired && park.mechanic_wage_progress != 0)
    {
        return Err("mechanic wage clock is invalid".into());
    }
    let route = park.mechanic_route;
    if (!park.mechanic_hired && route != MechanicRoute::default())
        || route.progress >= WALK_TICKS
        || (route.next.is_none() && route.progress != 0)
        || route.current.is_none() && (route.next.is_some() || route.target.is_some())
        || route
            .current
            .is_some_and(|id| park.object(id).is_none_or(|path| !path.is_public_path()))
        || route
            .next
            .is_some_and(|id| park.object(id).is_none_or(|path| !path.is_public_path()))
        || route
            .next
            .is_some_and(|id| !park.path_neighbors(route.current.unwrap()).contains(&id))
        || route.target.is_some_and(|id| park.ride_by_id(id).is_none())
    {
        return Err("mechanic route is invalid".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PathDirection, RideMaintenanceStatus, Speed, TilePos};

    fn routed_ride() -> (Park, RideId, ObjectId) {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let id = park.ride().unwrap().id;
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, id)
            .unwrap();
        for x in 8..=13 {
            park.place_path(TilePos { x, y: 8 }, 8).unwrap();
        }
        let mut entry = None;
        for y in 9..=11 {
            entry = Some(park.place_path(TilePos { x: 13, y }, 8).unwrap());
        }
        let entry = entry.unwrap();
        park.set_visitor_entry(entry).unwrap();
        (park, id, entry)
    }

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
        let (mut park, id, _entry) = routed_ride();
        park.advance(Speed::Normal);
        let current: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        let mut prior = current.clone();
        prior["schema_version"] = 27.into();
        prior["content_version"] = 27.into();
        crate::legacy_cost_test_support::strip_park_open(&mut prior);
        prior["park"]
            .as_object_mut()
            .unwrap()
            .remove("mechanic_route");
        prior["park"].as_object_mut().unwrap().remove("grounds");
        prior["park"]["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove("handyman_wages_cents");
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&prior).unwrap()).unwrap(),
            park
        );
        prior["park"]["mechanic_route"] = serde_json::json!({
            "current": null, "next": null, "progress": 0, "target": null
        });
        assert!(Park::from_save_bytes(&serde_json::to_vec(&prior).unwrap()).is_err());
        let mut old = current.clone();
        old["schema_version"] = 26.into();
        old["content_version"] = 26.into();
        crate::legacy_cost_test_support::strip_park_open(&mut old);
        let body = old["park"].as_object_mut().unwrap();
        body.remove("mechanic_route");
        body.remove("grounds");
        prior["park"].as_object_mut().unwrap().remove("grounds");
        body.remove("mechanic_hired");
        body.remove("mechanic_wage_progress");
        body["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove("wages_cents");
        body["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove("handyman_wages_cents");
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
        assert_eq!(park.ride_maintenance_status(id).unwrap().service_count, 0);
        assert_eq!(park.mechanic_status().target_ride, Some(id));
        assert!(park.mechanic_status().next_path.is_some());
        for _ in 0..900 {
            if park.ride_maintenance_status(id).unwrap().service_count == 1 {
                break;
            }
            park.advance(Speed::Normal);
        }
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

    #[test]
    fn mechanic_needs_a_public_route_and_recovers_after_path_edits() {
        let (mut park, id, entry) = routed_ride();
        park.advance(Speed::Normal);
        park.set_train_open(false);
        park.ride.as_mut().unwrap().completed += crate::MAINTENANCE_DUE_VISITS;
        park.set_mechanic_hired(true);
        let cut = park
            .objects()
            .iter()
            .find(|path| path.is_public_path() && path.pos == TilePos { x: 11, y: 8 })
            .unwrap()
            .id;
        park.remove_object(cut).unwrap();
        for _ in 0..300 {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.ride_maintenance_status(id).unwrap().service_count, 0);
        assert_eq!(park.mechanic_status().current_path, Some(entry));
        park.place_path(TilePos { x: 11, y: 8 }, 8).unwrap();
        let landing = park.ride_exit_access(id).public_exit.unwrap();
        assert!(park.can_service_ride(id));
        assert!(
            park.public_path_route(entry, landing).is_some(),
            "entry {:?}, neighbors {:?}, exit {:?}, paths {:?}",
            entry,
            park.path_neighbors(entry),
            landing,
            park.objects()
                .iter()
                .filter(|path| path.is_public_path())
                .map(|p| (p.id, p.pos, p.height))
                .collect::<Vec<_>>()
        );
        for _ in 0..900 {
            if park.ride_maintenance_status(id).unwrap().service_count == 1 {
                break;
            }
            park.advance(Speed::Normal);
        }
        assert_eq!(
            park.ride_maintenance_status(id).unwrap().service_count,
            1,
            "mechanic {:?}; exit {:?}",
            park.mechanic_status(),
            park.ride_exit_access(id)
        );
    }

    #[test]
    fn saved_mid_edge_mechanic_replays_exact_position_and_rejects_forgery() {
        let (mut park, _id, entry) = routed_ride();
        park.set_mechanic_hired(true);
        park.advance(Speed::Normal);
        let next = park.path_neighbors(entry)[0];
        park.mechanic_route.next = Some(next);
        park.mechanic_route.progress = 7;
        let bytes = park.to_save_bytes().unwrap();
        let mut loaded = Park::from_save_bytes(&bytes).unwrap();
        assert_eq!(loaded, park);
        assert_eq!(loaded.mechanic_position(), park.mechanic_position());
        let mut removed = loaded.clone();
        removed.remove_object(next).unwrap();
        assert_eq!(removed.mechanic_status().next_path, None);
        assert_eq!(removed.mechanic_status().walk_progress, 0);
        assert_eq!(
            Park::from_save_bytes(&removed.to_save_bytes().unwrap()).unwrap(),
            removed
        );
        for _ in 0..90 {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
        }
        assert_eq!(loaded, park);
        let mut forged: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        forged["park"]["mechanic_route"]["next"] = 999_999.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
    }
}
