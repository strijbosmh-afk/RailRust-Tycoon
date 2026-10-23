//! Compact track tools occupy the existing Build panel, including at 1024x680.
use super::*;
use crate::track_tools::{self, TrackCommand};
use park_sim::TrackBuildKind;

#[derive(Component)]
pub(crate) struct BuildPanel;
#[derive(Component)]
pub(crate) struct TrackPanel;
#[derive(Component)]
pub(crate) enum TrackLabel {
    Direction,
    Height,
    Status,
    Notice,
}

fn small_button(parent: &mut ChildSpawnerCommands, text: &str, command: TrackCommand, width: f32) {
    parent
        .spawn((
            Button,
            Action::Track(command),
            Node {
                width: px(width),
                height: px(26),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(BUTTON),
            BorderColor::all(BORDER),
            BorderRadius::all(px(5)),
        ))
        .with_children(|p| match command {
            TrackCommand::Turn => {
                p.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(INK),
                    TrackLabel::Direction,
                ));
            }
            _ => label(p, text, 12.0, INK),
        });
}

pub(super) fn setup(commands: &mut Commands) {
    commands
        .spawn((
            TrackPanel,
            GlobalZIndex(3),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: px(PATH_PANEL_TOP),
                right: px(PATH_PANEL_RIGHT),
                width: px(PATH_PANEL_WIDTH),
                height: px(PATH_PANEL_HEIGHT),
                padding: UiRect::all(px(12)),
                border: UiRect::all(px(1)),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
            BackgroundColor(PAPER),
            BorderColor::all(BORDER),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| {
            label(p, "TRACK DRAFT", 16.0, INK);
            p.spawn(row()).with_children(|p| {
                small_button(
                    p,
                    "Station",
                    TrackCommand::Piece(TrackBuildKind::Station),
                    100.0,
                );
                small_button(
                    p,
                    "Straight",
                    TrackCommand::Piece(TrackBuildKind::Straight),
                    100.0,
                );
            });
            p.spawn(row()).with_children(|p| {
                small_button(
                    p,
                    "Left curve",
                    TrackCommand::Piece(TrackBuildKind::CurveLeft),
                    100.0,
                );
                small_button(
                    p,
                    "Right curve",
                    TrackCommand::Piece(TrackBuildKind::CurveRight),
                    100.0,
                );
            });
            small_button(p, "Heading East [C]", TrackCommand::Turn, 207.0);
            p.spawn(row()).with_children(|p| {
                small_button(p, "-", TrackCommand::Height(-1), 32.0);
                p.spawn(Node {
                    width: px(129),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((
                        Text::new("Height 8 / PgUp-Dn"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(INK),
                        TrackLabel::Height,
                    ));
                });
                small_button(p, "+", TrackCommand::Height(1), 32.0);
            });
            p.spawn(row()).with_children(|p| {
                small_button(p, "Erase [X]", TrackCommand::Erase, 100.0);
                small_button(p, "Clear draft", TrackCommand::Clear, 100.0);
            });
            p.spawn(row()).with_children(|p| {
                small_button(p, "Apply track", TrackCommand::Apply, 100.0);
                small_button(p, "Done [Esc]", TrackCommand::Done, 100.0);
            });
            for (kind, color, height) in [
                (TrackLabel::Status, INK, 54.0),
                (TrackLabel::Notice, MUTED, 54.0),
            ] {
                p.spawn(Node {
                    height: px(height),
                    width: percent(100),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(color),
                        kind,
                    ));
                });
            }
        });
}

pub(crate) fn refresh(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut build: Query<&mut Node, (With<BuildPanel>, Without<TrackPanel>)>,
    mut panels: Query<&mut Node, (With<TrackPanel>, Without<BuildPanel>)>,
    mut labels: Query<(&TrackLabel, &mut Text)>,
    mut cached_status: Local<Option<(u64, String)>>,
) {
    for mut node in &mut build {
        node.display = if view.track_editor.active {
            Display::None
        } else {
            Display::Flex
        };
    }
    for mut node in &mut panels {
        node.display = if view.track_editor.active {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !view.track_editor.active {
        *cached_status = None;
        return;
    }
    // Draft edits and reopening the editor invalidate this geometry-only text.
    // Guest simulation ticks do not need to rewalk the entire proposed route.
    if cached_status
        .as_ref()
        .is_none_or(|(revision, _)| *revision != view.track_editor.revision)
    {
        *cached_status = Some((view.track_editor.revision, track_tools::status(&sim.0)));
    }
    for (kind, mut text) in &mut labels {
        let value = match kind {
            TrackLabel::Direction => format!(
                "Heading {} [C]",
                track_tools::direction_name(view.track_editor.direction)
            ),
            TrackLabel::Height => format!("Height {} / PgUp-Dn", view.track_editor.height),
            TrackLabel::Status => cached_status
                .as_ref()
                .expect("status cached above")
                .1
                .clone(),
            TrackLabel::Notice => {
                if let Some(preview) = view.track_editor.preview.filter(|preview| !preview.valid) {
                    sim.0
                        .validate_track_draft_piece(preview.spec)
                        .err()
                        .map_or_else(
                            || view.track_editor.notice.clone(),
                            |error| track_tools::error_message(&sim.0, error),
                        )
                } else {
                    view.track_editor.notice.clone()
                }
            }
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
