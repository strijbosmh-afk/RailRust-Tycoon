use super::*;
use crate::{PATH_UNITS, PathDirection, PathSlope, Speed, TilePos, crowds::PathGraph};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn public_park() -> Park {
    let mut park = Park::new(32, 32);
    park.install_crowd_fixture(42).unwrap();
    park
}

fn fixture() -> Park {
    let mut park = public_park();
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
    assert_eq!(park.ride_access(ride).status, RideAccessStatus::Ready);
    park
}

fn advance_until(park: &mut Park, limit: usize, ready: impl Fn(&Park) -> bool) {
    for _ in 0..limit {
        if ready(park) {
            return;
        }
        assert_single_file(park);
        park.advance(Speed::Normal);
    }
    assert!(ready(park), "expected admission state was not reached");
}

fn full_queue() -> Park {
    let mut park = fixture();
    advance_until(&mut park, 1200, |park| {
        let stats = park.ride_queue_stats(park.ride().unwrap().id);
        stats.queueing == 4 && stats.waiting == 4
    });
    park
}

fn assert_single_file(park: &Park) {
    let Some(ride) = park.ride() else { return };
    let access = park.ride_access(ride.id);
    let mut tickets = BTreeSet::new();
    let mut reserved = BTreeMap::new();
    let mut order = Vec::new();
    for guest in park.guests() {
        let Some(visit) = &guest.ride_visit else {
            for id in [guest.current, guest.next].into_iter().flatten() {
                assert!(park.object(id).unwrap().is_public_path());
            }
            continue;
        };
        assert_eq!(visit.ride, ride.id);
        let Some(ticket) = visit.ticket else {
            assert_eq!(visit.phase, RideVisitPhase::Approaching);
            continue;
        };
        assert!(ticket > 0 && ticket < ride.next_queue_ticket);
        assert!(tickets.insert(ticket), "duplicate admission ticket");
        assert!(guest.next.is_none() || guest.current != guest.next);
        for id in [guest.current, guest.next].into_iter().flatten() {
            let object = park.object(id).unwrap();
            if let Some(owner) = object.queue_owner {
                assert_eq!(owner, ride.id);
                assert_eq!(
                    reserved.insert(id, guest.id),
                    None,
                    "overlapping queue reservation"
                );
            }
        }
        if visit.phase == RideVisitPhase::Queueing && access.status == RideAccessStatus::Ready {
            let rank = |id: ObjectId| {
                access
                    .ordered_queue
                    .iter()
                    .position(|candidate| *candidate == id)
                    .unwrap_or(access.ordered_queue.len()) as i32
            };
            let current = rank(guest.current.unwrap());
            let next = guest.next.map_or(current, rank);
            let position =
                current * i32::from(WALK_TICKS) + (next - current) * i32::from(guest.progress);
            order.push((ticket, position));
        }
    }
    order.sort_unstable();
    assert!(
        order.windows(2).all(|pair| pair[0].1 < pair[1].1),
        "ticket order passed on a single-file queue"
    );
    if access.status == RideAccessStatus::Ready {
        assert!(tickets.len() <= access.queue_tiles);
    }
}

#[test]
fn simultaneous_applicants_obey_ids_fifo_capacity_and_unique_edge_reservations() {
    let mut park = fixture();
    let ride = park.ride().unwrap().id;
    let entry = *park
        .object(park.ride_access(ride).public_entry.unwrap())
        .unwrap();
    for guest in &mut park.guests {
        guest.current = Some(entry.id);
        guest.next = None;
        guest.destination = None;
        guest.anchor = entry.pos;
        guest.height = entry.height;
        guest.anchor_slope = entry.path_slope;
        guest.progress = 0;
        guest.state = GuestState::Waiting;
        guest.route.clear();
        guest.wait_ticks = 0;
    }
    let mut reordered = park.clone();
    reordered.guests.reverse();
    for _ in 0..240 {
        park.advance(Speed::Normal);
        reordered.advance(Speed::Normal);
        assert_single_file(&park);
        assert_single_file(&reordered);
        assert_eq!(
            park.ride_queue_order(ride),
            reordered.ride_queue_order(ride)
        );
    }
    assert_eq!(
        park.ride_queue_order(ride),
        vec![GuestId(1), GuestId(2), GuestId(3), GuestId(4)]
    );
    let stats = park.ride_queue_stats(ride);
    assert_eq!(
        (stats.capacity, stats.queueing, stats.waiting, stats.joined),
        (4, 4, 4, 4)
    );
    assert!(stats.turned_away > 0);
    for guest in park
        .guests()
        .iter()
        .filter(|guest| guest.ride_visit.is_some())
    {
        let path = park.object(guest.current.unwrap()).unwrap();
        assert_eq!(
            park.guest_position(guest.id).unwrap().point,
            path.path_center().unwrap()
        );
    }
}

