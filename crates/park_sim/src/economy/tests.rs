use super::*;
use crate::{GuestId, PathDirection, PathSlope, RideVisitPhase, Speed, TilePos};
use serde_json::{Value, json};

fn fixture(price: u32) -> Park {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    park.install_train_fixture().unwrap();
    let ride = park.ride().unwrap().id;
    park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, ride)
        .unwrap();
    for (y, height, slope) in [
        (12, 8, PathSlope::Flat),
        (13, 8, PathSlope::South),
        (14, 9, PathSlope::Flat),
        (15, 8, PathSlope::North),
    ] {
        park.place_queue_path(TilePos { x: 8, y }, height, slope, ride)
            .unwrap();
    }
    park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride)
        .unwrap();
    for x in 8..=13 {
        park.place_path(TilePos { x, y: 8 }, 8).unwrap();
    }
    for y in 9..=11 {
        park.place_path(TilePos { x: 13, y }, 8).unwrap();
    }
    park.set_ride_ticket_price(ride, price).unwrap();
    park
}

fn assert_ledger(park: &Park) {
    let spent: u64 = park
        .guests()
        .iter()
        .map(|guest| {
            assert_eq!(
                u64::from(guest.wallet_cents) + guest.spent_cents,
                u64::from(INITIAL_GUEST_WALLET_CENTS)
            );
            if let Some(passenger) = guest
                .ride_visit
                .as_ref()
                .and_then(|visit| visit.passenger.as_ref())
            {
                assert_eq!(passenger.fare_paid, passenger.departed);
                if !passenger.fare_paid {
                    assert!(passenger.fare_cents <= guest.wallet_cents);
                }
            }
            guest.spent_cents
        })
        .sum();
    assert_eq!(park.ride().map_or(0, |ride| ride.revenue_cents), spent);
    assert_eq!(park.cash_cents(), INITIAL_PARK_CASH_CENTS + spent);
    validate_economy(park).unwrap();
}
fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
    for _ in 0..12_000 {
        assert_ledger(park);
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!(
        "economy state not reached: {:?} {:?}",
        park.ride(),
        park.train()
    );
}
fn member(park: &Park, phase: RideVisitPhase) -> Option<GuestId> {
    park.guests()
        .iter()
        .find(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.phase == phase)
        })
        .map(|guest| guest.id)
}
fn occupied(park: &Park) -> bool {
    park.guests().iter().any(|guest| {
        guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| visit.passenger.is_some())
    })
}
fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(bytes, loaded.to_save_bytes().unwrap());
    loaded
}

#[test]
fn departure_settlement_is_once_per_owner_and_preserves_another_reserved_fare() {
    // Isolate the settlement boundary after both runtimes report departure.
    // Journey/pose validity is covered by the full natural-state integration proof.
    let mut park = fixture(0);
    let coaster = park.ride().unwrap().id;
    let carousel = park.place_carousel(TilePos { x: 2, y: 24 }, 8).unwrap();
    park.ticks = 1;
    for (index, owner, fare, departed) in [
        (0, coaster, 300, true),
        (1, carousel, 200, true),
        (2, coaster, 400, false),
    ] {
        park.guests[index].ride_visit = Some(crate::RideVisit {
            ride: owner,
            phase: RideVisitPhase::Riding,
            ticket: Some(1),
            waited_ticks: 0,
            passenger: Some(crate::PassengerJourney {
                seat: 0,
                departed,
                completed_lap: false,
                fare_cents: fare,
                fare_paid: false,
                transfer: None,
            }),
        });
    }
    let mut reversed = park.clone();
    reversed.guests.reverse();
    park.settle_departure_fares();
    reversed.settle_departure_fares();
    reversed.guests.reverse();
    assert_eq!(park, reversed);
    assert_eq!(park.guests[0].wallet_cents, 1700);
    assert_eq!(park.guests[1].wallet_cents, 1800);
    assert_eq!(park.guests[2].wallet_cents, 2000);
    assert_eq!(
        park.cash_cents(),
        INITIAL_PARK_CASH_CENTS - crate::CAROUSEL_BUILD_COST_CENTS + 500
    );
    assert_eq!(park.ride_economy_stats(coaster).revenue_cents, 300);
    assert_eq!(park.ride_economy_stats(carousel).revenue_cents, 200);
    assert_eq!(park.ride_economy_stats(coaster).reserved_fares_cents, 400);
    assert_eq!(
        park.finance_ledger()
            .entries
            .iter()
            .map(|e| (e.kind, e.ride, e.amount_cents))
            .collect::<Vec<_>>(),
        vec![
            (
                crate::FinanceKind::RideConstruction,
                Some(carousel),
                crate::CAROUSEL_BUILD_COST_CENTS
            ),
            (crate::FinanceKind::RideRevenue, Some(coaster), 300),
            (crate::FinanceKind::RideRevenue, Some(carousel), 200),
        ]
    );
    validate_economy(&park).unwrap();
    crate::finances::validate_finances(&park).unwrap();
    let once = park.clone();
    park.settle_departure_fares();
    assert_eq!(park, once);
    let mut forged = park.clone();
    forged.guests[0].ride_history[0].ride = carousel;
    assert!(validate_economy(&forged).is_err());
    let mut forged = park.clone();
    forged.finances.entries[1].ride = Some(carousel);
    assert_eq!(
        crate::finances::validate_finances(&forged),
        Err("ride income disagrees with attributed cash history".into())
    );
}

