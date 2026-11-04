# DSP (Digital Signal Processing) Specification

## 1. Oscillator Architecture (Dual Oscillator System)

The synthesizer features a dual-oscillator architecture (OSC 1 and OSC 2) inspired by analog subtractive synthesizers like the Minimoog and Prophet-5.

### OSC 1 (Primary Oscillator)
- **Waveforms**: Sawtooth, Square / Pulse, Triangle, Sine.
- **Pulse Width**: 5% to 95% duty cycle for square waves.
- **Octave**: Shiftable across `-2`, `-1`, `0`, `+1`, `+2` octaves.

### OSC 2 (Secondary / Detune Oscillator)
- **Waveforms**: Sawtooth, Square / Pulse, Triangle, Sine.
- **Pulse Width**: 5% to 95% duty cycle.
- **Octave**: `-2` to `+2` octaves relative to primary pitch.
- **Coarse Interval**: `-12` to `+12` semitones (enables tuning in fifths (+7), fourths (+5), or octave offsets).
- **Fine Detune**: `-50.0` to `+50.0` cents (produces chorus-like organic beating and phase interaction).

### Oscillator Mixer
- Linear crossfade balance from `0.0` (100% OSC 1) to `1.0` (100% OSC 2), defaulting to `0.5` (50% / 50% blend).

### Waveforms Supported
- **Sawtooth**: Classic bright harmonic content, basis for analog leads and basses.
- **Square / Pulse**: Rich in odd harmonics; includes Pulse Width Modulation (PWM).
- **Triangle**: Subtle, warm, flute-like tone.
- **Sine**: Pure fundamental tone.

### Anti-Aliasing (PolyBLEP)
Naive digital waveforms (e.g. sharp step in square waves) cause severe frequency foldover (aliasing) audible as inharmonic digital screeching at higher pitches.
- We use **Polynomial Bandlimited Step (PolyBLEP)** residual correction around discontinuities.
- PolyBLEP applies a 2nd-order polynomial smoothing at the wrap-around point `t = 0` and pulse transition `t = width`.
- Minimal CPU overhead with clean high register frequency response.

## 2. Resonant Filter Design

- **Topology**: State Variable Filter (SVF) (Chamberlin or Cytomic Andrew Simper SVF model).
- **Modes**:
  - Low-Pass (LPF): 12dB / 24dB per octave rolloff.
  - High-Pass (HPF).
  - Band-Pass (BPF).
- **Parameters**:
  - `Cutoff`: 20 Hz to 20,000 Hz (logarithmic scaling).
  - `Resonance (Q)`: 0.1 to 20.0 (capable of self-oscillation without blowing up output levels).
  - `Drive / Saturation`: Tanh-based soft-clipping inside the feedback loop to mimic analog component saturation.

## 3. Envelope Generator (ADSR)

Sample-accurate 4-stage envelope:
- **Attack**: Time to reach peak value from zero (0.001s to 5.0s, exponential curve).
- **Decay**: Time to descend from peak to sustain level (0.001s to 5.0s, exponential curve).
- **Sustain**: Level held while note is sustained (0.0 to 1.0).
- **Release**: Time to return to zero after note off (0.001s to 10.0s, exponential curve).

## 4. Voice & Polyphony Model

- **Monophonic Mode (Mono)**: Single voice (`voices[0]`) with legato priority, note stack history (up to 16 notes), and smooth monophonic portamento glide between consecutive notes.
- **Polyphonic Mode (Poly 8-Voice)**:
  - 8 independent synthesis voices, each with full dual oscillators, SVF filter, amplitude ADSR, filter ADSR, and LFO.
  - Dynamic voice stealing allocator:
    - Retriggers existing voice if the same note is played again.
    - Allocates inactive voices first.
    - If all 8 voices are occupied, steals the oldest released voice fading out.
    - If all 8 voices are currently held, steals the oldest held note.
  - Headroom normalization: Active voices are dynamically scaled by `1 / (1 + (voices - 1) * 0.35)` to prevent digital summing clipping while keeping single notes punchy.

## 5. Effects Architecture

### 5.1 Soft Saturation / Warm Overdrive
- Symmetric cubic soft-clipping polynomial saturation curve.
- Drive parameter `0.0` (clean bypass) to `1.0` (rich warm tube-style overdrive).
- Automatic makeup gain attenuation to prevent volume surges.

### 5.2 Roland Juno-Style Stereo Chorus
- Emulation of the classic Roland Juno-60 BBD (Bucket Brigade Device) analog chorus.
- Modes:
  - Mode I: Slow ensemble chorusing (`~0.51 Hz`, `1.2 ms` depth, `5.2 ms` delay center).
  - Mode II: Deep, lush spatial chorusing (`~0.86 Hz`, `2.4 ms` depth, `5.8 ms` delay center).
  - Mode I+II: Fast, expressive rotary vibrato (`~8.0 Hz`, `0.9 ms` depth).
- Quadrature LFO modulation (Left at phase `0°`, Right at phase `180°`) for ultra-wide stereo imaging.
- Fractional sample interpolation with zero allocation on a fixed ringbuffer.

### 5.3 Stereo Tape Delay
- Tape-modeled delay line with configurable delay time (`20 ms` to `1.0 s`).
- Soft-clipped feedback loop (`0.0` to `0.85`) to avoid runaway instability.
- Analog tape tone control: 1-pole lowpass filter in the feedback loop simulating high-frequency absorption on successive tape repeats.
- Stereo ping-pong cross-feedback: Left channel tapped at base delay time, Right channel tapped at `3/4` (dotted eighth feel) with cross-channel feedback for spatial depth.

