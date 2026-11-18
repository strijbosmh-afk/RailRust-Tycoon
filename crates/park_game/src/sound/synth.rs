//! Stereo score, physical ride effects and licensed crowd reactions.
use super::{Settings, score::Score};
use park_sim::{RideId, RideModel};
use std::f32::consts::TAU;
pub(super) const RATE: u32 = 44_100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Cue {
    Click,
    Build,
    Remove,
    Sale,
    Alert,
    Cheer,
    Staff,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RideEmitter {
    pub id: (RideId, u8),
    pub model: RideModel,
    pub speed: f32,
    pub lift: f32,
    pub drop: f32,
    pub cycle: f32,
    pub pan: f32,
    pub presence: f32,
    pub occupied: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MixState {
    pub settings: Settings,
    pub focused: bool,
    pub paused: bool,
    pub crowd: f32,
    pub crowd_pan: f32,
    pub happiness: f32,
    pub rides: [Option<RideEmitter>; 4],
    pub rain: bool,
}
impl Default for MixState {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            focused: true,
            paused: false,
            crowd: 0.0,
            crowd_pan: 0.0,
            happiness: 0.7,
            rides: [None; 4],
            rain: false,
        }
    }
}
struct Voice {
    cue: Cue,
    age: u32,
    delay: u32,
    pan: f32,
    pitch: f32,
    volume: f32,
}

#[derive(Default)]
struct RideVoice {
    current: Option<RideEmitter>,
    gain: f32,
    pan: f32,
    speed: f32,
    lift: f32,
    drop: f32,
}

pub(super) struct Mixer {
    pub state: MixState,
    sample: u64,
    noise: u32,
    wind: f32,
    gain: [f32; 6],
    rides: [RideVoice; 4],
    score: Score,
    cheer: Vec<f32>,
    voices: Vec<Voice>,
    next_crowd: u64,
    dropping: bool,
}
impl Default for Mixer {
    fn default() -> Self {
        // Canonical mono 44.1 kHz PCM produced from the retained CC0 original.
        let bytes = include_bytes!("../../../../assets/audio/guest-cheer.wav");
        let cheer = bytes[44..]
            .chunks_exact(2)
            .map(|v| f32::from(i16::from_le_bytes([v[0], v[1]])) / 32768.0)
            .collect();
        Self {
            state: MixState::default(),
            sample: 0,
            noise: 0x7331a945,
            wind: 0.0,
            gain: [0.0; 6],
            rides: std::array::from_fn(|_| RideVoice::default()),
            score: Score::default(),
            cheer,
            voices: Vec::new(),
            next_crowd: u64::from(RATE) * 7,
            dropping: false,
        }
    }
}

fn sine(t: f32, f: f32) -> f32 {
    (TAU * t * f).sin()
}
pub(super) fn osc(sample: u64, frequency: f32) -> f32 {
    let cycles = sample as f64 * f64::from(frequency) / f64::from(RATE);
    (cycles.fract() * std::f64::consts::TAU).sin() as f32
}
fn stereo(value: f32, pan: f32) -> [f32; 2] {
    [value * (1.0 - pan * 0.65), value * (1.0 + pan * 0.65)]
}

impl Mixer {
    pub(super) fn start_track(&mut self, track: usize) {
        self.sample = (0..track.min(2)).map(super::score::track_frames).sum();
    }

    pub fn cue(&mut self, cue: Cue) {
        if cue == Cue::Cheer {
            let presence = self
                .state
                .rides
                .iter()
                .flatten()
                .map(|r| r.presence)
                .fold(self.state.crowd, f32::max);
            if presence > 0.015 {
                self.cheer_at(self.state.crowd_pan, presence);
            }
        } else if self.voices.len() < 12 {
            self.voices.push(Voice {
                cue,
                age: 0,
                delay: 0,
                pan: 0.0,
                pitch: 1.0,
                volume: 1.0,
            });
        }
    }

    fn cheer_at(&mut self, pan: f32, volume: f32) {
        for (delay, pitch, offset) in [(0, 0.93, -0.16), (2205, 1.08, 0.15), (5292, 1.0, 0.0)] {
            if self.voices.len() < 12 {
                self.voices.push(Voice {
                    cue: Cue::Cheer,
                    age: 0,
                    delay,
                    pitch,
                    pan: (pan + offset).clamp(-1.0, 1.0),
                    volume: volume * 0.15,
                });
            }
        }
    }

