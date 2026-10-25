use super::*;
use crate::{PathDirection, ServiceKind, Speed, TilePos, TrackDesign, TrackKind};

fn paid_track() -> Park {
    let mut p = Park::new(32, 32);
    let d = TrackDesign::from_json(include_str!(
        "../../../../designs/pocket-oval.parktrack.json"
    ))
    .unwrap();
    p.replace_track_draft(&d.transformed(TilePos { x: 16, y: 16 }, 8, 0).unwrap())
        .unwrap();
    assert_eq!(p.park_value().park_cents, 0); // An unpaid draft is not an asset.
    p.apply_track_draft().unwrap();
    p
}

#[test]
fn borrowing_and_repayment_do_not_create_equity_or_operating_profit() {
    let mut p = Park::new(32, 32);
    let original = p.park_value();
    assert_eq!(original.park_cents, 0);
    assert_eq!(original.company_cents, 100_000);
    p.borrow_loan(10_000).unwrap();
    assert_eq!(p.park_value().company_cents, original.company_cents);
    p.repay_loan(10_000).unwrap();
    assert_eq!(p.park_value(), original);
    assert_eq!(p.finance_summary().operating_profit_cents, 0);
}

#[test]
fn accrued_interest_reduces_equity_once_and_payment_only_moves_cash() {
    let mut p = Park::new(32, 32);
    p.borrow_loan(10_000).unwrap();
    for _ in 0..900 {
        p.advance(Speed::Normal);
    }
    let before = p.park_value();
    assert_eq!(before.interest_due_cents, 25);
    assert_eq!(before.company_cents, 99_975);
    p.pay_loan_interest().unwrap();
    let after = p.park_value();
    assert_eq!(after.company_cents, before.company_cents);
    assert_eq!(after.cash_cents, before.cash_cents - 25);
    assert_eq!(after.interest_due_cents, 0);
    assert_eq!(p.finance_summary().operating_profit_cents, 0);
}

#[test]
fn purchases_exchange_cash_for_assets_and_demolition_records_only_the_remaining_basis() {
    let mut p = Park::new(32, 32);
    let ride = p.place_carousel(TilePos { x: 19, y: 22 }, 0).unwrap();
    let service = p
        .place_service(
            TilePos { x: 4, y: 5 },
            0,
            PathDirection::North,
            ServiceKind::Drink,
        )
        .unwrap();
    assert_eq!(ride.0, service.0);
    let v = p.park_value();
    assert_eq!(
        (v.carousel_cents, v.service_cents, v.company_cents),
        (30_000, 8_000, 100_000)
    );
    p.remove_carousel(ride).unwrap();
    assert_eq!(p.park_value().company_cents, 85_000);
    p.remove_object(p.service(service).unwrap().object.unwrap())
        .unwrap();
    let v = p.park_value();
    assert_eq!(v.park_cents, 0);
    assert_eq!(v.company_cents, 81_000);
    assert_eq!(v.installed_carousels, 0);
    assert_eq!(v.installed_services, 0);
}

#[test]
fn faulted_paid_remnants_and_replacements_reconcile_with_track_settlements() {
    let mut p = paid_track();
    let original = p.park_value();
    assert_eq!(original.track_cents, 27_000);
    assert_eq!(original.company_cents, 100_000);
    let spec = *p
        .circuit()
        .unwrap()
        .specs
        .iter()
        .find(|s| s.kind == TrackKind::Straight)
        .unwrap();
    p.copy_installed_track_to_draft().unwrap();
    let mut replacement = spec;
    replacement.kind = TrackKind::Brake;
    p.set_track_draft_piece(replacement).unwrap();
    p.apply_track_draft().unwrap();
    assert_eq!(p.park_value().track_cents, 28_000);
    assert_eq!(p.park_value().company_cents, 99_500);
    let paid = p
        .track_economy()
        .paid_pieces
        .iter()
        .find(|r| r.spec == replacement)
        .unwrap()
        .clone();
    p.remove_object(paid.object).unwrap();
    assert!(p.circuit().is_none());
    let v = p.park_value();
    assert_eq!(v.track_cents, 26_000);
    assert_eq!(v.paid_track_pieces, 15);
    assert_eq!(v.company_cents, 98_500);
    assert_eq!(
        v.track_cents,
        p.track_economy().construction_cents - 2 * p.track_economy().refund_cents
    );
    assert_eq!(
        crate::Park::from_save_bytes(&p.to_save_bytes().unwrap())
            .unwrap()
            .park_value(),
        v
    );
}

#[test]
fn book_value_can_be_negative_without_negative_cash_and_queries_are_immutable() {
    let mut p = Park::new(32, 32);
    p.borrow_loan(20_000).unwrap();
    for _ in 0..7 {
        let id = p.place_carousel(TilePos { x: 19, y: 22 }, 0).unwrap();
        p.remove_carousel(id).unwrap();
    }
    let bytes = p.to_save_bytes().unwrap();
    let v = p.park_value();
    assert_eq!(v.cash_cents, 15_000);
    assert_eq!(v.park_cents, 0);
    assert_eq!(v.company_cents, -5_000);
    assert_eq!(p.to_save_bytes().unwrap(), bytes);
    assert_eq!(Park::from_save_bytes(&bytes).unwrap().park_value(), v);
}

#[test]
fn free_legacy_track_has_no_invented_paid_value_and_old_receipts_stay_intact() {
    let mut p = Park::new(32, 32);
    p.install_train_fixture().unwrap();
    p.set_train_open(false);
    let before = p.to_save_bytes().unwrap();
    assert_eq!(p.park_value().park_cents, 0);
    assert_eq!(p.park_value().company_cents, 100_000);
    assert_eq!(p.to_save_bytes().unwrap(), before);
    let old =
        Park::from_save_bytes(include_bytes!("../../testdata/v20-paid-summary.park.json")).unwrap();
    assert_eq!(old.park_value().track_cents, 27_000);
    assert_eq!(old.park_value().carousel_cents, 30_000);
    assert_eq!(old.park_value().service_cents, 10_000);
    assert_eq!(old.park_value().principal_cents, 0);
}
