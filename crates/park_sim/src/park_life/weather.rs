use super::*;
use crate::{GuestThought, RideModel};

impl Park {
    pub fn ride_weather_exposed(&self, id: RideId) -> bool {
        !matches!(
            self.ride_by_id(id).and_then(|r| r.model),
            Some(RideModel::Carousel | RideModel::TeaCups)
        )
    }
    pub fn weather_blocks_ride(&self, id: RideId) -> bool {
        self.park_life
            .as_ref()
            .is_some_and(|l| l.weather == Weather::Storm)
            && self.ride_weather_exposed(id)
    }
    pub(crate) fn step_life_weather(&mut self) {
        let Some(life) = &mut self.park_life else {
            return;
        };
        life.weather_ticks = (life.weather_ticks + 1) % 1800;
        let next = match life.policy {
            WeatherPolicy::Automatic if life.weather_ticks == 0 => match life.weather {
                Weather::Sunny => Weather::Hot,
                Weather::Hot => Weather::Rain,
                Weather::Rain => Weather::Storm,
                Weather::Storm => Weather::Sunny,
            },
            WeatherPolicy::Automatic => life.weather,
            WeatherPolicy::Sunny => Weather::Sunny,
            WeatherPolicy::Hot => Weather::Hot,
            WeatherPolicy::Rain => Weather::Rain,
            WeatherPolicy::Storm => Weather::Storm,
        };
        self.change_life_weather(next);
        if !self.ticks.is_multiple_of(150) {
            return;
        }
        let cover = self.path_shelters().to_vec();
        let damaged: Vec<_> = self
            .park_incidents()
            .iter()
            .filter(|i| i.kind == IncidentKind::Vandalism)
            .map(|i| i.path)
            .collect();
        for guest in &mut self.guests {
            if guest
                .ride_visit
                .as_ref()
                .is_some_and(|v| v.passenger.is_some())
            {
                continue;
            }
            let sheltered = guest
                .current
                .is_some_and(|p| cover.contains(&p) && !damaged.contains(&p));
            match next {
                Weather::Hot => guest.needs.thirst = (guest.needs.thirst + 3).min(100),
                Weather::Rain | Weather::Storm if !sheltered => {
                    guest.satisfaction = guest.satisfaction.saturating_sub(1);
                    guest.think(GuestThought::NeedsRest, self.ticks);
                }
                Weather::Rain | Weather::Storm => {
                    guest.needs.energy = (guest.needs.energy + 2).min(100);
                }
                _ => {}
            }
            if guest.current.is_some_and(|p| damaged.contains(&p)) {
                guest.satisfaction = guest.satisfaction.saturating_sub(2);
            }
        }
        self.report_natural_incidents();
    }
    pub(super) fn change_life_weather(&mut self, next: Weather) {
        let life = self.park_life.as_mut().expect("park life enabled");
        let previous = life.weather;
        life.weather = next;
        if next == Weather::Storm {
            let ids: Vec<_> = self
                .all_rides()
                .filter(|r| self.ride_is_open(r.id) && self.ride_weather_exposed(r.id))
                .map(|r| r.id)
                .collect();
            for id in ids {
                if self.set_ride_open(id, false).is_ok()
                    && !self
                        .park_life
                        .as_ref()
                        .unwrap()
                        .weather_closed
                        .contains(&id)
                {
                    self.park_life.as_mut().unwrap().weather_closed.push(id);
                }
            }
        } else if previous == Weather::Storm {
            let ids = std::mem::take(&mut self.park_life.as_mut().unwrap().weather_closed);
            for id in ids {
                let _ = self.set_ride_open_checked(id, true);
            }
        }
    }
    fn report_natural_incidents(&mut self) {
        if !self.ticks.is_multiple_of(600) || self.park_incidents().len() >= INCIDENT_LIMIT {
            return;
        }
        let candidate = self.guests.iter().find_map(|g| {
            let path = g.current?;
            if self.object(path).is_none_or(|p| !p.is_public_path())
                || self.park_incidents().iter().any(|i| i.path == path)
            {
                return None;
            }
            if g.needs.nausea >= 65 || g.needs.energy <= 15 {
                Some((path, IncidentKind::Distress))
            } else if g.satisfaction < 35
                && g.id.0 % 3 == self.ticks / 600 % 3
                && (self.path_benches().iter().any(|b| b.path == path)
                    || self.ground_bins().iter().any(|b| b.path == path))
            {
                Some((path, IncidentKind::Vandalism))
            } else {
                None
            }
        });
        if let Some((path, kind)) = candidate {
            let life = self.park_life.as_mut().unwrap();
            if life.next_incident_id == u64::MAX {
                return;
            }
            life.incidents.push(ParkIncident {
                id: life.next_incident_id,
                path,
                kind,
                tick: self.ticks,
            });
            life.next_incident_id += 1;
        }
    }
}
