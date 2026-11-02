//! Ride name editing owns keyboard text while the ride modal is open.
use super::*;
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};
use park_sim::{MAX_RIDE_NAME_BYTES, MAX_RIDE_NAME_CHARS};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Focus,
    Apply,
    Reset,
}

pub(crate) fn queue(view: &mut ViewState, command: Command) {
    view.ride_name_commands.push(command);
    view.ui_pointer_consumed = true;
}

pub(crate) fn keyboard_text(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut view: ResMut<ViewState>,
) {
    for event in events.read() {
        if !view.rides_open
            || !view.ride_name_focused
            || view.capture_active
            || view.diagnostic_case.is_some()
            || view.modal_input_blocked
            || view.modal_transition_pending
            || event.state != ButtonState::Pressed
        {
            continue;
        }
        let held = |key| keys.pressed(key) || keys.just_pressed(key) || keys.just_released(key);
        let control = held(KeyCode::ControlLeft) || held(KeyCode::ControlRight);
        let alt = held(KeyCode::AltLeft) || held(KeyCode::AltRight);
        if control && !alt && event.key_code == KeyCode::KeyA {
            view.ride_name_select_all = true;
            continue;
        }
        if control && !alt {
            continue;
        }
        if matches!(event.logical_key, Key::Enter) {
            queue(&mut view, Command::Apply);
            continue;
        }
        if matches!(event.logical_key, Key::Tab | Key::Escape) {
            view.ride_name_focused = false;
            view.ride_name_select_all = false;
            continue;
        }
        match event.logical_key {
            Key::Backspace => {
                if view.ride_name_select_all {
                    view.ride_name_draft.clear();
                } else {
                    view.ride_name_draft.pop();
                }
                view.ride_name_select_all = false;
            }
            _ => {
                if let Some(text) = event.text.as_deref() {
                    let text: String = text.chars().filter(|c| !c.is_control()).collect();
                    if text.is_empty() {
                        continue;
                    }
                    if view.ride_name_select_all {
                        view.ride_name_draft.clear();
                    }
                    view.ride_name_select_all = false;
                    for ch in text.chars() {
                        if view.ride_name_draft.chars().count() >= MAX_RIDE_NAME_CHARS
                            || view.ride_name_draft.len() + ch.len_utf8() > MAX_RIDE_NAME_BYTES
                        {
                            break;
                        }
                        view.ride_name_draft.push(ch);
                    }
                }
            }
        }
    }
}

pub(crate) fn apply(mut sim: ResMut<Simulation>, mut view: ResMut<ViewState>) {
    let commands = std::mem::take(&mut view.ride_name_commands);
    if commands.is_empty()
        || !view.rides_open
        || view.help
        || view.guests_open
        || view.services_open
        || view.finance_open
        || view.designs_open
        || view.capture_active
        || view.diagnostic_case.is_some()
        || view.modal_input_blocked
        || view.ride_browse_pending != 0
        || view
            .save_pending
            .is_some_and(|pending| pending != SaveAction::Save)
    {
        return;
    }
    if commands.len() != 1 {
        view.ride_name_notice = Some("Choose one name action at a time.".into());
        return;
    }
    let Some(id) = rides::selected_id(&sim.0, &view) else {
        return;
    };
    match commands[0] {
        Command::Focus => {
            view.ride_name_draft = sim.0.ride_name(id).unwrap_or("").to_owned();
            view.ride_name_owner = Some(id);
            view.ride_name_focused = true;
            view.ride_name_select_all = true;
            view.ride_name_notice = Some("Type a name; Enter or Apply saves it.".into());
        }
        Command::Apply if view.ride_name_owner == Some(id) => {
            match sim.0.set_ride_name(id, &view.ride_name_draft) {
                Ok(()) => {
                    view.ride_name_draft = sim.0.ride_name(id).unwrap_or("").to_owned();
                    view.ride_name_focused = false;
                    view.ride_name_select_all = false;
                    view.ride_name_notice = Some("Ride renamed. Save the park to keep it.".into());
                }
                Err(error) => view.ride_name_notice = Some(format!("Name unchanged: {error}.")),
            }
        }
        Command::Apply => view.ride_name_notice = Some("Choose Edit name first.".into()),
        Command::Reset => match sim.0.reset_ride_name(id) {
            Ok(()) => {
                view.ride_name_draft = sim.0.ride_name(id).unwrap_or("").to_owned();
                view.ride_name_owner = Some(id);
                view.ride_name_focused = false;
                view.ride_name_select_all = false;
                view.ride_name_notice = Some("Default ride name restored. Save to keep it.".into());
            }
            Err(error) => view.ride_name_notice = Some(format!("Name unchanged: {error}.")),
        },
    }
}
