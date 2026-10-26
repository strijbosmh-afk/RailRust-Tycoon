use super::*;
use crate::{Speed, TilePos};
fn without_paint(p: &Park) -> serde_json::Value {
    let mut v: serde_json::Value = serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    v["park"].as_object_mut().unwrap().remove("ride_paints");
    v
}
#[test]
fn painting_a_paid_running_ride_preserves_every_other_field_and_future_behavior() {
    let (mut p, _) = crate::carousel::tests::fixture(true);
    for _ in 0..4000 {
        p.advance(Speed::Normal);
        if p.guests().iter().any(|g| {
            g.ride_visit
                .as_ref()
                .and_then(|v| v.passenger.as_ref())
                .is_some_and(|p| p.fare_paid)
        }) {
            break;
        }
    }
    assert!(p.guests().iter().any(|g| {
        g.ride_visit
            .as_ref()
            .and_then(|v| v.passenger.as_ref())
            .is_some_and(|p| p.fare_paid)
    }));
    let mut classic = p.clone();
    let ride = p.ride().unwrap().id;
    let before = without_paint(&p);
    p.set_ride_appearance(RideAppearance {
        ride,
        structure: RidePaint::Coral,
        supports: RidePaint::Sky,
        vehicles: RidePaint::Lagoon,
    })
    .unwrap();
    assert_eq!(without_paint(&p), before);
    for tick in 0..1800 {
        p.advance(Speed::Normal);
        classic.advance(Speed::Normal);
        if tick % 300 == 0 {
            p = Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap();
        }
    }
    assert_eq!(without_paint(&p), without_paint(&classic));
    p.set_ride_appearance(RideAppearance {
        ride,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(p, classic);
}
#[test]
fn colors_are_owned_by_stable_ride_ids_and_removed_ride_history_does_not_repaint_a_new_ride() {
    let mut p = Park::new(32, 32);
    let id = p.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    let old = p.clone();
    p.set_ride_appearance(RideAppearance {
        ride: id,
        structure: RidePaint::Violet,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(without_paint(&p), without_paint(&old));
    p.remove_carousel(id).unwrap();
    let saved = p.to_save_bytes().unwrap();
    assert_eq!(Park::from_save_bytes(&saved).unwrap(), p);
    let before = p.clone();
    assert_eq!(
        p.set_ride_appearance(RideAppearance {
            ride: id,
            ..Default::default()
        }),
        Err(RidePaintError::NotInstalled)
    );
    assert_eq!(p, before);
    let next = p.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    assert_ne!(next, id);
    assert_eq!(
        p.ride_appearance(next).unwrap().structure,
        RidePaint::Classic
    );
    assert_eq!(p.ride_appearance(id).unwrap().structure, RidePaint::Violet);
    let before = p.clone();
    assert_eq!(
        p.set_ride_appearance(RideAppearance {
            ride: RideId(999),
            ..Default::default()
        }),
        Err(RidePaintError::UnknownRide)
    );
    assert_eq!(p, before);
}
#[test]
fn strict_paint_saves_reject_unknown_owners_colors_missing_fields_and_noncanonical_records() {
    let mut p = Park::new(32, 32);
    p.install_train_fixture().unwrap();
    let id = p.ride().unwrap().id;
    p.set_ride_appearance(RideAppearance {
        ride: id,
        structure: RidePaint::Coral,
        ..Default::default()
    })
    .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    for (path, bad) in [
        ("/park/ride_paints/0/ride", serde_json::json!(999)),
        (
            "/park/ride_paints/0/structure",
            serde_json::json!("Unknown"),
        ),
        ("/park/ride_paints/0/structure", serde_json::json!(0)),
        (
            "/park/ride_paints/0/structure",
            serde_json::json!("Classic"),
        ),
    ] {
        let mut b = v.clone();
        *b.pointer_mut(path).unwrap() = bad;
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&b).unwrap()).is_err(),
            "{path}"
        );
    }
    let mut b = v.clone();
    b["park"]["ride_paints"][0]
        .as_object_mut()
        .unwrap()
        .remove("vehicles");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&b).unwrap()).is_err());
    let mut b = v.clone();
    b["park"]["ride_paints"] =
        serde_json::json!([v["park"]["ride_paints"][0], v["park"]["ride_paints"][0]]);
    assert!(Park::from_save_bytes(&serde_json::to_vec(&b).unwrap()).is_err());
    let mut b = v;
    b["park"].as_object_mut().unwrap().remove("ride_paints");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&b).unwrap()).is_err());
}

#[test]
fn authentic_v22_migration_preserves_every_original_field_and_classic_defaults() {
    let bytes = include_bytes!("../../testdata/v22-paid-visitors.park.json");
    let original: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let p = Park::from_save_bytes(bytes).unwrap();
    let mut current: serde_json::Value =
        serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    assert_eq!(
        current["park"]
            .as_object_mut()
            .unwrap()
            .remove("ride_paints"),
        Some(serde_json::json!([]))
    );
    assert_eq!(current["park"], original["park"]);
    let mut uninterrupted = p.clone();
    let mut resumed = p;
    for tick in 0..3600 {
        uninterrupted.advance(Speed::Normal);
        resumed.advance(Speed::Normal);
        if tick % 300 == 0 {
            resumed = Park::from_save_bytes(&resumed.to_save_bytes().unwrap()).unwrap();
        }
    }
    assert_eq!(uninterrupted, resumed);
    assert!(
        resumed
            .all_rides()
            .all(|r| resumed.ride_appearance(r.id).unwrap()
                == RideAppearance {
                    ride: r.id,
                    ..Default::default()
                })
    );
}
