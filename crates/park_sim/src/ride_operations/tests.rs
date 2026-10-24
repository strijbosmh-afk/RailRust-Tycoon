use super::*;
use crate::{CarouselPhase, PathDirection, Speed, TilePos, TrackKind};
use serde_json::{Value, json};

fn advance(park: &mut Park, ticks: usize) {
    for _ in 0..ticks {
        park.advance(Speed::Normal);
    }
}

fn seek(park: &mut Park, predicate: impl Fn(&Park) -> bool) {
    for _ in 0..24_000 {
        if predicate(park) {
            return;
        }
        park.advance(Speed::Normal);
    }
    panic!("ride operating condition not reached");
}

fn value(park: &Park) -> Value {
    serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap()
}

fn read(value: &Value) -> Result<Park, crate::SaveError> {
    Park::from_save_bytes(&serde_json::to_vec(value).unwrap())
}

fn roundtrip(park: &Park) -> Park {
    let bytes = park.to_save_bytes().unwrap();
    let loaded = Park::from_save_bytes(&bytes).unwrap();
    assert_eq!(park, &loaded);
    assert_eq!(loaded.to_save_bytes().unwrap(), bytes);
    loaded
}

fn empty_rides() -> (Park, RideId, RideId) {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    let coaster = park.ride().unwrap().id;
    park.set_train_open(false);
    let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    (park, coaster, carousel)
}

fn occupied(park: &Park, id: RideId) -> bool {
    park.guests.iter().any(|guest| {
        guest
            .ride_visit
            .as_ref()
            .is_some_and(|visit| visit.ride == id && visit.passenger.is_some())
    })
}

#[test]
fn changes_are_atomic_bounded_and_family_specific_and_noops_preserve_live_state() {
    let (mut park, coaster, carousel) = empty_rides();
    let defaults = RideOperatingSettings::default();
    assert_eq!(park.ride_operating_settings(coaster), Some(defaults));
    assert_eq!(park.ride_operating_settings(carousel), Some(defaults));
    assert_eq!(park.ride_operating_settings(RideId(999)), None);
    let mut invalid = Vec::new();
    for wait in [0, 29, 31, 901, u16::MAX] {
        invalid.push((
            coaster,
            RideOperatingSettings {
                station_wait_ticks: wait,
                ..defaults
            },
            RideOperatingError::InvalidStationWait,
        ));
    }
    for speed in [0, 11, 13, 49, u32::MAX] {
        invalid.push((
            coaster,
            RideOperatingSettings {
                lift_speed: speed,
                ..defaults
            },
            RideOperatingError::InvalidLiftSpeed,
        ));
    }
    for speed in [0, 7, 9, 41, u32::MAX] {
        invalid.push((
            coaster,
            RideOperatingSettings {
                brake_speed: speed,
                ..defaults
            },
            RideOperatingError::InvalidBrakeSpeed,
        ));
    }
    for cycle in [0, 359, 361, 1801, u16::MAX] {
        invalid.push((
            carousel,
            RideOperatingSettings {
                cycle_ticks: cycle,
                ..defaults
            },
            RideOperatingError::InvalidCycleDuration,
        ));
    }
    invalid.extend([
        (
            coaster,
            RideOperatingSettings {
                cycle_ticks: 360,
                ..defaults
            },
            RideOperatingError::UnsupportedSetting,
        ),
        (
            carousel,
            RideOperatingSettings {
                lift_speed: 12,
                ..defaults
            },
            RideOperatingError::UnsupportedSetting,
        ),
        (
            carousel,
            RideOperatingSettings {
                brake_speed: 8,
                ..defaults
            },
            RideOperatingError::UnsupportedSetting,
        ),
        (
            RideId(999),
            defaults,
            RideOperatingError::UnknownRide(RideId(999)),
        ),
    ]);
    let before = park.clone();
    let cache = park.path_cache_stats();
    for (id, settings, error) in invalid {
        assert_eq!(park.set_ride_operating_settings(id, settings), Err(error));
        assert_eq!(park, before);
        assert_eq!(park.path_cache_stats(), cache);
    }
    for id in [coaster, carousel] {
        park.set_ride_open(id, true).unwrap();
        let before = park.to_save_bytes().unwrap();
        assert_eq!(park.set_ride_operating_settings(id, defaults), Ok(false));
        assert_eq!(park.to_save_bytes().unwrap(), before);
        assert_eq!(
            park.ride_operating_settings_status(id),
            Err(RideOperatingError::CloseAndStop)
        );
        assert_eq!(
            park.set_ride_operating_settings(
                id,
                RideOperatingSettings {
                    station_wait_ticks: 30,
                    ..defaults
                }
            ),
            Err(RideOperatingError::CloseAndStop)
        );
        assert_eq!(park.to_save_bytes().unwrap(), before);
    }
}

