# AGY-Analog 01

**A real-time, low-latency analog synthesizer emulator built in Rust.**

AGY-Analog 01 is a software synthesizer inspired by the golden era of hardware synthesizers — the Minimoog Model D, Roland Juno-60/106, Sequential Prophet-5, Oberheim OB-Xa, Yamaha CS-80, Roland TB-303, and ARP Odyssey/2600. It delivers a faithful analog subtractive synthesis architecture with a modern real-time DSP engine and an immediate-mode native GUI, designed for live performance without compromise.

---

## Features

### Sound Engine
- **Dual Anti-Aliased Oscillators** (PolyBLEP) with Sawtooth, Square/Pulse, Triangle, and Sine waveforms
- **Pulse Width Modulation (PWM)** on Square waves (5%–95% duty cycle)
- **OSC 2 Coarse & Fine Tuning** — semitone intervals (±12 semi) + fine detune (±50 cents) for organic beating, unison, and harmonic intervals (fifths, octaves)
- **Oscillator Mixer** — continuous crossfade between OSC 1 and OSC 2
- **Resonant State Variable Filter (SVF)** — Low-Pass, High-Pass, and Band-Pass modes with cutoff (20 Hz–20 kHz) and resonance up to self-oscillation (Q 0.5–15)
- **Dual ADSR Envelopes** — sample-accurate amplitude and filter envelopes with exponential curves
- **Filter Envelope Modulation Depth** — positive and negative modulation ranges for classic pluck and sweep shapes
- **LFO** routable to Pitch (vibrato) or Filter Cutoff (auto-wah/wobble), with Sine, Triangle, Square waveforms at 0.01–20 Hz

### Polyphony & Performance
- **8-Voice Polyphony** with dynamic voice stealing and headroom normalization
- **Monophonic Mode** with legato-style note priority, 16-note stack history, and smooth **portamento glide** (0–2 s)
- **MIDI Input** — USB MIDI keyboard auto-detected on launch via `midir`
- **QWERTY Keyboard** — two-octave live input mapping (Ableton-style), with `Z`/`X` for real-time octave shifting

### Effects Chain
| Effect | Description |
|---|---|
| **Soft Saturation / Drive** | Symmetric cubic soft-clipping (0.0 clean → 1.0 warm tube overdrive) with automatic makeup gain |
| **Roland Juno Stereo Chorus** | BBD analog chorus emulation — Mode I (slow ensemble), Mode II (deep lush), Mode I+II (rotary vibrato) with quadrature LFO for ultra-wide stereo |
| **Stereo Tape Delay** | Analog tape delay (20 ms–1 s), soft-clipped feedback, 1-pole tone damping, ping-pong cross-feedback for spatial depth |

### Interface
- **Immediate-mode native GUI** via `egui`/`eframe` — no browser, no Electron, no latency
- **Analog-style rotary knobs** — drag to adjust, Shift for fine control, double-click to reset
- **Real-time phosphor-style oscilloscope** displaying the live audio waveform
- **Interactive virtual piano keyboard** with active note highlight glow
- **Preset system** — 23 factory presets inspired by classic hardware, plus JSON save/load for user patches
- **Master Effects Rack** — dedicated panel for Drive, Chorus, and Delay controls

---

## Factory Presets

The synthesizer ships with 23 factory presets spanning classic hardware archetypes:

| Category | Presets |
|---|---|
| **Basic** | Init Patch |
| **Minimoog Model D** | Minimoog Model D Bass, Minimoog 70s Lead |
| **Roland Juno-60/106** | Juno-106 Lush Strings, Juno-60 Hoover PWM, Juno Pop Pluck (Axel F), Juno-106 Solid Bass |
| **Sequential Prophet-5** | Prophet-5 Poly Brass, Prophet Fifth Sync Lead, Prophet Ambient Pad |
| **Oberheim OB-Xa** | OB-Xa 'Jump' Brass, Oberheim Warm Sweep |
| **Yamaha CS-80** | CS-80 Blade Runner Lead, CS-80 Vangelis Brass |
| **Roland TB-303** | TB-303 Acid Saw Bassline, TB-303 Square Squelch |
| **ARP Odyssey/2600** | ARP Odyssey Funk Lead, ARP 2600 Sci-Fi Laser |
| **Korg Polysix/MS-20** | MS-20 Aggressive Screamer, Polysix Arp Pluck |
| **Keys** | Analog Electric Piano, Vintage Clavinet D6 |
| **Modern** | Punchy 808 Bass, Synthwave Lead |

---

## Architecture

AGY-Analog 01 uses a strict dual-thread model optimized for real-time safety:

```
┌────────────────────────────────────────────────────────┐
│                  Main / UI Thread                      │
│                                                        │
│  [egui GUI Controls]        [QWERTY Key Events]       │
└─────────────┬──────────────────────────┬───────────────┘
              │                          │
              ▼                          ▼
      Parameter Updates              Note Events
      (Lock-free atomic/queue)       (Lock-free queue)
              │                          │
┌─────────────┴──────────────────────────┴───────────────┐
│                  Audio Thread (cpal)                   │
│                                                        │
│         [Synth Engine → Voices → DSP]                 │
│                      │                                 │
│                      ▼                                 │
│           [CoreAudio Output Buffer (stereo)]           │
└──────────────────────┬─────────────────────────────────┘
                       │  (Waveform snapshot)
                       ▼
              [Lock-free Ringbuffer]
                       │
                       ▼
               [UI Oscilloscope]
```

