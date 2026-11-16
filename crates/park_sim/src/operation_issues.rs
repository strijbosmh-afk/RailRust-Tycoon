//! Read-only, current operational problems with a concrete management target.
use crate::{GuestId, GuestState, Park, RideId, ServiceAccessStatus, ServiceId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationTarget {
    Ride(RideId),
    Facility(ServiceId),
    Guest(GuestId),
    Visitors,
    Staff,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationIssue {
    pub target: OperationTarget,
    pub summary: String,
    pub action: &'static str,
}

impl Park {
    /// Derived from the same access, upkeep and stock rules used during play.
    /// Deliberately closing a healthy ride/facility is a valid player choice.
    pub fn operation_issues(&self) -> Vec<OperationIssue> {
        let mut issues = Vec::new();
        let flow = self.visitor_flow_stats();
        if flow.enabled && !flow.entry_ready {
            issues.push(OperationIssue {
                target: OperationTarget::Visitors,
                summary: "Visitor entry is disconnected".into(),
                action: "Select a connected public path in Visitor flow",
            });
        }
        let mut rides: Vec<_> = self
            .all_rides()
            .filter(|r| self.ride_available(r.id))
            .collect();
        rides.sort_by_key(|r| r.id);
        for ride in rides {
            let reason = if self.ride_is_open(ride.id) {
                self.ride_opening_status(ride.id)
                    .err()
                    .map(|e| e.to_string())
            } else if self
                .ride_running_costs(ride.id)
                .is_some_and(|c| c.closed_for_funds)
            {
                Some("Closed for unpaid running costs".into())
            } else if self
                .ride_maintenance_status(ride.id)
                .is_some_and(|s| s.shutdown)
            {
                Some("Closed for maintenance".into())
            } else {
                None
            };
            if let Some(reason) = reason {
                issues.push(OperationIssue {
                    target: OperationTarget::Ride(ride.id),
                    summary: format!(
                        "{} #{} — {reason}",
                        self.ride_name(ride.id).unwrap_or("Ride"),
                        ride.id.0
                    ),
                    action: "Inspect ride access, running costs and maintenance",
                });
            }
        }
        for service in self.services().iter().filter(|s| s.object.is_some()) {
            let reason = if service.closed_for_funds {
                Some("Closed for unpaid running costs")
            } else if service.open
                && self.service_access(service.id).status == ServiceAccessStatus::Disconnected
            {
                Some("Public path is disconnected")
            } else if service.open
                && self
                    .facility_stocks()
                    .iter()
                    .any(|s| s.service == service.id && s.remaining == 0)
            {
                Some("Out of stock")
            } else {
                None
            };
            if let Some(reason) = reason {
                issues.push(OperationIssue {
                    target: OperationTarget::Facility(service.id),
                    summary: format!("{} #{} — {reason}", service.kind, service.id.0),
                    action: "Inspect this facility and its supply settings",
                });
            }
        }
        if self.park_staff().iter().any(|staff| !staff.working) {
            issues.push(OperationIssue {
                target: OperationTarget::Staff,
                summary: "Staff are awaiting wages".into(),
                action: "Review available cash and the staff budget",
            });
        }
        if let Some(guest) = self
            .guests()
            .iter()
            .find(|guest| guest.state == GuestState::Lost)
        {
            issues.push(OperationIssue {
                target: OperationTarget::Guest(guest.id),
                summary: format!("{} guests have lost their path", self.crowd_stats().lost),
                action: "Inspect a stranded guest and restore connected paths",
            });
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PathDirection, ServiceKind, TilePos};

    #[test]
    fn closed_services_are_intentional_and_issues_leave_state_unchanged() {
        let mut park = Park::new(32, 32);
        let service = park
            .place_service(
                TilePos { x: 4, y: 4 },
                0,
                PathDirection::North,
                ServiceKind::Food,
            )
            .unwrap();
        let before = park.clone();
        let issues = park.operation_issues();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].target, OperationTarget::Facility(service));
        assert_eq!(park, before);
        park.set_service_open(service, false).unwrap();
        assert!(park.operation_issues().is_empty());
    }

    #[test]
    fn disconnected_arrivals_and_lost_guests_have_management_targets() {
        let mut park = Park::new(32, 32);
        park.install_ground_starter_crowd(73).unwrap();
        let entry = park.guests()[0].current.unwrap();
        park.set_visitor_entry(entry).unwrap();
        park.set_visitor_arrivals(true).unwrap();
        park.remove_object(entry);
        let issues = park.operation_issues();
        assert!(issues.iter().any(|i| i.target == OperationTarget::Visitors));
        assert!(
            issues
                .iter()
                .any(|i| matches!(i.target, OperationTarget::Guest(_)))
        );
    }
}
