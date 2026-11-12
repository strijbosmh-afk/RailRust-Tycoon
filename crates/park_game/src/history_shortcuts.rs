//! Resolve history letters through the operating system's active keyboard layout.
use crate::track_tools::TrackCommand;
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};

#[derive(Resource, Default)]
pub(crate) struct HistoryShortcuts {
    pub(crate) observed_letter: bool,
    pub(crate) command: Option<TrackCommand>,
}

pub(crate) fn collect(
    keys: Res<ButtonInput<KeyCode>>,
    mut events: MessageReader<KeyboardInput>,
    mut shortcuts: ResMut<HistoryShortcuts>,
) {
    shortcuts.observed_letter = false;
    shortcuts.command = None;
    let held = |key| keys.pressed(key) || keys.just_pressed(key) || keys.just_released(key);
    let control = held(KeyCode::ControlLeft) || held(KeyCode::ControlRight);
    let shift = held(KeyCode::ShiftLeft) || held(KeyCode::ShiftRight);
    for event in events.read() {
        if event.state != ButtonState::Pressed || event.repeat {
            continue;
        }
        let Key::Character(letter) = &event.logical_key else {
            continue;
        };
        shortcuts.observed_letter = true;
        if !control {
            continue;
        }
        shortcuts.command = match letter.to_ascii_lowercase().as_str() {
            "z" if shift => Some(TrackCommand::Redo),
            "z" => Some(TrackCommand::Undo),
            "y" => Some(TrackCommand::Redo),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ViewState, construction_history::EditCommand};

    #[test]
    fn logical_z_on_an_azerty_physical_w_reaches_park_history() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<HistoryShortcuts>()
            .init_resource::<Time>()
            .init_resource::<ViewState>()
            .insert_resource(crate::Simulation(park_sim::Park::new(32, 32)))
            .add_message::<KeyboardInput>()
            .add_systems(Update, (collect, crate::keyboard_input).chain());
        let window = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ControlLeft);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::KeyW,
            logical_key: Key::Character("z".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
        app.update();
        assert_eq!(
            app.world().resource::<ViewState>().park_edit_pending,
            Some(EditCommand::Undo)
        );
        app.world_mut()
            .resource_mut::<ViewState>()
            .park_edit_pending = None;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ControlLeft);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyZ);
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::KeyZ,
            logical_key: Key::Character("w".into()),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
        app.update();
        assert!(
            app.world()
                .resource::<ViewState>()
                .park_edit_pending
                .is_none()
        );
        assert!(!app.world().resource::<ViewState>().track_editor.active);
    }
}
