//! Separate audio controls inside the existing Help modal input boundary.
use super::*;
use crate::sound::{Settings, SoundSettings};
#[derive(Component)]
pub(crate) struct AudioPanel;
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Control {
    Level(usize, i8),
    Mute,
    Defaults,
    Test,
}
#[derive(Component)]
pub(crate) enum AudioLabel {
    Level(usize),
    Mute,
    Status,
}
fn control(p: &mut ChildSpawnerCommands, text: &str, c: Control, width: f32) {
    p.spawn((
        Button,
        c,
        Node {
            width: px(width),
            height: px(32),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BUTTON),
        BorderRadius::all(px(5)),
    ))
    .with_children(|p| {
        label(p, text, 13.0, INK);
    });
}
pub(crate) fn setup(commands: &mut Commands) {
    commands.spawn((AudioPanel,GlobalZIndex(11),Node{display:Display::None,position_type:PositionType::Absolute,width:percent(100),height:percent(100),align_items:AlignItems::Center,justify_content:JustifyContent::Center,..default()},BackgroundColor(Color::srgba(0.04,0.12,0.09,0.65))))
        .with_children(|p|{p.spawn((Node{width:px(600),padding:UiRect::all(px(24)),flex_direction:FlexDirection::Column,row_gap:px(16),..default()},BackgroundColor(PAPER),BorderRadius::all(px(12))))
            .with_children(|p|{
                label(p,"The sounds of your park",23.0,INK);
                label(p,"Original meadow music, birds, crowd, rides and gentle feedback.\nF10 opens/closes this window. Sound fades out when focus is lost.",12.0,MUTED);
                for (i,name) in ["Master","Music","Ambience","Rides","Alerts & interface"].iter().enumerate(){
                    p.spawn(Node{justify_content:JustifyContent::SpaceBetween,..row()}).with_children(|p|{
                        p.spawn((Text::new(format!("{name}: --%")),TextFont{font_size:14.0,..default()},TextColor(INK),AudioLabel::Level(i)));
                        p.spawn(row()).with_children(|p|{control(p,"-10",Control::Level(i,-10),65.0);control(p,"+10",Control::Level(i,10),65.0);});
                    });
                }
                p.spawn(row()).with_children(|p|{
                    p.spawn((Button,Control::Mute,Node{width:px(135),height:px(32),align_items:AlignItems::Center,justify_content:JustifyContent::Center,..default()},BackgroundColor(BUTTON),BorderRadius::all(px(5))))
                        .with_children(|p|{p.spawn((Text::new("Mute"),TextFont{font_size:13.0,..default()},TextColor(INK),AudioLabel::Mute));});
                    control(p,"Test chime",Control::Test,120.0);control(p,"Reset defaults",Control::Defaults,160.0);
                });
                p.spawn((Text::new(""),TextFont{font_size:12.0,..default()},TextColor(MUTED),AudioLabel::Status));
                label(p,"Music and ambience continue while paused; ride sounds pause.\nSound settings save separately and never change park finances or time.",12.0,MUTED);
                button(p,"Back to the park [Esc]",Action::CloseHelp,240.0);
            });});
}
type ChangedControls<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static Control), (Changed<Interaction>, With<Button>)>;
pub(crate) fn apply(
    view: Res<ViewState>,
    mut settings: Option<ResMut<SoundSettings>>,
    buttons: ChangedControls<'_, '_>,
) {
    if !view.help || !view.audio_view || view.capture_active || view.modal_input_blocked {
        return;
    }
    let Some(settings) = settings.as_mut() else {
        return;
    };
    let requested: Vec<_> = buttons
        .iter()
        .filter(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, c)| *c)
        .collect();
    if requested.len() > 1 {
        settings.notice = "Choose one sound control at a time.".into();
        return;
    }
    let Some(c) = requested.first() else {
        return;
    };
    match *c {
        Control::Level(i, delta) if i < 5 => {
            settings.value.levels[i] =
                (i16::from(settings.value.levels[i]) + i16::from(delta)).clamp(0, 100) as u8;
        }
        Control::Mute => settings.value.muted = !settings.value.muted,
        Control::Defaults => settings.value = Settings::default(),
        Control::Test => {
            settings.test_requested = true;
            settings.notice =
                "Test chime requested at your current levels; mute and focus apply.".into();
            return;
        }
        _ => return,
    }
    settings.persist();
}
pub(crate) fn refresh(
    view: Res<ViewState>,
    settings: Option<Res<SoundSettings>>,
    mut panel: Query<&mut Node, With<AudioPanel>>,
    mut texts: Query<(&AudioLabel, &mut Text)>,
    mut buttons: Query<(&Control, &Interaction, &mut BackgroundColor)>,
) {
    for mut p in &mut panel {
        p.display = if view.help && view.audio_view {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !view.help || !view.audio_view {
        return;
    }
    let Some(settings) = settings else {
        return;
    };
    for (kind, mut text) in &mut texts {
        text.0 = match kind {
            AudioLabel::Level(i) => format!(
                "{}: {}%",
                ["Master", "Music", "Ambience", "Rides", "Alerts & interface"][*i],
                settings.value.levels[*i]
            ),
            AudioLabel::Mute => if settings.value.muted {
                "Unmute"
            } else {
                "Mute"
            }
            .into(),
            AudioLabel::Status => format!(
                "{}\n{}",
                settings.backend(),
                settings.notice.chars().take(210).collect::<String>()
            ),
        };
    }
    for (c, i, mut bg) in &mut buttons {
        bg.0 = if *c == Control::Mute && settings.value.muted {
            GOLD
        } else if *i == Interaction::Hovered {
            BORDER
        } else {
            BUTTON
        };
    }
}
