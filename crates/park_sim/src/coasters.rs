//! Independent coaster records with a selected construction workspace.
use crate::{Park, Ride, RideId, TrackEconomy, TrackPieceSpec, Train, TrainCircuit};

pub const MAX_COASTER_RECORDS: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoasterState {
    pub(crate) ride: Option<Ride>,
    pub(crate) circuit: Option<TrainCircuit>,
    pub(crate) train: Option<Train>,
    pub(crate) extra_trains: Vec<Train>,
    pub(crate) track_draft: Vec<TrackPieceSpec>,
    pub(crate) track_economy: TrackEconomy,
    pub(crate) cars: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PathDirection, PathSlope, Speed, TilePos};

    fn pair() -> (Park, RideId, RideId) {
        let mut source = Park::new(32, 32);
        source.install_train_fixture().unwrap();
        let specs = source.circuit().unwrap().specs().to_vec();
        let mut park = Park::new(64, 64);
        park.borrow_loan(100_000).unwrap();
        park.replace_track_draft(&specs).unwrap();
        park.apply_track_draft().unwrap();
        let first = park.ride().unwrap().id;
        park.new_coaster_draft().unwrap();
        let shifted: Vec<_> = specs
            .into_iter()
            .map(|mut p| {
                p.pos.x += 30;
                p
            })
            .collect();
        park.replace_track_draft(&shifted).unwrap();
        park.apply_track_draft().unwrap();
        let second = park.ride().unwrap().id;
        (park, first, second)
    }

    #[test]
    fn independently_built_coasters_keep_money_settings_motion_and_saved_drafts() {
        let (mut park, first, second) = pair();
        assert_eq!(park.coaster_ids(), vec![first, second]);
        let first_cost = park.coaster_economy(first).unwrap().construction_cents;
        let second_cost = park.coaster_economy(second).unwrap().construction_cents;
        assert!(first_cost > 0);
        assert_eq!(first_cost, second_cost);
        assert_eq!(
            park.ride_cashflow(first).unwrap().construction_cents,
            first_cost
        );
        park.set_coaster_car_count(first, 1).unwrap();
        assert_eq!(park.coaster_car_count(first), Some(1));
        assert_eq!(park.coaster_car_count(second), Some(3));
        assert!(park.test_coaster_for(first).unwrap().input.matches(&park));
        park.set_ride_open(first, true).unwrap();
        for _ in 0..300 {
            park.advance(Speed::Normal);
        }
        assert!(park.coaster_train(first).unwrap().speed > 0);
        assert_eq!(park.coaster_train(second).unwrap().speed, 0);
        assert_eq!(park.ride().unwrap().id, second);
        let bytes = park.to_save_bytes().unwrap();
        let mut loaded = Park::from_save_bytes(&bytes).unwrap();
        assert_eq!(loaded, park);
        assert_eq!(loaded.to_save_bytes().unwrap(), bytes);
        for _ in 0..600 {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
        }
        assert_eq!(park, loaded);
        let second_track = park.coaster_circuit(second).unwrap().clone();
        let first_specs = park.coaster_circuit(first).unwrap().specs().to_vec();
        let remnant = park.coaster_circuit(first).unwrap().pieces[2].object;
        let removed = park.coaster_circuit(first).unwrap().pieces[1].object;
        let refund = park.track_demolition_refund_cents(removed);
        let cash = park.cash_cents();
        park.remove_object(removed).unwrap();
        assert_eq!(park.cash_cents(), cash + refund);
        assert!(park.coaster_circuit(first).is_none());
        assert_eq!(park.coaster_circuit(second), Some(&second_track));
        assert_eq!(park.object_ride(remnant), Some(first));
        park.select_coaster(first).unwrap();
        assert!(matches!(
            park.ride_opening_status(second),
            Err(crate::RideOpeningError::Entrance(_))
        ));
        park.new_coaster_draft().unwrap();
        let before = park.clone();
        assert!(park.install_train_circuit(&first_specs).is_err());
        assert_eq!(park, before);
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    }

    #[test]
    fn two_coasters_accept_independent_guests_and_keep_departure_accounting_after_reload() {
        let (mut park, first, second) = pair();
        let mut entry = None;
        for x in 0..=49 {
            let path = park.place_path(TilePos { x, y: 16 }, 8).unwrap();
            entry.get_or_insert(path);
        }
        for (offset, id) in [(0, first), (30, second)] {
            park.place_ride_entrance(
                TilePos {
                    x: 8 + offset,
                    y: 11,
                },
                8,
                PathDirection::North,
                id,
            )
            .unwrap();
            for y in 12..=15 {
                park.place_queue_path(TilePos { x: 8 + offset, y }, 8, PathSlope::Flat, id)
                    .unwrap();
            }
            park.place_ride_exit(
                TilePos {
                    x: 8 + offset,
                    y: 9,
                },
                8,
                PathDirection::South,
                id,
            )
            .unwrap();
            for x in 8..=13 {
                park.place_path(
                    TilePos {
                        x: x + offset,
                        y: 8,
                    },
                    8,
                )
                .unwrap();
            }
            for y in 9..=15 {
                park.place_path(TilePos { x: 13 + offset, y }, 8).unwrap();
            }
            park.set_ride_ticket_price(id, 100).unwrap();
            park.set_ride_open_checked(id, true).unwrap();
        }
        park.set_visitor_entry(entry.unwrap()).unwrap();
        park.set_visitor_arrivals(true).unwrap();
        for _ in 0..30_000 {
            park.advance(Speed::Normal);
            if park.ride_by_id(first).unwrap().completed > 0
                && park.ride_by_id(second).unwrap().completed > 0
            {
                break;
            }
        }
        for id in [first, second] {
            assert!(park.ride_by_id(id).unwrap().completed > 0, "ride {id:?}");
            assert!(park.ride_by_id(id).unwrap().revenue_cents > 0);
        }
        let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        for _ in 0..600 {
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
        }
        assert_eq!(park, loaded);
    }

    #[test]
    fn empty_workspace_and_legacy_schema_boundary_are_preserved() {
        let (mut park, first, _) = pair();
        park.new_coaster_draft().unwrap();
        assert!(park.ride().is_none());
        let bytes = park.to_save_bytes().unwrap();
        assert_eq!(Park::from_save_bytes(&bytes).unwrap(), park);
        park.select_coaster(first).unwrap();
        park.new_coaster_draft().unwrap();
        assert!(park.ride().is_none());
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["schema_version"] = 44.into();
        value["content_version"] = 44.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
        value["schema_version"] = 45.into();
        value["content_version"] = 45.into();
        value["park"]
            .as_object_mut()
            .unwrap()
            .remove("other_coasters");
        assert!(Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    }
}

