use sfxr::presets::Preset;
use sfxr::synth::Synth;
use sfxr::wav::{parse_wav_info, to_wav_bytes};

#[test]
fn test_wav_serialization_and_header_validation() {
    let mut synth = Synth::new();
    let params = Preset::PickupCoin.generate();
    let mut buffer = synth.generate(&params, 44100);

    buffer.normalize(0.95);
    assert!((buffer.peak() - 0.95).abs() < 1e-3);

    let wav_bytes = to_wav_bytes(&buffer);
    assert!(wav_bytes.len() > 44, "WAV size must be > 44 bytes");

    let (rate, channels, num_samples) =
        parse_wav_info(&wav_bytes).expect("Failed to parse valid WAV header");

    assert_eq!(rate, 44100);
    assert_eq!(channels, 1);
    assert_eq!(num_samples, buffer.len());

    // Verify file starts with RIFF and WAVE
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    assert_eq!(&wav_bytes[8..12], b"WAVE");
    assert_eq!(&wav_bytes[12..16], b"fmt ");
    assert_eq!(&wav_bytes[36..40], b"data");
}

#[test]
fn test_i16_sample_conversion() {
    let mut synth = Synth::new();
    let params = Preset::BlipSelect.generate();
    let buffer = synth.generate(&params, 44100);

    let i16_samples = buffer.to_i16();
    assert_eq!(i16_samples.len(), buffer.len());

    // Verify non-zero values
    let max_val = i16_samples.iter().map(|&s| s.abs()).max().unwrap_or(0);
    assert!(max_val > 100);
}
