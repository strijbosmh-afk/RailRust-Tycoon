//! Named track designs share the editor's draft and never build live rails here.
use super::*;
use crate::design_library::{DesignCommand, DesignLibrary};

pub(crate) const PANEL_WIDTH: f32 = 900.0;
pub(crate) const PANEL_HEIGHT: f32 = 600.0;

#[derive(Component)]
pub(crate) struct DesignsPanel;

#[derive(Component)]
pub(crate) struct NameField;

#[derive(Component)]
pub(crate) struct ConfirmationRow;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DesignImage {
    Preview,
    Thumbnail(usize),
}

#[derive(Component)]
pub(crate) struct DesignThumbnail(pub(crate) usize);

#[derive(Component)]
pub(crate) struct ThumbnailLabel(pub(crate) usize);

#[derive(Component)]
pub(crate) enum DesignLabel {
    Name,
    Draft,
    Selection,
    Details,
    X,
    Y,
    Height,
    Rotation,
    Notice,
    PreviewStatus,
    GalleryPage,
}

fn text(parent: &mut ChildSpawnerCommands, value: &str, size: f32, kind: DesignLabel) {
    parent.spawn((
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(INK),
        kind,
    ));
}

fn design_button(
    parent: &mut ChildSpawnerCommands,
    caption: &str,
    command: DesignCommand,
    width: f32,
) {
    button(parent, caption, Action::Design(command), width);
}

