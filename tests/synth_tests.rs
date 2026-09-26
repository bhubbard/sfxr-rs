use sfxr::synth::{SoundParams, Synth, Waveform};

#[test]
fn test_waveform_synthesis_all_types() {
    let mut synth = Synth::new();
    let waveforms = [
        Waveform::Square,
        Waveform::Sawtooth,
        Waveform::Sine,
        Waveform::Noise,
        Waveform::Triangle,
    ];

    for wf in waveforms {
        let mut params = SoundParams::default();
        params.wave_type = wf;
        params.attack_time = 0.05;
        params.sustain_time = 0.3;
        params.decay_time = 0.3;

        let buffer = synth.generate(&params, 44100);

        assert!(
            !buffer.is_empty(),
            "Buffer should not be empty for waveform {:?}",
            wf
        );
        assert!(
            buffer.duration() > 0.05,
            "Duration should be > 0.05s for {:?}",
            wf
        );

        // Check bounds
        for (i, &s) in buffer.samples.iter().enumerate() {
            assert!(
                s >= -1.0 && s <= 1.0,
                "Sample {} at {} out of bounds: {}",
                i,
                buffer.duration(),
                s
            );
        }

        assert!(
            buffer.peak() > 0.01,
            "Waveform {:?} produced silent audio",
            wf
        );
    }
}

#[test]
fn test_frequency_slide_and_vibrato() {
    let mut synth = Synth::new();
    let mut params = SoundParams::default();
    params.wave_type = Waveform::Sine;
    params.start_frequency = 0.5;
    params.slide = -0.2; // Slide down
    params.vibrato_depth = 0.3;
    params.vibrato_speed = 0.4;
    params.sustain_time = 0.1;
    params.decay_time = 0.1;

    let buffer = synth.generate(&params, 44100);
    assert!(!buffer.is_empty());
    assert!(buffer.rms() > 0.05);
}

#[test]
fn test_filters_and_flanger() {
    let mut synth = Synth::new();
    let mut params = SoundParams::default();
    params.wave_type = Waveform::Noise;
    params.lpf_cutoff = 0.4;
    params.lpf_resonance = 0.5;
    params.hpf_cutoff = 0.1;
    params.flanger_offset = 0.2;
    params.flanger_sweep = 0.1;
    params.sustain_time = 0.1;
    params.decay_time = 0.1;

    let buffer = synth.generate(&params, 44100);
    assert!(!buffer.is_empty());
    assert!(buffer.peak() > 0.0);
}
