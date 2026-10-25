//! Real loan commands, unpaid interest, partial collections and financing reports.
use super::*;
use crate::{
    save_check::loans,
    ui::finance::{FinanceView, LoanCommand},
};

pub(super) fn prepare(p: &mut Park, view: &mut ViewState, scene: Scene) {
    *p = Park::new(32, 32);
    if matches!(scene, Scene::LoanBorrowed) {
        loans::command(p, LoanCommand::Borrow).unwrap();
    } else if matches!(scene, Scene::LoanLimit) {
        p.borrow_loan(park_sim::MAX_LOAN_CENTS).unwrap();
        view.finance_loan_notice = Some(loans::command(p, LoanCommand::Borrow).unwrap());
    } else if matches!(scene, Scene::LoanPartial) {
        *p = loans::partial().unwrap();
    } else if !matches!(scene, Scene::LoanEmpty) {
        *p = loans::install().unwrap();
        if !matches!(scene, Scene::LoanDue) {
            loans::recover(p).unwrap();
        }
    }
    view.help = false;
    view.rides_open = false;
    view.guests_open = false;
    view.services_open = false;
    view.designs_open = false;
    view.finance_open = true;
    view.finance_view = if matches!(scene, Scene::LoanSummary) {
        FinanceView::ParkSummary
    } else if matches!(scene, Scene::LoanTransactions) {
        FinanceView::Transactions
    } else {
        FinanceView::Loans
    };
    view.finance_view_pending = false;
    view.finance_page = 0;
    view.finance_page_pending = 0;
    view.finance_loan_pending = None;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    crate::save_check::summary::check(p).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loan_captures_use_actual_funding_arrears_and_payments() {
        for scene in [
            Scene::LoanEmpty,
            Scene::LoanBorrowed,
            Scene::LoanDue,
            Scene::LoanRepaid,
            Scene::LoanPartial,
            Scene::LoanSummary,
            Scene::LoanTransactions,
            Scene::LoanLimit,
        ] {
            let mut p = crate::demonstration_park();
            let mut view = ViewState::default();
            prepare(&mut p, &mut view, scene);
            assert_eq!(p.finance_summary().operating_profit_cents, 0);
            if matches!(scene, Scene::LoanDue | Scene::LoanPartial) {
                assert_eq!(p.loan_stats().interest_due_cents, 50);
                assert_eq!(p.cash_cents(), 0);
            }
            p.to_save_bytes().unwrap();
        }
    }
}
