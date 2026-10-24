use super::*;
use crate::{GuestState, RideDispatchPolicy, RideOperatingSettings, Speed, TrackKind};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn fixture() -> (Park, RideId, RideId) {
    let (mut park, carousel) = crate::carousel::tests::fixture(true);
    let coaster = park.ride().unwrap().id;
    park.set_ride_open(coaster, false).unwrap();
    park.set_ride_open(carousel, false).unwrap();
    (park, coaster, carousel)
}

fn choices(park: &Park) -> BTreeMap<GuestId, RideId> {
    let graph = crate::crowds::PathGraph::new(&park.objects);
    let accesses = park
        .all_rides()
        .map(|ride| (ride.id, park.ride_access(ride.id)))
        .collect();
    park.choose_guest_rides(&graph, &accesses)
}

fn value(park: &Park) -> Value {
    serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap()
}

fn read(value: &Value) -> Result<Park, crate::SaveError> {
    Park::from_save_bytes(&serde_json::to_vec(value).unwrap())
}

fn saved(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(bytes, loaded.to_save_bytes().unwrap());
    loaded
}

fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
    for _ in 0..36_000 {
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!("appeal condition not reached");
}

fn lower_coaster(park: &mut Park) {
    park.copy_installed_track_to_draft().unwrap();
    let specs = park.circuit().unwrap().specs().to_vec();
    for mut spec in specs {
        spec.entry.height = spec.entry.height.min(10);
        spec.exit.height = spec.exit.height.min(10);
        if matches!(spec.kind, TrackKind::Lift | TrackKind::Drop)
            && spec.entry.height == spec.exit.height
        {
            spec.kind = TrackKind::Straight;
        }
        park.set_track_draft_piece(spec).unwrap();
    }
    park.apply_track_draft().unwrap();
}

#[test]
fn bounded_static_factors_and_neutral_prior_have_explicit_extremes() {
    assert_eq!(layout_challenge(CoasterLayoutMetrics::default()), 0);
    assert_eq!(
        layout_challenge(CoasterLayoutMetrics {
            pieces: 8,
            curves: 4,
            ..Default::default()
        }),
        20
    );
    assert_eq!(
        layout_challenge(CoasterLayoutMetrics {
            pieces: 100,
            curves: 100,
            longest_drop_steps: 12,
            ..Default::default()
        }),
        100
    );
    assert_eq!(
        layout_challenge(CoasterLayoutMetrics {
            pieces: 100,
            curves: 100,
            longest_drop_steps: u32::MAX,
            ..Default::default()
        }),
        100
    );
    assert_eq!(reputation(0, 0), 30);
    assert_eq!(reputation(1, 5), 33);
    assert_eq!(reputation(5, 25), 40);
    assert_eq!(reputation(5, 5), 20);
    assert_eq!(reputation(500, 2500), 49);
    let count = crate::experience::MAX_RATINGS;
    assert_eq!(reputation(count, count * 5), 49);
    assert_eq!(reputation(count, count), 10);
    let profile = RideAppealProfile {
        layout_challenge: 100,
        review_count: count,
        reputation_rating_tenths: 10,
        reputation_penalty: 40,
    };
    assert_eq!(profile.for_preference(25).layout_penalty, 150);
    assert_eq!(profile.for_preference(25).total_penalty, 190);
    assert_eq!(profile.for_preference(85).layout_penalty, 30);
}

