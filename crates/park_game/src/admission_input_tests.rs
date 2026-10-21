//! Ride closure/save/load ordering with guests actually occupying a queue.
use super::*;
use park_sim::{PathDirection, PathSlope};

fn occupied_queue() -> Park {
    let mut park = train_park();
    let ride = park.ride().unwrap().id;
    park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, ride)
        .unwrap();
    for (y, height, slope) in [
        (12, 8, PathSlope::Flat),
        (13, 8, PathSlope::South),
        (14, 9, PathSlope::Flat),
        (15, 8, PathSlope::North),
    ] {
        park.place_queue_path(TilePos { x: 8, y }, height, slope, ride)
            .unwrap();
    }
    for _ in 0..1_800 {
        park.advance(Speed::Normal);
        let stats = park.ride_queue_stats(ride);
        if stats.queueing >= 2 && stats.moving > 0 {
            return park;
        }
    }
    panic!("the open connected ride must admit moving guests");
}

#[test]
fn paused_close_and_save_preserve_positions_and_load_wins_over_ride_commands() {
    let initial = occupied_queue();
    let ride = initial.ride().unwrap().id;
    let positions: Vec<_> = initial
        .guests()
        .iter()
        .map(|guest| (guest.id, initial.guest_position(guest.id).unwrap()))
        .collect();
    for keyboard in [true, false] {
        let directory = InputSaveDirectory::new();
        directory.store().save(&initial).unwrap();
        let (mut app, _) = persistence_input_app(
            &directory,
            initial.clone(),
            ViewState {
                speed: Speed::Paused,
                ..default()
            },
        );
        app.add_systems(
            Update,
            apply_train_actions
                .after(keyboard_input)
                .before(pointer_input),
        );
        if keyboard {
            persistence_key_frame(&mut app, &[KeyCode::KeyO, KeyCode::F5]);
        } else {
            let close = app
                .world_mut()
                .spawn((Button, Interaction::Pressed, ui::Action::ToggleTrain))
                .id();
            let save = app
                .world_mut()
                .spawn((Button, Interaction::Pressed, ui::Action::Save))
                .id();
            frame(&mut app);
            app.world_mut().despawn(close);
            app.world_mut().despawn(save);
        }
        let closed = app.world().resource::<Simulation>().0.clone();
        assert_eq!(closed.ticks(), initial.ticks());
        assert!(!closed.train().unwrap().open);
        let stats = closed.ride_queue_stats(ride);
        assert_eq!(stats.queueing, 0);
        assert_eq!(stats.leaving, initial.ride_queue_stats(ride).queueing);
        assert!(stats.draining && !stats.admitting);
        for &(id, position) in &positions {
            assert_eq!(closed.guest_position(id).unwrap(), position);
        }
        assert_eq!(directory.store().load().unwrap(), closed);
        assert_eq!(directory.store().load_backup().unwrap(), initial);
        assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);

        // A primary load suppresses a simultaneous Open command, then an
        // explicit Backup restores the exact occupied, open, mid-edge line.
        persistence_key_frame(&mut app, &[KeyCode::F9, KeyCode::KeyO]);
        assert_eq!(app.world().resource::<Simulation>().0, closed);
        persistence_key_frame(&mut app, &[KeyCode::ShiftLeft, KeyCode::F9, KeyCode::KeyO]);
        assert_eq!(app.world().resource::<Simulation>().0, initial);
        assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
        assert_eq!(
            app.world().resource::<ViewState>().path_tool.mode,
            crate::path_tools::PathTool::Inspect
        );
    }
}
