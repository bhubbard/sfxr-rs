use serde::{Deserialize, Serialize};

use crate::synth::{SoundParams, Waveform};

/// Iconic 8-bit sound presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    PickupCoin,
    LaserShoot,
    Explosion,
    Powerup,
    HitHurt,
    Jump,
    BlipSelect,
    SynthTone,
}

// PRNG helper
fn rnd() -> f32 {
    crate::synth::rand_float()
}

fn rnd_range(min: f32, max: f32) -> f32 {
    min + rnd() * (max - min)
}

fn rnd_bool() -> bool {
    rnd() > 0.5
}

impl Preset {
    /// Generates sound parameters for the chosen preset.
    pub fn generate(&self) -> SoundParams {
        let mut p = SoundParams::default();

        match self {
            Preset::PickupCoin => {
                p.wave_type = Waveform::Square;
                p.start_frequency = rnd_range(0.4, 0.9);
                p.attack_time = 0.0;
                p.sustain_time = 0.1;
                p.decay_time = rnd_range(0.1, 0.5);
                p.sustain_punch = rnd_range(0.3, 0.6);

                if rnd_bool() {
                    p.change_speed = rnd_range(0.5, 0.7);
                    p.change_amount = rnd_range(0.2, 0.6);
                }
            }
            Preset::LaserShoot => {
                p.wave_type = if rnd() < 0.33 {
                    Waveform::Sine
                } else if rnd() < 0.66 {
                    Waveform::Sawtooth
                } else {
                    Waveform::Square
                };

                p.start_frequency = rnd_range(0.5, 1.0);
                p.min_frequency = (p.start_frequency - rnd_range(0.2, 0.8)).max(0.1);
                p.slide = -rnd_range(0.15, 0.35);

                p.attack_time = 0.0;
                p.sustain_time = rnd_range(0.1, 0.3);
                p.decay_time = rnd_range(0.0, 0.4);
                p.sustain_punch = rnd_range(0.0, 0.3);

                if rnd_bool() {
                    p.square_duty = rnd_range(0.0, 0.5);
                    p.duty_sweep = rnd_range(-0.2, 0.2);
                }
                if rnd_bool() {
                    p.hpf_cutoff = rnd_range(0.0, 0.3);
                }
            }
            Preset::Explosion => {
                p.wave_type = Waveform::Noise;

                if rnd_bool() {
                    p.start_frequency = rnd_range(0.1, 0.5);
                    p.slide = rnd_range(-0.1, 0.3);
                } else {
                    p.start_frequency = rnd_range(0.2, 0.8);
                    p.slide = -rnd_range(0.2, 0.4);
                }

                p.attack_time = 0.0;
                p.sustain_time = rnd_range(0.1, 0.4);
                p.decay_time = rnd_range(0.2, 0.6);
                p.sustain_punch = rnd_range(0.2, 0.8);

                if rnd_bool() {
                    p.flanger_offset = rnd_range(-0.3, 0.3);
                    p.flanger_sweep = rnd_range(-0.3, 0.3);
                }
                if rnd_bool() {
                    p.lpf_cutoff = rnd_range(0.4, 0.9);
                    p.lpf_resonance = rnd_range(0.0, 0.5);
                }
            }
            Preset::Powerup => {
                p.wave_type = if rnd_bool() {
                    Waveform::Square
                } else {
                    Waveform::Sawtooth
                };

                p.start_frequency = rnd_range(0.2, 0.5);
                p.slide = rnd_range(0.1, 0.5);
                p.attack_time = 0.0;
                p.sustain_time = rnd_range(0.1, 0.5);
                p.decay_time = rnd_range(0.1, 0.5);

                if rnd_bool() {
                    p.repeat_speed = rnd_range(0.3, 0.7);
                }
            }
            Preset::HitHurt => {
                p.wave_type = if rnd_bool() {
                    Waveform::Noise
                } else {
                    Waveform::Square
                };

                p.start_frequency = rnd_range(0.2, 0.8);
                p.slide = -rnd_range(0.3, 0.7);
                p.attack_time = 0.0;
                p.sustain_time = rnd_range(0.05, 0.15);
                p.decay_time = rnd_range(0.1, 0.3);

                if rnd_bool() {
                    p.lpf_cutoff = rnd_range(0.6, 1.0);
                }
            }
            Preset::Jump => {
                p.wave_type = Waveform::Square;
                p.square_duty = rnd_range(0.0, 0.6);
                p.start_frequency = rnd_range(0.3, 0.6);
                p.slide = rnd_range(0.1, 0.3);
                p.attack_time = 0.0;
                p.sustain_time = rnd_range(0.1, 0.4);
                p.decay_time = rnd_range(0.1, 0.3);

                if rnd_bool() {
                    p.hpf_cutoff = rnd_range(0.0, 0.3);
                }
            }
            Preset::BlipSelect => {
                p.wave_type = if rnd_bool() {
                    Waveform::Square
                } else {
                    Waveform::Sawtooth
                };

                p.start_frequency = rnd_range(0.2, 0.6);
                p.attack_time = 0.0;
                p.sustain_time = rnd_range(0.05, 0.15);
                p.decay_time = rnd_range(0.0, 0.2);
                p.hpf_cutoff = 0.1;
            }
            Preset::SynthTone => {
                p.wave_type = Waveform::Sine;
                p.start_frequency = 0.44; // Concert A approximate
                p.attack_time = 0.05;
                p.sustain_time = 0.4;
                p.decay_time = 0.2;
                p.vibrato_depth = 0.15;
                p.vibrato_speed = 0.3;
            }
        }

        p
    }
}

