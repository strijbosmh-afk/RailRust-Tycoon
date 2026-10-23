//! Draft rails follow the same sampled centerline as their committed geometry.
use super::*;
use crate::track_tools::TrackEditor;
use park_sim::{TRACK_UNITS, TrackKind, TrackPieceSpec, TrackPoint};

pub fn pick_track_plane(point: Vec2, park: &Park, rotation: u8, height: u8) -> Option<TilePos> {
    path_preview::pick_path_plane(
        point + Vec2::Y * (PATH_TOP_OFFSET - 0.3) * HEIGHT_STEP,
        park,
        rotation,
        height,
    )
}

fn world_point(point: TrackPoint) -> Vec3 {
    Vec3::new(point.x as f32, point.y as f32, point.height as f32) / TRACK_UNITS as f32
}

fn projected(point: Vec3, park: &Park, rotation: u8) -> Vec3 {
    let last = park.width().saturating_sub(1) as f32;
    let (u, v) = match rotation % 4 {
        0 => (point.x, point.y),
        1 => (last - point.y, point.x),
        2 => (last - point.x, last - point.y),
        _ => (point.y, last - point.x),
    };
    project(u, v, point.z + 0.3, park.width()) + Vec3::Z * SELECTION_BIAS * 4.0
}

struct DraftGeometry {
    points: Vec<Vec3>,
    normals: Vec<Vec3>,
    half: f32,
}

impl DraftGeometry {
    fn new(spec: TrackPieceSpec) -> Self {
        let points: Vec<_> = spec.samples().into_iter().map(world_point).collect();
        let normals = (0..points.len())
            .map(|index| {
                let a = points[index.saturating_sub(1)];
                let b = points[(index + 1).min(points.len() - 1)];
                let delta = (b - a).truncate().normalize_or_zero();
                Vec3::new(-delta.y, delta.x, 0.0)
            })
            .collect();
        Self {
            points,
            normals,
            half: if spec.kind == TrackKind::Station {
                0.42
            } else {
                0.26
            },
        }
    }

    fn strip(&self, park: &Park, rotation: u8, index: usize, left: f32, right: f32) -> Surface {
        let a = self.points[index];
        let b = self.points[index + 1];
        let na = self.normals[index];
        let nb = self.normals[index + 1];
        Surface {
            vertices: [a + na * left, b + nb * left, b + nb * right, a + na * right]
                .map(|point| projected(point, park, rotation)),
            kind: SurfaceKind::Top,
        }
    }
}

/// Erase follows the actual visible draft ribbons at their own heights. The
/// build plane cannot redirect a click onto another, higher or lower tile.
pub fn pick_track_draft(
    point: Vec2,
    park: &Park,
    rotation: u8,
    mode: ViewMode,
) -> Option<TrackPieceSpec> {
    if !point.is_finite() {
        return None;
    }
    let mut closest: Option<(f32, (u32, u32), TrackPieceSpec)> = None;
    for &spec in park.track_draft() {
        let geometry = DraftGeometry::new(spec);
        for index in 0..geometry.points.len().saturating_sub(1) {
            let Some(depth) = geometry
                .strip(park, rotation, index, -geometry.half, geometry.half)
                .hit_depth(point)
            else {
                continue;
            };
            let tie = (spec.pos.x, spec.pos.y);
            if closest.is_none_or(|(old_depth, old_tie, _)| {
                depth > old_depth + 0.0001 || ((depth - old_depth).abs() <= 0.0001 && tie > old_tie)
            }) {
                closest = Some((depth, tie, spec));
            }
        }
    }
    let (depth, _, spec) = closest?;
    if let Some(hit) = pick_surface(point, park, rotation, mode)
        && selectable_surfaces(park, hit, rotation, mode)
            .into_iter()
            .filter_map(|surface| surface.hit_depth(point))
            .any(|world_depth| world_depth > depth + 0.0001)
    {
        return None;
    }
    Some(spec)
}

