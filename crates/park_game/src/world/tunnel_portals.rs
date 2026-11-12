//! Masonry entrances at connected paths that cross from open ground into a tunnel.
use super::*;
use park_sim::PathDirection;

fn entrances(
    park: &Park,
    object: &SurfaceObject,
    columns: &HashMap<TilePos, Vec<&SurfaceObject>>,
) -> Vec<(PathDirection, u8)> {
    if object.kind != ObjectKind::Path || !park.is_underground(object.id) {
        return Vec::new();
    }
    PathDirection::ALL
        .into_iter()
        .filter_map(|direction| {
            let height = object.path_edge_height(direction)?;
            let neighbor = direction.neighbor(object.pos)?;
            columns
                .get(&neighbor)?
                .iter()
                .any(|other| {
                    other.kind == ObjectKind::Path
                        && !park.is_underground(other.id)
                        && other.path_edge_height(direction.opposite()) == Some(height)
                        && (object.queue_owner.is_none()
                            || other.queue_owner.is_none()
                            || object.queue_owner == other.queue_owner)
                })
                .then_some((direction, height))
        })
        .collect()
}

pub(super) fn paint(
    mesh: &mut Painter,
    park: &Park,
    object: &SurfaceObject,
    rotation: u8,
    columns: &HashMap<TilePos, Vec<&SurfaceObject>>,
) {
    for (direction, height) in entrances(park, object, columns) {
        let (dx, dy) = direction.offset();
        let outward = Vec2::new(dx as f32, dy as f32);
        let across = Vec2::new(-outward.y, outward.x);
        let (x, y) = rotate(object.pos, park.width(), rotation);
        let point = |width: f32, rise: f32| {
            let offset = rotated_offset(outward * 0.508 + across * width, rotation);
            project(
                x as f32 + offset.x,
                y as f32 + offset.y,
                f32::from(height) + PATH_TOP_OFFSET + rise,
                park.width(),
            )
        };
        let face = |mesh: &mut Painter, left: f32, right: f32, bottom: f32, top: f32, ink| {
            mesh.surface(
                Surface {
                    vertices: [
                        point(left, bottom),
                        point(right, bottom),
                        point(right, top),
                        point(left, top),
                    ],
                    kind: SurfaceKind::LeftCliff,
                },
                ink,
            );
        };
        // The dark opening lies on the cliff plane. Its frame uses the same
        // projected world depths as the cliff, path and walking guests.
        face(mesh, -0.38, 0.38, 0.0, 1.65, rgb(26, 39, 45));
        face(mesh, -0.39, -0.29, 0.0, 1.72, rgb(190, 182, 152));
        face(mesh, 0.29, 0.39, 0.0, 1.72, rgb(157, 150, 128));
        face(mesh, -0.39, 0.39, 1.48, 1.76, rgb(212, 198, 158));
        for row in 1..5 {
            let y = row as f32 * 0.30;
            face(mesh, -0.39, -0.29, y, y + 0.022, rgb(99, 105, 93));
            face(mesh, 0.29, 0.39, y, y + 0.022, rgb(87, 96, 89));
        }
        for column in [-0.20, 0.0, 0.20] {
            face(mesh, column, column + 0.018, 1.48, 1.76, rgb(129, 126, 109));
        }
        face(mesh, -0.30, 0.30, 0.0, 0.045, rgb(225, 198, 134));
        face(
            mesh,
            -0.10,
            0.10,
            1.57,
            1.67,
            if object.queue_owner.is_some() {
                rgb(70, 173, 209)
            } else {
                rgb(244, 185, 62)
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portals_follow_connected_tunnel_mouths_and_all_world_rotations() {
        let mut park = Park::new(8, 8);
        let inside = TilePos { x: 3, y: 3 };
        park.set_terrain_height(inside, 3).unwrap();
        let tunnel = park.place_path(inside, 0).unwrap();
        let outside = park.place_path(TilePos { x: 3, y: 4 }, 0).unwrap();
        let columns =
            park.objects()
                .iter()
                .fold(HashMap::<_, Vec<_>>::new(), |mut columns, object| {
                    columns.entry(object.pos).or_default().push(object);
                    columns
                });
        assert_eq!(
            entrances(&park, park.object(tunnel).unwrap(), &columns),
            vec![(PathDirection::South, 0)]
        );
        let before = park.to_save_bytes().unwrap();
        for rotation in 0..4 {
            let mut painter = Painter::default();
            paint(
                &mut painter,
                &park,
                park.object(tunnel).unwrap(),
                rotation,
                &columns,
            );
            let mesh = painter.finish();
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("portal positions");
            };
            assert!(points.len() >= 60);
            assert!(points.iter().flatten().all(|value| value.is_finite()));
        }
        assert_eq!(park.to_save_bytes().unwrap(), before);
        drop(columns);
        park.remove_object(outside).unwrap();
        park.place_path(TilePos { x: 3, y: 4 }, 1).unwrap();
        let columns =
            park.objects()
                .iter()
                .fold(HashMap::<_, Vec<_>>::new(), |mut columns, object| {
                    columns.entry(object.pos).or_default().push(object);
                    columns
                });
        assert!(entrances(&park, park.object(tunnel).unwrap(), &columns).is_empty());
    }
}
