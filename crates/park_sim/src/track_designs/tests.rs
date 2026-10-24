use super::*;
use crate::{Axis, ObjectKind, Park, TrackBuildKind, TrackConstructionError};
use serde_json::{Value, json};

fn piece(x: u32, y: u32, height: u8, kind: TrackBuildKind) -> TrackPieceSpec {
    kind.spec(TilePos { x, y }, height, TrackDirection::East)
}

fn loop_specs() -> Vec<TrackPieceSpec> {
    use TrackBuildKind::{CurveRight, Station, Straight};
    use TrackDirection::{East, North, South, West};
    [
        (5, 4, Station, East),
        (6, 4, CurveRight, East),
        (6, 5, Straight, South),
        (6, 6, CurveRight, South),
        (5, 6, Straight, West),
        (4, 6, CurveRight, West),
        (4, 5, Straight, North),
        (4, 4, CurveRight, North),
    ]
    .into_iter()
    .map(|(x, y, kind, direction)| kind.spec(TilePos { x, y }, 8, direction))
    .collect()
}

fn legacy_design() -> TrackDesign {
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    TrackDesign::from_draft("Legacy grades", park.circuit().unwrap().specs()).unwrap()
}

#[test]
fn exact_order_geometry_and_unicode_name_roundtrip() {
    let mut pieces = legacy_design().pieces().to_vec();
    pieces.rotate_left(9);
    let design = TrackDesign::from_draft("  Départ 🎢  ", &pieces).unwrap();
    assert_eq!(design.name(), "Départ 🎢");
    assert_eq!(design.pieces(), pieces);
    let json = design.to_json().unwrap();
    let decoded = TrackDesign::from_json(&json).unwrap();
    assert_eq!(decoded, design);
    assert_eq!(decoded.to_json().unwrap(), json);
    assert_eq!(design.anchor(), legacy_design().anchor());
    let envelope: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope.as_object().unwrap().len(), 4);
    assert_eq!(envelope["format"], "parkbound-track-design");
    assert_eq!(envelope["version"], 1);
}

#[test]
fn all_clockwise_rotations_preserve_exact_legacy_grades_and_connected_circuit() {
    let design = legacy_design();
    assert_eq!(design.pieces().len(), 46);
    assert!(
        design
            .pieces()
            .iter()
            .any(|piece| piece.entry.height.abs_diff(piece.exit.height) == 2)
    );
    let (source, height) = design.anchor();
    assert_eq!(
        design.transformed(source, height, 0).unwrap(),
        design.pieces()
    );
    assert_eq!(
        design.transformed(source, height, 4).unwrap(),
        design.pieces()
    );
    for turns in 0..4 {
        let anchor = TilePos { x: 120, y: 120 };
        let moved = design.transformed(anchor, height + 3, turns).unwrap();
        for (old, new) in design.pieces().iter().zip(&moved) {
            let dx = i64::from(old.pos.x) - i64::from(source.x);
            let dy = i64::from(old.pos.y) - i64::from(source.y);
            let (x, y) = match turns {
                0 => (dx, dy),
                1 => (-dy, dx),
                2 => (-dx, -dy),
                _ => (dy, -dx),
            };
            assert_eq!(
                new.pos,
                TilePos {
                    x: (120 + x) as u32,
                    y: (120 + y) as u32
                }
            );
            assert_eq!(new.kind, old.kind);
            assert_eq!(new.entry.height, old.entry.height + 3);
            assert_eq!(new.exit.height, old.exit.height + 3);
        }
        let mut park = Park::new(256, 256);
        park.replace_track_draft(&moved).unwrap();
        assert_eq!(park.validate_track_draft().unwrap(), moved);
        park.apply_track_draft().unwrap();
        park.validate_train_circuit().unwrap();
        let turned = TrackDesign::from_draft("Turned", &moved).unwrap();
        assert_eq!(
            turned.transformed(source, height, (4 - turns) % 4).unwrap(),
            design.pieces()
        );
    }
}

