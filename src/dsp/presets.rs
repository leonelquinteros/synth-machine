use serde::{Deserialize, Serialize};

use super::effects::ChorusMode;
use super::lfo::{LfoTarget, LfoWaveform};
use super::params::{FilterMode, SynthParams, VoiceMode, Waveform};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub category: String,
    pub params: SynthParams,
}

pub struct PresetManager {
    presets: Vec<Preset>,
    current_index: usize,
}

impl PresetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            presets: Vec::new(),
            current_index: 0,
        };
        manager.load_factory_presets();
        manager
    }

    fn load_factory_presets(&mut self) {
        self.presets = vec![
            // ── Basic ────────────────────────────────────────────────────────
            Preset {
                name: "Init Patch".to_string(),
                category: "Basic".to_string(),
                params: SynthParams::default(),
            },

            // ── Minimoog Model D ─────────────────────────────────────────────
            Preset {
                name: "Minimoog Model D Bass".to_string(),
                category: "Bass".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: -1,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: -1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 4.5,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 380.0,
                    filter_resonance: 3.4,
                    filter_env_depth: 2.8,
                    filter_attack: 0.002,
                    filter_decay: 0.28,
                    filter_sustain: 0.05,
                    filter_release: 0.12,
                    amp_attack: 0.002,
                    amp_decay: 0.35,
                    amp_sustain: 0.45,
                    amp_release: 0.12,
                    drive: 0.28,
                    glide_time: 0.02,
                    ..Default::default()
                },
            },
            Preset {
                name: "Minimoog 70s Lead".to_string(),
                category: "Lead".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 7.0,
                    osc_mix: 0.45,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 1650.0,
                    filter_resonance: 4.5,
                    filter_env_depth: 1.5,
                    filter_attack: 0.005,
                    filter_decay: 0.35,
                    filter_sustain: 0.4,
                    filter_release: 0.2,
                    amp_attack: 0.005,
                    amp_decay: 0.3,
                    amp_sustain: 0.8,
                    amp_release: 0.2,
                    lfo_rate: 5.2,
                    lfo_depth: 0.15,
                    lfo_waveform: LfoWaveform::Sine,
                    lfo_target: LfoTarget::Pitch,
                    glide_time: 0.045,
                    drive: 0.25,
                    delay_time: 0.28,
                    delay_feedback: 0.35,
                    delay_tone: 0.6,
                    delay_mix: 0.22,
                    ..Default::default()
                },
            },

            // ── Roland Juno-60 / Juno-106 ────────────────────────────────────
            Preset {
                name: "Juno-106 Lush Strings".to_string(),
                category: "Pad".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: -1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 0.0,
                    osc_mix: 0.25, // sub-oscillator weight
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 1750.0,
                    filter_resonance: 1.6,
                    filter_env_depth: 0.9,
                    filter_attack: 0.45,
                    filter_decay: 0.8,
                    filter_sustain: 0.7,
                    filter_release: 0.85,
                    amp_attack: 0.35,
                    amp_decay: 0.6,
                    amp_sustain: 0.9,
                    amp_release: 0.85,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.85,
                    ..Default::default()
                },
            },
            Preset {
                name: "Juno-60 Hoover PWM".to_string(),
                category: "Pad".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Square,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.30,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 11.5,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 2200.0,
                    filter_resonance: 2.2,
                    amp_attack: 0.2,
                    amp_decay: 0.45,
                    amp_sustain: 0.85,
                    amp_release: 0.75,
                    lfo_rate: 0.85,
                    lfo_depth: 0.35,
                    lfo_waveform: LfoWaveform::Triangle,
                    lfo_target: LfoTarget::Filter,
                    chorus_mode: ChorusMode::Mode2,
                    chorus_mix: 0.80,
                    delay_time: 0.42,
                    delay_feedback: 0.40,
                    delay_tone: 0.55,
                    delay_mix: 0.30,
                    ..Default::default()
                },
            },
            Preset {
                name: "Juno Pop Pluck (Axel F)".to_string(),
                category: "Pluck".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 6.0,
                    osc_mix: 0.45,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 1200.0,
                    filter_resonance: 3.5,
                    filter_env_depth: 3.2,
                    filter_attack: 0.002,
                    filter_decay: 0.16,
                    filter_sustain: 0.0,
                    filter_release: 0.15,
                    amp_attack: 0.002,
                    amp_decay: 0.22,
                    amp_sustain: 0.1,
                    amp_release: 0.15,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.55,
                    delay_time: 0.25,
                    delay_feedback: 0.35,
                    delay_tone: 0.6,
                    delay_mix: 0.25,
                    ..Default::default()
                },
            },
            Preset {
                name: "Juno-106 Solid Bass".to_string(),
                category: "Bass".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Square,
                    osc1_octave: -1,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: -1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 3.0,
                    osc_mix: 0.35,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 650.0,
                    filter_resonance: 2.8,
                    filter_env_depth: 2.4,
                    filter_attack: 0.002,
                    filter_decay: 0.18,
                    filter_sustain: 0.1,
                    filter_release: 0.12,
                    amp_attack: 0.002,
                    amp_decay: 0.25,
                    amp_sustain: 0.4,
                    amp_release: 0.12,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.40,
                    drive: 0.15,
                    glide_time: 0.015,
                    ..Default::default()
                },
            },

            // ── Sequential Circuits Prophet-5 ────────────────────────────────
            Preset {
                name: "Prophet-5 Poly Brass".to_string(),
                category: "Brass".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 8.5,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 1400.0,
                    filter_resonance: 2.6,
                    filter_env_depth: 2.4,
                    filter_attack: 0.06,
                    filter_decay: 0.45,
                    filter_sustain: 0.30,
                    filter_release: 0.35,
                    amp_attack: 0.035,
                    amp_decay: 0.40,
                    amp_sustain: 0.75,
                    amp_release: 0.30,
                    drive: 0.12,
                    delay_time: 0.32,
                    delay_feedback: 0.25,
                    delay_tone: 0.6,
                    delay_mix: 0.20,
                    ..Default::default()
                },
            },
            Preset {
                name: "Prophet Fifth Sync Lead".to_string(),
                category: "Lead".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: 0,
                    osc2_semitone: 7, // Perfect 5th interval
                    osc2_detune_cents: 3.5,
                    osc_mix: 0.45,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 2600.0,
                    filter_resonance: 3.8,
                    filter_env_depth: 1.8,
                    filter_attack: 0.005,
                    filter_decay: 0.3,
                    filter_sustain: 0.35,
                    filter_release: 0.2,
                    amp_attack: 0.005,
                    amp_decay: 0.3,
                    amp_sustain: 0.6,
                    amp_release: 0.2,
                    glide_time: 0.04,
                    drive: 0.32,
                    delay_time: 0.28,
                    delay_feedback: 0.42,
                    delay_tone: 0.5,
                    delay_mix: 0.30,
                    ..Default::default()
                },
            },
            Preset {
                name: "Prophet Ambient Pad".to_string(),
                category: "Pad".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Triangle,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 10.0,
                    osc_mix: 0.4,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 950.0,
                    filter_resonance: 1.8,
                    filter_env_depth: 1.2,
                    filter_attack: 0.5,
                    filter_decay: 0.8,
                    filter_sustain: 0.6,
                    filter_release: 1.2,
                    amp_attack: 0.4,
                    amp_decay: 0.6,
                    amp_sustain: 0.85,
                    amp_release: 1.2,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.60,
                    delay_time: 0.48,
                    delay_feedback: 0.55,
                    delay_tone: 0.4,
                    delay_mix: 0.40,
                    ..Default::default()
                },
            },

            // ── Oberheim OB-Xa ───────────────────────────────────────────────
            Preset {
                name: "OB-Xa 'Jump' Brass".to_string(),
                category: "Brass".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 12.5,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 3200.0,
                    filter_resonance: 2.2,
                    filter_env_depth: 1.6,
                    filter_attack: 0.015,
                    filter_decay: 0.35,
                    filter_sustain: 0.45,
                    filter_release: 0.25,
                    amp_attack: 0.01,
                    amp_decay: 0.35,
                    amp_sustain: 0.80,
                    amp_release: 0.25,
                    drive: 0.18,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.25,
                    ..Default::default()
                },
            },
            Preset {
                name: "Oberheim Warm Sweep".to_string(),
                category: "Pad".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 9.0,
                    osc_mix: 0.4,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 650.0,
                    filter_resonance: 4.2,
                    filter_env_depth: 2.8,
                    filter_attack: 0.6,
                    filter_decay: 0.8,
                    filter_sustain: 0.4,
                    filter_release: 0.9,
                    amp_attack: 0.3,
                    amp_decay: 0.5,
                    amp_sustain: 0.8,
                    amp_release: 0.9,
                    chorus_mode: ChorusMode::Mode2,
                    chorus_mix: 0.65,
                    delay_time: 0.38,
                    delay_feedback: 0.45,
                    delay_tone: 0.5,
                    delay_mix: 0.35,
                    ..Default::default()
                },
            },

            // ── Yamaha CS-80 ─────────────────────────────────────────────────
            Preset {
                name: "CS-80 Blade Runner Lead".to_string(),
                category: "Lead".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 6.5,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 850.0,
                    filter_resonance: 3.2,
                    filter_env_depth: 2.6,
                    filter_attack: 0.22,
                    filter_decay: 0.6,
                    filter_sustain: 0.65,
                    filter_release: 0.7,
                    amp_attack: 0.18,
                    amp_decay: 0.5,
                    amp_sustain: 0.85,
                    amp_release: 0.7,
                    lfo_rate: 4.6,
                    lfo_depth: 0.18,
                    lfo_waveform: LfoWaveform::Sine,
                    lfo_target: LfoTarget::Pitch,
                    delay_time: 0.38,
                    delay_feedback: 0.50,
                    delay_tone: 0.55,
                    delay_mix: 0.45,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.40,
                    ..Default::default()
                },
            },
            Preset {
                name: "CS-80 Vangelis Brass".to_string(),
                category: "Brass".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: -1,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 8.0,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 1100.0,
                    filter_resonance: 2.5,
                    filter_env_depth: 2.0,
                    filter_attack: 0.12,
                    filter_decay: 0.5,
                    filter_sustain: 0.5,
                    filter_release: 0.5,
                    amp_attack: 0.08,
                    amp_decay: 0.4,
                    amp_sustain: 0.8,
                    amp_release: 0.5,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.50,
                    delay_time: 0.33,
                    delay_feedback: 0.40,
                    delay_tone: 0.6,
                    delay_mix: 0.30,
                    ..Default::default()
                },
            },

            // ── Roland TB-303 Bass Line ──────────────────────────────────────
            Preset {
                name: "TB-303 Acid Saw Bassline".to_string(),
                category: "Bass".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: -1,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: -1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 0.0,
                    osc_mix: 0.0, // pure OSC1 saw
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 380.0,
                    filter_resonance: 10.5,
                    filter_env_depth: 3.8,
                    filter_attack: 0.003,
                    filter_decay: 0.22,
                    filter_sustain: 0.0,
                    filter_release: 0.15,
                    amp_attack: 0.003,
                    amp_decay: 0.25,
                    amp_sustain: 0.35,
                    amp_release: 0.15,
                    glide_time: 0.065,
                    drive: 0.52,
                    delay_time: 0.26,
                    delay_feedback: 0.40,
                    delay_tone: 0.5,
                    delay_mix: 0.28,
                    ..Default::default()
                },
            },
            Preset {
                name: "TB-303 Square Squelch".to_string(),
                category: "Bass".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Square,
                    osc1_octave: -1,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: -1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 0.0,
                    osc_mix: 0.0,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 420.0,
                    filter_resonance: 9.8,
                    filter_env_depth: 3.4,
                    filter_attack: 0.002,
                    filter_decay: 0.18,
                    filter_sustain: 0.0,
                    filter_release: 0.12,
                    amp_attack: 0.002,
                    amp_decay: 0.22,
                    amp_sustain: 0.25,
                    amp_release: 0.12,
                    glide_time: 0.055,
                    drive: 0.45,
                    delay_time: 0.30,
                    delay_feedback: 0.35,
                    delay_tone: 0.55,
                    delay_mix: 0.22,
                    ..Default::default()
                },
            },

            // ── ARP Odyssey & 2600 ───────────────────────────────────────────
            Preset {
                name: "ARP Odyssey Funk Lead".to_string(),
                category: "Lead".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Square,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.30,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 8.0,
                    osc_mix: 0.45,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 1900.0,
                    filter_resonance: 5.2,
                    filter_env_depth: 2.2,
                    filter_attack: 0.003,
                    filter_decay: 0.25,
                    filter_sustain: 0.2,
                    filter_release: 0.15,
                    amp_attack: 0.003,
                    amp_decay: 0.25,
                    amp_sustain: 0.7,
                    amp_release: 0.15,
                    glide_time: 0.045,
                    drive: 0.25,
                    delay_time: 0.24,
                    delay_feedback: 0.35,
                    delay_tone: 0.6,
                    delay_mix: 0.25,
                    ..Default::default()
                },
            },
            Preset {
                name: "ARP 2600 Sci-Fi Laser".to_string(),
                category: "FX".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: 1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 15.0,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::BandPass,
                    filter_cutoff: 2200.0,
                    filter_resonance: 8.5,
                    filter_env_depth: -2.5,
                    filter_attack: 0.002,
                    filter_decay: 0.35,
                    filter_sustain: 0.0,
                    filter_release: 0.2,
                    amp_attack: 0.002,
                    amp_decay: 0.35,
                    amp_sustain: 0.0,
                    amp_release: 0.2,
                    lfo_rate: 14.5,
                    lfo_depth: 0.85,
                    lfo_waveform: LfoWaveform::Triangle,
                    lfo_target: LfoTarget::Filter,
                    drive: 0.35,
                    delay_time: 0.32,
                    delay_feedback: 0.60,
                    delay_tone: 0.7,
                    delay_mix: 0.45,
                    ..Default::default()
                },
            },

            // ── Korg Polysix & MS-20 ─────────────────────────────────────────
            Preset {
                name: "MS-20 Aggressive Screamer".to_string(),
                category: "Lead".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 16.0,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 1700.0,
                    filter_resonance: 8.2,
                    filter_env_depth: 2.2,
                    filter_attack: 0.005,
                    filter_decay: 0.3,
                    filter_sustain: 0.4,
                    filter_release: 0.2,
                    amp_attack: 0.005,
                    amp_decay: 0.3,
                    amp_sustain: 0.7,
                    amp_release: 0.2,
                    lfo_rate: 6.2,
                    lfo_depth: 0.20,
                    lfo_waveform: LfoWaveform::Square,
                    lfo_target: LfoTarget::Pitch,
                    glide_time: 0.035,
                    drive: 0.65,
                    delay_time: 0.26,
                    delay_feedback: 0.30,
                    delay_tone: 0.5,
                    delay_mix: 0.25,
                    ..Default::default()
                },
            },
            Preset {
                name: "Polysix Arp Pluck".to_string(),
                category: "Pluck".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Square,
                    osc2_octave: -1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 4.0,
                    osc_mix: 0.3,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 2100.0,
                    filter_resonance: 3.2,
                    filter_env_depth: 2.5,
                    filter_attack: 0.002,
                    filter_decay: 0.18,
                    filter_sustain: 0.0,
                    filter_release: 0.15,
                    amp_attack: 0.002,
                    amp_decay: 0.22,
                    amp_sustain: 0.05,
                    amp_release: 0.15,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.70,
                    delay_time: 0.25,
                    delay_feedback: 0.45,
                    delay_tone: 0.6,
                    delay_mix: 0.35,
                    ..Default::default()
                },
            },

            // ── Classic Electric Keys ────────────────────────────────────────
            Preset {
                name: "Analog Electric Piano".to_string(),
                category: "Keys".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Sine,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Triangle,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 5.0,
                    osc_mix: 0.4,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 1600.0,
                    filter_resonance: 1.8,
                    filter_env_depth: 1.8,
                    filter_attack: 0.002,
                    filter_decay: 0.45,
                    filter_sustain: 0.2,
                    filter_release: 0.35,
                    amp_attack: 0.002,
                    amp_decay: 0.60,
                    amp_sustain: 0.4,
                    amp_release: 0.35,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.45,
                    delay_time: 0.30,
                    delay_feedback: 0.25,
                    delay_tone: 0.5,
                    delay_mix: 0.20,
                    drive: 0.12,
                    ..Default::default()
                },
            },
            Preset {
                name: "Vintage Clavinet D6".to_string(),
                category: "Keys".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Square,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.85, // narrow pulse
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 6.0,
                    osc_mix: 0.35,
                    filter_mode: FilterMode::BandPass,
                    filter_cutoff: 2800.0,
                    filter_resonance: 4.8,
                    filter_env_depth: 2.0,
                    filter_attack: 0.002,
                    filter_decay: 0.14,
                    filter_sustain: 0.0,
                    filter_release: 0.10,
                    amp_attack: 0.002,
                    amp_decay: 0.16,
                    amp_sustain: 0.0,
                    amp_release: 0.10,
                    drive: 0.28,
                    chorus_mode: ChorusMode::Mode1,
                    chorus_mix: 0.30,
                    ..Default::default()
                },
            },

            // ── Modern & Bass Soundtracks ────────────────────────────────────
            Preset {
                name: "Punchy 808 Bass".to_string(),
                category: "Bass".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Mono,
                    osc1_waveform: Waveform::Sine,
                    osc1_octave: -1,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Triangle,
                    osc2_octave: -1,
                    osc2_semitone: 0,
                    osc2_detune_cents: 0.0,
                    osc_mix: 0.25,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 450.0,
                    filter_resonance: 1.8,
                    filter_env_depth: 2.5,
                    filter_attack: 0.002,
                    filter_decay: 0.22,
                    filter_sustain: 0.0,
                    filter_release: 0.15,
                    amp_attack: 0.002,
                    amp_decay: 0.45,
                    amp_sustain: 0.2,
                    amp_release: 0.15,
                    glide_time: 0.045,
                    drive: 0.25,
                    ..Default::default()
                },
            },
            Preset {
                name: "Synthwave Lead".to_string(),
                category: "Lead".to_string(),
                params: SynthParams {
                    voice_mode: VoiceMode::Poly8,
                    osc1_waveform: Waveform::Saw,
                    osc1_octave: 0,
                    osc1_pulse_width: 0.5,
                    osc2_waveform: Waveform::Saw,
                    osc2_octave: 0,
                    osc2_semitone: 0,
                    osc2_detune_cents: 14.0,
                    osc_mix: 0.5,
                    filter_mode: FilterMode::LowPass,
                    filter_cutoff: 3500.0,
                    filter_resonance: 2.8,
                    filter_env_depth: 1.2,
                    filter_attack: 0.01,
                    filter_decay: 0.4,
                    filter_sustain: 0.4,
                    filter_release: 0.3,
                    amp_attack: 0.01,
                    amp_decay: 0.3,
                    amp_sustain: 0.6,
                    amp_release: 0.25,
                    lfo_rate: 5.5,
                    lfo_depth: 0.22,
                    lfo_waveform: LfoWaveform::Sine,
                    lfo_target: LfoTarget::Pitch,
                    glide_time: 0.05,
                    drive: 0.2,
                    chorus_mode: ChorusMode::Mode1And2,
                    chorus_mix: 0.5,
                    delay_time: 0.375,
                    delay_feedback: 0.45,
                    delay_tone: 0.65,
                    delay_mix: 0.35,
                    ..Default::default()
                },
            },
        ];
    }

    pub fn presets(&self) -> &[Preset] {
        &self.presets
    }

    pub fn current_preset(&self) -> &Preset {
        &self.presets[self.current_index]
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn select_preset(&mut self, index: usize) -> Option<SynthParams> {
        if index < self.presets.len() {
            self.current_index = index;
            Some(self.presets[index].params)
        } else {
            None
        }
    }

    pub fn save_preset(&mut self, name: String, category: String, params: SynthParams) -> usize {
        let preset = Preset {
            name,
            category,
            params,
        };
        self.presets.push(preset);
        self.current_index = self.presets.len() - 1;
        self.current_index
    }

    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.presets)
    }

    pub fn import_json(&mut self, json_str: &str) -> Result<usize, serde_json::Error> {
        let imported: Vec<Preset> = serde_json::from_str(json_str)?;
        let count = imported.len();
        self.presets.extend(imported);
        Ok(count)
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_manager_factory_presets() {
        let manager = PresetManager::new();
        assert!(manager.presets().len() >= 20);
        assert_eq!(manager.current_preset().name, "Init Patch");
    }

    #[test]
    fn test_select_preset() {
        let mut manager = PresetManager::new();
        let bass_params = manager.select_preset(1);
        assert!(bass_params.is_some());
        assert_eq!(manager.current_preset().name, "Minimoog Model D Bass");
    }

    #[test]
    fn test_save_and_serialize_presets() {
        let mut manager = PresetManager::new();
        let custom_params = SynthParams {
            filter_cutoff: 4321.0,
            ..Default::default()
        };
        let new_idx = manager.save_preset("My Custom Sound".to_string(), "Custom".to_string(), custom_params);
        assert_eq!(manager.current_index(), new_idx);
        assert_eq!(manager.current_preset().name, "My Custom Sound");

        let json = manager.export_json().expect("JSON serialization failed");
        assert!(json.contains("My Custom Sound"));
        assert!(json.contains("4321"));

        // Import test
        let mut new_manager = PresetManager::new();
        let initial_count = new_manager.presets().len();
        let imported = new_manager.import_json(&json).expect("JSON import failed");
        assert_eq!(new_manager.presets().len(), initial_count + imported);
    }
}
