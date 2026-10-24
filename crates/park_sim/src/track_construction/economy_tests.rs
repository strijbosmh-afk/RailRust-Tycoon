use super::tests::{draft, graded_loop_specs, json_value, loop_specs, read, roundtrip};
use super::*;
use crate::{FinanceKind, Speed};
use serde_json::json;

fn paid_loop() -> Park {
    let mut park = Park::new(32, 32);
    draft(&mut park, &loop_specs(3, 3, 8));
    park.apply_track_draft().unwrap();
    park
}

#[test]
fn every_price_is_fixed_and_quotes_are_pure_with_net_cash_and_exact_retention() {
    let mut park = Park::new(32, 32);
    let specs = graded_loop_specs();
    draft(&mut park, &specs);
    let before = park.clone();
    let quote = park.track_construction_quote().unwrap();
    assert_eq!(park, before);
    assert_eq!(quote.construction_cents, 38_000);
    assert_eq!(quote.refund_cents, 0);
    assert_eq!(quote.retained_pieces, 0);
    assert_eq!(quote.new_pieces, 20);
    assert_eq!(quote.balance_after(), Some(62_000));
    park.apply_track_draft().unwrap();
    let id = park.circuit().unwrap().pieces[11].object;
    let mut changed = specs[11];
    changed.kind = TrackKind::Straight;
    park.set_track_draft_piece(changed).unwrap();
    let quote = park.track_construction_quote().unwrap();
    assert_eq!(
        (quote.construction_cents, quote.refund_cents),
        (1_000, 1_000)
    );
    assert_eq!(
        (
            quote.retained_pieces,
            quote.new_pieces,
            quote.removed_pieces
        ),
        (19, 1, 1)
    );
    let old_ids: Vec<_> = park
        .circuit()
        .unwrap()
        .pieces
        .iter()
        .map(|piece| piece.object)
        .collect();
    park.apply_track_draft().unwrap();
    assert!(park.object(id).is_none());
    for (index, piece) in park.circuit().unwrap().pieces.iter().enumerate() {
        assert_eq!(piece.object == old_ids[index], index != 11);
    }
    assert_eq!(park.cash_cents(), 62_000);
    roundtrip(&park);
}

#[test]
fn unaffordable_apply_is_atomic_but_guaranteed_refund_can_fund_replacement() {
    let mut park = paid_loop();
    park.cash_cents = 0; // Isolate the command's affordability boundary.
    park.next_object_id = u64::MAX;
    let before = park.clone();
    park.apply_track_draft().unwrap();
    assert_eq!(park, before);
    park.next_object_id = 100;
    let mut spec = park.circuit().unwrap().specs[2];
    spec.kind = TrackKind::Brake;
    park.set_track_draft_piece(spec).unwrap();
    let quote = park.track_construction_quote().unwrap();
    assert_eq!(quote.required_cents(), 1_500);
    assert_eq!(quote.balance_after(), None);
    let before = park.clone();
    assert_eq!(
        park.apply_track_draft(),
        Err(TrackConstructionError::InsufficientFunds {
            required_cents: 1_500,
            available_cents: 0
        })
    );
    assert_eq!(park, before);
    park.cash_cents = 1_500;
    park.apply_track_draft().unwrap();
    assert_eq!(park.cash_cents(), 0);
    spec.kind = TrackKind::Straight;
    park.set_track_draft_piece(spec).unwrap();
    assert_eq!(park.track_construction_quote().unwrap().required_cents(), 0);
    park.apply_track_draft().unwrap();
    assert_eq!(park.cash_cents(), 0);
    let entries = &park.finances.entries;
    assert_eq!(entries[entries.len() - 2].kind, FinanceKind::TrackRefund);
    assert_eq!(entries[entries.len() - 2].balance_cents, 1_000);
    assert_eq!(entries.last().unwrap().kind, FinanceKind::TrackConstruction);
    assert_eq!(entries.last().unwrap().balance_cents, 0);
}

