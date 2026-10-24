//! Generated access controls, modal boundaries and deliberate pointer placement.
use super::*;
use crate::{SaveSession, Speed, keyboard_input, persistence, pointer_input, ride_operations};
use bevy::{input::mouse::MouseWheel, window::WindowResolution};
use park_sim::{PathSlope, TilePos};
use std::{ffi::OsString, path::PathBuf};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
                "parkbound-access-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )))
    }
    fn session(&self) -> SaveSession {
        SaveSession::from_arguments([
            OsString::from("--save-dir"),
            self.0.clone().into_os_string(),
        ])
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        assert!(self.0.starts_with(std::env::temp_dir()));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> (Park, RideId, RideId) {
    let mut park = crate::demonstration_park();
    park.set_train_open(false);
    let coaster = park.ride().unwrap().id;
    let carousel = park.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
    (park, coaster, carousel)
}

fn test_app(park: Park, view: ViewState, directory: &Directory) -> (App, Entity) {
    let mut app = App::new();
    app.insert_resource(Simulation(park))
        .insert_resource(view)
        .insert_resource(directory.session())
        .insert_resource(Time::<()>::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_message::<MouseWheel>()
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            (
                ui::button_actions,
                keyboard_input,
                crate::coaster_testing::apply_commands,
                apply_commands,
                ride_operations::apply_commands,
                pointer_input,
                persistence::apply_save_action,
                ui::rides::refresh,
                ui::refresh,
            )
                .chain(),
        );
    let mut window = Window {
        resolution: WindowResolution::new(1024, 680),
        focused: true,
        ..default()
    };
    window.set_cursor_position(Some(Vec2::new(650.0, 360.0)));
    let entity = app.world_mut().spawn((window, PrimaryWindow)).id();
    app.update();
    (app, entity)
}

fn frame(app: &mut App, actions: &[ui::Action], keys: &[KeyCode], click: bool) {
    let mut buttons = app.world_mut().query::<(&ui::Action, &mut Interaction)>();
    for (action, mut interaction) in buttons.iter_mut(app.world_mut()) {
        *interaction = if actions.contains(action) {
            Interaction::Pressed
        } else {
            Interaction::None
        };
    }
    let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keyboard.reset_all();
    for key in keys {
        keyboard.press(*key);
    }
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.reset_all();
    if click {
        mouse.press(MouseButton::Left);
    }
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
}

fn view(id: RideId) -> ViewState {
    ViewState {
        rides_open: true,
        ride_tab: ui::rides::RideTab::Access,
        selected_ride: Some(id),
        speed: Speed::Paused,
        ..default()
    }
}

fn cursor_for(app: &App, window: Entity, point: Vec2) -> Vec2 {
    let window = app.world().get::<Window>(window).unwrap();
    let view = app.world().resource::<ViewState>();
    let park = &app.world().resource::<Simulation>().0;
    let (scale, center) = crate::camera_layout(window, view, park);
    let relative = (point - center) / scale;
    Vec2::new(
        window.width() * 0.5 + relative.x,
        window.height() * 0.5 - relative.y,
    )
}

#[test]
fn generated_prepare_button_only_arms_a_valid_ghost_until_deliberate_map_click() {
    let directory = Directory::new();
    let (park, coaster, _) = fixture();
    let before = park.clone();
    let (mut app, window) = test_app(park, view(coaster), &directory);
    let command = AccessCommand::Prepare(RideAccessBuildKind::Entrance);
    frame(&mut app, &[ui::Action::RideAccess(command)], &[], true);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    let state = app.world().resource::<ViewState>();
    assert!(!state.rides_open && state.prepared_access.is_some());
    assert_eq!(state.path_tool.mode, PathTool::Entrance);
    let ghost = state
        .path_tool
        .preview
        .expect("prepared ghost survives the closing click");
    assert!(ghost.valid);
    assert_eq!(ghost.queue_owner, Some(coaster));
    frame(&mut app, &[], &[], false);
    frame(&mut app, &[], &[], true);
    assert_eq!(
        app.world().resource::<Simulation>().0,
        before,
        "old UI cursor click cannot build"
    );
    frame(&mut app, &[], &[], false);
    let point = world::path_build_center(
        ghost.pos,
        &before,
        app.world().resource::<ViewState>().rotation,
        ghost.height,
        ghost.slope,
    )
    .unwrap();
    let cursor = cursor_for(&app, window, point);
    assert!(crate::is_map_position(
        cursor,
        app.world().get::<Window>(window).unwrap(),
        false
    ));
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(cursor));
    frame(&mut app, &[], &[], false);
    assert!(
        app.world()
            .resource::<ViewState>()
            .prepared_access
            .is_none()
    );
    frame(&mut app, &[], &[], true);
    let mut expected = before;
    expected
        .place_ride_entrance(
            ghost.pos,
            ghost.height,
            ghost.entrance_facing.unwrap(),
            coaster,
        )
        .unwrap();
    assert_eq!(app.world().resource::<Simulation>().0, expected);
    app.update();
    assert_eq!(
        app.world().resource::<Simulation>().0,
        expected,
        "held click must not place again"
    );
}

