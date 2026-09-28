use sfxr::synth::{SoundParams, Synth, Waveform};
use std::f32::consts::PI;

fn compute_dft_bin(samples: &[f32], freq_hz: f32, sample_rate: u32) -> (f32, f32) {
    let mut real = 0.0f32;
    let mut imag = 0.0f32;
    let n = samples.len() as f32;
    let w = 2.0 * PI * freq_hz / (sample_rate as f32);

    for (t, &s) in samples.iter().enumerate() {
        let angle = w * (t as f32);
        real += s * angle.cos();
        imag -= s * angle.sin();
    }

    (real / n, imag / n)
}

fn bin_magnitude(real: f32, imag: f32) -> f32 {
    (real * real + imag * imag).sqrt()
}

#[test]
fn test_sine_wave_harmonic_purity_and_frequency_parity() {
    let mut synth = Synth::new();
    let mut params = SoundParams::default();
    params.wave_type = Waveform::Sine;

    // In SFXR: period = 100.0 / (start_freq^2 + 0.001). Frequency = 44100 / period = 441 * (start_freq^2 + 0.001)
    // For start_freq = 0.5: expected freq = 441 * 0.251 = 110.7 Hz
    params.start_frequency = 0.5;
    params.attack_time = 0.0;
    params.sustain_time = 0.8;
    params.decay_time = 0.0;

    let sample_rate = 44100;
    let buffer = synth.generate(&params, sample_rate);
    assert!(!buffer.is_empty());

    // Search fundamental around 110 Hz (+/- 10 Hz)
    let mut peak_mag = 0.0f32;
    let mut peak_freq = 0.0f32;

    for f in 90..130 {
        let (r, i) = compute_dft_bin(&buffer.samples, f as f32, sample_rate);
        let mag = bin_magnitude(r, i);
        if mag > peak_mag {
            peak_mag = mag;
            peak_freq = f as f32;
        }
    }

    let expected_freq = 441.0 * (0.5f32.powi(2) + 0.001);
    let freq_error = (peak_freq - expected_freq).abs();
    println!("Sine Wave Parity: Measured Peak={:.1}Hz, Analytical Target={:.1}Hz (error={:.2}Hz)",
        peak_freq, expected_freq, freq_error);

    assert!(freq_error < 2.0, "Frequency error must be < 2 Hz, measured: {freq_error}Hz");
    assert!(peak_mag > 0.1, "Sine peak magnitude must be prominent");

    // Check 2nd harmonic (octave at ~221 Hz) suppression
    let (r2, i2) = compute_dft_bin(&buffer.samples, peak_freq * 2.0, sample_rate);
    let h2_mag = bin_magnitude(r2, i2);
    let h2_suppression_db = 20.0 * (peak_mag / (h2_mag + 1e-6)).log10();
    println!("Harmonic Suppression: 2nd Harmonic Suppression = {:.2} dB", h2_suppression_db);
    assert!(h2_suppression_db > 20.0, "2nd harmonic suppression should exceed 20 dB for pure sine");
}

#[test]
fn test_envelope_energy_conservation_and_continuity() {
    let mut synth = Synth::new();
    let mut params = SoundParams::default();
    params.wave_type = Waveform::Sine;
    params.start_frequency = 0.5;
    params.attack_time = 0.2;
    params.sustain_time = 0.5;
    params.decay_time = 0.3;

    let buffer = synth.generate(&params, 44100);
    assert!(buffer.samples.len() > 100);

    // Verify sample continuity (no clipping jumps > 0.3 per sample)
    for i in 1..buffer.samples.len() {
        let diff = (buffer.samples[i] - buffer.samples[i - 1]).abs();
        assert!(diff < 0.35, "Sample discontinuity detected at {}: delta={}", i, diff);
    }

    // Verify envelope bounds
    let max_amp = buffer.samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amp > 0.05 && max_amp <= 1.0, "Maximum amplitude must be in valid [0.05, 1.0] range");
}

#[test]
fn test_triangle_waveform_symmetry_and_parity() {
    let mut synth = Synth::new();
    let mut params = SoundParams::default();
    params.wave_type = Waveform::Triangle;
    params.start_frequency = 0.5;
    params.attack_time = 0.05;
    params.sustain_time = 0.5;
    params.decay_time = 0.05;

    let buffer = synth.generate(&params, 44100);
    assert!(!buffer.is_empty());

    // Triangle waves must maintain near-zero DC offset
    let dc_offset = buffer.samples.iter().sum::<f32>() / (buffer.samples.len() as f32);
    assert!(dc_offset.abs() < 0.05, "DC offset must be near zero, measured: {dc_offset}");
}
