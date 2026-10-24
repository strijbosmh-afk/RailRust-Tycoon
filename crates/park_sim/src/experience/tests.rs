use super::*;
use crate::{PathDirection, PathSlope, RideVisitPhase, Speed, TilePos};
use serde_json::{Value, json};

pub(crate) fn fixture(exit: bool, price: u32) -> Park {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    park.install_train_fixture().unwrap();
    let ride = park.ride().unwrap().id;
    park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, ride)
        .unwrap();
    for (y, h, slope) in [
        (12, 8, PathSlope::Flat),
        (13, 8, PathSlope::South),
        (14, 9, PathSlope::Flat),
        (15, 8, PathSlope::North),
    ] {
        park.place_queue_path(TilePos { x: 8, y }, h, slope, ride)
            .unwrap();
    }
    if exit {
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride)
            .unwrap();
        for x in 8..=13 {
            park.place_path(TilePos { x, y: 8 }, 8).unwrap();
        }
        for y in 9..=11 {
            park.place_path(TilePos { x: 13, y }, 8).unwrap();
        }
    }
    park.set_ride_ticket_price(ride, price).unwrap();
    park
}
fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
    for _ in 0..24_000 {
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!(
        "feedback event not reached: {:?} {:?}",
        park.ride(),
        park.train()
    );
}
fn phase(park: &Park, phase: RideVisitPhase) -> Option<GuestId> {
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
fn guest(park: &Park, id: GuestId) -> &Guest {
    park.guests().iter().find(|guest| guest.id == id).unwrap()
}
fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(&loaded, park);
    assert_eq!(loaded.to_save_bytes().unwrap(), bytes);
    loaded
}

#[test]
fn preferences_are_seeded_diverse_stable_and_do_not_draw_navigation_rng() {
    let mut park = fixture(false, 600);
    let original = park.clone();
    let rng = park.crowd_rng;
    let mut prices = std::collections::BTreeSet::new();
    let mut waits = std::collections::BTreeSet::new();
    for guest in park.guests() {
        assert_eq!(guest.preferences, GuestPreferences::for_guest(42, guest.id));
        assert!(guest.willing_to_pay(guest.preferences.max_ticket_price_cents));
        assert!(!guest.willing_to_pay(guest.preferences.max_ticket_price_cents + 1));
        prices.insert(guest.preferences.max_ticket_price_cents);
        waits.insert(guest.preferences.comfortable_queue_ticks);
    }
    assert!(prices.len() >= 3 && waits.len() >= 3);
    let stats = park.ride_experience_stats(park.ride().unwrap().id);
    assert!(stats.willing_guests > 0 && stats.willing_guests < 12);
    assert_eq!(stats.interested_guests, stats.willing_guests);
    assert_eq!(stats.average_rating_tenths, None);
    assert_eq!(
        park.guest_experience_summary(),
        GuestExperienceSummary {
            average_satisfaction: Some(70),
            unhappy_guests: 0,
            guest_count: 12
        }
    );
    park.advance(Speed::Paused);
    assert_eq!(park, original);
    assert_eq!(park.crowd_rng, rng);
    roundtrip(&park);
}

#[test]
fn value_refusal_is_distinct_from_wallet_refusal_and_does_not_repeat_happiness_penalties() {
    let mut park = fixture(false, 1100);
    let ride = park.ride().unwrap().id;
    seek(&mut park, |park| park.ride().unwrap().value_rejections > 0);
    assert_eq!(park.ride().unwrap().unaffordable_rejections, 0);
    assert_eq!(park.ride().unwrap().joined, 0);
    assert!(
        park.guests()
            .iter()
            .any(|guest| guest.thought == GuestThought::TooExpensive)
    );
    assert!(park.guests().iter().all(|guest| guest.satisfaction == 70));
    let rejected = park.ride().unwrap().value_rejections;
    park.set_ride_ticket_price(ride, 2100).unwrap();
    seek(&mut park, |park| {
        park.ride().unwrap().unaffordable_rejections > 0
    });
    assert_eq!(park.ride().unwrap().value_rejections, rejected);
    assert!(
        park.guests()
            .iter()
            .any(|guest| guest.thought == GuestThought::NotEnoughMoney)
    );
    assert!(
        park.guests()
            .iter()
            .all(|guest| guest.satisfaction == 70 && guest.spent_cents == 0)
    );
    park.set_ride_ticket_price(ride, 200).unwrap();
    seek(&mut park, |park| park.ride().unwrap().joined > 0);
    roundtrip(&park);
}

