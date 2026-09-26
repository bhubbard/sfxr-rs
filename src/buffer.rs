use serde::{Deserialize, Serialize};

/// Audio sample representation format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat {
    F32,
    I16,
}

/// Buffer containing synthesized audio samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioBuffer {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

impl AudioBuffer {
    pub fn new(sample_rate: u32, samples: Vec<f32>) -> Self {
        Self {
            sample_rate,
            samples,
        }
    }

    pub fn with_capacity(sample_rate: u32, capacity: usize) -> Self {
        Self {
            sample_rate,
            samples: Vec::with_capacity(capacity),
        }
    }

    /// Length in samples.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Duration in seconds.
    pub fn duration(&self) -> f32 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.samples.len() as f32 / self.sample_rate as f32
        }
    }

    /// Maximum absolute peak amplitude.
    pub fn peak(&self) -> f32 {
        self.samples.iter().fold(0.0f32, |max, &s| max.max(s.abs()))
    }

    /// Root Mean Square (RMS) energy.
    pub fn rms(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = self.samples.iter().map(|s| s * s).sum();
        (sum_sq / self.samples.len() as f32).sqrt()
    }

    /// Normalizes samples so the maximum peak matches `target_peak` (e.g. 0.95).
    pub fn normalize(&mut self, target_peak: f32) {
        let current_peak = self.peak();
        if current_peak > 1e-6 {
            let gain = target_peak / current_peak;
            for sample in &mut self.samples {
                *sample *= gain;
            }
        }
    }

    /// Converts normalized f32 samples (-1.0..1.0) to standard 16-bit signed PCM integers (-32768..32767).
    pub fn to_i16(&self) -> Vec<i16> {
        self.samples
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                if clamped >= 0.0 {
                    (clamped * 32767.0) as i16
                } else {
                    (clamped * 32768.0) as i16
                }
            })
            .collect()
    }
}
