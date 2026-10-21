//! A small original gate whose real faces are shared by art, picking and ghosts.

use park_sim::{ObjectId, PathDirection};

use super::*;

pub(super) struct Face {
    pub(super) surface: Surface,
    pub(super) ink: Ink,
}

pub(super) fn facing(park: &Park, object: ObjectId) -> Option<PathDirection> {
    let ride = park.ride()?;
    ride.entrance
        .iter()
        .chain(ride.exit.iter())
        .find(|gate| gate.object == object)
        .map(|gate| gate.facing)
}

fn projected(
    object: &SurfaceObject,
    facing: PathDirection,
    point: Vec3,
    size: u32,
    rotation: u8,
) -> Vec3 {
    let (dx, dy) = facing.offset();
    let direction = Vec2::new(dx as f32, dy as f32);
    let side = Vec2::new(-direction.y, direction.x);
    let offset = rotated_offset(direction * point.x + side * point.y, rotation);
    let (u, v) = rotate(object.pos, size, rotation);
    project(
        u as f32 + offset.x,
        v as f32 + offset.y,
        f32::from(object.height) + point.z,
        size,
    )
}

/// Use the physical header roof for inspection/visible-selection actions.
/// Its deck may be hidden while the player can still see and select the gate.
pub(super) fn center(
    object: &SurfaceObject,
    facing: PathDirection,
    size: u32,
    rotation: u8,
) -> Vec2 {
    projected(object, facing, Vec3::new(0.0, 0.0, 1.83), size, rotation).truncate()
}

/// Platform, two narrow posts and a header all fit inside the reserved tile
/// and its two-step clearance. The open doorway is never a solid hit rectangle.
pub(super) fn faces(
    object: &SurfaceObject,
    facing: PathDirection,
    size: u32,
    rotation: u8,
) -> Vec<Face> {
    let exit = object.kind == ObjectKind::RideExit;
    let mut faces = Vec::with_capacity(23);
    let mut face = |points: [Vec3; 4], ink| {
        let surface = Surface {
            vertices: points.map(|point| projected(object, facing, point, size, rotation)),
            kind: SurfaceKind::Top,
        };
        if surface.has_area() {
            faces.push(Face { surface, ink });
        }
    };
    for (min, max, colors) in [
        (
            Vec3::new(-0.5, -0.34, 0.0),
            Vec3::new(0.5, 0.34, PATH_TOP_OFFSET),
            if exit {
                [rgb(98, 163, 142), rgb(61, 117, 99), rgb(43, 87, 74)]
            } else {
                [rgb(78, 145, 181), rgb(43, 103, 140), rgb(32, 76, 112)]
            },
        ),
        (
            Vec3::new(-0.07, -0.34, PATH_TOP_OFFSET),
            Vec3::new(0.07, -0.25, 1.68),
            [rgb(239, 226, 175), rgb(186, 163, 109), rgb(146, 120, 77)],
        ),
        (
            Vec3::new(-0.07, 0.25, PATH_TOP_OFFSET),
            Vec3::new(0.07, 0.34, 1.68),
            [rgb(239, 226, 175), rgb(186, 163, 109), rgb(146, 120, 77)],
        ),
        (
            Vec3::new(-0.085, -0.34, 1.50),
            Vec3::new(0.085, 0.34, 1.83),
            if exit {
                [rgb(101, 195, 160), rgb(53, 145, 116), rgb(33, 103, 85)]
            } else {
                [rgb(234, 186, 84), rgb(200, 146, 53), rgb(157, 108, 36)]
            },
        ),
    ] {
        let upper = [
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(max.x, min.y, max.z),
        ];
        let lower = upper.map(|point| Vec3::new(point.x, point.y, min.z));
        face(upper, colors[0]);
        for index in 0..4 {
            let next = (index + 1) % 4;
            face(
                [upper[index], upper[next], lower[next], lower[index]],
                colors[1 + index % 2],
            );
        }
    }
    // An original floor arrow points into the entrance / out of the exit.
    // It is geometry on
    // the deck, so foreground posts/terrain can occlude it normally.
    let h = PATH_TOP_OFFSET + 0.008;
    let ink = rgb(244, 232, 172);
    let arrow = |points: [Vec3; 4]| {
        points.map(|point| {
            if exit {
                Vec3::new(-point.x, -point.y, point.z)
            } else {
                point
            }
        })
    };
    face(
        arrow([
            Vec3::new(-0.37, -0.035, h),
            Vec3::new(-0.37, 0.035, h),
            Vec3::new(0.12, 0.035, h),
            Vec3::new(0.12, -0.035, h),
        ]),
        ink,
    );
    face(
        arrow([
            Vec3::new(0.08, -0.13, h),
            Vec3::new(0.08, 0.13, h),
            Vec3::new(0.30, 0.006, h),
            Vec3::new(0.30, -0.006, h),
        ]),
        ink,
    );
    faces
}

