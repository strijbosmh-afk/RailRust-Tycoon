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
    pub carousel: bool,
}
impl Default for MixState {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            focused: true,
            paused: false,
            crowd: 0.3,
            coaster: 0.0,
            carousel: false,
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
            voices: Vec::new(),
        }
    }
}
fn sine(t: f32, f: f32) -> f32 {
    (TAU * t * f).sin()
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
            let t = self.sample as f32 / RATE as f32;
            self.noise = self.noise.wrapping_mul(1664525).wrapping_add(1013904223);
            let n = (self.noise >> 8) as f32 / 8388608.0 - 1.0;
            self.wind += (n - self.wind) * 0.02;
            // An original 32-note, 20-second meadow theme, with plucked melody
            // and an eight-bar bass/harmony. The soft attack prevents clicks.
            const MELODY: [i32; 32] = [
                72, 76, 79, 76, 74, 72, 69, 67, 72, 74, 76, 79, 81, 79, 76, 74, 69, 72, 76, 72, 67,
                69, 72, 74, 76, 74, 72, 69, 67, 71, 74, 79,
            ];
            let beat = t / 0.625;
            let note = MELODY[beat as usize % 32];
            let age = beat.fract() * 0.625;
            let env = (age * 60.0).min(1.0) * (-age * 7.0).exp();
            let f = frequency(note);
            let melody =
                (sine(t, f) + 0.24 * sine(t, f * 2.0) + 0.08 * sine(t, f * 3.0)) * env * 0.12;
            let bass = [48, 53, 57, 55][(beat as usize / 8) % 4];
            let b = frequency(bass);
            let music = melody + (sine(t, b) + 0.3 * sine(t, b * 1.5)) * 0.038;
            // Filter-free, very quiet wind/crowd texture and spaced bird chirps.
            let bird_age = (t + 2.0) % 7.0;
            let bird = if bird_age < 0.27 {
                sine(bird_age, 1800.0 + bird_age * 2100.0)
                    * (bird_age * std::f32::consts::PI / 0.27).sin()
                    * 0.045
            } else {
                0.0
            };
            let ambient = self.wind * (0.10 + self.state.crowd * 0.10)
                + sine(t, 143.0) * sine(t, 2.7) * self.state.crowd * 0.008
                + bird;
            let moving = if self.state.paused {
                0.0
            } else {
                self.state.coaster
            };
            let rail = n * 0.045 * moving + sine(t, 64.0 + moving * 40.0) * 0.03 * moving;
            let click = if (t * (7.0 + moving * 11.0)).fract() < 0.045 {
                n * 0.075 * moving
            } else {
                0.0
            };
            let carousel = if self.state.carousel && !self.state.paused {
                let f = frequency([72, 76, 79, 84, 79, 76, 74, 71][(t / 0.42) as usize % 8]);
                sine(t, f) * 0.045 * (1.0 - (t / 0.42).fract())
            } else {
                0.0
            };
            let rides = rail + click + carousel;
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
                    effects += sine(age, f) * env * amp;
                }
                v.age += 1;
            }
            self.voices.retain(|v| v.age < RATE / 2);
            // Soft bounded mix leaves headroom even with twelve simultaneous cues.
            let left = (music * self.gain[1]
                + ambient * self.gain[2]
                + rides * self.gain[3]
                + effects * self.gain[4])
                * self.gain[0];
            let right = (music * self.gain[1]
                + (ambient * 0.92 + bird * 0.08) * self.gain[2]
                + rides * self.gain[3]
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
}
