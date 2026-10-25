//! Real opt-in arrivals, paid departures, retained receipts and missing entry.
use super::*;
use crate::{save_check::visitors, ui::finance::FinanceView};
pub(super) fn prepare(p: &mut Park, view: &mut ViewState, scene: Scene) {
    *p = if matches!(scene, Scene::VisitorsOff) {
        crate::demonstration_park()
    } else {
        visitors::install().unwrap()
    };
    view.selected_guest = None;
    match scene {
        Scene::VisitorsPaid => view.selected_guest = Some(visitors::paid_departure(p).unwrap()),
        Scene::VisitorsTurnover | Scene::VisitorsFinance => {
            visitors::paid_departure(p).unwrap();
            visitors::mature(p).unwrap();
        }
        Scene::VisitorsBlocked => {
            let id = visitors::paid_departure(p).unwrap();
            view.selected_guest = Some(id);
            let entry = p.visitor_flow_stats().entry.unwrap();
            p.remove_object(entry).unwrap();
            for _ in 0..3000 {
                p.advance(Speed::Normal);
            }
            assert_eq!(p.visitor_flow_stats().entry, None);
            assert!(p.guests().iter().any(|g| g.id == id));
        }
        _ => {}
    }
    view.help = false;
    view.rides_open = false;
    view.services_open = false;
    view.designs_open = false;
    view.finance_open = matches!(scene, Scene::VisitorsFinance);
    view.finance_view = FinanceView::ParkSummary;
    view.guests_open = !view.finance_open;
    view.guest_flow_view = true;
    view.guest_flow_pending = false;
    view.visitor_pending = None;
    view.visitor_notice = None;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    visitors::check(p).unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visitor_captures_preserve_real_paid_trips_and_strict_retirement_history() {
        for scene in [
            Scene::VisitorsOff,
            Scene::VisitorsReady,
            Scene::VisitorsPaid,
            Scene::VisitorsTurnover,
            Scene::VisitorsBlocked,
            Scene::VisitorsFinance,
        ] {
            let mut p = crate::demonstration_park();
            let mut view = ViewState::default();
            prepare(&mut p, &mut view, scene);
            assert_eq!(view.speed, Speed::Paused);
            if matches!(scene, Scene::VisitorsPaid) {
                assert!(p.visitor_flow_stats().finishing_visits > 0);
            }
            if matches!(scene, Scene::VisitorsTurnover | Scene::VisitorsFinance) {
                assert!(p.departed_visitors().spent_cents > 0);
            }
        }
    }
}