pub(super) fn preview(
    painter: &mut Painter,
    object: &SurfaceObject,
    facing: PathDirection,
    size: u32,
    rotation: u8,
    valid: bool,
    demolish: bool,
) {
    let mut ink = if !valid {
        rgb(225, 68, 63)
    } else if demolish {
        rgb(235, 120, 51)
    } else {
        rgb(47, 183, 114)
    };
    let faces = faces(object, facing, size, rotation);
    for face in &faces {
        let mut fill = face.surface;
        for vertex in &mut fill.vertices {
            vertex.z += SELECTION_BIAS;
        }
        ink[3] = 0.22;
        painter.surface(fill, ink);
        ink[3] = 0.95;
        painter.surface_outline(face.surface, 1.2, SELECTION_BIAS * 2.0, ink);
    }
    if (!valid || demolish)
        && let Some(platform) = faces.first()
    {
        let points = platform.surface.vertices.map(Vec3::truncate);
        for (from, to) in [(points[0], points[2]), (points[1], points[3])] {
            painter.surface_line(platform.surface, from, to, 1.5, SELECTION_BIAS * 3.0, ink);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::mesh::VertexAttributeValues;
    use park_sim::{RideId, TrackKind};

    use crate::path_tools::PathPreview;

    use super::*;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        let Some(VertexAttributeValues::Float32x3(values)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("positions");
        };
        values
    }

    fn fixture() -> (Park, RideId, ObjectId) {
        fixture_gate(false)
    }

    fn fixture_gate(exit: bool) -> (Park, RideId, ObjectId) {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let ride = park.ride().unwrap().id;
        let station = park
            .circuit()
            .unwrap()
            .pieces
            .iter()
            .find(|piece| piece.kind == TrackKind::Station)
            .unwrap()
            .object;
        let station = *park.object(station).unwrap();
        let pos = PathDirection::South.neighbor(station.pos).unwrap();
        let gate = if exit {
            park.place_ride_exit(pos, station.height, PathDirection::North, ride)
        } else {
            park.place_ride_entrance(pos, station.height, PathDirection::North, ride)
        }
        .unwrap();
        // Isolate the gate's hit geometry from unrelated hills in the circuit.
        // A disconnected station leaves its entrance visible for repair.
        let track: Vec<_> = park
            .objects()
            .iter()
            .filter(|object| object.kind == ObjectKind::Track)
            .map(|object| object.id)
            .collect();
        for id in track {
            park.remove_object(id).unwrap();
        }
        (park, ride, gate)
    }

    #[test]
    fn exits_share_physical_pick_and_preview_faces_with_reversed_arrow_and_distinct_header() {
        let (park, _, id) = fixture_gate(true);
        let object = park.object(id).unwrap();
        let facing = facing(&park, id).unwrap();
        let before = park.clone();
        for rotation in 0..4 {
            let geometry = faces(object, facing, park.width(), rotation);
            let mut entrance = *object;
            entrance.kind = ObjectKind::RideEntrance;
            let original = faces(&entrance, facing, park.width(), rotation);
            assert_eq!(geometry.len(), original.len());
            // Changing gate purpose preserves structural geometry. Only the
            // directed floor arrow reverses and platform/header colors differ.
            for (exit, entry) in geometry[..geometry.len() - 2].iter().zip(&original) {
                assert_eq!(exit.surface.vertices, entry.surface.vertices);
            }
            assert_ne!(geometry[0].ink, original[0].ink);
            assert_ne!(
                geometry.last().unwrap().surface.vertices,
                original.last().unwrap().surface.vertices
            );
            assert_eq!(
                rendered_object_surfaces(&park, object, rotation).len(),
                geometry.len()
            );
            let center = surface_center(&park, SurfaceId::Object(id), rotation).unwrap();
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                assert_eq!(
                    pick_surface(center, &park, rotation, mode),
                    Some(SurfaceId::Object(id))
                );
                assert!(
                    !positions(&selection_mesh(
                        None,
                        Some(SurfaceId::Object(id)),
                        &park,
                        rotation,
                        mode
                    ))
                    .is_empty()
                );
            }
            for (valid, demolish) in [(true, false), (false, false), (true, true)] {
                let preview = path_preview_mesh(
                    Some(PathPreview {
                        pos: object.pos,
                        height: object.height,
                        slope: PathSlope::Flat,
                        queue_owner: None,
                        entrance_facing: Some(facing),
                        exit_gate: true,
                        valid,
                        demolish,
                    }),
                    &park,
                    rotation,
                );
                assert!(!positions(&preview).is_empty());
                assert!(
                    positions(&preview)
                        .iter()
                        .flatten()
                        .all(|value| value.is_finite())
                );
                let center_z = geometry
                    .iter()
                    .filter_map(|face| face.surface.hit_depth(center))
                    .max_by(f32::total_cmp)
                    .unwrap();
                assert!(positions(&preview).iter().any(|point| point[2] > center_z));
            }
        }
        assert_eq!(park, before);
    }

    #[test]
    fn gates_use_real_faces_with_an_open_doorway_inside_the_reserved_volume() {
        for facing in PathDirection::ALL {
            let object = SurfaceObject {
                id: ObjectId(7),
                pos: TilePos { x: 15, y: 15 },
                kind: ObjectKind::RideEntrance,
                axis: Axis::Y,
                path_slope: PathSlope::Flat,
                queue_owner: None,
                height: 8,
                clearance: 2,
            };
            for rotation in 0..4 {
                let geometry = faces(&object, facing, 32, rotation);
                assert!(geometry.len() >= 18);
                for vertex in geometry.iter().flat_map(|face| face.surface.vertices) {
                    assert!(vertex.is_finite());
                    let sum = vertex.z - BASE_DEPTH;
                    let height = (vertex.y + (sum - 31.0) * HALF_HEIGHT) / HEIGHT_STEP;
                    assert!((8.0 - 0.0001..=9.83 + 0.0001).contains(&height));
                    let difference = vertex.x / HALF_WIDTH;
                    let (u, v) = rotate(object.pos, 32, rotation);
                    let offset = Vec2::new(
                        (sum + difference) * 0.5 - u as f32,
                        (sum - difference) * 0.5 - v as f32,
                    );
                    assert!(offset.abs().max_element() <= 0.5001);
                }
                let doorway = projected(&object, facing, Vec3::new(0.0, 0.0, 1.25), 32, rotation);
                assert!(
                    geometry
                        .iter()
                        .all(|face| face.surface.hit_depth(doorway.truncate()).is_none()),
                    "an open doorway must not become a clearance-box hit target"
                );
            }
        }
    }

    #[test]
    fn entrance_paint_pick_selection_and_preview_share_faces_in_every_rotation_and_view() {
        let (park, _, gate) = fixture();
        let before = park.clone();
        let object = park.object(gate).unwrap();
        let facing = facing(&park, gate).unwrap();
        for rotation in 0..4 {
            let geometry = faces(object, facing, park.width(), rotation);
            let rendered = rendered_object_surfaces(&park, object, rotation);
            assert_eq!(rendered.len(), geometry.len());
            for (actual, expected) in rendered.iter().zip(&geometry) {
                assert_eq!(actual.vertices, expected.surface.vertices);
            }
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                let selectable =
                    selectable_surfaces(&park, SurfaceId::Object(gate), rotation, mode);
                assert_eq!(selectable.len(), geometry.len());
                let outline =
                    selection_mesh(None, Some(SurfaceId::Object(gate)), &park, rotation, mode);
                assert!(!positions(&outline).is_empty());
                // A header top is unambiguously visible above this fixture's floor.
                let header = projected(
                    object,
                    facing,
                    Vec3::new(0.0, 0.0, 1.83),
                    park.width(),
                    rotation,
                );
                assert_eq!(
                    pick_surface(header.truncate(), &park, rotation, mode),
                    Some(SurfaceId::Object(gate))
                );
                assert!(
                    selectable
                        .iter()
                        .filter_map(|face| face.hit_depth(header.truncate()))
                        .any(|depth| (depth - header.z).abs() < 0.0002)
                );
            }
            for (valid, demolish) in [(true, false), (false, false), (true, true)] {
                let mesh = path_preview_mesh(
                    Some(PathPreview {
                        pos: object.pos,
                        height: object.height,
                        slope: PathSlope::Flat,
                        queue_owner: None,
                        entrance_facing: Some(facing),
                        exit_gate: false,
                        valid,
                        demolish,
                    }),
                    &park,
                    rotation,
                );
                for &point in positions(&mesh) {
                    let point = Vec3::from_array(point);
                    assert!(geometry.iter().any(|face| {
                        face.surface
                            .hit_depth(point.truncate())
                            .is_some_and(|depth| {
                                (SELECTION_BIAS - 0.0002..=SELECTION_BIAS * 3.0 + 0.0002)
                                    .contains(&(point.z - depth))
                            })
                    }));
                }
                let Some(VertexAttributeValues::Float32x4(colors)) =
                    mesh.attribute(Mesh::ATTRIBUTE_COLOR)
                else {
                    panic!("colors")
                };
                assert!(colors.iter().all(|color| (0.0..1.0).contains(&color[3])));
            }
        }
        assert_eq!(park, before);
    }

    #[test]
    fn nearer_decks_hide_gate_faces_and_removal_clears_pick_and_selection() {
        let (mut park, _, gate) = fixture();
        let object = *park.object(gate).unwrap();
        let direction = facing(&park, gate).unwrap();
        // Stacking a complete path above the gate is legal touching clearance.
        let cover = park.place_path(object.pos, object.height + 2).unwrap();
        for rotation in 0..4 {
            let point = projected(
                &object,
                direction,
                Vec3::new(0.0, 0.0, 1.83),
                park.width(),
                rotation,
            )
            .truncate();
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                assert_eq!(
                    pick_surface(point, &park, rotation, mode),
                    Some(SurfaceId::Object(cover))
                );
                let gate_depth = faces(&object, direction, park.width(), rotation)
                    .iter()
                    .filter_map(|face| face.surface.hit_depth(point))
                    .max_by(f32::total_cmp)
                    .unwrap();
                let cover_depth = object_surfaces(&park, park.object(cover).unwrap(), rotation)[0]
                    .hit_depth(point)
                    .unwrap();
                assert!(cover_depth > gate_depth + SELECTION_BIAS * 3.0);
            }
        }
        park.remove_object(gate).unwrap();
        for rotation in 0..4 {
            assert!(surface_center(&park, SurfaceId::Object(gate), rotation).is_none());
            assert_eq!(
                selection_mesh(
                    None,
                    Some(SurfaceId::Object(gate)),
                    &park,
                    rotation,
                    ViewMode::Surface
                )
                .count_vertices(),
                0
            );
        }
    }

    #[test]
    fn visible_gate_action_center_stays_on_header_when_its_deck_is_hidden() {
        for rotation in 0..4 {
            let (mut park, _, gate) = fixture();
            let object = *park.object(gate).unwrap();
            let direction = facing(&park, gate).unwrap();
            let (u, v) = rotate(object.pos, park.width(), rotation);
            let foreground = unrotate(u + 1, v + 1, park.width(), rotation);
            let cover = park.place_path(foreground, object.height + 2).unwrap();
            let deck = projected(
                &object,
                direction,
                Vec3::new(0.0, 0.0, PATH_TOP_OFFSET),
                park.width(),
                rotation,
            )
            .truncate();
            let header = surface_center(&park, SurfaceId::Object(gate), rotation).unwrap();
            assert_eq!(header, center(&object, direction, park.width(), rotation));
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                assert_eq!(
                    pick_surface(deck, &park, rotation, mode),
                    Some(SurfaceId::Object(cover))
                );
                assert_eq!(
                    pick_surface(header, &park, rotation, mode),
                    Some(SurfaceId::Object(gate)),
                    "visible-selection actions must use the selectable header, not the obscured floor"
                );
            }
        }
    }
}