#[test]
fn inspection_and_preferences_are_pure_deterministic_and_owner_specific() {
    let (park, coaster, carousel) = fixture();
    let before = park.clone();
    let bytes = park.to_save_bytes().unwrap();
    let cache = park.path_cache_stats();
    let rng = park.crowd_rng;
    assert_eq!(
        park.ride_appeal_profile(carousel).unwrap().layout_challenge,
        20
    );
    assert_eq!(
        park.ride_appeal_profile(coaster)
            .unwrap()
            .reputation_penalty,
        20
    );
    let mut bands = BTreeSet::new();
    for id in 1..=256 {
        bands.insert(preferred_challenge(42, GuestId(id)));
    }
    assert_eq!(bands, BTreeSet::from([25, 45, 65, 85]));
    for guest in park.guests() {
        let preference = park.guest_preferred_challenge(guest.id).unwrap();
        assert!(bands.contains(&preference));
        for ride in [coaster, carousel] {
            let explanation = park.guest_ride_appeal(guest.id, ride).unwrap();
            assert_eq!(explanation.profile, park.ride_appeal_profile(ride).unwrap());
            assert_eq!(explanation.preferred_challenge, preference);
            assert_eq!(
                explanation.total_penalty,
                explanation.layout_penalty + u16::from(explanation.profile.reputation_penalty)
            );
        }
    }
    assert_eq!(park.guest_preferred_challenge(GuestId(999)), None);
    assert_eq!(park.guest_ride_appeal(GuestId(999), coaster), None);
    assert_eq!(park.ride_appeal_profile(RideId(999)), None);
    assert_eq!(park, before);
    assert_eq!(park.crowd_rng, rng);
    assert_eq!(park.path_cache_stats(), cache);
    assert_eq!(park.to_save_bytes().unwrap(), bytes);
    let restored = saved(&park);
    for guest in park.guests() {
        assert_eq!(
            park.guest_ride_appeal(guest.id, coaster),
            restored.guest_ride_appeal(guest.id, coaster)
        );
    }
}

#[test]
fn profiles_follow_only_installed_geometry_and_earned_reviews_not_test_or_settings() {
    let (mut park, coaster, carousel) = fixture();
    let initial = park.ride_appeal_profile(coaster).unwrap();
    assert_eq!(initial, park.ride_appeal_profile(coaster).unwrap());
    park.test_coaster().unwrap();
    park.set_ride_operating_settings(
        coaster,
        RideOperatingSettings {
            station_wait_ticks: 900,
            lift_speed: 12,
            brake_speed: 8,
            ..Default::default()
        },
    )
    .unwrap();
    park.set_ride_dispatch_policy(
        coaster,
        RideDispatchPolicy {
            minimum_passengers: 6,
            maximum_extra_wait_ticks: 1800,
        },
    )
    .unwrap();
    park.copy_installed_track_to_draft().unwrap();
    let removed = park.track_draft()[3].pos;
    park.remove_track_draft_piece(removed);
    assert_eq!(park.ride_appeal_profile(coaster), Some(initial));
    lower_coaster(&mut park);
    let lowered = park.ride_appeal_profile(coaster).unwrap();
    assert!(lowered.layout_challenge < initial.layout_challenge);
    assert_eq!(lowered.review_count, initial.review_count);
    saved(&park);
    let rail = park.circuit().unwrap().pieces[3].object;
    park.remove_object(rail).unwrap();
    assert_eq!(park.ride_appeal_profile(coaster), None);
    park.remove_carousel(carousel).unwrap();
    assert!(park.ride_by_id(carousel).is_some());
    assert_eq!(park.ride_appeal_profile(carousel), None);
    saved(&park);
}

#[test]
fn changed_layout_causally_changes_a_fresh_choice_without_new_rng_draws() {
    let (mut high, coaster, carousel) = fixture();
    let mut low = high.clone();
    lower_coaster(&mut low);
    assert_eq!(high.ride_access(coaster), low.ride_access(coaster));
    assert_eq!(
        high.ride_exit_access(coaster),
        low.ride_exit_access(coaster)
    );
    assert_eq!(high.guests(), low.guests());
    for park in [&mut high, &mut low] {
        park.set_ride_open(coaster, true).unwrap();
        park.set_ride_open(carousel, true).unwrap();
    }
    let eligible_guest = high
        .guests
        .iter()
        .position(|guest| {
            high.guest_preferred_challenge(guest.id) == Some(25)
                && high.guest_preferred_ride_kind(guest.id) == Some(RideKind::Coaster)
        })
        .expect("fixture includes a coaster fan with a low challenge preference");
    let guest_id = high.guests[eligible_guest].id;
    let paths: Vec<_> = high
        .objects
        .iter()
        .filter(|object| object.is_public_path())
        .copied()
        .collect();
    let mut changed = false;
    // Controlled candidate-ranking fixture: keep guest identity, preferences,
    // wallet, route topology and both prices equal between the two layouts.
    'search: for path in paths {
        for price in (0..=400).step_by(25) {
            for park in [&mut high, &mut low] {
                park.set_ride_ticket_price(coaster, price).unwrap();
                park.set_ride_ticket_price(carousel, 200).unwrap();
                let guest = &mut park.guests[eligible_guest];
                guest.current = Some(path.id);
                guest.anchor = path.pos;
                guest.height = path.height;
                guest.anchor_slope = path.path_slope;
                guest.wait_ticks = 0;
                guest.state = GuestState::Waiting;
            }
            let before_high = high.clone();
            let before_low = low.clone();
            let selected_high = choices(&high).get(&guest_id).copied();
            let selected_low = choices(&low).get(&guest_id).copied();
            assert_eq!(high, before_high);
            assert_eq!(low, before_low);
            if selected_high == Some(carousel) && selected_low == Some(coaster) {
                changed = true;
                break 'search;
            }
        }
    }
    assert!(
        changed,
        "lower installed challenge must change an actual fresh ranking"
    );
    assert_eq!(high.crowd_rng, low.crowd_rng);
    assert_eq!(high.ticks(), low.ticks());
    saved(&high);
    saved(&low);
}

