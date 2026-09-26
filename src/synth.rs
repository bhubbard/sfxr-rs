use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::buffer::AudioBuffer;

/// Supported generator waveforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Waveform {
    #[default]
    Square = 0,
    Sawtooth = 1,
    Sine = 2,
    Noise = 3,
    Triangle = 4,
}

impl Waveform {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Waveform::Sawtooth,
            2 => Waveform::Sine,
            3 => Waveform::Noise,
            4 => Waveform::Triangle,
            _ => Waveform::Square,
        }
    }
}

/// Comprehensive parameter set controlling SFXR synthesis.
///
/// All parameters are normalized to the range `[0.0, 1.0]` (or `[-1.0, 1.0]` for bi-directional sweeps)
/// to match Tomas Pettersson's SFXR model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoundParams {
    pub wave_type: Waveform,

    // Envelope
    pub attack_time: f32,
    pub sustain_time: f32,
    pub sustain_punch: f32,
    pub decay_time: f32,

    // Frequency dynamics
    pub start_frequency: f32,
    pub min_frequency: f32,
    pub slide: f32,
    pub delta_slide: f32,

    // Vibrato
    pub vibrato_depth: f32,
    pub vibrato_speed: f32,

    // Arpeggio / Pitch jump
    pub change_amount: f32,
    pub change_speed: f32,

    // Duty Cycle (Square wave)
    pub square_duty: f32,
    pub duty_sweep: f32,

    // Repeat
    pub repeat_speed: f32,

    // Flanger
    pub flanger_offset: f32,
    pub flanger_sweep: f32,

    // Low-Pass Filter
    pub lpf_cutoff: f32,
    pub lpf_ramp: f32,
    pub lpf_resonance: f32,

    // High-Pass Filter
    pub hpf_cutoff: f32,
    pub hpf_ramp: f32,

    // Master Output
    pub master_volume: f32,
}

impl Default for SoundParams {
    fn default() -> Self {
        Self {
            wave_type: Waveform::Square,

            attack_time: 0.0,
            sustain_time: 0.3,
            sustain_punch: 0.0,
            decay_time: 0.4,

            start_frequency: 0.3,
            min_frequency: 0.0,
            slide: 0.0,
            delta_slide: 0.0,

            vibrato_depth: 0.0,
            vibrato_speed: 0.0,

            change_amount: 0.0,
            change_speed: 0.0,

            square_duty: 0.0,
            duty_sweep: 0.0,

            repeat_speed: 0.0,

            flanger_offset: 0.0,
            flanger_sweep: 0.0,

            lpf_cutoff: 1.0,
            lpf_ramp: 0.0,
            lpf_resonance: 0.0,

            hpf_cutoff: 0.0,
            hpf_ramp: 0.0,

            master_volume: 0.5,
        }
    }
}

/// SFXR Sound Synthesizer.
pub struct Synth {
    // Envelope internal state
    env_stage: usize,
    env_time: usize,
    env_length: [usize; 3],

    // Frequency state
    fperiod: f64,
    fmaxperiod: f64,
    fslide: f64,
    fdslide: f64,
    period: i32,

    // Vibrato state
    vib_phase: f32,
    vib_speed: f32,
    vib_depth: f32,

    // Arpeggio state
    arp_time: usize,
    arp_limit: usize,
    arp_mod: f64,

    // Duty cycle state
    square_duty: f32,
    duty_sweep: f32,

    // Repeat state
    rep_time: usize,
    rep_limit: usize,

    // Flanger state
    flanger_buffer: [f32; 1024],
    flanger_pos: usize,
    flanger_offset: f32,
    flanger_delta: f32,

    // Filter state
    fltp: f32,
    fltdp: f32,
    fltw: f32,
    fltw_d: f32,
    fltdmp: f32,
    flthp: f32,
    flthp_d: f32,

    // Wave state
    phase: i32,
    noise_buffer: [f32; 32],
}

impl Default for Synth {
    fn default() -> Self {
        Self::new()
    }
}

