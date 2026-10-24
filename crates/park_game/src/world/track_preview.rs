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

    fn sample(&self, fraction: f32) -> (Vec3, Vec3) {
        let sample = fraction * (self.points.len() - 1) as f32;
        let index = (sample as usize).min(self.points.len() - 2);
        let within = sample - index as f32;
        (
            self.points[index].lerp(self.points[index + 1], within),
            self.normals[index]
                .lerp(self.normals[index + 1], within)
                .normalize_or_zero(),
        )
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
        match spec.kind {
            TrackKind::Station => {
                painter.surface(strip(-0.42, -0.35), ink);
                painter.surface(strip(0.35, 0.42), ink);
            }
            TrackKind::Lift => painter.surface(strip(-0.035, 0.035), ink),
            TrackKind::Brake => {
                for side in [-0.085, 0.085] {
                    painter.surface(strip(side - 0.025, side + 0.025), ink);
                }
            }
            TrackKind::Drop | TrackKind::Curve | TrackKind::Straight => {}
        }
    }
    // An explicit travel arrow removes left/right ambiguity after camera turns.
    // Both ends follow the sampled grade, including a two-step drop; a level
    // tangent at the midpoint would float above one end and cut into the other.
    let (tip, _) = geometry.sample(0.69);
    let (base, side) = geometry.sample(0.36);
    painter.triangle_3d(
        [tip, base + side * 0.13, base - side * 0.13]
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

    const GRADED_PIECES: [TrackBuildKind; 4] = [
        TrackBuildKind::Lift,
        TrackBuildKind::Drop,
        TrackBuildKind::SteepDrop,
        TrackBuildKind::Brake,
    ];
    const DIRECTIONS: [TrackDirection; 4] = [
        TrackDirection::North,
        TrackDirection::East,
        TrackDirection::South,
        TrackDirection::West,
    ];

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

    #[test]
    fn graded_travel_arrows_stay_on_the_rails_in_every_direction_and_rotation() {
        let park = Park::new(32, 32);
        for kind in GRADED_PIECES {
            for direction in DIRECTIONS {
                let spec = kind.spec(TilePos { x: 10, y: 10 }, 8, direction);
                let geometry = DraftGeometry::new(spec);
                for rotation in 0..4 {
                    let mut painter = Painter::default();
                    ghost(&mut painter, &park, spec, rotation, [1.0; 4]);
                    assert!(painter.positions.iter().flatten().all(|v| v.is_finite()));
                    let [a, b, c, _] = geometry
                        .strip(&park, rotation, 0, -geometry.half, geometry.half)
                        .vertices;
                    // Test the 3D plane even when a steep drop is edge-on in
                    // screen space. Remove only the intentional detail bias.
                    let normal = (b - a).cross(c - a).normalize();
                    for vertex in painter.positions.iter().rev().take(3) {
                        let point = Vec3::from_array(*vertex) - Vec3::Z * DETAIL_BIAS;
                        assert!(
                            (point - a).dot(normal).abs() < 0.0002,
                            "arrow left {kind:?} rail plane: {direction:?}, rotation {rotation}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn graded_erase_uses_the_visible_ribbon_in_every_direction_and_rotation() {
        for rotation in 0..4 {
            for kind in GRADED_PIECES {
                for direction in DIRECTIONS {
                    let mut park = Park::new(32, 32);
                    let far = kind.spec(unrotate(14, 17, 32, rotation), 8, direction);
                    let near = kind.spec(unrotate(18, 21, 32, rotation), 16, direction);
                    park.set_track_draft_piece(far).unwrap();
                    park.set_track_draft_piece(near).unwrap();
                    let geometry = DraftGeometry::new(near);
                    let visible = geometry
                        .strip(&park, rotation, 3, -geometry.half, geometry.half)
                        .has_area();
                    let point =
                        projected(world_point(near.samples()[4]), &park, rotation).truncate();
                    // Exactly edge-on ribbons render no filled target. They
                    // must not intercept a click on the world behind them.
                    assert_eq!(
                        pick_track_draft(point, &park, rotation, ViewMode::Surface),
                        visible.then_some(near),
                        "{kind:?}, {direction:?}, rotation {rotation}"
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
                    assert_eq!(
                        view.track_editor.preview.map(|p| p.spec),
                        visible.then_some(near)
                    );
                    assert_eq!(park.track_draft().len(), 2);
                    crate::track_tools::point_at(&mut park, &mut view, point, true);
                    if visible {
                        assert_eq!(park.track_draft(), &[far]);
                        assert_eq!(
                            pick_track_draft(point, &park, rotation, ViewMode::Surface),
                            Some(far)
                        );
                    } else {
                        assert_eq!(park.track_draft(), &[far, near]);
                    }
                }
            }
        }
    }

    #[test]
    fn foreground_terrain_blocks_graded_erase_but_cutaway_restores_visible_targets() {
        for rotation in 0..4 {
            for kind in GRADED_PIECES {
                for direction in DIRECTIONS {
                    let mut park = Park::new(32, 32);
                    let spec = kind.spec(unrotate(10, 10, 32, rotation), 8, direction);
                    park.set_track_draft_piece(spec).unwrap();
                    let geometry = DraftGeometry::new(spec);
                    let visible = geometry
                        .strip(&park, rotation, 3, -geometry.half, geometry.half)
                        .has_area();
                    let point =
                        projected(world_point(spec.samples()[4]), &park, rotation).truncate();
                    assert_eq!(
                        pick_track_draft(point, &park, rotation, ViewMode::Surface),
                        visible.then_some(spec)
                    );
                    park.set_terrain_height(unrotate(11, 11, 32, rotation), 12)
                        .unwrap();
                    assert_eq!(
                        pick_track_draft(point, &park, rotation, ViewMode::Surface),
                        None,
                        "terrain must hide {kind:?}, {direction:?}, rotation {rotation}"
                    );
                    assert_eq!(
                        pick_track_draft(point, &park, rotation, ViewMode::Underground),
                        visible.then_some(spec)
                    );
                    let mut view = crate::ViewState {
                        rotation,
                        track_editor: TrackEditor {
                            active: true,
                            erase: true,
                            ..default()
                        },
                        ..default()
                    };
                    crate::track_tools::point_at(&mut park, &mut view, point, true);
                    assert_eq!(park.track_draft(), &[spec]);
                }
            }
        }
    }
}