#[test]
fn affordability_access_and_single_ride_admission_precede_soft_appeal() {
    let (mut park, coaster, carousel) = fixture();
    for guest in &mut park.guests {
        guest.wait_ticks = 0;
    }
    park.set_ride_open(coaster, true).unwrap();
    park.set_ride_open(carousel, true).unwrap();
    park.set_ride_ticket_price(carousel, crate::MAX_TICKET_PRICE_CENTS)
        .unwrap();
    let selected = choices(&park);
    assert!(!selected.is_empty() && selected.values().all(|&ride| ride == coaster));
    park.set_ride_ticket_price(carousel, 100).unwrap();
    park.set_ride_ticket_price(coaster, crate::MAX_TICKET_PRICE_CENTS)
        .unwrap();
    let selected = choices(&park);
    assert!(!selected.is_empty() && selected.values().all(|&ride| ride == carousel));
    // Closing the affordable alternative retains the previous refusal policy:
    // the sole reachable open ride may be approached and refused downstream.
    park.set_ride_open(carousel, false).unwrap();
    assert!(choices(&park).values().all(|&ride| ride == coaster));
    park.set_ride_ticket_price(coaster, 100).unwrap();
    let selected = choices(&park);
    assert!(!selected.is_empty() && selected.values().all(|&ride| ride == coaster));
    assert!(selected.keys().any(|guest| {
        park.guest_ride_appeal(*guest, coaster)
            .unwrap()
            .layout_penalty
            > 0
    }));
    let gate = park.ride_by_id(coaster).unwrap().entrance.unwrap().object;
    park.remove_object(gate).unwrap();
    assert!(choices(&park).is_empty());
    saved(&park);
}

#[test]
fn real_reviews_change_reputation_and_inspection_never_changes_active_fares_or_visits() {
    let (mut park, coaster, carousel) = fixture();
    park.set_ride_open(carousel, true).unwrap();
    seek(&mut park, |park| {
        park.ride_by_id(carousel).unwrap().rating_count >= 5
    });
    let ride = park.ride_by_id(carousel).unwrap();
    assert!(ride.revenue_cents > 0 && ride.completed >= 5);
    let profile = park.ride_appeal_profile(carousel).unwrap();
    assert_eq!(profile.review_count, ride.rating_count);
    assert!(profile.reputation_rating_tenths > 30 && profile.reputation_penalty < 20);
    assert_eq!(park.ride_appeal_profile(coaster).unwrap().review_count, 0);
    seek(&mut park, |park| {
        park.guests.iter().any(|guest| {
            guest
                .ride_visit
                .as_ref()
                .and_then(|visit| visit.passenger.as_ref())
                .is_some_and(|passenger| passenger.fare_paid)
        })
    });
    let before = park.clone();
    let bytes = park.to_save_bytes().unwrap();
    let selected = choices(&park);
    for guest in &park.guests {
        park.guest_ride_appeal(guest.id, carousel).unwrap();
        if guest.ride_visit.is_some() || guest.service_visit.is_some() {
            assert!(!selected.contains_key(&guest.id));
        }
    }
    assert_eq!(park, before);
    assert_eq!(park.to_save_bytes().unwrap(), bytes);
    let mut restored = saved(&park);
    for _ in 0..1800 {
        park.advance(Speed::Normal);
        restored.advance(Speed::Normal);
        assert_eq!(park, restored);
    }
    saved(&park);
}

