//! Natural operating transactions and actual funds closure in production scenes.
use super::*;
use crate::save_check::{finance, services};
use park_sim::{ServiceId, ServiceKind};

fn prepare_state(park: &mut Park, view: &mut ViewState, scene: Scene) -> Option<ServiceId> {
    assert!(scene.is_finance());
    park.set_train_open(false);
    if matches!(scene, Scene::FinanceProfit) {
        let id = park
            .place_service(
                TilePos { x: 15, y: 21 },
                8,
                park_sim::PathDirection::North,
                ServiceKind::Drink,
            )
            .unwrap();
        view.selected_service = Some(id);
    } else {
        service_scenes::install(park, view);
    }
    if matches!(scene, Scene::FinanceLoss) {
        finance::seek(
            park,
            |p| p.finance_stats().net_profit_cents < 0,
            "real operating loss before sales",
        )
        .unwrap();
    } else if matches!(scene, Scene::FinanceProfit) {
        let ids: Vec<_> = park.services().iter().map(|s| s.id).collect();
        for id in ids {
            services::set_price(park, id, 1000).unwrap();
        }
        finance::seek(
            park,
            |p| {
                p.finance_stats().net_profit_cents > 0
                    && p.finance_stats().service_revenue_cents > 0
            },
            "actual positive net income after costs",
        )
        .unwrap();
    } else if matches!(scene, Scene::FinanceClosed | Scene::FinanceReopened) {
        finance::install_auxiliaries(park).unwrap();
        crate::save_check::facility_construction::spend_down(park).unwrap();
        finance::seek(
            park,
            |p| p.services().iter().any(|s| s.closed_for_funds),
            "automatic funds closure",
        )
        .unwrap();
        let closed = park
            .services()
            .iter()
            .find(|s| s.closed_for_funds)
            .unwrap()
            .id;
        if matches!(scene, Scene::FinanceClosed) {
            return Some(closed);
        }
        finance::close_all(park).unwrap();
        let revenue = park.ride().unwrap().revenue_cents;
        finance::install_income_ride(park).unwrap();
        finance::seek(
            park,
            |p| p.ride().unwrap().revenue_cents > revenue,
            "genuine new departure income",
        )
        .unwrap();
        let id = park
            .services()
            .iter()
            .find(|s| s.kind == ServiceKind::Drink && s.object.is_some())
            .unwrap()
            .id;
        park.set_service_open(id, true).unwrap();
        let served = park.service(id).unwrap().served;
        finance::seek(
            park,
            |p| p.service(id).unwrap().served > served,
            "reopened service completes a paid use",
        )
        .unwrap();
        return Some(id);
    } else if matches!(scene, Scene::FinanceRemoved) {
        finance::seek(
            park,
            |p| {
                p.services()
                    .iter()
                    .any(|s| s.revenue_cents > 0 && s.upkeep_cents > 0 && s.supplies_cents > 0)
            },
            "earned facility before demolition",
        )
        .unwrap();
        let old = park
            .services()
            .iter()
            .find(|s| s.revenue_cents > 0 && s.upkeep_cents > 0 && s.supplies_cents > 0)
            .unwrap()
            .clone();
        park.remove_object(old.object.unwrap()).unwrap();
        let after = park.service(old.id).unwrap();
        assert_eq!(
            (
                after.revenue_cents,
                after.upkeep_cents,
                after.supplies_cents
            ),
            (old.revenue_cents, old.upkeep_cents, old.supplies_cents)
        );
        assert!(after.object.is_none());
    } else {
        finance::seek(
            park,
            |p| {
                p.finance_stats().service_revenue_cents > 0
                    && p.finance_stats().supplies_cents > 0
                    && (!matches!(scene, Scene::FinanceHistory)
                        || p.finance_ledger().archived_entries > 0)
            },
            "income/cost ledger and requested rollover",
        )
        .unwrap();
    }
    None
}

fn present(park: &Park, view: &mut ViewState, scene: Scene, selected: Option<ServiceId>) {
    view.help = false;
    view.guests_open = false;
    // A genuine automatic closure is most legible in facility management.
    view.services_open = matches!(scene, Scene::FinanceClosed);
    view.finance_open = !view.services_open;
    view.finance_page = if matches!(scene, Scene::FinanceHistory) {
        usize::MAX
    } else {
        0
    };
    view.finance_page_pending = 0;
    view.selected_service = selected;
    view.selected = None;
    view.hovered = None;
    view.path_tool = ToolState::default();
    view.path_notice = None;
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    view.zoom = 2.0;
    let center = world::path_build_center(
        TilePos { x: 14, y: 20 },
        park,
        view.rotation,
        8,
        park_sim::PathSlope::Flat,
    )
    .unwrap();
    view.pan = center - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
}

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    let selected = prepare_state(park, view, scene);
    finance::bytes(park).unwrap();
    present(park, view, scene, selected);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finance_scenes_use_actual_transactions_closure_recovery_and_read_only_presentation() {
        for scene in [
            Scene::FinanceLedger,
            Scene::FinanceProfit,
            Scene::FinanceLoss,
            Scene::FinanceClosed,
            Scene::FinanceReopened,
            Scene::FinanceRemoved,
            Scene::FinanceHistory,
        ] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState {
                capture_active: true,
                ..default()
            };
            let selected = prepare_state(&mut park, &mut view, scene);
            let bytes = finance::bytes(&park).unwrap();
            for rotation in 0..4 {
                for mode in [ViewMode::Surface, ViewMode::Underground] {
                    view.rotation = rotation;
                    view.view_mode = mode;
                    present(&park, &mut view, scene, selected);
                    assert_eq!(bytes, park.to_save_bytes().unwrap());
                    assert_ne!(view.finance_open, view.services_open);
                    assert_eq!(view.speed, Speed::Paused);
                    assert!(view.pan.is_finite());
                }
            }
        }
    }
}