impl Default for CoasterState {
    fn default() -> Self {
        Self {
            ride: None,
            circuit: None,
            train: None,
            extra_trains: Vec::new(),
            track_draft: Vec::new(),
            track_economy: TrackEconomy::default(),
            cars: crate::DEFAULT_COASTER_CARS,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoasterSelectionError {
    UnknownRide,
    Capacity,
    UnbuiltDraft,
}

impl std::fmt::Display for CoasterSelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UnknownRide => "Choose an existing coaster",
            Self::Capacity => "The park has reached its 32 tracked ride records",
            Self::UnbuiltDraft => {
                "Clear or build the current draft before starting another coaster"
            }
        })
    }
}

impl std::error::Error for CoasterSelectionError {}

impl Park {
    pub fn coaster_ids(&self) -> Vec<RideId> {
        let mut ids: Vec<_> = self
            .ride
            .iter()
            .chain(self.other_coasters.iter().filter_map(|c| c.ride.as_ref()))
            .map(|ride| ride.id)
            .collect();
        ids.sort_unstable();
        ids
    }

    pub fn coaster_circuit(&self, id: RideId) -> Option<&TrainCircuit> {
        if self.ride.as_ref().is_some_and(|ride| ride.id == id) {
            self.circuit.as_ref()
        } else {
            self.other_coasters
                .iter()
                .find(|c| c.ride.as_ref().is_some_and(|ride| ride.id == id))?
                .circuit
                .as_ref()
        }
    }

