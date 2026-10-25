//! Real paid assets, debt, demolition losses and faulted remnants.
use super::*;
use crate::{
    save_check::{summary, value},
    ui::finance::FinanceView,
};

pub(super) fn prepare(p: &mut Park, view: &mut ViewState, scene: Scene) {
    *p = match scene {
        Scene::ValueEmpty => Park::new(32, 32),
        Scene::ValueLegacy => {
            let mut p = crate::demonstration_park();
            p.set_train_open(false);
            p
        }
        Scene::ValueBuilt => summary::install().unwrap(),
        Scene::ValueDebt => value::install().unwrap(),
        Scene::ValueEarned => {
            let mut p = value::install().unwrap();
            summary::mature(&mut p, true).unwrap();
            p
        }
        Scene::ValueNegative => value::negative().unwrap(),
        Scene::ValueFaulted => value::faulted().unwrap(),
        _ => unreachable!(),
    };
    view.help = false;
    view.rides_open = false;
    view.guests_open = false;
    view.services_open = false;
    view.designs_open = false;
    view.finance_open = true;
    view.finance_view = FinanceView::Value;
    view.finance_view_pending = false;
    view.finance_page = 0;
    view.finance_page_pending = 0;
    view.finance_loan_pending = None;
    view.path_tool = ToolState::default();
    view.speed = Speed::Paused;
    view.scene_dirty = true;
    value::inspect(p).unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valuation_captures_use_paid_basis_and_real_debt_with_strict_saved_states() {
        for scene in [
            Scene::ValueEmpty,
            Scene::ValueLegacy,
            Scene::ValueBuilt,
            Scene::ValueDebt,
            Scene::ValueEarned,
            Scene::ValueNegative,
            Scene::ValueFaulted,
        ] {
            let mut p = crate::demonstration_park();
            let mut view = ViewState::default();
            prepare(&mut p, &mut view, scene);
            assert_eq!(view.finance_view, FinanceView::Value);
            if matches!(scene, Scene::ValueNegative) {
                assert_eq!(p.park_value().company_cents, -5000);
            }
            if matches!(scene, Scene::ValueLegacy) {
                assert_eq!(p.park_value().park_cents, 0);
            }
        }
    }
}