#[test]
fn tabs_own_their_controls_and_browsing_cannot_redirect_an_access_command() {
    let directory = Directory::new();
    let (park, coaster, carousel) = fixture();
    let (mut app, _) = test_app(
        park.clone(),
        ViewState {
            ride_tab: ui::rides::RideTab::Operations,
            ..view(coaster)
        },
        &directory,
    );
    let tune = ui::Action::RideOperating(ride_operations::OperationCommand::Adjust(
        ride_operations::OperatingField::StationWait,
        1,
    ));
    frame(
        &mut app,
        &[ui::Action::RideAccess(AccessCommand::ShowAccess), tune],
        &[],
        false,
    );
    assert_eq!(
        app.world().resource::<ViewState>().ride_tab,
        ui::rides::RideTab::Access
    );
    assert_eq!(app.world().resource::<Simulation>().0, park);
    frame(&mut app, &[tune], &[], false);
    assert_eq!(
        app.world().resource::<Simulation>().0,
        park,
        "hidden operation controls must not apply"
    );
    frame(
        &mut app,
        &[ui::Action::RideAccess(AccessCommand::Prepare(
            RideAccessBuildKind::Entrance,
        ))],
        &[KeyCode::ArrowDown],
        false,
    );
    let state = app.world().resource::<ViewState>();
    assert_eq!(state.selected_ride, Some(carousel));
    assert!(state.rides_open && state.prepared_access.is_none());
    assert_eq!(state.ride_access_plan.as_ref().unwrap().ride, carousel);
    frame(
        &mut app,
        &[ui::Action::RideAccess(AccessCommand::ShowOperations)],
        &[],
        false,
    );
    assert_eq!(
        app.world().resource::<ViewState>().ride_tab,
        ui::rides::RideTab::Operations
    );
    frame(
        &mut app,
        &[ui::Action::RideAccess(AccessCommand::Prepare(
            RideAccessBuildKind::Entrance,
        ))],
        &[],
        false,
    );
    assert!(app.world().resource::<ViewState>().rides_open);
    assert_eq!(app.world().resource::<Simulation>().0, park);
    // A cached plan must not survive topology changes made while its tab is hidden.
    let gate = park
        .ride_access_plan(carousel)
        .unwrap()
        .suggestions
        .into_iter()
        .find(|suggestion| suggestion.kind == RideAccessBuildKind::Entrance)
        .unwrap();
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .place_ride_entrance(gate.pos, gate.height, gate.facing, carousel)
        .unwrap();
    frame(&mut app, &[], &[], false);
    frame(&mut app, &[], &[], false);
    frame(
        &mut app,
        &[ui::Action::RideAccess(AccessCommand::ShowAccess)],
        &[],
        false,
    );
    assert_eq!(
        app.world()
            .resource::<ViewState>()
            .ride_access_plan
            .as_ref()
            .unwrap()
            .entrance
            .status,
        app.world()
            .resource::<Simulation>()
            .0
            .ride_access(carousel)
            .status
    );
}

