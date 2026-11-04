# User Interface Specification

## 1. UI Philosophy

- **Simplicity First**: Minimal visual clutter, everything accessible on a single screen without deep sub-menus.
- **Immediate Responsiveness**: Knobs and sliders update parameters in real-time with zero lag.
- **Mouse & Keyboard Centric**:
  - Drag sliders / knobs vertically for fine parameter tuning.
  - Double-click to reset parameters to default values.
  - Interactive on-screen piano keys playable via mouse click.
  - Real-time QWERTY keyboard mapping for immediate live playing without external hardware.

## 2. Keyboard Key Mapping (QWERTY Piano)

Standard two-octave piano mapping (similar to Ableton Live / GarageBand):

```
Octave: [ Z (Down) ] [ X (Up) ]
Upper row (Black keys):     [W] [E]     [T] [Y] [U]     [O] [P]
Lower row (White keys):   [A] [S] [D] [F] [G] [H] [J] [K] [L] [;]
Note pitch:                C   D   E   F   G   A   B   C   D   E
```

## 3. UI Panels & Layout

1. **Header / Master Bar**:
   - Master volume slider.
   - Octave shift selector (`-2` to `+2`).
   - Active voice / note indicator.
   - Audio output device / status.

2. **Oscillator Section**:
   - Waveform picker (Saw, Square, Triangle, Sine, Noise).
   - Pulse Width slider (for square wave).
   - Pitch Coarse (semitones) & Fine detune (cents).

3. **Filter Section**:
   - Cutoff frequency (logarithmic curve from 20 Hz to 20 kHz).
   - Resonance (Q factor).
   - Filter mode selector (LPF, HPF, BPF).

4. **Envelopes Section**:
   - Amplitude ADSR sliders (Attack, Decay, Sustain, Release) with interactive curve preview.
   - Filter ADSR sliders & envelope modulation depth.

5. **Visualizer & Virtual Keyboard**:
   - Real-time audio waveform / oscilloscope display showing the actual generated output.
   - Visual piano keys highlighting currently triggered notes.