#[test]
fn price_commands_are_bounded_atomic_paused_and_independent_of_path_cache() {
    let mut park = fixture(0);
    park.advance(Speed::Normal);
    let before = park.clone();
    let cache = park.path_cache_stats();
    let ride = park.ride().unwrap().id;
    assert_eq!(
        park.set_ride_ticket_price(ride, MAX_TICKET_PRICE_CENTS + 1),
        Err(EconomyError::PriceTooHigh {
            price: MAX_TICKET_PRICE_CENTS + 1,
            max: MAX_TICKET_PRICE_CENTS
        })
    );
    assert_eq!(
        park.set_ride_ticket_price(RideId(999), 0),
        Err(EconomyError::UnknownRide(RideId(999)))
    );
    assert_eq!(park, before);
    assert_eq!(park.path_cache_stats(), cache);
    park.set_ride_ticket_price(ride, MAX_TICKET_PRICE_CENTS)
        .unwrap();
    let mut expected = before;
    expected.ride.as_mut().unwrap().ticket_price_cents = MAX_TICKET_PRICE_CENTS;
    assert_eq!(park, expected);
    park.advance(Speed::Paused);
    assert_eq!(park, expected);
    assert_eq!(park.path_cache_stats(), cache);
    assert_eq!(park.ride_economy_stats(ride).affordable_guests, 0);
    park.set_ride_ticket_price(ride, 0).unwrap();
    assert_eq!(park.ride_economy_stats(ride).affordable_guests, 12);
    roundtrip(&park);
}

#[test]
fn boarding_locks_fare_and_departure_collects_it_exactly_once() {
    let mut park = fixture(250);
    seek(&mut park, |park| {
        member(park, RideVisitPhase::Boarding).is_some()
    });
    let guest = member(&park, RideVisitPhase::Boarding).unwrap();
    let ride = park.ride().unwrap().id;
    assert_eq!(park.ride_economy_stats(ride).reserved_fares_cents, 250);
    assert_eq!(park.cash_cents(), INITIAL_PARK_CASH_CENTS);
    let poses = park.ride_guest_positions();
    park.set_ride_ticket_price(ride, 500).unwrap();
    assert_eq!(park.ride_guest_positions(), poses);
    roundtrip(&park);
    seek(&mut park, |park| {
        park.train().unwrap().speed > 0 && park.ride().unwrap().revenue_cents > 0
    });
    let first = park.guests().iter().find(|g| g.id == guest).unwrap();
    assert_eq!(first.wallet_cents, 1_750);
    assert_eq!(first.spent_cents, 250);
    assert_eq!(
        first
            .ride_visit
            .as_ref()
            .unwrap()
            .passenger
            .as_ref()
            .unwrap()
            .fare_cents,
        250
    );
    assert!(
        first
            .ride_visit
            .as_ref()
            .unwrap()
            .passenger
            .as_ref()
            .unwrap()
            .fare_paid
    );
    assert_eq!(park.ride_economy_stats(ride).reserved_fares_cents, 0);
    let balances: Vec<_> = park
        .guests()
        .iter()
        .map(|g| (g.id, g.wallet_cents, g.spent_cents))
        .collect();
    let cash = park.cash_cents();
    park.set_train_open(false);
    seek(&mut park, |park| !occupied(park));
    assert_eq!(
        park.guests()
            .iter()
            .map(|g| (g.id, g.wallet_cents, g.spent_cents))
            .collect::<Vec<_>>(),
        balances
    );
    assert_eq!(park.cash_cents(), cash);
    assert!(park.ride().unwrap().completed > 0);
    roundtrip(&park);
}