#[test]
fn paused_close_keeps_exact_mid_slope_position_and_finishes_reserved_edge_before_return() {
    let mut park = fixture();
    advance_until(&mut park, 1200, |park| {
        park.guests().iter().any(|guest| {
            guest.progress == WALK_TICKS / 2
                && guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
                && guest
                    .next
                    .and_then(|id| park.object(id))
                    .is_some_and(|path| path.path_slope != PathSlope::Flat)
        })
    });
    let guest = park
        .guests()
        .iter()
        .find(|guest| {
            guest.progress == WALK_TICKS / 2
                && guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|visit| visit.phase == RideVisitPhase::Queueing)
        })
        .unwrap()
        .clone();
    let target = guest.next.unwrap();
    let before = park.guest_position(guest.id).unwrap();
    let ticks = park.ticks();
    park.set_train_open(false);
    assert_eq!(park.ticks(), ticks);
    assert_eq!(park.guest_position(guest.id).unwrap(), before);
    assert_eq!(
        park.guests()
            .iter()
            .find(|g| g.id == guest.id)
            .unwrap()
            .ride_visit
            .as_ref()
            .unwrap()
            .phase,
        RideVisitPhase::Leaving
    );
    let paused = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, paused);
    let mut previous = before.point;
    for _ in guest.progress..WALK_TICKS {
        park.advance(Speed::Normal);
        assert_single_file(&park);
        let point = park.guest_position(guest.id).unwrap().point;
        assert!(
            (point.x - previous.x).abs() + (point.y - previous.y).abs()
                <= PATH_UNITS / i32::from(WALK_TICKS)
        );
        assert!((point.height - previous.height).abs() <= PATH_UNITS / i32::from(WALK_TICKS));
        previous = point;
    }
    let arrived = park.guests().iter().find(|g| g.id == guest.id).unwrap();
    assert_eq!(arrived.current, Some(target));
    assert_eq!(
        previous,
        park.object(target).unwrap().path_center().unwrap()
    );
    advance_until(&mut park, 1200, |park| {
        park.ride_queue_order(park.ride().unwrap().id).is_empty()
    });
    assert!(park.guests().iter().all(|guest| guest.ride_visit.is_none()));
    assert!(!park.ride().unwrap().draining);
}

#[test]
fn patience_drains_the_whole_line_once_and_does_not_re_admit_until_empty() {
    let mut park = full_queue();
    let ride = park.ride().unwrap().id;
    let members = park.ride_queue_order(ride);
    let joined = park.ride().unwrap().joined;
    advance_until(&mut park, QUEUE_PATIENCE_TICKS as usize, |park| {
        park.ride().unwrap().draining
    });
    let stats = park.ride_queue_stats(ride);
    assert_eq!(stats.oldest_wait_ticks, QUEUE_PATIENCE_TICKS);
    assert_eq!((stats.queueing, stats.leaving, stats.abandoned), (0, 4, 4));
    assert!(!stats.admitting);
    assert_eq!(park.ride_queue_order(ride), members);
    while !park.ride_queue_order(ride).is_empty() {
        assert_single_file(&park);
        park.advance(Speed::Normal);
        assert_eq!(park.ride().unwrap().joined, joined);
        assert_eq!(park.ride().unwrap().abandoned, 4);
        assert!(
            park.ticks() < 4000,
            "patience drain failed to make progress"
        );
    }
    assert!(!park.ride().unwrap().draining);
    assert!(park.guests().iter().all(|guest| guest.ride_visit.is_none()));
}

