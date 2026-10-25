//! Recorded ride spending, retained demolition history and real paid sales.
use super::*;
use crate::{
    save_check::{cashflow, opening, ride_choice},
    ui::finance::FinanceView,
};

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    if matches!(scene, Scene::CashflowEmpty) {
        *park = Park::new(32, 32);
    } else if matches!(scene, Scene::CashflowPage) {
        *park = crate::demonstration_park();
        park.set_train_open(false);
        for _ in 0..3 {
            let id = park.place_carousel(ride_choice::ORIGIN, 8).unwrap();
            park.remove_carousel(id).unwrap();
        }
        park.place_carousel(ride_choice::ORIGIN, 8).unwrap();
    } else {
        let (built, coaster, carousel) = cashflow::install().unwrap();
        *park = built;
        if matches!(scene, Scene::CashflowPaid) {
            for id in [coaster, carousel] {
                opening::toggle(park, id).unwrap();
            }
            ride_choice::seek(
                park,
                |p| ride_choice::paid_running(p, coaster) && ride_choice::paid_running(p, carousel),
                "both reported rides have actual paid passengers",
            )
            .unwrap();
        }
    }
    view.help = false;
    view.rides_open = false;
    view.guests_open = false;
    view.services_open = false;
    view.designs_open = false;
    view.finance_open = true;
    view.finance_view = FinanceView::Rides;
    view.finance_view_pending = false;
    view.finance_page = usize::from(matches!(scene, Scene::CashflowPage));
    view.finance_page_pending = 0;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    ride_choice::bytes(park).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cashflow_captures_use_original_paid_construction_and_retained_demolition_history() {
        for (scene, count) in [
            (Scene::CashflowEmpty, 0),
            (Scene::CashflowBuilt, 3),
            (Scene::CashflowPaid, 3),
            (Scene::CashflowPage, 5),
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState::default();
            prepare(&mut park, &mut view, scene);
            assert_eq!(park.ride_cashflows().len(), count);
            assert_eq!(view.finance_view, FinanceView::Rides);
            if matches!(scene, Scene::CashflowPaid) {
                assert_eq!(
                    park.ride_cashflows()
                        .iter()
                        .filter(|r| r.ticket_revenue_cents > 0)
                        .count(),
                    2
                );
            }
        }
    }
}
