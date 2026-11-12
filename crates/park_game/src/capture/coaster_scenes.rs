//! Multiple independent coaster rendering through production construction.
use super::*;

pub(crate) fn prepare_completion(
    park: &mut Park,
    view: &mut ViewState,
    detail: bool,
    controls: bool,
) {
    use park_sim::{PathDirection, PathSlope, TrackKind};
    let mut source = Park::new(32, 32);
    source.install_train_fixture().unwrap();
    let mut specs = source.circuit().unwrap().specs().to_vec();
    for p in &mut specs {
        p.entry.height -= 8;
        p.exit.height -= 8;
        match (p.pos.x, p.pos.y) {
            (9, 10) => p.kind = TrackKind::TransitionUp,
            (17, 10) => p.kind = TrackKind::Launch,
            (20, 10) => p.kind = TrackKind::HeartlineRoll,
            (21, 10) | (16, 4) => p.kind = TrackKind::Brake,
            (24, 10) => p.kind = TrackKind::BankedCurve,
            (19, 4) => {
                p.kind = TrackKind::TransitionUp;
                p.exit.height = 1;
            }
            (18, 4) => {
                p.kind = TrackKind::TransitionDown;
                p.entry.height = 1;
            }
            _ => {}
        }
    }
    *park = Park::new(64, 64);
    park.borrow_loan(100_000).unwrap();
    let mut ids = Vec::new();
    for offset in [0, 26] {
        let shifted: Vec<_> = specs
            .iter()
            .map(|p| {
                let mut p = *p;
                p.pos.x += offset;
                p
            })
            .collect();
        park.replace_track_draft(&shifted).unwrap();
        park.apply_track_draft().unwrap();
        let id = park.ride().unwrap().id;
        ids.push(id);
        park.set_ride_name(
            id,
            if offset == 0 {
                "Aurora Roll"
            } else {
                "Seaside Flyer"
            },
        )
        .unwrap();
        if offset == 0 {
            park.set_coaster_train_count(id, 3).unwrap();
        } else {
            park.apply_ride_palette_preset(id, park_sim::RidePalettePreset::Seaside)
                .unwrap();
        }
        park.place_ride_entrance(
            TilePos {
                x: 8 + offset,
                y: 11,
            },
            0,
            PathDirection::North,
            id,
        )
        .unwrap();
        for y in 12..=14 {
            park.place_queue_path(TilePos { x: 8 + offset, y }, 0, PathSlope::Flat, id)
                .unwrap();
        }
        park.place_ride_exit(
            TilePos {
                x: 8 + offset,
                y: 9,
            },
            0,
            PathDirection::South,
            id,
        )
        .unwrap();
        for x in 8..=13 {
            park.place_path(
                TilePos {
                    x: x + offset,
                    y: 8,
                },
                0,
            )
            .unwrap();
        }
        for y in 9..=14 {
            park.place_path(TilePos { x: 13 + offset, y }, 0).unwrap();
        }
        park.set_ride_ticket_price(id, 100).unwrap();
        if offset == 0 {
            park.new_coaster_draft().unwrap();
        }
    }
    let mut entry = None;
    for x in 0..=42 {
        entry.get_or_insert(park.place_path(TilePos { x, y: 15 }, 0).unwrap());
    }
    park.set_visitor_entry(entry.unwrap()).unwrap();
    park.set_visitor_arrivals(true).unwrap();
    for id in &ids {
        park.set_ride_open_checked(*id, true).unwrap();
    }
    for _ in 0..12000 {
        park.advance(Speed::Normal);
    }
    let first = ids[0];
    let mut inverted = None;
    for _ in 0..6000 {
        inverted = park
            .coaster_trains(first)
            .iter()
            .find(|t| {
                (165_000..195_000)
                    .contains(&park.coaster_circuit(first).unwrap().roll_at(t.distance))
            })
            .map(|t| t.number);
        if inverted.is_some() {
            break;
        }
        park.advance(Speed::Normal);
    }
    assert!(inverted.is_some());
    view.selected_ride = Some(first);
    view.rides_open = controls;
    view.speed = Speed::Paused;
    view.zoom = if detail { 3.2 } else { 1.5 };
    view.pan = if detail {
        world::leading_train_focus(park, first, inverted.unwrap(), view.rotation).unwrap()
    } else {
        world::tile_center(TilePos { x: 27, y: 10 }, 64, view.rotation)
            + Vec2::Y * 3.0 * world::HEIGHT_STEP
    };
    view.scene_dirty = true;
    assert!(
        ids.iter()
            .all(|id| park.ride_by_id(*id).unwrap().completed > 0)
    );
    assert_eq!(
        *park,
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap()
    );
}