#[test]
fn close_load_capture_and_stale_selection_preempt_access_preparation() {
    for case in 0..9 {
        let directory = Directory::new();
        let (mut park, coaster, carousel) = fixture();
        let mut state = view(coaster);
        match case {
            0 => state.capture_active = true,
            1 => state.diagnostic_case = Some(park_sim::diagnostics::DiagnosticCase::SaveScale),
            2 => state.save_pending = Some(SaveAction::Load),
            3 => state.save_pending = Some(SaveAction::Backup),
            4 => state.modal_transition_pending = true,
            5 => {
                park.remove_carousel(carousel).unwrap();
                state.selected_ride = Some(carousel);
            }
            _ => {}
        }
        queue(
            &mut state,
            AccessCommand::Prepare(RideAccessBuildKind::Entrance),
        );
        let before = park.clone();
        let (mut app, _) = test_app(park, state, &directory);
        let actions = if case == 6 {
            vec![ui::Action::CloseRides]
        } else {
            vec![]
        };
        let keys = match case {
            7 => vec![KeyCode::Escape],
            8 => vec![KeyCode::ShiftLeft, KeyCode::F9],
            _ => vec![],
        };
        if case >= 6 {
            // Reopen this fixture before testing the simultaneous transition.
            let mut state = app.world_mut().resource_mut::<ViewState>();
            state.rides_open = true;
            state.ride_tab = ui::rides::RideTab::Access;
            state.prepared_access = None;
            state.modal_transition_pending = false;
            queue(
                &mut state,
                AccessCommand::Prepare(RideAccessBuildKind::Entrance),
            );
        }
        frame(&mut app, &actions, &keys, false);
        assert_eq!(
            app.world().resource::<Simulation>().0,
            before,
            "case {case}"
        );
        let state = app.world().resource::<ViewState>();
        assert!(state.prepared_access.is_none(), "case {case}");
        assert!(state.ride_access_commands.is_empty());
    }
}

#[test]
fn existing_gates_are_found_and_unavailable_rides_have_no_build_action() {
    let directory = Directory::new();
    let (mut park, coaster, _) = fixture();
    let suggested = park
        .ride_access_plan(coaster)
        .unwrap()
        .suggestions
        .into_iter()
        .find(|suggestion| suggestion.kind == RideAccessBuildKind::Entrance)
        .unwrap();
    let gate = park
        .place_ride_entrance(suggested.pos, suggested.height, suggested.facing, coaster)
        .unwrap();
    let (mut app, _) = test_app(park.clone(), view(coaster), &directory);
    frame(
        &mut app,
        &[ui::Action::RideAccess(AccessCommand::FindEntrance)],
        &[],
        false,
    );
    let state = app.world().resource::<ViewState>();
    assert_eq!(state.path_tool.mode, PathTool::Inspect);
    assert_eq!(state.selected, Some(SurfaceId::Object(gate)));
    assert_eq!(app.world().resource::<Simulation>().0, park);
    let rail = park.circuit().unwrap().pieces[3].object;
    park.remove_object(rail).unwrap();
    let (mut unavailable, _) = test_app(park.clone(), view(coaster), &directory);
    assert!(!command_enabled(
        &park,
        unavailable.world().resource::<ViewState>(),
        AccessCommand::Prepare(RideAccessBuildKind::Exit)
    ));
    frame(
        &mut unavailable,
        &[ui::Action::RideAccess(AccessCommand::Prepare(
            RideAccessBuildKind::Exit,
        ))],
        &[],
        false,
    );
    assert_eq!(unavailable.world().resource::<Simulation>().0, park);
    assert!(
        unavailable
            .world()
            .resource::<ViewState>()
            .prepared_access
            .is_none()
    );
}

#[test]
fn prepared_tools_clear_on_modal_ride_tool_and_load_changes_without_touching_saves() {
    let directory = Directory::new();
    let (mut park, coaster, carousel) = fixture();
    let session = directory.session();
    let mut state = view(coaster);
    assert!(persistence::perform_action(
        &session,
        &mut park,
        &mut state,
        SaveAction::Save
    ));
    let bytes = std::fs::read(session.store.as_ref().unwrap().save_path()).unwrap();
    let before = park.clone();
    let prepare = AccessCommand::Prepare(RideAccessBuildKind::Entrance);
    for case in 0..4 {
        state = view(coaster);
        assert!(perform_command(&mut park, &mut state, coaster, prepare));
        assert_eq!(park, before);
        match case {
            0 => state.set_rides_modal(true),
            1 => state.select_ride(carousel),
            2 => {
                state.path_tool.set_mode(PathTool::Build);
                assert!(!hold_prepared_preview(
                    &mut park,
                    &mut state,
                    Vec2::ZERO,
                    false,
                    false
                ));
            }
            _ => {
                assert!(persistence::perform_action(
                    &session,
                    &mut park,
                    &mut state,
                    SaveAction::Load
                ));
            }
        }
        assert!(state.prepared_access.is_none());
        assert_eq!(park, before);
    }
    assert_eq!(
        std::fs::read(session.store.as_ref().unwrap().save_path()).unwrap(),
        bytes
    );
}