    pub fn render(&mut self, frames: usize) -> Vec<i16> {
        let mut out = Vec::with_capacity(frames * 2);
        let mut target = [0.0; 6];
        for (gain, value) in target.iter_mut().zip(self.state.settings.levels) {
            *gain = f32::from(value) / 100.0;
        }
        target[5] = f32::from(self.state.settings.reactions) / 100.0;
        if !self.state.focused || self.state.settings.muted {
            target[0] = 0.0;
        }
        let exciting = self
            .state
            .rides
            .iter()
            .flatten()
            .find(|ride| ride.occupied && ride.drop > 0.5 && ride.presence > 0.08)
            .copied();
        if let Some(ride) = exciting.filter(|_| !self.dropping && !self.state.paused) {
            self.cheer_at(ride.pan, ride.presence);
        }
        self.dropping = exciting.is_some();
        for _ in 0..frames {
            if self.sample >= self.next_crowd {
                self.next_crowd = self.sample + u64::from(RATE) * (7 + u64::from(self.noise % 7));
                if !self.state.paused && self.state.crowd > 0.08 && self.state.happiness > 0.5 {
                    self.cheer_at(self.state.crowd_pan, self.state.crowd * 0.45);
                }
            }
            for (gain, value) in self.gain.iter_mut().zip(target) {
                *gain += (value - *gain) * 0.001;
            }
            self.noise = self.noise.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = (self.noise >> 8) as f32 / 8388608.0 - 1.0;
            self.wind += (noise - self.wind) * 0.025;
            let mut rides = [0.0; 2];
            let mut close_ride = 0.0_f32;
            for (voice, target_ride) in self.rides.iter_mut().zip(self.state.rides) {
                let same = voice.current.map(|r| r.id) == target_ride.map(|r| r.id);
                if !same && voice.gain < 0.0001 {
                    voice.current = target_ride;
                }
                let presence = if !self.state.paused && (same || voice.gain < 0.0001) {
                    target_ride.map_or(0.0, |r| r.presence)
                } else {
                    0.0
                };
                voice.gain += (presence - voice.gain) * 0.002;
                if let Some(target_ride) =
                    target_ride.filter(|r| voice.current.is_some_and(|old| old.id == r.id))
                {
                    voice.current = Some(target_ride);
                }
                if let Some(ride) = voice.current {
                    voice.pan += (ride.pan - voice.pan) * 0.001;
                    voice.speed += (ride.speed - voice.speed) * 0.001;
                    voice.lift += (ride.lift - voice.lift) * 0.002;
                    voice.drop += (ride.drop - voice.drop) * 0.002;
                    let sound = ride_sound(self.sample, noise, self.wind, ride, voice) * voice.gain;
                    let channels = stereo(sound, voice.pan);
                    rides[0] += channels[0];
                    rides[1] += channels[1];
                    close_ride = close_ride.max(voice.gain);
                }
            }
            let clock = self.sample as f64 / f64::from(RATE);
            let bird_age = ((clock + 2.0) % 7.0) as f32;
            let bird = if bird_age < 0.27 {
                sine(bird_age, 1800.0 + bird_age * 2100.0)
                    * (bird_age * std::f32::consts::PI / 0.27).sin()
                    * 0.025
            } else {
                0.0
            };
            let rain = if self.state.rain {
                noise * 0.022 + self.wind * 0.12
            } else {
                0.0
            };
            let murmur = self.wind * 0.07 * self.state.crowd
                + osc(self.sample, 173.0) * osc(self.sample, 3.7) * self.state.crowd * 0.004;
            let ambient = stereo(murmur, self.state.crowd_pan);
            let music = self.score.sample(self.sample, noise);
            let mut effects = [0.0; 2];
            let mut reactions = [0.0; 2];
            for voice in &mut self.voices {
                if voice.delay > 0 {
                    voice.delay -= 1;
                    continue;
                }
                let value = if voice.cue == Cue::Cheer {
                    let position = voice.age as f32 * voice.pitch;
                    let index = position as usize;
                    let a = self.cheer.get(index).copied().unwrap_or(0.0);
                    let b = self.cheer.get(index + 1).copied().unwrap_or(0.0);
                    (a + (b - a) * position.fract()) * voice.volume
                } else {
                    cue_sound(voice.cue, voice.age, noise)
                };
                let bus = if matches!(voice.cue, Cue::Cheer | Cue::Staff) {
                    &mut reactions
                } else {
                    &mut effects
                };
                let value = stereo(value, voice.pan);
                bus[0] += value[0];
                bus[1] += value[1];
                voice.age += 1;
            }
            let cheer_length = self.cheer.len() as f32;
            self.voices.retain(|voice| {
                voice.delay > 0
                    || if voice.cue == Cue::Cheer {
                        (voice.age as f32 * voice.pitch) < cheer_length
                    } else {
                        voice.age < RATE / 2
                    }
            });
            let music_duck = 1.0 - close_ride * 0.42;
            for channel in 0..2 {
                let mixed = (music[channel] * self.gain[1] * music_duck
                    + (ambient[channel] + rain + bird) * self.gain[2]
                    + rides[channel] * self.gain[3]
                    + effects[channel] * self.gain[4]
                    + reactions[channel] * self.gain[5])
                    * self.gain[0];
                // Smooth compression retains headroom when several rides overlap.
                let limited = mixed / (1.0 + mixed.abs() * 0.35);
                out.push((limited.clamp(-0.9, 0.9) * 32767.0) as i16);
            }
            self.sample += 1;
        }
        out
    }
}