pub(crate) fn prepare(park: &mut Park, view: &mut ViewState) {
    let mut source = Park::new(32, 32);
    source.install_train_fixture().unwrap();
    let specs = source.circuit().unwrap().specs().to_vec();
    *park = Park::new(64, 64);
    park.borrow_loan(100_000).unwrap();
    park.replace_track_draft(&specs).unwrap();
    park.apply_track_draft().unwrap();
    let first = park.ride().unwrap().id;
    park.set_ride_name(first, "Golden Comet").unwrap();
    park.new_coaster_draft().unwrap();
    let shifted: Vec<_> = specs
        .into_iter()
        .map(|mut spec| {
            spec.pos.x += 26;
            spec
        })
        .collect();
    park.replace_track_draft(&shifted).unwrap();
    park.apply_track_draft().unwrap();
    let second = park.ride().unwrap().id;
    park.set_ride_name(second, "Seaside Flyer").unwrap();
    park.apply_ride_palette_preset(second, park_sim::RidePalettePreset::Seaside)
        .unwrap();
    park.set_ride_open(first, true).unwrap();
    for _ in 0..240 {
        park.advance(Speed::Normal);
    }
    park.set_ride_open(second, true).unwrap();
    for _ in 0..300 {
        park.advance(Speed::Normal);
    }
    view.selected_ride = Some(second);
    view.speed = Speed::Paused;
    view.zoom = 1.3;
    view.pan = world::tile_center(TilePos { x: 28, y: 9 }, 64, view.rotation)
        + Vec2::Y * 8.0 * world::HEIGHT_STEP
        - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
    view.scene_dirty = true;
    assert!(park.coaster_train(first).unwrap().speed > 0);
    assert!(park.coaster_train(second).unwrap().speed > 0);
    assert_eq!(
        *park,
        Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_from_a_modal_selects_the_ride_before_the_editor_is_shown() {
        let mut park = Park::new(64, 64);
        let mut view = ViewState::default();
        prepare(&mut park, &mut view);
        let first = park.coaster_ids()[0];
        park.new_coaster_draft().unwrap();
        park.set_track_draft_piece(park_sim::TrackBuildKind::Station.spec(
            TilePos { x: 10, y: 30 },
            0,
            park_sim::TrackDirection::East,
        ))
        .unwrap();
        view.selected_ride = Some(first);
        view.rides_open = true;
        let mut app = App::new();
        app.insert_resource(Simulation(park))
            .insert_resource(view)
            .insert_resource(ButtonInput::<KeyCode>::default())
            .init_resource::<Time>()
            .add_systems(
                Update,
                (
                    crate::ui::button_actions,
                    crate::keyboard_input,
                    crate::track_tools::apply_commands,
                )
                    .chain(),
            );
        app.world_mut()
            .spawn((Button, Interaction::Pressed, crate::ui::Action::EditTrack));
        app.update();
        let park = &app.world().resource::<Simulation>().0;
        assert_eq!(park.ride().unwrap().id, first);
        assert_eq!(park.track_draft().len(), 46);
        assert!(app.world().resource::<ViewState>().track_editor.active);
        let mut restored = park.clone();
        restored.new_coaster_draft().unwrap();
        assert_eq!(restored.track_draft().len(), 1);
    }

    #[test]
    fn independent_coasters_render_and_select_through_the_editor_in_every_rotation() {
        for rotation in 0..4 {
            let mut park = Park::new(32, 32);
            let mut view = ViewState {
                rotation,
                ..Default::default()
            };
            prepare(&mut park, &mut view);
            let ids = park.coaster_ids();
            for id in &ids {
                let object = park.coaster_circuit(*id).unwrap().pieces[0].object;
                assert_eq!(park.object_ride(object), Some(*id));
                assert!(
                    world::surface_center(&park, park_sim::SurfaceId::Object(object), rotation)
                        .is_some()
                );
            }
            let mesh = world::train_mesh(&park, rotation, view.view_mode);
            assert!(mesh.count_vertices() > 100);
            view.track_editor.active = true;
            view.selected_ride = Some(ids[0]);
            crate::track_tools::begin(&mut park, &mut view);
            assert_eq!(view.selected_ride, Some(ids[0]));
            crate::track_tools::apply_command(
                &mut park,
                &mut view,
                crate::track_tools::TrackCommand::NewCoaster,
            );
            assert!(park.ride().is_none());
            assert!(park.coaster_circuit(ids[0]).is_some());
            assert!(park.coaster_circuit(ids[1]).is_some());
            crate::track_tools::apply_command(
                &mut park,
                &mut view,
                crate::track_tools::TrackCommand::NextCoaster,
            );
            assert_eq!(view.selected_ride, Some(ids[0]));
        }
    }
}
