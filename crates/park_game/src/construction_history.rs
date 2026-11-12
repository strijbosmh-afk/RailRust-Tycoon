//! Bounded, session-only undo for edits made while simulation time is paused.
//! An edit restores an exact park snapshot only while no simulation tick has run.
use bevy::prelude::*;
use park_sim::{Park, Speed};

use crate::{Simulation, ViewState};

const HISTORY_LIMIT: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EditCommand {
    Undo,
    Redo,
}

#[derive(Resource, Default)]
pub(crate) struct EditHistory {
    undo: Vec<Park>,
    redo: Vec<Park>,
    before: Option<Park>,
    generation: u64,
    skip_finish: bool,
}

impl EditHistory {
    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn clear(&mut self, generation: u64) {
        self.undo.clear();
        self.redo.clear();
        self.before = None;
        self.generation = generation;
        self.skip_finish = false;
    }

    fn push(stack: &mut Vec<Park>, park: Park) {
        if stack.len() == HISTORY_LIMIT {
            stack.remove(0);
        }
        stack.push(park);
    }

    fn record(&mut self, before: Park, after: &Park) {
        if before != *after {
            Self::push(&mut self.undo, before);
            self.redo.clear();
        }
    }

    fn restore(&mut self, park: &mut Park, command: EditCommand) -> bool {
        let (source, destination) = match command {
            EditCommand::Undo => (&mut self.undo, &mut self.redo),
            EditCommand::Redo => (&mut self.redo, &mut self.undo),
        };
        let Some(previous) = source.pop() else {
            return false;
        };
        Self::push(destination, std::mem::replace(park, previous));
        self.skip_finish = true;
        true
    }
}

fn session_valid(view: &ViewState) -> bool {
    view.speed == Speed::Paused
        && !view.capture_active
        && !view.front_modal
        && view.diagnostic_case.is_none()
}

fn available(view: &ViewState) -> bool {
    session_valid(view) && !view.navigation_open && !view.designs_open && !view.help
}

pub(crate) fn begin(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pressed: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut history: ResMut<EditHistory>,
) {
    history.before = None;
    history.skip_finish = false;
    if history.generation != view.park_generation || !session_valid(&view) {
        history.clear(view.park_generation);
        return;
    }
    let actionable = mouse.is_some_and(|input| input.just_pressed(MouseButton::Left))
        || keys.is_some_and(|input| input.get_just_pressed().next().is_some())
        || pressed
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed);
    if actionable {
        history.before = Some(sim.0.clone());
    }
}

pub(crate) fn apply(
    mut sim: ResMut<Simulation>,
    mut view: ResMut<ViewState>,
    mut history: ResMut<EditHistory>,
) {
    let Some(command) = view.park_edit_pending.take() else {
        return;
    };
    view.ui_pointer_consumed = true;
    if !available(&view)
        || view.edits_blocked()
        || view.track_editor.active
        || view.save_pending.is_some()
    {
        view.path_notice = Some("Pause and close menus before using edit history.".into());
        return;
    }
    if !history.restore(&mut sim.0, command) {
        view.path_notice = Some("No paused edit in that direction.".into());
        return;
    }
    let (profile, custom) = crate::persistence::terrain_style(&sim.0);
    view.terrain_profile = profile;
    view.terrain_custom = custom;
    view.selected = view
        .selected
        .filter(|surface| sim.0.surface_position(*surface).is_some());
    view.hovered = None;
    view.prepared_access = None;
    view.coaster_testing.report = None;
    view.path_tool.clear_pointer();
    view.track_editor = Default::default();
    view.scene_dirty = true;
    view.path_notice = Some(
        match command {
            EditCommand::Undo => "Undid last paused edit. Cash and park state restored.",
            EditCommand::Redo => "Redid paused edit. Cash and park state restored.",
        }
        .into(),
    );
}

