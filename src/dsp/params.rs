use serde::{Deserialize, Serialize};

use crate::dsp::effects::ChorusMode;
use crate::dsp::lfo::{LfoTarget, LfoWaveform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceMode {
    Mono,
    Poly8,
}

impl VoiceMode {
    pub const ALL: [VoiceMode; 2] = [VoiceMode::Mono, VoiceMode::Poly8];

    pub fn name(&self) -> &'static str {
        match self {
            VoiceMode::Mono => "Mono (Legato)",
            VoiceMode::Poly8 => "Poly (8-Voice)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Waveform {
    Saw,
    Square,
    Triangle,
    Sine,
}

impl Waveform {
    pub const ALL: [Waveform; 4] = [
        Waveform::Saw,
        Waveform::Square,
        Waveform::Triangle,
        Waveform::Sine,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Waveform::Saw => "Saw",
            Waveform::Square => "Square / Pulse",
            Waveform::Triangle => "Triangle",
            Waveform::Sine => "Sine",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
}

impl FilterMode {
    pub const ALL: [FilterMode; 3] = [
        FilterMode::LowPass,
        FilterMode::HighPass,
        FilterMode::BandPass,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            FilterMode::LowPass => "Low Pass (LPF)",
            FilterMode::HighPass => "High Pass (HPF)",
            FilterMode::BandPass => "Band Pass (BPF)",
        }
    }
}

/// Global synthesis parameters shared from UI to DSP
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SynthParams {
    pub master_volume: f32, // 0.0 .. 1.0
    pub octave_shift: i32,  // Global master octave shift: -2 .. +2

    // OSC 1 (Primary Oscillator)
    pub osc1_waveform: Waveform,
    pub osc1_pulse_width: f32, // 0.05 .. 0.95
    pub osc1_octave: i32,      // -2 .. +2

    // OSC 2 (Secondary / Detune Oscillator)
    pub osc2_waveform: Waveform,
    pub osc2_pulse_width: f32,  // 0.05 .. 0.95
    pub osc2_octave: i32,       // -2 .. +2
    pub osc2_semitone: i32,     // -12 .. +12 semitones (e.g. +7 fifth)
    pub osc2_detune_cents: f32, // -50.0 .. +50.0 cents

    // Oscillator Mixer
    pub osc_mix: f32, // 0.0 (100% OSC1) .. 1.0 (100% OSC2)

    // Filter
    pub filter_cutoff: f32,    // 20.0 .. 20000.0 Hz
    pub filter_resonance: f32, // 0.5 .. 15.0
    pub filter_mode: FilterMode,

    // Filter ADSR Envelope (modulates cutoff)
    pub filter_attack: f32,   // 0.001 .. 3.0 s
    pub filter_decay: f32,    // 0.001 .. 3.0 s
    pub filter_sustain: f32,  // 0.0 .. 1.0
    pub filter_release: f32,  // 0.001 .. 5.0 s
    /// Modulation depth in octaves of cutoff movement (−4 .. +4)
    pub filter_env_depth: f32,

    // Amplitude ADSR Envelope
    pub amp_attack: f32,  // 0.001 .. 3.0 s
    pub amp_decay: f32,   // 0.001 .. 3.0 s
    pub amp_sustain: f32, // 0.0 .. 1.0
    pub amp_release: f32, // 0.001 .. 5.0 s

    // LFO
    pub lfo_rate: f32,          // 0.01 .. 20.0 Hz
    pub lfo_depth: f32,         // 0.0 .. 1.0  (interpretation depends on target)
    pub lfo_waveform: LfoWaveform,
    pub lfo_target: LfoTarget,

    // Portamento / Glide
    /// Glide time in seconds (0.0 = instant / off)
    pub glide_time: f32, // 0.0 .. 2.0 s

    // ── Phase 4: Polyphony & Effects ─────────────────────────────────────────
    pub voice_mode: VoiceMode,

    // Soft Saturation / Warm Overdrive
    pub drive: f32, // 0.0 (clean) .. 1.0 (warm saturated overdrive)

    // Roland Juno-style Stereo Chorus
    pub chorus_mode: ChorusMode,
    pub chorus_mix: f32, // 0.0 .. 1.0

    // Stereo Tape Delay
    pub delay_time: f32,     // 0.02 .. 1.0 s
    pub delay_feedback: f32, // 0.0 .. 0.85
    pub delay_tone: f32,     // 0.1 .. 1.0 (tape high-frequency damping)
    pub delay_mix: f32,      // 0.0 .. 1.0
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            octave_shift: 0,

            osc1_waveform: Waveform::Saw,
            osc1_pulse_width: 0.5,
            osc1_octave: 0,

            osc2_waveform: Waveform::Square,
            osc2_pulse_width: 0.5,
            osc2_octave: 0,
            osc2_semitone: 0,
            osc2_detune_cents: 6.0, // slight detune for classic warm analog chorusing

            osc_mix: 0.5, // 50/50 balance

            filter_cutoff: 2500.0,
            filter_resonance: 2.0,
            filter_mode: FilterMode::LowPass,

            filter_attack: 0.01,
            filter_decay: 0.3,
            filter_sustain: 0.0,
            filter_release: 0.3,
            filter_env_depth: 0.0, // disabled by default

            amp_attack: 0.01,
            amp_decay: 0.2,
            amp_sustain: 0.6,
            amp_release: 0.25,

            lfo_rate: 4.0,
            lfo_depth: 0.0, // disabled by default
            lfo_waveform: LfoWaveform::Sine,
            lfo_target: LfoTarget::Pitch,

            glide_time: 0.0, // off by default

            voice_mode: VoiceMode::Mono,
            drive: 0.0,
            chorus_mode: ChorusMode::Off,
            chorus_mix: 0.5,
            delay_time: 0.35,
            delay_feedback: 0.35,
            delay_tone: 0.6,
            delay_mix: 0.0,
        }
    }
}