#[test]
fn changes_preserve_other_state_and_settings_survive_track_repair_and_carousel_tombstones() {
    let (mut park, coaster, carousel) = empty_rides();
    let defaults = RideOperatingSettings::default();
    let custom = RideOperatingSettings {
        station_wait_ticks: 900,
        lift_speed: 48,
        brake_speed: 40,
        ..defaults
    };
    let before = value(&park);
    assert_eq!(park.set_ride_operating_settings(coaster, custom), Ok(true));
    let mut after = value(&park);
    after["park"]["ride_operating_settings"] = before["park"]["ride_operating_settings"].clone();
    assert_eq!(after, before);
    assert_eq!(park.ride_operating_settings(carousel), Some(defaults));
    park.copy_installed_track_to_draft().unwrap();
    let part = park.circuit().unwrap().pieces[3].object;
    park.remove_object(part).unwrap();
    let faulted = park.clone();
    assert_eq!(
        park.set_ride_operating_settings(coaster, defaults),
        Err(RideOperatingError::Unavailable)
    );
    assert_eq!(park, faulted);
    // Faulted legacy remnants are deliberately unowned by draft construction.
    // Use the existing fixture recovery path; a draft must not claim them.
    assert!(matches!(
        park.apply_track_draft(),
        Err(crate::TrackConstructionError::Track(
            crate::TrainError::Placement(crate::PlacementError::Occupied { .. })
        ))
    ));
    assert_eq!(park, faulted);
    park.install_train_fixture().unwrap();
    assert_eq!(park.ride_operating_settings(coaster), Some(custom));
    roundtrip(&park);
    // A subsequent real replacement must retain the same ride settings too.
    let mut brake = *park
        .track_draft()
        .iter()
        .find(|spec| spec.kind == TrackKind::Straight)
        .unwrap();
    brake.kind = TrackKind::Brake;
    park.set_track_draft_piece(brake).unwrap();
    park.apply_track_draft().unwrap();
    assert_eq!(park.ride_operating_settings(coaster), Some(custom));
    roundtrip(&park);
    let carousel_settings = RideOperatingSettings {
        cycle_ticks: 1800,
        station_wait_ticks: 30,
        ..defaults
    };
    park.set_ride_operating_settings(carousel, carousel_settings)
        .unwrap();
    park.remove_carousel(carousel).unwrap();
    let removed = park.clone();
    assert_eq!(
        park.set_ride_operating_settings(carousel, defaults),
        Err(RideOperatingError::Unavailable)
    );
    assert_eq!(park, removed);
    let replacement = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    assert_eq!(park.ride_operating_settings(replacement), Some(defaults));
    assert_eq!(
        park.ride_operating_settings(carousel),
        Some(carousel_settings)
    );
    assert_eq!(park.ride_operating_settings(coaster), Some(custom));
    assert_eq!(
        park.set_ride_operating_settings(coaster, defaults),
        Ok(true)
    );
    assert!(
        park.ride_operating_settings
            .iter()
            .all(|entry| entry.ride != coaster)
    );
    roundtrip(&park);
}

