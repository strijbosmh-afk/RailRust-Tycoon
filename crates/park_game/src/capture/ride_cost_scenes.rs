//! Real billing, paid funds closure and refund-funded recovery.
use super::*;
use crate::{
    ride_operations::{self, OperationCommand},
    save_check::ride_costs,
    ui::rides::RideTab,
};
pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let (built, id, reserve) = ride_costs::install().unwrap();
    *park = built;
    if !matches!(scene, Scene::CostsReady) {
        ride_costs::paid(park, id).unwrap();
    }
    if matches!(
        scene,
        Scene::CostsClosed | Scene::CostsReopened | Scene::CostsFinance
    ) {
        ride_costs::closed(park, id).unwrap();
    }
    if matches!(scene, Scene::CostsReopened) {
        ride_costs::reopen(park, id, reserve).unwrap();
    }
    if matches!(scene, Scene::CostsClosed) {
        let before = park.clone();
        assert!(!ride_operations::perform_command(
            park,
            view,
            id,
            OperationCommand::ToggleOpen
        ));
        assert_eq!(*park, before);
    }
    view.help = false;
    view.guests_open = false;
    view.services_open = false;
    view.designs_open = false;
    view.rides_open = !matches!(scene, Scene::CostsFinance);
    view.finance_open = matches!(scene, Scene::CostsFinance);
    view.finance_view = crate::ui::finance::FinanceView::Rides;
    view.finance_view_pending = false;
    view.finance_page = 0;
    view.selected_ride = Some(id);
    view.ride_tab = RideTab::Operations;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cost_captures_use_real_bills_and_do_not_fake_funds_or_paid_passengers() {
        for scene in [
            Scene::CostsReady,
            Scene::CostsPaid,
            Scene::CostsClosed,
            Scene::CostsReopened,
            Scene::CostsFinance,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState::default();
            prepare(&mut park, &mut view, scene);
            let id = park.ride().unwrap().id;
            assert_eq!(
                park.ride_running_costs(id).unwrap().closed_for_funds,
                matches!(scene, Scene::CostsClosed | Scene::CostsFinance)
            );
            crate::save_check::ride_choice::bytes(&park).unwrap();
        }
    }
}