/// Live musical events sent to the audio engine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SynthEvent {
    NoteOn { note: u8, velocity: f32 },
    NoteOff { note: u8 },
    UpdateParams(SynthParams),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_properties() {
        assert_eq!(Waveform::ALL.len(), 4);
        for &wf in &Waveform::ALL {
            assert!(!wf.name().is_empty());
        }
        assert_eq!(Waveform::Saw.name(), "Saw");
        assert_eq!(Waveform::Square.name(), "Square / Pulse");
        assert_eq!(Waveform::Triangle.name(), "Triangle");
        assert_eq!(Waveform::Sine.name(), "Sine");
    }

    #[test]
    fn test_filter_mode_properties() {
        assert_eq!(FilterMode::ALL.len(), 3);
        for &mode in &FilterMode::ALL {
            assert!(!mode.name().is_empty());
        }
        assert_eq!(FilterMode::LowPass.name(), "Low Pass (LPF)");
        assert_eq!(FilterMode::HighPass.name(), "High Pass (HPF)");
        assert_eq!(FilterMode::BandPass.name(), "Band Pass (BPF)");
    }

    #[test]
    fn test_default_synth_params_sanity() {
        let params = SynthParams::default();
        assert!(params.master_volume > 0.0 && params.master_volume <= 1.0);
        assert_eq!(params.octave_shift, 0);

        assert_eq!(params.osc1_waveform, Waveform::Saw);
        assert_eq!(params.osc1_pulse_width, 0.5);
        assert_eq!(params.osc1_octave, 0);

        assert_eq!(params.osc2_waveform, Waveform::Square);
        assert_eq!(params.osc2_pulse_width, 0.5);
        assert_eq!(params.osc2_octave, 0);
        assert_eq!(params.osc2_semitone, 0);
        assert_eq!(params.osc2_detune_cents, 6.0);

        assert_eq!(params.osc_mix, 0.5);

        assert!(params.filter_cutoff >= 20.0 && params.filter_cutoff <= 20000.0);
        assert!(params.filter_resonance >= 0.5);
        assert_eq!(params.filter_mode, FilterMode::LowPass);

        assert!(params.amp_attack > 0.0);
        assert!(params.amp_decay > 0.0);
        assert!(params.amp_sustain > 0.0 && params.amp_sustain <= 1.0);
        assert!(params.amp_release > 0.0);
    }
}