#[test]
fn all_prepared_suggestions_keep_draft_history_money_and_owners_unchanged() {
    let (mut park, coaster, carousel) = fixture();
    park.copy_installed_track_to_draft().unwrap();
    for id in [coaster, carousel] {
        let plan = park.ride_access_plan(id).unwrap();
        assert!(!plan.suggestions.is_empty());
        for suggestion in plan.suggestions {
            let before = park.clone();
            let mut state = view(id);
            let history = (
                state.track_editor.undo_len(),
                state.track_editor.redo_len(),
                state.track_editor.revision,
            );
            assert!(perform_command(
                &mut park,
                &mut state,
                id,
                AccessCommand::Prepare(suggestion.kind)
            ));
            assert_eq!(park, before);
            assert_eq!(state.selected_ride, Some(id));
            assert_eq!(state.path_tool.height, suggestion.height);
            assert_eq!(state.path_tool.slope, suggestion.slope);
            assert_eq!(state.path_tool.mode, tool(suggestion.kind));
            assert_eq!(
                (
                    state.track_editor.undo_len(),
                    state.track_editor.redo_len(),
                    state.track_editor.revision
                ),
                history
            );
        }
    }
}

#[test]
fn preview_revalidates_if_another_edit_occupies_the_suggested_tile() {
    let (mut park, coaster, _) = fixture();
    let mut state = view(coaster);
    assert!(perform_command(
        &mut park,
        &mut state,
        coaster,
        AccessCommand::Prepare(RideAccessBuildKind::Entrance)
    ));
    let prepared = state.prepared_access.unwrap();
    let suggestion = prepared.suggestion;
    park.place_sloped_path(suggestion.pos, suggestion.height, PathSlope::Flat)
        .unwrap();
    let before = park.clone();
    assert!(hold_prepared_preview(
        &mut park,
        &mut state,
        Vec2::ZERO,
        false,
        false
    ));
    assert!(state.path_tool.preview.is_none_or(|preview| !preview.valid));
    let point = suggestion_point(&park, &state, suggestion).unwrap();
    path_tools::point_at(&mut park, &mut state, point, true);
    assert_eq!(park, before);
}

#[test]
fn intermediate_queue_and_public_path_buttons_arm_the_first_ranked_legal_tile() {
    let directory = Directory::new();
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    park.set_train_open(false);
    let id = park.ride().unwrap().id;
    for kind in [
        RideAccessBuildKind::Entrance,
        RideAccessBuildKind::Queue,
        RideAccessBuildKind::PublicPath,
    ] {
        let plan = park.ride_access_plan(id).unwrap();
        let suggestion = plan
            .suggestions
            .iter()
            .find(|suggestion| suggestion.kind == kind)
            .copied()
            .expect("fixture needs this construction step");
        let before = park.clone();
        let (mut app, _) = test_app(park.clone(), view(id), &directory);
        frame(
            &mut app,
            &[ui::Action::RideAccess(AccessCommand::Prepare(kind))],
            &[],
            false,
        );
        assert_eq!(app.world().resource::<Simulation>().0, before);
        let state = app.world().resource::<ViewState>();
        assert_eq!(state.path_tool.mode, tool(kind));
        assert_eq!(state.path_tool.height, suggestion.height);
        assert_eq!(state.path_tool.slope, suggestion.slope);
        let preview = state.path_tool.preview.expect("intermediate build ghost");
        assert!(preview.valid);
        assert_eq!(preview.pos, suggestion.pos);
        match kind {
            RideAccessBuildKind::Entrance => {
                park.place_ride_entrance(suggestion.pos, suggestion.height, suggestion.facing, id)
                    .unwrap();
            }
            RideAccessBuildKind::Queue => {
                park.place_queue_path(suggestion.pos, suggestion.height, suggestion.slope, id)
                    .unwrap();
            }
            RideAccessBuildKind::PublicPath => {
                park.place_sloped_path(suggestion.pos, suggestion.height, suggestion.slope)
                    .unwrap();
            }
            _ => unreachable!(),
        }
    }
    assert_eq!(
        park.ride_access(id).status,
        park_sim::RideAccessStatus::Ready
    );
}

