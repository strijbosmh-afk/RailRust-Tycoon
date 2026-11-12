use crate::*;

fn specifications() -> Vec<TrackPieceSpec> {
    let mut source = Park::new(32, 32);
    source.install_train_fixture().unwrap();
    let mut specs = source.circuit().unwrap().specs().to_vec();
    for p in &mut specs {
        p.entry.height -= 8;
        p.exit.height -= 8;
        match (p.pos.x, p.pos.y) {
            (9, 10) => p.kind = TrackKind::TransitionUp,
            (17, 10) => p.kind = TrackKind::Launch,
            (20, 10) => p.kind = TrackKind::HeartlineRoll,
            (21, 10) | (16, 4) => p.kind = TrackKind::Brake,
            (24, 10) => p.kind = TrackKind::BankedCurve,
            (19, 4) => {
                p.kind = TrackKind::TransitionUp;
                p.exit.height = 1;
            }
            (18, 4) => {
                p.kind = TrackKind::TransitionDown;
                p.entry.height = 1;
            }
            _ => {}
        }
    }
    specs
}

#[test]
fn custom_coaster_build_test_correct_operate_repair_reload_and_export() {
    let specs = specifications();
    let mut park = Park::new(64, 64);
    park.borrow_loan(100_000).unwrap();
    let mut incomplete = specs.clone();
    incomplete.pop();
    park.replace_track_draft(&incomplete).unwrap();
    let rejected = park.clone();
    assert!(park.apply_track_draft().is_err());
    assert_eq!(park, rejected);
    park.replace_track_draft(&specs).unwrap();
    let quote = park.track_construction_quote().unwrap();
    let cash = park.cash_cents();
    park.apply_track_draft().unwrap();
    assert_eq!(park.cash_cents(), cash - quote.construction_cents);
    let id = park.ride().unwrap().id;
    let report = park.test_coaster_for(id).unwrap();
    assert_eq!(report.layout.inversions, 1);
    assert_eq!(report.layout.transitions, 3);
    assert!(report.forces.min_vertical_millig < 0);
    assert!(report.forces.max_vertical_millig > 1000);
    park.set_coaster_train_count(id, 3).unwrap();
    assert!(!report.input.matches(&park));
    let mut no_blocks = specs.clone();
    for p in &mut no_blocks {
        if p.kind == TrackKind::Brake {
            p.kind = TrackKind::Straight;
        }
    }
    park.replace_track_draft(&no_blocks).unwrap();
    let rejected = park.clone();
    assert!(park.track_construction_quote().is_err());
    assert!(park.apply_track_draft().is_err());
    assert_eq!(park, rejected);
    park.replace_track_draft(&specs).unwrap();
    park.apply_track_draft().unwrap();
    park.place_ride_entrance(TilePos { x: 8, y: 11 }, 0, PathDirection::North, id)
        .unwrap();
    for y in 12..=14 {
        park.place_queue_path(TilePos { x: 8, y }, 0, PathSlope::Flat, id)
            .unwrap();
    }
    let entry = park.place_path(TilePos { x: 8, y: 15 }, 0).unwrap();
    park.place_ride_exit(TilePos { x: 8, y: 9 }, 0, PathDirection::South, id)
        .unwrap();
    for x in 8..=13 {
        park.place_path(TilePos { x, y: 8 }, 0).unwrap();
    }
    for y in 9..=15 {
        park.place_path(TilePos { x: 13, y }, 0).unwrap();
    }
    for x in 9..=12 {
        park.place_path(TilePos { x, y: 15 }, 0).unwrap();
    }
    park.set_visitor_entry(entry).unwrap();
    park.set_visitor_arrivals(true).unwrap();
    park.set_ride_ticket_price(id, 100).unwrap();
    park.set_ride_open_checked(id, true).unwrap();
    let mut inverted = false;
    for tick in 0..18000 {
        park.advance(Speed::Normal);
        inverted |= park.coaster_trains(id).iter().any(|t| {
            (160_000..200_000).contains(&park.coaster_circuit(id).unwrap().roll_at(t.distance))
        });
        if tick % 3000 == 0 {
            Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        }
    }
    assert!(inverted && park.ride().unwrap().completed >= 6);
    assert!(park.service_ride(id).is_err());
    let bytes = park.to_save_bytes().unwrap();
    let mut loaded = Park::from_save_bytes(&bytes).unwrap();
    for _ in 0..900 {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
    }
    assert_eq!(park, loaded);
    park.set_ride_open(id, false).unwrap();
    for _ in 0..14000 {
        park.advance(Speed::Normal);
    }
    assert!(!park.ride_is_running(id));
    park.service_ride(id).unwrap();
    assert_eq!(park.ride_maintenance_status(id).unwrap().service_count, 1);
    let exported = TrackDesign::from_draft("Aurora roll", park.circuit().unwrap().specs())
        .unwrap()
        .to_json()
        .unwrap();
    let imported = TrackDesign::from_json(&exported).unwrap();
    park.new_coaster_draft().unwrap();
    park.replace_track_draft(
        &imported
            .transformed(TilePos { x: 38, y: 10 }, 0, 0)
            .unwrap(),
    )
    .unwrap();
    park.apply_track_draft().unwrap();
    let second = park.ride().unwrap().id;
    assert_ne!(id, second);
    assert_eq!(park.test_coaster_for(second).unwrap().layout.inversions, 1);
    assert_eq!(park.coaster_train_count(id), Some(3));
    assert_eq!(park.coaster_train_count(second), Some(1));
    assert_eq!(
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
        park
    );
    let mut destination = Park::new(64, 64);
    destination
        .replace_track_draft(&imported.transformed(TilePos { x: 8, y: 10 }, 0, 1).unwrap())
        .unwrap();
    destination.apply_track_draft().unwrap();
    assert_eq!(destination.test_coaster().unwrap().layout.inversions, 1);
    assert_eq!(
        Park::from_save_bytes(&destination.to_save_bytes().unwrap()).unwrap(),
        destination
    );
}