#[test]
fn earned_reputation_alone_breaks_a_fresh_choice_tie_without_mutating_either_park() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct ChoiceFactors {
        route: std::collections::VecDeque<crate::ObjectId>,
        queue: usize,
        fare: u32,
        wallet: u32,
        willing: bool,
        family_mismatch: bool,
        completed: u64,
        challenge: u8,
        preferred_challenge: u8,
        layout_penalty: u16,
        reputation_penalty: u8,
    }
    impl ChoiceFactors {
        fn score(&self) -> u128 {
            self.route.len() as u128 * 10
                + self.queue as u128 * 10
                + u128::from(self.fare / 25)
                + u128::from(self.family_mismatch) * 80
                + u128::from(self.completed) * 20
                + u128::from(self.layout_penalty)
                + u128::from(self.reputation_penalty)
        }
    }
    fn factors(park: &Park, target: GuestId, owner: RideId) -> ChoiceFactors {
        let guest = park.guests.iter().find(|guest| guest.id == target).unwrap();
        let ride = park.ride_by_id(owner).unwrap();
        let access = park.ride_access(owner);
        let queue = park.ride_queue_stats(owner);
        assert!(queue.admitting);
        assert_eq!(queue.queueing, 0);
        let graph = crate::crowds::PathGraph::new(&park.objects);
        let appeal = park.guest_ride_appeal(target, owner).unwrap();
        ChoiceFactors {
            route: graph
                .route(guest.current.unwrap(), access.public_entry.unwrap())
                .unwrap(),
            queue: queue.queueing,
            fare: ride.ticket_price_cents,
            wallet: guest.wallet_cents,
            willing: guest.willing_to_pay(ride.ticket_price_cents),
            family_mismatch: park.ride_kind(owner) != park.guest_preferred_ride_kind(target),
            completed: guest
                .ride_history
                .iter()
                .find(|history| history.ride == owner)
                .map_or(0, |history| history.completed),
            challenge: appeal.profile.layout_challenge,
            preferred_challenge: appeal.preferred_challenge,
            layout_penalty: appeal.layout_penalty,
            reputation_penalty: appeal.profile.reputation_penalty,
        }
    }

    let (unrated, coaster, carousel) = fixture();
    assert!(
        coaster < carousel,
        "exact ties must favor the older coaster ID"
    );
    let fresh_template = unrated.guests[0].clone();
    let mut reviewed = unrated.clone();
    reviewed.set_ride_open(carousel, true).unwrap();
    seek(&mut reviewed, |park| {
        let ride = park.ride_by_id(carousel).unwrap();
        ride.rating_count > 0
            && ride.revenue_cents > 0
            && park
                .ride_appeal_profile(carousel)
                .unwrap()
                .reputation_penalty
                < 20
    });
    reviewed.set_ride_open(carousel, false).unwrap();
    seek(&mut reviewed, |park| {
        !park.ride_is_running(carousel)
            && park.guests.iter().all(|guest| guest.ride_visit.is_none())
    });
    // All reviews and revenue above came through the normal trip/exit rules.
    // Other guests' histories and park finances may differ; none is an input
    // to this fresh target's ranking once both owner queues are empty.
    assert_eq!(unrated.objects, reviewed.objects);
    assert_eq!(unrated.circuit, reviewed.circuit);
    assert_eq!(unrated.crowd_seed, reviewed.crowd_seed);
    assert_eq!(reviewed.ride_by_id(coaster).unwrap().rating_count, 0);
    saved(&unrated);
    saved(&reviewed);
    let graph = crate::crowds::PathGraph::new(&unrated.objects);
    let entries = [coaster, carousel].map(|owner| unrated.ride_access(owner).public_entry.unwrap());
    let paths: Vec<_> = unrated
        .objects
        .iter()
        .filter(|object| {
            object.is_public_path()
                && entries
                    .iter()
                    .all(|entry| graph.route(object.id, *entry).is_some())
        })
        .copied()
        .collect();
    let mut matched = None;
    'search: for raw_id in unrated.next_guest_id..unrated.next_guest_id + 16 {
        let target = GuestId(raw_id);
        for path in &paths {
            let mut neutral = unrated.clone();
            let mut earned = reviewed.clone();
            let mut guest = fresh_template.clone();
            guest.id = target;
            guest.current = Some(path.id);
            guest.anchor = path.pos;
            guest.height = path.height;
            guest.anchor_slope = path.path_slope;
            guest.wait_ticks = 0;
            guest.preferences =
                crate::GuestPreferences::for_guest(unrated.crowd_seed.unwrap(), target);
            guest.needs = crate::GuestNeeds::for_guest(unrated.crowd_seed.unwrap(), target);
            assert!(guest.ride_history.is_empty() && guest.ride_visit.is_none());
            for park in [&mut neutral, &mut earned] {
                park.guests.push(guest.clone());
                park.next_guest_id = raw_id + 1;
                for owner in [coaster, carousel] {
                    park.set_ride_ticket_price(owner, 0).unwrap();
                    park.set_ride_open(owner, true).unwrap();
                }
            }
            let coaster_score = factors(&neutral, target, coaster).score();
            let carousel_score = factors(&neutral, target, carousel).score();
            let difference = coaster_score.abs_diff(carousel_score);
            // At most $4 changes, in whole score-point steps. Every seeded
            // guest accepts this range; both branches receive identical fares.
            if difference > 16 {
                continue;
            }
            let owner_to_raise = if coaster_score < carousel_score {
                coaster
            } else {
                carousel
            };
            for park in [&mut neutral, &mut earned] {
                park.set_ride_ticket_price(owner_to_raise, difference as u32 * 25)
                    .unwrap();
            }
            matched = Some((neutral, earned, target));
            break 'search;
        }
    }
    let (neutral, earned, target) =
        matched.expect("bounded fresh guest/anchor search produces an exact affordable tie");
    assert_eq!(neutral.guests.last(), earned.guests.last());
    let mut neutral_scores = Vec::new();
    let mut earned_scores = Vec::new();
    for owner in [coaster, carousel] {
        let before = factors(&neutral, target, owner);
        let after = factors(&earned, target, owner);
        assert!(before.wallet >= before.fare && before.willing);
        assert_eq!(before.completed, 0);
        let mut comparison = after.clone();
        comparison.reputation_penalty = before.reputation_penalty;
        assert_eq!(
            before, comparison,
            "every target choice input except reputation stays equal"
        );
        if owner == carousel {
            assert!(after.reputation_penalty < before.reputation_penalty);
        } else {
            assert_eq!(after.reputation_penalty, before.reputation_penalty);
        }
        neutral_scores.push(before.score());
        earned_scores.push(after.score());
    }
    assert_eq!(neutral_scores[0], neutral_scores[1]);
    assert!(earned_scores[1] < earned_scores[0]);
    for (park, expected) in [(&neutral, coaster), (&earned, carousel)] {
        let before = park.clone();
        let bytes = park.to_save_bytes().unwrap();
        let cache = park.path_cache_stats();
        assert_eq!(choices(park).get(&target), Some(&expected));
        assert_eq!(choices(park).get(&target), Some(&expected));
        assert_eq!(park, &before);
        assert_eq!(park.path_cache_stats(), cache);
        assert_eq!(park.to_save_bytes().unwrap(), bytes);
        let loaded = saved(park);
        assert_eq!(choices(&loaded).get(&target), Some(&expected));
    }
}

