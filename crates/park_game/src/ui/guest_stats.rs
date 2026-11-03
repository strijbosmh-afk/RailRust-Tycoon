//! Read-only, park-wide visitor snapshot derived from active guest state.
use super::*;
use park_sim::{GuestState, Park, RideVisitPhase};
use std::collections::VecDeque;

const SAMPLE_TICKS: u64 = park_sim::TICKS_PER_SECOND * 30;
const HISTORY_SAMPLES: usize = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TrendSample {
    tick: u64,
    satisfaction: u8,
    urgent_percent: u8,
}

#[derive(Resource, Default)]
pub(crate) struct GuestStatsHistory {
    generation: u64,
    samples: VecDeque<TrendSample>,
}

impl GuestStatsHistory {
    pub(crate) fn observe(&mut self, park: &Park, generation: u64) {
        let tick = park.ticks();
        if self.generation != generation || self.samples.back().is_some_and(|last| tick < last.tick)
        {
            self.samples.clear();
            self.generation = generation;
        }
        if self
            .samples
            .back()
            .is_some_and(|last| tick.saturating_sub(last.tick) < SAMPLE_TICKS)
        {
            return;
        }
        let stats = Snapshot::from_park(park);
        let satisfaction = if stats.total == 0 {
            0
        } else {
            ((stats.satisfaction_sum + stats.total as u64 / 2) / stats.total as u64) as u8
        };
        let urgent_percent = stats.fraction(stats.urgent_any).round() as u8;
        if self.samples.len() == HISTORY_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(TrendSample {
            tick,
            satisfaction,
            urgent_percent,
        });
    }
}

pub(crate) fn sample_history(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    mut history: ResMut<GuestStatsHistory>,
) {
    history.observe(&sim.0, view.park_generation);
}

#[derive(Component)]
pub(crate) struct GuestStatsBody;

#[derive(Component, Clone, Copy)]
pub(crate) enum StatLabel {
    Headline,
    Mood(usize),
    Need(usize),
    Activity,
    TrendSummary,
}

#[derive(Component, Clone, Copy)]
pub(crate) enum StatBar {
    Mood(usize),
    Need(usize),
    TrendSatisfaction(usize),
    TrendUrgent(usize),
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Snapshot {
    total: usize,
    satisfaction_sum: u64,
    mood: [usize; 3],
    urgent: [usize; 3],
    urgent_any: usize,
    riding: usize,
    queueing: usize,
    services: usize,
    lost: usize,
}

impl Snapshot {
    fn from_park(park: &Park) -> Self {
        let mut stats = Self::default();
        for guest in park.guests() {
            stats.total += 1;
            stats.satisfaction_sum += u64::from(guest.satisfaction);
            stats.mood
                [usize::from(guest.satisfaction >= 40) + usize::from(guest.satisfaction >= 70)] +=
                1;
            let urgent = [
                guest.needs.hunger >= 70,
                guest.needs.thirst >= 70,
                guest.needs.toilet >= 70,
            ];
            for (count, flagged) in stats.urgent.iter_mut().zip(urgent) {
                *count += usize::from(flagged);
            }
            stats.urgent_any += usize::from(urgent.contains(&true));
            stats.riding += usize::from(
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|v| v.phase == RideVisitPhase::Riding),
            );
            stats.queueing += usize::from(
                guest
                    .ride_visit
                    .as_ref()
                    .is_some_and(|v| v.phase == RideVisitPhase::Queueing),
            );
            stats.services += usize::from(guest.service_visit.is_some());
            stats.lost += usize::from(guest.state == GuestState::Lost);
        }
        stats
    }

    fn fraction(&self, count: usize) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            100.0 * count as f32 / self.total as f32
        }
    }
}

fn text(p: &mut ChildSpawnerCommands, initial: &str, size: f32, kind: StatLabel) {
    p.spawn((
        Text::new(initial),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(INK),
        kind,
    ));
}