#[test]
fn inversion_geometry_clearance_design_and_save_versions_agree() {
    let spec = TrackBuildKind::HeartlineRoll.spec(TilePos { x: 4, y: 4 }, 0, TrackDirection::East);
    let samples = spec.samples();
    assert_eq!(samples.len(), 33);
    assert_eq!(samples[16].height, 2 * TRACK_UNITS);
    assert_eq!(spec.roll_millidegrees(500), 180_000);
    assert_eq!(spec.roll_millidegrees(1000), 360_000);
    assert!(samples.windows(2).all(|pair| pair[1].x > pair[0].x));
    let mut park = Park::new(32, 32);
    park.set_track_draft_piece(spec).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    value["schema_version"] = 46.into();
    value["content_version"] = 46.into();
    assert!(Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    let design = TrackDesign::from_draft("Roll", &[spec])
        .unwrap()
        .to_json()
        .unwrap();
    let old = design.replace("\"version\": 2", "\"version\": 1");
    assert!(TrackDesign::from_json(&old).is_err());
    assert_eq!(TrackDesign::from_json(&design).unwrap().pieces(), &[spec]);
    let mut blocked = Park::new(32, 32);
    blocked
        .set_terrain_height(TilePos { x: 4, y: 4 }, 1)
        .unwrap();
    assert!(blocked.set_track_draft_piece(spec).is_err());
}

#[test]
fn old_banked_coaster_migrates_its_moving_train_to_the_new_centerline() {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    let mut specs = park.circuit().unwrap().specs().to_vec();
    let index = specs
        .iter()
        .position(|s| s.kind == TrackKind::Curve)
        .unwrap();
    specs[index].kind = TrackKind::BankedCurve;
    park.set_train_open(false);
    park.replace_track_draft(&specs).unwrap();
    park.apply_track_draft().unwrap();
    let mut lengths = Vec::new();
    for spec in &specs {
        let mut old = *spec;
        if old.kind == TrackKind::BankedCurve {
            old.kind = TrackKind::Curve;
        }
        lengths.push(
            old.samples()
                .windows(2)
                .map(|p| crate::trains::point_distance(p[0], p[1]))
                .sum::<u32>(),
        );
    }
    let distance = lengths[..index].iter().sum::<u32>() + lengths[index] / 2;
    let mut value: serde_json::Value =
        serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    value["schema_version"] = 46.into();
    value["content_version"] = 46.into();
    let train = &mut value["park"]["train"];
    train["open"] = true.into();
    train["departed"] = true.into();
    train["phase"] = "Cruising".into();
    train["distance"] = distance.into();
    train["lap_progress"] = (distance - 768).into();
    train["piece_index"] = index.into();
    train["speed"] = 32.into();
    let migrated = Park::from_save_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(migrated.train().unwrap().piece_index, index);
    assert!(migrated.train().unwrap().distance > distance);
    assert!(
        migrated
            .circuit()
            .unwrap()
            .sample(migrated.train().unwrap().distance)
            .height
            > specs[index].entry.height as i32 * TRACK_UNITS
    );
    assert_eq!(
        Park::from_save_bytes(&migrated.to_save_bytes().unwrap()).unwrap(),
        migrated
    );
}