#[test]
fn version_seventeen_identity_migration_preserves_loading_state_and_new_engine_continuation() {
    let (mut park, coaster, carousel) = fixture();
    for id in [coaster, carousel] {
        park.set_ride_operating_settings(
            id,
            RideOperatingSettings {
                station_wait_ticks: 30,
                ..Default::default()
            },
        )
        .unwrap();
        park.set_ride_dispatch_policy(
            id,
            RideDispatchPolicy {
                minimum_passengers: 6,
                maximum_extra_wait_ticks: 1800,
            },
        )
        .unwrap();
        park.set_ride_open(id, true).unwrap();
    }
    seek(&mut park, |park| {
        park.ride_dispatch_progress(coaster)
            .unwrap()
            .extra_waited_ticks
            > 0
    });
    for paid in [false, true] {
        if paid {
            seek(&mut park, |park| {
                park.guests.iter().any(|guest| {
                    guest
                        .ride_visit
                        .as_ref()
                        .and_then(|visit| visit.passenger.as_ref())
                        .is_some_and(|passenger| passenger.fare_paid)
                })
            });
        }
        let mut old = value(&park);
        let body = old["park"].clone();
        old["schema_version"] = json!(17);
        old["park"].as_object_mut().unwrap().remove("coaster_cars");
        old["content_version"] = json!(17);
        let source_bytes = serde_json::to_vec(&old).unwrap();
        let migrated = Park::from_save_bytes(&source_bytes).unwrap();
        assert_eq!(migrated, park);
        assert_eq!(value(&migrated)["park"], body);
        assert_eq!(serde_json::to_vec(&old).unwrap(), source_bytes);
        let mut normal = migrated.clone();
        let mut grouped = migrated.clone();
        let mut paused = migrated;
        paused.advance(Speed::Paused);
        assert_eq!(paused, normal);
        for _ in 0..1200 {
            normal.advance(Speed::Normal);
        }
        for _ in 0..300 {
            grouped.advance(Speed::Fastest);
        }
        assert_eq!(normal, grouped);
        let mut restored = saved(&normal);
        for _ in 0..300 {
            restored.advance(Speed::Normal);
            normal.advance(Speed::Normal);
            assert_eq!(normal, restored);
        }
    }
}