#[test]
fn occupied_deletion_keeps_owned_lost_anchor_and_repair_uses_fresh_identity() {
    let mut park = full_queue();
    let ride = park.ride().unwrap().id;
    let bridge = *park
        .objects()
        .iter()
        .find(|object| object.pos == TilePos { x: 8, y: 13 } && object.queue_owner == Some(ride))
        .unwrap();
    let guest = park
        .guests()
        .iter()
        .find(|guest| guest.current == Some(bridge.id))
        .unwrap()
        .id;
    park.remove_object(bridge.id).unwrap();
    let lost = park
        .guests()
        .iter()
        .find(|candidate| candidate.id == guest)
        .unwrap();
    assert_eq!(
        (lost.current, lost.next, lost.state),
        (None, None, GuestState::Lost)
    );
    assert_eq!(
        (lost.anchor, lost.height, lost.anchor_slope),
        (bridge.pos, bridge.height, bridge.path_slope)
    );
    assert_eq!(
        lost.ride_visit.as_ref().unwrap().phase,
        RideVisitPhase::Leaving
    );
    assert_eq!(
        park.guest_position(guest).unwrap().point,
        bridge.path_center().unwrap()
    );
    let broken = park.clone();
    assert!(
        park.place_queue_path(
            bridge.pos,
            bridge.height,
            bridge.path_slope,
            RideId(ride.0 + 1)
        )
        .is_err()
    );
    assert_eq!(park, broken);
    let replacement = park
        .place_queue_path(bridge.pos, bridge.height, bridge.path_slope, ride)
        .unwrap();
    assert_ne!(replacement, bridge.id);
    advance_until(&mut park, 1200, |park| {
        park.ride_queue_order(ride).is_empty()
    });
    assert!(park.guests().iter().all(|guest| guest.ride_visit.is_none()));
    assert_single_file(&park);
}

#[test]
fn public_walks_keep_legacy_rng_when_no_ride_can_admit() {
    let no_ride = public_park();
    let mut no_gate = public_park();
    no_gate.install_train_fixture().unwrap();
    let mut closed = fixture();
    closed.set_train_open(false);
    let mut disconnected = fixture();
    let bridge = disconnected
        .objects()
        .iter()
        .find(|object| object.pos == TilePos { x: 8, y: 13 })
        .unwrap()
        .id;
    disconnected.remove_object(bridge).unwrap();
    let mut exhausted = fixture();
    let ride = exhausted.ride.as_mut().unwrap();
    ride.next_queue_ticket = u64::MAX;
    ride.joined = u64::MAX - 1;
    ride.abandoned = u64::MAX - 1;
    for mut park in [no_ride, no_gate, closed, disconnected, exhausted] {
        let graph = PathGraph::new(&park.objects);
        let mut reference = park.clone();
        let mut rng = park.crowd_rng;
        for _ in 0..600 {
            reference.ticks += 1;
            reference.step_needs();
            for guest in &mut reference.guests {
                graph.step_guest(guest, &mut rng, crate::visitors::WalkGoal::Explore);
            }
            park.advance(Speed::Normal);
            assert_eq!(park.guests, reference.guests);
            assert_eq!(park.crowd_rng, rng);
        }
    }
}

