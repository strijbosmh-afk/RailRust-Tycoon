//! Player-funded scenery and path benches. Fixture scenery remains free.
use crate::{FinanceKind, ObjectId, ObjectKind, Park, PlacementError, SceneryKind, TilePos};

pub const BENCH_CAP: usize = 256;
pub const BENCH_COST_CENTS: u64 = 100;
pub const SIGN_COST_CENTS: u64 = 200;
pub const FENCE_COST_CENTS: u64 = 300;
pub const DECORATION_REFUND_PERCENT: u64 = 50;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaidScenery {
    pub object: ObjectId,
    pub cost_cents: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathBench {
    pub path: ObjectId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathSign {
    pub path: ObjectId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathFence {
    pub path: ObjectId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathAdditionKind {
    Bench,
    Sign,
    Fence,
}
impl PathAdditionKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bench => "Bench",
            Self::Sign => "Sign",
            Self::Fence => "Fence",
        }
    }
    pub const fn cost_cents(self) -> u64 {
        match self {
            Self::Bench => BENCH_COST_CENTS,
            Self::Sign => SIGN_COST_CENTS,
            Self::Fence => FENCE_COST_CENTS,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecorationState {
    #[serde(
        deserialize_with = "crate::saves::bounded_objects::<_,_,{crate::MAX_SCENERY_OBJECTS}>"
    )]
    pub scenery: Vec<PaidScenery>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,BENCH_CAP>")]
    pub benches: Vec<PathBench>,
    #[serde(
        default,
        deserialize_with = "crate::saves::bounded_objects::<_,_,BENCH_CAP>"
    )]
    pub signs: Vec<PathSign>,
    #[serde(
        default,
        deserialize_with = "crate::saves::bounded_objects::<_,_,BENCH_CAP>"
    )]
    pub fences: Vec<PathFence>,
    pub construction_cents: u64,
    pub refund_cents: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchError {
    InvalidPath,
    AtCapacity,
    InsufficientFunds {
        required_cents: u64,
        available_cents: u64,
    },
}
pub type PathAdditionError = BenchError;
impl std::fmt::Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => f.write_str("select a public path"),
            Self::AtCapacity => f.write_str("path furniture limit reached"),
            Self::InsufficientFunds {
                required_cents,
                available_cents,
            } => write!(
                f,
                "path furniture costs ${}.{:02}; park cash is ${}.{:02}",
                required_cents / 100,
                required_cents % 100,
                available_cents / 100,
                available_cents % 100
            ),
        }
    }
}

impl SceneryKind {
    pub const fn build_cost_cents(self) -> u64 {
        match self {
            Self::Tree => 250,
            Self::FlowerBed => 150,
        }
    }
}

impl Park {
    pub fn path_benches(&self) -> &[PathBench] {
        &self.decorations.benches
    }
    pub fn path_signs(&self) -> &[PathSign] {
        &self.decorations.signs
    }
    pub fn path_fences(&self) -> &[PathFence] {
        &self.decorations.fences
    }

    pub fn toggle_path_addition(
        &mut self,
        path: ObjectId,
        kind: PathAdditionKind,
    ) -> Result<bool, PathAdditionError> {
        if kind == PathAdditionKind::Bench {
            return self.toggle_path_bench(path);
        }
        if self
            .object(path)
            .is_none_or(|object| !object.is_public_path())
        {
            return Err(BenchError::InvalidPath);
        }
        let present = match kind {
            PathAdditionKind::Sign => self
                .decorations
                .signs
                .binary_search_by_key(&path, |item| item.path)
                .ok(),
            PathAdditionKind::Fence => self
                .decorations
                .fences
                .binary_search_by_key(&path, |item| item.path)
                .ok(),
            PathAdditionKind::Bench => unreachable!(),
        };
        if let Some(index) = present {
            match kind {
                PathAdditionKind::Sign => {
                    self.decorations.signs.remove(index);
                }
                PathAdditionKind::Fence => {
                    self.decorations.fences.remove(index);
                }
                PathAdditionKind::Bench => unreachable!(),
            }
            self.record_decoration_refund(kind.cost_cents());
            return Ok(false);
        }
        let count = match kind {
            PathAdditionKind::Sign => self.decorations.signs.len(),
            PathAdditionKind::Fence => self.decorations.fences.len(),
            PathAdditionKind::Bench => unreachable!(),
        };
        if count >= BENCH_CAP {
            return Err(BenchError::AtCapacity);
        }
        let cost = kind.cost_cents();
        if self.cash_cents < cost {
            return Err(BenchError::InsufficientFunds {
                required_cents: cost,
                available_cents: self.cash_cents,
            });
        }
        match kind {
            PathAdditionKind::Sign => {
                let index = self
                    .decorations
                    .signs
                    .partition_point(|item| item.path < path);
                self.decorations.signs.insert(index, PathSign { path });
            }
            PathAdditionKind::Fence => {
                let index = self
                    .decorations
                    .fences
                    .partition_point(|item| item.path < path);
                self.decorations.fences.insert(index, PathFence { path });
            }
            PathAdditionKind::Bench => unreachable!(),
        }
        self.decorations.construction_cents += cost;
        self.record_cash(FinanceKind::DecorationConstruction, None, cost);
        Ok(true)
    }

