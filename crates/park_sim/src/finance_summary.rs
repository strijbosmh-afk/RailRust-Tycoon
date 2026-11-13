//! Read-only lifetime operating results, separate from capital cash movements.
use crate::Park;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OperatingProfit {
    pub revenue_cents: u64,
    pub upkeep_cents: u64,
    pub supplies_cents: u64,
    pub profit_cents: i128,
}

impl OperatingProfit {
    fn new(revenue_cents: u64, upkeep_cents: u64, supplies_cents: u64) -> Self {
        Self {
            revenue_cents,
            upkeep_cents,
            supplies_cents,
            profit_cents: i128::from(revenue_cents)
                - i128::from(upkeep_cents)
                - i128::from(supplies_cents),
        }
    }
}

/// Recorded lifetime amounts include removed owners and archived transactions.
/// These are neither asset values nor a forecast or a monthly income statement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParkFinanceSummary {
    pub research_cents: u64,
    pub promotion_cents: u64,
    pub rides: OperatingProfit,
    pub services: OperatingProfit,
    pub admission_revenue_cents: u64,
    pub advertising_cents: u64,
    pub operating_profit_cents: i128,
    pub staff_wages_cents: u64,
    pub construction_cents: u64,
    pub refunds_cents: u64,
    pub loan_borrowed_cents: u64,
    pub loan_repaid_cents: u64,
    pub loan_interest_cents: u64,
    pub net_cash_cents: i128,
    pub starting_cash_cents: u64,
    pub cash_cents: u64,
}

impl Park {
    pub fn finance_summary(&self) -> ParkFinanceSummary {
        let stats = self.finance_stats();
        let rides = OperatingProfit::new(stats.ride_revenue_cents, stats.ride_upkeep_cents, 0);
        let services = OperatingProfit::new(
            stats.service_revenue_cents,
            stats.service_upkeep_cents,
            stats.supplies_cents,
        );
        let operating_profit_cents =
            rides.profit_cents + services.profit_cents + i128::from(stats.admission_revenue_cents)
                - i128::from(stats.wages_cents)
                - i128::from(stats.advertising_cents)
                - i128::from(stats.research_cents)
                - i128::from(stats.promotion_cents);
        ParkFinanceSummary {
            research_cents: stats.research_cents,
            promotion_cents: stats.promotion_cents,
            rides,
            services,
            admission_revenue_cents: stats.admission_revenue_cents,
            advertising_cents: stats.advertising_cents,
            operating_profit_cents,
            staff_wages_cents: stats.wages_cents,
            construction_cents: stats.construction_cents,
            refunds_cents: stats.demolition_refunds_cents,
            loan_borrowed_cents: self.loans.borrowed_cents,
            loan_repaid_cents: self.loans.repaid_cents,
            loan_interest_cents: self.loans.interest_paid_cents,
            net_cash_cents: operating_profit_cents + i128::from(stats.demolition_refunds_cents)
                - i128::from(stats.construction_cents)
                + i128::from(self.loans.borrowed_cents)
                - i128::from(self.loans.repaid_cents)
                - i128::from(self.loans.interest_paid_cents),
            starting_cash_cents: self.starting_cash_cents(),
            cash_cents: self.cash_cents(),
        }
    }
}

#[cfg(test)]
mod tests;
