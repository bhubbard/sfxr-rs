use thiserror::Error;

pub mod buffer;
pub mod presets;
pub mod synth;
pub mod wav;

pub use buffer::{AudioBuffer, SampleFormat};
pub use presets::{mutate, randomize, Preset};
pub use synth::{SoundParams, Synth, Waveform};
pub use wav::{parse_wav_info, to_wav_bytes, write_wav};

#[derive(Debug, Error, PartialEq)]
pub enum SfxrError {
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(u32),

    #[error("Invalid WAV header or structure: {0}")]
    InvalidWavHeader(String),

    #[error("Audio buffer is empty")]
    EmptyBuffer,

    #[error("I/O error during WAV encoding: {0}")]
    IoError(String),
}

/// Convenience function to generate an `AudioBuffer` directly from `SoundParams` at 44.1kHz.
pub fn generate(params: &SoundParams) -> AudioBuffer {
    let mut synth = Synth::new();
    synth.generate(params, 44100)
}

/// Convenience function to generate standard 16-bit PCM WAV bytes directly from `SoundParams`.
pub fn generate_wav(params: &SoundParams) -> Vec<u8> {
    let buffer = generate(params);
    to_wav_bytes(&buffer)
}