impl Synth {
    pub fn new() -> Self {
        let mut synth = Self {
            env_stage: 0,
            env_time: 0,
            env_length: [0; 3],

            fperiod: 0.0,
            fmaxperiod: 0.0,
            fslide: 0.0,
            fdslide: 0.0,
            period: 0,

            vib_phase: 0.0,
            vib_speed: 0.0,
            vib_depth: 0.0,

            arp_time: 0,
            arp_limit: 0,
            arp_mod: 0.0,

            square_duty: 0.0,
            duty_sweep: 0.0,

            rep_time: 0,
            rep_limit: 0,

            flanger_buffer: [0.0; 1024],
            flanger_pos: 0,
            flanger_offset: 0.0,
            flanger_delta: 0.0,

            fltp: 0.0,
            fltdp: 0.0,
            fltw: 0.0,
            fltw_d: 0.0,
            fltdmp: 0.0,
            flthp: 0.0,
            flthp_d: 0.0,

            phase: 0,
            noise_buffer: [0.0; 32],
        };

        // Populate initial noise buffer
        let mut seed = 0x12345678u32;
        for sample in &mut synth.noise_buffer {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *sample = ((seed & 0xFFFF) as f32 / 32768.0) - 1.0;
        }

        synth
    }

    /// Resets synthesis parameters for a new sound run or tempo repeat.
    fn reset_sample(&mut self, params: &SoundParams, restart: bool) {
        self.fperiod = 100.0 / (params.start_frequency as f64 * params.start_frequency as f64 + 0.001);
        self.period = self.fperiod as i32;
        self.fmaxperiod = 100.0 / (params.min_frequency as f64 * params.min_frequency as f64 + 0.001);
        self.fslide = 1.0 - (params.slide as f64).powi(3) * 0.01;
        self.fdslide = -(params.delta_slide as f64).powi(3) * 0.000001;

        self.square_duty = 0.5 - params.square_duty * 0.5;
        self.duty_sweep = -params.duty_sweep * 0.00005;

        if params.change_amount >= 0.0 {
            self.arp_mod = 1.0 - (params.change_amount as f64).powi(2) * 0.9;
        } else {
            self.arp_mod = 1.0 + (params.change_amount as f64).powi(2) * 10.0;
        }
        self.arp_time = 0;
        self.arp_limit = if params.change_speed == 1.0 {
            0
        } else {
            ((1.0 - params.change_speed).powi(2) * 20000.0 + 32.0) as usize
        };

        if restart {
            self.phase = 0;

            // Envelope initialization with perceptual curve
            self.env_stage = 0;
            self.env_time = 0;
            self.env_length[0] = (params.attack_time * params.attack_time * 100000.0) as usize;
            self.env_length[1] = (params.sustain_time * params.sustain_time * 100000.0) as usize;
            self.env_length[2] = (params.decay_time * params.decay_time * 100000.0) as usize;

            // Filters
            self.fltw = params.lpf_cutoff.powi(3) * 0.1;
            self.fltw_d = 1.0 + params.lpf_ramp * 0.0001;
            self.fltdmp = 5.0 / (1.0 + params.lpf_resonance.powi(2) * 20.0) * (0.01 + self.fltw);
            if self.fltdmp > 0.8 {
                self.fltdmp = 0.8;
            }
            self.fltdp = 0.0;
            self.fltp = 0.0;

            self.flthp = 0.0;
            self.flthp_d = params.hpf_cutoff.powi(2) * 0.1;

            // Vibrato
            self.vib_phase = 0.0;
            self.vib_speed = params.vibrato_speed.powi(2) * 0.01;
            self.vib_depth = params.vibrato_depth * 0.5;

            // Flanger
            self.flanger_offset = 0.0;
            self.flanger_delta = params.flanger_sweep.powi(3) * 0.0002;
            self.flanger_pos = 0;
            self.flanger_buffer.fill(0.0);

            // Repeat
            self.rep_time = 0;
            self.rep_limit = if params.repeat_speed == 0.0 {
                0
            } else {
                ((1.0 - params.repeat_speed).powi(2) * 20000.0 + 32.0) as usize
            };
        }
    }

