use sfxr::presets::{mutate, randomize, Preset};
use sfxr::synth::Synth;

#[test]
fn test_all_iconic_presets() {
    let mut synth = Synth::new();

    let presets = [
        Preset::PickupCoin,
        Preset::LaserShoot,
        Preset::Explosion,
        Preset::Powerup,
        Preset::HitHurt,
        Preset::Jump,
        Preset::BlipSelect,
        Preset::SynthTone,
    ];

    for preset in presets {
        let params = preset.generate();
        let buffer = synth.generate(&params, 44100);

        assert!(
            !buffer.is_empty(),
            "Preset {:?} produced empty buffer",
            preset
        );
        assert!(
            buffer.duration() > 0.02,
            "Preset {:?} duration too short: {}s",
            preset,
            buffer.duration()
        );
        assert!(
            buffer.peak() > 0.01,
            "Preset {:?} is inaudible (peak = {})",
            preset,
            buffer.peak()
        );
    }
}

#[test]
fn test_mutator_and_randomizer() {
    let mut synth = Synth::new();
    let base_params = Preset::LaserShoot.generate();
    let mutated = mutate(&base_params, 0.1);

    // Should slightly differ but still synthesize cleanly
    let buffer = synth.generate(&mutated, 44100);
    assert!(!buffer.is_empty());

    // Randomizer should generate valid sound
    let random_params = randomize();
    let rand_buffer = synth.generate(&random_params, 44100);
    assert!(!rand_buffer.is_empty());
}