fn ride_sound(sample: u64, noise: f32, water: f32, ride: RideEmitter, voice: &RideVoice) -> f32 {
    let clock = sample as f64 / f64::from(RATE) + (ride.id.0.0 % 31) as f64 * 0.137;
    let pulse_age = (clock * (5.0 + f64::from(voice.speed) * 18.0)).fract() as f32;
    let chain =
        (noise * 0.06 + osc(sample, 1800.0) * 0.018) * (-pulse_age * 36.0).exp() * voice.lift;
    let rolling = (noise * 0.040 + osc(sample, 65.0 + voice.speed * 75.0) * 0.026) * voice.speed;
    match ride.model {
        RideModel::LogFlume | RideModel::RiverRapids => {
            let rapids = if ride.model == RideModel::RiverRapids {
                0.6
            } else {
                0.25
            };
            water * (0.35 + voice.speed * 0.55 + voice.drop * 1.5)
                + noise * (0.016 + voice.drop * 0.10 + rapids * 0.03)
                + chain * 0.65
                + osc(sample, 350.0 + 100.0 * osc(sample, 2.1)) * 0.008 * voice.speed
        }
        RideModel::ClassicCoaster | RideModel::SteelCoaster | RideModel::InvertedCoaster => {
            rolling
                + chain
                + water * 0.15 * voice.drop
                + noise * (-pulse_age * 55.0).exp() * voice.speed * 0.045
        }
        RideModel::ParkRailway => {
            rolling * 0.6 + noise * (-pulse_age * 22.0).exp() * voice.speed * 0.09
        }
        RideModel::Monorail => {
            (osc(sample, 145.0) + osc(sample, 290.0) * 0.25) * voice.speed * 0.04
        }
        RideModel::Carousel => {
            let note = [72, 76, 79, 84, 81, 79, 76, 74][(clock / 0.33) as usize % 8];
            let f = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
            (osc(sample, f) + osc(sample, f * 2.0) * 0.35)
                * 0.035
                * (1.0 - (clock / 0.33).fract() as f32)
                + osc(sample, 58.0) * 0.009
        }
        RideModel::FerrisWheel | RideModel::ObservationTower => {
            osc(sample, 76.0) * 0.015 + water * 0.012
        }
        RideModel::TeaCups => osc(sample, 95.0) * 0.018 + water * 0.018,
        RideModel::Swing | RideModel::PirateShip => {
            let swing = (ride.cycle * TAU * 3.0).sin().abs();
            water * (0.03 + swing * 0.16) + osc(sample, 122.0) * swing * 0.018
        }
        RideModel::DropTower => {
            let falling = ((ride.cycle - 0.46) * 12.0).clamp(0.0, 1.0)
                * ((0.72 - ride.cycle) * 12.0).clamp(0.0, 1.0);
            osc(sample, 82.0) * 0.02 + (water * 0.55 + noise * 0.09) * falling
        }
    }
}