fn value_cell(parent: &mut ChildSpawnerCommands, width: f32, kind: DesignLabel) {
    parent
        .spawn(Node {
            width: px(width),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| text(p, "", 13.0, kind));
}

fn preview_column(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            width: px(320),
            height: percent(100),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        })
        .with_children(|p| {
            label(p, "SELECTED DESIGN", 12.0, MUTED);
            p.spawn((
                DesignImage::Preview,
                ImageNode::default(),
                Node {
                    width: px(320),
                    height: px(180),
                    flex_shrink: 0.0,
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(238, 242, 228)),
                BorderColor::all(BORDER),
                BorderRadius::all(px(6)),
            ));
            p.spawn(Node {
                width: percent(100),
                height: px(84),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|p| text(p, "", 12.0, DesignLabel::PreviewStatus));
            for row_index in 0..2 {
                p.spawn(Node {
                    width: percent(100),
                    height: px(108),
                    flex_shrink: 0.0,
                    column_gap: px(8),
                    ..row()
                })
                .with_children(|p| {
                    for column_index in 0..2 {
                        let slot = row_index * 2 + column_index;
                        p.spawn((
                            DesignThumbnail(slot),
                            Button,
                            Action::Design(DesignCommand::Select(usize::MAX)),
                            Node {
                                display: Display::None,
                                width: px(156),
                                height: px(108),
                                flex_shrink: 0.0,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(2),
                                border: UiRect::all(px(1)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(BUTTON),
                            BorderColor::all(BORDER),
                            BorderRadius::all(px(5)),
                        ))
                        .with_children(|p| {
                            p.spawn((
                                DesignImage::Thumbnail(slot),
                                ImageNode::default(),
                                Node {
                                    width: percent(100),
                                    height: px(70),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                            ));
                            p.spawn(Node {
                                width: percent(100),
                                height: px(32),
                                flex_shrink: 0.0,
                                padding: UiRect::axes(px(4), px(0)),
                                overflow: Overflow::clip(),
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn((
                                    ThumbnailLabel(slot),
                                    Text::new(""),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextLayout::new_with_linebreak(
                                        bevy::text::LineBreak::AnyCharacter,
                                    ),
                                    TextColor(INK),
                                ));
                            });
                        });
                    }
                });
            }
            text(p, "No saved designs yet", 11.0, DesignLabel::GalleryPage);
        });
}

pub(crate) fn setup(commands: &mut Commands) {
    commands
        .spawn((
            DesignsPanel,
            GlobalZIndex(10),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.09, 0.65)),
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    width: px(PANEL_WIDTH),
                    height: px(PANEL_HEIGHT),
                    padding: UiRect::all(px(20)),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(16),
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                preview_column(p);
                p.spawn(Node {
                    width: px(520),
                    height: percent(100),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(Node {
                        height: px(30),
                        flex_shrink: 0.0,
                        justify_content: JustifyContent::SpaceBetween,
                        ..row()
                    })
                    .with_children(|p| {
                        label(p, "Track designs", 23.0, INK);
                        design_button(p, "Close [Esc]", DesignCommand::Close, 116.0);
                    });
                    label(
                        p,
                        "Save a draft for reuse. Load into the editor, then Apply to build.\nOpen from Track tools with Ctrl+D.",
                        12.0,
                        MUTED,
                    );
                    p.spawn((
                        NameField,
                        Button,
                        Action::Design(DesignCommand::NameFocus),
                        Node {
                            width: percent(100),
                            height: px(32),
                            flex_shrink: 0.0,
                            padding: UiRect::axes(px(9), px(5)),
                            border: UiRect::all(px(1)),
                            overflow: Overflow::clip(),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(BUTTON),
                        BorderColor::all(BORDER),
                        BorderRadius::all(px(5)),
                    ))
                    .with_children(|p| text(p, "Design name (32 characters)", 13.0, DesignLabel::Name));
                    p.spawn(row()).with_children(|p| {
                        design_button(p, "Save new", DesignCommand::SaveNew, 154.0);
                        design_button(p, "Update selected", DesignCommand::UpdateSelected, 166.0);
                        value_cell(p, 174.0, DesignLabel::Draft);
                    });
                    p.spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        ..row()
                    })
                    .with_children(|p| {
                        design_button(p, "< Previous", DesignCommand::Browse(-1), 100.0);
                        text(p, "No saved designs", 13.0, DesignLabel::Selection);
                        design_button(p, "Next >", DesignCommand::Browse(1), 100.0);
                    });
                    p.spawn(Node {
                        height: px(42),
                        width: percent(100),
                        flex_shrink: 0.0,
                        overflow: Overflow::clip(),
                        ..default()
                    })
                    .with_children(|p| text(p, "", 14.0, DesignLabel::Details));
                    label(
                        p,
                        "Anchor: first station (or first piece) / Use its entry height",
                        12.0,
                        MUTED,
                    );
                    p.spawn(row()).with_children(|p| {
                        design_button(p, "-", DesignCommand::MoveX(-1), 32.0);
                        value_cell(p, 172.0, DesignLabel::X);
                        design_button(p, "+", DesignCommand::MoveX(1), 32.0);
                        design_button(p, "-", DesignCommand::MoveY(-1), 32.0);
                        value_cell(p, 172.0, DesignLabel::Y);
                        design_button(p, "+", DesignCommand::MoveY(1), 32.0);
                    });
                    p.spawn(row()).with_children(|p| {
                        design_button(p, "-", DesignCommand::Height(-1), 32.0);
                        value_cell(p, 172.0, DesignLabel::Height);
                        design_button(p, "+", DesignCommand::Height(1), 32.0);
                        p.spawn((
                            Button,
                            Action::Design(DesignCommand::Turn),
                            Node {
                                width: px(253),
                                height: px(30),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(px(1)),
                                ..default()
                            },
                            BackgroundColor(BUTTON),
                            BorderColor::all(BORDER),
                            BorderRadius::all(px(5)),
                        ))
                        .with_children(|p| text(p, "Turn / 0 degrees", 13.0, DesignLabel::Rotation));
                    });
                    p.spawn(row()).with_children(|p| {
                        design_button(p, "Load into draft", DesignCommand::LoadDraft, 253.0);
                        design_button(p, "Delete selected", DesignCommand::DeleteSelected, 253.0);
                    });
                    p.spawn(Node {
                        height: px(46),
                        width: percent(100),
                        flex_shrink: 0.0,
                        overflow: Overflow::clip(),
                        ..default()
                    })
                    .with_children(|p| text(p, "", 12.0, DesignLabel::Notice));
                    p.spawn((ConfirmationRow, row())).with_children(|p| {
                        design_button(p, "Confirm", DesignCommand::Confirm, 253.0);
                        design_button(p, "Cancel", DesignCommand::Cancel, 253.0);
                    });
                });
            });
        });
}

