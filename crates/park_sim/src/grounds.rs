//! Opt-in grounds activity and one path-walking handyman.
use crate::{ObjectId, PATH_UNITS, Park, PathDirection, PathPoint, WALK_TICKS};

pub const GROUND_MESS_CAP: usize = 64;
pub const GROUND_MESS_TICKS: u64 = 600;
pub const GROUND_BIN_CAP: usize = 64;
pub const GROUND_BIN_CAPACITY: u8 = 4;
pub const HANDYMAN_PATROL_CAP: usize = 64;
pub const HANDYMAN_WAGE_TICKS: u16 = 900;
pub const HANDYMAN_WAGE_CENTS: u64 = 50;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundBin {
    pub path: ObjectId,
    pub fill: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroundBinError {
    InvalidPath,
    AtCapacity,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PatrolError {
    InvalidPath,
    AtCapacity,
}
impl std::fmt::Display for PatrolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidPath => "select a public path",
            Self::AtCapacity => "patrol area limit reached",
        })
    }
}

const fn enabled() -> bool {
    true
}
impl std::fmt::Display for GroundBinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidPath => "select a public path",
            Self::AtCapacity => "bin limit reached",
        })
    }
}

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
    #[serde(default)]
    target_bin: Option<ObjectId>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GroundsState {
    pub active: bool,
    pub assigned: bool,
    pub next_mess_id: u64,
    pub cleaned: u64,
    #[serde(default)]
    pub wage_progress: u16,
    #[serde(default)]
    pub bins_emptied: u64,
    #[serde(
        default,
        deserialize_with = "crate::saves::bounded_objects::<_,_,GROUND_BIN_CAP>"
    )]
    pub bins: Vec<GroundBin>,
    #[serde(
        default,
        deserialize_with = "crate::saves::bounded_vec::<_,_,HANDYMAN_PATROL_CAP>"
    )]
    pub patrol: Vec<ObjectId>,
    #[serde(default = "enabled")]
    pub clean_enabled: bool,
    #[serde(default = "enabled")]
    pub bins_enabled: bool,
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
            wage_progress: 0,
            bins_emptied: 0,
            bins: Vec::new(),
            patrol: Vec::new(),
            clean_enabled: true,
            bins_enabled: true,
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
    pub wage_progress: u16,
    pub wages_paid_cents: u64,
    pub bins: usize,
    pub full_bins: usize,
    pub bins_emptied: u64,
    pub patrol_paths: usize,
    pub clean_enabled: bool,
    pub bins_enabled: bool,
    pub uncovered_messes: usize,
    pub uncovered_bins: usize,
    pub current_path: Option<ObjectId>,
    pub next_path: Option<ObjectId>,
    pub walk_progress: u16,
    pub target_mess: Option<u64>,
    pub target_bin: Option<ObjectId>,
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
            wage_progress: state.wage_progress,
            wages_paid_cents: self.finances.archived.handyman_wages_cents
                + self
                    .finances
                    .entries
                    .iter()
                    .filter(|entry| entry.kind == crate::FinanceKind::HandymanWage)
                    .map(|entry| entry.amount_cents)
                    .sum::<u64>(),
            bins: state.bins.len(),
            full_bins: state
                .bins
                .iter()
                .filter(|bin| bin.fill == GROUND_BIN_CAPACITY)
                .count(),
            bins_emptied: state.bins_emptied,
            patrol_paths: state.patrol.len(),
            clean_enabled: state.clean_enabled,
            bins_enabled: state.bins_enabled,
            uncovered_messes: state
                .messes
                .iter()
                .filter(|mess| !self.can_clean_mess(mess))
                .count(),
            uncovered_bins: state
                .bins
                .iter()
                .filter(|bin| bin.fill > 0 && !self.can_empty_bin(bin))
                .count(),
            current_path: state.route.current,
            next_path: state.route.next,
            walk_progress: state.route.progress,
            target_mess: state.route.target,
            target_bin: state.route.target_bin,
        }
    }

    pub fn ground_messes(&self) -> &[GroundMess] {
        &self.grounds.messes
    }

    pub fn ground_bins(&self) -> &[GroundBin] {
        &self.grounds.bins
    }

    pub fn handyman_patrol(&self) -> &[ObjectId] {
        &self.grounds.patrol
    }

    fn in_handyman_patrol(&self, path: ObjectId) -> bool {
        self.grounds.patrol.is_empty() || self.grounds.patrol.binary_search(&path).is_ok()
    }

    fn can_clean_mess(&self, mess: &GroundMess) -> bool {
        self.grounds.clean_enabled && self.in_handyman_patrol(mess.path)
    }

    fn can_empty_bin(&self, bin: &GroundBin) -> bool {
        self.grounds.bins_enabled && self.in_handyman_patrol(bin.path)
    }

    fn clear_handyman_target(&mut self) {
        self.grounds.route.target = None;
        self.grounds.route.target_bin = None;
    }

    /// Empty area means the whole connected park. Each edit preserves the
    /// worker's current edge but discards a target that may no longer qualify.
    pub fn toggle_handyman_patrol(&mut self, path: ObjectId) -> Result<bool, PatrolError> {
        if self
            .object(path)
            .is_none_or(|object| !object.is_public_path())
        {
            return Err(PatrolError::InvalidPath);
        }
        if let Ok(index) = self.grounds.patrol.binary_search(&path) {
            self.grounds.patrol.remove(index);
            self.clear_handyman_target();
            return Ok(false);
        }
        if self.grounds.patrol.len() == HANDYMAN_PATROL_CAP {
            return Err(PatrolError::AtCapacity);
        }
        let index = self.grounds.patrol.partition_point(|id| *id < path);
        self.grounds.patrol.insert(index, path);
        self.grounds.active = true;
        self.clear_handyman_target();
        Ok(true)
    }

    pub fn toggle_handyman_cleaning(&mut self) -> bool {
        self.grounds.active = true;
        self.grounds.clean_enabled = !self.grounds.clean_enabled;
        self.clear_handyman_target();
        self.grounds.clean_enabled
    }

    pub fn toggle_handyman_bins(&mut self) -> bool {
        self.grounds.active = true;
        self.grounds.bins_enabled = !self.grounds.bins_enabled;
        self.clear_handyman_target();
        self.grounds.bins_enabled
    }

    /// Attach or remove one free bin on an existing public path. Installing a
    /// bin starts grounds activity even if no handyman has been assigned.
    pub fn toggle_ground_bin(&mut self, path: ObjectId) -> Result<bool, GroundBinError> {
        if self
            .object(path)
            .is_none_or(|object| !object.is_public_path())
        {
            return Err(GroundBinError::InvalidPath);
        }
        if let Ok(index) = self
            .grounds
            .bins
            .binary_search_by_key(&path, |bin| bin.path)
        {
            self.grounds.bins.remove(index);
            if self.grounds.route.target_bin == Some(path) {
                self.grounds.route.target_bin = None;
            }
            return Ok(false);
        }
        if self.grounds.bins.len() == GROUND_BIN_CAP {
            return Err(GroundBinError::AtCapacity);
        }
        let index = self.grounds.bins.partition_point(|bin| bin.path < path);
        self.grounds.bins.insert(index, GroundBin { path, fill: 0 });
        self.grounds.active = true;
        Ok(true)
    }

    /// First assignment starts grounds activity for this park. Dismissing the
    /// worker leaves existing and future mess visible until reassigned.
    pub fn set_handyman_assigned(&mut self, assigned: bool) -> bool {
        if self.grounds.assigned == assigned {
            return false;
        }
        self.grounds.assigned = assigned;
        self.grounds.active |= assigned;
        self.grounds.wage_progress = 0;
        self.grounds.route = HandymanRoute::default();
        true
    }

    pub(crate) fn step_handyman_wages(&mut self) {
        if !self.grounds.assigned {
            return;
        }
        if self.grounds.wage_progress + 1 < HANDYMAN_WAGE_TICKS {
            self.grounds.wage_progress += 1;
        } else if self.cash_cents < HANDYMAN_WAGE_CENTS {
            self.set_handyman_assigned(false);
        } else {
            self.grounds.wage_progress = 0;
            self.record_cash(crate::FinanceKind::HandymanWage, None, HANDYMAN_WAGE_CENTS);
        }
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
            if kind == GroundMessKind::Litter
                && let Ok(index) = self
                    .grounds
                    .bins
                    .binary_search_by_key(&path, |bin| bin.path)
                && self.grounds.bins[index].fill < GROUND_BIN_CAPACITY
            {
                self.grounds.bins[index].fill += 1;
            } else {
                self.grounds.messes.push(GroundMess {
                    id: self.grounds.next_mess_id,
                    path,
                    kind,
                });
                self.grounds.next_mess_id += 1;
            }
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
                self.grounds.route.target_bin = None;
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
                .filter(|mess| self.can_clean_mess(mess))
                .and_then(|mess| self.public_path_route(current, mess.path).map(|_| id))
        });
        let target_bin = self.grounds.route.target_bin.filter(|id| {
            self.grounds
                .bins
                .iter()
                .any(|bin| bin.path == *id && bin.fill > 0 && self.can_empty_bin(bin))
                && self.public_path_route(current, *id).is_some()
        });
        let choice = if target.is_some() {
            target.map(|id| (Some(id), None))
        } else if target_bin.is_some() {
            target_bin.map(|id| (None, Some(id)))
        } else {
            None
        };
        let choice = choice.or_else(|| {
            let mess = self
                .grounds
                .messes
                .iter()
                .filter(|mess| self.can_clean_mess(mess))
                .filter_map(|mess| {
                    let distance = self.public_path_route(current, mess.path)?.len();
                    Some(((distance, 0_u8, mess.id), (Some(mess.id), None)))
                });
            let bins = self
                .grounds
                .bins
                .iter()
                .filter(|bin| bin.fill > 0 && self.can_empty_bin(bin))
                .filter_map(|bin| {
                    let distance = self.public_path_route(current, bin.path)?.len();
                    Some(((distance, 1_u8, bin.path.0), (None, Some(bin.path))))
                });
            mess.chain(bins)
                .min_by_key(|(key, _)| *key)
                .map(|(_, target)| target)
        });
        let (target, target_bin) = choice.unwrap_or((None, None));
        self.grounds.route.target = target;
        self.grounds.route.target_bin = target_bin;
        let destination = target
            .and_then(|id| self.grounds.messes.iter().find(|mess| mess.id == id))
            .map(|mess| mess.path)
            .or(target_bin)
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
            if let Some(path) = target_bin
                && let Ok(index) = self
                    .grounds
                    .bins
                    .binary_search_by_key(&path, |bin| bin.path)
                && self.grounds.bins[index].fill > 0
            {
                self.grounds.bins[index].fill = 0;
                self.grounds.bins_emptied = self.grounds.bins_emptied.saturating_add(1);
                self.grounds.route.target_bin = None;
            }
            return;
        }
        if let Some(mut route) = self.public_path_route(current, destination) {
            self.grounds.route.next = route.pop_front();
        } else {
            self.grounds.route.target = None;
            self.grounds.route.target_bin = None;
        }
    }

    pub(crate) fn reconcile_grounds_after_removal(&mut self, removed: ObjectId) {
        self.grounds.messes.retain(|mess| mess.path != removed);
        self.grounds.bins.retain(|bin| bin.path != removed);
        self.grounds.patrol.retain(|path| *path != removed);
        if self.grounds.route.current == Some(removed) {
            self.grounds.route = HandymanRoute::default();
        } else if self.grounds.route.next == Some(removed) {
            self.grounds.route.next = None;
            self.grounds.route.progress = 0;
            self.grounds.route.target = None;
            self.grounds.route.target_bin = None;
        }
        if self
            .grounds
            .route
            .target
            .is_some_and(|id| !self.grounds.messes.iter().any(|mess| mess.id == id))
        {
            self.grounds.route.target = None;
        }
        if self.grounds.route.target_bin.is_some_and(|id| {
            !self
                .grounds
                .bins
                .iter()
                .any(|bin| bin.path == id && bin.fill > 0)
        }) {
            self.grounds.route.target_bin = None;
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
        || state.bins.len() > GROUND_BIN_CAP
        || state.wage_progress >= HANDYMAN_WAGE_TICKS
        || (!state.assigned && state.wage_progress != 0)
        || state.next_mess_id == 0
        || (!state.active
            && (state.assigned
                || state.cleaned != 0
                || state.next_mess_id != 1
                || state.bins_emptied != 0
                || !state.bins.is_empty()
                || !state.patrol.is_empty()
                || !state.clean_enabled
                || !state.bins_enabled
                || !state.messes.is_empty()
                || state.route != HandymanRoute::default()))
        || (state.assigned && !state.active)
        || (!state.assigned && state.route != HandymanRoute::default())
        || state.route.progress >= WALK_TICKS
        || (state.route.next.is_none() && state.route.progress != 0)
        || (state.route.current.is_none()
            && (state.route.next.is_some()
                || state.route.target.is_some()
                || state.route.target_bin.is_some()))
        || (state.route.target.is_some() && state.route.target_bin.is_some())
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
        || state.route.target.is_some_and(|id| {
            !state
                .messes
                .iter()
                .any(|mess| mess.id == id && park.can_clean_mess(mess))
        })
        || state.route.target_bin.is_some_and(|id| {
            !state
                .bins
                .iter()
                .any(|bin| bin.path == id && bin.fill > 0 && park.can_empty_bin(bin))
        })
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
    let mut prior_bin = None;
    for bin in &state.bins {
        if prior_bin.is_some_and(|id| id >= bin.path)
            || bin.fill > GROUND_BIN_CAPACITY
            || park
                .object(bin.path)
                .is_none_or(|path| !path.is_public_path())
        {
            return Err("grounds bin identity, path or fill is invalid".into());
        }
        prior_bin = Some(bin.path);
    }
    if state.patrol.len() > HANDYMAN_PATROL_CAP {
        return Err("handyman patrol exceeds its limit".into());
    }
    let mut prior_path = None;
    for path in &state.patrol {
        if prior_path.is_some_and(|prior| prior >= *path)
            || park
                .object(*path)
                .is_none_or(|object| !object.is_public_path())
        {
            return Err("handyman patrol path is invalid".into());
        }
        prior_path = Some(*path);
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
    fn patrol_limits_work_without_blocking_transit_and_duties_are_independent() {
        let (mut park, _entry, near) = park();
        let far = park.place_path(TilePos { x: 7, y: 5 }, 0).unwrap();
        park.set_handyman_assigned(true);
        park.grounds.messes = vec![
            GroundMess {
                id: 1,
                path: near,
                kind: GroundMessKind::Litter,
            },
            GroundMess {
                id: 2,
                path: far,
                kind: GroundMessKind::Vomit,
            },
        ];
        park.grounds.next_mess_id = 3;
        assert_eq!(park.toggle_handyman_patrol(far), Ok(true));
        park.advance(Speed::Normal);
        assert_eq!(park.grounds_status().next_path, Some(near));
        assert_eq!(park.grounds_status().target_mess, Some(2));
        assert_eq!(park.grounds_status().uncovered_messes, 1);
        for _ in 0..90 {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.ground_messes().len(), 1);
        assert_eq!(park.ground_messes()[0].path, near);
        assert_eq!(park.grounds_status().cleaned, 1);
        assert!(!park.toggle_handyman_cleaning());
        assert_eq!(park.toggle_handyman_patrol(far), Ok(false));
        assert_eq!(park.toggle_ground_bin(near), Ok(true));
        park.grounds.bins[0].fill = 2;
        for _ in 0..90 {
            park.advance(Speed::Normal);
            if park.grounds.bins[0].fill == 0 {
                break;
            }
        }
        assert_eq!(park.grounds.bins[0].fill, 0);
        assert_eq!(park.ground_messes().len(), 1);
        assert!(!park.toggle_handyman_bins());
        park.grounds.bins[0].fill = 1;
        for _ in 0..90 {
            park.advance(Speed::Normal);
        }
        assert_eq!(park.grounds.bins[0].fill, 1);
        assert_eq!(park.grounds_status().uncovered_bins, 1);
        assert!(park.grounds_status().uncovered_messes > 0);
    }

    #[test]
    fn patrol_demolition_and_schema_31_migration_are_strict() {
        let (mut park, _entry, path) = park();
        let original = park.clone();
        let mut old: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        old["schema_version"] = 31.into();
        old["content_version"] = 31.into();
        crate::legacy_cost_test_support::strip_park_open(&mut old);
        for field in ["patrol", "clean_enabled", "bins_enabled"] {
            old["park"]["grounds"]
                .as_object_mut()
                .unwrap()
                .remove(field);
        }
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).unwrap(),
            original
        );
        old["park"]["grounds"]["patrol"] = serde_json::json!([]);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()).is_err());
        let mut current: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        current["park"]["grounds"]
            .as_object_mut()
            .unwrap()
            .remove("bins_enabled");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&current).unwrap()).is_err());
        assert_eq!(park.toggle_handyman_patrol(path), Ok(true));
        park.remove_object(path).unwrap();
        assert!(park.handyman_patrol().is_empty());
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
        let mut forged: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        forged["park"]["grounds"]["patrol"] = serde_json::json!([path]);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
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
        crate::legacy_cost_test_support::strip_park_open(&mut old);
        old["park"].as_object_mut().unwrap().remove("grounds");
        old["park"]["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove("handyman_wages_cents");
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

    #[test]
    fn handyman_wages_are_recorded_and_insufficient_cash_dismisses() {
        let (mut paid, _, _) = park();
        paid.set_handyman_assigned(true);
        for _ in 0..HANDYMAN_WAGE_TICKS - 1 {
            paid.advance(Speed::Normal);
        }
        assert_eq!(paid.grounds_status().wage_progress, HANDYMAN_WAGE_TICKS - 1);
        assert_eq!(paid.grounds_status().wages_paid_cents, 0);
        paid.advance(Speed::Normal);
        assert_eq!(paid.grounds_status().wage_progress, 0);
        assert_eq!(paid.grounds_status().wages_paid_cents, HANDYMAN_WAGE_CENTS);
        assert_eq!(paid.finance_stats().wages_cents, HANDYMAN_WAGE_CENTS);
        assert_eq!(
            paid.cash_cents(),
            crate::INITIAL_PARK_CASH_CENTS - HANDYMAN_WAGE_CENTS
        );
        assert_eq!(
            Park::from_save_bytes(&paid.to_save_bytes().unwrap()).unwrap(),
            paid
        );
        let (mut poor, _, _) = park();
        poor.cash_cents = HANDYMAN_WAGE_CENTS - 1;
        poor.set_handyman_assigned(true);
        for _ in 0..HANDYMAN_WAGE_TICKS {
            poor.advance(Speed::Normal);
        }
        assert!(!poor.grounds_status().assigned);
        assert_eq!(poor.grounds_status().wage_progress, 0);
        assert_eq!(poor.cash_cents(), HANDYMAN_WAGE_CENTS - 1);
    }

    #[test]
    fn bins_capture_litter_worker_empties_and_demolition_reconciles() {
        let (mut park, entry, dirt) = park();
        assert_eq!(park.toggle_ground_bin(dirt), Ok(true));
        assert!(park.grounds_status().active);
        assert_eq!(
            park.ground_bins(),
            &[GroundBin {
                path: dirt,
                fill: 0
            }]
        );
        park.grounds.bins[0].fill = GROUND_BIN_CAPACITY;
        park.set_handyman_assigned(true);
        park.advance(Speed::Normal);
        assert_eq!(park.grounds_status().target_bin, Some(dirt));
        assert_eq!(park.grounds_status().next_path, Some(dirt));
        for _ in 0..8 {
            park.advance(Speed::Normal);
        }
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        for _ in 0..60 {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
            assert_eq!(loaded, park);
        }
        assert_eq!(park.ground_bins()[0].fill, 0);
        assert_eq!(park.grounds_status().bins_emptied, 1);
        assert_eq!(park.toggle_ground_bin(dirt), Ok(false));
        assert!(park.ground_bins().is_empty());
        assert_eq!(park.toggle_ground_bin(entry), Ok(true));
        park.remove_object(entry).unwrap();
        assert!(park.ground_bins().is_empty());
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn schema_29_migrates_paid_staff_and_bins_strictly() {
        let (mut park, _, dirt) = park();
        park.set_handyman_assigned(true);
        park.advance(Speed::Normal);
        let current: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        let mut prior = current.clone();
        prior["schema_version"] = 29.into();
        prior["content_version"] = 29.into();
        crate::legacy_cost_test_support::strip_park_open(&mut prior);
        let grounds = prior["park"]["grounds"].as_object_mut().unwrap();
        grounds.remove("bins");
        grounds.remove("bins_emptied");
        grounds.remove("wage_progress");
        grounds.remove("patrol");
        grounds.remove("clean_enabled");
        grounds.remove("bins_enabled");
        grounds["route"]
            .as_object_mut()
            .unwrap()
            .remove("target_bin");
        prior["park"]["finances"]["archived"]
            .as_object_mut()
            .unwrap()
            .remove("handyman_wages_cents");
        let migrated = Park::from_save_bytes(&serde_json::to_vec(&prior).unwrap()).unwrap();
        assert!(migrated.grounds_status().assigned);
        assert_eq!(migrated.grounds_status().wage_progress, 0);
        assert_eq!(migrated.ground_bins().len(), 0);
        assert_eq!(
            migrated.grounds_status().current_path,
            park.grounds_status().current_path
        );
        let mut smuggled = prior.clone();
        smuggled["park"]["grounds"]["bins"] = serde_json::json!([]);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&smuggled).unwrap()).is_err());
        let mut missing = current.clone();
        missing["park"]["grounds"]
            .as_object_mut()
            .unwrap()
            .remove("bins");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&missing).unwrap()).is_err());
        let mut invalid = current;
        invalid["park"]["grounds"]["bins"] = serde_json::json!([{"path":dirt,"fill":255}]);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());
    }
}
