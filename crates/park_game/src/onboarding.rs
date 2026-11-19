//! A replayable guide that observes the actual park and opens existing tools.
use std::collections::BTreeSet;

use crate::{Simulation, ViewState, accessibility::Preferences};
use bevy::prelude::*;
use park_sim::{ObjectId, Park, RideAccessStatus, RideId, RideModel, Speed};

#[derive(Resource, Default)]
pub(crate) struct Guide {
    active: bool,
    step: usize,
    generation: u64,
    baseline_paths: BTreeSet<ObjectId>,
    baseline_rides: Vec<park_sim::RideId>,
    saved_revision: u64,
}
#[derive(Component)]
struct Panel;
#[derive(Component)]
enum GuideText {
    Body,
    Button(Control),
}
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Control {
    Tool,
    Next,
    Back,
    Skip,
}
const STEPS: [(&str, &str, &str); 7] = [
    (
        "Welcome to your first park",
        "Use the mouse wheel to zoom, drag with the middle button to pan, or use the arrow keys. Tab and Enter also operate menus. This guide can be skipped and replayed from Help.",
        "Fit park",
    ),
    (
        "Extend a public path",
        "Choose Paths and click empty, owned ground beside a public path. The preview shows the height, price and any obstruction. A valid connection uses matching edge heights.",
        "Choose Paths",
    ),
    (
        "Build a gentle ride",
        "Place a Carousel on clear owned ground beside your path. Green previews are valid. The price is paid only when construction succeeds. Inspect it to manage operation.",
        "Place Carousel",
    ),
    (
        "Connect its entrance and exit",
        "In Rides, choose your new ride and open Access. Build an entrance and a queue that ends at one public path. Connect an exit to a public path. Resolve the displayed reason if access is incomplete.",
        "Ride access",
    ),
    (
        "Welcome your guests",
        "Open the ride after its checks pass. Visitor Flow chooses a connected public entry, enables arrivals and opens the park. Let time run until someone completes a ride. Guests need affordable tickets and services.",
        "Visitor Flow",
    ),
    (
        "Save your park",
        "Use Save or your configured Save shortcut. The park library keeps your progress and a previous backup. Wait for the Saved message before closing.",
        "Save park",
    ),
    (
        "Resume whenever you like",
        "Open the Park menu to return to the title. Continue latest park loads your saved park. You can also choose a park or its backup from Load Park. Help contains construction and coaster reference pages.",
        "Park menu",
    ),
];
pub(crate) fn configure(app: &mut App) {
    app.init_resource::<Guide>()
        .add_systems(Startup, setup.after(crate::ui::setup))
        .add_systems(
            Update,
            (update, block_pointer)
                .chain()
                .after(crate::ui::button_actions)
                .before(crate::front_end::input),
        );
}
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Panel,
            GlobalZIndex(18),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: percent(25),
                bottom: px(42),
                width: percent(49),
                max_width: px(620),
                padding: UiRect::all(px(13)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            BackgroundColor(crate::ui::PAPER),
            BorderRadius::all(px(10)),
        ))
        .with_children(|p| {
            p.spawn((
                GuideText::Body,
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(crate::ui::INK),
            ));
            p.spawn(crate::ui::row()).with_children(|p| {
                for (name, control) in [
                    ("Open tool", Control::Tool),
                    ("Next", Control::Next),
                    ("Back", Control::Back),
                    ("Skip guide", Control::Skip),
                ] {
                    p.spawn((
                        Button,
                        control,
                        Node {
                            min_height: px(32),
                            padding: UiRect::all(px(7)),
                            ..default()
                        },
                        BackgroundColor(crate::ui::BUTTON),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            GuideText::Button(control),
                            Text::new(name),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(crate::ui::INK),
                        ));
                    });
                }
            });
        });
}
fn update(
    mut guide: ResMut<Guide>,
    prefs: Res<Preferences>,
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
    buttons: Query<(&Interaction, &Control), Changed<Interaction>>,
    mut panels: Query<&mut Node, With<Panel>>,
    mut labels: Query<(&GuideText, &mut Text)>,
) {
    if std::mem::take(&mut view.tutorial_requested) {
        *guide = Guide {
            active: true,
            generation: view.park_generation,
            baseline_paths: sim
                .0
                .objects()
                .iter()
                .filter(|o| o.is_public_path())
                .map(|o| o.id)
                .collect(),
            baseline_rides: sim.0.all_rides().map(|r| r.id).collect(),
            saved_revision: view.successful_saves,
            ..default()
        };
    }
    if guide.generation != view.park_generation {
        guide.active = false;
    }
    let visible = guide.active
        && !view.capture_active
        && !view.front_modal
        && !view.accessibility_open
        && !view.help
        && !view.navigation_open
        && !view.catalogue_open
        && !view.rides_open
        && !view.services_open
        && !view.guests_open
        && !view.finance_open
        && !view.designs_open
        && !view.management_open
        && !view.workshop_open
        && !view.park_life_open;
    for mut panel in &mut panels {
        panel.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !visible {
        return;
    }
    let complete = practice_complete(&guide, &sim.0, &view);
    let (title, copy, tool) = STEPS[guide.step];
    for (kind, mut label) in &mut labels {
        label.0 = match kind {
            GuideText::Button(Control::Tool) => tool.into(),
            GuideText::Button(Control::Next) if guide.step == STEPS.len() - 1 => {
                "Next - finish guide".into()
            }
            GuideText::Button(Control::Next) => "Next step - practice is optional".into(),
            GuideText::Button(Control::Back) => "Previous step".into(),
            GuideText::Button(Control::Skip) => "Close guide - replay from Help".into(),
            GuideText::Body => format!(
                "{} / {} · {}\n{}\nPractice: {}\nNext always continues; you can practise now or return later.\nPaths [{}]  Queues [{}]  Rides [{}]  Save [{}]",
                guide.step + 1,
                STEPS.len(),
                title,
                copy,
                if complete {
                    "Complete"
                } else {
                    practice_hint(guide.step)
                },
                prefs.value.shortcut(4),
                prefs.value.shortcut(5),
                prefs.value.shortcut(13),
                prefs.value.shortcut(15)
            ),
        };
    }
    for (interaction, control) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        view.ui_pointer_consumed = true;
        match control {
            Control::Skip => guide.active = false,
            Control::Next => {
                guide.step += 1;
                if guide.step == STEPS.len() {
                    guide.active = false;
                    guide.step = 0;
                }
            }
            Control::Back => guide.step = guide.step.saturating_sub(1),
            Control::Tool => match guide.step {
                0 => view.reset_camera(),
                1 => view.set_path_tool(crate::path_tools::PathTool::Build),
                2 => {
                    view.path_tool.ride_model = RideModel::Carousel;
                    view.path_tool.flat_ride_kind = RideModel::Carousel.base_kind();
                    view.set_path_tool(crate::path_tools::PathTool::Carousel);
                }
                3 => {
                    let ride = practice_ride(&guide, &sim.0, &view);
                    view.set_rides_modal(true);
                    view.ride_tab = crate::ui::rides::RideTab::Access;
                    view.selected_ride = ride;
                }
                4 => {
                    view.set_modal(false, true, false, false);
                    view.guest_flow_view = true;
                    view.set_speed(Speed::Normal);
                }
                5 => view.request_save(crate::SaveAction::Save),
                _ => view.front_request_menu = true,
            },
        }
        break;
    }
}

