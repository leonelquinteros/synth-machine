# System Architecture Specification

## 1. Threading Model

For live audio synthesis, thread discipline is non-negotiable:
- **Audio Thread (Real-Time)**:
  - Must never allocate memory on heap (`malloc`/`free`, `Vec::push`, `Box::new`).
  - Must never lock mutexes that could cause priority inversion or blocking.
  - Must never perform file I/O or network calls.
  - Interacts with macOS CoreAudio via `cpal`.
  - Consumes parameter changes and note events from lock-free queues.
  - Produces audio frames directly into the output buffer provided by `cpal`.
  - Optionally publishes small visualizer ringbuffer slices (oscilloscope/RMS) back to the UI thread using lock-free channels.

- **UI / Main Thread**:
  - Handles the window lifecycle, rendering via `eframe` (`egui`).
  - Captures user input (mouse drag on knobs/sliders, QWERTY keystrokes, UI clicks).
  - Listens to MIDI events (via `midir`) on an input callback thread and forwards them.
  - Pushes parameter updates and NoteOn/NoteOff events into the lock-free queue consumed by the audio thread.

## 2. Inter-Thread Communication

```
 ┌────────────────────────────────────────────────────────┐
 │                      Main / UI Thread                  │
 │                                                        │
 │  [egui GUI Controls]        [QWERTY Key Events]       │
 └─────────────┬──────────────────────────┬───────────────┘
               │                          │
               ▼                          ▼
       Parameter Updates              Note Events
       (Lock-free atomic/queue)       (Lock-free queue)
               │                          │
 ┌─────────────┴──────────────────────────┴───────────────┐
 │                      Audio Thread (cpal)               │
 │                                                        │
 │           [Synth Engine -> Voices -> DSP]              │
 │                          │                             │
 │                          ▼                             │
 │             [CoreAudio Output Buffer]                  │
 └──────────────────────────┬─────────────────────────────┘
                            │
                            ▼ (Waveform snapshot)
                   [Lock-free Ringbuffer]
                            │
                            ▼
                    [UI Oscilloscope]
```

## 3. Core Technologies

- **Language**: Rust (stable toolchain).
- **Audio Output**: `cpal` (CoreAudio on macOS, ALSA/JACK on Linux, WASAPI on Windows).
- **GUI**: `eframe` / `egui` (immediate-mode UI rendered via wgpu / Glow / Metal).
- **MIDI**: `midir` for cross-platform MIDI port enumeration and input polling.
- **Lock-free Data Structures**: `crossbeam-channel` or `rtrb` for bounded real-time audio communication.
