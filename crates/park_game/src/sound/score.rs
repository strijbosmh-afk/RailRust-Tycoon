//! Three original arrangements with evolving sections and overlapping endings.
use super::synth::{RATE, osc};

pub(super) const TITLES: [&str; 3] = ["Sunhaven Parade", "Lakeside Swing", "Starlight Carnival"];
const TEMPOS: [u64; 3] = [112, 104, 124];
const CROSSFADE: u64 = RATE as u64 * 3;
const MELODIES: [[i8; 32]; 3] = [
    [
        0, 4, 7, 12, 11, 7, 4, 2, 0, 4, 9, 7, 5, 4, 2, -1, 4, 7, 12, 14, 12, 9, 7, 4, 5, 9, 7, 5,
        4, 2, 0, -1,
    ],
    [
        7, -1, 9, 7, 4, 2, 0, 2, 4, 7, 9, -1, 7, 4, 2, -1, 9, 12, 14, 12, 9, 7, 4, 2, 5, 9, 7, 4,
        2, 4, 0, -1,
    ],
    [
        0, 0, 7, 12, 9, 7, 5, 4, 2, 4, 7, 9, 7, 4, 2, -1, 4, 7, 9, 12, 14, 12, 9, 7, 5, 7, 9, 5, 4,
        2, 0, -1,
    ],
];
const CHORDS: [[usize; 3]; 8] = [
    [0, 4, 7],
    [9, 12, 16],
    [5, 9, 12],
    [7, 11, 14],
    [2, 5, 9],
    [5, 9, 12],
    [7, 11, 14],
    [0, 4, 7],
];

pub(super) fn track_frames(track: usize) -> u64 {
    u64::from(RATE) * 60 * 128 / TEMPOS[track] - CROSSFADE
}

pub(super) fn position(sample: u64) -> (usize, u64) {
    let duration: u64 = (0..3).map(track_frames).sum();
    let mut remaining = sample % duration;
    for track in 0..3 {
        if remaining < track_frames(track) {
            return (track, remaining);
        }
        remaining -= track_frames(track);
    }
    unreachable!("playlist duration includes every track")
}

pub(super) struct Score {
    frequencies: [f32; 128],
    delay: Vec<[f32; 2]>,
    delay_index: usize,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            frequencies: std::array::from_fn(|note| {
                440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
            }),
            delay: vec![[0.0; 2]; RATE as usize / 4],
            delay_index: 0,
        }
    }
}

impl Score {
    pub(super) fn sample(&mut self, sample: u64, noise: f32) -> [f32; 2] {
        let (track, local) = position(sample);
        let variation = sample / (0..3).map(track_frames).sum::<u64>();
        let mut sound = self.arrangement(track, local + CROSSFADE, variation, noise);
        let remaining = track_frames(track) - local;
        if remaining < CROSSFADE {
            let blend = 1.0 - remaining as f32 / CROSSFADE as f32;
            let next = self.arrangement((track + 1) % 3, CROSSFADE - remaining, variation, noise);
            for channel in 0..2 {
                sound[channel] = sound[channel] * (1.0 - blend) + next[channel] * blend;
            }
        }
        let echo = self.delay[self.delay_index];
        self.delay[self.delay_index] = [
            sound[0] * 0.18 + echo[1] * 0.26,
            sound[1] * 0.18 + echo[0] * 0.26,
        ];
        self.delay_index = (self.delay_index + 1) % self.delay.len();
        [sound[0] + echo[0], sound[1] + echo[1]]
    }

