//! Facility building and management, without growing the permanent chrome.
use park_sim::{Park, Service, ServiceId, ServiceKind};

use super::*;

pub(crate) const PANEL_WIDTH: f32 = 560.0;
pub(crate) const PANEL_HEIGHT: f32 = 420.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ServiceCommand {
    PriceSteps { service: ServiceId, steps: i32 },
    SetOpen { service: ServiceId, open: bool },
}

#[derive(Component)]
pub(crate) struct ServicesPanel;
#[derive(Component)]
pub(crate) enum ServiceLabel {
    Selected,
    Details,
    Price,
    Notice,
    Open,
}

fn text(parent: &mut ChildSpawnerCommands, initial: &str, kind: ServiceLabel, size: f32) {
    parent.spawn((
        Text::new(initial),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(INK),
        kind,
    ));
}

pub(super) fn setup(commands: &mut Commands) {
    commands
        .spawn((
            ServicesPanel,
            GlobalZIndex(9),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.09, 0.65)),
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    width: px(PANEL_WIDTH),
                    height: px(PANEL_HEIGHT),
                    padding: UiRect::all(px(20)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(5),
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..row()
                })
                .with_children(|p| {
                    label(p, "Services", 23.0, INK);
                    button(p, "Close [M/Esc]", Action::CloseServices, 130.0);
                });
                p.spawn(row()).with_children(|p| {
                    for kind in [ServiceKind::Food, ServiceKind::Drink, ServiceKind::Toilet] {
                        button(
                            p,
                            &format!("Build {kind}"),
                            Action::BuildService(kind),
                            166.0,
                        );
                    }
                });
                label(
                    p,
                    "Place a facility; face its counter toward a public path.",
                    11.0,
                    MUTED,
                );
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..row()
                })
                .with_children(|p| {
                    button(p, "<", Action::BrowseService(-1), 45.0);
                    text(p, "No facilities", ServiceLabel::Selected, 13.0);
                    button(p, ">", Action::BrowseService(1), 45.0);
                });
                p.spawn(Node {
                    height: px(110),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|p| {
                    text(
                        p,
                        "Build food, drink or a toilet to meet guest needs.",
                        ServiceLabel::Details,
                        12.0,
                    );
                });
                p.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..row()
                })
                .with_children(|p| {
                    button(p, "- F6", Action::ServicePrice(-1), 65.0);
                    text(p, "Price unavailable", ServiceLabel::Price, 13.0);
                    button(p, "+ F7", Action::ServicePrice(1), 65.0);
                    p.spawn((
                        Button,
                        Action::ToggleService,
                        Node {
                            width: px(100),
                            height: px(30),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(BUTTON),
                        BorderColor::all(BORDER),
                        BorderRadius::all(px(5)),
                    ))
                    .with_children(|p| text(p, "Open [O]", ServiceLabel::Open, 13.0));
                });
                p.spawn(Node {
                    height: px(44),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|p| text(p, "", ServiceLabel::Notice, 11.0));
                label(
                    p,
                    "Up/Down: facility  F6/F7: price  O: open/close  Space: pause",
                    11.0,
                    MUTED,
                );
            });
        });
}

fn selected<'a>(park: &'a Park, view: &mut ViewState) -> Option<&'a Service> {
    let mut active: Vec<_> = park
        .services()
        .iter()
        .filter(|service| service.object.is_some())
        .collect();
    active.sort_unstable_by_key(|service| service.id);
    let delta = std::mem::take(&mut view.service_browse_pending);
    if active.is_empty() {
        view.selected_service = None;
        return None;
    }
    let preferred = view.selected_service.or_else(|| match view.selected {
        Some(SurfaceId::Object(object)) => park.service_by_object(object).map(|service| service.id),
        _ => None,
    });
    let index = preferred
        .and_then(|id| active.iter().position(|service| service.id == id))
        .unwrap_or(0);
    let index = (index as i64 + i64::from(delta)).rem_euclid(active.len() as i64) as usize;
    view.selected_service = Some(active[index].id);
    Some(active[index])
}

