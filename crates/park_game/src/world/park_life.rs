use super::*;

pub(super) fn paint(mesh: &mut Painter, park: &Park, rotation: u8, mode: ViewMode) {
    for id in park.path_shelters() {
        let Some(path) = park.object(*id).filter(|p| object_visible(park, p, mode)) else {
            continue;
        };
        let mut roof = object_surfaces(park, path, rotation)[0];
        let shift = Vec3::Y * 22.0 + Vec3::Z * 0.02;
        for vertex in &mut roof.vertices {
            let base = *vertex;
            *vertex += shift;
            mesh.line(
                base.truncate(),
                vertex.truncate(),
                1.5,
                vertex.z,
                rgb(42, 87, 91),
            );
        }
        let color = if park.path_is_sheltered(*id) {
            rgb(37, 174, 202)
        } else {
            rgb(122, 106, 96)
        };
        mesh.surface(roof, color);
        let [a, b, c, d] = roof.vertices;
        for t in [0.2, 0.5, 0.8] {
            let from = a.lerp(b, t);
            let to = d.lerp(c, t);
            mesh.line(
                from.truncate(),
                to.truncate(),
                3.0,
                from.z.max(to.z) + 0.001,
                rgb(255, 219, 95),
            );
        }
    }
    for worker in park.park_staff() {
        let Some(path) = worker
            .current
            .and_then(|id| park.object(id))
            .filter(|p| object_visible(park, p, mode))
        else {
            continue;
        };
        let Some(point) = park.staff_position(worker.id) else {
            continue;
        };
        let base = project_path_point(point, park.width(), rotation);
        let p = base.truncate();
        let z = base.z + 0.045;
        let color = match worker.role {
            park_sim::StaffRole::Mechanic => rgb(32, 170, 191),
            park_sim::StaffRole::Handyman => rgb(246, 198, 42),
            park_sim::StaffRole::Security => rgb(62, 84, 191),
            park_sim::StaffRole::Entertainer => rgb(218, 56, 145),
            park_sim::StaffRole::Attendant => rgb(245, 245, 234),
        };
        mesh.surface_ellipse(
            object_surfaces(park, path, rotation)[0],
            p,
            5.0,
            2.5,
            DETAIL_BIAS * 3.0,
            rgb(50, 85, 65),
        );
        let stride = if worker.next.is_some() {
            ((worker.progress as f32 / 30.0) * std::f32::consts::TAU).sin() * 2.0
        } else {
            0.0
        };
        for side in [-1.0, 1.0] {
            mesh.line(
                p + Vec2::new(side * 1.8, 8.0),
                p + Vec2::new(side * (1.8 + stride), 0.5),
                2.2,
                z,
                rgb(35, 48, 67),
            );
            mesh.line(
                p + Vec2::new(side * 3.4, 16.0),
                p + Vec2::new(side * 5.0, 9.0 + stride),
                1.8,
                z + 0.001,
                rgb(236, 183, 134),
            );
        }
        mesh.quad(
            [
                p + Vec2::new(-3.8, 7.0),
                p + Vec2::new(3.8, 7.0),
                p + Vec2::new(3.2, 17.0),
                p + Vec2::new(-3.2, 17.0),
            ],
            z + 0.002,
            color,
        );
        mesh.ellipse(
            p + Vec2::new(0.0, 20.0),
            3.2,
            3.5,
            z + 0.004,
            rgb(236, 183, 134),
        );
        mesh.ellipse(p + Vec2::new(0.0, 23.0), 4.0, 1.5, z + 0.005, color);
        mesh.line(
            p + Vec2::new(-1.3, 20.5),
            p + Vec2::new(1.3, 20.5),
            0.7,
            z + 0.006,
            rgb(53, 55, 66),
        );
        match worker.role {
            park_sim::StaffRole::Entertainer => {
                mesh.line(
                    p + Vec2::new(4.0, 12.0),
                    p + Vec2::new(8.0, 35.0),
                    0.6,
                    z + 0.007,
                    rgb(245, 244, 224),
                );
                mesh.ellipse(
                    p + Vec2::new(8.0, 36.0),
                    5.0,
                    6.0,
                    z + 0.008,
                    rgb(244, 69, 72),
                );
            }
            park_sim::StaffRole::Handyman => mesh.line(
                p + Vec2::new(5.0, 16.0),
                p + Vec2::new(10.0, -1.0),
                2.0,
                z + 0.007,
                rgb(143, 87, 41),
            ),
            park_sim::StaffRole::Attendant => {
                mesh.line(
                    p + Vec2::new(0.0, 10.0),
                    p + Vec2::new(0.0, 15.0),
                    1.5,
                    z + 0.007,
                    rgb(225, 63, 58),
                );
                mesh.line(
                    p + Vec2::new(-2.2, 12.5),
                    p + Vec2::new(2.2, 12.5),
                    1.5,
                    z + 0.008,
                    rgb(225, 63, 58),
                );
            }
            _ => mesh.ellipse(
                p + Vec2::new(-1.0, 14.0),
                1.3,
                1.5,
                z + 0.007,
                rgb(255, 211, 72),
            ),
        }
    }
    for incident in park.park_incidents() {
        let Some(path) = park
            .object(incident.path)
            .filter(|p| object_visible(park, p, mode))
        else {
            continue;
        };
        let Some(point) = path.path_center() else {
            continue;
        };
        let base = project_path_point(point, park.width(), rotation);
        mesh.line(
            base.truncate() + Vec2::new(0.0, 8.0),
            base.truncate() + Vec2::new(0.0, 15.0),
            2.0,
            base.z + 0.05,
            rgb(238, 100, 40),
        );
        mesh.ellipse(
            base.truncate() + Vec2::new(0.0, 5.0),
            1.3,
            1.3,
            base.z + 0.05,
            rgb(238, 100, 40),
        );
    }
    if matches!(
        park.park_life_status().weather,
        park_sim::Weather::Rain | park_sim::Weather::Storm
    ) {
        for (index, path) in park
            .objects()
            .iter()
            .filter(|p| p.is_public_path() && object_visible(park, p, mode))
            .step_by(3)
            .take(64)
            .enumerate()
        {
            let Some(point) = path.path_center() else {
                continue;
            };
            let base = project_path_point(point, park.width(), rotation);
            let height = ((park.ticks() + index as u64 * 7) % 35) as f32;
            let a = base.truncate() + Vec2::new(-height * 0.3, height + 8.0);
            mesh.line(
                a,
                a + Vec2::new(-2.0, -6.0),
                0.9,
                base.z + 0.05,
                rgb(155, 208, 232),
            );
        }
    }
}