    fn arrangement(&self, track: usize, sample: u64, variation: u64, noise: f32) -> [f32; 2] {
        let beat = sample as f64 * TEMPOS[track] as f64 / (f64::from(RATE) * 60.0);
        let bar = beat as usize / 4;
        let section = bar / 4 % 8;
        let key = [0, 2, 5][track];
        let chord = CHORDS[(bar / 2 + [0, 4, 2][track]) % 8];
        let swing = if track == 1 { 0.61 } else { 0.5 };
        let offbeat = beat.fract() >= swing;
        let eighth = beat as usize * 2 + usize::from(offbeat);
        let eighth_age = ((beat.fract() - if offbeat { swing } else { 0.0 }) * 60.0
            / TEMPOS[track] as f64) as f32;
        let beat_age = (beat.fract() * 60.0 / TEMPOS[track] as f64) as f32;
        let motif_index = (eighth
            + if variation % 2 == 1 && section == 6 {
                16
            } else {
                0
            })
            % 32;
        let interval = MELODIES[track][motif_index];
        let chorus = matches!(section, 4 | 5 | 7);
        let sparse = section == 3;
        let lead = if interval >= 0 && (!sparse || eighth.is_multiple_of(2)) {
            let note = (72 + key) as i32 + i32::from(interval) + if chorus { 12 } else { 0 };
            self.instrument(sample, note as usize, eighth_age, track)
                * if section == 0 { 0.65 } else { 1.0 }
        } else {
            0.0
        };

        let bass_note = 36 + key + chord[if (beat as usize % 4) == 2 { 2 } else { 0 }];
        let bass_env = attack(beat_age, 0.008) * (-beat_age * 5.0).exp();
        let bass = (osc(sample, self.frequencies[bass_note])
            + osc(sample, self.frequencies[bass_note] * 2.0) * 0.18)
            * bass_env
            * 0.072;

        let bar_age = (beat % 4.0 * 60.0 / TEMPOS[track] as f64) as f32;
        let bar_length = 240.0 / TEMPOS[track] as f32;
        let pad_env = attack(bar_age, 0.07) * ((bar_length - bar_age) / 0.12).clamp(0.0, 1.0);
        let pad = chord
            .iter()
            .map(|note| osc(sample, self.frequencies[48 + key + note]))
            .sum::<f32>()
            * pad_env
            * 0.009;
        let rhythm = chord
            .iter()
            .map(|note| self.instrument(sample, 60 + key + note, eighth_age, 1))
            .sum::<f32>()
            * if offbeat && !sparse { 0.18 } else { 0.04 };
        let answer = if chorus && offbeat {
            self.instrument(sample, 72 + key + chord[(eighth / 2) % 3], eighth_age, 2) * 0.27
        } else {
            0.0
        };

        let kick = if (beat as usize).is_multiple_of(2) || (track == 2 && !sparse) {
            let phase = 46.0 * beat_age + 6.0 * (1.0 - (-beat_age * 24.0).exp());
            (phase * std::f32::consts::TAU).sin()
                * (-beat_age * 16.0).exp()
                * attack(beat_age, 0.002)
                * 0.085
        } else {
            0.0
        };
        let snare = if (beat as usize) % 2 == 1 {
            (noise * 0.7 + osc(sample, 180.0) * 0.3)
                * (-beat_age * 24.0).exp()
                * attack(beat_age, 0.002)
                * 0.05
        } else {
            0.0
        };
        let hat = noise
            * (-eighth_age * 85.0).exp()
            * attack(eighth_age, 0.001)
            * if track == 1 { 0.010 } else { 0.018 };
        let fill = if bar % 8 == 7 && beat % 4.0 > 3.0 {
            let age = ((beat * 4.0).fract() * 15.0 / TEMPOS[track] as f64) as f32;
            noise * (-age * 60.0).exp() * 0.015
        } else {
            0.0
        };
        let drums = (kick + snare + hat + fill) * if sparse { 0.45 } else { 1.0 };
        [
            lead * 0.87 + rhythm * 0.65 + bass + pad + drums + answer * 0.55,
            lead * 0.67 + rhythm + bass + pad * 0.90 + drums * 0.95 + answer,
        ]
    }

    fn instrument(&self, sample: u64, note: usize, age: f32, kind: usize) -> f32 {
        let f = self.frequencies[note];
        let (decay, harmonics) = match kind {
            0 => (7.5, [1.0, 0.32, 0.12]),
            1 => (10.0, [1.0, 0.10, 0.23]),
            _ => (5.2, [1.0, 0.36, 0.09]),
        };
        let body = osc(sample, f) * harmonics[0]
            + osc(sample, f * 2.0) * harmonics[1]
            + osc(sample, f * 3.0) * harmonics[2];
        body * attack(age, 0.008) * (-age * decay).exp() * 0.085
    }
}

fn attack(age: f32, seconds: f32) -> f32 {
    (age / seconds).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playlist_visits_three_long_tracks_and_wraps_without_a_gap() {
        let mut start = 0;
        for track in 0..3 {
            assert!(track_frames(track) > u64::from(RATE) * 55);
            assert_eq!(position(start), (track, 0));
            start += track_frames(track);
            assert_eq!(position(start - 1).0, track);
        }
        assert_eq!(position(start), (0, 0));
    }

    #[test]
    fn arrangements_have_distinct_stereo_pcm_and_bounded_transitions() {
        let scores = [Score::default(), Score::default(), Score::default()];
        let beginnings: Vec<Vec<[f32; 2]>> = (0..3)
            .map(|track| {
                (0..2000)
                    .map(|sample| scores[track].arrangement(track, sample, 0, 0.2))
                    .collect()
            })
            .collect();
        assert_ne!(beginnings[0], beginnings[1]);
        assert_ne!(beginnings[1], beginnings[2]);
        assert!(beginnings.iter().flatten().any(|s| s[0] != s[1]));
        for boundary in [track_frames(0), track_frames(0) + track_frames(1)] {
            let mut score = Score::default();
            let mut previous = score.sample(boundary - 1000, 0.0);
            for sample in boundary - 999..boundary + 1000 {
                let next = score.sample(sample, 0.0);
                assert!(next.iter().all(|v| v.is_finite() && v.abs() < 0.8));
                assert!((next[0] - previous[0]).abs() < 0.12);
                previous = next;
            }
        }
    }
}