#[test]
fn repaired_exact_anchor_is_reserved_before_younger_guests_on_reversed_escape_route() {
    let mut park = full_queue();
    let ride = park.ride().unwrap().id;
    let access = park.ride_access(ride);
    let gate = park.ride().unwrap().entrance.as_ref().unwrap().object;
    let bridge = *park
        .objects()
        .iter()
        .find(|path| path.pos == TilePos { x: 8, y: 13 } && path.queue_owner == Some(ride))
        .unwrap();
    let landing = park
        .objects()
        .iter()
        .find(|path| path.pos == TilePos { x: 8, y: 14 } && path.queue_owner == Some(ride))
        .unwrap()
        .id;
    let anchored_id = park
        .guests()
        .iter()
        .find(|guest| guest.current == Some(bridge.id))
        .unwrap()
        .id;
    let younger_id = park
        .guests()
        .iter()
        .find(|guest| guest.current == Some(landing))
        .unwrap()
        .id;
    let anchored_ticket = park
        .guests()
        .iter()
        .find(|guest| guest.id == anchored_id)
        .unwrap()
        .ride_visit
        .as_ref()
        .unwrap()
        .ticket
        .unwrap();
    let younger_ticket = park
        .guests()
        .iter()
        .find(|guest| guest.id == younger_id)
        .unwrap()
        .ride_visit
        .as_ref()
        .unwrap()
        .ticket
        .unwrap();
    assert!(younger_ticket > anchored_ticket);

    // Replace the gate with a public exit and remove the old public boundary.
    // The formerly younger/tail-side guest now wants to move north, through
    // the older visitor's missing footprint, during a reverse-ticket drain.
    park.remove_object(gate).unwrap();
    park.place_path(TilePos { x: 8, y: 11 }, 8).unwrap();
    park.remove_object(access.public_entry.unwrap()).unwrap();
    park.remove_object(bridge.id).unwrap();
    let anchor_before = park.guest_position(anchored_id).unwrap().point;
    let younger_before = park.guest_position(younger_id).unwrap().point;
    let replacement = park
        .place_queue_path(bridge.pos, bridge.height, bridge.path_slope, ride)
        .unwrap();
    assert_ne!(replacement, bridge.id);
    assert_eq!(
        park.guests()
            .iter()
            .find(|guest| guest.id == anchored_id)
            .unwrap()
            .current,
        None
    );

    park.advance(Speed::Normal);
    let anchored = park
        .guests()
        .iter()
        .find(|guest| guest.id == anchored_id)
        .unwrap();
    let younger = park
        .guests()
        .iter()
        .find(|guest| guest.id == younger_id)
        .unwrap();
    assert_eq!(anchored.current, Some(replacement));
    assert_eq!(anchored.next, None);
    assert_eq!(younger.current, Some(landing));
    assert_eq!(
        younger.next, None,
        "a younger guest entered the recovered visitor's exact anchor"
    );
    assert_eq!(
        park.guest_position(anchored_id).unwrap().point,
        anchor_before
    );
    assert_eq!(
        park.guest_position(younger_id).unwrap().point,
        younger_before
    );
    assert_single_file(&park);
    let restored = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    assert_eq!(park, restored);
}

#[test]
fn admitted_and_draining_crowds_are_identical_under_speed_grouping_and_pause() {
    let mut normal = fixture();
    let mut fast = normal.clone();
    let mut fastest = normal.clone();
    let initial = normal.clone();
    normal.advance(Speed::Paused);
    assert_eq!(normal, initial);
    for _ in 0..2400 {
        normal.advance(Speed::Normal);
    }
    for _ in 0..1200 {
        fast.advance(Speed::Fast);
    }
    for _ in 0..600 {
        fastest.advance(Speed::Fastest);
    }
    assert_eq!(normal, fast);
    assert_eq!(normal, fastest);
    assert!(normal.ride().unwrap().joined >= 4);
    assert!(
        normal.ride().unwrap().abandoned >= 4,
        "the workload must cross a patience drain"
    );
    let paused = normal.clone();
    normal.advance(Speed::Paused);
    assert_eq!(normal, paused);
}

#[test]
fn final_ticket_remains_valid_and_continues_after_allocator_exhaustion() {
    let mut park = fixture();
    let ride = park.ride.as_mut().unwrap();
    ride.next_queue_ticket = u64::MAX - 1;
    ride.joined = u64::MAX - 2;
    ride.abandoned = u64::MAX - 2;
    advance_until(&mut park, 1200, |park| {
        !park.ride_queue_order(park.ride().unwrap().id).is_empty()
    });
    let ride_id = park.ride().unwrap().id;
    assert_eq!(park.ride().unwrap().next_queue_ticket, u64::MAX);
    assert_eq!(park.ride_queue_order(ride_id).len(), 1);
    let visit = park
        .guests()
        .iter()
        .find_map(|guest| {
            guest
                .ride_visit
                .as_ref()
                .filter(|visit| visit.ticket.is_some())
        })
        .unwrap();
    assert_eq!(visit.ticket, Some(u64::MAX - 1));
    let mut loaded = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    for _ in 0..180 {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
        assert_eq!(park, loaded);
        assert_single_file(&park);
        assert_eq!(park.ride().unwrap().joined, u64::MAX - 1);
        assert_eq!(park.ride_queue_stats(ride_id).queueing, 1);
    }
    park.set_train_open(false);
    assert_eq!(park.ride().unwrap().abandoned, u64::MAX - 1);
    let closed = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    assert_eq!(park, closed);
    advance_until(&mut park, 1200, |park| {
        park.ride_queue_order(ride_id).is_empty()
    });
    assert!(!park.ride_queue_stats(ride_id).admitting);
}

