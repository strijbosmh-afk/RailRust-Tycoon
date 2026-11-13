//! Scenario authoring uses the same park construction tools and checked saves.
use super::*;
use crate::SaveSession;
use crate::{
    design_file_dialog::DialogRequest,
    scenario_store::{ScenarioEntry, ScenarioStore},
};
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};
use park_sim::{ChallengeDefinition, Objective, ObjectiveMetric, ScenarioDocument};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Command {
    Close,
    Tab(u8),
    Refresh,
    Select(usize),
    Previous,
    Next,
    Name,
    Description,
    Search,
    ClearSearch,
    New,
    Edit,
    Build,
    Save,
    Play,
    Confirm,
    Cancel,
    Rename,
    Delete,
    Import,
    Export,
    AddGoal,
    RemoveGoal,
    Goal(usize),
    Metric,
    Target(bool),
    Deadline,
    Charging,
    Funds(bool),
    Research,
    Loans,
    Arrivals,
    Ride(usize),
    Facility(usize),
    Test,
    ReturnEditor,
    ReturnPark,
    Designs,
}
#[derive(Clone, Copy, Default)]
enum Field {
    #[default]
    Name,
    Description,
    Search,
}
#[derive(Clone, Copy)]
enum Pending {
    Play(u64),
    Edit(u64),
    Delete(u64),
    Rename(u64),
    ReturnPark,
}
#[derive(Resource, Default)]
pub(crate) struct Workshop {
    tab: u8,
    entries: Vec<ScenarioEntry>,
    selected: Option<u64>,
    page: usize,
    search: String,
    preview_summary: String,
    definition: ChallengeDefinition,
    field: Field,
    goal: usize,
    notice: String,
    original: Option<park_sim::Park>,
    test_draft: Option<park_sim::Park>,
    editing: bool,
    editing_id: Option<u64>,
    generation: u64,
    pending: Option<Pending>,
    transfer: Option<bool>,
}
#[derive(Component)]
pub(crate) struct Panel;
#[derive(Component)]
pub(crate) struct Body;
#[derive(Component)]
pub(crate) struct Banner;
#[derive(Component)]
pub(crate) struct Row(usize);
#[derive(Component)]
pub(crate) struct Section(u8);

