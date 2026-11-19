//! Three durable advice requests per park, presented through the Help modal boundary.
use super::*;
use crate::front_end::FrontEnd;

#[derive(Resource, Default)]
pub(crate) struct AdviceWindow {
    tips: Option<[park_sim::ProfitTip; 2]>,
    notice: String,
}
#[derive(Component)]
pub(crate) struct Panel;
#[derive(Component)]
pub(crate) enum AdviceLabel {
    Title(usize),
    Text(usize),
    Allowance,
    Notice,
}

pub(super) fn setup(commands: &mut Commands) {
    commands.insert_resource(AdviceWindow::default());
    commands
        .spawn((
            Panel,
            GlobalZIndex(11),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.1, 0.07, 0.72)),
        ))
        .with_children(|p| {
            p.spawn((
                layout::ScrollLayer::Help,
                ScrollPosition::default(),
                layout::dialog(94.0, 690.0, None),
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                p.spawn(row()).with_children(|p| {
                    label(p, "Profit tips", 24.0, INK);
                    button(p, "Close tips [Esc]", Action::CloseHelp, 40.0);
                });
                p.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(GOLD),
                    AdviceLabel::Allowance,
                ));
                for index in 0..2 {
                    p.spawn((
                        Node {
                            padding: UiRect::all(px(14)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(8),
                            margin: UiRect::top(px(8)),
                            ..default()
                        },
                        BackgroundColor(theme::INSET),
                        BorderRadius::all(px(6)),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(INK),
                            AdviceLabel::Title(index),
                        ));
                        p.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 15.0,
                                ..default()
                            },
                            TextColor(INK),
                            AdviceLabel::Text(index),
                        ));
                    });
                }
                p.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(MUTED),
                    AdviceLabel::Notice,
                ));
            });
        });
}

pub(crate) fn apply(
    mut view: ResMut<ViewState>,
    mut sim: ResMut<Simulation>,
    mut front: ResMut<FrontEnd>,
    mut advice: ResMut<AdviceWindow>,
) {
    if !std::mem::take(&mut view.tips_requested)
        || view.edits_blocked()
        || sim.0.tips_used() >= park_sim::MAX_TIP_USES
    {
        return;
    }
    let result = front.consume_tip_use(&mut sim.0, &mut view);
    advice.tips = None;
    advice.notice = match result {
        Ok(_) => {
            advice.tips = Some(sim.0.profit_tips());
            "This use is recorded for this park, including after loading or restarting. Advice reflects the park when requested.".into()
        }
        Err(error) => format!("Could not record a tip use. {error}"),
    };
    view.set_modal(true, false, false, false);
    view.audio_view = false;
    view.tips_view = true;
    view.modal_transition_pending = true;
}

pub(crate) fn refresh(
    view: Res<ViewState>,
    sim: Res<Simulation>,
    mut advice: ResMut<AdviceWindow>,
    mut panels: Query<&mut Node, With<Panel>>,
    mut labels: Query<(&AdviceLabel, &mut Text)>,
) {
    if view.capture_active && view.tips_view && advice.tips.is_none() {
        advice.tips = Some(sim.0.profit_tips());
        advice.notice = "Preview of advice for this park.".into();
    }
    for mut panel in &mut panels {
        panel.display = if view.help && view.tips_view {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (kind, mut text) in &mut labels {
        let value = match kind {
            AdviceLabel::Allowance => format!(
                "{} of 3 uses spent · {} remaining",
                sim.0.tips_used(),
                park_sim::MAX_TIP_USES - sim.0.tips_used()
            ),
            AdviceLabel::Notice => advice.notice.clone(),
            AdviceLabel::Title(index) => advice
                .tips
                .as_ref()
                .map_or(String::new(), |tips| tips[*index].title.into()),
            AdviceLabel::Text(index) => advice
                .tips
                .as_ref()
                .map_or(String::new(), |tips| tips[*index].text.clone()),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{park_library::ParkLibrary, persistence::SaveSession};

    #[test]
    fn storage_failure_withholds_advice_and_does_not_spend_a_use() {
        let root = std::env::temp_dir().join(format!(
            "parkbound-tips-denied-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&root, b"A file cannot be a save directory").unwrap();
        let session =
            SaveSession::from_arguments(["--save-dir".into(), root.clone().into_os_string()]);
        let mut front = FrontEnd::new(true, &session);
        front.capture_scene("demo-park").unwrap();
        let mut app = App::new();
        app.insert_resource(front)
            .insert_resource(Simulation(crate::new_game_park()))
            .insert_resource(crate::new_game_view())
            .init_resource::<AdviceWindow>()
            .add_systems(Update, (super::super::button_actions, apply).chain());
        app.world_mut()
            .spawn((Button, Interaction::Pressed, Action::Tips));
        app.update();
        assert_eq!(app.world().resource::<Simulation>().0.tips_used(), 0);
        let advice = app.world().resource::<AdviceWindow>();
        assert!(advice.tips.is_none());
        assert!(advice.notice.contains("Could not record"));
        assert_eq!(
            std::fs::read(&root).unwrap(),
            b"A file cannot be a save directory"
        );
        std::fs::remove_file(root).unwrap();
    }

    #[test]
    fn header_button_charges_once_shows_two_tips_and_rejects_a_fourth_after_restart() {
        let root = std::env::temp_dir().join(format!(
            "parkbound-tips-ui-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let session =
            SaveSession::from_arguments(["--save-dir".into(), root.clone().into_os_string()]);
        let mut front = FrontEnd::new(true, &session);
        front.capture_scene("demo-park").unwrap();
        let mut app = App::new();
        app.insert_resource(front)
            .insert_resource(Simulation(crate::new_game_park()))
            .insert_resource(crate::new_game_view())
            .init_resource::<AdviceWindow>()
            .add_systems(Update, (super::super::button_actions, apply).chain());
        for expected in 1..=3 {
            let button = app
                .world_mut()
                .spawn((Button, Interaction::Pressed, Action::Tips))
                .id();
            app.update();
            assert_eq!(app.world().resource::<Simulation>().0.tips_used(), expected);
            assert_eq!(
                app.world()
                    .resource::<AdviceWindow>()
                    .tips
                    .as_ref()
                    .unwrap()
                    .len(),
                2
            );
            assert!(app.world().resource::<ViewState>().tips_view);
            app.world_mut().despawn(button);
            let mut view = app.world_mut().resource_mut::<ViewState>();
            view.set_modal(false, false, false, false);
            view.modal_transition_pending = false;
            view.modal_input_blocked = false;
        }
        let library = ParkLibrary::new(&root);
        let id = library.entries().unwrap()[0].id;
        // The initial auto-created snapshot predates the tips. The library's
        // high-water count still governs a fresh load without a later Save.
        let reopened = library.load(id, false).unwrap();
        assert_eq!(reopened.tips_used(), 3);
        app.insert_resource(Simulation(reopened));
        app.world_mut()
            .spawn((Button, Interaction::Pressed, Action::Tips));
        app.update();
        assert_eq!(app.world().resource::<Simulation>().0.tips_used(), 3);
        assert!(!app.world().resource::<ViewState>().tips_view);
        assert!(library.record_tip_use(id, 0).is_err());
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        assert!(
            root.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("parkbound-tips-ui-")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