#[test]
fn wait_discomfort_and_joined_cancellation_apply_once_including_paused_commands() {
    let mut park = fixture(false, 0);
    seek(&mut park, |park| {
        park.guests()
            .iter()
            .any(|guest| guest.thought == GuestThought::WaitingTooLong)
    });
    let id = park
        .guests()
        .iter()
        .find(|guest| guest.thought == GuestThought::WaitingTooLong)
        .unwrap()
        .id;
    assert_eq!(guest(&park, id).satisfaction, 65);
    let thought_tick = guest(&park, id).thought_tick;
    for _ in 0..30 {
        park.advance(Speed::Normal);
    }
    assert_eq!(guest(&park, id).satisfaction, 65);
    assert_eq!(guest(&park, id).thought_tick, thought_tick);
    let points: Vec<_> = park
        .guests()
        .iter()
        .map(|guest| park.guest_position(guest.id))
        .collect();
    park.set_train_open(false);
    assert_eq!(guest(&park, id).satisfaction, 57);
    assert_eq!(guest(&park, id).thought, GuestThought::RideCancelled);
    assert_eq!(
        park.guests()
            .iter()
            .map(|guest| park.guest_position(guest.id))
            .collect::<Vec<_>>(),
        points
    );
    let closed = park.clone();
    park.set_train_open(false);
    park.advance(Speed::Paused);
    assert_eq!(park, closed);
    seek(&mut park, |park| guest(park, id).ride_visit.is_none());
    assert_eq!(guest(&park, id).satisfaction, 57);
    assert_eq!(guest(&park, id).ratings_given, 0);
    roundtrip(&park);
}

#[test]
fn completed_public_return_produces_one_rating_from_locked_fare_and_wait() {
    let mut park = fixture(true, 200);
    seek(&mut park, |park| {
        phase(park, RideVisitPhase::Exiting).is_some()
    });
    let id = phase(&park, RideVisitPhase::Exiting).unwrap();
    let prior = guest(&park, id).clone();
    let visit = prior.ride_visit.as_ref().unwrap();
    assert!(visit.passenger.as_ref().unwrap().completed_lap);
    let fare = visit.passenger.as_ref().unwrap().fare_cents;
    let price_penalty = if fare <= prior.preferences.max_ticket_price_cents / 2 {
        0
    } else if fare <= prior.preferences.max_ticket_price_cents {
        1
    } else {
        2
    };
    let score = (5
        - price_penalty
        - (visit.waited_ticks / prior.preferences.comfortable_queue_ticks).min(2))
    .max(1) as u8;
    let count = park.ride().unwrap().rating_count;
    let cash = park.cash_cents();
    let mut rating_satisfaction = prior.satisfaction;
    let mut levels = [prior.needs.hunger, prior.needs.thirst, prior.needs.toilet];
    for offset in 1..=u64::from(crate::RIDE_TRANSFER_TICKS) {
        for (value, cadence) in levels.iter_mut().zip([90, 60, 120]) {
            if (park.ticks() + offset).is_multiple_of(cadence) {
                if *value == 84 {
                    rating_satisfaction = rating_satisfaction.saturating_sub(5);
                }
                *value = value.saturating_add(1).min(100);
            }
        }
    }
    for _ in 1..crate::RIDE_TRANSFER_TICKS {
        park.advance(Speed::Normal);
        assert_eq!(park.ride().unwrap().rating_count, count);
    }
    park.advance(Speed::Normal);
    let returned = guest(&park, id);
    assert!(returned.ride_visit.is_none());
    assert_eq!(returned.ratings_given, prior.ratings_given + 1);
    assert_eq!(returned.last_rating, Some(score));
    assert_eq!(returned.rating_total, prior.rating_total + u64::from(score));
    assert_eq!(
        returned.satisfaction,
        ((3 * u16::from(rating_satisfaction) + 20 * u16::from(score)) / 4) as u8
    );
    assert_eq!(returned.thought_tick, park.ticks());
    assert_eq!(
        returned.thought,
        if score >= 3 {
            GuestThought::EnjoyedRide
        } else {
            GuestThought::DisappointedByRide
        }
    );
    assert_eq!(park.ride().unwrap().rating_count, count + 1);
    assert_eq!(park.cash_cents(), cash);
    let mut loaded = roundtrip(&park);
    for _ in 0..100 {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
        assert_eq!(park, loaded);
    }
    assert_eq!(guest(&park, id).ratings_given, prior.ratings_given + 1);
}