    /// Synthesizes the full audio buffer for the given parameters.
    pub fn generate(&mut self, params: &SoundParams, sample_rate: u32) -> AudioBuffer {
        self.reset_sample(params, true);

        let mut output = Vec::with_capacity(44100);
        let max_samples = 44100 * 10; // Safety cap: 10 seconds

        while self.env_stage < 3 && output.len() < max_samples {
            // Repeat cycle
            if self.rep_limit > 0 {
                self.rep_time += 1;
                if self.rep_time >= self.rep_limit {
                    self.rep_time = 0;
                    self.reset_sample(params, false);
                }
            }

            // Arpeggio / pitch jump
            if self.arp_limit > 0 {
                self.arp_time += 1;
                if self.arp_time >= self.arp_limit {
                    self.arp_limit = 0;
                    self.fperiod *= self.arp_mod;
                }
            }

            // Frequency slides
            self.fslide += self.fdslide;
            self.fperiod *= self.fslide;
            if self.fperiod > self.fmaxperiod {
                self.fperiod = self.fmaxperiod;
                if params.min_frequency > 0.0 {
                    break;
                }
            }

            let mut rfperiod = self.fperiod;
            if self.vib_depth > 0.0 {
                self.vib_phase += self.vib_speed;
                rfperiod = self.fperiod * (1.0 + self.vib_phase.sin() as f64 * self.vib_depth as f64);
            }

            self.period = (rfperiod as i32).max(8);

            // Duty cycle sweep
            if params.wave_type == Waveform::Square {
                self.square_duty = (self.square_duty + self.duty_sweep).clamp(0.0, 0.5);
            }

            // Envelope calculation
            self.env_time += 1;
            if self.env_time > self.env_length[self.env_stage] {
                self.env_time = 0;
                self.env_stage += 1;
            }

            let env_vol = match self.env_stage {
                0 => {
                    // Attack
                    if self.env_length[0] > 0 {
                        self.env_time as f32 / self.env_length[0] as f32
                    } else {
                        1.0
                    }
                }
                1 => {
                    // Sustain with optional punch
                    if self.env_length[1] > 0 {
                        1.0 + (1.0 - (self.env_time as f32 / self.env_length[1] as f32))
                            * 2.0
                            * params.sustain_punch
                    } else {
                        1.0
                    }
                }
                // Decay
                2 if self.env_length[2] > 0 => {
                    1.0 - (self.env_time as f32 / self.env_length[2] as f32)
                }
                _ => 0.0,
            };

            // Flanger offset calculation
            self.flanger_offset += self.flanger_delta;
            let flanger_int_offset = self.flanger_offset.abs() as usize;

            // Generate raw waveform sample
            self.phase += 1;
            if self.phase >= self.period {
                self.phase %= self.period;
                if params.wave_type == Waveform::Noise {
                    for sample in &mut self.noise_buffer {
                        *sample = rand_float() * 2.0 - 1.0;
                    }
                }
            }

            let fp = self.phase as f32 / self.period as f32;
            let mut sample = match params.wave_type {
                Waveform::Square => {
                    if fp < self.square_duty {
                        0.5
                    } else {
                        -0.5
                    }
                }
                Waveform::Sawtooth => 1.0 - fp * 2.0,
                Waveform::Sine => (fp * 2.0 * PI).sin(),
                Waveform::Noise => {
                    let idx = (fp * 32.0) as usize % 32;
                    self.noise_buffer[idx]
                }
                Waveform::Triangle => {
                    if fp < 0.5 {
                        fp * 4.0 - 1.0
                    } else {
                        3.0 - fp * 4.0
                    }
                }
            };

            // Low-pass & High-pass filters
            let pp = self.fltp;
            self.fltw = (self.fltw * self.fltw_d).clamp(0.0, 0.1);

            if params.lpf_cutoff < 1.0 {
                self.fltdp += (sample - self.fltp) * self.fltw;
                self.fltdp -= self.fltdp * self.fltdmp;
            } else {
                self.fltp = sample;
                self.fltdp = 0.0;
            }
            self.fltp += self.fltdp;

            self.flthp += (self.fltp - pp) * self.flthp_d;
            sample = self.fltp - self.flthp;

            // Flanger effect
            if params.flanger_offset != 0.0 || params.flanger_sweep != 0.0 {
                self.flanger_buffer[self.flanger_pos] = sample;
                let delay_idx = (self.flanger_pos + 1024 - (flanger_int_offset % 1024)) % 1024;
                sample += self.flanger_buffer[delay_idx];
                self.flanger_pos = (self.flanger_pos + 1) % 1024;
            }

            // Apply master volume and envelope
            let final_sample = (sample * env_vol * params.master_volume).clamp(-1.0, 1.0);
            output.push(final_sample);
        }

        AudioBuffer::new(sample_rate, output)
    }
}

// Thread-safe fast pseudo-random float generator
use std::sync::atomic::{AtomicU32, Ordering};
static RNG_STATE: AtomicU32 = AtomicU32::new(0x87654321);

pub(crate) fn rand_float() -> f32 {
    let mut current = RNG_STATE.load(Ordering::Relaxed);
    loop {
        let next = current.wrapping_mul(1103515245).wrapping_add(12345);
        match RNG_STATE.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return (next & 0x7FFFFFFF) as f32 / 2147483648.0,
            Err(actual) => current = actual,
        }
    }
}