fn practice_ride(guide: &Guide, park: &Park, view: &ViewState) -> Option<RideId> {
    park.all_rides()
        .find(|ride| !guide.baseline_rides.contains(&ride.id))
        .map(|ride| ride.id)
        .or_else(|| {
            view.selected_ride
                .filter(|id| park.ride_by_id(*id).is_some())
        })
        .or_else(|| {
            park.all_rides()
                .find(|ride| park.ride_model(ride.id) == Some(RideModel::Carousel))
                .map(|ride| ride.id)
        })
        .or_else(|| park.all_rides().next().map(|ride| ride.id))
}

fn practice_complete(guide: &Guide, park: &Park, view: &ViewState) -> bool {
    match guide.step {
        1 => park.objects().iter().any(|path| {
            path.is_public_path()
                && !guide.baseline_paths.contains(&path.id)
                && !park.path_neighbors(path.id).is_empty()
        }),
        2 => park
            .all_rides()
            .any(|ride| !guide.baseline_rides.contains(&ride.id)),
        3 => practice_ride(guide, park, view).is_some_and(|id| {
            park.ride_access(id).status == RideAccessStatus::Ready
                && park.ride_opening_status(id).is_ok()
        }),
        4 => practice_ride(guide, park, view)
            .and_then(|id| park.ride_by_id(id))
            .is_some_and(|ride| ride.completed > 0),
        5 => view.successful_saves != guide.saved_revision,
        _ => true,
    }
}

