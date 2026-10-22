//! Economy snapshots use real fare commands and naturally reached guest states.
use super::*;
use park_sim::{INITIAL_PARK_CASH_CENTS, MAX_TICKET_PRICE_CENTS, PathSlope, RideVisitPhase};

const SEEK_LIMIT: usize = 18_000;

fn check(park: &Park) {
    crate::save_check::economy::require_accounting(park)
        .expect("economy capture must conserve money");
}

fn roundtrip(park: &Park) {
    check(park);
    assert_eq!(
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
        *park
    );
}

fn price(park: &mut Park, cents: u32) {
    crate::save_check::economy::set_price_checked(park, cents)
        .expect("economy capture price must preserve locked fares");
    roundtrip(park);
}

fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool, label: &str) {
    for _ in 0..SEEK_LIMIT {
        if predicate(park) {
            return;
        }
        crate::save_check::economy::advance_checked(park)
            .expect("economy capture must charge each locked fare exactly once");
    }
    panic!("economy capture did not reach {label} within {SEEK_LIMIT} ticks");
}

fn reservation(park: &Park) -> bool {
    park.guests().iter().any(|guest| {
        guest.ride_visit.as_ref().is_some_and(|visit| {
            visit.phase == RideVisitPhase::Boarding
                && visit.passenger.as_ref().is_some_and(|passenger| {
                    passenger.fare_cents == 300
                        && !passenger.fare_paid
                        && !passenger.departed
                        && passenger
                            .transfer
                            .as_ref()
                            .is_some_and(|transfer| transfer.progress == 15)
                })
        })
    })
}

