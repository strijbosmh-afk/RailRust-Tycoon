//! Deterministic, state-based suggestions. Reading advice never changes the simulation.
use crate::{ChargingRule, OperationTarget, Park, ServiceAccessStatus, ServiceKind};

pub const MAX_TIP_USES: u8 = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfitTip {
    pub title: &'static str,
    pub text: String,
}

fn money(cents: i128) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let absolute = cents.unsigned_abs();
    format!("{sign}${}.{:02}", absolute / 100, absolute % 100)
}

impl Park {
    pub const fn tips_used(&self) -> u8 {
        self.tips_used
    }

    /// Merge durable usage from the park library without rewinding a saved counter.
    pub fn retain_tip_uses(&mut self, uses: u8) -> Result<(), &'static str> {
        if uses > MAX_TIP_USES {
            return Err("Tip usage exceeds three per park");
        }
        self.tips_used = self.tips_used.max(uses);
        Ok(())
    }

    pub fn profit_tips(&self) -> [ProfitTip; 2] {
        let mut candidates = Vec::new();
        let mut add =
            |priority, title, text| candidates.push((priority, ProfitTip { title, text }));
        let flow = self.visitor_flow_stats();
        let finance = self.finance_summary();
        let experience = self.guest_experience_summary();
        let needs = self.guest_needs_summary();
        let lost = self.crowd_stats().lost;

        if lost > 0 {
            add(
                100,
                "Reconnect lost guests",
                format!(
                    "{lost} guests are lost. Use the Guests report to locate them and reconnect their paths to the entrance, rides and facilities so they can spend again."
                ),
            );
        }
        if let Some(issue) = self
            .operation_issues()
            .into_iter()
            .find(|i| matches!(i.target, OperationTarget::Ride(_)))
        {
            add(
                95,
                "Restore ride income",
                format!(
                    "{}. {}. Restore access or resolve the operating problem before investing in another attraction.",
                    issue.summary, issue.action
                ),
            );
        }
        if !flow.park_open || (flow.enabled && !flow.entry_ready) {
            add(
                98,
                "Restore visitor arrivals",
                format!(
                    "The park is {} and its visitor entrance is {}. Open Visitor flow, connect the entrance and open the park to let new paying guests arrive.",
                    if flow.park_open { "open" } else { "closed" },
                    if flow.entry_ready {
                        "connected"
                    } else {
                        "disconnected"
                    }
                ),
            );
        } else if !flow.enabled {
            add(
                55,
                "Bring in new customers",
                format!(
                    "Visitor arrivals are disabled. Your current {} guests provide the available customer base. Enable arrivals after checking that the entrance is connected and basic services can handle more guests.",
                    needs.guest_count
                ),
            );
        }

        if let Some((kind, count)) = [
            (ServiceKind::Food, needs.hungry_guests),
            (ServiceKind::Drink, needs.thirsty_guests),
            (ServiceKind::Toilet, needs.toilet_guests),
            (ServiceKind::Rest, needs.tired_guests),
        ]
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .max_by_key(|(_, count)| *count)
        {
            let facilities: Vec<_> = self
                .services()
                .iter()
                .filter(|s| {
                    s.object.is_some()
                        && (s.kind == kind
                            || (s.kind == ServiceKind::IceCream
                                && matches!(kind, ServiceKind::Food | ServiceKind::Drink)))
                })
                .collect();
            let ready = facilities
                .iter()
                .any(|s| self.service_access(s.id).status == ServiceAccessStatus::Ready);
            if !ready {
                let action = if !facilities.is_empty() {
                    format!(
                        "Reconnect or reopen an existing {kind} facility before paying to build another."
                    )
                } else if self.facility_content_available(kind) && self.service_researched(kind) {
                    format!(
                        "A {kind} facility costs {}. {}",
                        money(i128::from(kind.build_cost_cents())),
                        if self.cash_cents() >= kind.build_cost_cents() {
                            "Place its counter next to a connected public path."
                        } else {
                            "Build cash reserves before purchasing it."
                        }
                    )
                } else {
                    format!(
                        "Check the catalogue and research availability for {kind}, then plan a connected facility."
                    )
                };
                add(
                    90,
                    "Meet guests' needs",
                    format!(
                        "{count} guests currently need {kind}. {action} Comfortable guests can stay and spend longer."
                    ),
                );
            } else if count * 3 >= needs.guest_count.max(1) {
                add(
                    65,
                    "Improve service access",
                    format!(
                        "{count} of {} guests need {kind}, although an open connected facility exists. Check its queues, supplies and prices before adding capacity.",
                        needs.guest_count
                    ),
                );
            }
        }

        if self.cash_cents() < 20_000 || finance.operating_profit_cents < 0 {
            add(
                80,
                "Protect your cash reserve",
                format!(
                    "Cash is {} and the recorded operating result is {}. Review the Finance report, pause optional construction and close persistently unused operating capacity until income covers costs.",
                    money(i128::from(self.cash_cents())),
                    money(finance.operating_profit_cents)
                ),
            );
        }
        if experience.unhappy_guests > 0 {
            add(
                70,
                "Keep guests in the park",
                format!(
                    "{} guests are unhappy; average satisfaction is {}%. Inspect their current thoughts, needs and ride value complaints. Fix the most common problem before spending on marketing.",
                    experience.unhappy_guests,
                    experience.average_satisfaction.unwrap_or(0)
                ),
            );
        }
        if let Some(ride) = self
            .all_rides()
            .filter(|r| {
                self.ride_available(r.id) && r.value_rejections > 0 && r.ticket_price_cents > 0
            })
            .max_by_key(|r| r.value_rejections)
        {
            add(
                68,
                "Review a ride's ticket value",
                format!(
                    "{} has recorded {} value refusals at a current ticket price of {}. Compare its ratings with guest budgets and test a lower price while watching revenue and queue length.",
                    self.ride_name(ride.id).unwrap_or("This ride"),
                    ride.value_rejections,
                    money(i128::from(ride.ticket_price_cents))
                ),
            );
        } else if flow.charging_rule != ChargingRule::FreeRides
            && let Some(ride) = self.all_rides().find(|r| {
                self.ride_available(r.id) && self.ride_is_open(r.id) && r.ticket_price_cents == 0
            })
        {
            add(
                60,
                "Review free ride pricing",
                format!(
                    "{} is open with free tickets. Your charging rules allow ride fees. Try a modest price and check guest affordability and actual ride revenue before raising it further.",
                    self.ride_name(ride.id).unwrap_or("This ride")
                ),
            );
        }
        let loan = self.loan_stats();
        if loan.principal_cents > 0
            && self.cash_cents() > loan.principal_cents.saturating_add(20_000)
        {
            add(
                62,
                "Reduce interest costs",
                format!(
                    "Loan principal is {} and cash is {}. Repaying part of the loan can reduce interest while retaining a reserve for running costs.",
                    money(i128::from(loan.principal_cents)),
                    money(i128::from(self.cash_cents()))
                ),
            );
        }
        add(
            10,
            "Compare income with running costs",
            format!(
                "Recorded ride revenue is {} and service revenue is {}. Compare each attraction's and facility's receipts with upkeep and supplies, then focus spending on the ones guests actually use.",
                money(i128::from(finance.rides.revenue_cents)),
                money(i128::from(finance.services.revenue_cents))
            ),
        );
        add(
            5,
            "Grow at an affordable pace",
            format!(
                "The park has {} guests and {} cash. Keep a running-cost reserve, connect a new attraction to existing paths, and check its demand before funding the next expansion.",
                needs.guest_count,
                money(i128::from(self.cash_cents()))
            ),
        );
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        [candidates.remove(0).1, candidates.remove(0).1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn advice_is_deterministic_read_only_and_responds_to_park_state() {
        let mut park = crate::demo_park::build_demo_park().unwrap();
        let before = park.clone();
        let first = park.profit_tips();
        assert_eq!(first, park.profit_tips());
        assert_ne!(first[0].title, first[1].title);
        assert_eq!(park, before);
        park.set_park_open(false);
        assert!(
            park.profit_tips()
                .iter()
                .any(|tip| tip.title == "Restore visitor arrivals")
        );
    }
    #[test]
    fn counter_survives_save_and_refuses_invalid_or_lower_values() {
        let mut park = Park::new(4, 4);
        park.retain_tip_uses(2).unwrap();
        park.retain_tip_uses(1).unwrap();
        assert_eq!(park.tips_used(), 2);
        assert!(park.retain_tip_uses(4).is_err());
        let restored = Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap();
        assert_eq!(restored.tips_used(), 2);
        let mut legacy: serde_json::Value =
            serde_json::from_slice(&park.to_save_bytes().unwrap()).unwrap();
        legacy["schema_version"] = 51.into();
        legacy["content_version"] = 51.into();
        assert!(Park::from_save_bytes(&serde_json::to_vec(&legacy).unwrap()).is_err());
        legacy["park"].as_object_mut().unwrap().remove("tips_used");
        assert_eq!(
            Park::from_save_bytes(&serde_json::to_vec(&legacy).unwrap())
                .unwrap()
                .tips_used(),
            0
        );
    }
}