pub(crate) fn open(view: &mut ViewState) {
    view.set_modal(false, false, false, false);
    view.workshop_open = true;
    view.ui_pointer_consumed = true;
}
impl Workshop {
    fn visible(&self) -> Vec<&ScenarioEntry> {
        let search = self.search.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&search))
            .collect()
    }
    fn refresh(&mut self, store: &ScenarioStore) {
        match store.list() {
            Ok(rows) => {
                self.entries = rows;
                if self
                    .selected
                    .is_some_and(|id| self.entries.iter().all(|e| e.id != id))
                {
                    self.selected = None;
                }
                self.page = self.page.min(self.visible().len().saturating_sub(1) / 5);
            }
            Err(e) => self.notice = e,
        }
    }
    fn begin_edit(
        &mut self,
        park: &mut park_sim::Park,
        view: &mut ViewState,
        mut draft: park_sim::Park,
        definition: ChallengeDefinition,
        id: Option<u64>,
    ) {
        if self.original.is_none() {
            self.original = Some(park.clone());
        }
        draft.clear_challenge();
        *park = draft;
        self.definition = definition;
        self.editing = true;
        self.tab = 1;
        self.editing_id = id;
        self.generation = view.park_generation;
        self.test_draft = None;
        view.front_new_park = true;
        view.scenario_from_title = false;
        view.pause();
        view.scene_dirty = true;
        self.notice="Editor active. Use Build park to edit terrain, land, paths, rides and facilities. Reopen Scenarios to save or test.".into();
    }
}
fn store(session: &SaveSession) -> Result<ScenarioStore, String> {
    Ok(ScenarioStore::new(
        session
            .store
            .as_ref()
            .ok_or("Save folder unavailable")?
            .directory(),
    ))
}
pub(crate) fn command(
    park: &mut park_sim::Park,
    view: &mut ViewState,
    w: &mut Workshop,
    store: &ScenarioStore,
    action: Command,
) {
    if !view.workshop_open
        || view.front_modal
        || view.capture_active
        || view.diagnostic_case.is_some()
    {
        return;
    }
    view.ui_pointer_consumed = true;
    if !w.editing
        && matches!(
            action,
            Command::Save
                | Command::AddGoal
                | Command::RemoveGoal
                | Command::Goal(_)
                | Command::Metric
                | Command::Target(_)
                | Command::Deadline
                | Command::Charging
                | Command::Funds(_)
                | Command::Research
                | Command::Loans
                | Command::Arrivals
                | Command::Ride(_)
                | Command::Facility(_)
                | Command::Test
        )
    {
        w.notice = "Create or edit a scenario first.".into();
        return;
    }
    let result: Result<(), String> = (|| {
        match action {
            Command::Tab(tab) => {
                if tab > 0 && !w.editing {
                    return Err("Create or edit a scenario first".into());
                }
                w.tab = tab;
            }
            Command::Close | Command::Build => {
                if matches!(action, Command::Close) && view.scenario_from_title {
                    view.scenario_return_title = true;
                }
                view.set_modal(false, false, false, false);
            }
            Command::Refresh => w.refresh(store),
            Command::Select(row) => {
                w.selected = w.visible().get(w.page * 5 + row).map(|e| e.id);
                if let Some(id) = w.selected {
                    match store.load(id) {
                        Ok(doc) => {
                            let preview = doc.template()?;
                            w.preview_summary = format!(
                                "{} × {} · Cash {} · {} rides · {} facilities",
                                preview.width(),
                                preview.height(),
                                money(preview.cash_cents()),
                                preview.all_rides().count(),
                                preview.services().len()
                            );
                            if !w.editing {
                                w.definition = doc.definition;
                            }
                        }
                        Err(e) => w.notice = e,
                    }
                }
            }
            Command::Previous => w.page = w.page.saturating_sub(1),
            Command::Next => w.page = (w.page + 1).min(w.visible().len().saturating_sub(1) / 5),
            Command::Name => w.field = Field::Name,
            Command::Description => w.field = Field::Description,
            Command::Search => w.field = Field::Search,
            Command::ClearSearch => {
                w.search.clear();
                w.page = 0;
            }
            Command::New => w.begin_edit(
                park,
                view,
                park.clone(),
                ChallengeDefinition::default(),
                None,
            ),
            Command::Edit => {
                w.pending = Some(Pending::Edit(w.selected.ok_or("Select a saved scenario")?));
            }
            Command::Save => {
                if !w.editing {
                    return Err("Create or edit a scenario first".into());
                }
                let doc = ScenarioDocument::from_park(park, w.definition.clone())?;
                let id = store.save(w.editing_id, &doc)?;
                w.editing_id = Some(id);
                w.selected = Some(id);
                w.refresh(store);
                w.notice =
                    "Scenario saved with its map, funds, content, rules and objectives.".into();
            }
            Command::Play => {
                w.pending = Some(Pending::Play(w.selected.ok_or("Select a saved scenario")?))
            }
            Command::Delete => {
                w.pending = Some(Pending::Delete(
                    w.selected.ok_or("Select a saved scenario")?,
                ))
            }
            Command::Rename => {
                w.pending = Some(Pending::Rename(
                    w.selected.ok_or("Select a saved scenario")?,
                ))
            }
            Command::ReturnPark => w.pending = Some(Pending::ReturnPark),
            Command::Cancel => w.pending = None,
            Command::Confirm => match w.pending.take() {
                Some(Pending::Edit(id)) => {
                    let doc = store.load(id)?;
                    w.begin_edit(park, view, doc.template()?, doc.definition, Some(id));
                }
                Some(Pending::Delete(id)) => {
                    store.archive(id)?;
                    w.selected = None;
                    w.refresh(store);
                    w.notice = "Scenario moved to the recoverable archive.".into();
                }
                Some(Pending::Rename(id)) => {
                    store.rename(id, &w.definition.name)?;
                    w.refresh(store);
                    w.notice = "Scenario renamed.".into();
                }
                Some(Pending::Play(id)) => {
                    *park = store.load(id)?.play()?;
                    w.original = None;
                    w.test_draft = None;
                    w.editing = false;
                    w.tab = 0;
                    view.front_new_park = true;
                    view.scenario_from_title = false;
                    view.park_generation = view.park_generation.wrapping_add(1);
                    w.generation = view.park_generation;
                    view.scene_dirty = true;
                    view.pause();
                    view.set_modal(false, false, false, false);
                }
                Some(Pending::ReturnPark) => {
                    if let Some(original) = w.original.take() {
                        *park = original;
                        view.scene_dirty = true;
                    }
                    w.test_draft = None;
                    w.editing = false;
                    w.tab = 0;
                    w.notice =
                        "Returned to the original park. Saved scenarios remain in the library."
                            .into();
                }
                None => {}
            },
            Command::Import => w.transfer = Some(false),
            Command::Export => {
                w.selected.ok_or("Select a saved scenario")?;
                w.transfer = Some(true);
            }
            Command::AddGoal => {
                if w.definition.goals.len() < 4 {
                    if let Some(metric) = ObjectiveMetric::ALL
                        .into_iter()
                        .find(|m| w.definition.goals.iter().all(|g| g.metric != *m))
                    {
                        w.definition.goals.push(Objective {
                            metric,
                            target: metric.step(),
                        });
                        w.goal = w.definition.goals.len() - 1;
                    }
                }
            }
            Command::RemoveGoal => {
                if w.definition.goals.len() > 1 {
                    w.definition
                        .goals
                        .remove(w.goal.min(w.definition.goals.len() - 1));
                    w.goal = 0;
                }
            }
            Command::Goal(i) => w.goal = i.min(w.definition.goals.len() - 1),
            Command::Metric => {
                let len = w.definition.goals.len();
                let i = w.goal.min(len - 1);
                let current = w.definition.goals[i].metric;
                let start = ObjectiveMetric::ALL
                    .iter()
                    .position(|m| *m == current)
                    .unwrap();
                for offset in 1..=9 {
                    let next = ObjectiveMetric::ALL[(start + offset) % 9];
                    if w.definition
                        .goals
                        .iter()
                        .enumerate()
                        .all(|(n, g)| n == i || g.metric != next)
                    {
                        w.definition.goals[i] = Objective {
                            metric: next,
                            target: next.step(),
                        };
                        break;
                    }
                }
            }
            Command::Target(up) => {
                let len = w.definition.goals.len();
                let g = &mut w.definition.goals[w.goal.min(len - 1)];
                g.target = if up {
                    g.target.saturating_add(g.metric.step())
                } else {
                    g.target.saturating_sub(g.metric.step()).max(1)
                };
            }
            Command::Deadline => {
                w.definition.deadline_ticks = match w.definition.deadline_ticks {
                    None => Some(9000),
                    Some(9000) => Some(18000),
                    Some(18000) => Some(36000),
                    _ => None,
                }
            }
            Command::Charging => {
                w.definition.charging_rule = w.definition.charging_rule.next();
            }
            Command::Funds(up) => {
                let budget = park.starting_cash_cents();
                park.set_starting_budget(if up {
                    (budget + 50_000).min(10_000_000)
                } else {
                    budget.saturating_sub(50_000).max(10_000)
                })?;
            }
            Command::Research => park.set_sandbox_research(!park.all_research_available()),
            Command::Loans => park.set_loans_allowed(!park.loans_allowed()),
            Command::Arrivals => park
                .set_visitor_arrivals(!park.visitor_flow_stats().enabled)
                .map_err(|e| e.to_string())?,
            Command::Ride(i) => {
                if let Some(model) = park_sim::RideModel::ALL.get(i) {
                    park.set_content_available(
                        Some(*model),
                        None,
                        !park.ride_content_available(*model),
                    );
                }
            }
            Command::Facility(i) => {
                if let Some(kind) = park_sim::ServiceKind::ALL.get(i) {
                    park.set_content_available(
                        None,
                        Some(*kind),
                        !park.facility_content_available(*kind),
                    );
                }
            }
            Command::Test => {
                if !w.editing {
                    return Err("Create or edit a scenario first".into());
                }
                let doc = ScenarioDocument::from_park(park, w.definition.clone())?;
                let test = doc.play()?;
                w.test_draft = Some(park.clone());
                *park = test;
                view.front_new_park = true;
                view.scenario_from_title = false;
                w.editing = false;
                view.scene_dirty = true;
                view.set_modal(false, false, false, false);
                view.speed = park_sim::Speed::Normal;
            }
            Command::ReturnEditor => {
                if let Some(draft) = w.test_draft.take() {
                    *park = draft;
                    w.editing = true;
                    w.tab = 1;
                    view.pause();
                    view.scene_dirty = true;
                }
            }
            Command::Designs => {
                view.set_modal(false, false, false, false);
                crate::track_tools::begin(park, view);
                view.modal_input_blocked = false;
                crate::design_library::request_open(view);
            }
        }
        Ok(())
    })();
    if let Err(e) = result {
        w.notice = e;
    }
}
pub(crate) fn buttons(
    mut w: ResMut<Workshop>,
    mut sim: ResMut<Simulation>,
    mut view: ResMut<ViewState>,
    session: Res<SaveSession>,
    keys: Res<ButtonInput<KeyCode>>,
    actions: Query<(&Interaction, &Action), Changed<Interaction>>,
) {
    if w.generation != view.park_generation {
        w.original = None;
        w.test_draft = None;
        w.editing = false;
        w.generation = view.park_generation;
    }
    if !view.workshop_open {
        return;
    }
    let store = match store(&session) {
        Ok(s) => s,
        Err(e) => {
            w.notice = e;
            return;
        }
    };
    if w.entries.is_empty() {
        w.refresh(&store);
    }
    if keys.just_pressed(KeyCode::F9) || keys.just_pressed(KeyCode::Escape) {
        return;
    }
    for (i, a) in &actions {
        if *i == Interaction::Pressed
            && let Action::Workshop(command) = a
        {
            command_action(&mut sim.0, &mut view, &mut w, &store, *command);
            break;
        }
    }
}
fn command_action(
    park: &mut park_sim::Park,
    view: &mut ViewState,
    w: &mut Workshop,
    store: &ScenarioStore,
    c: Command,
) {
    command(park, view, w, store, c)
}
pub(crate) fn keyboard_text(
    mut w: ResMut<Workshop>,
    view: Res<ViewState>,
    mut events: MessageReader<KeyboardInput>,
) {
    if !view.workshop_open {
        return;
    }
    let field = w.field;
    let (target, max) = match field {
        Field::Name => (&mut w.definition.name, 40),
        Field::Description => (&mut w.definition.description, 240),
        Field::Search => (&mut w.search, 80),
    };
    for event in events.read().filter(|e| e.state == ButtonState::Pressed) {
        match &event.logical_key {
            Key::Backspace => {
                target.pop();
            }
            Key::Enter => {}
            _ => {
                if let Some(text) = &event.text {
                    for c in text.chars().filter(|c| !c.is_control()) {
                        if target.chars().count() < max {
                            target.push(c);
                        }
                    }
                }
            }
        }
    }
    if matches!(field, Field::Search) {
        w.page = 0;
        if w.selected
            .is_some_and(|id| w.visible().iter().all(|e| e.id != id))
        {
            w.selected = None;
        }
    }
}
pub(crate) fn transfer(
    _thread: NonSend<crate::design_transfer::DialogThread>,
    mut input: crate::design_transfer::DialogInput,
    mut w: ResMut<Workshop>,
    session: Res<SaveSession>,
    mut view: ResMut<ViewState>,
) {
    let Some(export) = w.transfer.take() else {
        return;
    };
    if !view.workshop_open || view.capture_active {
        return;
    }
    let result = (|| {
        let store = store(&session)?;
        let request = if export {
            DialogRequest::ScenarioExport {
                suggested_name: w.definition.name.clone(),
            }
        } else {
            DialogRequest::ScenarioImport
        };
        let Some(path) = input.choose(request)? else {
            return Ok("File choice cancelled.".into());
        };
        if export {
            let doc = store.load(w.selected.ok_or("Select a saved scenario")?)?;
            crate::scenario_store::export_document(&path, &doc)?;
            Ok("Scenario exported.".into())
        } else {
            let doc = crate::scenario_store::read_document(&path)?;
            w.definition = doc.definition.clone();
            w.selected = Some(store.save(None, &doc)?);
            w.refresh(&store);
            Ok("Scenario imported. Select Play or Edit copy.".into())
        }
    })();
    w.notice = result.unwrap_or_else(|e| e);
    view.ui_pointer_consumed = true;
    view.modal_transition_pending = true;
}
fn control(p: &mut ChildSpawnerCommands, label: &str, c: Command) {
    button(p, label, Action::Workshop(c), 154.0);
}
fn controls(p: &mut ChildSpawnerCommands, section: u8, items: &[(&str, Command)]) {
    p.spawn((
        Section(section),
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(5),
            row_gap: px(5),
            ..row()
        },
    ))
    .with_children(|p| {
        for (name, command) in items {
            control(p, name, *command);
        }
    });
}
pub(crate) fn setup(commands: &mut Commands) {
    commands.spawn((
        Banner,
        Text::new(""),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(INK),
        BackgroundColor(PAPER),
        GlobalZIndex(5),
        Node {
            position_type: PositionType::Absolute,
            left: px(SIDEBAR + 12.0),
            top: px(HEADER + 37.0),
            max_width: px(690.0),
            padding: UiRect::all(px(6)),
            ..default()
        },
    ));
    commands
        .spawn((
            Panel,
            GlobalZIndex(15),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.1, 0.07, 0.7)),
        ))
        .with_children(|p| {
            p.spawn((
                layout::ScrollLayer::Workshop,
                ScrollPosition::default(),
                layout::dialog(94.0, 1050.0, Some(620.0)),
                BackgroundColor(PAPER),
                BorderRadius::all(px(10)),
            ))
            .with_children(|p| {
                p.spawn(row()).with_children(|p| {
                    label(p, "Scenarios and editor", 24.0, INK);
                    control(p, "Close [Esc]", Command::Close);
                });
                p.spawn(row()).with_children(|p| {
                    for (name, c) in [
                        ("Scenario library", Command::Tab(0)),
                        ("Goals and rules", Command::Tab(1)),
                        ("Available content", Command::Tab(2)),
                    ] {
                        control(p, name, c);
                    }
                });
                p.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(INK),
                    Body,
                ));
                controls(
                    p,
                    0,
                    &[
                        ("Search", Command::Search),
                        ("Clear search", Command::ClearSearch),
                        ("Refresh", Command::Refresh),
                        ("Previous", Command::Previous),
                        ("Next", Command::Next),
                    ],
                );
                for i in 0..5 {
                    p.spawn((
                        Button,
                        Action::Workshop(Command::Select(i)),
                        Row(i),
                        Node {
                            min_height: px(30),
                            padding: UiRect::all(px(6)),
                            ..default()
                        },
                        BackgroundColor(BUTTON),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(INK),
                            Row(i),
                        ));
                    });
                }
                controls(
                    p,
                    0,
                    &[
                        ("New from this park", Command::New),
                        ("Edit selected", Command::Edit),
                        ("Play selected", Command::Play),
                        ("Edit name", Command::Name),
                        ("Rename selected", Command::Rename),
                        ("Archive selected", Command::Delete),
                        ("Import", Command::Import),
                        ("Export", Command::Export),
                        ("Track designs", Command::Designs),
                        ("Return to editor", Command::ReturnEditor),
                    ],
                );
                controls(
                    p,
                    3,
                    &[
                        ("Build park", Command::Build),
                        ("Save scenario", Command::Save),
                        ("Test scenario", Command::Test),
                        ("Edit name", Command::Name),
                        ("Edit description", Command::Description),
                        ("Return to park", Command::ReturnPark),
                    ],
                );
                controls(
                    p,
                    4,
                    &[
                        ("Confirm action", Command::Confirm),
                        ("Cancel action", Command::Cancel),
                    ],
                );
                controls(
                    p,
                    1,
                    &[
                        ("Add goal", Command::AddGoal),
                        ("Remove goal", Command::RemoveGoal),
                        ("Cycle metric", Command::Metric),
                        ("Target -", Command::Target(false)),
                        ("Target +", Command::Target(true)),
                        ("Deadline", Command::Deadline),
                        ("Charging rule", Command::Charging),
                        ("Budget - $500", Command::Funds(false)),
                        ("Budget + $500", Command::Funds(true)),
                        ("Research mode", Command::Research),
                        ("Loans", Command::Loans),
                        ("Visitor flow", Command::Arrivals),
                    ],
                );
                controls(
                    p,
                    1,
                    &[
                        ("Select goal 1", Command::Goal(0)),
                        ("Select goal 2", Command::Goal(1)),
                        ("Select goal 3", Command::Goal(2)),
                        ("Select goal 4", Command::Goal(3)),
                    ],
                );
                p.spawn((
                    Section(2),
                    Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(5),
                        row_gap: px(5),
                        ..row()
                    },
                ))
                .with_children(|p| {
                    for (i, model) in park_sim::RideModel::ALL.iter().enumerate() {
                        button(p, model.name(), Action::Workshop(Command::Ride(i)), 240.0);
                    }
                    for (i, kind) in park_sim::ServiceKind::ALL.iter().enumerate() {
                        control(p, &kind.to_string(), Command::Facility(i));
                    }
                });
            });
        });
}
type WorkshopNodes<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static Panel>,
        Option<&'static Row>,
        Option<&'static Section>,
        Option<&'static Banner>,
    ),
    Or<(With<Panel>, With<Row>, With<Section>, With<Banner>)>,
