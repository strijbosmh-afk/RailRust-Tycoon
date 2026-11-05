//! Opt-in grounds activity and one path-walking handyman.
use crate::{ObjectId, PATH_UNITS, Park, PathDirection, PathPoint, WALK_TICKS};

pub const GROUND_MESS_CAP: usize = 64;
pub const GROUND_MESS_TICKS: u64 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GroundMessKind {
    Litter,
    Vomit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundMess {
    pub id: u64,
    pub path: ObjectId,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub kind: GroundMessKind,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HandymanRoute {
    #[serde(deserialize_with = "crate::saves::required_option")]
    current: Option<ObjectId>,
    #[serde(deserialize_with = "crate::saves::required_option")]
    next: Option<ObjectId>,
    progress: u16,
    #[serde(deserialize_with = "crate::saves::required_option")]
    target: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GroundsState {
    pub active: bool,
    pub assigned: bool,
    pub next_mess_id: u64,
    pub cleaned: u64,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,GROUND_MESS_CAP>")]
    pub messes: Vec<GroundMess>,
    #[serde(deserialize_with = "crate::saves::strict_object")]
    route: HandymanRoute,
}

impl Default for GroundsState {
    fn default() -> Self {
        Self {
            active: false,
            assigned: false,
            next_mess_id: 1,
            cleaned: 0,
            messes: Vec::new(),
            route: HandymanRoute::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroundsStatus {
    pub active: bool,
    pub assigned: bool,
    pub litter: usize,
    pub vomit: usize,
    pub cleaned: u64,
    pub current_path: Option<ObjectId>,
    pub next_path: Option<ObjectId>,
    pub walk_progress: u16,
    pub target_mess: Option<u64>,
}

impl Park {
    pub fn grounds_status(&self) -> GroundsStatus {
        let state = &self.grounds;
        GroundsStatus {
            active: state.active,
            assigned: state.assigned,
            litter: state
                .messes
                .iter()
                .filter(|mess| mess.kind == GroundMessKind::Litter)
                .count(),
            vomit: state
                .messes
                .iter()
                .filter(|mess| mess.kind == GroundMessKind::Vomit)
                .count(),
            cleaned: state.cleaned,
            current_path: state.route.current,
            next_path: state.route.next,
            walk_progress: state.route.progress,
            target_mess: state.route.target,
        }
    }

    pub fn ground_messes(&self) -> &[GroundMess] {
        &self.grounds.messes
    }

    /// First assignment starts grounds activity for this park. Dismissing the
    /// worker leaves existing and future mess visible until reassigned.
    pub fn set_handyman_assigned(&mut self, assigned: bool) -> bool {
        if self.grounds.assigned == assigned {
            return false;
        }
        self.grounds.assigned = assigned;
        self.grounds.active |= assigned;
        self.grounds.route = HandymanRoute::default();
        true
    }

    pub(crate) fn step_grounds(&mut self) {
        if !self.grounds.active {
            return;
        }
        if self.ticks.is_multiple_of(GROUND_MESS_TICKS)
            && self.grounds.messes.len() < GROUND_MESS_CAP
            && self.grounds.next_mess_id < u64::MAX
            && let Some((path, kind)) = self
                .guests
                .iter()
                .filter_map(|guest| {
                    let path = guest.current?;
                    (guest.next.is_none()
                        && self
                            .object(path)
                            .is_some_and(|object| object.is_public_path())
                        && !self.grounds.messes.iter().any(|mess| mess.path == path))
                    .then_some((guest.id, path, guest.needs.nausea))
                })
                .min_by_key(|(guest, _, _)| *guest)
                .map(|(_, path, nausea)| {
                    (
                        path,
                        if nausea >= 40 {
                            GroundMessKind::Vomit
                        } else {
                            GroundMessKind::Litter
                        },
                    )
                })
        {
            self.grounds.messes.push(GroundMess {
                id: self.grounds.next_mess_id,
                path,
                kind,
            });
            self.grounds.next_mess_id += 1;
        }
        if self.ticks.is_multiple_of(90) {
            for guest in &mut self.guests {
                if guest.next.is_none()
                    && guest.current.is_some_and(|path| {
                        self.grounds.messes.iter().any(|mess| mess.path == path)
                    })
                {
                    guest.satisfaction = guest.satisfaction.saturating_sub(1);
                }
            }
        }
        if !self.grounds.assigned {
            return;
        }
        let entry = self
            .visitors
            .entry
            .filter(|id| self.object(*id).is_some_and(|path| path.is_public_path()));
        if self
            .grounds
            .route
            .current
            .is_none_or(|id| self.object(id).is_none_or(|path| !path.is_public_path()))
        {
            self.grounds.route = HandymanRoute {
                current: entry,
                ..HandymanRoute::default()
            };
        }
        let Some(current) = self.grounds.route.current else {
            return;
        };
        if let Some(next) = self.grounds.route.next {
            if !self.path_neighbors(current).contains(&next) {
                self.grounds.route.next = None;
                self.grounds.route.progress = 0;
                self.grounds.route.target = None;
            } else {
                self.grounds.route.progress += 1;
                if self.grounds.route.progress < WALK_TICKS {
                    return;
                }
                self.grounds.route.current = Some(next);
                self.grounds.route.next = None;
                self.grounds.route.progress = 0;
            }
        }
        let current = self.grounds.route.current.expect("retained path");
        let target = self.grounds.route.target.and_then(|id| {
            self.grounds
                .messes
                .iter()
                .find(|mess| mess.id == id)
                .and_then(|mess| self.public_path_route(current, mess.path).map(|_| id))
        });
        let target = target.or_else(|| {
            self.grounds
                .messes
                .iter()
                .filter_map(|mess| {
                    let route = self.public_path_route(current, mess.path)?;
                    Some((route.len(), mess.id))
                })
                .min()
                .map(|(_, id)| id)
        });
        self.grounds.route.target = target;
        let destination = target
            .and_then(|id| self.grounds.messes.iter().find(|mess| mess.id == id))
            .map(|mess| mess.path)
            .or(entry);
        let Some(destination) = destination else {
            return;
        };
        if current == destination {
            if let Some(id) = target {
                self.grounds.messes.retain(|mess| mess.id != id);
                self.grounds.cleaned = self.grounds.cleaned.saturating_add(1);
                self.grounds.route.target = None;
            }
            return;
        }
        if let Some(mut route) = self.public_path_route(current, destination) {
            self.grounds.route.next = route.pop_front();
        } else {
            self.grounds.route.target = None;
        }
    }

    pub(crate) fn reconcile_grounds_after_removal(&mut self, removed: ObjectId) {
        self.grounds.messes.retain(|mess| mess.path != removed);
        if self.grounds.route.current == Some(removed) {
            self.grounds.route = HandymanRoute::default();
        } else if self.grounds.route.next == Some(removed) {
            self.grounds.route.next = None;
            self.grounds.route.progress = 0;
            self.grounds.route.target = None;
        }
        if self
            .grounds
            .route
            .target
            .is_some_and(|id| !self.grounds.messes.iter().any(|mess| mess.id == id))
        {
            self.grounds.route.target = None;
        }
    }

    pub fn handyman_position(&self) -> Option<PathPoint> {
        if !self.grounds.assigned {
            return None;
        }
        let route = self.grounds.route;
        let current = self.object(route.current?)?;
        let from = current.path_center()?;
        let Some(next) = route.next.and_then(|id| self.object(id)) else {
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
    let state = &park.grounds;
    if state.messes.len() > GROUND_MESS_CAP
        || state.next_mess_id == 0
        || (!state.active
            && (state.assigned
                || state.cleaned != 0
                || state.next_mess_id != 1
                || !state.messes.is_empty()
                || state.route != HandymanRoute::default()))
        || (state.assigned && !state.active)
        || (!state.assigned && state.route != HandymanRoute::default())
        || state.route.progress >= WALK_TICKS
        || (state.route.next.is_none() && state.route.progress != 0)
        || (state.route.current.is_none()
            && (state.route.next.is_some() || state.route.target.is_some()))
        || state
            .route
            .current
            .is_some_and(|id| park.object(id).is_none_or(|path| !path.is_public_path()))
        || state
            .route
            .next
            .is_some_and(|id| park.object(id).is_none_or(|path| !path.is_public_path()))
        || state.route.next.is_some_and(|id| {
            !park
                .path_neighbors(state.route.current.unwrap())
                .contains(&id)
        })
        || state
            .route
            .target
            .is_some_and(|id| !state.messes.iter().any(|mess| mess.id == id))
    {
        return Err("grounds route or status is invalid".into());
    }
    let mut previous = 0;
    let mut paths = std::collections::BTreeSet::new();
    for mess in &state.messes {
        if mess.id <= previous
            || mess.id >= state.next_mess_id
            || !paths.insert(mess.path)
            || park
                .object(mess.path)
                .is_none_or(|path| !path.is_public_path())
        {
            return Err("grounds mess identity or path is invalid".into());
        }
        previous = mess.id;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Speed, TilePos};

    fn park() -> (Park, ObjectId, ObjectId) {
        let mut park = Park::new(32, 32);
        let entry = park.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
        let dirt = park.place_path(TilePos { x: 6, y: 5 }, 0).unwrap();
        park.set_visitor_entry(entry).unwrap();
        (park, entry, dirt)
    }

    #[test]
    fn handyman_walks_cleans_and_continues_after_loading() {
        let (mut park, _entry, dirt) = park();
        park.set_handyman_assigned(true);
        park.grounds.messes.push(GroundMess {
            id: 1,
            path: dirt,
            kind: GroundMessKind::Litter,
        });
        park.grounds.next_mess_id = 2;
        park.advance(Speed::Normal);
        assert_eq!(park.grounds_status().next_path, Some(dirt));
        for _ in 0..12 {
            park.advance(Speed::Normal);
        }
        assert!(park.grounds_status().walk_progress > 0);
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(loaded, park);
        for _ in 0..80 {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
            assert_eq!(loaded, park);
        }
        assert!(park.ground_messes().is_empty());
        assert_eq!(park.grounds_status().cleaned, 1);
    }

    #[test]
    fn demolition_clears_mess_and_route_without_awarding_cleaning() {
        let (mut park, _, dirt) = park();
        park.set_handyman_assigned(true);
        park.grounds.messes.push(GroundMess {
            id: 1,
            path: dirt,
            kind: GroundMessKind::Vomit,
        });
        park.grounds.next_mess_id = 2;
        park.advance(Speed::Normal);
        park.remove_object(dirt).unwrap();
        assert!(park.ground_messes().is_empty());
        assert_eq!(park.grounds_status().cleaned, 0);
        assert_eq!(park.grounds_status().next_path, None);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn schema_28_migrates_and_current_rejects_invalid_grounds() {
        let (park, _, dirt) = park();
        let mut value: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        let mut old = value.clone();
        old["schema_version"] = 28.into();
        old["content_version"] = 28.into();
        old["park"].as_object_mut().unwrap().remove("grounds");
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).unwrap(),
            park
        );
        old["park"]["grounds"] = value["park"]["grounds"].clone();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).is_err());
        value["park"].as_object_mut().unwrap().remove("grounds");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
        let mut bad: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        bad["park"]["grounds"]["active"] = true.into();
        bad["park"]["grounds"]["next_mess_id"] = 2.into();
        bad["park"]["grounds"]["messes"] =
            serde_json::json!([{"id":1,"path":dirt,"kind":"Litter"}]);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&bad).unwrap()).is_ok());
        bad["park"]["grounds"]["messes"][0]["path"] = 999999.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }

    #[test]
    fn visitors_naturally_dirty_paths_only_after_first_assignment() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(7).unwrap();
        for _ in 0..1_800 {
            park.advance(Speed::Normal);
        }
        assert!(!park.grounds_status().active);
        assert!(park.ground_messes().is_empty());
        park.set_handyman_assigned(true);
        park.set_handyman_assigned(false);
        for _ in 0..12_000 {
            if !park.ground_messes().is_empty() {
                break;
            }
            park.advance(Speed::Normal);
        }
        assert!(!park.ground_messes().is_empty());
        assert!(!park.grounds_status().assigned);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }
}
