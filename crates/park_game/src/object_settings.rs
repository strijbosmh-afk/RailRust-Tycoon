//! Short right-clicks inspect the visible owner; right-drag remains camera control.
use bevy::prelude::Vec2;
use park_sim::{Park, RideId, ServiceId, SurfaceId};

use crate::ViewState;

const DRAG_THRESHOLD: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Ride(RideId),
    Service(ServiceId),
}

fn target(park: &Park, surface: SurfaceId) -> Option<Target> {
    let SurfaceId::Object(object) = surface else {
        return None;
    };
    park.service_by_object(object)
        .map(|service| Target::Service(service.id))
        .or_else(|| {
            park.object_ride(object)
                .filter(|id| {
                    park.ride_by_id(*id).is_some()
                        && park
                            .carousel(*id)
                            .is_none_or(|ride| !ride.objects.is_empty())
                })
                .map(Target::Ride)
        })
}

struct Press {
    cursor: Vec2,
    surface: Option<SurfaceId>,
    camera: (Vec2, f32, u8, u64),
    dragged: bool,
    cancelled: bool,
}

#[derive(Default)]
pub(crate) struct RightClick {
    press: Option<Press>,
}

impl RightClick {
    pub(crate) fn reset(&mut self) {
        self.press = None;
    }

    pub(crate) fn begin(&mut self, cursor: Vec2, surface: Option<SurfaceId>, view: &ViewState) {
        self.press = Some(Press {
            cursor,
            surface,
            camera: (view.pan, view.zoom, view.rotation, view.park_generation),
            dragged: false,
            cancelled: false,
        });
    }

    pub(crate) fn track(&mut self, cursor: Vec2) {
        if let Some(press) = &mut self.press {
            press.dragged |= press.cursor.distance_squared(cursor) > DRAG_THRESHOLD.powi(2);
        }
    }

    pub(crate) fn cancel(&mut self) {
        if let Some(press) = &mut self.press {
            press.cancelled = true;
        }
    }

    pub(crate) fn dragged(&self) -> bool {
        self.press.as_ref().is_some_and(|press| press.dragged)
    }

    pub(crate) fn finish(&mut self, park: &Park, view: &mut ViewState) -> bool {
        let Some(press) = self.press.take() else {
            return false;
        };
        if press.cancelled
            || press.dragged
            || view.edits_blocked()
            || press.camera != (view.pan, view.zoom, view.rotation, view.park_generation)
        {
            return false;
        }
        let Some(surface) = view.hovered else {
            return false;
        };
        let Some(owner) = target(park, surface) else {
            return false;
        };
        if press.surface.and_then(|surface| target(park, surface)) != Some(owner) {
            return false;
        }
        match owner {
            Target::Ride(id) => {
                view.set_rides_modal(true);
                view.select_ride(id);
            }
            Target::Service(id) => {
                view.set_modal(false, false, true, false);
                view.selected_service = Some(id);
                view.service_browse_pending = 0;
                view.service_commands.clear();
                view.service_notice = None;
            }
        }
        view.selected = Some(surface);
        view.modal_transition_pending = true;
        true
    }
}
