//! Original procedural score, ambience and effects. No recorded or licensed assets.
use super::Settings;
use std::f32::consts::TAU;
pub(super) const RATE: u32 = 44_100;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Cue {
    Click,
    Build,
    Remove,
    Sale,
    Alert,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MixState {
    pub settings: Settings,
    pub focused: bool,
    pub paused: bool,
    pub crowd: f32,
    pub coaster: f32,
    pub coaster_pan: f32,
    pub coaster_presence: f32,
    pub carousel: bool,
    pub carousel_pan: f32,
    pub carousel_presence: f32,
}
impl Default for MixState {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            focused: true,
            paused: false,
            crowd: 0.3,
            coaster: 0.0,
            coaster_pan: 0.0,
            coaster_presence: 1.0,
            carousel: false,
            carousel_pan: 0.0,
            carousel_presence: 1.0,
        }
    }
}
struct Voice {
    cue: Cue,
    age: u32,
}
pub(super) struct Mixer {
    pub state: MixState,
    sample: u64,
    noise: u32,
    wind: f32,
    gain: [f32; 5],
    ride_pan: [f32; 2],
    ride_presence: [f32; 2],
    voices: Vec<Voice>,
}
impl Default for Mixer {
    fn default() -> Self {
        Self {
            state: MixState::default(),
            sample: 0,
            noise: 0x7331a945,
            wind: 0.0,
            gain: [0.0; 5],
            ride_pan: [0.0; 2],
            ride_presence: [0.0; 2],
            voices: Vec::new(),
        }
    }
}
fn sine(t: f32, f: f32) -> f32 {
    (TAU * t * f).sin()
}
// Keep oscillator phase precise after hours of play. A global f32 time loses
// individual samples well before a long park session ends.
fn osc(sample: u64, frequency: f32) -> f32 {
    let cycles = sample as f64 * f64::from(frequency) / f64::from(RATE);
    (cycles.fract() * std::f64::consts::TAU).sin() as f32
}
fn frequency(note: i32) -> f32 {
    440.0 * 2.0f32.powf((note - 69) as f32 / 12.0)
}
impl Mixer {
    pub fn cue(&mut self, cue: Cue) {
        if self.voices.len() < 12 {
            self.voices.push(Voice { cue, age: 0 });
        }
    }
    pub fn render(&mut self, frames: usize) -> Vec<i16> {
        let mut out = Vec::with_capacity(frames * 2);
        let mut target = self.state.settings.levels.map(|v| f32::from(v) / 100.0);
        if !self.state.focused || self.state.settings.muted {
            target[0] = 0.0;
        }
        for _ in 0..frames {
            for (g, t) in self.gain.iter_mut().zip(target) {
                *g += (t - *g) * 0.001;
            }
            for (index, (pan, presence)) in [
                (self.state.coaster_pan, self.state.coaster_presence),
                (self.state.carousel_pan, self.state.carousel_presence),
            ]
            .into_iter()
            .enumerate()
            {
                self.ride_pan[index] += (pan.clamp(-1.0, 1.0) - self.ride_pan[index]) * 0.0007;
                self.ride_presence[index] +=
                    (presence.clamp(0.0, 1.0) - self.ride_presence[index]) * 0.0007;
            }
            let clock = self.sample as f64 / f64::from(RATE);
            self.noise = self.noise.wrapping_mul(1664525).wrapping_add(1013904223);
            let n = (self.noise >> 8) as f32 / 8388608.0 - 1.0;
            self.wind += (n - self.wind) * 0.02;
            // Two original twenty-second phrases make a full A/B theme.
            // Plucked attacks prevent clicks at each phrase boundary.
            const MELODY_A: [i32; 32] = [
                72, 76, 79, 76, 74, 72, 69, 67, 72, 74, 76, 79, 81, 79, 76, 74, 69, 72, 76, 72, 67,
                69, 72, 74, 76, 74, 72, 69, 67, 71, 74, 79,
            ];
            const MELODY_B: [i32; 32] = [
                79, 81, 84, 81, 79, 76, 74, 72, 76, 79, 81, 79, 76, 74, 72, 69, 74, 76, 79, 76, 72,
                74, 76, 72, 69, 72, 74, 76, 79, 76, 74, 72,
            ];
            let beat = clock / 0.625;
            let melody_notes = if (beat as usize / 32).is_multiple_of(2) {
                &MELODY_A
            } else {
                &MELODY_B
            };
            let note = melody_notes[beat as usize % 32];
            let age = beat.fract() as f32 * 0.625;
            let env = (age * 60.0).min(1.0) * (-age * 7.0).exp();
            let f = frequency(note);
            let melody = (osc(self.sample, f)
                + 0.24 * osc(self.sample, f * 2.0)
                + 0.08 * osc(self.sample, f * 3.0))
                * env
                * 0.12;
            let bass = [48, 53, 57, 55][(beat as usize / 8) % 4];
            let b = frequency(bass);
            let bass_age = (beat / 2.0).fract() as f32 * 1.25;
            let bass_env = (bass_age * 28.0).min(1.0) * (-bass_age * 2.3).exp();
            let bass_line =
                (osc(self.sample, b) + 0.22 * osc(self.sample, b * 2.0)) * bass_env * 0.038;
            // Four original harmony beds and a light answering arpeggio add
            // depth across both phrases of the forty-second theme.
            const CHORDS: [[i32; 3]; 4] = [[60, 64, 67], [57, 60, 65], [55, 59, 62], [59, 62, 67]];
            let bar = (clock / 5.0) as usize % CHORDS.len();
            let bar_age = (clock % 5.0) as f32;
            let pad_env = (bar_age / 0.35).min(1.0) * ((5.0 - bar_age) / 0.35).min(1.0);
            let pad = CHORDS[bar]
                .iter()
                .map(|note| osc(self.sample, frequency(*note)))
                .sum::<f32>()
                * pad_env
                * 0.008;
            let arpeggio_beat = clock / 0.3125;
            let arpeggio_age = arpeggio_beat.fract() as f32 * 0.3125;
            let arpeggio_note = CHORDS[bar][arpeggio_beat as usize % 3] + 12;
            let arpeggio = osc(self.sample, frequency(arpeggio_note))
                * (arpeggio_age * 80.0).min(1.0)
                * (-arpeggio_age * 13.0).exp()
                * (0.006 + self.state.crowd * 0.012);
            let music = melody + bass_line + pad + arpeggio;
            // Quiet smoothed wind/crowd texture and spaced bird chirps.
            let bird_age = ((clock + 2.0) % 7.0) as f32;
            let bird = if bird_age < 0.27 {
                sine(bird_age, 1800.0 + bird_age * 2100.0)
                    * (bird_age * std::f32::consts::PI / 0.27).sin()
                    * 0.045
            } else {
                0.0
            };
            let ambient = self.wind * (0.10 + self.state.crowd * 0.10)
                + osc(self.sample, 143.0) * osc(self.sample, 2.7) * self.state.crowd * 0.008
                + bird;
            let moving = if self.state.paused {
                0.0
            } else {
                self.state.coaster
            };
            let rail = n * 0.045 * moving + osc(self.sample, 64.0 + moving * 40.0) * 0.03 * moving;
            let click = if (clock * (7.0 + f64::from(moving) * 11.0)).fract() < 0.045 {
                n * 0.075 * moving
            } else {
                0.0
            };
            let carousel = if self.state.carousel && !self.state.paused {
                let f = frequency([72, 76, 79, 84, 79, 76, 74, 71][(clock / 0.42) as usize % 8]);
                (osc(self.sample, f) + 0.22 * osc(self.sample, f * 2.0))
                    * 0.039
                    * (1.0 - (clock / 0.42).fract() as f32)
            } else {
                0.0
            };
            let coaster = (rail + click) * self.ride_presence[0];
            let carousel = carousel * self.ride_presence[1];
            let mut effects = 0.0;
            for v in &mut self.voices {
                let age = v.age as f32 / RATE as f32;
                let (f, duration, amp) = match v.cue {
                    Cue::Click => (720.0, 0.055, 0.10),
                    Cue::Build => (420.0 + age * 1100.0, 0.18, 0.15),
                    Cue::Remove => (260.0 - age * 500.0, 0.16, 0.13),
                    Cue::Sale => (if age < 0.09 { 1046.5 } else { 1318.5 }, 0.22, 0.10),
                    Cue::Alert => (if age < 0.15 { 440.0 } else { 349.2 }, 0.35, 0.13),
                };
                if age < duration {
                    let env = (age * 120.0).min(1.0) * (1.0 - age / duration).powi(2);
                    let transient = match v.cue {
                        Cue::Build | Cue::Remove => n * (1.0 - age * 25.0).max(0.0) * 0.22,
                        _ => 0.0,
                    };
                    effects += (sine(age, f) + 0.18 * sine(age, f * 2.0) + transient) * env * amp;
                }
                v.age += 1;
            }
            self.voices.retain(|v| v.age < RATE / 2);
            // Soft bounded mix leaves headroom even with twelve simultaneous cues.
            let left_rides = coaster * (1.0 - self.ride_pan[0] * 0.6)
                + carousel * (1.0 - self.ride_pan[1] * 0.6);
            let right_rides = coaster * (1.0 + self.ride_pan[0] * 0.6)
                + carousel * (1.0 + self.ride_pan[1] * 0.6);
            let left = ((music + arpeggio * 0.18) * self.gain[1]
                + (ambient + bird * 0.12) * self.gain[2]
                + left_rides * self.gain[3]
                + effects * self.gain[4])
                * self.gain[0];
            let right = ((music - arpeggio * 0.18) * self.gain[1]
                + (ambient * 0.92 - bird * 0.08) * self.gain[2]
                + right_rides * self.gain[3]
                + effects * self.gain[4])
                * self.gain[0];
            out.extend([left, right].map(|s| (s.clamp(-0.9, 0.9) * 32767.0) as i16));
            self.sample += 1;
        }
        out
    }
}
pub(super) fn wav(samples: &[i16]) -> Vec<u8> {
    let size = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(size as usize + 44);
    bytes.extend(b"RIFF");
    bytes.extend((size + 36).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(2u16.to_le_bytes());
    bytes.extend(RATE.to_le_bytes());
    bytes.extend((RATE * 4).to_le_bytes());
    bytes.extend(4u16.to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(size.to_le_bytes());
    for s in samples {
        bytes.extend(s.to_le_bytes());
    }
    bytes
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oscillator_retains_individual_sample_phase_after_two_days() {
        let base = u64::from(RATE) * 60 * 60 * 48;
        assert!((osc(base + 1, 440.0) - osc(1, 440.0)).abs() < 0.00001);
        assert!((osc(base + 1, 440.0) - osc(base, 440.0)).abs() > 0.05);
    }
    #[test]
    fn original_mix_is_repeatable_bounded_and_each_bus_can_be_silenced() {
        let mut a = Mixer::default();
        let mut b = Mixer::default();
        a.state.coaster = 1.0;
        b.state.coaster = 1.0;
        for _ in 0..20 {
            a.cue(Cue::Alert);
            b.cue(Cue::Alert);
        }
        let x = a.render(RATE as usize);
        assert_eq!(x, b.render(RATE as usize));
        assert!(x.iter().any(|s| *s != 0));
        assert!(x.iter().all(|s| s.abs() < 30000));
        let mut silent = Mixer::default();
        silent.state.settings.levels = [0; 5];
        silent.cue(Cue::Build);
        assert!(silent.render(4096).iter().all(|s| *s == 0));
        let w = wav(&x);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(w.len(), x.len() * 2 + 44);
    }

    #[test]
    fn ride_pan_moves_energy_between_channels_and_presence_fades_far_rides() {
        fn energy(pan: f32, presence: f32) -> (i64, i64) {
            let mut mixer = Mixer::default();
            mixer.state.settings.levels = [100, 0, 0, 100, 0];
            mixer.state.coaster = 1.0;
            mixer.state.coaster_pan = pan;
            mixer.state.coaster_presence = presence;
            let pcm = mixer.render(RATE as usize / 2);
            pcm.chunks_exact(2).fold((0, 0), |(left, right), frame| {
                (
                    left + i64::from(frame[0]).abs(),
                    right + i64::from(frame[1]).abs(),
                )
            })
        }
        let (left, right) = energy(-1.0, 1.0);
        assert!(left > right * 2);
        let (left, right) = energy(1.0, 1.0);
        assert!(right > left * 2);
        assert_eq!(energy(1.0, 0.0), (0, 0));
    }
}