pub(crate) fn finish(sim: Res<Simulation>, view: Res<ViewState>, mut history: ResMut<EditHistory>) {
    if history.generation != view.park_generation || !session_valid(&view) {
        history.clear(view.park_generation);
        return;
    }
    if history.skip_finish {
        history.before = None;
        history.skip_finish = false;
        return;
    }
    if let Some(before) = history.before.take() {
        if before.ticks() != sim.0.ticks() {
            history.clear(view.park_generation);
        } else {
            history.record(before, &sim.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::{SceneryKind, TilePos};

    fn build_on_key(keys: Res<ButtonInput<KeyCode>>, mut sim: ResMut<Simulation>) {
        if keys.just_pressed(KeyCode::KeyP) {
            sim.0
                .build_scenery(TilePos { x: 5, y: 5 }, SceneryKind::Tree)
                .unwrap();
        }
    }

    #[test]
    fn real_update_order_and_buttons_undo_paid_build_without_replaying_time() {
        let mut app = App::new();
        let before = Park::new(32, 32);
        app.insert_resource(Simulation(before.clone()))
            .insert_resource(ViewState {
                speed: Speed::Paused,
                ..Default::default()
            })
            .insert_resource(ButtonInput::<KeyCode>::default())
            .init_resource::<EditHistory>()
            .add_systems(
                Update,
                (
                    begin,
                    crate::ui::button_actions,
                    build_on_key,
                    apply,
                    finish,
                )
                    .chain(),
            );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyP);
        app.update();
        let built = app.world().resource::<Simulation>().0.clone();
        assert_ne!(built, before);
        assert!(app.world().resource::<EditHistory>().can_undo());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();

        let undo = app
            .world_mut()
            .spawn((Button, Interaction::Pressed, crate::ui::Action::UndoPark))
            .id();
        app.update();
        assert_eq!(app.world().resource::<Simulation>().0, before);
        assert!(app.world().resource::<EditHistory>().can_redo());
        app.world_mut().despawn(undo);

        app.world_mut()
            .spawn((Button, Interaction::Pressed, crate::ui::Action::RedoPark));
        app.update();
        assert_eq!(app.world().resource::<Simulation>().0, built);
        assert_eq!(
            Park::from_save_bytes(&built.to_save_bytes().unwrap()),
            Ok(built)
        );
        app.world_mut()
            .resource_mut::<ViewState>()
            .set_speed(Speed::Normal);
        app.update();
        assert!(!app.world().resource::<EditHistory>().can_undo());
    }

    #[test]
    fn paused_edits_restore_exact_cash_objects_and_ledger_then_branch() {
        let mut history = EditHistory::default();
        let mut park = Park::new(32, 32);
        let before = park.clone();
        park.build_scenery(TilePos { x: 5, y: 5 }, SceneryKind::Tree)
            .unwrap();
        let built = park.clone();
        history.record(before.clone(), &park);
        assert!(history.restore(&mut park, EditCommand::Undo));
        assert_eq!(park, before);
        assert!(history.restore(&mut park, EditCommand::Redo));
        assert_eq!(park, built);
        assert!(history.restore(&mut park, EditCommand::Undo));
        park.build_scenery(TilePos { x: 6, y: 5 }, SceneryKind::FlowerBed)
            .unwrap();
        history.record(before, &park);
        assert!(!history.can_redo());
        assert!(history.can_undo());
        assert_eq!(
            Park::from_save_bytes(&park.to_save_bytes().unwrap()),
            Ok(park)
        );
    }

    #[test]
    fn no_op_and_clear_do_not_leave_stale_history() {
        let mut history = EditHistory::default();
        let park = Park::new(32, 32);
        history.record(park.clone(), &park);
        assert!(!history.can_undo());
        let mut changed = park.clone();
        changed
            .build_scenery(TilePos { x: 5, y: 5 }, SceneryKind::Tree)
            .unwrap();
        history.record(park, &changed);
        history.clear(1);
        assert!(!history.can_undo() && !history.can_redo());
        assert!(!history.restore(&mut changed, EditCommand::Undo));
    }
}