fn selection_details(library: &DesignLibrary) -> String {
    let Some(design) = library.selected.and_then(|i| library.entries.get(i)) else {
        return "Save the current draft to start your library.".into();
    };
    let pieces = design.pieces();
    let min_x = pieces.iter().map(|piece| piece.pos.x).min().unwrap_or(0);
    let max_x = pieces.iter().map(|piece| piece.pos.x).max().unwrap_or(0);
    let min_y = pieces.iter().map(|piece| piece.pos.y).min().unwrap_or(0);
    let max_y = pieces.iter().map(|piece| piece.pos.y).max().unwrap_or(0);
    let (width, depth) = if library.rotation.is_multiple_of(2) {
        (max_x - min_x + 1, max_y - min_y + 1)
    } else {
        (max_y - min_y + 1, max_x - min_x + 1)
    };
    format!(
        "{}\n{} pieces / {width} x {depth} tiles at this rotation",
        design.name(),
        pieces.len()
    )
}

fn notice(park: &park_sim::Park, library: &DesignLibrary) -> String {
    let name = library
        .selected
        .and_then(|i| library.entries.get(i))
        .map_or("this design", |design| design.name());
    match library.confirmation {
        Some(DesignCommand::UpdateSelected) => {
            format!(
                "Replace \"{name}\" with the current draft and name?\nThe previous saved design will be overwritten."
            )
        }
        Some(DesignCommand::DeleteSelected) => {
            format!(
                "Delete \"{name}\" from the library?\nYour draft and installed track will stay as they are."
            )
        }
        Some(DesignCommand::LoadDraft) => format!(
            "Replace the current {}-piece draft with \"{name}\"?\nYou can Undo this load after returning to Track tools.",
            park.track_draft().len()
        ),
        _ if !library.available && library.notice.is_empty() => {
            "The design library is unavailable. Your current draft remains in Track tools.".into()
        }
        _ => library.notice.clone(),
    }
}

type DesignButtons<'w, 's> = Query<
    'w,
    's,
    (
        &'static Action,
        &'static Interaction,
        &'static mut BackgroundColor,
    ),
    (With<Button>, Without<DesignThumbnail>),
>;

type ThumbnailButtons<'w, 's> = Query<
    'w,
    's,
    (
        &'static DesignThumbnail,
        &'static mut Node,
        &'static mut Action,
        &'static mut BorderColor,
        &'static Interaction,
        &'static mut BackgroundColor,
    ),
    (
        Without<NameField>,
        Without<DesignsPanel>,
        Without<ConfirmationRow>,
    ),
>;

type DesignPanels<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<DesignsPanel>,
        Without<ConfirmationRow>,
        Without<DesignThumbnail>,
    ),
>;

type ConfirmationRows<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<ConfirmationRow>,
        Without<DesignsPanel>,
        Without<DesignThumbnail>,
    ),
>;

#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct DesignControls<'w, 's> {
    name_fields:
        Query<'w, 's, &'static mut BorderColor, (With<NameField>, Without<DesignThumbnail>)>,
    buttons: DesignButtons<'w, 's>,
    thumbnails: ThumbnailButtons<'w, 's>,
    thumbnail_labels: Query<
        'w,
        's,
        (
            &'static ThumbnailLabel,
            &'static mut Text,
            &'static mut TextFont,
        ),
        Without<DesignLabel>,
    >,
}