#[test]
fn admission_save_continuation_survives_close_delete_repair_and_reopen() {
    let mut original = full_queue();
    let mut loaded = Park::from_save_bytes(&original.to_save_bytes().unwrap()).unwrap();
    let ride = original.ride().unwrap().id;
    for tick in 0..360 {
        for park in [&mut original, &mut loaded] {
            match tick {
                10 => park.set_train_open(false),
                20 => {
                    let bridge = park
                        .objects()
                        .iter()
                        .find(|object| {
                            object.pos == TilePos { x: 8, y: 13 }
                                && object.queue_owner == Some(ride)
                        })
                        .unwrap()
                        .id;
                    park.remove_object(bridge).unwrap();
                }
                90 => {
                    park.place_queue_path(TilePos { x: 8, y: 13 }, 8, PathSlope::South, ride)
                        .unwrap();
                }
                180 => park.set_train_open(true),
                _ => {}
            }
            park.advance(Speed::Normal);
            assert_single_file(park);
        }
        assert_eq!(original, loaded, "continuation diverged at tick {tick}");
        if tick % 30 == 0 {
            loaded = Park::from_save_bytes(&loaded.to_save_bytes().unwrap()).unwrap();
            assert_eq!(original, loaded);
        }
    }
}

#[test]
fn save_rejects_duplicated_reservations_tickets_owners_and_incoherent_visits() {
    let park = full_queue();
    let base: Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    let members: Vec<_> = park
        .guests()
        .iter()
        .enumerate()
        .filter(|(_, guest)| {
            guest
                .ride_visit
                .as_ref()
                .is_some_and(|visit| visit.ticket.is_some())
        })
        .map(|(index, _)| index)
        .collect();
    let (a, b) = (members[0], members[1]);
    let mut invalid = Vec::new();
    for (field, value) in [
        ("ticket", Value::Null),
        ("ticket", json!(0)),
        ("ride", json!(999)),
        ("waited_ticks", json!(QUEUE_PATIENCE_TICKS + 1)),
        ("phase", json!("Approaching")),
        ("phase", json!("Leaving")),
    ] {
        let mut data = base.clone();
        data["park"]["guests"][a]["ride_visit"][field] = value;
        invalid.push(data);
    }
    let mut duplicate_ticket = base.clone();
    duplicate_ticket["park"]["guests"][b]["ride_visit"]["ticket"] =
        base["park"]["guests"][a]["ride_visit"]["ticket"].clone();
    invalid.push(duplicate_ticket);
    let mut duplicate_occupancy = base.clone();
    for field in [
        "current",
        "next",
        "destination",
        "anchor",
        "height",
        "anchor_slope",
        "progress",
        "state",
        "route",
        "wait_ticks",
    ] {
        duplicate_occupancy["park"]["guests"][b][field] = base["park"]["guests"][a][field].clone();
    }
    invalid.push(duplicate_occupancy);
    let mut no_permission = base.clone();
    no_permission["park"]["guests"][a]["ride_visit"] = Value::Null;
    invalid.push(no_permission);
    let mut cooldown = base.clone();
    cooldown["park"]["guests"][a]["ride_cooldown"] = json!(RIDE_COOLDOWN_TICKS + 1);
    invalid.push(cooldown);
    let mut reused_allocator = base.clone();
    reused_allocator["park"]["ride"]["next_queue_ticket"] = json!(1);
    invalid.push(reused_allocator);
    for (index, data) in invalid.into_iter().enumerate() {
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&data).unwrap()).is_err(),
            "invalid admission case {index} accepted"
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&park.to_save_bytes().unwrap()).unwrap(),
            base
        );
    }
}
