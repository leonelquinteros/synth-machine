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

## 4. Device Management

### Audio Output Device Selection
- `AudioSystem::list_output_devices() -> Vec<String>` — enumerates all output device names on the default `cpal` host. Called each frame the settings panel is open (cheap — does not open any stream).
- `AudioSystem::new_with_device(device_name: Option<&str>) -> Result<Self, _>` — builds a complete new audio stream on a named device. `None` falls back to the system default. Used by `AudioSystem::new()` and by the settings panel when the user picks a new device.
- **Stream rebuild**: when the user selects a different output device in the UI, `SynthApp` calls `new_with_device`, replaces `self.audio`, and immediately sends a `SynthEvent::UpdateParams` with the current `SynthParams` so the fresh DSP engine starts with the correct state. The old `Stream` is dropped (and CoreAudio resources released) automatically via `Drop`.

### MIDI Input Port Selection
- `list_midi_input_ports() -> Vec<String>` — free function that opens a temporary `MidiInput` handle to enumerate ports, then drops it. Safe to call from the UI thread.
- `MidiManager::connect_to_port_index(port_index: usize)` — drops the current `MidiInputConnection` (if any) and opens a fresh one on the port at the given index. The `Sender<SynthEvent>` is stored inside `MidiManager` so reconnection is self-contained and requires no arguments from the caller beyond the port index.
- **Hot-swap**: because `MidiInputConnection` owns a background thread for MIDI I/O, dropping the old connection terminates that thread cleanly before the new one is started.

