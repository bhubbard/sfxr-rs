# 🔊 sfxr-rs

[![GitHub Pages](https://img.shields.io/badge/Live%20Synthesizer-GitHub%20Pages-brightgreen?style=for-the-badge&logo=github)](https://bhubbard.github.io/sfxr-rs/)
[![Rust Edition 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue?style=for-the-badge)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-Passing-success?style=for-the-badge&logo=githubactions)](tests/)

> **Pure Rust procedural 8-bit/16-bit sound effect generator, synthesizer, and WAV serializer.**
> Faithfully based on Tomas Pettersson’s classic **SFXR** procedural audio engine.

---

## 🎛️ [Launch Live Interactive Synthesizer](https://bhubbard.github.io/sfxr-rs/)

Design retro sound effects directly in your web browser:
- **Real-Time Web Audio**: Instant playback powered by the SFXR synthesis DSP engine.
- **Oscilloscope Waveform**: Canvas visualizer rendering audio waveforms in real-time.
- **Iconic Presets**: Coin, Laser, Explosion, Powerup, Hit/Hurt, Jump, Blip/Select, Synth Tone.
- **Mutator & Randomizer**: Create unlimited subtle variations or wild procedural sounds.
- **Instant WAV Export**: Generate and download standard 44.1kHz 16-bit PCM `.wav` files.
- **Code Generator**: One-click copy ready-to-paste Rust `SoundParams` code.

---

## 🚀 Key Features

- **5 Fundamental Waveforms**:
  - `Square` with variable duty cycle and sweep
  - `Sawtooth`
  - `Sine`
  - `White Noise` (deterministic PRNG buffer)
  - `Triangle`
- **Perceptual ADSR Envelope**:
  - Attack, Sustain, and Decay times shaped exponentially ($t^2 \times 100{,}000$ samples) with sustain punch.
- **Frequency Dynamics**:
  - Base frequency, frequency limit clamp, linear pitch slide, and quadratic delta slide (ramp acceleration).
- **Modulation & Pitch Jumps**:
  - Sinusoidal vibrato with configurable depth and speed.
  - Multi-octave arpeggio (pitch jump) with change amount and speed.
- **Dual Resonant Filters**:
  - Low-Pass Filter with dynamic sweep ramp and resonance damping.
  - High-Pass Filter with cutoff sweep ramp.
- **Chorus / Flanger**:
  - 1024-sample circular delay line with bi-directional sweep.
- **Tempo Repeat**:
  - Re-triggers sounds at periodic rhythmic intervals.
- **Audio Output & WAV Serializer**:
  - Renders to raw normalized `f32` and signed `i16` PCM buffers.
  - Pure Rust RIFF WAVE byte encoder (no external audio C-bindings or system dependencies).

---

## 📦 Installation

Add `sfxr` to your `Cargo.toml`:

```toml
[dependencies]
sfxr = "0.1.0"
```

---

## 💻 Quick Start

### 1. Generate an Iconic Preset Sound and Save to WAV

```rust
use sfxr::{Preset, to_wav_bytes, Synth};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate parameters for the iconic coin pickup sound
    let params = Preset::PickupCoin.generate();

    // Synthesize 44.1kHz audio buffer
    let mut synth = Synth::new();
    let mut buffer = synth.generate(&params, 44100);

    // Optional: normalize peak amplitude to 0.95
    buffer.normalize(0.95);

    // Serialize to standard 16-bit Mono WAV bytes
    let wav_bytes = to_wav_bytes(&buffer);

    // Write directly to disk
    let mut file = File::create("coin.wav")?;
    file.write_all(&wav_bytes)?;

    println!("Saved coin.wav ({} samples, {:.2}s)", buffer.len(), buffer.duration());
    Ok(())
}
```

### 2. Custom Sound Synthesis with Full Parameter Control

```rust
use sfxr::{SoundParams, Waveform, generate_wav};
use std::fs::write;

fn main() {
    let laser = SoundParams {
        wave_type: Waveform::Sawtooth,
        start_frequency: 0.65,
        min_frequency: 0.10,
        slide: -0.30,
        decay_time: 0.20,
        sustain_time: 0.10,
        sustain_punch: 0.25,
        hpf_cutoff: 0.15,
        ..Default::default()
    };

    let wav_bytes = generate_wav(&laser);
    write("laser.wav", wav_bytes).unwrap();
}
```

### 3. Procedural Variations with `mutate` and `randomize`

```rust
use sfxr::{mutate, randomize, Preset};

// Mutate an explosion preset by +/- 10% to generate varied sound effects
let base_explosion = Preset::Explosion.generate();
let variation_1 = mutate(&base_explosion, 0.10);
let variation_2 = mutate(&base_explosion, 0.10);

// Generate wild randomized SFX
let random_sfx = randomize();
```

---

## 🧪 Running Tests

```bash
cargo test
```

Test suites verify:
- Waveform synthesis consistency and amplitude boundedness across all 5 waveforms.
- Frequency slide, vibrato modulation, and cutoff filters.
- Preset generation, parameter mutation, and randomizer stability.
- RIFF/WAVE 16-bit PCM header compliance, sample byte-packing, and duration calculations.

---

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