fn ghost(painter: &mut Painter, park: &Park, spec: TrackPieceSpec, rotation: u8, ink: Ink) {
    let geometry = DraftGeometry::new(spec);
    let points = &geometry.points;
    if points.len() < 2 {
        return;
    }
    let mut fill = ink;
    fill[3] = 0.26;
    for index in 0..points.len() - 1 {
        let strip = |left, right| geometry.strip(park, rotation, index, left, right);
        painter.surface(strip(-geometry.half, geometry.half), fill);
        for side in [-0.18, 0.18] {
            painter.surface(strip(side - 0.035, side + 0.035), ink);
        }
        if spec.kind == TrackKind::Station {
            painter.surface(strip(-0.42, -0.35), ink);
            painter.surface(strip(0.35, 0.42), ink);
        }
    }
    // An explicit travel arrow removes left/right ambiguity after camera turns.
    let middle = points.len() / 2;
    let center = if points.len() == 2 {
        points[0].lerp(points[1], 0.55)
    } else {
        points[middle]
    };
    let side = geometry.normals[middle];
    let heading = Vec3::new(side.y, -side.x, 0.0);
    painter.triangle_3d(
        [
            center + heading * 0.19,
            center - heading * 0.14 + side * 0.13,
            center - heading * 0.14 - side * 0.13,
        ]
        .map(|point| projected(point, park, rotation) + Vec3::Z * DETAIL_BIAS),
        ink,
    );
}

pub fn track_preview_mesh(editor: &TrackEditor, park: &Park, rotation: u8) -> Mesh {
    let mut painter = Painter::default();
    if !editor.active {
        return painter.finish();
    }
    for &spec in park.track_draft() {
        if editor
            .preview
            .is_none_or(|preview| preview.spec.pos != spec.pos)
        {
            ghost(&mut painter, park, spec, rotation, [0.12, 0.70, 0.88, 0.9]);
        }
    }
    if let Some(preview) = editor.preview {
        let ink = if preview.erase {
            [0.98, 0.47, 0.12, 0.95]
        } else if preview.valid {
            [0.15, 0.82, 0.39, 0.95]
        } else {
            [0.94, 0.16, 0.22, 0.95]
        };
        ghost(&mut painter, park, preview.spec, rotation, ink);
    }
    painter.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use park_sim::{TrackBuildKind, TrackDirection};

    #[test]
    fn draft_curve_uses_sampled_arc_and_all_camera_rotations_pick_the_same_tile() {
        let mut park = Park::new(32, 32);
        let pos = TilePos { x: 20, y: 20 };
        let spec = TrackBuildKind::CurveRight.spec(pos, 8, TrackDirection::East);
        park.set_track_draft_piece(spec).unwrap();
        assert!(spec.samples().len() > 2);
        let editor = TrackEditor {
            active: true,
            ..default()
        };
        for rotation in 0..4 {
            let center = crate::track_tools::build_center(&park, pos, 8, rotation);
            assert_eq!(pick_track_plane(center, &park, rotation, 8), Some(pos));
            let mesh = track_preview_mesh(&editor, &park, rotation);
            let Some(VertexAttributeValues::Float32x3(vertices)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("positions")
            };
            assert!(vertices.len() > 50);
            assert!(vertices.iter().flatten().all(|value| value.is_finite()));
        }
    }

    #[test]
    fn erase_hits_frontmost_draft_at_its_own_height_in_every_rotation() {
        for rotation in 0..4 {
            let mut park = Park::new(32, 32);
            // These two roofs project to the same position. Only the front
            // ribbon is visible; selected construction height is unrelated.
            let far = TrackBuildKind::Station.spec(
                unrotate(14, 17, 32, rotation),
                8,
                TrackDirection::East,
            );
            let near = TrackBuildKind::Station.spec(
                unrotate(18, 21, 32, rotation),
                16,
                TrackDirection::East,
            );
            park.set_track_draft_piece(far).unwrap();
            park.set_track_draft_piece(near).unwrap();
            let point = projected(world_point(near.samples()[4]), &park, rotation).truncate();
            assert_eq!(
                pick_track_draft(point, &park, rotation, ViewMode::Surface),
                Some(near)
            );
            let mut view = crate::ViewState {
                rotation,
                track_editor: TrackEditor {
                    active: true,
                    erase: true,
                    height: 0,
                    ..default()
                },
                ..default()
            };
            crate::track_tools::point_at(&mut park, &mut view, point, false);
            assert_eq!(view.track_editor.preview.unwrap().spec, near);
            assert_eq!(park.track_draft().len(), 2);
            crate::track_tools::point_at(&mut park, &mut view, point, true);
            assert_eq!(park.track_draft(), &[far]);
            assert_eq!(
                pick_track_draft(point, &park, rotation, ViewMode::Surface),
                Some(far)
            );
        }
    }
}