#[test]
fn exit_return_route_can_be_built_with_generated_prepare_and_deliberate_pointer_clicks() {
    use park_sim::{PathDirection, RideExitStatus};

    for rotation in 0..4 {
        let directory = Directory::new();
        let mut park = crate::demonstration_park();
        park.set_train_open(false);
        let id = park.ride().unwrap().id;
        park.place_ride_entrance(TilePos { x: 8, y: 11 }, 8, PathDirection::North, id)
            .unwrap();
        for y in 12..=15 {
            park.place_queue_path(TilePos { x: 8, y }, 8, PathSlope::Flat, id)
                .unwrap();
        }
        park.place_ride_exit(TilePos { x: 8, y: 9 }, 8, PathDirection::South, id)
            .unwrap();
        for x in 8..=13 {
            park.place_path(TilePos { x, y: 8 }, 8).unwrap();
        }
        assert_eq!(
            park.ride_exit_access(id).status,
            RideExitStatus::PublicDisconnected
        );
        let (mut app, window) = test_app(
            park,
            ViewState {
                rotation,
                ..view(id)
            },
            &directory,
        );
        let prepare =
            ui::Action::RideAccess(AccessCommand::Prepare(RideAccessBuildKind::PublicPath));
        for step in 0..30 {
            let before = app.world().resource::<Simulation>().0.clone();
            if before.ride_exit_access(id).status == RideExitStatus::Ready {
                break;
            }
            let suggestion = before
                .ride_access_plan(id)
                .unwrap()
                .suggestions
                .into_iter()
                .find(|suggestion| suggestion.kind == RideAccessBuildKind::PublicPath)
                .unwrap_or_else(|| panic!("no next path at rotation {rotation}, step {step}"));
            if !app.world().resource::<ViewState>().rides_open {
                frame(&mut app, &[ui::Action::Rides], &[], false);
                frame(&mut app, &[], &[], false);
                frame(
                    &mut app,
                    &[ui::Action::RideAccess(AccessCommand::ShowAccess)],
                    &[],
                    false,
                );
            }
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .set_cursor_position(Some(Vec2::new(650.0, 360.0)));
            frame(&mut app, &[prepare], &[], true);
            assert_eq!(app.world().resource::<Simulation>().0, before);
            let state = app.world().resource::<ViewState>();
            assert!(
                state.prepared_access.is_some(),
                "rotation {rotation}, step {step}: {:?}",
                state.ride_access_notice
            );
            assert_eq!(state.path_tool.mode, PathTool::Build);
            assert_eq!(state.path_tool.height, suggestion.height);
            assert_eq!(state.path_tool.slope, suggestion.slope);
            let ghost = state
                .path_tool
                .preview
                .expect("prepared return path must have a visible ghost");
            assert!(ghost.valid);
            assert_eq!(ghost.pos, suggestion.pos);
            let point = suggestion_point(&before, state, suggestion).unwrap();
            assert_eq!(
                state.pan,
                point - Vec2::Y * f32::from(before.max_surface_height()) * world::HEIGHT_STEP * 0.5
            );
            assert!(!world::sloped_path_plane_occluded(
                point,
                &before,
                rotation,
                suggestion.height,
                suggestion.slope,
                state.view_mode
            ));
            if visible_suggestion_point(&before, rotation, world::ViewMode::Surface, suggestion)
                .is_none()
            {
                assert_eq!(state.view_mode, world::ViewMode::Underground);
            }
            // The button's click, then another click at that unchanged pointer,
            // must both leave construction and the paused simulation untouched.
            frame(&mut app, &[], &[], false);
            frame(&mut app, &[], &[], true);
            assert_eq!(app.world().resource::<Simulation>().0, before);
            frame(&mut app, &[], &[], false);
            let cursor = cursor_for(&app, window, point);
            assert!(crate::is_map_position(
                cursor,
                app.world().get::<Window>(window).unwrap(),
                false
            ));
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .set_cursor_position(Some(cursor));
            frame(&mut app, &[], &[], false);
            assert!(
                app.world()
                    .resource::<ViewState>()
                    .prepared_access
                    .is_none()
            );
            frame(&mut app, &[], &[], true);
            let mut expected = before;
            expected
                .place_sloped_path(suggestion.pos, suggestion.height, suggestion.slope)
                .unwrap();
            assert_eq!(
                app.world().resource::<Simulation>().0,
                expected,
                "rotation {rotation}, step {step}: {:?}",
                app.world().resource::<ViewState>().path_notice
            );
            frame(&mut app, &[], &[], false);
        }
        assert_eq!(
            app.world()
                .resource::<Simulation>()
                .0
                .ride_exit_access(id)
                .status,
            RideExitStatus::Ready,
            "rotation {rotation}"
        );
    }
}