#[test]
fn closing_before_departure_releases_reservation_without_any_debit() {
    for phase in [RideVisitPhase::Boarding, RideVisitPhase::Riding] {
        let mut park = fixture(350);
        seek(&mut park, |park| {
            member(park, phase).is_some() && park.train().unwrap().speed == 0
        });
        let ride = park.ride().unwrap().id;
        assert!(park.ride_economy_stats(ride).reserved_fares_cents > 0);
        let poses = park.ride_guest_positions();
        park.set_train_open(false);
        assert_eq!(poses, park.ride_guest_positions());
        let paused = park.clone();
        park.advance(Speed::Paused);
        assert_eq!(park, paused);
        roundtrip(&park);
        seek(&mut park, |park| !occupied(park));
        assert_eq!(park.ride().unwrap().boarded, park.ride().unwrap().exited);
        assert_eq!(park.ride().unwrap().completed, 0);
        assert_eq!(park.ride().unwrap().revenue_cents, 0);
        assert_eq!(park.cash_cents(), INITIAL_PARK_CASH_CENTS);
        assert_eq!(park.ride_economy_stats(ride).reserved_fares_cents, 0);
        assert!(park.guests().iter().all(
            |guest| guest.wallet_cents == INITIAL_GUEST_WALLET_CENTS && guest.spent_cents == 0
        ));
        roundtrip(&park);
    }
}

#[test]
fn reopening_before_departure_keeps_locked_fare_and_charges_only_on_motion() {
    let mut park = fixture(225);
    seek(&mut park, |park| {
        member(park, RideVisitPhase::Boarding).is_some()
    });
    let guest = member(&park, RideVisitPhase::Boarding).unwrap();
    let poses = park.ride_guest_positions();
    park.set_train_open(false);
    park.set_ride_ticket_price(park.ride().unwrap().id, 500)
        .unwrap();
    park.set_train_open(true);
    assert_eq!(poses, park.ride_guest_positions());
    assert_eq!(park.cash_cents(), INITIAL_PARK_CASH_CENTS);
    let mut loaded = roundtrip(&park);
    for _ in 0..3_600 {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
        assert_eq!(park, loaded);
        assert_ledger(&park);
        if park
            .guests()
            .iter()
            .find(|g| g.id == guest)
            .unwrap()
            .spent_cents
            > 0
        {
            break;
        }
    }
    assert_eq!(
        park.guests()
            .iter()
            .find(|g| g.id == guest)
            .unwrap()
            .spent_cents,
        225
    );
}

