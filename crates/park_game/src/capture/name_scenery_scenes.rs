//! In-engine captures for saved ride names and planted scenery.
use super::*;
use park_sim::SceneryKind;

pub(super) fn prepare(park: &mut Park, view: &mut ViewState, scene: Scene) {
    *park = crate::save_check::names::install().expect("name capture fixture");
    view.help = false;
    view.guests_open = false;
    view.finance_open = false;
    view.services_open = false;
    view.designs_open = false;
    view.speed = Speed::Paused;
    view.path_tool = ToolState::default();
    view.scene_dirty = true;
    match scene {
        Scene::NamesCoaster | Scene::NamesCarousel => {
            let coaster = park.ride().expect("coaster").id;
            let carousel = park
                .carousels()
                .iter()
                .find(|ride| !ride.objects.is_empty())
                .expect("carousel")
                .ride
                .id;
            let id = if matches!(scene, Scene::NamesCarousel) {
                carousel
            } else {
                coaster
            };
            view.rides_open = true;
            view.selected_ride = Some(id);
            view.ride_tab = crate::ui::rides::RideTab::Operations;
            view.ride_name_owner = Some(id);
            view.ride_name_draft = park.ride_name(id).expect("saved ride name").into();
            view.ride_name_focused = false;
        }
        Scene::Scenery => {
            view.rides_open = false;
            let clear = |park: &Park, kind| {
                (22..29)
                    .flat_map(|y| (12..22).map(move |x| TilePos { x, y }))
                    .find(|pos| park.validate_scenery(*pos, kind).is_ok())
                    .expect("clear capture land")
            };
            let tree = clear(park, SceneryKind::Tree);
            let tree_id = park
                .build_scenery(tree, SceneryKind::Tree)
                .expect("tree capture");
            let bench_pos = (tree.x.saturating_sub(2)..=tree.x + 2)
                .flat_map(|x| {
                    (tree.y.saturating_sub(2)..=tree.y + 2).map(move |y| TilePos { x, y })
                })
                .find(|pos| {
                    park.tile(*pos)
                        .is_some_and(|tile| park.validate_path(*pos, tile.height).is_ok())
                })
                .expect("clear capture bench path");
            let bench_height = park.tile(bench_pos).unwrap().height;
            let bench_path = park
                .place_path(bench_pos, bench_height)
                .expect("bench path capture");
            park.toggle_path_bench(bench_path).expect("bench capture");
            park.toggle_path_addition(bench_path, park_sim::PathAdditionKind::Sign)
                .expect("sign capture");
            park.toggle_path_addition(bench_path, park_sim::PathAdditionKind::Fence)
                .expect("fence capture");
            let flowers = clear(park, SceneryKind::FlowerBed);
            park.build_scenery(flowers, SceneryKind::FlowerBed)
                .expect("flowers capture");
            view.path_tool
                .set_mode(PathTool::Scenery(SceneryKind::FlowerBed));
            view.selected = Some(SurfaceId::Object(tree_id));
            view.zoom = 2.2;
            view.pan = world::elevated_tile_center(park, tree, view.rotation)
                .expect("tree position")
                - Vec2::Y * f32::from(park.max_surface_height()) * world::HEIGHT_STEP * 0.5;
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_and_scenery_capture_fixtures_are_valid() {
        for scene in [Scene::NamesCoaster, Scene::NamesCarousel, Scene::Scenery] {
            let mut park = crate::demonstration_park();
            let mut view = ViewState::default();
            prepare(&mut park, &mut view, scene);
            assert_eq!(
                Park::from_save_bytes(&park.to_save_bytes().unwrap()).unwrap(),
                park
            );
            assert_eq!(view.speed, Speed::Paused);
        }
    }
}