fn cue_sound(cue: Cue, age: u32, noise: f32) -> f32 {
    let age = age as f32 / RATE as f32;
    let (f, duration, amp) = match cue {
        Cue::Click => (720.0, 0.055, 0.10),
        Cue::Build => (420.0 + age * 1100.0, 0.18, 0.15),
        Cue::Remove => (260.0 - age * 500.0, 0.16, 0.13),
        Cue::Sale => (if age < 0.09 { 1046.5 } else { 1318.5 }, 0.22, 0.10),
        Cue::Staff => (880.0 + 110.0 * sine(age, 5.0), 0.28, 0.06),
        Cue::Alert => (if age < 0.15 { 440.0 } else { 349.2 }, 0.35, 0.13),
        Cue::Cheer => return 0.0,
    };
    if age >= duration {
        return 0.0;
    }
    let env = (age * 120.0).min(1.0) * (1.0 - age / duration).powi(2);
    let transient = if matches!(cue, Cue::Build | Cue::Remove) {
        noise * (1.0 - age * 25.0).max(0.0) * 0.22
    } else {
        0.0
    };
    (sine(age, f) + 0.18 * sine(age, f * 2.0) + transient) * env * amp
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
pub(super) fn test_ride(model: RideModel) -> RideEmitter {
    RideEmitter {
        id: (RideId(1), 0),
        model,
        speed: 1.0,
        lift: 0.0,
        drop: 0.0,
        cycle: 0.4,
        pan: 0.0,
        presence: 1.0,
        occupied: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reactions_have_independent_volume_and_other_tracks_are_distinct() {
        let mut a = Mixer::default();
        a.state.settings.levels = [100, 0, 0, 0, 0];
        a.state.settings.reactions = 0;
        a.state.crowd = 0.7;
        a.cue(Cue::Cheer);
        assert!(a.render(RATE as usize).iter().all(|x| *x == 0));
        a.state.settings.reactions = 80;
        a.cue(Cue::Cheer);
        assert!(a.render(RATE as usize).iter().any(|x| *x != 0));
        let day = Mixer::default().render(5000);
        let mut next = Mixer::default();
        next.start_track(1);
        assert_ne!(day, next.render(5000));
    }
    #[test]
    fn oscillator_retains_individual_sample_phase_after_two_days() {
        let base = u64::from(RATE) * 60 * 60 * 48;
        assert!((osc(base + 1, 440.0) - osc(1, 440.0)).abs() < 0.00001);
        assert!((osc(base + 1, 440.0) - osc(base, 440.0)).abs() > 0.05);
    }
    #[test]
    fn pcm_is_independent_of_output_block_sizes() {
        let mut whole = Mixer::default();
        whole.state.crowd = 0.8;
        let mut blocks = Mixer {
            state: whole.state,
            ..Default::default()
        };
        let expected = whole.render(320_000);
        let mut actual = Vec::new();
        while actual.len() < expected.len() {
            actual.extend(blocks.render(713.min((expected.len() - actual.len()) / 2)));
        }
        assert_eq!(actual, expected);
    }
    #[test]
    fn mix_is_repeatable_bounded_and_muting_silences_all_buses() {
        let mut a = Mixer::default();
        let mut b = Mixer::default();
        a.state.rides[0] = Some(test_ride(RideModel::SteelCoaster));
        b.state = a.state;
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
        assert_eq!(&wav(&x)[0..4], b"RIFF");
        assert_eq!(wav(&x).len(), x.len() * 2 + 44);
    }
    #[test]
    fn ride_panning_and_distance_are_applied_to_real_pcm() {
        let energy = |pan, presence| {
            let mut mixer = Mixer::default();
            mixer.state.settings.levels = [100, 0, 0, 100, 0];
            let mut ride = test_ride(RideModel::SteelCoaster);
            ride.pan = pan;
            ride.presence = presence;
            mixer.state.rides[0] = Some(ride);
            mixer
                .render(RATE as usize / 2)
                .chunks_exact(2)
                .fold((0i64, 0i64), |(l, r), v| {
                    (l + i64::from(v[0]).abs(), r + i64::from(v[1]).abs())
                })
        };
        let (l, r) = energy(-1.0, 1.0);
        assert!(l > r * 2);
        let (l, r) = energy(1.0, 1.0);
        assert!(r > l * 2);
        assert_eq!(energy(1.0, 0.0), (0, 0));
    }
    #[test]
    fn water_chain_and_rolling_have_distinct_sound_and_source_changes_fade() {
        let render = |model, lift| {
            let mut mixer = Mixer::default();
            mixer.state.settings.levels = [100, 0, 0, 100, 0];
            let mut ride = test_ride(model);
            ride.lift = lift;
            mixer.state.rides[0] = Some(ride);
            mixer.render(9000)
        };
        assert_ne!(
            render(RideModel::LogFlume, 0.0),
            render(RideModel::SteelCoaster, 0.0)
        );
        assert_ne!(
            render(RideModel::SteelCoaster, 1.0),
            render(RideModel::SteelCoaster, 0.0)
        );
        let bytes = include_bytes!("../../../../assets/audio/guest-cheer.wav");
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(bytes[24..28].try_into().unwrap()), RATE);
        assert_eq!(u16::from_le_bytes(bytes[22..24].try_into().unwrap()), 1);
        let mut mixer = Mixer::default();
        mixer.state.settings.levels = [100, 0, 0, 100, 0];
        mixer.state.rides[0] = Some(test_ride(RideModel::SteelCoaster));
        mixer.render(4096);
        mixer.state.rides[0] = None;
        let fading = mixer.render(8192);
        assert!(fading[..128].iter().any(|sample| *sample != 0));
        assert!(
            fading[fading.len() - 128..]
                .iter()
                .all(|sample| *sample == 0)
        );
    }
}