#[test]
fn unaffordable_seekers_and_queued_fronts_leave_without_paying_and_can_retry() {
    let mut expensive = fixture(MAX_TICKET_PRICE_CENTS);
    seek(&mut expensive, |park| {
        park.ride().unwrap().unaffordable_rejections > 0
    });
    assert!(!occupied(&expensive));
    assert_eq!(expensive.ride().unwrap().joined, 0);
    assert_eq!(expensive.cash_cents(), INITIAL_PARK_CASH_CENTS);
    expensive
        .set_ride_ticket_price(expensive.ride().unwrap().id, 300)
        .unwrap();
    seek(&mut expensive, |park| {
        park.ride().unwrap().revenue_cents > 0
    });
    roundtrip(&expensive);

    let mut raised = fixture(0);
    seek(&mut raised, |park| {
        park.ride_queue_stats(park.ride().unwrap().id).queueing == 4
    });
    raised
        .set_ride_ticket_price(raised.ride().unwrap().id, MAX_TICKET_PRICE_CENTS)
        .unwrap();
    seek(&mut raised, |park| {
        park.ride_queue_stats(park.ride().unwrap().id).draining
    });
    assert!(raised.ride().unwrap().unaffordable_rejections > 0);
    assert_eq!(raised.ride().unwrap().boarded, 0);
    assert_eq!(raised.ride().unwrap().revenue_cents, 0);
    assert!(raised.guests().iter().any(|guest| {
        guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| visit.phase == RideVisitPhase::Leaving)
    }));
    roundtrip(&raised);
    seek(&mut raised, |park| {
        !park.ride_queue_stats(park.ride().unwrap().id).draining
    });
    raised
        .set_ride_ticket_price(raised.ride().unwrap().id, 200)
        .unwrap();
    seek(&mut raised, |park| park.ride().unwrap().revenue_cents > 0);
    roundtrip(&raised);
}

#[test]
fn spending_survives_fixture_reinstallation_fault_repair_and_exhausted_wallets() {
    let mut park = fixture(1000);
    // Willing guests can afford two real departures before their wallets empty.
    seek(&mut park, |park| {
        park.guests().iter().any(|guest| guest.wallet_cents == 0)
    });
    park.set_train_open(false);
    seek(&mut park, |park| !occupied(park));
    assert!(park.guests().iter().any(|guest| guest.wallet_cents == 0));
    let wallets: Vec<_> = park
        .guests()
        .iter()
        .map(|guest| (guest.id, guest.wallet_cents, guest.spent_cents))
        .collect();
    let ride = park.ride().unwrap().clone();
    let cash = park.cash_cents();
    park.install_crowd_fixture(999).unwrap();
    park.install_train_fixture().unwrap();
    let track = park.circuit().unwrap().pieces[5].object;
    park.remove_object(track).unwrap();
    roundtrip(&park);
    park.install_train_fixture().unwrap();
    assert_eq!(park.ride().unwrap(), &ride);
    assert_eq!(park.cash_cents(), cash);
    assert_eq!(
        park.guests()
            .iter()
            .map(|guest| (guest.id, guest.wallet_cents, guest.spent_cents))
            .collect::<Vec<_>>(),
        wallets
    );
    park.set_ride_ticket_price(ride.id, 1).unwrap();
    park.set_train_open(true);
    for _ in 0..3_600 {
        park.advance(Speed::Normal);
        assert_ledger(&park);
    }
    for (id, _, spent) in wallets.into_iter().filter(|(_, wallet, _)| *wallet == 0) {
        let guest = park.guests().iter().find(|guest| guest.id == id).unwrap();
        assert_eq!(guest.spent_cents, spent);
        assert_eq!(guest.wallet_cents, 0);
    }
    assert!(park.ride().unwrap().unaffordable_rejections > 0);
    roundtrip(&park);
}