#[test]
fn predeparture_cancellation_changes_satisfaction_once_without_review_or_charge() {
    let mut park = fixture(true, 200);
    seek(&mut park, |park| {
        phase(park, RideVisitPhase::Boarding).is_some()
    });
    let id = phase(&park, RideVisitPhase::Boarding).unwrap();
    let before = guest(&park, id).satisfaction;
    park.set_train_open(false);
    seek(&mut park, |park| guest(park, id).ride_visit.is_none());
    assert_eq!(guest(&park, id).satisfaction, before.saturating_sub(8));
    assert_eq!(guest(&park, id).thought, GuestThought::RideCancelled);
    assert_eq!(guest(&park, id).ratings_given, 0);
    assert_eq!(park.ride().unwrap().rating_count, 0);
    assert_eq!(park.ride().unwrap().revenue_cents, 0);
    roundtrip(&park);
}

#[test]
fn higher_price_at_queue_front_drains_physically_and_recovery_can_earn_reviews() {
    let mut park = fixture(true, 0);
    let ride = park.ride().unwrap().id;
    seek(&mut park, |park| park.ride_queue_stats(ride).queueing == 4);
    park.set_ride_ticket_price(ride, 1100).unwrap();
    seek(&mut park, |park| {
        park.guests().iter().any(|guest| {
            guest.thought == GuestThought::TooExpensive
                && guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == RideVisitPhase::Leaving)
        })
    });
    assert!(park.ride().unwrap().value_rejections > 0);
    assert_eq!(park.ride().unwrap().unaffordable_rejections, 0);
    assert_eq!(park.ride().unwrap().boarded, 0);
    roundtrip(&park);
    seek(&mut park, |park| !park.ride_queue_stats(ride).draining);
    park.set_ride_ticket_price(ride, 200).unwrap();
    seek(&mut park, |park| park.ride().unwrap().rating_count > 0);
    roundtrip(&park);
}

