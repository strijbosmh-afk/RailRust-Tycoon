//! Player commands during real station transfers and an occupied moving train.
use super::*;
use crate::path_tools::{PathTool, ToolState};
use park_sim::{ObjectId, PathDirection, PathSlope, RideVisitPhase};

fn journey_phase(phase: RideVisitPhase) -> Park {
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
    park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, ride)
        .unwrap();
    for x in 8..=13 {
        park.place_path(TilePos { x, y: 8 }, 8).unwrap();
    }
    for y in 9..=11 {
        park.place_path(TilePos { x: 13, y }, 8).unwrap();
    }
    for _ in 0..12_000 {
        park.advance(Speed::Normal);
        if park.guests().iter().any(|guest| {
            guest.ride_visit.as_ref().is_some_and(|visit| {
                visit.phase == phase
                    && visit.passenger.as_ref().is_some_and(|passenger| {
                        if phase == RideVisitPhase::Riding {
                            passenger.departed && park.train().unwrap().speed > 0
                        } else {
                            passenger
                                .transfer
                                .as_ref()
                                .is_some_and(|transfer| transfer.progress > 0)
                        }
                    })
            })
        }) {
            return park;
        }
    }
    panic!("natural ride journey did not reach {phase:?}");
}

#[test]
fn close_save_and_both_loads_preserve_in_flight_passengers() {
    for phase in [
        RideVisitPhase::Boarding,
        RideVisitPhase::Riding,
        RideVisitPhase::Exiting,
    ] {
        let initial = journey_phase(phase);
        let positions: Vec<_> = initial
            .guests()
            .iter()
            .map(|guest| (guest.id, initial.guest_position(guest.id)))
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
            assert_eq!(
                closed.ride_guest_positions(),
                initial.ride_guest_positions()
            );
            for &(id, position) in &positions {
                assert_eq!(closed.guest_position(id), position);
            }
            assert_eq!(directory.store().load().unwrap(), closed);
            assert_eq!(directory.store().load_backup().unwrap(), initial);
            let files = saved_file_pair(&directory.store());
            persistence_key_frame(&mut app, &[KeyCode::F9, KeyCode::KeyO]);
            assert_eq!(app.world().resource::<Simulation>().0, closed);
            persistence_key_frame(&mut app, &[KeyCode::ShiftLeft, KeyCode::F9, KeyCode::KeyO]);
            assert_eq!(app.world().resource::<Simulation>().0, initial);
            assert_eq!(app.world().resource::<ViewState>().speed, Speed::Paused);
            assert_eq!(
                app.world().resource::<ViewState>().path_tool.mode,
                PathTool::Inspect
            );
            assert_eq!(saved_file_pair(&directory.store()), files);
        }
    }
}

fn aim_surface(app: &mut App, window: Entity, id: ObjectId) {
    let park = &app.world().resource::<Simulation>().0;
    let center = world::surface_center(park, SurfaceId::Object(id), 0).unwrap();
    let point = std::iter::once(center)
        .chain((-26..=26).step_by(2).flat_map(|y| {
            (-32..=32)
                .step_by(2)
                .map(move |x| center + Vec2::new(x as f32, y as f32))
        }))
        .find(|point| {
            world::pick_surface(*point, park, 0, world::ViewMode::Surface)
                == Some(SurfaceId::Object(id))
        })
        .unwrap();
    let pan = point - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    app.world_mut().resource_mut::<ViewState>().pan = pan;
    let cursor = cursor_for_world(app, window, point);
    assert!(is_map_position(
        cursor,
        app.world().get::<Window>(window).unwrap(),
        false
    ));
    set_cursor(app, window, Some(cursor));
}

#[test]
fn active_walkway_demolition_is_blocked_by_pointer_and_delete_without_losing_selection() {
    for phase in [RideVisitPhase::Boarding, RideVisitPhase::Exiting] {
        let park = journey_phase(phase);
        let transfer = park
            .guests()
            .iter()
            .filter_map(|guest| guest.ride_visit.as_ref())
            .filter(|visit| visit.phase == phase)
            .find_map(|visit| visit.passenger.as_ref()?.transfer.as_ref())
            .unwrap();
        for id in [transfer.path, transfer.portal] {
            for pointer in [false, true] {
                let directory = InputSaveDirectory::new();
                directory.store().save(&park).unwrap();
                directory.store().save(&park).unwrap();
                let files = saved_file_pair(&directory.store());
                let (mut app, window) = persistence_input_app(
                    &directory,
                    park.clone(),
                    ViewState {
                        speed: Speed::Paused,
                        selected: Some(SurfaceId::Object(id)),
                        path_tool: ToolState {
                            mode: if pointer {
                                PathTool::Demolish
                            } else {
                                PathTool::Inspect
                            },
                            ..default()
                        },
                        ..default()
                    },
                );
                app.add_systems(
                    Update,
                    apply_path_actions
                        .after(keyboard_input)
                        .before(pointer_input),
                );
                if pointer {
                    aim_surface(&mut app, window, id);
                    frame(&mut app);
                    assert!(
                        !app.world()
                            .resource::<ViewState>()
                            .path_tool
                            .preview
                            .unwrap()
                            .valid
                    );
                    app.world_mut()
                        .resource_mut::<ButtonInput<MouseButton>>()
                        .press(MouseButton::Left);
                    frame(&mut app);
                } else {
                    persistence_key_frame(&mut app, &[KeyCode::Delete]);
                }
                assert_eq!(app.world().resource::<Simulation>().0, park);
                let view = app.world().resource::<ViewState>();
                assert_eq!(view.selected, Some(SurfaceId::Object(id)));
                assert!(view.path_notice.as_ref().unwrap().contains("not removed"));
                assert_eq!(saved_file_pair(&directory.store()), files);
            }
        }
    }
}