fn stat_row(p: &mut ChildSpawnerCommands, kind: StatBar, color: Color) {
    p.spawn(Node {
        height: px(34),
        align_items: AlignItems::Center,
        column_gap: px(7),
        ..row()
    })
    .with_children(|p| {
        p.spawn(Node {
            width: px(130),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|p| {
            text(
                p,
                "",
                11.0,
                match kind {
                    StatBar::Mood(i) => StatLabel::Mood(i),
                    StatBar::Need(i) => StatLabel::Need(i),
                    StatBar::TrendSatisfaction(_) | StatBar::TrendUrgent(_) => unreachable!(),
                },
            )
        });
        p.spawn((
            Node {
                width: px(160),
                height: px(13),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(217, 225, 210)),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| {
            p.spawn((
                kind,
                Node {
                    width: percent(0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(color),
                BorderRadius::all(px(6)),
            ));
        });
    });
}

pub(crate) fn setup(p: &mut ChildSpawnerCommands) {
    p.spawn((
        GuestStatsBody,
        Node {
            display: Display::None,
            height: px(350),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(11),
            ..default()
        },
    ))
    .with_children(|p| {
        text(p, "Park-wide visitor snapshot", 18.0, StatLabel::Headline);
        p.spawn(Node {
            column_gap: px(12),
            ..row()
        })
        .with_children(|p| {
            p.spawn((
                Node {
                    width: px(344),
                    padding: UiRect::all(px(10)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(7),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(232, 238, 224)),
                BorderRadius::all(px(6)),
            ))
            .with_children(|p| {
                label(p, "SATISFACTION", 12.0, INK);
                for (index, color) in [
                    Color::srgb_u8(196, 75, 61),
                    Color::srgb_u8(215, 153, 52),
                    Color::srgb_u8(70, 148, 108),
                ]
                .into_iter()
                .enumerate()
                {
                    stat_row(p, StatBar::Mood(index), color);
                }
                label(p, "Low / mixed / high, out of active guests", 10.0, MUTED);
            });
            p.spawn((
                Node {
                    width: px(344),
                    padding: UiRect::all(px(10)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(7),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(232, 238, 224)),
                BorderRadius::all(px(6)),
            ))
            .with_children(|p| {
                label(p, "URGENT NEEDS", 12.0, INK);
                for index in 0..3 {
                    stat_row(p, StatBar::Need(index), Color::srgb_u8(196, 75, 61));
                }
                label(p, "70+ urgency; one guest may need more", 10.0, MUTED);
            });
        });
        text(p, "", 12.0, StatLabel::Activity);
        p.spawn((
            Node {
                height: px(88),
                padding: UiRect::axes(px(10), px(6)),
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(232, 238, 224)),
            BorderRadius::all(px(6)),
        ))
        .with_children(|p| {
            text(p, "", 11.0, StatLabel::TrendSummary);
            p.spawn(Node {
                height: px(55),
                align_items: AlignItems::End,
                column_gap: px(4),
                ..row()
            })
            .with_children(|p| {
                for index in 0..HISTORY_SAMPLES {
                    p.spawn(Node {
                        width: px(24),
                        height: percent(100),
                        align_items: AlignItems::End,
                        column_gap: px(2),
                        ..row()
                    })
                    .with_children(|p| {
                        for (kind, color) in [
                            (
                                StatBar::TrendSatisfaction(index),
                                Color::srgb_u8(70, 148, 108),
                            ),
                            (StatBar::TrendUrgent(index), Color::srgb_u8(196, 75, 61)),
                        ] {
                            p.spawn((
                                kind,
                                Node {
                                    width: px(10),
                                    height: percent(0),
                                    ..default()
                                },
                                BackgroundColor(color),
                            ));
                        }
                    });
                }
            });
        });
    });
}

pub(crate) fn refresh(
    sim: Res<Simulation>,
    view: Res<ViewState>,
    history: Res<GuestStatsHistory>,
    mut labels: Query<(&StatLabel, &mut Text)>,
    mut bars: Query<(&StatBar, &mut Node)>,
) {
    if !view.guests_open || !view.guest_stats_view {
        return;
    }
    let stats = Snapshot::from_park(&sim.0);
    let mood_names = ["Low 0-39", "Mixed 40-69", "High 70-100"];
    let need_names = ["Hunger", "Thirst", "Toilet"];
    for (kind, mut text) in &mut labels {
        text.0 = match kind {
            StatLabel::Headline => format!("Park-wide visitor snapshot / {} active", stats.total),
            StatLabel::Mood(i) => format!("{}  {}", mood_names[*i], stats.mood[*i]),
            StatLabel::Need(i) => format!("{}  {}", need_names[*i], stats.urgent[*i]),
            StatLabel::Activity => format!(
                "Currently {} guests have an urgent need.  {} riding / {} queueing / {} at services / {} lost.\nLive snapshot; choose Guest list to inspect someone.",
                stats.urgent_any, stats.riding, stats.queueing, stats.services, stats.lost,
            ),
            StatLabel::TrendSummary => {
                let latest = history.samples.back().copied();
                format!(
                    "30s history: {} / 24  |  satisfaction {} / 100 (green)  |  urgent {}% (red)",
                    history.samples.len(),
                    latest.map_or(0, |sample| sample.satisfaction),
                    latest.map_or(0, |sample| sample.urgent_percent),
                )
            }
        };
    }
    for (kind, mut node) in &mut bars {
        let count = match kind {
            StatBar::Mood(i) => stats.mood[*i],
            StatBar::Need(i) => stats.urgent[*i],
            StatBar::TrendSatisfaction(i) | StatBar::TrendUrgent(i) => {
                let start = HISTORY_SAMPLES - history.samples.len();
                let value = i
                    .checked_sub(start)
                    .and_then(|index| history.samples.get(index));
                node.height = percent(value.map_or(0.0, |sample| {
                    f32::from(match kind {
                        StatBar::TrendSatisfaction(_) => sample.satisfaction,
                        StatBar::TrendUrgent(_) => sample.urgent_percent,
                        _ => unreachable!(),
                    })
                }));
                continue;
            }
        };
        node.width = percent(stats.fraction(count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_accounts_for_every_active_guest_without_double_counting_mood() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(42).unwrap();
        let stats = Snapshot::from_park(&park);
        assert_eq!(stats.total, park.guests().len());
        assert_eq!(stats.mood.iter().sum::<usize>(), stats.total);
        assert!(stats.urgent.iter().all(|count| *count <= stats.total));
        assert_eq!(Snapshot::from_park(&Park::new(32, 32)).fraction(0), 0.0);
    }

    #[test]
    fn history_samples_on_cadence_is_bounded_and_resets_for_loaded_park() {
        let mut park = Park::new(32, 32);
        park.install_crowd_fixture(42).unwrap();
        let mut history = GuestStatsHistory::default();
        history.observe(&park, 0);
        assert_eq!(history.samples.len(), 1);
        let first = history.samples[0];
        for _ in 0..SAMPLE_TICKS - 1 {
            park.advance(park_sim::Speed::Normal);
        }
        history.observe(&park, 0);
        assert_eq!(history.samples.len(), 1);
        park.advance(park_sim::Speed::Normal);
        history.observe(&park, 0);
        assert_eq!(history.samples.len(), 2);
        assert_eq!(history.samples[0], first);
        while history.samples.len() < HISTORY_SAMPLES {
            history.samples.push_back(first);
        }
        history.observe(&park, 0);
        assert_eq!(history.samples.len(), HISTORY_SAMPLES);
        history.observe(&park, 1);
        assert_eq!(history.samples.len(), 1);
        assert_eq!(history.samples[0].tick, park.ticks());
        assert_eq!(history.generation, 1);
    }
}