#[test]
fn paid_remnants_keep_single_refunds_through_fault_save_and_reload() {
    let mut park = paid_loop();
    let ids: Vec<_> = park
        .circuit()
        .unwrap()
        .pieces
        .iter()
        .map(|piece| piece.object)
        .collect();
    assert_eq!(park.track_demolition_refund_cents(ids[0]), 5_000);
    let cash = park.cash_cents();
    park.remove_object(ids[0]).unwrap();
    assert!(park.circuit().is_none());
    assert_eq!(park.train().unwrap().phase, TrainPhase::Fault);
    assert_eq!(park.cash_cents(), cash + 5_000);
    let before = park.clone();
    assert!(park.remove_object(ids[0]).is_none());
    assert_eq!(park, before);
    let mut park = roundtrip(&park);
    for id in &ids[1..] {
        assert!(park.track_demolition_refund_cents(*id) > 0);
        park.remove_object(*id).unwrap();
        roundtrip(&park);
    }
    assert!(park.track_economy.paid_pieces.is_empty());
    assert_eq!(park.track_economy.refund_cents, 9_500);
    assert_eq!(park.cash_cents(), 90_500);
    assert!(
        park.finances
            .entries
            .iter()
            .all(|entry| entry.ride == Some(park.ride().unwrap().id))
    );
}

#[test]
fn repeated_replacements_archive_without_profit_and_continue_after_reload() {
    let mut park = paid_loop();
    let mut spec = park.circuit().unwrap().specs[2];
    for index in 0..70 {
        spec.kind = if index % 2 == 0 {
            TrackKind::Brake
        } else {
            TrackKind::Straight
        };
        park.set_track_draft_piece(spec).unwrap();
        let cash = park.cash_cents();
        park.apply_track_draft().unwrap();
        assert!(park.cash_cents() <= cash);
        roundtrip(&park);
    }
    assert_eq!(park.finances.entries.len(), crate::FINANCE_LEDGER_CAPACITY);
    assert_eq!(park.finances.archived_entries, 13);
    assert_eq!(park.track_economy.construction_cents, 124_000);
    assert_eq!(park.track_economy.refund_cents, 52_500);
    assert_eq!(park.cash_cents(), 28_500);
    park.set_train_open(true);
    let mut loaded = roundtrip(&park);
    for _ in 0..2_000 {
        park.advance(Speed::Normal);
        loaded.advance(Speed::Normal);
    }
    assert_eq!(park, loaded);
    roundtrip(&park);
}

#[test]
fn free_schema_fourteen_track_gets_no_windfall_but_new_replacement_is_paid() {
    let mut park = Park::new(32, 32);
    let specs = loop_specs(3, 3, 8);
    park.install_train_circuit(&specs).unwrap();
    park.set_train_open(false);
    draft(&mut park, &specs);
    let mut old = json_value(&park);
    old["schema_version"] = json!(14);
    old["content_version"] = json!(14);
    old["park"].as_object_mut().unwrap().remove("track_economy");
    let mut migrated = read(&old).unwrap();
    assert_eq!(migrated, park);
    assert_eq!(
        migrated
            .track_construction_quote()
            .unwrap()
            .construction_cents,
        0
    );
    let mut spec = specs[2];
    spec.kind = TrackKind::Brake;
    migrated.set_track_draft_piece(spec).unwrap();
    let quote = migrated.track_construction_quote().unwrap();
    assert_eq!(
        (
            quote.construction_cents,
            quote.refund_cents,
            quote.retained_pieces
        ),
        (2_000, 0, 7)
    );
    migrated.apply_track_draft().unwrap();
    assert_eq!(migrated.track_economy.paid_pieces.len(), 1);
    assert_eq!(migrated.cash_cents(), 98_000);
    roundtrip(&migrated);
    let mut hybrid = old.clone();
    hybrid["park"]["track_economy"] = json!(TrackEconomy::default());
    assert!(read(&hybrid).is_err());
    let mut smuggled = json_value(&migrated);
    smuggled["schema_version"] = json!(14);
    smuggled["content_version"] = json!(14);
    smuggled["park"]
        .as_object_mut()
        .unwrap()
        .remove("track_economy");
    assert!(read(&smuggled).is_err());
}