    pub fn coaster_train(&self, id: RideId) -> Option<&Train> {
        if self.ride.as_ref().is_some_and(|ride| ride.id == id) {
            self.train.as_ref()
        } else {
            self.other_coasters
                .iter()
                .find(|c| c.ride.as_ref().is_some_and(|ride| ride.id == id))?
                .train
                .as_ref()
        }
    }

    pub(crate) fn coaster_train_mut(&mut self, id: RideId) -> Option<&mut Train> {
        if self.ride.as_ref().is_some_and(|ride| ride.id == id) {
            self.train.as_mut()
        } else {
            self.other_coasters
                .iter_mut()
                .find(|c| c.ride.as_ref().is_some_and(|ride| ride.id == id))?
                .train
                .as_mut()
        }
    }

    pub fn coaster_economy(&self, id: RideId) -> Option<&TrackEconomy> {
        if self.ride.as_ref().is_some_and(|ride| ride.id == id) {
            Some(&self.track_economy)
        } else {
            self.other_coasters
                .iter()
                .find(|c| c.ride.as_ref().is_some_and(|ride| ride.id == id))
                .map(|c| &c.track_economy)
        }
    }

    pub fn select_coaster(&mut self, id: RideId) -> Result<bool, CoasterSelectionError> {
        if self.ride.as_ref().is_some_and(|ride| ride.id == id) {
            return Ok(false);
        }
        let index = self
            .other_coasters
            .iter()
            .position(|c| c.ride.as_ref().is_some_and(|ride| ride.id == id))
            .ok_or(CoasterSelectionError::UnknownRide)?;
        self.swap_coaster(index);
        // An unbuilt workspace is retained so selecting another ride never
        // silently discards a draft. It is reused by New coaster.
        Ok(true)
    }

    pub fn new_coaster_draft(&mut self) -> Result<bool, CoasterSelectionError> {
        if self.ride.is_none() {
            return if self.track_draft.is_empty() {
                Ok(false)
            } else {
                Err(CoasterSelectionError::UnbuiltDraft)
            };
        }
        if let Some(index) = self.other_coasters.iter().position(|c| c.ride.is_none()) {
            self.swap_coaster(index);
            return Ok(true);
        }
        if self.coaster_ids().len() >= MAX_COASTER_RECORDS {
            return Err(CoasterSelectionError::Capacity);
        }
        self.other_coasters.push(CoasterState::default());
        self.swap_coaster(self.other_coasters.len() - 1);
        Ok(true)
    }

    /// Run existing selected-coaster operations against one record and restore
    /// the construction selection. No clock, guest or financial state is copied.
    pub(crate) fn with_coaster<T>(&mut self, id: RideId, action: impl FnOnce(&mut Self) -> T) -> T {
        let index = self
            .other_coasters
            .iter()
            .position(|c| c.ride.as_ref().is_some_and(|ride| ride.id == id));
        if let Some(index) = index {
            self.swap_coaster(index);
            let result = action(self);
            self.swap_coaster(index);
            result
        } else {
            action(self)
        }
    }

    pub(crate) fn swap_coaster(&mut self, index: usize) {
        let other = &mut self.other_coasters[index];
        std::mem::swap(&mut self.ride, &mut other.ride);
        std::mem::swap(&mut self.circuit, &mut other.circuit);
        std::mem::swap(&mut self.train, &mut other.train);
        std::mem::swap(&mut self.extra_trains, &mut other.extra_trains);
        std::mem::swap(&mut self.track_draft, &mut other.track_draft);
        std::mem::swap(&mut self.track_economy, &mut other.track_economy);
        std::mem::swap(&mut self.coaster_cars, &mut other.cars);
    }

    pub(crate) fn step_coasters(&mut self) {
        for id in self.coaster_ids() {
            self.with_coaster(id, Self::step_coaster_fleet);
        }
    }
}
