//! Related construction controls share a labelled group and local settings.
use super::*;

pub(super) fn setup(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..row()
        })
        .with_children(|p| {
            label(p, "BUILD", 16.0, INK);
            toolbar(
                p,
                &[
                    (
                        "Build catalogue",
                        Action::Catalogue(catalogue::Command::Open),
                    ),
                    ("Services [M]", Action::Services),
                    ("Demolish [X]", Action::SetPathTool(PathTool::Demolish)),
                ],
            );
        });

    group(parent, "PATHS", |p| {
        toolbar(
            p,
            &[
                ("Path [P]", Action::SetPathTool(PathTool::Build)),
                ("Queue [J]", Action::SetPathTool(PathTool::BuildQueue)),
                ("Slope [C]", Action::CyclePathShape),
            ],
        );
        dynamic_label(p, "Path height: 0", 11.0, MUTED, Label::PathHeight);
        toolbar(
            p,
            &[
                ("Lower path [PgDn]", Action::PathHeight(-1)),
                ("Raise path [PgUp]", Action::PathHeight(1)),
            ],
        );
    });

    group(parent, "LANDSCAPE", |p| {
        toolbar(
            p,
            &[
                (
                    "Raise terrain [B]",
                    Action::SetPathTool(PathTool::RaiseTerrain),
                ),
                (
                    "Lower terrain [V]",
                    Action::SetPathTool(PathTool::LowerTerrain),
                ),
                ("Buy land / $5", Action::SetPathTool(PathTool::BuyLand)),
            ],
        );
        toolbar(
            p,
            &[
                ("Water", Action::SetPathTool(PathTool::PaintWater)),
                ("Grass", Action::SetPathTool(PathTool::PaintGrass)),
                ("Sand", Action::SetPathTool(PathTool::PaintSand)),
            ],
        );
        p.spawn(Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..row()
        })
        .with_children(|p| {
            dynamic_label(p, "Brush: 1 × 1", 11.0, MUTED, Label::TerrainBrushRadius);
            toolbar(
                p,
                &[
                    ("Smaller brush", Action::TerrainBrushRadius(-1)),
                    ("Larger brush", Action::TerrainBrushRadius(1)),
                ],
            );
        });
    });

    group(parent, "SCENERY & PARK ITEMS", |p| {
        toolbar(
            p,
            &[
                (
                    "Tree",
                    Action::SetPathTool(PathTool::Scenery(SceneryKind::Tree)),
                ),
                (
                    "Flowers",
                    Action::SetPathTool(PathTool::Scenery(SceneryKind::FlowerBed)),
                ),
                ("Sign / $2", Action::ToggleSign),
                ("Fence / $3", Action::ToggleFence),
            ],
        );
        toolbar(
            p,
            &[
                ("Bin on path", Action::ToggleBin),
                ("Bench on path / $1", Action::ToggleBench),
                ("Patrol path", Action::TogglePatrol),
            ],
        );
    });

    for (initial, kind, color) in [
        ("Choose a build tool.", Label::PathHint, INK),
        ("Last: no build changes yet.", Label::PathResult, MUTED),
    ] {
        parent
            .spawn(Node {
                width: percent(100),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|p| dynamic_label(p, initial, 11.0, color, kind));
    }
    label(parent, "Esc: return to inspection", 11.0, MUTED);
}

fn toolbar(parent: &mut ChildSpawnerCommands, actions: &[(&str, Action)]) {
    parent
        .spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(6),
            row_gap: px(6),
            ..row()
        })
        .with_children(|p| {
            for (caption, action) in actions {
                button(p, caption, *action, 40.0);
            }
        });
}

fn group(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0,
                row_gap: px(5),
                padding: UiRect::all(px(7)),
                margin: UiRect::top(px(5)),
                ..default()
            },
            BackgroundColor(theme::INSET),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| {
            label(p, title, 11.0, theme::ACCENT);
            content(p);
        });
}