/// Slightly mutates sound parameters by +/- `amount` to create audio variations.
pub fn mutate(params: &SoundParams, amount: f32) -> SoundParams {
    let mut p = params.clone();
    let delta = |val: f32, min_val: f32, max_val: f32| -> f32 {
        let diff = (rnd() * 2.0 - 1.0) * amount;
        (val + diff).clamp(min_val, max_val)
    };

    p.attack_time = delta(p.attack_time, 0.0, 1.0);
    p.sustain_time = delta(p.sustain_time, 0.0, 1.0);
    p.sustain_punch = delta(p.sustain_punch, 0.0, 1.0);
    p.decay_time = delta(p.decay_time, 0.0, 1.0);

    p.start_frequency = delta(p.start_frequency, 0.0, 1.0);
    p.min_frequency = delta(p.min_frequency, 0.0, 1.0);
    p.slide = delta(p.slide, -1.0, 1.0);
    p.delta_slide = delta(p.delta_slide, -1.0, 1.0);

    p.vibrato_depth = delta(p.vibrato_depth, 0.0, 1.0);
    p.vibrato_speed = delta(p.vibrato_speed, 0.0, 1.0);

    p.change_amount = delta(p.change_amount, -1.0, 1.0);
    p.change_speed = delta(p.change_speed, 0.0, 1.0);

    p.square_duty = delta(p.square_duty, 0.0, 1.0);
    p.duty_sweep = delta(p.duty_sweep, -1.0, 1.0);

    p.repeat_speed = delta(p.repeat_speed, 0.0, 1.0);
    p.flanger_offset = delta(p.flanger_offset, -1.0, 1.0);
    p.flanger_sweep = delta(p.flanger_sweep, -1.0, 1.0);

    p.lpf_cutoff = delta(p.lpf_cutoff, 0.0, 1.0);
    p.lpf_ramp = delta(p.lpf_ramp, -1.0, 1.0);
    p.lpf_resonance = delta(p.lpf_resonance, 0.0, 1.0);

    p.hpf_cutoff = delta(p.hpf_cutoff, 0.0, 1.0);
    p.hpf_ramp = delta(p.hpf_ramp, -1.0, 1.0);

    p
}

/// Generates a completely randomized set of SFXR sound parameters.
pub fn randomize() -> SoundParams {
    SoundParams {
        wave_type: Waveform::from_u32((rnd() * 5.0) as u32),

        attack_time: if rnd_bool() { rnd_range(0.0, 0.2) } else { 0.0 },
        sustain_time: rnd_range(0.05, 0.5),
        sustain_punch: if rnd_bool() { rnd_range(0.0, 0.5) } else { 0.0 },
        decay_time: rnd_range(0.1, 0.6),

        start_frequency: rnd_range(0.1, 0.9),
        min_frequency: if rnd_bool() { rnd_range(0.0, 0.3) } else { 0.0 },
        slide: if rnd_bool() { rnd_range(-0.5, 0.5) } else { 0.0 },
        delta_slide: if rnd_bool() { rnd_range(-0.2, 0.2) } else { 0.0 },

        vibrato_depth: if rnd_bool() { rnd_range(0.0, 0.4) } else { 0.0 },
        vibrato_speed: if rnd_bool() { rnd_range(0.1, 0.8) } else { 0.0 },

        change_amount: if rnd_bool() { rnd_range(-0.5, 0.5) } else { 0.0 },
        change_speed: if rnd_bool() { rnd_range(0.2, 0.8) } else { 0.0 },

        square_duty: rnd_range(0.0, 0.8),
        duty_sweep: if rnd_bool() { rnd_range(-0.3, 0.3) } else { 0.0 },

        repeat_speed: if rnd() < 0.25 { rnd_range(0.2, 0.8) } else { 0.0 },

        flanger_offset: if rnd() < 0.25 { rnd_range(-0.3, 0.3) } else { 0.0 },
        flanger_sweep: if rnd() < 0.25 { rnd_range(-0.3, 0.3) } else { 0.0 },

        lpf_cutoff: if rnd_bool() { rnd_range(0.3, 1.0) } else { 1.0 },
        lpf_ramp: if rnd_bool() { rnd_range(-0.3, 0.3) } else { 0.0 },
        lpf_resonance: if rnd_bool() { rnd_range(0.0, 0.6) } else { 0.0 },

        hpf_cutoff: if rnd() < 0.3 { rnd_range(0.0, 0.4) } else { 0.0 },
        hpf_ramp: if rnd() < 0.3 { rnd_range(-0.2, 0.2) } else { 0.0 },

        master_volume: 0.5,
    }
}
