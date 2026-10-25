//! Summary states use real purchases, receipts, bills and retained histories.
use super::*;
use crate::{
    save_check::{ride_costs, summary},
    ui::finance::FinanceView,
};

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    if matches!(scene, Scene::SummaryEmpty) {
        *park = Park::new(32, 32);
    } else if matches!(scene, Scene::SummaryClosed) {
        let (built, id, _) = ride_costs::install().unwrap();
        *park = built;
        ride_costs::paid(park, id).unwrap();
        ride_costs::closed(park, id).unwrap();
    } else if matches!(scene, Scene::SummaryProfit) {
        *park = crate::demonstration_park();
        finance_scenes::prepare(park, view, Scene::FinanceProfit);
        assert!(park.finance_summary().services.profit_cents > 0);
    } else {
        *park = summary::install().unwrap();
        if matches!(scene, Scene::SummaryMixed | Scene::SummaryRemoved) {
            summary::mature(park, matches!(scene, Scene::SummaryRemoved)).unwrap();
        }
    }
    view.help = false;
    view.rides_open = false;
    view.guests_open = false;
    view.services_open = false;
    view.designs_open = false;
    view.finance_open = true;
    view.finance_view = FinanceView::ParkSummary;
    view.finance_view_pending = false;
    view.finance_page = 0;
    view.finance_page_pending = 0;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    summary::inspect(park).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn summary_capture_states_use_real_operations_and_preserve_strict_saves() {
        for scene in [
            Scene::SummaryEmpty,
            Scene::SummaryBuilt,
            Scene::SummaryMixed,
            Scene::SummaryRemoved,
            Scene::SummaryClosed,
            Scene::SummaryProfit,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState::default();
            prepare(&mut park, &mut view, scene);
            assert_eq!(view.finance_view, FinanceView::ParkSummary);
            if matches!(scene, Scene::SummaryBuilt) {
                assert_eq!(park.finance_summary().operating_profit_cents, 0);
                assert!(park.finance_summary().net_cash_cents < 0);
            }
            if matches!(scene, Scene::SummaryRemoved) {
                assert!(
                    park.services()
                        .iter()
                        .any(|s| s.object.is_none() && s.revenue_cents > 0)
                );
            }
        }
    }
}
