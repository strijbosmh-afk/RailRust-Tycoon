//! Research, prepaid consumables and management accounts share the cash ledger.
use crate::{FinanceKind, Park, RideCategory, RideModel, ServiceId, ServiceKind};

pub const RESEARCH_PROJECT_CENTS: u64 = 4_000;
pub const MANAGEMENT_PERIOD_TICKS: u64 = 1_800;
pub const PROMOTION_COST_CENTS: u64 = 5_000;
pub const PROMOTION_TICKS: u16 = 9_000;
pub const STOCK_CAPACITY: u16 = 100;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ResearchPriority {
    Gentle,
    #[default]
    Thrill,
    Water,
    Transport,
    Coasters,
    Facilities,
}
impl ResearchPriority {
    pub const ALL: [Self; 6] = [
        Self::Gentle,
        Self::Thrill,
        Self::Water,
        Self::Transport,
        Self::Coasters,
        Self::Facilities,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Gentle => "Gentle rides",
            Self::Thrill => "Thrill rides",
            Self::Water => "Water rides",
            Self::Transport => "Transport",
            Self::Coasters => "Coasters",
            Self::Facilities => "Facilities",
        }
    }
    fn index(self) -> usize {
        Self::ALL.iter().position(|p| *p == self).unwrap()
    }
    fn projects(self) -> u64 {
        match self {
            Self::Gentle | Self::Transport => 1,
            Self::Thrill | Self::Facilities => 3,
            _ => 2,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ResearchFunding {
    #[default]
    Paused,
    Low,
    Medium,
    High,
}
impl ResearchFunding {
    pub fn next(self) -> Self {
        match self {
            Self::Paused => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Paused,
        }
    }
    pub fn cost(self) -> u64 {
        match self {
            Self::Paused => 0,
            Self::Low => 100,
            Self::Medium => 250,
            Self::High => 500,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Paused => "Paused",
            Self::Low => "$1 / 30s",
            Self::Medium => "$2.50 / 30s",
            Self::High => "$5 / 30s",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Promotion {
    #[default]
    Family,
    Enthusiast,
}
impl Promotion {
    pub fn label(self) -> &'static str {
        match self {
            Self::Family => "Family week",
            Self::Enthusiast => "Coaster enthusiasts",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityStock {
    pub service: ServiceId,
    pub baseline_served: u64,
    pub baseline_supplies_cents: u64,
    pub purchased: u64,
    pub remaining: u16,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagementReport {
    pub tick: u64,
    pub cash_cents: u64,
    pub revenue_cents: u64,
    pub expenses_cents: u64,
    pub capital_net_cents: i128,
    pub financing_net_cents: i128,
    pub debt_cents: u64,
    pub park_value_cents: u64,
    pub rating: u8,
    pub popularity: u8,
    pub reliability: u8,
}
impl ManagementReport {
    pub fn operating_profit_cents(&self) -> i128 {
        i128::from(self.revenue_cents) - i128::from(self.expenses_cents)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Management {
    pub enabled_tick: u64,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub priority: ResearchPriority,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub funding: ResearchFunding,
    pub research_progress: u16,
    pub research_spent: [u64; 6],
    pub automatic_stock: bool,
    pub staff_budget_cents: u64,
    pub payroll_epoch: u64,
    pub payroll_spent_cents: u64,
    #[serde(deserialize_with = "crate::saves::string_enum")]
    pub promotion: Promotion,
    pub promotion_remaining: u16,
    pub promotions_bought: u64,
    pub promotion_candidates: u64,
    pub promotion_admissions: u64,
    pub promotion_revenue_cents: u64,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,256>")]
    pub stocks: Vec<FacilityStock>,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_,_,60>")]
    pub reports: Vec<ManagementReport>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagementStatus {
    pub enabled: bool,
    pub priority: ResearchPriority,
    pub funding: ResearchFunding,
    pub research_spent: [u64; 6],
    pub automatic_stock: bool,
    pub staff_budget_cents: u64,
    pub promotion: Promotion,
    pub promotion_remaining: u16,
    pub promotions_bought: u64,
    pub promotion_candidates: u64,
    pub promotion_admissions: u64,
    pub promotion_revenue_cents: u64,
}

fn ride_research(model: RideModel) -> Option<(ResearchPriority, u64)> {
    let priority = match model.category() {
        RideCategory::Gentle => ResearchPriority::Gentle,
        RideCategory::Thrill => ResearchPriority::Thrill,
        RideCategory::Water => ResearchPriority::Water,
        RideCategory::Transport => ResearchPriority::Transport,
        RideCategory::Coaster => ResearchPriority::Coasters,
    };
    let rank = match model {
        RideModel::Carousel
        | RideModel::FerrisWheel
        | RideModel::TeaCups
        | RideModel::ParkRailway
        | RideModel::ClassicCoaster => return None,
        RideModel::DropTower | RideModel::RiverRapids | RideModel::InvertedCoaster => 2,
        RideModel::PirateShip => 3,
        _ => 1,
    };
    Some((priority, rank))
}
fn service_rank(kind: ServiceKind) -> u64 {
    match kind {
        ServiceKind::IceCream => 1,
        ServiceKind::Souvenir => 2,
        ServiceKind::FirstAid => 3,
        _ => 0,
    }
}
impl Park {
    /// Existing parks opt in explicitly. Starting a new game calls this once.
    pub fn enable_management(&mut self) {
        if self.management.is_some() {
            return;
        }
        self.management = Some(Management {
            enabled_tick: self.ticks,
            priority: ResearchPriority::Thrill,
            funding: ResearchFunding::Paused,
            research_progress: 0,
            research_spent: [0; 6],
            automatic_stock: true,
            staff_budget_cents: 2_000,
            payroll_epoch: self.ticks / 900,
            payroll_spent_cents: 0,
            promotion: Promotion::Family,
            promotion_remaining: 0,
            promotions_bought: 0,
            promotion_candidates: 0,
            promotion_admissions: 0,
            promotion_revenue_cents: 0,
            stocks: Vec::new(),
            reports: Vec::new(),
        });
        self.sync_facility_stocks();
        let report = self.management_report();
        self.management.as_mut().unwrap().reports.push(report);
    }
    pub fn management_status(&self) -> ManagementStatus {
        let m = self.management.as_ref();
        ManagementStatus {
            enabled: m.is_some(),
            priority: m.map_or(ResearchPriority::default(), |m| m.priority),
            funding: m.map_or(ResearchFunding::default(), |m| m.funding),
            research_spent: m.map_or([0; 6], |m| m.research_spent),
            automatic_stock: m.is_some_and(|m| m.automatic_stock),
            staff_budget_cents: m.map_or(2_000, |m| m.staff_budget_cents),
            promotion: m.map_or(Promotion::default(), |m| m.promotion),
            promotion_remaining: m.map_or(0, |m| m.promotion_remaining),
            promotions_bought: m.map_or(0, |m| m.promotions_bought),
            promotion_candidates: m.map_or(0, |m| m.promotion_candidates),
            promotion_admissions: m.map_or(0, |m| m.promotion_admissions),
            promotion_revenue_cents: m.map_or(0, |m| m.promotion_revenue_cents),
        }
    }
    pub fn set_research(
        &mut self,
        priority: ResearchPriority,
        funding: ResearchFunding,
    ) -> Result<(), String> {
        if self.scenario_status().map(|s| s.kind) == Some(crate::ScenarioKind::MeadowChallenge)
            && funding == ResearchFunding::High
        {
            return Err("Meadow Challenge permits at most $2.50 research per 30 seconds".into());
        }
        self.enable_management();
        let m = self.management.as_mut().unwrap();
        m.priority = priority;
        m.funding = funding;
        Ok(())
    }
    pub fn ride_researched(&self, model: RideModel) -> bool {
        self.management.as_ref().is_none_or(|m| {
            ride_research(model).is_none_or(|(p, rank)| {
                m.research_spent[p.index()] >= rank * RESEARCH_PROJECT_CENTS
            })
        })
    }
    pub fn service_researched(&self, kind: ServiceKind) -> bool {
        self.management
            .as_ref()
            .is_none_or(|m| m.research_spent[5] >= service_rank(kind) * RESEARCH_PROJECT_CENTS)
    }
    pub fn research_next_name(&self) -> String {
        let Some(m) = &self.management else {
            return "Enable management to begin research".into();
        };
        if m.priority == ResearchPriority::Facilities {
            return ServiceKind::ALL
                .into_iter()
                .find(|k| !self.service_researched(*k))
                .map_or_else(|| "All facilities researched".into(), |k| k.to_string());
        }
        RideModel::ALL
            .into_iter()
            .find(|r| {
                ride_research(*r).is_some_and(|(p, _)| p == m.priority) && !self.ride_researched(*r)
            })
            .map_or_else(|| "Category complete".into(), |r| r.to_string())
    }
    pub fn facility_stocks(&self) -> &[FacilityStock] {
        self.management.as_ref().map_or(&[], |m| &m.stocks)
    }
    pub fn set_automatic_stock(&mut self, enabled: bool) {
        self.enable_management();
        self.management.as_mut().unwrap().automatic_stock = enabled;
    }
    pub fn set_staff_budget(&mut self, cents: u64) -> Result<(), String> {
        if cents > 2_000 || !cents.is_multiple_of(100) {
            return Err("Use a staff budget from $0 to $20 in $1 steps".into());
        }
        self.enable_management();
        self.management.as_mut().unwrap().staff_budget_cents = cents;
        Ok(())
    }
    pub(crate) fn sync_facility_stocks(&mut self) {
        let Some(m) = self.management.as_mut() else {
            return;
        };
        for s in &self.services {
            if m.stocks.iter().all(|row| row.service != s.id) {
                m.stocks.push(FacilityStock {
                    service: s.id,
                    baseline_served: s.served,
                    baseline_supplies_cents: s.supplies_cents,
                    purchased: 0,
                    remaining: 0,
                });
            }
        }
        m.stocks.sort_by_key(|s| s.service);
    }
    pub fn order_facility_stock(&mut self, id: ServiceId, units: u16) -> Result<(), String> {
        let service = self.service(id).ok_or("Choose a facility")?;
        if service.object.is_none() {
            return Err("This facility has been demolished".into());
        }
        let cost = u64::from(service.kind.supply_cost_cents());
        if units == 0 || units > STOCK_CAPACITY || cost * u64::from(units) > self.cash_cents {
            return Err("Order needs sufficient cash and 1–100 units".into());
        }
        self.enable_management();
        self.sync_facility_stocks();
        let m = self.management.as_mut().unwrap();
        let stock = m.stocks.iter_mut().find(|s| s.service == id).unwrap();
        if stock.remaining + units > STOCK_CAPACITY
            || stock.purchased > crate::services::MAX_SERVICE_USES - u64::from(units)
        {
            return Err("Facility stock capacity reached".into());
        }
        stock.remaining += units;
        stock.purchased += u64::from(units);
        self.services
            .iter_mut()
            .find(|s| s.id == id)
            .unwrap()
            .supplies_cents += cost * u64::from(units);
        // Each unit keeps the established fixed-price receipt and archival rules.
        for _ in 0..units {
            self.record_cash(FinanceKind::ServiceSupplies, Some(id), cost);
        }
        Ok(())
    }
    pub(crate) fn stock_available(&self, id: ServiceId) -> bool {
        self.management
            .as_ref()
            .is_none_or(|m| m.stocks.iter().any(|s| s.service == id && s.remaining > 0))
    }
    pub(crate) fn consume_stock(&mut self, id: ServiceId) -> bool {
        let Some(m) = self.management.as_mut() else {
            return false;
        };
        let stock = m
            .stocks
            .iter_mut()
            .find(|s| s.service == id)
            .expect("active facility stock");
        stock.remaining -= 1;
        true
    }
    pub(crate) fn stock_cost_valid(&self, id: ServiceId, initial_served: u64) -> bool {
        let s = self.service(id).unwrap();
        match self
            .management
            .as_ref()
            .and_then(|m| m.stocks.iter().find(|r| r.service == id))
        {
            Some(stock) => {
                stock
                    .purchased
                    .checked_mul(u64::from(s.kind.supply_cost_cents()))
                    .and_then(|cost| stock.baseline_supplies_cents.checked_add(cost))
                    == Some(s.supplies_cents)
            }
            None => {
                u128::from(s.supplies_cents)
                    == u128::from(s.served - initial_served)
                        * u128::from(s.kind.supply_cost_cents())
            }
        }
    }
    pub fn start_promotion(&mut self, promotion: Promotion) -> Result<(), String> {
        if self.scenario_status().map(|s| s.kind) == Some(crate::ScenarioKind::MeadowChallenge) {
            return Err("Meadow Challenge allows local flyers only".into());
        }
        if !self.visitors.park_open
            || !self.visitors.enabled
            || !self.visitor_flow_stats().entry_ready
        {
            return Err("Open the park with a connected entry and arrivals first".into());
        }
        let category = match promotion {
            Promotion::Family => RideCategory::Gentle,
            Promotion::Enthusiast => RideCategory::Coaster,
        };
        if !self.all_rides().any(|r| {
            self.ride_available(r.id)
                && self.ride_is_open(r.id)
                && self
                    .ride_model(r.id)
                    .is_some_and(|m| m.category() == category)
        }) {
            return Err("Open a matching gentle ride or coaster before promoting it".into());
        }
        if self
            .management
            .as_ref()
            .is_some_and(|m| m.promotion_remaining > 0)
        {
            return Err("A promotion is already active".into());
        }
        if self.cash_cents < PROMOTION_COST_CENTS {
            return Err("Promotion needs $50".into());
        }
        self.enable_management();
        let m = self.management.as_mut().unwrap();
        if m.promotions_bought >= 1_000_000 {
            return Err("Promotion capacity reached".into());
        }
        m.promotion = promotion;
        m.promotion_remaining = PROMOTION_TICKS;
        m.promotions_bought += 1;
        self.record_cash(FinanceKind::Promotion, None, PROMOTION_COST_CENTS);
        Ok(())
    }
    pub(crate) fn promotion_active(&self) -> bool {
        self.management
            .as_ref()
            .is_some_and(|m| m.promotion_remaining > 0)
    }
    pub(crate) fn attribute_promotion(&mut self, admitted: bool, price: u16) {
        if let Some(m) = self
            .management
            .as_mut()
            .filter(|m| m.promotion_remaining > 0)
        {
            m.promotion_candidates += 1;
            m.promotion_admissions += u64::from(admitted);
            if admitted {
                m.promotion_revenue_cents += u64::from(price);
            }
        }
    }
    pub(crate) fn management_payroll(&mut self, cost: u64) -> bool {
        let Some(m) = self.management.as_mut() else {
            return true;
        };
        let epoch = self.ticks / 900;
        if epoch != m.payroll_epoch {
            m.payroll_epoch = epoch;
            m.payroll_spent_cents = 0;
        }
        if m.payroll_spent_cents + cost > m.staff_budget_cents {
            return false;
        }
        m.payroll_spent_cents += cost;
        true
    }
    pub(crate) fn step_management(&mut self) {
        let Some(m) = self.management.as_mut() else {
            return;
        };
        m.promotion_remaining = m.promotion_remaining.saturating_sub(1);
        m.research_progress += 1;
        let charge = if m.research_progress == 900 {
            m.research_progress = 0;
            let left = m.priority.projects() * RESEARCH_PROJECT_CENTS
                - m.research_spent[m.priority.index()];
            m.funding.cost().min(left).min(self.cash_cents / 50 * 50)
        } else {
            0
        };
        if charge > 0 {
            let m = self.management.as_mut().unwrap();
            m.research_spent[m.priority.index()] += charge;
            self.record_cash(FinanceKind::Research, None, charge);
        }
        self.sync_facility_stocks();
        if self.management.as_ref().unwrap().automatic_stock {
            let ids: Vec<_> = self
                .services
                .iter()
                .filter(|s| s.object.is_some() && s.open)
                .filter(|s| {
                    self.facility_stocks()
                        .iter()
                        .any(|r| r.service == s.id && r.remaining < 5)
                })
                .map(|s| s.id)
                .collect();
            for id in ids {
                let _ = self.order_facility_stock(id, 20);
            }
        }
    }
    pub(crate) fn finish_management_tick(&mut self) {
        if self.management.is_some() && self.ticks.is_multiple_of(MANAGEMENT_PERIOD_TICKS) {
            let report = self.management_report();
            let m = self.management.as_mut().unwrap();
            if m.reports.len() == 60 {
                m.reports.remove(0);
            }
            m.reports.push(report);
        }
    }
    pub fn management_reports(&self) -> &[ManagementReport] {
        self.management.as_ref().map_or(&[], |m| &m.reports)
    }
    pub fn management_report(&self) -> ManagementReport {
        let f = self.finance_summary();
        let rating = if self.guests.is_empty() {
            50
        } else {
            (self
                .guests
                .iter()
                .map(|g| u64::from(g.satisfaction))
                .sum::<u64>()
                / self.guests.len() as u64) as u8
        };
        let total = self.all_rides().count();
        let reliable = self
            .all_rides()
            .filter(|r| {
                self.ride_maintenance_status(r.id)
                    .is_some_and(|s| !s.shutdown)
            })
            .count();
        let reliability = if total == 0 {
            100
        } else {
            (reliable * 100 / total) as u8
        };
        let candidates = self
            .visitors
            .spawned
            .saturating_add(self.visitors.turned_away);
        let popularity = if candidates == 0 {
            50
        } else {
            (self.visitors.spawned * 100 / candidates) as u8
        };
        ManagementReport {
            tick: self.ticks,
            cash_cents: f.cash_cents,
            revenue_cents: f.rides.revenue_cents
                + f.services.revenue_cents
                + f.admission_revenue_cents,
            expenses_cents: f.rides.upkeep_cents
                + f.services.upkeep_cents
                + f.services.supplies_cents
                + f.staff_wages_cents
                + f.advertising_cents
                + f.research_cents
                + f.promotion_cents,
            capital_net_cents: i128::from(f.refunds_cents) - i128::from(f.construction_cents),
            financing_net_cents: i128::from(f.loan_borrowed_cents)
                - i128::from(f.loan_repaid_cents)
                - i128::from(f.loan_interest_cents),
            debt_cents: self.loan_stats().principal_cents + self.loan_stats().interest_due_cents,
            park_value_cents: self.park_value().park_cents,
            rating,
            popularity,
            reliability,
        }
    }
    pub fn management_notices(&self) -> Vec<String> {
        let mut notices = Vec::new();
        if self.cash_cents < 5_000 {
            notices.push("Cash below $50. Review operating costs and the Loans account.".into());
        }
        let reports = self.management_reports();
        if reports.len() >= 2
            && reports[reports.len() - 1].operating_profit_cents()
                < reports[reports.len() - 2].operating_profit_cents()
        {
            notices.push(
                "The last minute made an operating loss. Review ride and facility profit.".into(),
            );
        }
        for stock in self.facility_stocks().iter().filter(|s| s.remaining == 0) {
            if let Some(s) = self
                .service(stock.service)
                .filter(|s| s.object.is_some() && s.open)
            {
                notices.push(format!(
                    "{} #{} is out of stock. Order supplies or enable automatic ordering.",
                    s.kind, s.id.0
                ));
            }
        }
        if self.park_staff().iter().any(|s| !s.working) {
            notices.push("Staff are awaiting wages. Check cash and the staff budget.".into());
        }
        if self.visitors.enabled && !self.visitor_flow_stats().entry_ready {
            notices.push(
                "Arrivals need a connected public entry. Open Visitor flow to select one.".into(),
            );
        }
        if notices.is_empty() {
            notices.push("Accounts and operations have no current management alerts.".into());
        }
        notices
    }
}

pub(crate) fn validate(park: &Park) -> Result<(), String> {
    let Some(m) = &park.management else {
        if park.finances.archived.research_cents != 0
            || park.finances.archived.promotion_cents != 0
            || park
                .finances
                .entries
                .iter()
                .any(|e| matches!(e.kind, FinanceKind::Research | FinanceKind::Promotion))
        {
            return Err("management spending requires a management account".into());
        }
        return Ok(());
    };
    if m.enabled_tick > park.ticks
        || m.research_progress >= 900
        || m.research_spent
            .iter()
            .zip(ResearchPriority::ALL)
            .any(|(v, p)| *v > p.projects() * RESEARCH_PROJECT_CENTS || !v.is_multiple_of(50))
        || m.staff_budget_cents > 2_000
        || !m.staff_budget_cents.is_multiple_of(100)
        || m.payroll_epoch > park.ticks / 900
        || m.payroll_spent_cents > 2_000
        || !m.payroll_spent_cents.is_multiple_of(100)
        || m.promotion_remaining > PROMOTION_TICKS
        || m.promotions_bought > 1_000_000
        || m.promotion_admissions > m.promotion_candidates
        || m.promotion_candidates > park.visitors.spawned + park.visitors.turned_away
        || m.promotion_revenue_cents > m.promotion_admissions * 2_000
        || (m.promotions_bought == 0 && (m.promotion_remaining != 0 || m.promotion_candidates != 0))
    {
        return Err("management research, payroll or promotion history is invalid".into());
    }
    if park
        .scenario_status()
        .is_some_and(|s| s.kind == crate::ScenarioKind::MeadowChallenge)
        && (m.funding == ResearchFunding::High || m.promotion_remaining != 0)
    {
        return Err("management policy violates Meadow Challenge rules".into());
    }
    let spent = |kind| {
        park.finances
            .entries
            .iter()
            .filter(|e| e.kind == kind)
            .map(|e| e.amount_cents)
            .sum::<u64>()
    };
    if park.finances.archived.research_cents + spent(FinanceKind::Research)
        != m.research_spent.iter().sum::<u64>()
        || park.finances.archived.promotion_cents + spent(FinanceKind::Promotion)
            != m.promotions_bought * PROMOTION_COST_CENTS
    {
        return Err("management spending disagrees with the cash ledger".into());
    }
    let mut last = None;
    for stock in &m.stocks {
        let s = park
            .service(stock.service)
            .ok_or("stock owner is missing")?;
        if last.is_some_and(|id| id >= stock.service)
            || stock.remaining > STOCK_CAPACITY
            || stock.purchased > crate::services::MAX_SERVICE_USES
            || stock.baseline_served > s.served
            || stock.baseline_supplies_cents > s.supplies_cents
            || stock
                .purchased
                .checked_sub(s.served - stock.baseline_served)
                != Some(u64::from(stock.remaining))
            || stock
                .baseline_supplies_cents
                .checked_add(stock.purchased * u64::from(s.kind.supply_cost_cents()))
                != Some(s.supplies_cents)
        {
            return Err("facility stock and supply receipts disagree".into());
        }
        last = Some(stock.service);
    }
    if m.stocks.len() != park.services.len() {
        return Err("facility stock account is missing".into());
    }
    let mut previous: Option<&ManagementReport> = None;
    let current = park.management_report();
    for r in &m.reports {
        if r.tick < m.enabled_tick
            || r.tick > park.ticks
            || previous.is_some_and(|p| {
                p.tick >= r.tick
                    || p.revenue_cents > r.revenue_cents
                    || p.expenses_cents > r.expenses_cents
            })
            || r.rating > 100
            || r.popularity > 100
            || r.reliability > 100
            || r.revenue_cents > current.revenue_cents
            || r.expenses_cents > current.expenses_cents
            || i128::from(crate::INITIAL_PARK_CASH_CENTS)
                + r.operating_profit_cents()
                + r.capital_net_cents
                + r.financing_net_cents
                != i128::from(r.cash_cents)
        {
            return Err("historic management account does not reconcile".into());
        }
        previous = Some(r);
    }
    if m.reports.is_empty() {
        return Err("management opening report is missing".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