#[test]
fn access_plan_survives_simulation_ticks_and_refreshes_after_reopening() {
    let directory = Directory::new();
    let (mut park, id, _) = fixture();
    park.set_train_open(true);
    let (mut app, _) = test_app(park, view(id), &directory);
    let initial = app
        .world()
        .resource::<ViewState>()
        .ride_access_plan
        .clone()
        .unwrap();
    assert!(!initial.ride_running);
    {
        let mut sim = app.world_mut().resource_mut::<Simulation>();
        for _ in 0..2_000 {
            sim.0.advance(Speed::Normal);
            if sim.0.ride_is_running(id) {
                break;
            }
        }
        assert!(sim.0.ride_is_running(id));
    }
    frame(&mut app, &[], &[], false);
    assert_eq!(
        app.world()
            .resource::<ViewState>()
            .ride_access_plan
            .as_ref(),
        Some(&initial)
    );
    assert!(
        app.world()
            .resource::<Simulation>()
            .0
            .ride_access_plan(id)
            .unwrap()
            .ride_running
    );

    frame(&mut app, &[ui::Action::CloseRides], &[], false);
    frame(&mut app, &[], &[], false);
    assert!(
        app.world()
            .resource::<ViewState>()
            .ride_access_plan
            .is_none()
    );
    app.world_mut()
        .resource_mut::<Simulation>()
        .0
        .place_ride_entrance(
            TilePos { x: 8, y: 11 },
            8,
            park_sim::PathDirection::North,
            id,
        )
        .unwrap();
    frame(&mut app, &[ui::Action::Rides], &[], false);
    frame(&mut app, &[], &[], false);
    frame(
        &mut app,
        &[ui::Action::RideAccess(AccessCommand::ShowAccess)],
        &[],
        false,
    );
    let refreshed = app
        .world()
        .resource::<ViewState>()
        .ride_access_plan
        .as_ref()
        .unwrap();
    assert!(refreshed.ride_running);
    assert_eq!(
        refreshed.entrance.status,
        park_sim::RideAccessStatus::NoQueue
    );
    assert!(refreshed.entrance_candidates.is_empty());
}

#[test]
fn track_occluded_path_center_has_an_exposed_target_that_real_pointer_can_build() {
    let directory = Directory::new();
    let mut park = Park::new(32, 32);
    park.install_train_fixture().unwrap();
    park.set_train_open(false);
    // The height-16 rail at (17,10) projects over this empty height-8 tile.
    // Its center is hidden, but the ordinary path deck has exposed sides.
    let suggestion = RideAccessSuggestion {
        kind: RideAccessBuildKind::PublicPath,
        pos: TilePos { x: 13, y: 6 },
        height: 8,
        facing: park_sim::PathDirection::South,
        slope: PathSlope::Flat,
    };
    park.validate_sloped_path(suggestion.pos, suggestion.height, suggestion.slope)
        .unwrap();
    let mut state = ViewState::default();
    let center = world::path_build_center(suggestion.pos, &park, 0, 8, PathSlope::Flat).unwrap();
    assert!(world::sloped_path_plane_occluded(
        center,
        &park,
        0,
        8,
        PathSlope::Flat,
        state.view_mode
    ));
    let Some(SurfaceId::Object(blocker)) = world::pick_surface(center, &park, 0, state.view_mode)
    else {
        panic!("track must cover the proposed tile center");
    };
    assert_eq!(
        park.object(blocker).unwrap().kind,
        park_sim::ObjectKind::Track
    );
    let point = suggestion_point(&park, &state, suggestion).expect("exposed path edge");
    assert_ne!(point, center);
    focus(&park, &mut state, point);
    let before = park.clone();
    let (mut app, window) = test_app(park, state, &directory);
    frame(
        &mut app,
        &[ui::Action::SetPathTool(PathTool::Build)],
        &[],
        false,
    );
    frame(&mut app, &[], &[], false);
    let cursor = cursor_for(&app, window, point);
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(cursor));
    frame(&mut app, &[], &[], false);
    let preview = app
        .world()
        .resource::<ViewState>()
        .path_tool
        .preview
        .unwrap();
    assert!(preview.valid);
    assert_eq!(preview.pos, suggestion.pos);
    assert_eq!(app.world().resource::<Simulation>().0, before);
    frame(&mut app, &[], &[], true);
    let mut expected = before;
    expected
        .place_sloped_path(suggestion.pos, suggestion.height, suggestion.slope)
        .unwrap();
    assert_eq!(app.world().resource::<Simulation>().0, expected);
}
