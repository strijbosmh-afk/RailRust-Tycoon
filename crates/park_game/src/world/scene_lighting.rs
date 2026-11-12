use super::*;
use crate::display::{Lighting, Weather};

pub(crate) fn ambience_mesh(
    park: &Park,
    rotation: u8,
    mode: ViewMode,
    lighting: Lighting,
    weather: Weather,
    phase: u8,
) -> Mesh {
    let mut painter = Painter::default();
    if mode == ViewMode::Underground {
        return painter.finish();
    }
    for object in park
        .objects()
        .iter()
        .filter(|object| {
            object_visible(park, object, mode)
                && (object.kind == ObjectKind::Service
                    || object.kind == ObjectKind::RideEntrance
                    || (object.is_public_path() && object.id.0.is_multiple_of(6)))
        })
        .take(128)
    {
        let offset = Vec2::new(0.30, -0.26);
        let rotated = rotated_offset(offset, rotation);
        let (x, y) = rotate(object.pos, park.width(), rotation);
        let height = if object.kind == ObjectKind::Path {
            object_base_at(object, offset) + PATH_TOP_OFFSET
        } else {
            f32::from(object.height)
        };
        let foot = project(
            x as f32 + rotated.x,
            y as f32 + rotated.y,
            height,
            park.width(),
        );
        let head = project(
            x as f32 + rotated.x,
            y as f32 + rotated.y,
            height + 1.0,
            park.width(),
        );
        painter.line(
            foot.truncate(),
            head.truncate(),
            1.8,
            foot.z + DETAIL_BIAS,
            rgb(47, 69, 64),
        );
        painter.ellipse(
            foot.truncate(),
            3.2,
            1.4,
            foot.z + DETAIL_BIAS * 2.0,
            rgb(82, 97, 77),
        );
        if lighting != Lighting::Day {
            painter.ellipse(
                head.truncate(),
                7.5,
                6.5,
                head.z + DETAIL_BIAS,
                [1.0, 0.65, 0.14, 0.18],
            );
        }
        painter.ellipse(
            head.truncate(),
            2.8,
            3.0,
            head.z + DETAIL_BIAS * 2.0,
            if lighting == Lighting::Day {
                rgb(235, 229, 183)
            } else {
                rgb(255, 220, 109)
            },
        );
        painter.line(
            head.truncate() + Vec2::new(-3.8, 3.3),
            head.truncate() + Vec2::new(3.8, 3.3),
            1.6,
            head.z + DETAIL_BIAS * 3.0,
            rgb(41, 63, 59),
        );
    }
    if weather == Weather::Rain {
        for index in 0..160_u32 {
            let seed = index.wrapping_mul(2_654_435_761);
            let x = (seed % (park.width() * 16)) as f32 / 16.0;
            let y = ((seed >> 8) % (park.height() * 16)) as f32 / 16.0;
            let pos = TilePos {
                x: (x as u32).min(park.width() - 1),
                y: (y as u32).min(park.height() - 1),
            };
            let (u, v) = rotate(pos, park.width(), rotation);
            let height = f32::from(park.tile(pos).unwrap().height)
                + 1.0
                + ((index + u32::from(phase)) % 16) as f32 / 3.0;
            let point = project(u as f32, v as f32, height, park.width());
            painter.line(
                point.truncate(),
                point.truncate() + Vec2::new(-3.0, -8.0),
                0.8,
                point.z + DETAIL_BIAS,
                [0.63, 0.82, 0.97, 0.46],
            );
        }
    }
    painter.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lighting_and_rain_geometry_are_bounded_and_preserve_the_park() {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        let before = park.to_save_bytes().unwrap();
        for rotation in 0..4 {
            for lighting in [Lighting::Day, Lighting::Sunset, Lighting::Night] {
                let mesh = ambience_mesh(
                    &park,
                    rotation,
                    ViewMode::Surface,
                    lighting,
                    Weather::Rain,
                    7,
                );
                let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    panic!("ambience positions");
                };
                assert!(!points.is_empty());
                assert!(points.len() < 40_000);
                assert!(points.iter().flatten().all(|value| value.is_finite()));
            }
        }
        assert_eq!(before, park.to_save_bytes().unwrap());
    }
}