#[test]
fn money_continuation_matches_complete_ticks_at_all_speeds() {
    let mut normal = fixture(250);
    let mut fast = normal.clone();
    let mut fastest = normal.clone();
    // The final affordable-price command occurs at tick 4,800. Retain enough
    // bounded time afterward for a fresh queue, all station transfers, a paid
    // lap and public return; this is journey coverage, not a timing benchmark.
    for group in 0..3_600 {
        if [300, 800, 1_200].contains(&group) {
            let price = if group == 300 {
                450
            } else if group == 800 {
                MAX_TICKET_PRICE_CENTS
            } else {
                150
            };
            for park in [&mut normal, &mut fast, &mut fastest] {
                park.set_ride_ticket_price(park.ride().unwrap().id, price)
                    .unwrap();
            }
        }
        if group == 500 || group == 600 {
            for park in [&mut normal, &mut fast, &mut fastest] {
                park.set_train_open(group == 600);
            }
        }
        for _ in 0..4 {
            normal.advance(Speed::Normal);
        }
        fast.advance(Speed::Fast);
        fast.advance(Speed::Fast);
        fastest.advance(Speed::Fastest);
        assert_eq!(normal, fast);
        assert_eq!(normal, fastest);
        assert_ledger(&normal);
        if group % 300 == 0 {
            fastest = roundtrip(&fastest);
        }
    }
    assert!(normal.ride().unwrap().revenue_cents > 0);
    assert!(
        normal.ride().unwrap().completed > 0,
        "paid public return missing after 14,400 ticks: {:?} {:?}",
        normal.ride(),
        normal.train()
    );
    assert!(normal.ride().unwrap().unaffordable_rejections > 0);
    roundtrip(&normal);
}

fn v5_value(park: &Park) -> Value {
    let mut value = crate::experience::tests::strip_feedback(
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap(),
    );
    value["schema_version"] = json!(5);
    value["content_version"] = json!(5);
    value["park"].as_object_mut().unwrap().remove("cash_cents");
    for guest in value["park"]["guests"].as_array_mut().unwrap() {
        assert_eq!(guest["spent_cents"], json!(0));
        guest.as_object_mut().unwrap().remove("wallet_cents");
        guest.as_object_mut().unwrap().remove("spent_cents");
        if let Some(passenger) = guest
            .pointer_mut("/ride_visit/passenger")
            .and_then(Value::as_object_mut)
        {
            assert_eq!(passenger["fare_cents"], json!(0));
            passenger.remove("fare_cents");
            passenger.remove("fare_paid");
        }
    }
    if let Some(ride) = value["park"]["ride"].as_object_mut() {
        for field in [
            "ticket_price_cents",
            "revenue_cents",
            "unaffordable_rejections",
        ] {
            ride.remove(field);
        }
    }
    value
}

#[test]
fn v5_passengers_migrate_free_with_exact_future_and_strict_old_shapes() {
    let mut source = fixture(0);
    for phase in [
        RideVisitPhase::Boarding,
        RideVisitPhase::Riding,
        RideVisitPhase::Exiting,
    ] {
        seek(&mut source, |park| member(park, phase).is_some());
        crate::experience::tests::reset_feedback(&mut source);
        let legacy = v5_value(&source);
        let mut loaded = Park::from_save_bytes(&serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert_eq!(loaded, source);
        let mut expected = source.clone();
        for park in [&mut expected, &mut loaded] {
            park.set_ride_ticket_price(park.ride().unwrap().id, 600)
                .unwrap();
        }
        for _ in 0..600 {
            expected.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
            assert_eq!(loaded, expected);
            assert_ledger(&loaded);
        }
        roundtrip(&loaded);
        for (path, field, value) in [
            ("/park", "cash_cents", json!(100000)),
            ("/park/ride", "revenue_cents", json!(0)),
            ("/park/guests/0", "wallet_cents", json!(2000)),
        ] {
            let mut invalid = legacy.clone();
            invalid.pointer_mut(path).unwrap()[field] = value;
            assert!(Park::from_save_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
        let passenger = source
            .guests()
            .iter()
            .position(|guest| {
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.passenger.is_some())
            })
            .unwrap();
        let mut invalid = legacy;
        invalid["park"]["guests"][passenger]["ride_visit"]["passenger"]["fare_cents"] = json!(0);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());
        source.advance(Speed::Normal);
    }
}

#[test]
fn balanced_spending_without_journey_history_is_rejected() {
    let mut park = fixture(300);
    for active in [false, true] {
        if active {
            seek(&mut park, |park| park.ride().unwrap().revenue_cents > 0);
        }
        let index = if active {
            park.guests()
                .iter()
                .position(|guest| {
                    guest
                        .ride_visit
                        .as_ref()
                        .and_then(|visit| visit.passenger.as_ref())
                        .is_some_and(|passenger| passenger.fare_paid)
                })
                .unwrap()
        } else {
            0
        };
        let guest = &park.guests()[index];
        assert_eq!(guest.rides_completed, 0);
        if !active {
            assert_eq!(park.ride().unwrap().boarded, 0);
        }
        let bytes = park.to_save_bytes().unwrap();
        let mut forged: Value = serde_json::from_slice(&bytes).unwrap();
        let extra = 100_u32;
        // Keep every money conservation equation balanced, but invent a paid
        // trip before any completed return (or extra spending on the first trip).
        forged["park"]["guests"][index]["wallet_cents"] = json!(guest.wallet_cents - extra);
        forged["park"]["guests"][index]["spent_cents"] =
            json!(guest.spent_cents + u64::from(extra));
        forged["park"]["ride"]["revenue_cents"] =
            json!(park.ride().unwrap().revenue_cents + u64::from(extra));
        forged["park"]["cash_cents"] = json!(park.cash_cents() + u64::from(extra));
        // Preserve the new owner totals as well, so the rejection specifically
        // proves the absence of a real completed/active paid trip.
        forged["park"]["guests"][index]["ride_history"] = json!([{
            "ride": park.ride().unwrap().id,
            "spent_cents": guest.spent_cents + u64::from(extra),
            "completed": 0,
            "rating_count": 0,
            "rating_total": 0
        }]);
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()),
            Err(crate::SaveError::InvalidState(
                "attributed spending has no completed or active paid ride".into()
            ))
        );
        assert_eq!(park.to_save_bytes().unwrap(), bytes);
        roundtrip(&park);
    }
}

