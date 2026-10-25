//! Bounded principal, prorated simple interest and explicit financing cashflows.
use crate::{FinanceKind, Park};

pub const LOAN_STEP_CENTS: u64 = 10_000;
pub const MAX_LOAN_CENTS: u64 = 200_000;
pub const LOAN_INTEREST_TICKS: u64 = 1_800;
// 0.5% per simulation minute. Retaining the fraction prevents close/repay cycling
// from discarding accrued sub-cent interest. Principal is always below divisor.
const MAX_LIFETIME_BORROWED: u64 = (u64::MAX / 16 / LOAN_STEP_CENTS) * LOAN_STEP_CENTS;
const INTEREST_DIVISOR: u32 = 360_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LoanAccount {
    pub principal_cents: u64,
    pub borrowed_cents: u64,
    pub repaid_cents: u64,
    pub interest_accrued_cents: u64,
    pub interest_paid_cents: u64,
    pub interest_fraction: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoanStats {
    pub principal_cents: u64,
    pub available_cents: u64,
    pub borrowed_cents: u64,
    pub repaid_cents: u64,
    pub interest_accrued_cents: u64,
    pub interest_paid_cents: u64,
    pub interest_due_cents: u64,
    pub interest_per_minute_cents: u64,
    pub next_collection_ticks: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoanError {
    InvalidAmount,
    Limit,
    Principal,
    Funds,
    NothingDue,
    Overflow,
}
impl std::fmt::Display for LoanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidAmount => "Use positive $100 steps",
            Self::Limit => "Loan limit is $2000",
            Self::Principal => "Repayment exceeds the loan balance",
            Self::Funds => "Need cash for repayment and all accrued interest",
            Self::NothingDue => "No payable interest or no cash available",
            Self::Overflow => "Loan transaction exceeds accounting limits",
        })
    }
}
impl std::error::Error for LoanError {}

impl Park {
    pub fn loan_stats(&self) -> LoanStats {
        let a = self.loans;
        LoanStats {
            principal_cents: a.principal_cents,
            available_cents: MAX_LOAN_CENTS - a.principal_cents,
            borrowed_cents: a.borrowed_cents,
            repaid_cents: a.repaid_cents,
            interest_accrued_cents: a.interest_accrued_cents,
            interest_paid_cents: a.interest_paid_cents,
            interest_due_cents: a.interest_accrued_cents - a.interest_paid_cents,
            interest_per_minute_cents: a.principal_cents / 200,
            next_collection_ticks: LOAN_INTEREST_TICKS - self.ticks % LOAN_INTEREST_TICKS,
        }
    }
    pub fn borrow_loan(&mut self, amount: u64) -> Result<(), LoanError> {
        if amount == 0 || !amount.is_multiple_of(LOAN_STEP_CENTS) {
            return Err(LoanError::InvalidAmount);
        }
        if amount > self.loan_stats().available_cents {
            return Err(LoanError::Limit);
        }
        let borrowed = self
            .loans
            .borrowed_cents
            .checked_add(amount)
            .ok_or(LoanError::Overflow)?;
        if borrowed > MAX_LIFETIME_BORROWED {
            return Err(LoanError::Overflow);
        }
        self.cash_cents
            .checked_add(amount)
            .ok_or(LoanError::Overflow)?;
        self.loans.borrowed_cents = borrowed;
        self.loans.principal_cents += amount;
        self.record_cash(FinanceKind::LoanBorrow, None, amount);
        Ok(())
    }
    /// Settles whole-cent accrued interest before principal, atomically. The
    /// saved sub-cent remainder survives even a complete principal repayment.
    pub fn repay_loan(&mut self, amount: u64) -> Result<(), LoanError> {
        if amount == 0 || !amount.is_multiple_of(LOAN_STEP_CENTS) {
            return Err(LoanError::InvalidAmount);
        }
        if amount > self.loans.principal_cents {
            return Err(LoanError::Principal);
        }
        let due = self.loan_stats().interest_due_cents;
        if amount
            .checked_add(due)
            .is_none_or(|need| need > self.cash_cents)
        {
            return Err(LoanError::Funds);
        }
        let repaid = self
            .loans
            .repaid_cents
            .checked_add(amount)
            .ok_or(LoanError::Overflow)?;
        self.pay_loan_interest_amount(due);
        self.loans.repaid_cents = repaid;
        self.loans.principal_cents -= amount;
        self.record_cash(FinanceKind::LoanRepayment, None, amount);
        Ok(())
    }
    pub fn pay_loan_interest(&mut self) -> Result<u64, LoanError> {
        let amount = self.cash_cents.min(self.loan_stats().interest_due_cents);
        if amount == 0 {
            return Err(LoanError::NothingDue);
        }
        self.pay_loan_interest_amount(amount);
        Ok(amount)
    }
    fn pay_loan_interest_amount(&mut self, amount: u64) {
        self.loans.interest_paid_cents += amount;
        self.record_cash(FinanceKind::LoanInterest, None, amount);
    }
    pub(crate) fn step_loan_interest(&mut self) {
        let fraction = u64::from(self.loans.interest_fraction) + self.loans.principal_cents;
        self.loans.interest_accrued_cents += fraction / u64::from(INTEREST_DIVISOR);
        self.loans.interest_fraction = (fraction % u64::from(INTEREST_DIVISOR)) as u32;
        if self.ticks.is_multiple_of(LOAN_INTEREST_TICKS) {
            let amount = self.cash_cents.min(self.loan_stats().interest_due_cents);
            self.pay_loan_interest_amount(amount);
        }
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    let a = park.loans;
    if a.borrowed_cents > MAX_LIFETIME_BORROWED
        || a.principal_cents > MAX_LOAN_CENTS
        || !a.principal_cents.is_multiple_of(LOAN_STEP_CENTS)
        || !a.borrowed_cents.is_multiple_of(LOAN_STEP_CENTS)
        || !a.repaid_cents.is_multiple_of(LOAN_STEP_CENTS)
        || a.borrowed_cents.checked_sub(a.repaid_cents) != Some(a.principal_cents)
        || a.interest_paid_cents > a.interest_accrued_cents
        || (a.principal_cents == 0 && a.interest_paid_cents != a.interest_accrued_cents)
        || a.interest_fraction >= INTEREST_DIVISOR
        || u128::from(a.interest_accrued_cents) * u128::from(INTEREST_DIVISOR)
            + u128::from(a.interest_fraction)
            > u128::from(park.ticks) * u128::from(MAX_LOAN_CENTS)
        || (a.borrowed_cents == 0 && a != LoanAccount::default())
    {
        return Err("loan principal, lifetime counters or interest accrual is invalid".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
