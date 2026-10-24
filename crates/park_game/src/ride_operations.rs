//! Deliberate, selected-ride operating changes. Simulation owns validation.
use bevy::prelude::*;
use park_sim::{
    Park, RideDispatchPolicy, RideId, RideKind, RideOperatingError, RideOperatingSettings,
};

use crate::{SaveAction, Simulation, ViewState, ui};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperatingField {
    StationWait,
    LiftSpeed,
    BrakeSpeed,
    CycleDuration,
    MinimumPassengers,
    MaxExtraWait,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperationCommand {
    Adjust(OperatingField, i8),
    ResetDefaults,
    ToggleOpen,
}

impl OperatingField {
    pub(crate) fn supported(self, kind: RideKind) -> bool {
        match self {
            Self::StationWait | Self::MinimumPassengers | Self::MaxExtraWait => true,
            Self::LiftSpeed | Self::BrakeSpeed => kind != RideKind::Carousel,
            Self::CycleDuration => kind == RideKind::Carousel,
        }
    }

    fn adjusted(self, mut value: RideOperatingSettings, direction: i8) -> RideOperatingSettings {
        fn change(value: u32, direction: i8, step: u32, min: u32, max: u32) -> u32 {
            (i64::from(value) + i64::from(direction) * i64::from(step))
                .clamp(i64::from(min), i64::from(max)) as u32
        }
        match self {
            Self::StationWait => {
                value.station_wait_ticks = change(
                    u32::from(value.station_wait_ticks),
                    direction,
                    u32::from(park_sim::STATION_WAIT_STEP_TICKS),
                    u32::from(park_sim::MIN_STATION_WAIT_TICKS),
                    u32::from(park_sim::MAX_STATION_WAIT_TICKS),
                ) as u16;
            }
            Self::LiftSpeed => {
                value.lift_speed = change(
                    value.lift_speed,
                    direction,
                    park_sim::LIFT_SPEED_STEP,
                    park_sim::MIN_LIFT_SPEED,
                    park_sim::MAX_LIFT_SPEED,
                );
            }
            Self::BrakeSpeed => {
                value.brake_speed = change(
                    value.brake_speed,
                    direction,
                    park_sim::BRAKE_SPEED_STEP,
                    park_sim::MIN_BRAKE_SPEED,
                    park_sim::MAX_BRAKE_SPEED,
                );
            }
            Self::CycleDuration => {
                value.cycle_ticks = change(
                    u32::from(value.cycle_ticks),
                    direction,
                    u32::from(park_sim::CAROUSEL_CYCLE_STEP_TICKS),
                    u32::from(park_sim::MIN_CAROUSEL_CYCLE_TICKS),
                    u32::from(park_sim::MAX_CAROUSEL_CYCLE_TICKS),
                ) as u16;
            }
            Self::MinimumPassengers | Self::MaxExtraWait => {}
        }
        value
    }

    fn dispatch(self) -> bool {
        matches!(self, Self::MinimumPassengers | Self::MaxExtraWait)
    }

    fn adjusted_policy(self, mut policy: RideDispatchPolicy, direction: i8) -> RideDispatchPolicy {
        match self {
            Self::MinimumPassengers => {
                policy.minimum_passengers = (i16::from(policy.minimum_passengers)
                    + i16::from(direction))
                .clamp(0, i16::from(park_sim::MAX_DISPATCH_PASSENGERS))
                    as u8
            }
            Self::MaxExtraWait => {
                policy.maximum_extra_wait_ticks = (i32::from(policy.maximum_extra_wait_ticks)
                    + i32::from(direction) * i32::from(park_sim::EXTRA_WAIT_STEP_TICKS))
                .clamp(
                    i32::from(park_sim::MIN_EXTRA_WAIT_TICKS),
                    i32::from(park_sim::MAX_EXTRA_WAIT_TICKS),
                ) as u16
            }
            _ => {}
        }
        policy
    }
}

pub(crate) fn queue(view: &mut ViewState, command: OperationCommand) {
    if view.ride_operating_commands.len() < 16 && !view.ride_operating_commands.contains(&command) {
        view.ride_operating_commands.push(command);
    }
}

fn input_blocked(view: &ViewState) -> bool {
    !view.rides_open
        || view.ride_tab != ui::rides::RideTab::Operations
        || view.help
        || view.guests_open
        || view.services_open
        || view.finance_open
        || view.designs_open
        || view.modal_input_blocked
        || view.modal_transition_pending
        || view.capture_active
        || view.diagnostic_case.is_some()
        || view
            .save_pending
            .is_some_and(|action| action != SaveAction::Save)
}

pub(crate) fn command_enabled(park: &Park, view: &ViewState, command: OperationCommand) -> bool {
    if input_blocked(view) || view.ride_browse_pending != 0 {
        return false;
    }
    let Some(id) = ui::rides::selected_id(park, view) else {
        return false;
    };
    let Some(settings) = park.ride_operating_settings(id) else {
        return false;
    };
    if command == OperationCommand::ToggleOpen {
        return !matches!(
            park.ride_operating_settings_status(id),
            Err(RideOperatingError::Unavailable | RideOperatingError::UnknownRide(_))
        );
    }
    if park.ride_operating_settings_status(id).is_err() {
        return false;
    }
    let Some(policy) = park.ride_dispatch_policy(id) else {
        return false;
    };
    match command {
        OperationCommand::Adjust(field, direction) if field.dispatch() => {
            field.adjusted_policy(policy, direction) != policy
        }
        OperationCommand::Adjust(field, direction) => {
            park.ride_kind(id).is_some_and(|kind| field.supported(kind))
                && field.adjusted(settings, direction) != settings
        }
        OperationCommand::ResetDefaults => {
            settings != RideOperatingSettings::default() || policy != RideDispatchPolicy::default()
        }
        OperationCommand::ToggleOpen => true,
    }
}

pub(crate) fn apply_commands(
    mut sim: ResMut<Simulation>,
    mut view: ResMut<ViewState>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
) {
    let commands = std::mem::take(&mut view.ride_operating_commands);
    if commands.is_empty() || input_blocked(&view) {
        return;
    }
    // Browsing is resolved by the panel's later refresh. Never apply a setting
    // to the old ride when the same frame is selecting another one.
    if view.ride_browse_pending != 0
        || keys
            .as_ref()
            .is_some_and(|keys| keys.just_pressed(KeyCode::F9))
    {
        return;
    }
    let Some(id) = ui::rides::selected_id(&sim.0, &view) else {
        view.ride_operating_notice = Some("Choose an available ride first.".into());
        return;
    };
    if commands.contains(&OperationCommand::ToggleOpen) {
        perform_command(&mut sim.0, &mut view, id, OperationCommand::ToggleOpen);
    } else if commands.contains(&OperationCommand::ResetDefaults) {
        perform_command(&mut sim.0, &mut view, id, OperationCommand::ResetDefaults);
    } else {
        for command in commands {
            perform_command(&mut sim.0, &mut view, id, command);
        }
    }
}

/// Shared by visible controls and isolated production proofs. Does not change
/// selection, speed, track draft, history, price or the selected ride's identity.
pub(crate) fn perform_command(
    park: &mut Park,
    view: &mut ViewState,
    id: RideId,
    command: OperationCommand,
) -> bool {
    if command == OperationCommand::ToggleOpen {
        if let Err(error @ (RideOperatingError::Unavailable | RideOperatingError::UnknownRide(_))) =
            park.ride_operating_settings_status(id)
        {
            view.ride_operating_notice = Some(format!("Ride unchanged: {error}"));
            return false;
        }
        let open = !park.ride_is_open(id);
        return match park.set_ride_open(id, open) {
            Ok(()) => {
                view.ride_operating_notice = Some(if open {
                    "Ride opened. Close and empty it before tuning.".into()
                } else {
                    "Close requested. Any current trip finishes before tuning.".into()
                });
                true
            }
            Err(error) => {
                view.ride_operating_notice = Some(format!("Ride unchanged: {error}"));
                false
            }
        };
    }
    let Some(settings) = park.ride_operating_settings(id) else {
        view.ride_operating_notice = Some("Ride is unavailable. Settings are unchanged.".into());
        return false;
    };
    if let OperationCommand::Adjust(field, direction) = command
        && field.dispatch()
    {
        let Some(policy) = park.ride_dispatch_policy(id) else {
            return false;
        };
        return match park.set_ride_dispatch_policy(id, field.adjusted_policy(policy, direction)) {
            Ok(changed) => {
                view.ride_operating_notice = Some(
                    if changed {
                        "Loading policy updated. Save the park to keep it."
                    } else {
                        "Loading policy already matches. No changes made."
                    }
                    .into(),
                );
                changed
            }
            Err(error) => {
                view.ride_operating_notice = Some(format!("Settings unchanged: {error}"));
                false
            }
        };
    }
    if command == OperationCommand::ResetDefaults {
        let Some(policy) = park.ride_dispatch_policy(id) else {
            return false;
        };
        if settings == RideOperatingSettings::default() && policy == RideDispatchPolicy::default() {
            view.ride_operating_notice = Some("Settings already match. No changes made.".into());
            return false;
        }
        // Both domains accept these fixed defaults for every ride family and
        // share the same guard. Validate both before either write; neither
        // setter changes eligibility and no simulation tick runs between them.
        if let Err(error) = park
            .ride_operating_settings_status(id)
            .and_then(|()| park.ride_dispatch_policy_status(id))
        {
            view.ride_operating_notice = Some(format!("Settings unchanged: {error}"));
            return false;
        }
        park.set_ride_operating_settings(id, RideOperatingSettings::default())
            .expect("fixed operating defaults and shared edit guard are validated");
        park.set_ride_dispatch_policy(id, RideDispatchPolicy::default())
            .expect("fixed dispatch defaults and shared edit guard are validated");
        view.ride_operating_notice =
            Some("Operating settings and loading policy reset. Save to keep them.".into());
        return true;
    }
    let requested = match command {
        OperationCommand::Adjust(field, direction) => {
            if !park.ride_kind(id).is_some_and(|kind| field.supported(kind)) {
                view.ride_operating_notice =
                    Some("This setting does not apply to this ride.".into());
                return false;
            }
            field.adjusted(settings, direction)
        }
        OperationCommand::ResetDefaults => RideOperatingSettings::default(),
        OperationCommand::ToggleOpen => unreachable!(),
    };
    match park.set_ride_operating_settings(id, requested) {
        Ok(changed) => {
            view.ride_operating_notice = Some(if changed {
                "Settings updated. Save the park to keep them.".into()
            } else {
                "Settings already match. No changes made.".into()
            });
            changed
        }
        Err(error) => {
            view.ride_operating_notice = Some(format!("Settings unchanged: {error}"));
            false
        }
    }
}

#[cfg(test)]
#[path = "ride_operations_input_tests.rs"]
mod tests;
