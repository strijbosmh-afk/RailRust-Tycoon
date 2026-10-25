use super::*;
use crate::Speed;

#[test]
fn borrowing_and_repayment_are_atomic_funding_without_operating_profit() {
    let mut park = Park::new(32, 32);
    let before = park.clone();
    for amount in [0, 1, 9999, 200001] {
        assert!(park.borrow_loan(amount).is_err());
        assert_eq!(park, before);
    }
    park.borrow_loan(20_000).unwrap();
    assert_eq!(park.cash_cents(), 120_000);
    assert_eq!(park.finance_summary().operating_profit_cents, 0);
    park.repay_loan(10_000).unwrap();
    assert_eq!(park.cash_cents(), 110_000);
    assert_eq!(park.loan_stats().principal_cents, 10_000);
    assert_eq!(
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
        park
    );
}

#[test]
fn prorated_interest_is_preserved_through_pause_repayment_and_restart() {
    let mut park = Park::new(32, 32);
    park.borrow_loan(10_000).unwrap();
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.loan_stats().interest_due_cents, 25);
    assert_eq!(park.loan_stats().interest_paid_cents, 0);
    let before = park.clone();
    park.advance(Speed::Paused);
    assert_eq!(park, before);
    park = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
    park.repay_loan(10_000).unwrap();
    assert_eq!(park.cash_cents(), 99_975);
    assert_eq!(park.loan_stats().interest_paid_cents, 25);
    assert_eq!(park.loan_stats().principal_cents, 0);
    for _ in 0..900 {
        park.advance(Speed::Normal);
    }
    assert_eq!(park.cash_cents(), 99_975);
}

fn cashless() -> Park {
    let mut p = Park::new(32, 32);
    p.borrow_loan(10_000).unwrap();
    p.place_carousel(crate::TilePos { x: 19, y: 22 }, 0)
        .unwrap();
    for x in (2..=16).step_by(2) {
        p.place_service(
            crate::TilePos { x, y: 2 },
            0,
            crate::PathDirection::North,
            crate::ServiceKind::Toilet,
        )
        .unwrap();
    }
    assert_eq!(p.cash_cents(), 0);
    p
}

#[test]
fn unpaid_interest_survives_restart_and_repayment_requires_all_due_interest() {
    let mut p = cashless();
    for _ in 0..1800 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.loan_stats().interest_due_cents, 50);
    assert_eq!(p.loan_stats().interest_paid_cents, 0);
    p = Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap();
    p.borrow_loan(10_000).unwrap();
    let before = p.clone();
    assert_eq!(p.repay_loan(10_000), Err(LoanError::Funds));
    assert_eq!(p, before);
    assert_eq!(p.pay_loan_interest(), Ok(50));
    assert_eq!(p.cash_cents(), 9950);
    assert_eq!(p.finance_summary().operating_profit_cents, 0);
    assert_eq!(p.finance_summary().loan_interest_cents, 50);
    p.to_save_bytes().unwrap();
}

#[test]
fn subcent_interest_cannot_be_reset_by_repaying_or_reloading() {
    let mut p = Park::new(32, 32);
    for _ in 0..36 {
        p.borrow_loan(10_000).unwrap();
        p.advance(Speed::Normal);
        p.repay_loan(10_000).unwrap();
        p = Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap();
    }
    assert_eq!(p.loan_stats().interest_accrued_cents, 1);
    assert_eq!(p.loan_stats().interest_paid_cents, 1);
    assert_eq!(p.cash_cents(), 99_999);
    assert_eq!(p.loan_stats().principal_cents, 0);
}

#[test]
fn limits_and_archived_financing_reconcile_without_becoming_owner_income() {
    let mut p = Park::new(32, 32);
    for _ in 0..160 {
        p.borrow_loan(MAX_LOAN_CENTS).unwrap();
        let before = p.clone();
        assert_eq!(p.borrow_loan(LOAN_STEP_CENTS), Err(LoanError::Limit));
        assert_eq!(p, before);
        p.repay_loan(MAX_LOAN_CENTS).unwrap();
    }
    assert!(p.finance_ledger().archived_entries > 0);
    assert!(
        p.finance_ledger().archived_rides.is_empty()
            && p.finance_ledger().archived_services.is_empty()
    );
    let r = p.finance_summary();
    assert_eq!(r.operating_profit_cents, 0);
    assert_eq!(r.net_cash_cents, 0);
    assert_eq!(r.loan_borrowed_cents, 32_000_000);
    assert_eq!(r.loan_repaid_cents, 32_000_000);
    assert_eq!(
        Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap(),
        p
    );
}