pub(crate) fn refresh(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    mut panels: Query<&mut Node, With<ServicesPanel>>,
    mut labels: Query<(&ServiceLabel, &mut Text)>,
) {
    for mut panel in &mut panels {
        panel.display = if view.services_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !view.services_open {
        view.service_browse_pending = 0;
        return;
    }
    let facility = selected(&sim.0, &mut view);
    let stats = facility.map(|service| sim.0.service_stats(service.id));
    for (kind, mut text) in &mut labels {
        let value=match kind {
            ServiceLabel::Selected=>facility.map_or_else(||"No facilities".into(),|service|format!("{} #{}",service.kind,service.id.0)),
            ServiceLabel::Details=>facility.zip(stats.as_ref()).map_or_else(
                ||"Build food, drink or a toilet to meet guest needs.\nEach serves one guest at a time on its public path.\nOpen facilities pay upkeep even when disconnected.\nClose a facility to stop future running costs.\nN opens park finances.".into(),
                |(service,stats)| {
                    let status=if stats.closed_for_funds {"Closed: park cannot cover costs".into()}else{stats.status.to_string()};
                    let per_minute=u64::from(service.kind.upkeep_cost_cents()) * (60*TICKS_PER_SECOND/u64::from(park_sim::SERVICE_UPKEEP_TICKS));
                    format!("{status} / Faces {}\nSeeking {} / Waiting {} / Using {}\nServed {} / Revenue +{}\nUpkeep -{} / Supplies -{} / Profit {}\nOpen upkeep {}/min / Supplies {}/completed use\nPrice refusals {} / N: park finances",
                        direction_name(service.facing),stats.approaching,stats.waiting,stats.using,
                        compact_count(stats.served),money(stats.revenue_cents),money(stats.upkeep_cents),money(stats.supplies_cents),finance::signed_money(stats.profit_cents),
                        money(per_minute),money(u64::from(service.kind.supply_cost_cents())),compact_count(stats.unaffordable_rejections))
                }),
            ServiceLabel::Price=>facility.map_or_else(||"Price unavailable".into(),|service|format!("Price {}",money(u64::from(service.price_cents)))),
            ServiceLabel::Open=>if facility.is_some_and(|service|service.open){"Close [O]"}else{"Open [O]"}.into(),
            ServiceLabel::Notice=>if facility.is_some_and(|service|service.closed_for_funds) {
                "Insufficient park cash stopped this facility. No debt was created.\nReopen to retry; it may close again if cash is still insufficient.".into()
            }else{view.service_notice.clone().unwrap_or_else(||
                "Open upkeep runs even without customers; closing stops it.\nSupplies are paid on completion. Price locks when use starts.\nClosing cancels unfinished use without charging the guest.".into())},
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::TilePos;

    #[test]
    fn facility_selector_prefers_world_identity_wraps_and_skips_retired_history() {
        let mut park = Park::new(32, 32);
        let first = park
            .place_service(
                TilePos { x: 4, y: 5 },
                8,
                PathDirection::North,
                ServiceKind::Food,
            )
            .unwrap();
        let second = park
            .place_service(
                TilePos { x: 5, y: 5 },
                8,
                PathDirection::North,
                ServiceKind::Drink,
            )
            .unwrap();
        let third = park
            .place_service(
                TilePos { x: 6, y: 5 },
                8,
                PathDirection::North,
                ServiceKind::Toilet,
            )
            .unwrap();
        let object = park.service(second).unwrap().object.unwrap();
        let mut view = ViewState {
            selected: Some(SurfaceId::Object(object)),
            ..default()
        };
        assert_eq!(selected(&park, &mut view).unwrap().id, second);
        view.service_browse_pending = -1;
        assert_eq!(selected(&park, &mut view).unwrap().id, first);
        view.service_browse_pending = -1;
        assert_eq!(selected(&park, &mut view).unwrap().id, third);
        park.remove_object(park.service(third).unwrap().object.unwrap())
            .unwrap();
        assert_eq!(selected(&park, &mut view).unwrap().id, first);
        assert_eq!(view.service_browse_pending, 0);
    }
}
