use super::*;
use crate::{Guest, RideCategory};
impl Park {
    pub fn guest_party(&self, id: GuestId) -> Option<(u64, PartyStyle, usize, bool)> {
        let p = self
            .park_life
            .as_ref()?
            .parties
            .iter()
            .find(|p| p.members.contains(&id))?;
        Some((p.id, p.style, p.members.len(), p.returning))
    }
    pub fn guest_life_description(&self, id: GuestId) -> String {
        let Some(g) = self.guests.iter().find(|g| g.id == id) else {
            return String::new();
        };
        let group = self.guest_party(id).map_or_else(
            || "Independent visitor".into(),
            |(id, style, size, returning)| {
                format!(
                    "{} #{id} / {size} members{}",
                    style.label(),
                    if returning { " / returning" } else { "" }
                )
            },
        );
        let shelter = if g.current.is_some_and(|p| self.path_is_sheltered(p)) {
            "Under shelter"
        } else {
            "Outdoors"
        };
        format!(
            "{group}\n{shelter} / {}",
            self.park_life_status().weather.label()
        )
    }
    pub(crate) fn begin_arrival_party(&mut self) {
        let Some(life) = &mut self.park_life else {
            return;
        };
        if life.parties.len() >= PARTY_LIMIT || life.next_party_id == u64::MAX {
            return;
        }
        let returning = life.returns.first().is_some_and(|r| r.due <= self.ticks);
        let style = if returning {
            life.returns.remove(0).style
        } else {
            match life.next_party_id % 3 {
                0 => PartyStyle::Family,
                1 => PartyStyle::Thrill,
                _ => PartyStyle::Relaxed,
            }
        };
        life.parties.push(Party {
            id: life.next_party_id,
            style,
            members: Vec::new(),
            returning,
            satisfaction_total: 0,
            departed_members: 0,
        });
        life.next_party_id += 1;
    }
    pub(crate) fn finish_arrival_party(&mut self) {
        if let Some(life) = &mut self.park_life {
            life.parties.retain(|p| !p.members.is_empty());
        }
    }
    pub(crate) fn join_arrival_party(&mut self, id: GuestId) {
        let Some(life) = &mut self.park_life else {
            return;
        };
        let Some(p) = life.parties.last_mut().filter(|p| p.members.len() < 4) else {
            return;
        };
        p.members.push(id);
        if p.returning {
            life.returning_arrivals += 1;
        }
        if let Some(g) = self.guests.iter_mut().find(|g| g.id == id) {
            g.preferences.comfortable_queue_ticks = match p.style {
                PartyStyle::Family => 1050,
                PartyStyle::Thrill => 1350,
                PartyStyle::Relaxed => 750,
            };
        }
    }
    pub(crate) fn record_life_departure(&mut self, g: &Guest) {
        let Some(life) = &mut self.park_life else {
            return;
        };
        let mut style = None;
        for p in &mut life.parties {
            if p.members.contains(&g.id) {
                p.members.retain(|id| *id != g.id);
                p.satisfaction_total += u16::from(g.satisfaction);
                p.departed_members += 1;
                if p.members.is_empty()
                    && p.satisfaction_total >= u16::from(p.departed_members) * 60
                {
                    style = Some(p.style);
                }
            }
        }
        life.parties.retain(|p| !p.members.is_empty());
        if let Some(style) = style
            && life.returns.len() < 32
        {
            life.returns.push(ReturnVisit {
                style,
                due: self.ticks.saturating_add(6000),
            });
        }
    }
    pub(crate) fn step_life_guests(&mut self) {
        let Some(life) = &self.park_life else { return };
        let mut leaving = Vec::new();
        for party in &life.parties {
            let leave = party.members.iter().any(|id| {
                self.visitors.leaving(*id)
                    || self.guests.iter().find(|g| g.id == *id).is_some_and(|g| {
                        g.satisfaction < 20
                            && self.visitors.stays.iter().any(|s| {
                                s.guest == g.id && self.ticks.saturating_sub(s.entered_tick) >= 1800
                            })
                    })
            });
            if leave {
                leaving.extend_from_slice(&party.members);
            }
        }
        for id in leaving {
            let _ = self.request_guest_departure(id);
        }
        // Scenery appreciation is local and bounded. It does not change guest money.
        if self.ticks.is_multiple_of(300) {
            let scenery: Vec<_> = self
                .objects
                .iter()
                .filter(|o| {
                    matches!(
                        o.kind,
                        crate::ObjectKind::SceneryTree | crate::ObjectKind::FlowerBed
                    )
                })
                .map(|o| o.pos)
                .collect();
            for g in &mut self.guests {
                if g.satisfaction < 80
                    && scenery
                        .iter()
                        .any(|p| p.x.abs_diff(g.anchor.x) + p.y.abs_diff(g.anchor.y) <= 2)
                {
                    g.satisfaction += 1;
                }
            }
        }
    }
    pub(crate) fn life_ride_penalty(&self, guest: GuestId, ride: RideId) -> u128 {
        let Some(life) = &self.park_life else {
            return 0;
        };
        let model = self.ride_by_id(ride).and_then(|r| r.model);
        let category = model.map(|m| m.category());
        let family =
            self.guest_party(guest)
                .map_or(0, |(_, style, _, _)| match (style, category) {
                    (PartyStyle::Family, Some(RideCategory::Gentle | RideCategory::Transport)) => 0,
                    (PartyStyle::Relaxed, Some(RideCategory::Gentle | RideCategory::Transport)) => {
                        0
                    }
                    (PartyStyle::Thrill, Some(RideCategory::Thrill | RideCategory::Coaster)) => 0,
                    _ => 80,
                });
        let weather = match (life.weather, category) {
            (Weather::Rain, Some(RideCategory::Water)) => 150,
            (Weather::Hot, Some(RideCategory::Water)) => 0,
            (Weather::Hot, _) => 20,
            _ => 0,
        };
        family + weather
    }
    pub(crate) fn life_walking_targets(&self) -> std::collections::BTreeMap<GuestId, ObjectId> {
        let mut targets = std::collections::BTreeMap::new();
        let Some(life) = &self.park_life else {
            return targets;
        };
        for g in &self.guests {
            if g.next.is_some()
                || self.visitors.leaving(g.id)
                || g.ride_visit.is_some()
                || g.service_visit.is_some()
            {
                continue;
            }
            if matches!(life.weather, Weather::Rain | Weather::Storm)
                && let Some(current) = g.current
                && let Some(target) = life
                    .shelters
                    .iter()
                    .filter(|p| self.path_is_sheltered(**p))
                    .filter_map(|p| Some((self.public_path_route(current, *p)?.len(), *p)))
                    .min()
            {
                targets.insert(g.id, target.1);
                continue;
            }
            if let Some(p) = life.parties.iter().find(|p| p.members.contains(&g.id))
                && let Some(leader) = p
                    .members
                    .first()
                    .filter(|id| **id != g.id)
                    .and_then(|id| self.guests.iter().find(|g| g.id == *id))
                && let Some(path) = leader
                    .current
                    .filter(|id| self.object(*id).is_some_and(|p| p.is_public_path()))
            {
                targets.insert(g.id, path);
            }
        }
        targets
    }
}