fn prepare_state(park: &mut Park, view: &mut ViewState, scene: Scene) {
    assert!(scene.is_economy());
    let ids: Vec<_> = park.guests().iter().map(|guest| guest.id).collect();
    let circuit = park.circuit().cloned();
    let ride = journey_scenes::install(park, view);
    price(
        park,
        match scene {
            Scene::EconomyUnaffordable => MAX_TICKET_PRICE_CENTS,
            Scene::EconomyFree => 0,
            _ => 300,
        },
    );
    match scene {
        Scene::EconomyReserved | Scene::EconomyPriceChange | Scene::EconomyCancelled => {
            seek(park, reservation, "an unpaid locked fare during boarding");
            assert_eq!(park.ride_economy_stats(ride).revenue_cents, 0);
            assert_eq!(park.cash_cents(), INITIAL_PARK_CASH_CENTS);
            if matches!(scene, Scene::EconomyPriceChange) {
                price(park, 700);
                assert!(reservation(park));
                assert_eq!(park.ride_economy_stats(ride).reserved_fares_cents, 300);
            } else if matches!(scene, Scene::EconomyCancelled) {
                let wallets: Vec<_> = park
                    .guests()
                    .iter()
                    .map(|guest| (guest.wallet_cents, guest.spent_cents))
                    .collect();
                park.set_train_open(false);
                roundtrip(park);
                seek(
                    park,
                    |park| {
                        let stats = park.ride_journey_stats(ride);
                        stats.exited > 0 && stats.boarding + stats.riding + stats.exiting == 0
                    },
                    "predeparture public return without a charge",
                );
                assert_eq!(
                    park.guests()
                        .iter()
                        .map(|guest| (guest.wallet_cents, guest.spent_cents))
                        .collect::<Vec<_>>(),
                    wallets
                );
                assert_eq!(park.ride_journey_stats(ride).completed, 0);
                assert_eq!(park.ride_economy_stats(ride).revenue_cents, 0);
                assert_eq!(park.ride_economy_stats(ride).reserved_fares_cents, 0);
            }
        }
        Scene::EconomyEarned => {
            seek(
                park,
                |park| {
                    park.train()
                        .is_some_and(|train| train.speed > 0 && train.phase == TrainPhase::Lift)
                        && park.ride_economy_stats(ride).revenue_cents > 0
                        && park.guests().iter().any(|guest| {
                            guest.ride_visit.as_ref().is_some_and(|visit| {
                                visit.phase == RideVisitPhase::Riding
                                    && visit.passenger.as_ref().is_some_and(|passenger| {
                                        passenger.fare_paid && passenger.fare_cents > 0
                                    })
                            })
                        })
                },
                "paid passengers on the moving lift",
            );
        }
        Scene::EconomyUnaffordable => {
            seek(
                park,
                |park| park.ride_economy_stats(ride).unaffordable_rejections > 0,
                "an actual insufficient-wallet rejection",
            );
            let stats = park.ride_economy_stats(ride);
            assert_eq!(stats.affordable_guests, 0);
            assert_eq!(stats.revenue_cents, 0);
            assert_eq!(stats.reserved_fares_cents, 0);
            assert_eq!(park.cash_cents(), INITIAL_PARK_CASH_CENTS);
        }
        Scene::EconomyFree => {
            seek(
                park,
                |park| park.ride_journey_stats(ride).completed > 0,
                "a completed free trip",
            );
            assert_eq!(park.ride_economy_stats(ride).revenue_cents, 0);
            assert_eq!(park.cash_cents(), INITIAL_PARK_CASH_CENTS);
        }
        _ => unreachable!(),
    }
    assert_eq!(
        park.guests()
            .iter()
            .map(|guest| guest.id)
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(park.circuit(), circuit.as_ref());
    assert_eq!(park.ride().unwrap().id, ride);
    roundtrip(park);
}

fn present(park: &Park, view: &mut ViewState, scene: Scene) {
    view.path_tool = ToolState::default();
    view.selected = None;
    view.hovered = None;
    view.speed = Speed::Paused;
    view.path_notice = Some(
        match scene {
            Scene::EconomyReserved => "Fare reserved. The guest pays only when the train departs.",
            Scene::EconomyPriceChange => {
                "New price applies to future reservations. Existing fare is unchanged."
            }
            Scene::EconomyEarned => {
                "Train departed. Locked fares became ride revenue and park cash."
            }
            Scene::EconomyCancelled => "Closed before departure. Guests return without paying.",
            Scene::EconomyUnaffordable => "Guests cannot afford this price. No money was charged.",
            Scene::EconomyFree => "Free ride completed. Wallets and park cash are unchanged.",
            _ => unreachable!(),
        }
        .into(),
    );
    let center = if matches!(scene, Scene::EconomyEarned) {
        let train = park.train().unwrap();
        let piece = park.circuit().unwrap().pieces[train.piece_index].object;
        world::surface_center(park, SurfaceId::Object(piece), view.rotation).unwrap()
    } else {
        world::path_build_center(
            TilePos { x: 8, y: 10 },
            park,
            view.rotation,
            8,
            PathSlope::Flat,
        )
        .unwrap()
    };
    view.zoom = 2.2;
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    prepare_state(park, view, scene);
    present(park, view, scene);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn economy_scenes_reach_real_accounting_states_and_presentation_cannot_move_money() {
        for terrain in [TerrainProfile::Flat, TerrainProfile::Terraced] {
            for scene in [
                Scene::EconomyReserved,
                Scene::EconomyPriceChange,
                Scene::EconomyEarned,
                Scene::EconomyCancelled,
                Scene::EconomyUnaffordable,
                Scene::EconomyFree,
            ] {
                let mut park = crate::demonstration_park();
                park.try_apply_terrain_profile(terrain).unwrap();
                let mut view = ViewState::default();
                prepare_state(&mut park, &mut view, scene);
                let state = park.clone();
                let bytes = park.to_save_bytes().unwrap();
                for rotation in 0..4 {
                    for mode in [ViewMode::Surface, ViewMode::Underground] {
                        view.rotation = rotation;
                        view.view_mode = mode;
                        present(&park, &mut view, scene);
                        assert_eq!(park, state);
                        assert_eq!(park.to_save_bytes().unwrap(), bytes);
                        assert_eq!(view.speed, Speed::Paused);
                        assert!(view.pan.is_finite());
                        assert!(view.path_tool.preview.is_none());
                        assert!(view.save_pending.is_none());
                    }
                }
            }
        }
    }
}