#[test]
fn unfinished_designs_anchor_to_first_piece_or_first_station_without_reordering() {
    let straight = piece(10, 10, 6, TrackBuildKind::Straight);
    let station = piece(12, 11, 8, TrackBuildKind::Station);
    let second = piece(14, 11, 9, TrackBuildKind::Station);
    let unfinished = TrackDesign::from_draft("Unfinished", &[straight]).unwrap();
    assert_eq!(unfinished.anchor(), (straight.pos, 6));
    let moved = unfinished
        .transformed(TilePos { x: 20, y: 20 }, 10, 1)
        .unwrap();
    assert_eq!(moved[0].entry.edge, TrackDirection::North);
    assert_eq!(moved[0].exit.edge, TrackDirection::South);
    let multiple = TrackDesign::from_draft("Two stations", &[straight, station, second]).unwrap();
    assert_eq!(multiple.anchor(), (station.pos, 8));
    assert_eq!(multiple.pieces(), &[straight, station, second]);
    let mut park = Park::new(32, 32);
    park.replace_track_draft(multiple.pieces()).unwrap();
    assert!(park.apply_track_draft().is_err());
}

#[test]
fn naming_is_trimmed_bounded_by_unicode_scalars_and_rejects_controls() {
    let pieces = [piece(1, 1, 8, TrackBuildKind::Straight)];
    for empty in ["", "   ", "\u{2003}"] {
        assert_eq!(
            TrackDesign::from_draft(empty, &pieces),
            Err(TrackDesignError::EmptyName)
        );
    }
    for control in ["\nName", "Name\t", "A\0B", "A\u{7f}B", "A\u{85}B"] {
        assert_eq!(
            TrackDesign::from_draft(control, &pieces),
            Err(TrackDesignError::NameContainsControl)
        );
    }
    let longest = "🎢".repeat(MAX_TRACK_DESIGN_NAME_CHARS);
    assert_eq!(
        TrackDesign::from_draft(&longest, &pieces).unwrap().name(),
        longest
    );
    assert_eq!(
        TrackDesign::from_draft(&(longest + "x"), &pieces),
        Err(TrackDesignError::NameTooLong)
    );
}

#[test]
fn structure_rejects_empty_duplicate_unsupported_high_outside_and_excessive_pieces() {
    let valid = piece(0, 0, 8, TrackBuildKind::Straight);
    assert_eq!(
        TrackDesign::from_draft("Empty", &[]),
        Err(TrackDesignError::EmptyDesign)
    );
    assert_eq!(
        TrackDesign::from_draft("Duplicate", &[valid, valid]),
        Err(TrackDesignError::Geometry(
            TrackConstructionError::DuplicateTile(valid.pos)
        ))
    );
    let malformed = TrackPieceSpec {
        kind: TrackKind::Lift,
        ..valid
    };
    let high = piece(1, 1, 31, TrackBuildKind::Straight);
    let outside = piece(MAX_MAP_SIZE, 1, 8, TrackBuildKind::Straight);
    for invalid in [malformed, high, outside] {
        assert!(TrackDesign::from_draft("Invalid", &[valid, invalid]).is_err());
    }
    let maximum: Vec<_> = (0..MAX_TRACK_DRAFT_PIECES)
        .rev()
        .map(|index| {
            piece(
                (index % 16) as u32,
                (index / 16) as u32,
                30,
                TrackBuildKind::Straight,
            )
        })
        .collect();
    let design = TrackDesign::from_draft("Maximum", &maximum).unwrap();
    assert_eq!(
        TrackDesign::from_json(&design.to_json().unwrap())
            .unwrap()
            .pieces(),
        maximum
    );
    let mut excess = maximum;
    excess.push(piece(40, 40, 30, TrackBuildKind::Straight));
    assert_eq!(
        TrackDesign::from_draft("Excess", &excess),
        Err(TrackDesignError::Geometry(
            TrackConstructionError::DraftFull
        ))
    );
}