**Audio thread discipline is strictly enforced:**
- No heap allocation inside the audio callback (no `Vec::push`, `Box::new`, `String`)
- No blocking locks (`Mutex`) — only lock-free atomics and bounded channels (`crossbeam-channel`, `ringbuf`)
- No file I/O, logging, or system calls in the real-time path

---

## QWERTY Keyboard Mapping

Standard two-octave piano layout (identical to Ableton Live / GarageBand):

```
Octave shift:  [ Z (down) ]  [ X (up) ]

Black keys:      [W] [E]     [T] [Y] [U]     [O] [P]
White keys:   [A] [S] [D] [F] [G] [H] [J] [K] [L] [;]
Note:          C   D   E   F   G   A   B   C   D   E
```

---

## Prerequisites

- **Rust toolchain**: 1.75+ stable (edition 2024)
- **Platform**: macOS (CoreAudio). Linux (ALSA/JACK via `cpal`) and Windows (WASAPI) are supported by the underlying libraries but untested.
- **No external C libraries required** — all dependencies managed via Cargo.

For MIDI, connect a USB MIDI keyboard before launching. The app auto-detects the first available MIDI input port.

---

## Quick Start

```bash
# Clone the repository
git clone <repo-url>
cd synth

# Run in development mode
cargo run

# Build optimized release binary (recommended for live performance)
cargo build --release
./target/release/synth
```

---

## Development

```bash
# Run the full test suite (69 tests: unit, pipeline, concurrency stress)
cargo test

# Check code formatting and lints
cargo check
cargo clippy

# Build release binary
cargo build --release
```

### Project Layout

```
synth/
├── Cargo.toml               # Project manifest & dependencies
├── specs/                   # Authoritative design specifications
│   ├── README.md            # Specs index
│   ├── architecture.md      # Threading model & communication design
│   ├── dsp_spec.md          # Oscillators, filters, envelopes, effects
│   ├── ui_spec.md           # UI layout & keyboard mapping
│   └── roadmap.md           # Feature phases & milestones
├── src/
│   ├── main.rs              # Application entry point
│   ├── audio/               # cpal stream setup & CoreAudio bridge
│   ├── dsp/                 # DSP algorithms
│   │   ├── oscillator.rs    # Dual PolyBLEP oscillators
│   │   ├── filter.rs        # State Variable Filter (SVF)
│   │   ├── envelope.rs      # Sample-accurate ADSR
│   │   ├── lfo.rs           # Low Frequency Oscillator
│   │   ├── voice.rs         # Voice allocator, polyphony, portamento
│   │   ├── effects.rs       # Drive, Juno Chorus, Tape Delay
│   │   ├── params.rs        # SynthParams (shared state struct)
│   │   └── presets.rs       # Factory preset library & manager
│   └── ui/                  # egui application views and components
│       ├── app.rs           # Main application layout & panels
│       ├── knob.rs          # Analog-style rotary knob widget
│       ├── keyboard.rs      # Virtual piano keyboard & QWERTY handler
│       └── visualizer.rs    # Phosphor oscilloscope display
└── tests/
    ├── audio_pipeline_test.rs    # End-to-end DSP pipeline tests
    └── concurrency_stress_test.rs # Multi-thread real-time safety tests
```

---

## Dependencies

| Crate | Purpose |
|---|---|
| [`cpal`](https://crates.io/crates/cpal) | Cross-platform audio I/O (CoreAudio on macOS) |
| [`eframe`](https://crates.io/crates/eframe) / `egui` | Immediate-mode native GUI framework |
| [`midir`](https://crates.io/crates/midir) | Cross-platform MIDI input |
| [`crossbeam-channel`](https://crates.io/crates/crossbeam-channel) | Lock-free inter-thread messaging |
| [`ringbuf`](https://crates.io/crates/ringbuf) | Lock-free SPSC ringbuffer for oscilloscope data |
| [`serde`](https://crates.io/crates/serde) + `serde_json` | Preset serialization / JSON export & import |

---

## Specifications

Detailed design specifications live in the [`specs/`](specs/) directory and serve as the authoritative reference for DSP behavior, threading rules, and UI layout. When implementing new features, consult and update these documents first.

- [`specs/architecture.md`](specs/architecture.md) — Threading model & real-time constraints
- [`specs/dsp_spec.md`](specs/dsp_spec.md) — Oscillator, filter, envelope, voice & effects design
- [`specs/ui_spec.md`](specs/ui_spec.md) — UI layout, keyboard mapping, control behavior
- [`specs/roadmap.md`](specs/roadmap.md) — Feature phases & milestones

---

## Contributing / AI Agent Guide

See [`AGENTS.md`](AGENTS.md) for the full developer bootstrap guide, including critical audio thread discipline rules that **must never be violated**, directory structure, and build commands.

---

## License

This project is a personal synthesizer emulator project. License TBD.