#[test]
fn full_schedules_save_resume_and_speed_grouping_preserve_feedback_exactly() {
    let mut normal = fixture(true, 200);
    let mut fast = normal.clone();
    let mut fastest = normal.clone();
    for group in 0..3600 {
        if [100, 400, 700].contains(&group) {
            let price = if group == 100 {
                600
            } else if group == 400 {
                1100
            } else {
                200
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
        if group % 600 == 0 {
            fastest = roundtrip(&fastest);
        }
    }
    assert!(normal.ride().unwrap().rating_count > 0);
    assert!(normal.ride().unwrap().value_rejections > 0);
    assert_eq!(
        normal.ride().unwrap().rating_count,
        normal
            .guests()
            .iter()
            .map(|guest| guest.ratings_given)
            .sum::<u64>()
    );
    roundtrip(&normal);
}

pub(crate) fn strip_feedback(value: Value) -> Value {
    let mut value = crate::services::tests::strip_services(value);
    value["schema_version"] = json!(6);
    value["content_version"] = json!(6);
    for guest in value["park"]["guests"].as_array_mut().unwrap() {
        for field in [
            "preferences",
            "satisfaction",
            "thought",
            "thought_tick",
            "ratings_given",
            "rating_total",
            "last_rating",
        ] {
            guest.as_object_mut().unwrap().remove(field);
        }
    }
    if let Some(ride) = value["park"]["ride"].as_object_mut() {
        for field in ["value_rejections", "rating_count", "rating_total"] {
            ride.remove(field);
        }
    }
    value
}

pub(crate) fn reset_feedback(park: &mut Park) {
    crate::services::tests::reset_needs(park);
    for guest in &mut park.guests {
        guest.preferences = GuestPreferences::for_guest(park.crowd_seed.unwrap_or(0), guest.id);
        guest.satisfaction = 70;
        guest.thought = GuestThought::Exploring;
        guest.thought_tick = 0;
        guest.ratings_given = 0;
        guest.rating_total = 0;
        guest.last_rating = None;
        for history in &mut guest.ride_history {
            history.rating_count = 0;
            history.rating_total = 0;
        }
    }
    if let Some(ride) = &mut park.ride {
        ride.value_rejections = 0;
        ride.rating_count = 0;
        ride.rating_total = 0;
    }
}

#[test]
fn v6_migration_preserves_every_old_field_and_starts_only_prospective_feedback() {
    let mut source = fixture(true, 200);
    seek(&mut source, |park| park.ride().unwrap().revenue_cents > 0);
    let old = strip_feedback(serde_json::from_slice(&source.to_save_bytes().unwrap()).unwrap());
    let bytes = serde_json::to_vec(&old).unwrap();
    let mut first = Park::from_save_bytes(&bytes).unwrap();
    let mut second = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(
        strip_feedback(serde_json::from_slice(&first.to_save_bytes().unwrap()).unwrap()),
        old
    );
    assert_eq!(first.crowd_rng, source.crowd_rng);
    for guest in first.guests() {
        assert_eq!(guest.preferences, GuestPreferences::for_guest(42, guest.id));
        assert_eq!(guest.satisfaction, 70);
        assert_eq!(guest.thought, GuestThought::Exploring);
        assert_eq!(guest.thought_tick, 0);
        assert_eq!(guest.ratings_given, 0);
        assert_eq!(guest.last_rating, None);
    }
    for _ in 0..6000 {
        first.advance(Speed::Normal);
        second.advance(Speed::Normal);
        assert_eq!(first, second);
    }
    assert!(first.ride().unwrap().rating_count > 0);
    roundtrip(&first);
    for (path, field, value) in [
        ("/park/guests/0", "satisfaction", json!(70)),
        ("/park/ride", "rating_count", json!(0)),
    ] {
        let mut forged = old.clone();
        forged.pointer_mut(path).unwrap()[field] = value;
        assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
    }
}

#[test]
fn invalid_preferences_feedback_and_rating_aggregates_are_rejected() {
    let mut park = fixture(true, 200);
    seek(&mut park, |park| park.ride().unwrap().rating_count > 0);
    let base: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    let index = park
        .guests()
        .iter()
        .position(|guest| guest.ratings_given > 0)
        .unwrap();
    let stem = format!("/park/guests/{index}");
    for (path, value) in [
        (
            format!("{stem}/preferences/max_ticket_price_cents"),
            json!(500),
        ),
        (
            format!("{stem}/preferences/comfortable_queue_ticks"),
            json!(0),
        ),
        (format!("{stem}/satisfaction"), json!(101)),
        (format!("{stem}/thought"), json!("Unknown")),
        (format!("{stem}/thought_tick"), json!(park.ticks() + 1)),
        (format!("{stem}/ratings_given"), json!(u64::MAX)),
        (format!("{stem}/rating_total"), json!(u64::MAX)),
        (format!("{stem}/last_rating"), json!(0)),
        (format!("{stem}/last_rating"), Value::Null),
        ("/park/ride/rating_count".into(), json!(u64::MAX)),
    ] {
        let mut forged = base.clone();
        *forged.pointer_mut(&path).unwrap() = value;
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err(),
            "accepted {path}"
        );
    }
    for field in [
        "preferences",
        "satisfaction",
        "thought",
        "thought_tick",
        "ratings_given",
        "rating_total",
        "last_rating",
    ] {
        let mut forged = base.clone();
        forged
            .pointer_mut(&stem)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
    }
    for pair in [(16, 16), (8, 7), (7, 8), (7, 6), (6, 7)] {
        let mut forged = base.clone();
        forged["schema_version"] = json!(pair.0);
        forged["content_version"] = json!(pair.1);
        assert!(Park::from_save_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
    }
    // Align the entire feedback ledger first: each positive control must pass,
    // so changing only its latest score proves the history check is effective.
    for (count, total, valid_last, invalid_last) in [(1, 5, 5, 1), (2, 10, 5, 1), (2, 2, 1, 5)] {
        let mut control = park.clone();
        let guest = &mut control.guests[index];
        guest.ratings_given = count;
        guest.rides_completed = count;
        guest.rating_total = total;
        guest.last_rating = Some(valid_last);
        let history = guest.ride_history_mut(park.ride().unwrap().id);
        history.completed = count;
        history.rating_count = count;
        history.rating_total = total;
        let ride = control.ride.as_mut().unwrap();
        ride.rating_count = control.guests.iter().map(|guest| guest.ratings_given).sum();
        ride.rating_total = control.guests.iter().map(|guest| guest.rating_total).sum();
        ride.completed = control
            .guests
            .iter()
            .map(|guest| guest.rides_completed)
            .sum();
        assert_eq!(validate_experience(&control), Ok(()));
        control.guests[index].last_rating = Some(invalid_last);
        assert_eq!(
            validate_experience(&control),
            Err(if u64::from(invalid_last) > total {
                "guest preferences or feedback history is invalid"
            } else {
                "last guest rating disagrees with rating history"
            }
            .into())
        );
    }
    roundtrip(&park);
}

#[test]
fn full_rating_counter_prevents_new_reservations_without_overflow() {
    let mut park = fixture(true, 0);
    let owner = park.ride().unwrap().id;
    // Exercise the owner-bound capacity predicate with the same applicant in
    // both controls; a guest with no visit would be rejected for another reason.
    let with_applicants = |park: &Park| {
        let mut applicants = park.clone();
        for guest in &mut applicants.guests[..2] {
            guest.ride_visit = Some(crate::RideVisit {
                ride: owner,
                phase: crate::RideVisitPhase::Approaching,
                ticket: None,
                waited_ticks: 0,
                passenger: None,
            });
        }
        applicants
    };
    let available = with_applicants(&park);
    assert!(available.can_reserve_rating(0));
    assert!(available.can_reserve_rating(1));
    park.guests[0].rides_completed = MAX_RATINGS;
    park.guests[0].ratings_given = MAX_RATINGS;
    park.guests[0].rating_total = MAX_RATINGS * 5;
    park.guests[0].last_rating = Some(5);
    let ride = park.ride.as_mut().unwrap();
    ride.completed = MAX_RATINGS;
    ride.exited = MAX_RATINGS;
    ride.boarded = MAX_RATINGS;
    ride.joined = MAX_RATINGS;
    ride.next_queue_ticket = MAX_RATINGS + 1;
    ride.rating_count = MAX_RATINGS;
    ride.rating_total = MAX_RATINGS * 5;
    let id = ride.id;
    let history = park.guests[0].ride_history_mut(id);
    history.completed = MAX_RATINGS;
    history.rating_count = MAX_RATINGS;
    history.rating_total = MAX_RATINGS * 5;
    let full = with_applicants(&park);
    assert!(!full.can_reserve_rating(0));
    assert!(!full.can_reserve_rating(1));
    roundtrip(&park);
    for _ in 0..6000 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.ride().unwrap().rating_count, MAX_RATINGS);
    assert_eq!(park.ride().unwrap().rating_total, u64::MAX);
    roundtrip(&park);
}

#[test]
fn rating_formula_covers_price_boundaries_legacy_fares_and_queue_penalties() {
    let fixture = fixture(true, 0);
    for (fare, waited, expected) in [
        (300, 0, 5),
        (301, 0, 4),
        (600, 0, 4),
        (601, 0, 3),
        (300, 750, 4),
        (300, 1500, 3),
        (600, 1500, 2),
        (601, 1800, 1),
    ] {
        let mut guest = fixture.guests[0].clone();
        guest.preferences = GuestPreferences::default();
        let mut ride = fixture.ride().unwrap().clone();
        guest.rate_completed_trip(&mut ride, fare, waited, 42);
        assert_eq!(guest.last_rating, Some(expected));
        assert_eq!(guest.rating_total, u64::from(expected));
        assert_eq!(ride.rating_count, 1);
        assert_eq!(ride.rating_total, u64::from(expected));
        assert_eq!(
            guest.satisfaction,
            ((210 + u16::from(expected) * 20) / 4) as u8
        );
        assert_eq!(guest.thought_tick, 42);
        assert_eq!(
            guest.thought,
            if expected >= 3 {
                GuestThought::EnjoyedRide
            } else {
                GuestThought::DisappointedByRide
            }
        );
    }
}

#[test]
fn completed_reviews_are_attributed_to_the_owner_and_survive_idle_removal() {
    let mut park = fixture(true, 0);
    let coaster = park.ride().unwrap().id;
    let carousel = park.place_carousel(TilePos { x: 2, y: 24 }, 8).unwrap();
    for (owner, index, waited) in [(coaster, 0, 0), (carousel, 1, 1800)] {
        let (ride, guests) = park.ride_and_guests_mut(owner).unwrap();
        let guest = &mut guests[index];
        guest.rides_completed += 1;
        ride.completed += 1;
        guest.rate_completed_trip(ride, 0, waited, 0);
    }
    validate_experience(&park).unwrap();
    assert_eq!(park.ride_experience_stats(coaster).rating_total, 5);
    assert_eq!(park.ride_experience_stats(carousel).rating_count, 1);
    let historical = park.ride_experience_stats(carousel);
    park.remove_carousel(carousel).unwrap();
    assert_eq!(park.ride_experience_stats(carousel), historical);
    validate_experience(&park).unwrap();
    let mut forged = park.clone();
    forged.guests[0].ride_history[0].ride = carousel;
    assert!(validate_experience(&forged).is_err());
}