#[test]
fn coaster_wait_lift_and_brake_settings_drive_motion_and_roundtrip_mid_lap() {
    for (wait, lift, brake) in [(30, 12, 8), (900, 48, 40)] {
        let (mut park, coaster, _) = empty_rides();
        let settings = RideOperatingSettings {
            station_wait_ticks: wait,
            lift_speed: lift,
            brake_speed: brake,
            ..Default::default()
        };
        park.set_ride_operating_settings(coaster, settings).unwrap();
        park.set_train_open(true);
        let position = park.train().unwrap().distance;
        advance(&mut park, usize::from(wait));
        assert_eq!(park.train().unwrap().distance, position);
        assert_eq!(park.train().unwrap().dwell_ticks, 0);
        let mut seen_lift = false;
        let mut seen_brake = false;
        for tick in 0..6000 {
            let train = park.train().unwrap();
            let old_kind = park.circuit().unwrap().pieces[train.piece_index].kind;
            park.advance(Speed::Normal);
            let train = park.train().unwrap();
            if train.laps == 1 {
                break;
            }
            if old_kind == TrackKind::Lift {
                assert_eq!(train.speed, lift);
                seen_lift = true;
            }
            if old_kind == TrackKind::Brake && train.speed == brake {
                seen_brake = true;
            }
            if tick % 100 == 0 {
                roundtrip(&park);
            }
        }
        assert!(seen_lift && seen_brake);
        let train = park.train().unwrap();
        assert_eq!(train.laps, 1);
        assert_eq!(train.dwell_ticks, wait);
        assert_eq!(train.distance, position);
        assert_eq!(train.speed, 0);
        roundtrip(&park);
    }
}

#[test]
fn carousel_duration_is_exact_and_history_remains_valid_after_retuning() {
    let (mut park, _, id) = empty_rides();
    for cycle in [360, 540, 1800, 720] {
        let settings = RideOperatingSettings {
            station_wait_ticks: 30,
            cycle_ticks: cycle,
            ..Default::default()
        };
        park.set_ride_operating_settings(id, settings).unwrap();
        let initial_seats = park.carousel_seat_positions(id);
        let cycles = park.carousel(id).unwrap().cycles;
        park.set_ride_open(id, true).unwrap();
        advance(&mut park, 29);
        assert_eq!(park.carousel(id).unwrap().phase, CarouselPhase::Station);
        park.advance(Speed::Normal);
        assert_eq!(park.carousel(id).unwrap().phase, CarouselPhase::Running);
        assert_eq!(park.carousel(id).unwrap().progress, 0);
        let mut restored = roundtrip(&park);
        park.set_ride_open(id, false).unwrap();
        restored.set_ride_open(id, false).unwrap();
        for tick in 1..=cycle {
            park.advance(Speed::Normal);
            restored.advance(Speed::Normal);
            assert_eq!(park, restored);
            if tick < cycle {
                assert_eq!(park.carousel(id).unwrap().progress, tick);
                assert_eq!(park.carousel(id).unwrap().cycles, cycles);
            }
        }
        assert_eq!(park.carousel(id).unwrap().phase, CarouselPhase::Closed);
        assert_eq!(park.carousel(id).unwrap().cycles, cycles + 1);
        assert_eq!(park.carousel_seat_positions(id), initial_seats);
        roundtrip(&park);
    }
}

#[test]
fn paid_passengers_finish_their_cycle_and_must_exit_before_retuning() {
    for coaster_target in [false, true] {
        let (mut park, carousel) = crate::carousel::tests::fixture(true);
        let coaster = park.ride().unwrap().id;
        let id = if coaster_target { coaster } else { carousel };
        let other = if coaster_target { carousel } else { coaster };
        park.set_ride_open(other, false).unwrap();
        park.set_ride_open(id, false).unwrap();
        let settings = RideOperatingSettings {
            station_wait_ticks: 30,
            lift_speed: if coaster_target { 12 } else { 24 },
            brake_speed: if coaster_target { 8 } else { 20 },
            cycle_ticks: if coaster_target { 720 } else { 540 },
        };
        park.set_ride_operating_settings(id, settings).unwrap();
        park.set_ride_open(id, true).unwrap();
        seek(&mut park, |p| p.ride_is_running(id) && occupied(p, id));
        let paid = park.ride_by_id(id).unwrap().revenue_cents;
        assert!(paid > 0);
        // A missing exit keeps completed passengers visibly in their seats.
        let exit = park.ride_by_id(id).unwrap().exit.unwrap();
        park.remove_object(exit.object).unwrap();
        park.set_ride_open(id, false).unwrap();
        let changed = RideOperatingSettings {
            station_wait_ticks: 180,
            ..settings
        };
        let moving = park.clone();
        assert_eq!(
            park.set_ride_operating_settings(id, changed),
            Err(RideOperatingError::CloseAndStop)
        );
        assert_eq!(park, moving);
        let mut loaded = roundtrip(&park);
        for _ in 0..6000 {
            if !park.ride_is_running(id) {
                break;
            }
            park.advance(Speed::Normal);
            loaded.advance(Speed::Normal);
            assert_eq!(park, loaded);
        }
        assert!(!park.ride_is_running(id));
        assert!(occupied(&park, id));
        let waiting = park.clone();
        assert_eq!(
            park.set_ride_operating_settings(id, changed),
            Err(RideOperatingError::Passengers)
        );
        assert_eq!(park, waiting);
        assert_eq!(park.ride_by_id(id).unwrap().revenue_cents, paid);
        let pos = if coaster_target {
            TilePos { x: 8, y: 9 }
        } else {
            TilePos { x: 20, y: 21 }
        };
        park.place_ride_exit(pos, 8, PathDirection::South, id)
            .unwrap();
        seek(&mut park, |p| !occupied(p, id));
        let before_change = value(&park);
        assert_eq!(park.set_ride_operating_settings(id, changed), Ok(true));
        let mut after_change = value(&park);
        after_change["park"]["ride_operating_settings"] =
            before_change["park"]["ride_operating_settings"].clone();
        assert_eq!(before_change, after_change);
        let ride = park.ride_by_id(id).unwrap();
        assert_eq!(ride.revenue_cents, paid);
        assert_eq!(ride.boarded, ride.exited);
        assert_eq!(
            ride.completed,
            park.guests
                .iter()
                .flat_map(|g| &g.ride_history)
                .filter(|h| h.ride == id)
                .map(|h| h.completed)
                .sum::<u64>()
        );
        roundtrip(&park);
        park.set_ride_open(id, true).unwrap();
        let mut restored = roundtrip(&park);
        for _ in 0..1200 {
            park.advance(Speed::Normal);
            restored.advance(Speed::Normal);
            assert_eq!(park, restored);
        }
        roundtrip(&park);
    }
}

