//! Selected ride painting, with explicit, atomic commands and clear modal ownership.
use super::*;
use park_sim::{RideAppearance, RideKind, RidePaint};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Part {
    Structure,
    Supports,
    Vehicles,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Show,
    Step(Part, i8),
    Reset,
    Find,
    Conflict,
}
#[derive(Component)]
pub(crate) enum PaintLabel {
    Name(Part),
    Notice,
}
#[derive(Component)]
pub(crate) struct Swatch(Part);
pub(crate) fn queue(view: &mut ViewState, command: Command) {
    view.paint_commands.push(command);
}
fn get(a: RideAppearance, p: Part) -> RidePaint {
    match p {
        Part::Structure => a.structure,
        Part::Supports => a.supports,
        Part::Vehicles => a.vehicles,
    }
}
fn set(a: &mut RideAppearance, p: Part, v: RidePaint) {
    match p {
        Part::Structure => a.structure = v,
        Part::Supports => a.supports = v,
        Part::Vehicles => a.vehicles = v,
    }
}
pub(crate) fn setup(p: &mut ChildSpawnerCommands) {
    p.spawn((rides::RideSection(rides::RideTab::Colors),Node{display:Display::None,width:percent(100),flex_direction:FlexDirection::Column,row_gap:px(10),..default()})).with_children(|p|{
        label(p,"Make it your own",18.0,INK);
        label(p,"Painting is free, including while a ride is open.\nChoose colors, then show the ride in the park.",12.0,MUTED);
        for (part,name) in [(Part::Structure,"Structure / rails or carousel deck"),(Part::Supports,"Supports / columns and mast"),(Part::Vehicles,"Vehicles / cars and carousel seats")]{
            p.spawn(Node{flex_direction:FlexDirection::Column,row_gap:px(4),..default()}).with_children(|p|{
                label(p,name,12.0,INK);
                p.spawn(row()).with_children(|p|{
                    p.spawn((Swatch(part),Node{width:px(34),height:px(30),..default()},BackgroundColor(BORDER),BorderRadius::all(px(4))));
                    button(p,"<",Action::RidePaint(Command::Step(part,-1)),40.0);
                    p.spawn((Text::new("Classic"),TextFont{font_size:14.0,..default()},TextColor(INK),PaintLabel::Name(part),Node{width:px(122),..default()}));
                    button(p,">",Action::RidePaint(Command::Step(part,1)),40.0);
                });
            });
        }
        button(p,"Restore classic colors",Action::RidePaint(Command::Reset),356.0);
        button(p,"Show in park",Action::RidePaint(Command::Find),356.0);
        p.spawn((Text::new(""),TextFont{font_size:12.0,..default()},TextColor(MUTED),PaintLabel::Notice));
    });
}
pub(crate) fn apply(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    let commands = std::mem::take(&mut view.paint_commands);
    if commands.is_empty()
        || !view.rides_open
        || view.help
        || view.guests_open
        || view.services_open
        || view.finance_open
        || view.designs_open
        || view.capture_active
        || view.modal_input_blocked
        || view.diagnostic_case.is_some()
        || view.ride_browse_pending != 0
        || view.save_pending.is_some_and(|p| p != SaveAction::Save)
    {
        return;
    }
    if commands.len() != 1 || commands[0] == Command::Conflict {
        view.paint_notice = Some("Choose one color control at a time.".into());
        return;
    }
    if commands[0] == Command::Show {
        view.ride_tab = rides::RideTab::Colors;
        view.paint_notice = None;
        return;
    }
    if view.ride_tab != rides::RideTab::Colors {
        return;
    }
    let Some(id) = rides::selected_id(&sim.0, &view) else {
        return;
    };
    if commands[0] == Command::Find {
        crate::focus_train(&sim.0, &mut view);
        view.set_rides_modal(false);
        view.modal_transition_pending = true;
        return;
    }
    let Some(old) = sim.0.ride_appearance(id) else {
        return;
    };
    let mut next = old;
    match commands[0] {
        Command::Step(part, delta) => {
            let i = RidePaint::ALL
                .iter()
                .position(|p| *p == get(old, part))
                .unwrap();
            set(
                &mut next,
                part,
                RidePaint::ALL[(i as i32 + i32::from(delta)).rem_euclid(RidePaint::ALL.len() as i32)
                    as usize],
            );
        }
        Command::Reset => {
            next = RideAppearance {
                ride: id,
                ..Default::default()
            }
        }
        _ => return,
    }
    if next == old {
        view.paint_notice = Some("These colors are already selected.".into());
        return;
    }
    view.paint_notice = Some(match sim.0.set_ride_appearance(next) {
        Ok(()) => {
            view.scene_dirty = true;
            "Colors changed. Save the park to keep them.".into()
        }
        Err(e) => format!("Colors unchanged: {e}"),
    });
}
pub(crate) fn refresh(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut labels: Query<(&PaintLabel, &mut Text)>,
    mut swatches: Query<(&Swatch, &mut BackgroundColor)>,
) {
    if !view.rides_open || view.ride_tab != rides::RideTab::Colors {
        return;
    }
    let Some(id) = rides::selected_id(&sim.0, &view) else {
        return;
    };
    let Some(a) = sim.0.ride_appearance(id) else {
        return;
    };
    for (l, mut t) in &mut labels {
        t.0=match l{PaintLabel::Name(p)=>get(a,*p).name().into(),PaintLabel::Notice=>view.paint_notice.clone().unwrap_or_else(||"Classic keeps the original look. Save after repainting.\nGate and queue colors keep their navigation meaning.".into())};
    }
    for (part, mut color) in &mut swatches {
        let fallback = match (sim.0.ride_kind(id), part.0) {
            (Some(RideKind::Carousel), Part::Structure) => [77, 159, 150],
            (Some(RideKind::Carousel), Part::Supports) => [246, 194, 83],
            (Some(RideKind::Carousel), Part::Vehicles) => [246, 231, 185],
            (_, Part::Structure) => [228, 235, 212],
            (_, Part::Supports) => [134, 164, 157],
            (_, Part::Vehicles) => [218, 67, 57],
        };
        let c = get(a, part.0).rgb().unwrap_or(fallback);
        color.0 = Color::srgb_u8(c[0], c[1], c[2]);
    }
}