#[test]
fn malformed_money_and_missing_v6_fields_are_rejected_without_mutation() {
    let mut park = fixture(300);
    seek(&mut park, |park| {
        member(park, RideVisitPhase::Boarding).is_some()
    });
    let before = park.clone();
    let base: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    let index = park
        .guests()
        .iter()
        .position(|guest| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.passenger.is_some())
        })
        .unwrap();
    let passenger = format!("/park/guests/{index}/ride_visit/passenger");
    for (path, value) in [
        ("/park/cash_cents".into(), json!(u64::MAX)),
        ("/park/ride/revenue_cents".into(), json!(1)),
        ("/park/ride/ticket_price_cents".into(), json!(10001)),
        ("/park/guests/0/wallet_cents".into(), json!(-1)),
        ("/park/guests/0/wallet_cents".into(), json!(u32::MAX)),
        ("/park/guests/0/spent_cents".into(), json!(u64::MAX)),
        (format!("{passenger}/fare_paid"), json!(true)),
        (format!("{passenger}/fare_cents"), json!(2001)),
        (format!("{passenger}/fare_cents"), json!(3.5)),
    ] {
        let mut invalid = base.clone();
        *invalid.pointer_mut(&path).unwrap() = value;
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err(),
            "accepted {path}"
        );
        assert_eq!(park, before);
    }
    for (path, field) in [
        ("/park".to_owned(), "cash_cents"),
        ("/park/ride".into(), "ticket_price_cents"),
        ("/park/ride".into(), "revenue_cents"),
        ("/park/ride".into(), "unaffordable_rejections"),
        ("/park/guests/0".into(), "wallet_cents"),
        ("/park/guests/0".into(), "spent_cents"),
        (passenger.clone(), "fare_cents"),
        (passenger, "fare_paid"),
    ] {
        let mut invalid = base.clone();
        invalid
            .pointer_mut(&path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err(),
            "accepted absent {field}"
        );
    }
    for (schema, content) in [
        (u32::MAX, u32::MAX),
        (16, 15),
        (15, 16),
        (8, 7),
        (7, 8),
        (7, 6),
        (6, 7),
        (6, 5),
        (5, 6),
    ] {
        let mut invalid = base.clone();
        invalid["schema_version"] = json!(schema);
        invalid["content_version"] = json!(content);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());
    }
}