fn practice_hint(step: usize) -> &'static str {
    match step {
        1 => "Add a NEW public path tile connected to another public path.",
        2 => "Build a new Carousel, or continue to inspect an existing ride.",
        3 => "Connect a ride's entrance, queue and public exit route.",
        4 => "Let a guest complete a trip on the selected ride.",
        5 => "Save successfully using the Save button or your shortcut.",
        _ => "Explore at your own pace.",
    }
}

fn block_pointer(
    view: ResMut<ViewState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    panels: Query<(&ComputedNode, &UiGlobalTransform), With<Panel>>,
) {
    let mut view = view;
    let Some(cursor) = windows.iter().next().and_then(Window::cursor_position) else {
        return;
    };
    let dpi = windows.iter().next().map_or(1.0, Window::scale_factor);
    for (node, position) in &panels {
        if node.size().min_element() > 0.0
            && (cursor * dpi - position.translation)
                .abs()
                .cmple(node.size() / 2.0)
                .all()
        {
            view.ui_pointer_consumed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn completed(app: &App) -> bool {
        practice_complete(
            app.world().resource::<Guide>(),
            &app.world().resource::<Simulation>().0,
            app.world().resource::<ViewState>(),
        )
    }

    #[test]
    fn optional_practice_observes_construction_visits_and_successful_saves() {
        use park_sim::{Park, PathDirection, PathSlope, TerrainProfile, TilePos};
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Flat);
        let mut app = App::new();
        app.insert_resource(Preferences::load(None))
            .insert_resource(Guide::default())
            .insert_resource(Simulation(park))
            .insert_resource(ViewState {
                tutorial_requested: true,
                ..default()
            })
            .add_systems(Update, update);
        app.update();
        let button = app
            .world_mut()
            .spawn((Interaction::None, Control::Next))
            .id();
        fn next(app: &mut App, button: Entity) {
            *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
            app.update();
            *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
            app.update();
        }
        next(&mut app, button);
        assert_eq!(app.world().resource::<Guide>().step, 1);
        assert!(!completed(&app));
        {
            let mut sim = app.world_mut().resource_mut::<Simulation>();
            for x in 2..=10 {
                sim.0.place_path(TilePos { x, y: 4 }, 0).unwrap();
            }
        }
        assert!(completed(&app));
        next(&mut app, button);
        assert_eq!(app.world().resource::<Guide>().step, 2);
        assert!(!completed(&app));
        let id = app
            .world_mut()
            .resource_mut::<Simulation>()
            .0
            .build_catalogue_ride(RideModel::Carousel, TilePos { x: 5, y: 7 }, 0, 0)
            .unwrap();
        assert!(completed(&app));
        next(&mut app, button);
        assert_eq!(app.world().resource::<Guide>().step, 3);
        assert!(!completed(&app));
        {
            let mut sim = app.world_mut().resource_mut::<Simulation>();
            sim.0
                .place_queue_path(TilePos { x: 5, y: 5 }, 0, PathSlope::Flat, id)
                .unwrap();
            sim.0
                .place_ride_entrance(TilePos { x: 5, y: 6 }, 0, PathDirection::South, id)
                .unwrap();
            sim.0
                .place_ride_exit(TilePos { x: 7, y: 6 }, 0, PathDirection::South, id)
                .unwrap();
            sim.0.place_path(TilePos { x: 7, y: 5 }, 0).unwrap();
            let entry = sim
                .0
                .objects()
                .iter()
                .find(|o| o.pos == TilePos { x: 2, y: 4 })
                .unwrap()
                .id;
            sim.0.set_visitor_entry(entry).unwrap();
            sim.0.set_visitor_arrivals(true).unwrap();
            sim.0.set_ride_ticket_price(id, 100).unwrap();
            sim.0.set_ride_open_checked(id, true).unwrap();
        }
        assert!(completed(&app));
        next(&mut app, button);
        assert_eq!(app.world().resource::<Guide>().step, 4);
        {
            let mut sim = app.world_mut().resource_mut::<Simulation>();
            for _ in 0..12000 {
                sim.0.advance(Speed::Normal);
                if sim.0.ride_by_id(id).unwrap().completed > 0 {
                    break;
                }
            }
            assert!(sim.0.ride_by_id(id).unwrap().completed > 0);
        }
        assert!(completed(&app));
        next(&mut app, button);
        assert_eq!(app.world().resource::<Guide>().step, 5);
        assert!(!completed(&app));
        app.world_mut().resource_mut::<ViewState>().successful_saves += 1;
        assert!(completed(&app));
        next(&mut app, button);
        assert_eq!(app.world().resource::<Guide>().step, 6);
        next(&mut app, button);
        assert!(!app.world().resource::<Guide>().active);
    }
    #[test]
    fn guide_uses_real_tools_and_can_be_replayed_without_changing_the_park() {
        let mut app = App::new();
        app.insert_resource(Preferences::load(None))
            .insert_resource(Guide::default())
            .insert_resource(Simulation(crate::new_game_park()))
            .insert_resource(ViewState {
                tutorial_requested: true,
                ..default()
            })
            .add_systems(Update, update);
        app.update();
        assert!(app.world().resource::<Guide>().active);
        app.world_mut().resource_mut::<Guide>().step = 1;
        let before = app
            .world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .unwrap();
        let id = app
            .world_mut()
            .spawn((Interaction::Pressed, Control::Tool))
            .id();
        app.update();
        assert_eq!(
            app.world().resource::<ViewState>().path_tool.mode,
            crate::path_tools::PathTool::Build
        );
        assert_eq!(
            app.world()
                .resource::<Simulation>()
                .0
                .to_save_bytes()
                .unwrap(),
            before
        );
        *app.world_mut().get_mut::<Interaction>(id).unwrap() = Interaction::None;
        app.update();
        app.world_mut().entity_mut(id).insert(Control::Skip);
        *app.world_mut().get_mut::<Interaction>(id).unwrap() = Interaction::Pressed;
        app.update();
        assert!(!app.world().resource::<Guide>().active);
        app.world_mut()
            .resource_mut::<ViewState>()
            .tutorial_requested = true;
        app.update();
        assert_eq!(app.world().resource::<Guide>().step, 0);
    }

    fn guide_app(park: Park) -> App {
        let mut app = App::new();
        app.insert_resource(Preferences::load(None))
            .insert_resource(Guide::default())
            .insert_resource(Simulation(park))
            .insert_resource(ViewState {
                tutorial_requested: true,
                ..default()
            })
            .add_systems(Startup, setup)
            .add_systems(Update, update);
        app.update();
        app
    }

    fn button(app: &mut App, wanted: Control) -> Entity {
        app.world_mut()
            .query::<(Entity, &Control)>()
            .iter(app.world())
            .find(|(_, control)| **control == wanted)
            .map(|(entity, _)| entity)
            .unwrap()
    }

    fn click(app: &mut App, entity: Entity) {
        app.world_mut().entity_mut(entity).insert(Interaction::None);
        app.update();
        app.world_mut()
            .entity_mut(entity)
            .insert(Interaction::Pressed);
        app.update();
    }

    #[test]
    fn next_can_browse_all_seven_steps_without_building_or_spending() {
        for park in [
            crate::new_game_park(),
            park_sim::demo_park::build_demo_park().unwrap(),
        ] {
            let mut app = guide_app(park);
            let before = app
                .world()
                .resource::<Simulation>()
                .0
                .to_save_bytes()
                .unwrap();
            let next = button(&mut app, Control::Next);
            for step in 0..STEPS.len() {
                assert!(app.world().resource::<Guide>().active);
                assert_eq!(app.world().resource::<Guide>().step, step);
                click(&mut app, next);
                // A held button is a single interaction, not repeated navigation.
                app.update();
                if step + 1 < STEPS.len() {
                    assert_eq!(app.world().resource::<Guide>().step, step + 1);
                }
            }
            assert!(!app.world().resource::<Guide>().active);
            assert_eq!(
                app.world()
                    .resource::<Simulation>()
                    .0
                    .to_save_bytes()
                    .unwrap(),
                before
            );
            assert!(
                app.world_mut()
                    .query_filtered::<&Node, With<Panel>>()
                    .iter(app.world())
                    .all(|node| node.display == Display::None)
            );
        }
    }

    #[test]
    fn path_practice_uses_new_connected_ids_not_a_net_count() {
        use park_sim::{TerrainProfile, TilePos};
        let mut park = Park::new(32, 32);
        park.apply_terrain_profile(TerrainProfile::Flat);
        let old = park.place_path(TilePos { x: 2, y: 4 }, 0).unwrap();
        park.place_path(TilePos { x: 3, y: 4 }, 0).unwrap();
        let mut app = guide_app(park);
        app.world_mut().resource_mut::<Guide>().step = 1;
        assert!(!completed(&app));
        let isolated = app
            .world_mut()
            .resource_mut::<Simulation>()
            .0
            .place_path(TilePos { x: 12, y: 12 }, 0)
            .unwrap();
        assert!(!completed(&app));
        {
            let mut sim = app.world_mut().resource_mut::<Simulation>();
            sim.0.remove_object(isolated).unwrap();
            sim.0.remove_object(old).unwrap();
            sim.0.place_path(TilePos { x: 4, y: 4 }, 0).unwrap();
        }
        assert_eq!(app.world().resource::<Simulation>().0.objects().len(), 2);
        assert!(completed(&app));
    }

    #[test]
    fn demo_access_can_use_an_existing_ride_and_labels_explain_navigation() {
        let mut app = guide_app(park_sim::demo_park::build_demo_park().unwrap());
        app.world_mut().resource_mut::<Guide>().step = 1;
        app.update();
        let body = app
            .world_mut()
            .query::<(&GuideText, &Text)>()
            .iter(app.world())
            .find(|(kind, _)| matches!(kind, GuideText::Body))
            .unwrap()
            .1
            .0
            .clone();
        assert!(body.contains("NEW public path tile"));
        assert!(body.contains("Next always continues"));
        app.world_mut().resource_mut::<Guide>().step = 3;
        let tool = button(&mut app, Control::Tool);
        let before = app
            .world()
            .resource::<Simulation>()
            .0
            .to_save_bytes()
            .unwrap();
        click(&mut app, tool);
        let view = app.world().resource::<ViewState>();
        assert!(view.rides_open);
        assert_eq!(view.ride_tab, crate::ui::rides::RideTab::Access);
        let ride = view.selected_ride.unwrap();
        assert_eq!(
            app.world().resource::<Simulation>().0.ride_model(ride),
            Some(RideModel::Carousel)
        );
        assert_eq!(
            app.world()
                .resource::<Simulation>()
                .0
                .to_save_bytes()
                .unwrap(),
            before
        );
    }

    #[test]
    fn foreground_panels_block_guide_navigation() {
        let mut app = guide_app(crate::new_game_park());
        let next = button(&mut app, Control::Next);
        app.world_mut().resource_mut::<ViewState>().help = true;
        click(&mut app, next);
        assert_eq!(app.world().resource::<Guide>().step, 0);
        app.world_mut().resource_mut::<ViewState>().help = false;
        click(&mut app, next);
        assert_eq!(app.world().resource::<Guide>().step, 1);
    }
}
