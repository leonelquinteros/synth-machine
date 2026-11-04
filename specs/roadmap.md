# Synthesizer Feature Roadmap

## Phase 1: MVP (✅ Completed)
- [x] Specs documentation & bootstrap guide (`AGENTS.md`, `GEMINI.md`, `CLAUDE.md`).
- [x] Core DSP audio pipeline:
  - [x] Single PolyBLEP anti-aliased oscillator (Saw, Square/PWM, Triangle, Sine).
  - [x] Resonant State Variable Filter (SVF Low-Pass, High-Pass, Band-Pass).
  - [x] Sample-accurate Amp ADSR envelope.
- [x] Real-time audio engine with `cpal` (CoreAudio on macOS) and lock-free thread communication.
- [x] Native `egui` user interface:
  - [x] Parameter controls (Waveform, Filter Cutoff & Resonance, Amp ADSR, Master Volume).
  - [x] QWERTY keyboard live input mapping (2 octaves, Z/X octave shift).
  - [x] Interactive virtual keyboard on screen with active note glow.
- [x] Comprehensive test suite (49 unit & functional tests, 0 warnings).

## Phase 2: Live Performance & Controls (✅ Completed)
- [x] MIDI input via `midir` (USB MIDI keyboard auto-connect on launch).
- [x] Real-time oscilloscope / audio visualizer (phosphor-style CRT display).
- [x] Analog-style rotary knobs (drag acceleration, Shift=fine, double-click reset).
- [x] Preset save/load system (factory presets + JSON serialization).

## Phase 3: Classic Analog Enhancements (✅ Completed)
- [x] Dual oscillators (OSC 1 + OSC 2 with mix, octave shift, semitone interval, fine detune).
- [x] Bug fix: release tail pitch anchor — oscillator no longer drifts during note release.
- [x] Filter ADSR envelope modulation (envelope depth & filter envelope controls).
- [x] Low Frequency Oscillator (LFO) routable to Pitch (vibrato) or Cutoff (auto-wah/wobble).
- [x] Monophonic Portamento (smooth pitch glide between notes, glide time knob).
- [x] 8 curated analog factory presets demonstrating Phase 3 features.

## Phase 4: Polyphony & Effects (✅ Completed)
- [x] 8-voice polyphonic allocator with dynamic voice stealing & headroom scaling.
- [x] Monophonic & Polyphonic mode toggles (supporting legato glide in Mono, chords in Poly).
- [x] True stereo audio pipeline (`(f32, f32)` output callback).
- [x] Analog-style stereo chorus (Roland Juno BBD emulation with quadrature LFO).
- [x] Stereo tape delay with analog tone damping & cross-feedback ping-pong spread.
- [x] Soft saturation / warm tube overdrive with automatic makeup gain.
- [x] Dedicated Master Effects Rack in UI with real-time controls.
- [x] Factory presets upgraded to showcase polyphonic pads, stereo chorus, and tape delay.

## Phase 5: Device Management & Preset Library Expansion (✅ Completed)
- [x] Audio output device selection — enumerate all CoreAudio output devices, pick any at runtime (stream rebuilt transparently, DSP state preserved).
- [x] MIDI input port selection — enumerate all connected MIDI ports, hot-swap connection without restarting the app.
- [x] In-app ⚙ Settings panel in the header bar (collapsible, non-modal).
- [x] Inline error reporting for failed device opens (shown in header status row).
- [x] Factory preset library expanded to 23 presets spanning iconic hardware: Minimoog Model D, Roland Juno-60/106, Sequential Prophet-5, Oberheim OB-Xa, Yamaha CS-80, Roland TB-303, ARP Odyssey/2600, Korg Polysix/MS-20, and classic electric keys.
- [x] Preset dropdown upgraded to show category tags `[Category] Name` with wider ComboBox (210 px).

