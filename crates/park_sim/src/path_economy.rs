//! Paid player path construction. Older fixture and migrated paths have no paid basis.
use crate::{
    FinanceKind, ObjectId, ObjectKind, Park, PathSlope, PlacementError, RideAccessError, RideId,
    TilePos,
};

pub const PUBLIC_PATH_COST_CENTS: u64 = 100;
pub const QUEUE_PATH_COST_CENTS: u64 = 200;
pub const SLOPE_SURCHARGE_CENTS: u64 = 50;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaidPath {
    pub object: ObjectId,
    pub cost_cents: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PathEconomy {
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,{crate::saves::MAX_PATHS}>")]
    pub paid: Vec<PaidPath>,
    pub construction_cents: u64,
    pub refund_cents: u64,
}

pub const fn path_cost_cents(queue: bool, slope: PathSlope) -> u64 {
    (if queue {
        QUEUE_PATH_COST_CENTS
    } else {
        PUBLIC_PATH_COST_CENTS
    }) + (if matches!(slope, PathSlope::Flat) {
        0
    } else {
        SLOPE_SURCHARGE_CENTS
    })
}

impl Park {
    pub fn paid_paths(&self) -> &[PaidPath] {
        &self.path_economy.paid
    }
    pub fn path_construction_total_cents(&self) -> u64 {
        self.path_economy.construction_cents
    }
    pub fn path_refund_total_cents(&self) -> u64 {
        self.path_economy.refund_cents
    }

    fn afford_path(&self, cost: u64) -> Result<(), PlacementError> {
        if self.cash_cents < cost {
            return Err(PlacementError::InsufficientFunds {
                required_cents: cost,
                available_cents: self.cash_cents,
            });
        }
        Ok(())
    }

    pub fn validate_paid_path(
        &self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
    ) -> Result<u64, PlacementError> {
        self.validate_sloped_path(pos, height, slope)?;
        let cost = path_cost_cents(false, slope);
        self.afford_path(cost)?;
        Ok(cost)
    }

    pub fn build_paid_path(
        &mut self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
    ) -> Result<ObjectId, PlacementError> {
        let cost = self.validate_paid_path(pos, height, slope)?;
        let id = self.place_sloped_path(pos, height, slope)?;
        self.record_path_purchase(id, cost);
        Ok(id)
    }

    pub fn validate_paid_queue_path(
        &self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
        ride: RideId,
    ) -> Result<u64, RideAccessError> {
        self.validate_queue_path(pos, height, slope, ride)?;
        let cost = path_cost_cents(true, slope);
        self.afford_path(cost)?;
        Ok(cost)
    }

    pub fn build_paid_queue_path(
        &mut self,
        pos: TilePos,
        height: u8,
        slope: PathSlope,
        ride: RideId,
    ) -> Result<ObjectId, RideAccessError> {
        let cost = self.validate_paid_queue_path(pos, height, slope, ride)?;
        let id = self.place_queue_path(pos, height, slope, ride)?;
        self.record_path_purchase(id, cost);
        Ok(id)
    }

    fn record_path_purchase(&mut self, id: ObjectId, cost: u64) {
        let index = self
            .path_economy
            .paid
            .partition_point(|record| record.object < id);
        self.path_economy.paid.insert(
            index,
            PaidPath {
                object: id,
                cost_cents: cost,
            },
        );
        self.path_economy.construction_cents += cost;
        self.record_cash(FinanceKind::PathConstruction, None, cost);
    }

    pub fn path_demolition_refund_cents(&self, id: ObjectId) -> u64 {
        self.path_economy
            .paid
            .binary_search_by_key(&id, |record| record.object)
            .ok()
            .map_or(0, |index| self.path_economy.paid[index].cost_cents / 2)
    }

    pub(crate) fn refund_removed_path(&mut self, id: ObjectId) {
        let Ok(index) = self
            .path_economy
            .paid
            .binary_search_by_key(&id, |record| record.object)
        else {
            return;
        };
        let refund = self.path_economy.paid.remove(index).cost_cents / 2;
        self.path_economy.refund_cents += refund;
        self.record_cash(FinanceKind::PathRefund, None, refund);
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    let state = &park.path_economy;
    let mut previous = None;
    let mut active = 0_u64;
    for record in &state.paid {
        if previous.is_some_and(|id| id >= record.object) {
            return Err("paid path IDs are not strictly ordered".into());
        }
        let object = park
            .object(record.object)
            .ok_or("paid path object is missing")?;
        if object.kind != ObjectKind::Path
            || record.cost_cents != path_cost_cents(object.queue_owner.is_some(), object.path_slope)
        {
            return Err("paid path identity or cost is invalid".into());
        }
        active = active
            .checked_add(record.cost_cents)
            .ok_or("paid path cost overflow")?;
        previous = Some(record.object);
    }
    if state.construction_cents < active
        || state.construction_cents - active != state.refund_cents.saturating_mul(2)
        || state.refund_cents > state.construction_cents / 2
        || state.construction_cents
            > park
                .finances
                .next_entry_id
                .saturating_sub(1)
                .saturating_mul(250)
    {
        return Err("paid path purchases or refunds are invalid".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paid_public_and_queue_paths_charge_and_refund_once_after_reload() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let ride = park.ride().unwrap().id;
        let cash = park.cash_cents();
        let public = park
            .build_paid_path(TilePos { x: 4, y: 4 }, 8, PathSlope::Flat)
            .unwrap();
        let queue = park
            .build_paid_queue_path(TilePos { x: 5, y: 4 }, 8, PathSlope::East, ride)
            .unwrap();
        assert_eq!(cash - park.cash_cents(), 350);
        assert_eq!(park.path_demolition_refund_cents(public), 50);
        assert_eq!(park.path_demolition_refund_cents(queue), 125);
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(loaded.paid_paths().len(), 2);
        loaded.remove_object(public).unwrap();
        loaded.remove_object(queue).unwrap();
        assert_eq!(loaded.cash_cents(), cash - 175);
        assert_eq!(loaded.path_demolition_refund_cents(public), 0);
        assert_eq!(
            Park::from_save_bytes(&loaded.to_save_bytes().unwrap()),
            Ok(loaded)
        );
    }

    #[test]
    fn insufficient_funds_and_collision_preserve_entire_park() {
        let mut park = Park::new(32, 32);
        park.cash_cents = 0;
        let before = park.clone();
        assert!(matches!(
            park.build_paid_path(TilePos { x: 4, y: 4 }, 8, PathSlope::Flat),
            Err(PlacementError::InsufficientFunds { .. })
        ));
        assert_eq!(park, before);
        park.cash_cents = 1_000;
        let path = park
            .build_paid_path(TilePos { x: 4, y: 4 }, 8, PathSlope::Flat)
            .unwrap();
        let before = park.clone();
        assert!(
            park.build_paid_path(TilePos { x: 4, y: 4 }, 8, PathSlope::Flat)
                .is_err()
        );
        assert_eq!(park, before);
        assert_eq!(park.paid_paths()[0].object, path);
    }

    #[test]
    fn schema_43_paths_remain_free_and_schema_44_requires_a_basis_field() {
        let mut old = Park::new(32, 32);
        let original = old.place_path(TilePos { x: 4, y: 4 }, 8).unwrap();
        let mut document: serde_json::Value =
            serde_json::from_slice(&old.to_save_bytes().unwrap()).unwrap();
        document["schema_version"] = 43.into();
        document["content_version"] = 43.into();
        document["park"]
            .as_object_mut()
            .unwrap()
            .remove("path_economy");
        old = Park::from_save_bytes(&serde_json::to_vec(&document).unwrap()).unwrap();
        assert_eq!(old.path_demolition_refund_cents(original), 0);
        let paid = old
            .build_paid_path(TilePos { x: 5, y: 4 }, 8, PathSlope::Flat)
            .unwrap();
        assert_eq!(old.path_demolition_refund_cents(paid), 50);
        assert_eq!(
            Park::from_save_bytes(&old.to_save_bytes().unwrap()),
            Ok(old.clone())
        );
        document["schema_version"] = 44.into();
        document["content_version"] = 44.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&document).unwrap()).is_err());
    }

    #[test]
    fn archived_path_receipts_still_reconcile_with_removed_objects() {
        let mut park = Park::new(32, 32);
        let cash = park.cash_cents();
        let paths: Vec<_> = (0..80)
            .map(|index| {
                park.build_paid_path(
                    TilePos {
                        x: index % 20 + 2,
                        y: index / 20 + 2,
                    },
                    8,
                    PathSlope::Flat,
                )
                .unwrap()
            })
            .collect();
        for id in paths {
            park.remove_object(id).unwrap();
        }
        assert_eq!(park.cash_cents(), cash - 4_000);
        assert!(park.finance_ledger().archived_entries > 0);
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()),
            Ok(park)
        );
    }
}
