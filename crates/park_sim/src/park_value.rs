//! Recorded book values, not market prices or a forecast of earning capacity.
use crate::Park;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParkValue {
    pub track_cents: u64,
    pub carousel_cents: u64,
    pub service_cents: u64,
    pub park_cents: u64,
    pub cash_cents: u64,
    pub principal_cents: u64,
    pub interest_due_cents: u64,
    pub gross_assets_cents: i128,
    pub company_cents: i128,
    pub paid_track_pieces: usize,
    pub installed_carousels: usize,
    pub installed_services: usize,
}

impl Park {
    pub fn park_value(&self) -> ParkValue {
        let track_cents = self
            .track_economy
            .paid_pieces
            .iter()
            .map(|p| p.paid_cents)
            .sum();
        let (carousel_cents, installed_carousels) = self
            .carousels
            .iter()
            .filter(|c| !c.objects.is_empty())
            .fold((0, 0), |(sum, n), c| {
                (sum + c.construction_cost_cents, n + 1)
            });
        let (service_cents, installed_services) = self
            .services
            .iter()
            .filter(|s| s.object.is_some())
            .fold((0, 0), |(sum, n), s| {
                (sum + s.construction_cost_cents, n + 1)
            });
        let park_cents = track_cents + carousel_cents + service_cents;
        let loan = self.loan_stats();
        let gross_assets_cents = i128::from(self.cash_cents) + i128::from(park_cents);
        ParkValue {
            track_cents,
            carousel_cents,
            service_cents,
            park_cents,
            cash_cents: self.cash_cents,
            principal_cents: loan.principal_cents,
            interest_due_cents: loan.interest_due_cents,
            gross_assets_cents,
            company_cents: gross_assets_cents
                - i128::from(loan.principal_cents)
                - i128::from(loan.interest_due_cents),
            paid_track_pieces: self.track_economy.paid_pieces.len(),
            installed_carousels,
            installed_services,
        }
    }
}

#[cfg(test)]
mod tests;