#[test]
fn placement_rejects_negative_overflow_and_grade_height_bounds_without_clamping() {
    let design = TrackDesign::from_draft(
        "Grades",
        &[
            piece(10, 10, 8, TrackBuildKind::Station),
            piece(9, 10, 9, TrackBuildKind::Lift),
            piece(11, 10, 8, TrackBuildKind::SteepDrop),
        ],
    )
    .unwrap();
    for anchor in [
        TilePos { x: 0, y: 10 },
        TilePos { x: 255, y: 10 },
        TilePos {
            x: u32::MAX,
            y: u32::MAX,
        },
    ] {
        assert_eq!(
            design.transformed(anchor, 8, 0),
            Err(TrackDesignError::PositionOutOfBounds)
        );
    }
    for height in [0, 1, 29, 30, 255] {
        assert_eq!(
            design.transformed(TilePos { x: 10, y: 10 }, height, 0),
            Err(TrackDesignError::HeightOutOfBounds)
        );
    }
    assert!(design.transformed(TilePos { x: 10, y: 10 }, 2, 0).is_ok());
    assert!(design.transformed(TilePos { x: 10, y: 10 }, 28, 0).is_ok());
    let mut park = Park::new(32, 32);
    park.replace_track_draft(&loop_specs()).unwrap();
    let before = park.clone();
    let beyond_park = design.transformed(TilePos { x: 32, y: 32 }, 8, 0).unwrap();
    assert!(park.replace_track_draft(&beyond_park).is_err());
    assert_eq!(park, before);
}

#[test]
fn strict_codec_rejects_unknown_missing_duplicate_fields_and_positional_forms() {
    let design =
        TrackDesign::from_draft("Strict", &[piece(1, 1, 8, TrackBuildKind::Straight)]).unwrap();
    let base: Value = serde_json::from_str(&design.to_json().unwrap()).unwrap();
    let mut cases = Vec::new();
    for pointer in [
        "",
        "/pieces/0",
        "/pieces/0/pos",
        "/pieces/0/entry",
        "/pieces/0/exit",
    ] {
        let mut unknown = base.clone();
        unknown
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(false));
        cases.push(unknown);
        let mut positional = base.clone();
        let object = positional.pointer_mut(pointer).unwrap();
        *object = Value::Array(object.as_object().unwrap().values().cloned().collect());
        cases.push(positional);
    }
    for field in ["format", "version", "name", "pieces"] {
        let mut missing = base.clone();
        missing.as_object_mut().unwrap().remove(field);
        cases.push(missing);
    }
    for pointer in [
        "/pieces/0/kind",
        "/pieces/0/entry/edge",
        "/pieces/0/exit/edge",
    ] {
        let mut tagged = base.clone();
        let field = tagged.pointer_mut(pointer).unwrap();
        let name = field.as_str().unwrap().to_owned();
        *field = json!({name: null});
        cases.push(tagged);
    }
    for invalid in cases {
        assert!(
            TrackDesign::from_json(&invalid.to_string()).is_err(),
            "accepted {invalid}"
        );
    }
    let text = design.to_json().unwrap();
    assert!(
        TrackDesign::from_json(&text.replacen(
            "\"version\": 1",
            "\"version\": 1, \"version\": 1",
            1
        ))
        .is_err()
    );
    assert!(TrackDesign::from_json(&text.replacen("\"x\": 1", "\"x\": 1, \"x\": 1", 1)).is_err());
    assert!(TrackDesign::from_json(&(text.clone() + "{}")).is_err());
    assert!(TrackDesign::from_json(&text[..text.len() - 1]).is_err());
}