    pub fn decoration_refund_quote(&self, object: ObjectId) -> Option<u64> {
        let mut path_cost = 0;
        if self
            .decorations
            .benches
            .binary_search_by_key(&object, |bench| bench.path)
            .is_ok()
        {
            path_cost += BENCH_COST_CENTS;
        }
        if self
            .decorations
            .signs
            .binary_search_by_key(&object, |item| item.path)
            .is_ok()
        {
            path_cost += SIGN_COST_CENTS;
        }
        if self
            .decorations
            .fences
            .binary_search_by_key(&object, |item| item.path)
            .is_ok()
        {
            path_cost += FENCE_COST_CENTS;
        }
        if path_cost != 0 {
            return Some(path_cost * DECORATION_REFUND_PERCENT / 100);
        }
        self.decorations
            .scenery
            .iter()
            .find(|item| item.object == object)
            .map(|item| item.cost_cents * DECORATION_REFUND_PERCENT / 100)
    }

    pub fn quote_scenery(&self, pos: TilePos, kind: SceneryKind) -> Result<u64, PlacementError> {
        self.validate_scenery(pos, kind)?;
        let cost = kind.build_cost_cents();
        if self.cash_cents < cost {
            return Err(PlacementError::InsufficientFunds {
                required_cents: cost,
                available_cents: self.cash_cents,
            });
        }
        Ok(cost)
    }

    pub fn build_scenery(
        &mut self,
        pos: TilePos,
        kind: SceneryKind,
    ) -> Result<ObjectId, PlacementError> {
        let cost = self.quote_scenery(pos, kind)?;
        let id = self.place_scenery(pos, kind)?;
        self.decorations.scenery.push(PaidScenery {
            object: id,
            cost_cents: cost,
        });
        self.decorations.construction_cents += cost;
        self.record_cash(FinanceKind::DecorationConstruction, None, cost);
        Ok(id)
    }

    /// Toggle a paid bench on a public path. Removing it returns half its cost.
    pub fn toggle_path_bench(&mut self, path: ObjectId) -> Result<bool, BenchError> {
        if self
            .object(path)
            .is_none_or(|object| !object.is_public_path())
        {
            return Err(BenchError::InvalidPath);
        }
        if let Ok(index) = self
            .decorations
            .benches
            .binary_search_by_key(&path, |bench| bench.path)
        {
            self.decorations.benches.remove(index);
            self.record_decoration_refund(BENCH_COST_CENTS);
            return Ok(false);
        }
        if self.decorations.benches.len() >= BENCH_CAP {
            return Err(BenchError::AtCapacity);
        }
        if self.cash_cents < BENCH_COST_CENTS {
            return Err(BenchError::InsufficientFunds {
                required_cents: BENCH_COST_CENTS,
                available_cents: self.cash_cents,
            });
        }
        let index = self
            .decorations
            .benches
            .partition_point(|bench| bench.path < path);
        self.decorations.benches.insert(index, PathBench { path });
        self.decorations.construction_cents += BENCH_COST_CENTS;
        self.record_cash(FinanceKind::DecorationConstruction, None, BENCH_COST_CENTS);
        Ok(true)
    }

