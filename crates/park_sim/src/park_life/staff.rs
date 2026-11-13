use super::*;
use crate::{FinanceKind, PATH_UNITS, PathDirection, ServiceVisitPhase, WALK_TICKS};

impl Park {
    pub fn hire_park_staff(&mut self, role: StaffRole) -> Result<u64, String> {
        self.enable_park_life();
        let entry = self.visitor_flow_stats().entry;
        let life = self.park_life.as_mut().unwrap();
        if life.staff.len() == PARK_STAFF_LIMIT || life.next_staff_id == u64::MAX {
            return Err("Staff limit reached".into());
        }
        let id = life.next_staff_id;
        life.next_staff_id += 1;
        life.staff.push(StaffMember {
            id,
            role,
            working: true,
            current: entry,
            next: None,
            progress: 0,
            destination: None,
            task: StaffTask::Patrol,
            task_ticks: 0,
            wage_ticks: 0,
            wages_paid_cents: 0,
            completed: 0,
            patrol: Vec::new(),
        });
        Ok(id)
    }
    pub fn dismiss_park_staff(&mut self, id: u64) -> bool {
        let Some(life) = self.park_life.as_mut() else {
            return false;
        };
        let before = life.staff.len();
        life.staff.retain(|s| s.id != id);
        before != life.staff.len()
    }
    pub fn toggle_staff_patrol(&mut self, id: u64, path: ObjectId) -> Result<bool, String> {
        if self.object(path).is_none_or(|p| !p.is_public_path()) {
            return Err("Select a public path for the patrol area".into());
        }
        let worker = self
            .park_life
            .as_mut()
            .and_then(|l| l.staff.iter_mut().find(|s| s.id == id))
            .ok_or("Select a staff member")?;
        match worker.patrol.binary_search(&path) {
            Ok(i) => {
                worker.patrol.remove(i);
                Ok(false)
            }
            Err(i) => {
                if worker.patrol.len() == 64 {
                    return Err("Patrol area limit reached".into());
                }
                worker.patrol.insert(i, path);
                worker.destination = None;
                worker.task_ticks = 0;
                Ok(true)
            }
        }
    }
    pub fn clear_staff_patrol(&mut self, id: u64) {
        if let Some(s) = self
            .park_life
            .as_mut()
            .and_then(|l| l.staff.iter_mut().find(|s| s.id == id))
        {
            s.patrol.clear();
            s.destination = None;
            s.task_ticks = 0;
        }
    }
    pub(crate) fn step_park_staff(&mut self) {
        let count = self.park_staff().len();
        for index in 0..count {
            let mut worker = self.park_staff()[index].clone();
            worker.wage_ticks += 1;
            if worker.wage_ticks == PARK_STAFF_WAGE_TICKS {
                if self.cash_cents < PARK_STAFF_WAGE_CENTS
                    || !self.management_payroll(PARK_STAFF_WAGE_CENTS)
                {
                    worker.working = false;
                    worker.wage_ticks -= 1;
                } else {
                    self.record_cash(FinanceKind::StaffWage, None, PARK_STAFF_WAGE_CENTS);
                    worker.wages_paid_cents += PARK_STAFF_WAGE_CENTS;
                    self.park_life.as_mut().unwrap().wages_paid_cents += PARK_STAFF_WAGE_CENTS;
                    worker.wage_ticks = 0;
                    worker.working = true;
                }
            }
            if worker.working {
                self.move_park_worker(&mut worker);
            }
            self.park_life.as_mut().unwrap().staff[index] = worker;
        }
    }
    fn move_park_worker(&mut self, worker: &mut StaffMember) {
        if worker.current.is_none() {
            worker.current = self.visitor_flow_stats().entry;
        }
        let Some(current) = worker.current else {
            return;
        };
        if let Some(next) = worker.next {
            worker.progress += 1;
            if worker.progress < WALK_TICKS {
                return;
            }
            worker.current = Some(next);
            worker.next = None;
            worker.progress = 0;
        }
        let current = worker.current.unwrap_or(current);
        let mut jobs = self.worker_jobs(worker);
        jobs.retain(|(path, task)| {
            worker.patrol.is_empty() || worker.patrol.contains(path) || *task == StaffTask::Patrol
        });
        let choice = jobs
            .into_iter()
            .filter_map(|(path, task)| {
                if self.park_staff().iter().any(|s| {
                    s.id != worker.id && s.working && s.task == task && s.destination == Some(path)
                }) {
                    return None;
                }
                let distance = self.public_path_route(current, path)?.len();
                Some((distance, path, task))
            })
            .min_by_key(|(distance, path, _)| (*distance, *path));
        let (destination, task) = if let Some((_, path, task)) = choice {
            (path, task)
        } else {
            let candidates: Vec<_> = self
                .objects
                .iter()
                .filter(|p| {
                    p.is_public_path()
                        && (worker.patrol.is_empty() || worker.patrol.contains(&p.id))
                })
                .map(|p| p.id)
                .collect();
            if candidates.is_empty() {
                worker.destination = None;
                return;
            }
            let path = candidates[((self.ticks / 180 + worker.id) as usize) % candidates.len()];
            (path, StaffTask::Patrol)
        };
        if worker.destination != Some(destination) || worker.task != task {
            worker.task_ticks = 0;
        }
        worker.destination = Some(destination);
        worker.task = task;
        if current == destination {
            worker.task_ticks += 1;
            let work_ticks = if worker.role == StaffRole::Attendant {
                30
            } else {
                120
            };
            if worker.task_ticks >= work_ticks {
                worker.task_ticks = 0;
                if self.complete_worker_task(worker, current) {
                    worker.completed = worker.completed.saturating_add(1);
                }
            }
        } else if let Some(mut route) = self.public_path_route(current, destination) {
            worker.next = route.pop_front();
            worker.progress = 0;
        } else {
            worker.destination = None;
            worker.task_ticks = 0;
        }
    }
    fn worker_jobs(&self, worker: &StaffMember) -> Vec<(ObjectId, StaffTask)> {
        match worker.role {
            StaffRole::Mechanic => self
                .all_rides()
                .filter(|r| {
                    self.can_service_ride(r.id)
                        && self
                            .ride_maintenance_status(r.id)
                            .is_some_and(|m| m.inspection_due)
                })
                .filter_map(|r| Some((self.ride_exit_access(r.id).public_exit?, StaffTask::Repair)))
                .collect(),
            StaffRole::Handyman => self
                .grounds
                .messes
                .iter()
                .map(|m| (m.path, StaffTask::Clean))
                .chain(
                    self.grounds
                        .bins
                        .iter()
                        .filter(|b| b.fill > 0)
                        .map(|b| (b.path, StaffTask::EmptyBin)),
                )
                .collect(),
            StaffRole::Security => self
                .park_incidents()
                .iter()
                .map(|i| (i.path, StaffTask::ResolveIncident))
                .collect(),
            StaffRole::Entertainer => self
                .guests
                .iter()
                .filter(|g| {
                    g.satisfaction < 85 && g.ride_visit.is_none() && !self.visitors.leaving(g.id)
                })
                .filter_map(|g| Some((g.current?, StaffTask::Entertain)))
                .collect(),
            StaffRole::Attendant => {
                // A stable counter lets staff help during a normal three-second
                // service visit instead of continually chasing moving guests.
                let counters: Vec<_> = self
                    .services
                    .iter()
                    .filter(|s| s.object.is_some() && s.open)
                    .filter_map(|s| {
                        Some((self.service_access(s.id).public_path?, StaffTask::Assist))
                    })
                    .collect();
                if !counters.is_empty() {
                    return counters;
                }
                self.guests
                    .iter()
                    .filter(|g| g.needs.nausea >= 40 || g.needs.energy <= 30)
                    .filter_map(|g| Some((g.current?, StaffTask::Assist)))
                    .collect()
            }
        }
    }
    fn complete_worker_task(&mut self, worker: &StaffMember, path: ObjectId) -> bool {
        match worker.task {
            StaffTask::Repair => {
                let id = self
                    .all_rides()
                    .filter(|r| {
                        self.can_service_ride(r.id)
                            && self
                                .ride_maintenance_status(r.id)
                                .is_some_and(|m| m.inspection_due)
                    })
                    .find(|r| self.ride_exit_access(r.id).public_exit == Some(path))
                    .map(|r| r.id);
                id.is_some_and(|id| self.service_ride(id).is_ok())
            }
            StaffTask::Clean => {
                if let Some(i) = self.grounds.messes.iter().position(|m| m.path == path) {
                    self.grounds.messes.remove(i);
                    self.grounds.cleaned = self.grounds.cleaned.saturating_add(1);
                    true
                } else {
                    false
                }
            }
            StaffTask::EmptyBin => {
                if let Some(b) = self
                    .grounds
                    .bins
                    .iter_mut()
                    .find(|b| b.path == path && b.fill > 0)
                {
                    b.fill = 0;
                    self.grounds.bins_emptied = self.grounds.bins_emptied.saturating_add(1);
                    true
                } else {
                    false
                }
            }
            StaffTask::ResolveIncident => {
                let l = self.park_life.as_mut().unwrap();
                let before = l.incidents.len();
                l.incidents.retain(|i| i.path != path);
                l.resolved = l
                    .resolved
                    .saturating_add((before - l.incidents.len()) as u64);
                before != l.incidents.len()
            }
            StaffTask::Entertain => {
                let mut helped = false;
                for g in &mut self.guests {
                    if g.current == Some(path) && g.satisfaction < 85 {
                        g.satisfaction = (g.satisfaction + 4).min(100);
                        helped = true;
                    }
                }
                helped
            }
            StaffTask::Assist => {
                let mut helped = false;
                for g in &mut self.guests {
                    if g.current == Some(path) {
                        if g.needs.nausea >= 40 || g.needs.energy <= 30 {
                            g.needs.nausea = g.needs.nausea.saturating_sub(5);
                            g.needs.energy = (g.needs.energy + 5).min(100);
                            helped = true;
                        }
                        if let Some(v) = &mut g.service_visit {
                            if v.phase == ServiceVisitPhase::Using {
                                v.progress = (v.progress + 15).min(crate::SERVICE_USE_TICKS - 1);
                                helped = true;
                            } else if v.phase == ServiceVisitPhase::Waiting {
                                v.waited_ticks = v.waited_ticks.saturating_sub(30);
                                helped = true;
                            }
                        }
                    }
                }
                helped
            }
            StaffTask::Patrol => false,
        }
    }
}

pub(super) fn position(park: &Park, worker: &StaffMember) -> Option<PathPoint> {
    let current = park.object(worker.current?)?;
    let from = current.path_center()?;
    let Some(next) = worker.next.and_then(|id| park.object(id)) else {
        return Some(from);
    };
    let to = next.path_center()?;
    let edge = PathDirection::ALL
        .into_iter()
        .find(|e| e.neighbor(current.pos) == Some(next.pos))?;
    let middle = PathPoint {
        x: (from.x + to.x) / 2,
        y: (from.y + to.y) / 2,
        height: i32::from(current.path_edge_height(edge)?) * PATH_UNITS,
    };
    let half = WALK_TICKS / 2;
    let (a, b, t) = if worker.progress <= half {
        (from, middle, worker.progress)
    } else {
        (middle, to, worker.progress - half)
    };
    let mix = |a, b| a + (b - a) * i32::from(t) / i32::from(half);
    Some(PathPoint {
        x: mix(a.x, b.x),
        y: mix(a.y, b.y),
        height: mix(a.height, b.height),
    })
}
