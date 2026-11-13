use super::*;
use crate::ui::park_life::Command;

fn press(app: &mut App, action: ui::Action) {
    let entity = app
        .world_mut()
        .spawn((Button, Interaction::Pressed, action))
        .id();
    frame(app);
    app.world_mut().despawn(entity);
}

#[test]
fn management_buttons_save_staff_and_load_preempts_same_frame_hiring() {
    let directory = InputSaveDirectory::new();
    let mut view = ViewState::default();
    ui::park_life::open(&mut view);
    let (mut app, _) = persistence_input_app(&directory, Park::new(32, 32), view);
    press(
        &mut app,
        ui::Action::ParkLife(Command::Hire(park_sim::StaffRole::Mechanic)),
    );
    assert_eq!(app.world().resource::<Simulation>().0.park_staff().len(), 1);
    persistence_key_frame(&mut app, &[KeyCode::F5]);
    let saved = directory.store().load().unwrap();
    let entity = app
        .world_mut()
        .spawn((
            Button,
            Interaction::Pressed,
            ui::Action::ParkLife(Command::Hire(park_sim::StaffRole::Security)),
        ))
        .id();
    persistence_key_frame(&mut app, &[KeyCode::F9]);
    app.world_mut().despawn(entity);
    assert_eq!(app.world().resource::<Simulation>().0, saved);
}

#[test]
fn shelter_path_click_is_consumed_and_escape_cancels_the_picker() {
    let directory = InputSaveDirectory::new();
    let mut park = Park::new(32, 32);
    let tile = TilePos { x: 16, y: 16 };
    let id = park.place_path(tile, 0).unwrap();
    let mut view = ViewState::default();
    ui::park_life::open(&mut view);
    let (mut app, window) = persistence_input_app(&directory, park, view);
    press(&mut app, ui::Action::ParkLife(Command::Shelter));
    let point = world::tile_center(tile, 32, 0) + Vec2::Y * 0.22 * world::HEIGHT_STEP;
    let cursor = cursor_for_world(&app, window, point);
    set_cursor(&mut app, window, Some(cursor));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    frame(&mut app);
    assert!(app.world().resource::<Simulation>().0.path_is_sheltered(id));
    assert!(app.world().resource::<ViewState>().park_life_open);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    press(&mut app, ui::Action::ParkLife(Command::Shelter));
    persistence_key_frame(&mut app, &[KeyCode::Escape]);
    assert!(app.world().resource::<ViewState>().park_life_pick.is_none());
}