#[test]
fn version_seventeen_stays_strict_and_derived_appeal_is_not_serialized() {
    let (park, coaster, _) = fixture();
    let base = value(&park);
    assert_eq!(base["schema_version"], crate::SAVE_SCHEMA_VERSION);
    assert_eq!(base["content_version"], crate::SAVE_CONTENT_VERSION);
    for version in [17, crate::SAVE_SCHEMA_VERSION] {
        let mut source = base.clone();
        source["schema_version"] = json!(version);
        source["content_version"] = json!(version);
        if version < 19 {
            source["park"]
                .as_object_mut()
                .unwrap()
                .remove("coaster_cars");
        }
        assert!(read(&source).is_ok());
        for (path, field) in [
            ("/park", "ride_appeal"),
            ("/park/guests/0", "preferred_challenge"),
            ("/park/ride", "layout_challenge"),
        ] {
            let mut malformed = source.clone();
            malformed.pointer_mut(path).unwrap()[field] = json!(25);
            assert!(read(&malformed).is_err());
        }
        let mut missing = source.clone();
        missing["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_dispatch");
        assert!(read(&missing).is_err());
        let mut wrong_shape = source.clone();
        wrong_shape["park"]["ride_dispatch"] = json!([[coaster.0, [1, 30], 0]]);
        assert!(read(&wrong_shape).is_err());
        let mut wrong_timer = source.clone();
        wrong_timer["park"]["ride_dispatch"] = json!([{"ride": coaster.0, "policy": {"minimum_passengers": 1, "maximum_extra_wait_ticks": 30}, "extra_waited_ticks": 1}]);
        assert!(read(&wrong_timer).is_err());
        let text = serde_json::to_string(&source).unwrap();
        let duplicate = text.replacen(
            "\"ride_dispatch\":[]",
            "\"ride_dispatch\":[],\"ride_dispatch\":[]",
            1,
        );
        assert_ne!(text, duplicate);
        assert!(Park::from_save_bytes(duplicate.as_bytes()).is_err());
    }
    for pair in [(17, 18), (18, 17), (20, 20)] {
        let mut malformed = base.clone();
        malformed["schema_version"] = json!(pair.0);
        malformed["content_version"] = json!(pair.1);
        assert!(read(&malformed).is_err());
    }
}
