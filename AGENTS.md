# AGENTS.md - Agent Bootstrap & Developer Guide

Welcome to the **Analog Synth Emulator** repository. This document is the starting point for AI agents and human developers working on this project.

## 1. Project Overview

This is an analog synthesizer emulator built in Rust designed for low-latency live performance. It features:
- A real-time DSP audio engine powered by `cpal` (running natively on macOS CoreAudio).
- An immediate-mode native graphical interface powered by `egui` (`eframe`).
- Lock-free, real-time-safe inter-thread communication between the UI and audio threads.
- Comprehensive technical specs in the [`specs/`](specs/) directory.

## 2. Prerequisites & Environment

- **Rust toolchain**: 1.99+ (stable).
- **Platform**: macOS (uses CoreAudio directly; cross-platform capable via `cpal`).
- **Dependencies**: No external C libraries required; all dependencies are managed via Cargo.

## 3. Quick Start Commands

```bash
# Run in development mode
cargo run

# Build release binary (optimized for live performance DSP)
cargo build --release

# Run tests
cargo test

# Check code formatting & lints
cargo check
cargo clippy
```

## 4. Directory Structure

```
synth/
├── AGENTS.md             # This file (Agent bootstrap guide)
├── GEMINI.md             # Gemini-specific reference pointing to AGENTS.md
├── CLAUDE.md             # Claude-specific reference pointing to AGENTS.md
├── Cargo.toml            # Project manifest & dependencies
├── specs/                # Authoritative design specifications
│   ├── README.md         # Specs index
│   ├── architecture.md   # Threading model & communication
│   ├── dsp_spec.md       # Oscillators, filters, envelopes
│   ├── ui_spec.md        # UI layout & keyboard mapping
│   └── roadmap.md        # Feature phases & milestones
└── src/
    ├── main.rs           # Application entrypoint
    ├── audio/            # cpal stream setup & CoreAudio bridge
    ├── dsp/              # DSP algorithms (oscillator, filter, envelope, voice)
    └── ui/               # egui application views, controls, and piano
```

## 5. Critical Development Guidelines

1. **Audio Thread Discipline (STRICT)**:
   - The audio callback runs in a high-priority real-time thread.
   - **NEVER** allocate memory in the audio callback (`Vec::push`, `Box::new`, `String`, etc.).
   - **NEVER** take blocking mutexes (`std::sync::Mutex`) or perform channel operations that can block. Use lock-free atomics or bounded ringbuffers (`crossbeam_channel::TryRecv`, `rtrb`).
   - **NEVER** perform file I/O, logging, or system calls inside the audio callback.

2. **UI & State Synchronization**:
   - UI runs in `egui` (immediate mode). Keep UI lightweight and update parameters via atomic representations or message channels.
   - Parameters should be smoothed or interpolated in DSP to avoid audible clicks when moving sliders/knobs.

3. **Specifications First**:
   - Check and adhere to the documents in [`specs/`](specs/) before modifying architectural or DSP behaviors. Update the specs if new requirements or algorithms are introduced.

4. **Implementation Tracking**:
   - Do not commit transient implementation scratchpads or temporary task checklists into the repository root. Keep repository code clean and documented.
