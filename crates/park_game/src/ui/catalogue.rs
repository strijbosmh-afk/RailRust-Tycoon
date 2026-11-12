//! Finite ride and facility catalogue, using the normal paid construction tools.
use super::*;
use park_sim::{RideCategory, RideModel, ServiceKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Command {
    Open,
    Close,
    Category(u8),
    Ride(RideModel),
    Facility(ServiceKind),
}
#[derive(Component)]
pub(crate) struct CataloguePanel;
#[derive(Component)]
pub(crate) struct CatalogueEntry(u8);

pub(crate) fn command(view: &mut ViewState, command: Command) {
    if view.capture_active || view.front_modal || view.diagnostic_case.is_some() {
        return;
    }
    if command != Command::Open && !view.catalogue_open {
        return;
    }
    match command {
        Command::Open => {
            view.set_modal(false, false, false, false);
            view.catalogue_open = true;
        }
        Command::Close => view.set_modal(false, false, false, false),
        Command::Category(category) => view.catalogue_category = category,
        Command::Ride(model) => {
            view.set_modal(false, false, false, false);
            view.path_tool.ride_model = model;
            view.path_tool.flat_ride_kind = model.base_kind();
            view.path_tool.ride_rotation = 0;
            view.path_tool.set_mode(PathTool::Carousel);
            view.track_editor.active = false;
            view.path_notice = None;
        }
        Command::Facility(kind) => {
            view.set_modal(false, false, false, false);
            view.path_tool.set_mode(PathTool::Service(kind));
            view.track_editor.active = false;
            view.path_notice = None;
        }
    }
    view.modal_transition_pending = true;
    view.ui_pointer_consumed = true;
}
fn category_index(category: RideCategory) -> u8 {
    match category {
        RideCategory::Gentle => 0,
        RideCategory::Thrill => 1,
        RideCategory::Water => 2,
        RideCategory::Transport => 3,
        RideCategory::Coaster => 4,
    }
}
pub(super) fn setup(commands: &mut Commands) {
    commands.spawn((CataloguePanel, GlobalZIndex(10), Node {
        display: Display::None, position_type: PositionType::Absolute, width: percent(100), height: percent(100),
        align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default()
    }, BackgroundColor(Color::srgba(0.04,0.12,0.09,0.68)))).with_children(|p| {
        p.spawn((layout::ScrollLayer::Catalogue, ScrollPosition::default(), layout::dialog(94.0, 760.0, Some(630.0)), BackgroundColor(PAPER), BorderRadius::all(px(10)))).with_children(|p| {
            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..row() }).with_children(|p| {
                label(p,"Build catalogue",24.0,INK);
                button(p,"Close [Esc]",Action::Catalogue(Command::Close),120.0);
            });
            label(p,"Choose a ride or facility, then place it in your park. C rotates the preview.",12.0,MUTED);
            p.spawn(Node { flex_wrap: FlexWrap::Wrap, ..row() }).with_children(|p| {
                for (index,title) in ["Gentle","Thrill","Water","Transport","Coasters","Facilities"].iter().enumerate() {
                    button(p,title,Action::Catalogue(Command::Category(index as u8)),106.0);
                }
            });
            for model in RideModel::ALL {
                p.spawn((CatalogueEntry(category_index(model.category())),Node { flex_direction: FlexDirection::Column, padding: UiRect::all(px(10)), row_gap: px(5), border: UiRect::all(px(1)), flex_shrink: 0.0, ..default() }, BorderColor::all(BORDER))).with_children(|p| {
                    let (width,depth)=model.dimensions();
                    let capacity=if model.tracked(){usize::from(model.seats_per_car()*model.max_cars())}else{model.flat_capacity()};
                    button(p,model.name(),Action::Catalogue(Command::Ride(model)),250.0);
                    label(p,model.description(),12.0,INK);
                    let cost=if model.tracked(){ model.layout(park_sim::TilePos{x:0,y:0},0,0).unwrap().into_iter().map(park_sim::track_piece_cost_cents).sum() }else{model.flat_cost()};
                    label(p,&format!("{}  /  {width} x {depth} tiles  /  {capacity} seats per train or cycle",money(cost)),12.0,MUTED);
                });
            }
            for kind in ServiceKind::ALL {
                p.spawn((CatalogueEntry(5),Node{ flex_direction: FlexDirection::Column, padding: UiRect::all(px(9)), row_gap: px(4), border: UiRect::all(px(1)), flex_shrink:0.0,..default()},BorderColor::all(BORDER))).with_children(|p|{
                    button(p,&format!("Build {kind}"),Action::Catalogue(Command::Facility(kind)),250.0);
                    let effect=match kind {
                        ServiceKind::Food=>"Meals relieve hunger.",ServiceKind::Drink=>"Drinks relieve thirst.",ServiceKind::Toilet=>"Toilets meet bathroom needs.",ServiceKind::Rest=>"Rest restores energy and eases nausea.",
                        ServiceKind::IceCream=>"Ice cream relieves hunger and thirst together.",ServiceKind::Souvenir=>"Gifts satisfy the desire to take a souvenir home.",ServiceKind::Information=>"Park maps satisfy the need for information.",ServiceKind::FirstAid=>"First aid treats nausea and restores energy."
                    };
                    label(p,&format!("{}  /  1 tile  /  {effect}",money(kind.build_cost_cents())),12.0,INK);
                    label(p,&format!("Default price {} / upkeep {} / supplies {} per visit",money(u64::from(kind.default_price_cents())),money(u64::from(kind.upkeep_cost_cents())),money(u64::from(kind.supply_cost_cents()))),11.0,MUTED);
                });
            }
        });
    });
}
pub(crate) fn refresh(
    view: Res<ViewState>,
    mut panels: Query<&mut Node, With<CataloguePanel>>,
    mut entries: Query<(&CatalogueEntry, &mut Node), Without<CataloguePanel>>,
) {
    for mut node in &mut panels {
        node.display = if view.catalogue_open && !view.front_modal {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (entry, mut node) in &mut entries {
        node.display = if entry.0 == view.catalogue_category {
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_catalogue_choice_reaches_paid_placement_and_blocks_click_through() {
        for model in RideModel::ALL {
            let mut view = ViewState::default();
            command(&mut view, Command::Open);
            assert!(view.catalogue_open);
            assert!(view.edits_blocked());
            command(&mut view, Command::Ride(model));
            assert!(!view.catalogue_open);
            assert_eq!(view.path_tool.ride_model, model);
            assert_eq!(view.path_tool.mode, PathTool::Carousel);
            assert!(view.modal_transition_pending);
            view.path_tool.cycle_shape();
            assert_eq!(view.path_tool.ride_rotation, 1);
        }
        for kind in ServiceKind::ALL {
            let mut view = ViewState::default();
            command(&mut view, Command::Open);
            command(&mut view, Command::Category(5));
            command(&mut view, Command::Facility(kind));
            assert_eq!(view.path_tool.mode, PathTool::Service(kind));
        }
    }
}