#[test]
fn current_settings_are_required_strict_bounded_unique_and_owned() {
    let (mut park, coaster, carousel) = empty_rides();
    park.set_ride_operating_settings(
        coaster,
        RideOperatingSettings {
            station_wait_ticks: 30,
            ..Default::default()
        },
    )
    .unwrap();
    park.set_ride_operating_settings(
        carousel,
        RideOperatingSettings {
            cycle_ticks: 540,
            ..Default::default()
        },
    )
    .unwrap();
    let base = value(&park);
    assert_eq!(base["schema_version"], json!(crate::SAVE_SCHEMA_VERSION));
    assert_eq!(base["content_version"], json!(crate::SAVE_CONTENT_VERSION));
    let record = base["park"]["ride_operating_settings"][0].clone();
    let mut missing = base.clone();
    missing["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_operating_settings");
    assert!(read(&missing).is_err());
    for malformed in [
        json!(null),
        json!({}),
        json!([null]),
        json!([[coaster.0, record["settings"]]]),
        json!([record.clone(), record.clone()]),
        json!([base["park"]["ride_operating_settings"][1], record.clone()]),
        json!(vec![record.clone(); crate::MAX_RIDE_RECORDS + 1]),
    ] {
        let mut forged = base.clone();
        forged["park"]["ride_operating_settings"] = malformed;
        assert!(read(&forged).is_err());
    }
    for field in [
        "station_wait_ticks",
        "lift_speed",
        "brake_speed",
        "cycle_ticks",
    ] {
        let mut forged = base.clone();
        forged["park"]["ride_operating_settings"][0]["settings"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(read(&forged).is_err(), "accepted missing {field}");
        for invalid in [
            json!(null),
            json!("30"),
            json!(-1),
            json!(30.0),
            json!(1e30),
        ] {
            let mut forged = base.clone();
            forged["park"]["ride_operating_settings"][0]["settings"][field] = invalid;
            assert!(read(&forged).is_err(), "accepted malformed {field}");
        }
    }
    for (path, invalid) in [
        ("/park/ride_operating_settings/0/ride", json!(999)),
        (
            "/park/ride_operating_settings/0/settings",
            json!([30, 24, 20, 720]),
        ),
        (
            "/park/ride_operating_settings/0/settings",
            json!(RideOperatingSettings::default()),
        ),
        (
            "/park/ride_operating_settings/0/settings/station_wait_ticks",
            json!(31),
        ),
        (
            "/park/ride_operating_settings/0/settings/lift_speed",
            json!(52),
        ),
        (
            "/park/ride_operating_settings/0/settings/brake_speed",
            json!(4),
        ),
        (
            "/park/ride_operating_settings/0/settings/cycle_ticks",
            json!(360),
        ),
        (
            "/park/ride_operating_settings/1/settings/lift_speed",
            json!(12),
        ),
        (
            "/park/ride_operating_settings/1/settings/brake_speed",
            json!(8),
        ),
        (
            "/park/ride_operating_settings/1/settings/cycle_ticks",
            json!(541),
        ),
    ] {
        let mut forged = base.clone();
        *forged.pointer_mut(path).unwrap() = invalid;
        assert!(read(&forged).is_err(), "accepted {path}");
    }
    for path in [
        "/park/ride_operating_settings/0",
        "/park/ride_operating_settings/0/settings",
    ] {
        let mut forged = base.clone();
        forged.pointer_mut(path).unwrap()["future"] = json!(0);
        assert!(read(&forged).is_err());
    }
    let text = serde_json::to_string(&base).unwrap();
    let duplicate = text.replace(
        "\"station_wait_ticks\":30",
        "\"station_wait_ticks\":30,\"station_wait_ticks\":30",
    );
    assert_ne!(duplicate, text);
    assert!(Park::from_save_bytes(duplicate.as_bytes()).is_err());
    roundtrip(&park);
}

#[test]
fn schema_fifteen_preserves_every_prior_field_and_default_future_behavior() {
    let (mut park, carousel) = crate::carousel::tests::fixture(true);
    seek(&mut park, |p| {
        p.ride_is_running(carousel) && occupied(p, carousel)
    });
    park.copy_installed_track_to_draft().unwrap();
    let mut old = value(&park);
    old["schema_version"] = json!(15);
    old["content_version"] = json!(15);
    old["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_operating_settings");
    old["park"].as_object_mut().unwrap().remove("ride_dispatch");
    let mut migrated = read(&old).unwrap();
    assert_eq!(migrated, park);
    for ride in migrated.all_rides() {
        assert_eq!(
            migrated.ride_operating_settings(ride.id),
            Some(RideOperatingSettings::default())
        );
    }
    let mut projection = value(&migrated);
    projection["schema_version"] = json!(15);
    projection["content_version"] = json!(15);
    projection["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_operating_settings");
    projection["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_dispatch");
    assert_eq!(projection, old);
    let mut smuggled = old.clone();
    smuggled["park"]["ride_operating_settings"] = json!([]);
    assert!(read(&smuggled).is_err());
    for _ in 0..1800 {
        migrated.advance(Speed::Normal);
        park.advance(Speed::Normal);
        assert_eq!(migrated, park);
    }
    roundtrip(&migrated);
}

#[test]
fn timer_validation_uses_current_settings_but_legacy_keeps_original_counter_bounds() {
    let (mut park, coaster, carousel) = empty_rides();
    park.set_ride_operating_settings(
        coaster,
        RideOperatingSettings {
            station_wait_ticks: 900,
            ..Default::default()
        },
    )
    .unwrap();
    park.set_ride_operating_settings(
        carousel,
        RideOperatingSettings {
            station_wait_ticks: 900,
            cycle_ticks: 1800,
            ..Default::default()
        },
    )
    .unwrap();
    park.set_ride_open(coaster, true).unwrap();
    park.set_ride_open(carousel, true).unwrap();
    roundtrip(&park);
    let base = value(&park);
    for path in ["/park/train/dwell_ticks", "/park/carousels/0/dwell_ticks"] {
        let mut invalid = base.clone();
        *invalid.pointer_mut(path).unwrap() = json!(901);
        assert!(read(&invalid).is_err());
    }
    let mut defaults = base.clone();
    defaults["park"]["ride_operating_settings"] = json!([]);
    assert!(read(&defaults).is_err());

    let (mut original, _, id) = empty_rides();
    original.set_ride_open(id, true).unwrap();
    advance(&mut original, 90 + 720);
    original.set_ride_open(id, false).unwrap();
    let mut legacy = value(&original);
    legacy["schema_version"] = json!(15);
    legacy["content_version"] = json!(15);
    legacy["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_operating_settings");
    legacy["park"]
        .as_object_mut()
        .unwrap()
        .remove("ride_dispatch");
    assert!(read(&legacy).is_ok());
    // A forged early completion passes the new minimum-duration bound but was
    // never possible under the fixed legacy cycle, so v15 must still reject it.
    legacy["park"]["ticks"] = json!(500);
    assert!(read(&legacy).is_err());
}
