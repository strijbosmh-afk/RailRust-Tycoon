//! Derive a bounded sound scene from the same camera and physical park positions.
use super::synth::{MixState, RideEmitter};
use crate::{ViewState, world};
use bevy::prelude::*;
use park_sim::{CarouselPhase, Park, RideId, RideVisitPhase, TrainPhase};

pub(super) fn scene(park: &Park, view: &ViewState, window: &Window, ui_scale: f32) -> MixState {
    let mut state = MixState::default();
    if view.front_modal {
        return state;
    }
    let (scale, center) = crate::camera_layout_scaled(window, view, park, ui_scale);
    let radius = window.width().min(window.height()) * 0.32;
    let detail = (view.zoom / 3.0).clamp(0.12, 1.0);
    let spatial = |point| attenuation(point, center, scale, radius, detail);
    let occupied = |id: RideId, train: Option<u8>| {
        park.guests().iter().any(|guest| {
            guest.ride_visit.as_ref().is_some_and(|visit| {
                visit.ride == id
                    && visit.phase == RideVisitPhase::Riding
                    && train.is_none_or(|number| {
                        visit.passenger.as_ref().is_some_and(|passenger| {
                            passenger.seat / park.coaster_seats_per_train(id).unwrap_or(1) == number
                        })
                    })
            })
        })
    };
    let mut emitters = Vec::new();
    for id in park.coaster_ids() {
        let Some(model) = park.ride_model(id) else {
            continue;
        };
        let water = matches!(
            model,
            park_sim::RideModel::LogFlume | park_sim::RideModel::RiverRapids
        );
        for train in park.coaster_trains(id) {
            if train.in_depot
                || (train.blocked && !water)
                || matches!(train.phase, TrainPhase::Closed | TrainPhase::Fault)
                || (train.phase == TrainPhase::Station && !water)
            {
                continue;
            }
            let Some(point) = world::leading_train_focus(park, id, train.number, view.rotation)
            else {
                continue;
            };
            let (pan, presence) = spatial(point);
            emitters.push(RideEmitter {
                id: (id, train.number),
                model,
                pan,
                presence,
                speed: (train.speed as f32 / 96.0).clamp(0.0, 1.0),
                lift: f32::from(train.phase == TrainPhase::Lift),
                drop: f32::from(train.phase == TrainPhase::Drop),
                cycle: 0.0,
                occupied: occupied(id, Some(train.number)),
            });
        }
    }
    for ride in park
        .carousels()
        .iter()
        .filter(|ride| ride.phase == CarouselPhase::Running)
    {
        let Some(model) = park.ride_model(ride.ride.id) else {
            continue;
        };
        let Some(point) = world::ride_focus(park, ride.ride.id, view.rotation) else {
            continue;
        };
        let (pan, presence) = spatial(point);
        let cycle_ticks = park
            .ride_operating_settings(ride.ride.id)
            .map_or(720, |settings| settings.cycle_ticks);
        emitters.push(RideEmitter {
            id: (ride.ride.id, 3),
            model,
            pan,
            presence,
            speed: 0.6,
            lift: 0.0,
            drop: 0.0,
            cycle: f32::from(ride.progress) / f32::from(cycle_ticks),
            occupied: occupied(ride.ride.id, None),
        });
    }
    emitters.sort_by(|a, b| {
        b.presence
            .total_cmp(&a.presence)
            .then_with(|| a.id.cmp(&b.id))
    });
    emitters.truncate(state.rides.len());
    // Stable identities prevent a slight camera movement from swapping all voices.
    emitters.sort_by_key(|emitter| emitter.id);
    for (slot, emitter) in state.rides.iter_mut().zip(emitters) {
        *slot = Some(emitter);
    }

    let mut weight = 0.0;
    let mut pan_weight = 0.0;
    let mut happiness = 0.0;
    for guest in park.guests() {
        let Some(point) = world::guest_anchor(park, guest.id, view.rotation) else {
            continue;
        };
        let (pan, presence) = spatial(point.truncate());
        weight += presence;
        pan_weight += pan * presence;
        happiness += f32::from(guest.satisfaction) / 100.0 * presence;
    }
    if weight > 0.0 {
        state.crowd = 1.0 - (-weight / 10.0).exp();
        state.crowd_pan = pan_weight / weight;
        state.happiness = happiness / weight;
    }
    state.rain = park.park_life_status().weather == park_sim::Weather::Rain;
    state
}

fn attenuation(point: Vec2, center: Vec2, scale: f32, radius: f32, detail: f32) -> (f32, f32) {
    let delta = (point - center) / scale.max(0.001);
    let distance = delta.length() / radius.max(1.0);
    let presence = detail / (1.0 + distance * distance).powi(2);
    ((delta.x / radius.max(1.0)).clamp(-1.0, 1.0), presence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_focus_increases_presence_and_offscreen_sources_fade() {
        let (_, overview) = attenuation(Vec2::ZERO, Vec2::ZERO, 3.0, 250.0, 0.2);
        let (_, close) = attenuation(Vec2::ZERO, Vec2::ZERO, 0.5, 250.0, 1.0);
        let (pan, distant) = attenuation(Vec2::new(1500.0, 0.0), Vec2::ZERO, 0.5, 250.0, 1.0);
        assert!(close > overview * 4.0);
        assert!(distant < 0.001);
        assert_eq!(pan, 1.0);
    }

    #[test]
    fn camera_scene_is_pure_bounded_and_title_mutes_park_sources() {
        let park = crate::demonstration_park();
        let before = park.to_save_bytes().unwrap();
        let mut view = ViewState {
            zoom: 4.0,
            front_modal: false,
            ..Default::default()
        };
        let sound = scene(&park, &view, &Window::default(), 1.0);
        assert!(sound.rides.iter().any(Option::is_some));
        assert!(sound.rides.iter().flatten().count() <= 4);
        assert!(
            sound
                .rides
                .iter()
                .flatten()
                .all(|r| r.presence.is_finite() && r.presence <= 1.0)
        );
        view.front_modal = true;
        let title = scene(&park, &view, &Window::default(), 1.0);
        assert_eq!(title.rides, [None; 4]);
        assert_eq!(title.crowd, 0.0);
        assert_eq!(park.to_save_bytes().unwrap(), before);
    }

    #[test]
    fn real_camera_zoom_and_rotations_keep_the_focused_ride_audible() {
        let park = crate::demonstration_park();
        let window = Window::default();
        let initial = scene(&park, &ViewState::default(), &window, 1.0);
        let id = initial.rides.iter().flatten().next().unwrap().id;
        for rotation in 0..4 {
            let point = if id.1 == 3 {
                world::ride_focus(&park, id.0, rotation).unwrap()
            } else {
                world::leading_train_focus(&park, id.0, id.1, rotation).unwrap()
            };
            let mut levels = Vec::new();
            for zoom in [0.7, 4.0] {
                let mut view = ViewState {
                    rotation,
                    zoom,
                    front_modal: false,
                    ..Default::default()
                };
                let (_, center) = crate::camera_layout_scaled(&window, &view, &park, 1.0);
                view.pan += point - center;
                let sound = scene(&park, &view, &window, 1.0);
                let ride = sound
                    .rides
                    .iter()
                    .flatten()
                    .find(|ride| ride.id == id)
                    .unwrap();
                assert!(ride.pan.abs() < 0.001);
                levels.push(ride.presence);
            }
            assert!(levels[1] > levels[0] * 4.0);
        }
    }
}
