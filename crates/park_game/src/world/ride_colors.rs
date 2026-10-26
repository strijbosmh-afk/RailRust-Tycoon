//! Semantic paint shading. Geometry, picking faces and passenger colors stay intact.
use super::*;
use park_sim::RidePaint;
pub(super) fn inks(paint: RidePaint, classic: [Ink; 3]) -> [Ink; 3] {
    let Some(c) = paint.rgb() else {
        return classic;
    };
    [1.0, 0.72, 0.52].map(|shade| {
        rgb(
            (f32::from(c[0]) * shade) as u8,
            (f32::from(c[1]) * shade) as u8,
            (f32::from(c[2]) * shade) as u8,
        )
    })
}
pub(super) fn darken(inks: [Ink; 3], amount: f32) -> [Ink; 3] {
    inks.map(|mut i| {
        for c in &mut i[..3] {
            *c *= amount;
        }
        i
    })
}
pub(super) fn vehicle(paint: RidePaint, ink: Ink) -> Ink {
    for (group, amount) in [
        ([rgb(218, 67, 57), rgb(160, 40, 45), rgb(114, 32, 40)], 1.0),
        ([rgb(196, 66, 54), rgb(142, 43, 44), rgb(101, 35, 42)], 0.90),
    ] {
        if let Some(i) = group.iter().position(|c| *c == ink) {
            return if paint == RidePaint::Classic {
                ink
            } else {
                darken(inks(paint, group), amount)[i]
            };
        }
    }
    ink
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paint_changes_color_but_never_ride_geometry_picking_or_passengers() {
        let mut p = Park::new(32, 32);
        p.install_train_fixture().unwrap();
        let id = p.ride().unwrap().id;
        let carousel = p.place_carousel(TilePos { x: 19, y: 22 }, 8).unwrap();
        let original = p.clone();
        p.set_ride_appearance(park_sim::RideAppearance {
            ride: id,
            structure: RidePaint::Coral,
            supports: RidePaint::Sky,
            vehicles: RidePaint::Lagoon,
        })
        .unwrap();
        p.set_ride_appearance(park_sim::RideAppearance {
            ride: carousel,
            structure: RidePaint::Violet,
            supports: RidePaint::Sunflower,
            vehicles: RidePaint::Lagoon,
        })
        .unwrap();
        for r in 0..4 {
            for mode in [ViewMode::Surface, ViewMode::Underground] {
                let a = build_scene(&original, r, true, mode);
                let b = build_scene(&p, r, true, mode);
                assert_eq!(
                    a.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3(),
                    b.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3()
                );
                assert_ne!(
                    format!("{:?}", a.attribute(Mesh::ATTRIBUTE_COLOR)),
                    format!("{:?}", b.attribute(Mesh::ATTRIBUTE_COLOR))
                );
                for object in p.objects() {
                    let target = SurfaceId::Object(object.id);
                    if let Some(center) = surface_center(&p, target, r) {
                        assert_eq!(
                            pick_surface(center, &p, r, mode),
                            pick_surface(center, &original, r, mode)
                        );
                    }
                }
                let a = train_mesh(&original, r, mode);
                let b = train_mesh(&p, r, mode);
                assert_eq!(
                    a.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3(),
                    b.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3()
                );
            }
        }
    }
}