#[test]
fn fixed_speed_grouping_and_collection_priority_are_deterministic() {
    let mut a = Park::new(32, 32);
    a.borrow_loan(MAX_LOAN_CENTS).unwrap();
    let mut b = a.clone();
    for _ in 0..1800 {
        a.advance(Speed::Normal);
    }
    for _ in 0..450 {
        b.advance(Speed::Fastest);
    }
    assert_eq!(a, b);
    assert_eq!(a.loan_stats().interest_paid_cents, 1000);
    assert_eq!(a.finance_summary().operating_profit_cents, 0);
}

#[test]
fn current_save_rejects_forged_missing_or_misattributed_financing() {
    let mut p = Park::new(32, 32);
    p.borrow_loan(10_000).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&p.to_save_bytes().unwrap()).unwrap();
    for (path, bad) in [
        ("/park/loans/principal_cents", serde_json::json!(20_000)),
        ("/park/loans/principal_cents", serde_json::json!(u64::MAX)),
        ("/park/loans/interest_fraction", serde_json::json!(360000)),
        ("/park/loans/interest_accrued_cents", serde_json::json!(1)),
        ("/park/finances/entries/0/ride", serde_json::json!(1)),
        (
            "/park/finances/entries/0/amount_cents",
            serde_json::json!(9999),
        ),
        (
            "/park/finances/archived/loan_borrowed_cents",
            serde_json::json!(10000),
        ),
    ] {
        let mut v = value.clone();
        *v.pointer_mut(path).unwrap() = bad;
        assert!(
            Park::from_save_bytes(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{path}"
        );
    }
    let mut v = value;
    v["park"].as_object_mut().unwrap().remove("loans");
    assert!(Park::from_save_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
}

#[test]
fn authentic_v20_preserves_every_old_field_and_starts_without_debt() {
    let old: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../testdata/v20-paid-summary.park.json"))
            .unwrap();
    let park =
        Park::from_save_bytes(include_bytes!("../../testdata/v20-paid-summary.park.json")).unwrap();
    let modern: serde_json::Value = serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
    fn preserved(old: &serde_json::Value, new: &serde_json::Value) {
        match old {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    preserved(v, &new[k]);
                }
            }
            serde_json::Value::Array(rows) => {
                assert_eq!(rows.len(), new.as_array().unwrap().len());
                for (a, b) in rows.iter().zip(new.as_array().unwrap()) {
                    preserved(a, b);
                }
            }
            _ => assert_eq!(old, new),
        }
    }
    preserved(&old["park"], &modern["park"]);
    assert_eq!(park.loan_stats().principal_cents, 0);
    assert_eq!(park.loan_stats().interest_accrued_cents, 0);
    assert!(
        park.finance_summary().rides.revenue_cents > 0
            && park.finance_summary().services.revenue_cents > 0
    );
}

#[test]
fn partial_collection_retains_arrears_without_compounding() {
    let mut p = cashless();
    for _ in 0..1800 {
        p.advance(Speed::Normal);
    }
    p.borrow_loan(10_000).unwrap();
    p.pay_loan_interest().unwrap();
    assert_eq!(p.cash_cents(), 9950);
    for _ in 0..180_000 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.cash_cents(), 0);
    assert_eq!(p.loan_stats().interest_due_cents, 50);
    for _ in 0..1800 {
        p.advance(Speed::Normal);
    }
    assert_eq!(p.loan_stats().interest_due_cents, 150);
    assert_eq!(
        Park::from_save_bytes(&p.to_save_bytes().unwrap()).unwrap(),
        p
    );
}

#[test]
fn interest_collection_precedes_ride_upkeep_on_the_same_tick() {
    let mut p = Park::new(32, 32);
    p.install_train_fixture().unwrap();
    p.borrow_loan(10_000).unwrap();
    for _ in 0..1800 {
        p.advance(Speed::Normal);
    }
    let rows = &p.finance_ledger().entries;
    assert_eq!(rows[rows.len() - 2].kind, FinanceKind::LoanInterest);
    assert_eq!(rows[rows.len() - 1].kind, FinanceKind::RideUpkeep);
    assert_eq!(rows[rows.len() - 2].tick, rows[rows.len() - 1].tick);
}