pub(crate) fn refresh(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    library: Res<DesignLibrary>,
    mut panels: DesignPanels<'_, '_>,
    mut confirmation_rows: ConfirmationRows<'_, '_>,
    mut controls: DesignControls,
    mut labels: Query<(&DesignLabel, &mut Text)>,
) {
    for mut panel in &mut panels {
        panel.display = if view.designs_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !view.designs_open {
        return;
    }
    let page_start = crate::design_preview::page_start(&library);
    for (slot, mut node, mut action, mut border, interaction, mut background) in
        &mut controls.thumbnails
    {
        let index = page_start + slot.0;
        let populated = index < library.entries.len();
        node.display = if populated {
            Display::Flex
        } else {
            Display::None
        };
        let command = DesignCommand::Select(if populated { index } else { usize::MAX });
        *action = Action::Design(command);
        let selected = populated && library.selected == Some(index);
        *border = BorderColor::all(if selected { GOLD } else { BORDER });
        let enabled =
            populated && crate::design_library::command_enabled(&sim.0, &library, command);
        background.0 = if !enabled {
            Color::srgb_u8(239, 240, 232)
        } else if !view.capture_active && *interaction == Interaction::Pressed {
            GOLD
        } else if !view.capture_active && *interaction == Interaction::Hovered {
            Color::srgb_u8(207, 220, 197)
        } else if selected {
            Color::srgb_u8(238, 232, 204)
        } else {
            BUTTON
        };
    }
    for (slot, mut text, mut font) in &mut controls.thumbnail_labels {
        let name = library
            .entries
            .get(page_start + slot.0)
            .map_or("", |design| design.name());
        if text.0 != name {
            text.0 = name.into();
        }
        // Even 32 wide characters fit in two caption lines at the smaller size.
        let size = if name.chars().count() > 26 { 9.0 } else { 11.0 };
        if font.font_size != size {
            font.font_size = size;
        }
    }
    for (action, interaction, mut background) in &mut controls.buttons {
        let Action::Design(command) = action else {
            continue;
        };
        let enabled = crate::design_library::command_enabled(&sim.0, &library, *command);
        background.0 = if !enabled {
            Color::srgb_u8(239, 240, 232)
        } else if !view.capture_active && *interaction == Interaction::Pressed {
            GOLD
        } else if !view.capture_active && *interaction == Interaction::Hovered {
            Color::srgb_u8(207, 220, 197)
        } else {
            BUTTON
        };
    }
    for mut row in &mut confirmation_rows {
        row.display = if library.confirmation.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut border in &mut controls.name_fields {
        *border = BorderColor::all(if library.name_focused { GOLD } else { BORDER });
    }
    for (kind, mut text) in &mut labels {
        let value = match kind {
            DesignLabel::Name if library.name.is_empty() && !library.name_focused => {
                "Design name (32 characters)".into()
            }
            DesignLabel::Name => format!(
                "{}{}",
                library.name,
                if library.name_focused { " |" } else { "" }
            ),
            DesignLabel::Draft => format!("Draft: {} pieces", sim.0.track_draft().len()),
            DesignLabel::Selection => library
                .selected
                .filter(|index| *index < library.entries.len())
                .map_or_else(
                    || "No saved designs".into(),
                    |index| format!("Design {} / {}", index + 1, library.entries.len()),
                ),
            DesignLabel::Details => selection_details(&library),
            DesignLabel::X => format!("X: {}", library.anchor.x + 1),
            DesignLabel::Y => format!("Y: {}", library.anchor.y + 1),
            DesignLabel::Height => format!("Entry height: {}", library.height),
            DesignLabel::Rotation => {
                format!(
                    "Turn clockwise / {} degrees",
                    u16::from(library.rotation) * 90
                )
            }
            DesignLabel::Notice => notice(&sim.0, &library),
            DesignLabel::PreviewStatus => crate::design_preview::status_text(&sim.0, &library),
            DesignLabel::GalleryPage if library.entries.is_empty() => "No saved designs yet".into(),
            DesignLabel::GalleryPage => format!(
                "Designs {}-{} of {} / click a layout to select",
                page_start + 1,
                (page_start + crate::design_preview::PAGE_SIZE).min(library.entries.len()),
                library.entries.len()
            ),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