#[test]
fn paid_basis_and_ledger_tampering_rejects_missing_duplicate_forged_or_wrong_owner_records() {
    let park = paid_loop();
    let base = json_value(&park);
    for (path, bad) in [
        ("/park/track_economy", json!(null)),
        ("/park/track_economy/paid_pieces", json!([])),
        ("/park/track_economy/paid_pieces/0/paid_cents", json!(0)),
        ("/park/track_economy/paid_pieces/0/object", json!(999)),
        (
            "/park/track_economy/paid_pieces/0/spec/kind",
            json!("Brake"),
        ),
        ("/park/track_economy/construction_cents", json!(19_500)),
        ("/park/track_economy/refund_cents", json!(250)),
        ("/park/finances/entries/0/ride", json!(999)),
        ("/park/finances/entries/0/service", json!(1)),
        ("/park/finances/entries/0/kind", json!("RideConstruction")),
        ("/park/finances/entries/0/kind", json!("TrackRefund")),
    ] {
        let mut forged = base.clone();
        *forged.pointer_mut(path).unwrap() = bad;
        assert!(read(&forged).is_err(), "accepted tampered {path}");
    }
    let mut duplicate = base.clone();
    let record = duplicate["park"]["track_economy"]["paid_pieces"][0].clone();
    duplicate["park"]["track_economy"]["paid_pieces"]
        .as_array_mut()
        .unwrap()
        .push(record);
    assert!(read(&duplicate).is_err());
    let mut missing = base.clone();
    missing["park"]
        .as_object_mut()
        .unwrap()
        .remove("track_economy");
    assert!(read(&missing).is_err());
    let mut unknown = base;
    unknown["park"]["track_economy"]["unexpected"] = json!(1);
    assert!(read(&unknown).is_err());
}

#[test]
fn fixture_recovery_cannot_reinterpret_paid_brakes_as_free_straights() {
    let mut park = paid_loop();
    let mut specs = park.circuit().unwrap().specs.clone();
    specs[2].kind = TrackKind::Brake;
    park.set_track_draft_piece(specs[2]).unwrap();
    park.apply_track_draft().unwrap();
    let removed = park.circuit().unwrap().pieces[0].object;
    park.remove_object(removed).unwrap();
    let mut wrong = specs.clone();
    wrong[2].kind = TrackKind::Straight;
    let before = park.clone();
    assert!(park.install_train_circuit(&wrong).is_err());
    assert_eq!(park, before);
    park.install_train_circuit(&specs).unwrap();
    roundtrip(&park);
    // Restored diagnostic station is free; remaining paid pieces retain basis.
    assert_eq!(park.track_economy.paid_pieces.len(), 7);
}

#[test]
fn forged_conserved_history_cannot_claim_a_purchase_below_the_cheapest_piece() {
    let mut park = Park::new(32, 32);
    park.install_train_circuit(&loop_specs(3, 3, 8)).unwrap();
    let mut forged = json_value(&park);
    // The arithmetic balances exactly, but no $5 piece/$2.50 refund exists.
    forged["park"]["track_economy"]["construction_cents"] = json!(500);
    forged["park"]["track_economy"]["refund_cents"] = json!(250);
    forged["park"]["cash_cents"] = json!(99_750);
    assert!(
        read(&forged)
            .unwrap_err()
            .to_string()
            .contains("track economy")
    );
    // A partial impossible refund also fails when a real live basis remains.
    let paid = paid_loop();
    let mut forged = json_value(&paid);
    forged["park"]["track_economy"]["construction_cents"] = json!(19_500);
    forged["park"]["track_economy"]["refund_cents"] = json!(250);
    forged["park"]["cash_cents"] = json!(80_750);
    assert!(
        read(&forged)
            .unwrap_err()
            .to_string()
            .contains("track economy")
    );
}