    pub(crate) fn refund_removed_decoration(&mut self, removed: ObjectId) {
        if let Ok(index) = self
            .decorations
            .benches
            .binary_search_by_key(&removed, |bench| bench.path)
        {
            self.decorations.benches.remove(index);
            self.record_decoration_refund(BENCH_COST_CENTS);
        }
        if let Ok(index) = self
            .decorations
            .signs
            .binary_search_by_key(&removed, |item| item.path)
        {
            self.decorations.signs.remove(index);
            self.record_decoration_refund(SIGN_COST_CENTS);
        }
        if let Ok(index) = self
            .decorations
            .fences
            .binary_search_by_key(&removed, |item| item.path)
        {
            self.decorations.fences.remove(index);
            self.record_decoration_refund(FENCE_COST_CENTS);
        }
        if let Some(index) = self
            .decorations
            .scenery
            .iter()
            .position(|scenery| scenery.object == removed)
        {
            let record = self.decorations.scenery.remove(index);
            self.record_decoration_refund(record.cost_cents);
        }
    }

    fn record_decoration_refund(&mut self, cost: u64) {
        let refund = cost * DECORATION_REFUND_PERCENT / 100;
        self.decorations.refund_cents += refund;
        self.record_cash(FinanceKind::DecorationRefund, None, refund);
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    let state = &park.decorations;
    let mut prior = None;
    for bench in &state.benches {
        if prior.is_some_and(|id| id >= bench.path)
            || park
                .object(bench.path)
                .is_none_or(|object| !object.is_public_path())
        {
            return Err("decoration bench path is invalid".into());
        }
        prior = Some(bench.path);
    }
    for paths in [
        state.signs.iter().map(|item| item.path).collect::<Vec<_>>(),
        state
            .fences
            .iter()
            .map(|item| item.path)
            .collect::<Vec<_>>(),
    ] {
        let mut prior = None;
        for path in paths {
            if prior.is_some_and(|id| id >= path)
                || park
                    .object(path)
                    .is_none_or(|object| !object.is_public_path())
            {
                return Err("decoration addition path is invalid".into());
            }
            prior = Some(path);
        }
    }
    let mut prior = None;
    for scenery in &state.scenery {
        if prior.is_some_and(|id| id >= scenery.object)
            || park.object(scenery.object).is_none_or(|object| {
                !matches!(object.kind, ObjectKind::SceneryTree | ObjectKind::FlowerBed)
                    || scenery.cost_cents
                        != match object.kind {
                            ObjectKind::SceneryTree => SceneryKind::Tree.build_cost_cents(),
                            _ => SceneryKind::FlowerBed.build_cost_cents(),
                        }
            })
        {
            return Err("paid scenery identity or cost is invalid".into());
        }
        prior = Some(scenery.object);
    }
    let active_cost: u64 = state
        .scenery
        .iter()
        .map(|item| item.cost_cents)
        .sum::<u64>()
        + state.benches.len() as u64 * BENCH_COST_CENTS
        + state.signs.len() as u64 * SIGN_COST_CENTS
        + state.fences.len() as u64 * FENCE_COST_CENTS;
    if state.construction_cents < active_cost
        || state.refund_cents.checked_mul(2) != state.construction_cents.checked_sub(active_cost)
        || u128::from(state.construction_cents)
            > u128::from(park.finances.next_entry_id.saturating_sub(1)) * 300
        || u128::from(state.refund_cents)
            > u128::from(park.finances.next_entry_id.saturating_sub(1)) * 150
    {
        return Err("decoration costs or refunds are invalid".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_land(park: &Park, kind: SceneryKind) -> TilePos {
        (0..park.height())
            .flat_map(|y| (0..park.width()).map(move |x| TilePos { x, y }))
            .find(|pos| park.validate_scenery(*pos, kind).is_ok())
            .unwrap()
    }

    #[test]
    fn paid_scenery_and_benches_refund_once_and_roundtrip() {
        let mut park = Park::new(32, 32);
        let path = park.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
        let starting_cash = park.cash_cents();
        let tree = park
            .build_scenery(clear_land(&park, SceneryKind::Tree), SceneryKind::Tree)
            .unwrap();
        assert_eq!(park.toggle_path_bench(path), Ok(true));
        assert_eq!(park.cash_cents(), starting_cash - 350);
        let restored = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(restored, park);
        assert_eq!(park.remove_object(tree).unwrap().id, tree);
        assert_eq!(park.remove_object(path).unwrap().id, path);
        assert_eq!(park.cash_cents(), starting_cash - 175);
        assert!(park.path_benches().is_empty());
        assert!(park.remove_object(tree).is_none());
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn insufficient_cash_does_not_change_objects_or_ledger() {
        let mut park = Park::new(32, 32);
        let path = park.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
        park.cash_cents = 0;
        let before_objects = park.objects().len();
        let before_entries = park.finance_ledger().entries.len();
        assert!(matches!(
            park.build_scenery(clear_land(&park, SceneryKind::Tree), SceneryKind::Tree),
            Err(PlacementError::InsufficientFunds { .. })
        ));
        assert!(matches!(
            park.toggle_path_bench(path),
            Err(BenchError::InsufficientFunds { .. })
        ));
        assert!(matches!(
            park.toggle_path_addition(path, PathAdditionKind::Sign),
            Err(BenchError::InsufficientFunds { .. })
        ));
        assert!(matches!(
            park.toggle_path_addition(path, PathAdditionKind::Fence),
            Err(BenchError::InsufficientFunds { .. })
        ));
        assert_eq!(park.objects().len(), before_objects);
        assert_eq!(park.finance_ledger().entries.len(), before_entries);
    }

    #[test]
    fn archived_decoration_receipts_survive_save_and_load() {
        let mut park = Park::new(32, 32);
        let path = park.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
        for _ in 0..70 {
            assert_eq!(park.toggle_path_bench(path), Ok(true));
            assert_eq!(park.toggle_path_bench(path), Ok(false));
        }
        assert!(park.finance_ledger().archived_entries > 0);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn schema_40_migrates_free_scenery_and_current_rejects_unbacked_decoration() {
        let mut park = Park::new(32, 32);
        let pos = clear_land(&park, SceneryKind::Tree);
        let id = park.place_scenery(pos, SceneryKind::Tree).unwrap();
        let mut json: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        let mut old = json.clone();
        old["schema_version"] = 40.into();
        old["content_version"] = 40.into();
        old["park"].as_object_mut().unwrap().remove("decorations");
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()),
            Ok(park.clone())
        );
        assert_eq!(park.decoration_refund_quote(id), None);
        json["park"]["decorations"]["scenery"] =
            serde_json::json!([{"object": id.0, "cost_cents": 250}]);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
        json["park"]["decorations"]
            .as_object_mut()
            .unwrap()
            .remove("refund_cents");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&json).unwrap()).is_err());
    }

