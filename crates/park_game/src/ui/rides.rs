//! Stable ride selection and the build entry point; no extra permanent panels.
use super::*;
use park_sim::{CAROUSEL_BUILD_COST_CENTS, CAROUSEL_REFUND_PERCENT, Park, RideId, RideKind};

pub(crate) const PANEL_WIDTH: f32 = 560.0;
pub(crate) const PANEL_HEIGHT: f32 = 465.0;

pub(crate) fn active_ids(park: &Park) -> Vec<RideId> {
    let mut ids: Vec<_> = park
        .all_rides()
        .filter(|ride| {
            park.carousel(ride.id)
                .is_none_or(|carousel| !carousel.objects.is_empty())
        })
        .map(|ride| ride.id)
        .collect();
    ids.sort_unstable();
    ids
}

pub(crate) fn selected_id(park: &Park, view: &ViewState) -> Option<RideId> {
    let ids = active_ids(park);
    match view.selected_ride {
        Some(id) => ids.contains(&id).then_some(id),
        None => ids.first().copied(),
    }
}

pub(crate) fn name(park: &Park, id: RideId) -> &'static str {
    match park.ride_kind(id) {
        Some(RideKind::Carousel) => "Meadow Carousel",
        _ => "Meadow Flyer",
    }
}

pub(crate) fn construction_details(park: &Park, id: RideId) -> String {
    park.carousel(id).map_or_else(
        || {
            let paid = park.track_economy().paid_pieces.iter().map(|piece| piece.paid_cents).sum();
            format!(
                "Live track paid {} / 50% back on removal\nZ: quote replacement / select rail + Delete: remove",
                money(paid)
            )
        },
        |carousel| match park.carousel_demolition_quote(id) {
            Ok(refund) => format!(
                "Paid {} / Demolition refund {}",
                money(carousel.construction_cost_cents),
                money(refund)
            ),
            Err(_) => format!(
                "Paid {} / Demolition blocked while occupied",
                money(carousel.construction_cost_cents)
            ),
        },
    )
}

#[derive(Component)]
pub(crate) struct RidesPanel;
#[derive(Component)]
pub(crate) enum RideLabel {
    Selected,
    Details,
}

pub(super) fn setup(commands: &mut Commands) {
    commands.spawn((RidesPanel, GlobalZIndex(9), Node {
        display: Display::None, position_type: PositionType::Absolute,
        width: percent(100), height: percent(100), align_items: AlignItems::Center,
        justify_content: JustifyContent::Center, ..default()
    }, BackgroundColor(Color::srgba(0.04,0.12,0.09,0.65))))
    .with_children(|p| { p.spawn((Node { width:px(PANEL_WIDTH),height:px(PANEL_HEIGHT),
        padding:UiRect::all(px(20)),flex_direction:FlexDirection::Column,row_gap:px(12),..default()
    },BackgroundColor(PAPER),BorderRadius::all(px(10))))
    .with_children(|p| {
        p.spawn(Node {justify_content:JustifyContent::SpaceBetween,..row()}).with_children(|p| {
            label(p,"Rides",23.0,INK);
            button(p,"Use ride [Y/Esc]",Action::CloseRides,150.0);
        });
        p.spawn(Node {justify_content:JustifyContent::SpaceBetween,..row()}).with_children(|p| {
            button(p,"< Up",Action::BrowseRide(-1),65.0);
            p.spawn((Text::new("No ride"),TextFont{font_size:17.0,..default()},TextColor(INK),RideLabel::Selected));
            button(p,"Down >",Action::BrowseRide(1),65.0);
        });
        p.spawn(Node{height:px(125),flex_shrink:0.0,..default()}).with_children(|p| {
            p.spawn((Text::new(""),TextFont{font_size:13.0,..default()},TextColor(INK),RideLabel::Details));
        });
        button(p,&format!("Build carousel / {}",money(CAROUSEL_BUILD_COST_CENTS)),Action::BuildCarousel,310.0);
        button(p,"Edit coaster track [Z]",Action::EditTrack,310.0);
        label(p,&format!("Six seats; 3 x 3 tiles. Queue and gates placed separately.\nDemolition returns {}% of the amount paid for this ride.\nOne carousel at a time. Choose height in Build Tools.",CAROUSEL_REFUND_PERCENT),12.0,MUTED);
        label(p,"Selected ride: O open/close / F find / F6-F7 ticket price\nJ queue / I entrance / K exit / Space pause",12.0,MUTED);
    }); });
}

pub(crate) fn refresh(
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    mut panels: Query<&mut Node, With<RidesPanel>>,
    mut labels: Query<(&RideLabel, &mut Text)>,
) {
    for mut node in &mut panels {
        node.display = if view.rides_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    let ids = active_ids(&sim.0);
    let delta = std::mem::take(&mut view.ride_browse_pending);
    if ids.is_empty() {
        view.selected_ride = None;
        return;
    }
    let selected = selected_id(&sim.0, &view).unwrap_or(ids[0]);
    let index = ids.iter().position(|id| *id == selected).unwrap_or(0);
    let index = if view.rides_open {
        (index as i64 + i64::from(delta)).rem_euclid(ids.len() as i64) as usize
    } else {
        index
    };
    if view.rides_open || view.selected_ride.is_some() {
        view.select_ride(ids[index]);
    }
    if !view.rides_open {
        return;
    }
    let id = ids[index];
    let journey = sim.0.ride_journey_stats(id);
    let access = sim.0.ride_access(id);
    let economy = sim.0.ride_economy_stats(id);
    let experience = sim.0.ride_experience_stats(id);
    for (kind, mut text) in &mut labels {
        let value = match kind {
            RideLabel::Selected => format!("{} / #{}", name(&sim.0, id), id.0),
            RideLabel::Details => format!(
                "{} / Ticket {}\nQueue: {}\nExit: {}\nSeats: {}/{} occupied / Completed {}\nRevenue {} / Reviews: {} ({} opinions)\n{}",
                if sim.0.ride_is_open(id) {
                    "Open"
                } else {
                    "Closed"
                },
                money(u64::from(economy.ticket_price_cents)),
                access.status,
                journey.exit_status,
                journey.boarding + journey.riding + journey.exiting,
                journey.capacity,
                compact_count(journey.completed),
                money(economy.revenue_cents),
                guests::rating_label(experience.average_rating_tenths),
                compact_count(experience.rating_count),
                construction_details(&sim.0, id)
            ),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