#[test]
fn codec_rejects_wrong_format_versions_and_bounded_collections_before_use() {
    let design =
        TrackDesign::from_draft("Bounded", &[piece(1, 1, 8, TrackBuildKind::Straight)]).unwrap();
    let base: Value = serde_json::from_str(&design.to_json().unwrap()).unwrap();
    let mut wrong = base.clone();
    wrong["format"] = json!("parkbound-save");
    assert_eq!(
        TrackDesign::from_json(&wrong.to_string()),
        Err(TrackDesignError::InvalidFormat)
    );
    wrong = base.clone();
    wrong["version"] = json!(2);
    assert_eq!(
        TrackDesign::from_json(&wrong.to_string()),
        Err(TrackDesignError::UnsupportedVersion(2))
    );
    wrong = base.clone();
    wrong["pieces"] = json!(vec![base["pieces"][0].clone(); MAX_TRACK_DRAFT_PIECES + 1]);
    assert!(matches!(
        TrackDesign::from_json(&wrong.to_string()),
        Err(TrackDesignError::Json(_))
    ));
    let oversized = " ".repeat(MAX_TRACK_DESIGN_BYTES + 1);
    assert_eq!(
        TrackDesign::from_json(&oversized),
        Err(TrackDesignError::TooLarge {
            actual: MAX_TRACK_DESIGN_BYTES + 1,
            max: MAX_TRACK_DESIGN_BYTES
        })
    );
    wrong = base;
    wrong["pieces"][0]["unknown"] = json!("x");
    let key = "x".repeat(2_000);
    wrong.as_object_mut().unwrap().insert(key, json!(0));
    if let Err(TrackDesignError::Json(message)) = TrackDesign::from_json(&wrong.to_string()) {
        assert!(message.chars().count() <= 512);
    } else {
        panic!("unknown field accepted");
    }
}

#[test]
fn design_is_independent_of_paid_state_and_reuse_defers_charges_to_apply() {
    let specs = loop_specs();
    let mut source = Park::new(32, 32);
    source.replace_track_draft(&specs).unwrap();
    let unpaid = TrackDesign::from_draft("Reusable", source.track_draft()).unwrap();
    source.apply_track_draft().unwrap();
    assert!(!source.track_economy().paid_pieces.is_empty());
    let before = source.clone();
    let paid = TrackDesign::from_draft("Reusable", source.track_draft()).unwrap();
    assert_eq!(source, before);
    assert_eq!(paid, unpaid);
    assert_eq!(paid.to_json().unwrap(), unpaid.to_json().unwrap());
    let mut target = Park::new(32, 32);
    let original = target.clone();
    let moved = paid.transformed(TilePos { x: 15, y: 15 }, 10, 1).unwrap();
    target.replace_track_draft(&moved).unwrap();
    let mut expected = original.clone();
    expected.track_draft = moved;
    assert_eq!(target, expected);
    let required = target.track_construction_quote().unwrap().required_cents();
    assert!(required > 0);
    target.apply_track_draft().unwrap();
    assert_eq!(target.cash_cents(), original.cash_cents() - required);
    let value: Value = serde_json::from_slice(&target.to_save_bytes().unwrap()).unwrap();
    assert_eq!(value["schema_version"], 16);
    assert_eq!(value["content_version"], 16);
}

#[test]
fn incomplete_and_obstructed_designs_remain_reusable_but_apply_rejects_atomically() {
    let specs = loop_specs();
    let design = TrackDesign::from_draft("Blocked later", &specs).unwrap();
    let mut park = Park::new(32, 32);
    park.place_object(specs[2].pos, ObjectKind::Track, Axis::Y, 8, 2)
        .unwrap();
    park.replace_track_draft(design.pieces()).unwrap();
    let before = park.clone();
    assert!(park.apply_track_draft().is_err());
    assert_eq!(park, before);
    let draft = TrackDesign::from_draft("Unfinished", &specs[..3]).unwrap();
    park.replace_track_draft(draft.pieces()).unwrap();
    let before = park.clone();
    assert!(park.apply_track_draft().is_err());
    assert_eq!(park, before);
    assert_eq!(
        TrackDesign::from_json(&draft.to_json().unwrap()).unwrap(),
        draft
    );
}