    #[test]
    fn signs_and_fences_share_a_path_and_refund_once_on_demolition() {
        let mut park = Park::new(32, 32);
        let path = park.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
        let cash = park.cash_cents();
        for kind in [
            PathAdditionKind::Bench,
            PathAdditionKind::Sign,
            PathAdditionKind::Fence,
        ] {
            assert_eq!(park.toggle_path_addition(path, kind), Ok(true));
        }
        assert_eq!(park.cash_cents(), cash - 600);
        assert_eq!(park.decoration_refund_quote(path), Some(300));
        assert_eq!(park.park_value().decorations_cents, 600);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
        assert_eq!(park.remove_object(path).unwrap().id, path);
        assert_eq!(park.cash_cents(), cash - 300);
        assert!(
            park.path_benches().is_empty()
                && park.path_signs().is_empty()
                && park.path_fences().is_empty()
        );
        assert_eq!(park.park_value().decorations_cents, 0);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }

    #[test]
    fn schema_41_migrates_existing_paid_bench_and_42_requires_new_fields() {
        let mut park = Park::new(32, 32);
        let path = park.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
        park.toggle_path_bench(path).unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        let mut old = value.clone();
        old["schema_version"] = 41.into();
        old["content_version"] = 41.into();
        old["park"]["decorations"]
            .as_object_mut()
            .unwrap()
            .remove("signs");
        old["park"]["decorations"]
            .as_object_mut()
            .unwrap()
            .remove("fences");
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&old).unwrap()),
            Ok(park)
        );
        value["park"]["decorations"]
            .as_object_mut()
            .unwrap()
            .remove("signs");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn mixed_sign_and_fence_receipts_archive_and_reload() {
        let mut park = Park::new(32, 32);
        let path = park.place_path(TilePos { x: 5, y: 5 }, 0).unwrap();
        for _ in 0..65 {
            for kind in [PathAdditionKind::Sign, PathAdditionKind::Fence] {
                assert_eq!(park.toggle_path_addition(path, kind), Ok(true));
                assert_eq!(park.toggle_path_addition(path, kind), Ok(false));
            }
        }
        assert!(park.finance_ledger().archived_entries > 0);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
            park
        );
    }
}