>;
type WorkshopText<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Text,
        Option<&'static Body>,
        Option<&'static Row>,
        Option<&'static Banner>,
    ),
    Or<(With<Body>, With<Row>, With<Banner>)>,
>;
pub(crate) fn refresh(
    sim: Res<Simulation>,
    w: Res<Workshop>,
    view: Res<ViewState>,
    mut nodes: WorkshopNodes,
    mut texts: WorkshopText,
) {
    let park = &sim.0;
    let status = park.challenge_status();
    let visible = w.visible();
    let banner_text = if w.editing {
        "Scenario editor · Build normally, then reopen Scenarios and editor to save or test.".into()
    } else {
        status.as_ref().map_or(String::new(), |s| {
            format!(
                "{} · {} · {}",
                s.definition.name,
                if s.completed_tick.is_some() {
                    "Completed"
                } else if s.failed_tick.is_some() {
                    "Deadline missed"
                } else {
                    "In progress"
                },
                s.values
                    .iter()
                    .zip(&s.definition.goals)
                    .map(|(v, g)| format!("{} {}/{}", g.metric.label(), v, g.target))
                    .collect::<Vec<_>>()
                    .join(" / ")
            )
        })
    };
    for (mut node, panel, row, section, banner) in &mut nodes {
        let show = if panel.is_some() {
            view.workshop_open
        } else if banner.is_some() {
            !view.front_modal && !view.workshop_open && !banner_text.is_empty()
        } else if let Some(section) = section {
            match section.0 {
                3 => w.editing,
                4 => w.pending.is_some(),
                0 => w.tab == 0,
                _ => w.editing && section.0 == w.tab,
            }
        } else {
            view.workshop_open
                && w.tab == 0
                && row.is_some_and(|r| visible.get(w.page * 5 + r.0).is_some())
        };
        node.display = if show { Display::Flex } else { Display::None };
    }
    let d = &w.definition;
    let goals = d
        .goals
        .iter()
        .enumerate()
        .map(|(i, g)| {
            format!(
                "{}{} ≥ {}",
                if i == w.goal { "> " } else { "" },
                g.metric.label(),
                g.target
            )
        })
        .collect::<Vec<_>>()
        .join(" / ");
    let pending = match w.pending {
        Some(Pending::Play(_)) => {
            "Confirm replaces the current unsaved park and starts this scenario."
        }
        Some(Pending::Edit(_)) => {
            "Confirm replaces unsaved editor changes with the selected template."
        }
        Some(Pending::Delete(_)) => "Confirm moves this scenario to its recoverable archive.",
        Some(Pending::Rename(_)) => "Confirm applies the name field to the selected scenario.",
        Some(Pending::ReturnPark) => {
            "Confirm restores the original park and discards unsaved editor/test changes."
        }
        None => "",
    };
    let mode = match w.field {
        Field::Name => "name",
        Field::Description => "description",
        Field::Search => "search",
    };
    let settings = if w.tab == 0 {
        format!(
            "Search [{}] · {} scenarios · Page {}\n{}",
            w.search,
            visible.len(),
            w.page + 1,
            w.preview_summary
        )
    } else {
        format!(
            "{} × {} · Budget {} · Cash {} · Research {} · Loans {} · Arrivals {}",
            park.width(),
            park.height(),
            money(park.starting_cash_cents()),
            money(park.cash_cents()),
            if park.all_research_available() {
                "all"
            } else {
                "funded"
            },
            park.loans_allowed(),
            park.visitor_flow_stats().enabled
        )
    };
    let blocked = if w.tab == 2 {
        format!(
            "Unavailable rides [{}]\nUnavailable facilities [{}]",
            park_sim::RideModel::ALL
                .iter()
                .filter(|m| !park.ride_content_available(**m))
                .map(|m| m.name())
                .collect::<Vec<_>>()
                .join(", "),
            park_sim::ServiceKind::ALL
                .iter()
                .filter(|k| !park.facility_content_available(**k))
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        String::new()
    };
    let body = format!(
        "{}\n{}\nName [{}] · Typing edits {}\n{}\n{}\nDeadline {} · Charging {}\n{}\n{}",
        if w.editing {
            "Scenario editor"
        } else {
            "Scenario library"
        },
        w.notice,
        d.name,
        mode,
        d.description,
        goals,
        d.deadline_ticks
            .map_or("none".into(), |t| format!("{} seconds", t / 30)),
        d.charging_rule.label(),
        settings,
        if pending.is_empty() {
            &blocked
        } else {
            pending
        }
    );
    for (mut text, body_tag, row, banner) in &mut texts {
        if body_tag.is_some() {
            text.0 = body.clone();
        } else if banner.is_some() {
            text.0 = banner_text.clone();
        } else if let Some(row) = row {
            text.0 = visible.get(w.page * 5 + row.0).map_or(String::new(), |e| {
                format!(
                    "{}{} · {}",
                    if w.selected == Some(e.id) { "> " } else { "" },
                    e.name,
                    e.error.as_ref().unwrap_or(&e.summary)
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "scenario-workshop-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    fn setup_system(mut commands: Commands) {
        setup(&mut commands);
    }
    #[test]
    fn editor_saves_tests_restores_and_preserves_original_park() {
        let root = root();
        let store = ScenarioStore::new(&root);
        let mut w = Workshop::default();
        let mut view = ViewState::default();
        let mut park = park_sim::Park::new(32, 32);
        let original = park.clone();
        open(&mut view);
        command(&mut park, &mut view, &mut w, &store, Command::New);
        command(&mut park, &mut view, &mut w, &store, Command::Funds(true));
        assert_eq!(park.starting_cash_cents(), 150_000);
        command(&mut park, &mut view, &mut w, &store, Command::Save);
        assert_eq!(store.list().unwrap().len(), 1);
        command(&mut park, &mut view, &mut w, &store, Command::Test);
        assert!(park.challenge_status().is_some());
        for _ in 0..50 {
            park.advance(park_sim::Speed::Normal);
        }
        open(&mut view);
        command(&mut park, &mut view, &mut w, &store, Command::ReturnEditor);
        assert_eq!(park.ticks(), 0);
        assert_eq!(park.starting_cash_cents(), 150_000);
        command(&mut park, &mut view, &mut w, &store, Command::ReturnPark);
        command(&mut park, &mut view, &mut w, &store, Command::Confirm);
        assert_eq!(park, original);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn editor_refresh_queries_and_modal_guards_work() {
        let mut app = App::new();
        let mut view = ViewState::default();
        open(&mut view);
        app.insert_resource(view)
            .insert_resource(Simulation(park_sim::Park::new(32, 32)))
            .init_resource::<Workshop>()
            .add_systems(Startup, setup_system)
            .add_systems(Update, refresh);
        app.update();
        let world = app.world_mut();
        let mut text = world.query_filtered::<&Text, With<Body>>();
        assert!(text.iter(world).all(|t| t.0.contains("Scenario library")));
    }
}

pub(crate) fn prepare_capture(
    w: &mut Workshop,
    park: &mut park_sim::Park,
    view: &mut ViewState,
    editing: bool,
) {
    open(view);
    w.definition = ChallengeDefinition {
        name: "Lakeside Adventure".into(),
        description: "Welcome families to a scenic park and develop new attractions.".into(),
        ..Default::default()
    };
    w.editing = editing;
    w.tab = if editing { 1 } else { 0 };
    w.original = Some(park.clone());
    w.generation = view.park_generation;
    w.notice =
        "Scenario template includes the current map, rides, facilities, land rights and budget."
            .into();
}
