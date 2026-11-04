use super::effects::{DriveEffect, StereoChorus, TapeDelay};
use super::envelope::AdsrEnvelope;
use super::filter::StateVariableFilter;
use super::lfo::{Lfo, LfoTarget};
use super::oscillator::Oscillator;
use super::params::{SynthEvent, SynthParams, VoiceMode};

#[inline]
pub fn midi_to_freq(note: u8, octave_shift: i32) -> f32 {
    let shifted = (note as i32 + octave_shift * 12).clamp(0, 127) as f32;
    440.0 * 2.0_f32.powf((shifted - 69.0) / 12.0)
}

// ─────────────────────────────────────────────────────────────────────────────
//  SynthVoice — Individual Synthesis Voice
// ─────────────────────────────────────────────────────────────────────────────

pub struct SynthVoice {
    pub sample_rate: f32,
    pub oscillator1: Oscillator,
    pub oscillator2: Oscillator,
    pub filter: StateVariableFilter,
    pub amp_envelope: AdsrEnvelope,
    pub filter_envelope: AdsrEnvelope,
    pub lfo: Lfo,
    pub smoothed_cutoff: f32,
    pub smoothed_freq: f32,
    pub active_note: Option<u8>,
    pub release_note: u8,
    pub velocity: f32,
    pub age: u64,
}

impl SynthVoice {
    pub fn new(sample_rate: f32) -> Self {
        let init_freq = midi_to_freq(69, 0);
        Self {
            sample_rate,
            oscillator1: Oscillator::new(sample_rate),
            oscillator2: Oscillator::new(sample_rate),
            filter: StateVariableFilter::new(sample_rate),
            amp_envelope: AdsrEnvelope::new(sample_rate),
            filter_envelope: AdsrEnvelope::new(sample_rate),
            lfo: Lfo::new(sample_rate),
            smoothed_cutoff: 2500.0,
            smoothed_freq: init_freq,
            active_note: None,
            release_note: 69,
            velocity: 0.0,
            age: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.oscillator1.set_sample_rate(self.sample_rate);
        self.oscillator2.set_sample_rate(self.sample_rate);
        self.filter.set_sample_rate(self.sample_rate);
        self.amp_envelope.set_sample_rate(self.sample_rate);
        self.filter_envelope.set_sample_rate(self.sample_rate);
        self.lfo.set_sample_rate(self.sample_rate);
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.amp_envelope.is_active() || self.active_note.is_some()
    }

    pub fn note_on(
        &mut self,
        note: u8,
        velocity: f32,
        params: &SynthParams,
        age: u64,
        is_mono: bool,
    ) {
        let target_freq = midi_to_freq(note, params.octave_shift + params.osc1_octave);
        if !self.is_active() || (!is_mono && params.glide_time < 0.001) {
            self.smoothed_freq = target_freq;
        }
        self.active_note = Some(note);
        self.release_note = note;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.age = age;
        self.amp_envelope.gate_on();
        self.filter_envelope.gate_on();
        self.lfo.reset();
    }

    pub fn note_off(&mut self) {
        if let Some(n) = self.active_note {
            self.release_note = n;
        }
        self.active_note = None;
        self.amp_envelope.gate_off();
        self.filter_envelope.gate_off();
    }

    #[inline]
    pub fn process_sample(&mut self, params: &SynthParams, sample_rate: f32) -> f32 {
        if !self.amp_envelope.is_active() && self.active_note.is_none() {
            return 0.0;
        }

        let note = self.active_note.unwrap_or(self.release_note);
        let global_oct = params.octave_shift;

        // ── LFO ─────────────────────────────────────────────────────────────
        let lfo_val = self.lfo.process(params.lfo_rate, params.lfo_waveform);

        // ── Portamento ───────────────────────────────────────────────────────
        let target_freq = midi_to_freq(note, global_oct + params.osc1_octave);
        let alpha_glide = if params.glide_time < 0.001 {
            1.0
        } else {
            1.0 - (-1.0 / (sample_rate * params.glide_time)).exp()
        };
        self.smoothed_freq += alpha_glide * (target_freq - self.smoothed_freq);
        let freq1 = self.smoothed_freq;

        // ── LFO → Pitch (vibrato) ────────────────────────────────────────────
        let pitch_mod = if params.lfo_target == LfoTarget::Pitch {
            2.0_f32.powf(lfo_val * params.lfo_depth / 12.0)
        } else {
            1.0
        };

        // ── OSC 1 ────────────────────────────────────────────────────────────
        let wave1 = self.oscillator1.process(
            freq1 * pitch_mod,
            params.osc1_waveform,
            params.osc1_pulse_width,
        );

        // ── OSC 2 (detune relative to base) ──────────────────────────────────
        let osc2_octave_offset = (params.osc2_octave - params.osc1_octave) as f32;
        let detune_semitones =
            params.osc2_semitone as f32 + params.osc2_detune_cents / 100.0;
        let freq2 = freq1
            * 2.0_f32.powf(osc2_octave_offset)
            * 2.0_f32.powf(detune_semitones / 12.0)
            * pitch_mod;

        let wave2 = self.oscillator2.process(
            freq2,
            params.osc2_waveform,
            params.osc2_pulse_width,
        );

        // ── Oscillator mixer ─────────────────────────────────────────────────
        let mix = params.osc_mix.clamp(0.0, 1.0);
        let mixed = wave1 * (1.0 - mix) + wave2 * mix;

        // ── Filter cutoff smoothing ──────────────────────────────────────────
        let alpha_cut = 0.005;
        self.smoothed_cutoff += alpha_cut * (params.filter_cutoff - self.smoothed_cutoff);
        let mut effective_cutoff = self.smoothed_cutoff;

        // ── Filter envelope modulation ───────────────────────────────────────
        let filter_env = self.filter_envelope.process(
            params.filter_attack,
            params.filter_decay,
            params.filter_sustain,
            params.filter_release,
        );
        effective_cutoff *= 2.0_f32.powf(filter_env * params.filter_env_depth);
        effective_cutoff = effective_cutoff.clamp(20.0, 20000.0);

        // ── LFO → Filter (auto-wah) ──────────────────────────────────────────
        if params.lfo_target == LfoTarget::Filter {
            let lfo_oct = lfo_val * params.lfo_depth * 2.0;
            effective_cutoff = (effective_cutoff * 2.0_f32.powf(lfo_oct)).clamp(20.0, 20000.0);
        }

        // ── Resonant SVF filter ──────────────────────────────────────────────
        let filtered = self.filter.process(
            mixed,
            effective_cutoff,
            params.filter_resonance,
            params.filter_mode,
        );

        // ── Amplitude envelope ───────────────────────────────────────────────
        let amp_env = self.amp_envelope.process(
            params.amp_attack,
            params.amp_decay,
            params.amp_sustain,
            params.amp_release,
        );

        filtered * amp_env * self.velocity
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  SynthEngine — Polyphonic Engine & Effects Pipeline
// ─────────────────────────────────────────────────────────────────────────────

pub const MAX_VOICES: usize = 8;

pub struct SynthEngine {
    pub sample_rate: f32,
    pub voices: [SynthVoice; MAX_VOICES],
    pub params: SynthParams,

    // Note tracking & stack for monophonic priority and virtual keyboard
    pub note_stack: [u8; 16],
    pub note_stack_len: usize,
    pub active_note: Option<u8>,
    pub release_note: u8,
    pub smoothed_freq: f32,
    pub amp_envelope: AdsrEnvelope,
    pub voice_age_counter: u64,

    // Phase 4 Effects Pipeline
    pub drive_fx: DriveEffect,
    pub chorus_fx: StereoChorus,
    pub delay_fx: TapeDelay,
}

impl SynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        let params = SynthParams::default();
        let init_freq = midi_to_freq(69, 0);
        let voices = std::array::from_fn(|_| SynthVoice::new(sample_rate));
        Self {
            sample_rate,
            voices,
            params,
            note_stack: [0; 16],
            note_stack_len: 0,
            active_note: None,
            release_note: 69,
            smoothed_freq: init_freq,
            amp_envelope: AdsrEnvelope::new(sample_rate),
            voice_age_counter: 0,
            drive_fx: DriveEffect::new(),
            chorus_fx: StereoChorus::new(sample_rate),
            delay_fx: TapeDelay::new(sample_rate),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        for voice in &mut self.voices {
            voice.set_sample_rate(self.sample_rate);
        }
        self.amp_envelope.set_sample_rate(self.sample_rate);
        self.chorus_fx.set_sample_rate(self.sample_rate);
        self.delay_fx.set_sample_rate(self.sample_rate);
    }

    pub fn handle_event(&mut self, event: SynthEvent) {
        match event {
            SynthEvent::NoteOn { note, velocity } => {
                let velocity = velocity.clamp(0.0, 1.0);
                self.voice_age_counter = self.voice_age_counter.wrapping_add(1);
                let current_age = self.voice_age_counter;

                // Track held notes in stack (de-duplicated)
                if !self.note_stack[..self.note_stack_len].contains(&note)
                    && self.note_stack_len < self.note_stack.len()
                {
                    self.note_stack[self.note_stack_len] = note;
                    self.note_stack_len += 1;
                }
                self.active_note = Some(note);
                self.release_note = note;
                self.amp_envelope.gate_on();

                match self.params.voice_mode {
                    VoiceMode::Mono => {
                        self.voices[0].note_on(note, velocity, &self.params, current_age, true);
                        self.smoothed_freq = self.voices[0].smoothed_freq;
                    }
                    VoiceMode::Poly8 => {
                        let target_idx = if let Some(idx) = self.voices.iter().position(|v| v.active_note == Some(note)) {
                            idx
                        } else if let Some(idx) = self.voices.iter().position(|v| !v.is_active()) {
                            idx
                        } else {
                            // Steal voice: prefer releasing voice, then oldest held note
                            let releasing = self.voices.iter().enumerate()
                                .filter(|(_, v)| v.active_note.is_none())
                                .min_by_key(|(_, v)| v.age);
                            if let Some((idx, _)) = releasing {
                                idx
                            } else {
                                self.voices.iter().enumerate()
                                    .min_by_key(|(_, v)| v.age)
                                    .map(|(idx, _)| idx)
                                    .unwrap_or(0)
                            }
                        };
                        self.voices[target_idx].note_on(note, velocity, &self.params, current_age, false);
                        self.smoothed_freq = self.voices[target_idx].smoothed_freq;
                    }
                }
            }
            SynthEvent::NoteOff { note } => {
                // Remove from held note stack
                if let Some(pos) = self.note_stack[..self.note_stack_len]
                    .iter()
                    .position(|&n| n == note)
                {
                    for i in pos..(self.note_stack_len - 1) {
                        self.note_stack[i] = self.note_stack[i + 1];
                    }
                    self.note_stack_len -= 1;
                }

                match self.params.voice_mode {
                    VoiceMode::Mono => {
                        if self.note_stack_len > 0 {
                            let prev = self.note_stack[self.note_stack_len - 1];
                            self.active_note = Some(prev);
                            self.release_note = prev;
                            self.voices[0].note_on(prev, self.voices[0].velocity, &self.params, self.voice_age_counter, true);
                        } else {
                            if let Some(n) = self.active_note {
                                self.release_note = n;
                            }
                            self.active_note = None;
                            self.voices[0].note_off();
                            self.amp_envelope.gate_off();
                        }
                        self.smoothed_freq = self.voices[0].smoothed_freq;
                    }
                    VoiceMode::Poly8 => {
                        for voice in &mut self.voices {
                            if voice.active_note == Some(note) {
                                voice.note_off();
                            }
                        }
                        if self.note_stack_len > 0 {
                            let prev = self.note_stack[self.note_stack_len - 1];
                            self.active_note = Some(prev);
                        } else {
                            self.active_note = None;
                            self.amp_envelope.gate_off();
                        }
                    }
                }
            }
            SynthEvent::UpdateParams(new_params) => {
                self.params = new_params;
            }
        }
    }

    /// Render next stereo audio frame — must never allocate.
    #[inline]
    pub fn process_frame(&mut self) -> (f32, f32) {
        let mut summed_audio = 0.0_f32;
        let mut active_count = 0_usize;

        match self.params.voice_mode {
            VoiceMode::Mono => {
                if self.voices[0].is_active() {
                    summed_audio = self.voices[0].process_sample(&self.params, self.sample_rate);
                }
                self.smoothed_freq = self.voices[0].smoothed_freq;
            }
            VoiceMode::Poly8 => {
                let mut latest_age = 0;
                for voice in &mut self.voices {
                    if voice.is_active() {
                        summed_audio += voice.process_sample(&self.params, self.sample_rate);
                        active_count += 1;
                        if voice.age >= latest_age {
                            latest_age = voice.age;
                            self.smoothed_freq = voice.smoothed_freq;
                        }
                    }
                }
                if active_count > 1 {
                    let headroom = 1.0 / (1.0 + (active_count - 1) as f32 * 0.35);
                    summed_audio *= headroom;
                }
            }
        }

        // Step master amp envelope for telemetry/test inspection
        let _ = self.amp_envelope.process(
            self.params.amp_attack,
            self.params.amp_decay,
            self.params.amp_sustain,
            self.params.amp_release,
        );

        // 1. Soft Saturation / Overdrive
        let saturated = self.drive_fx.process(summed_audio, self.params.drive);

        // 2. Roland Juno-style Stereo Chorus
        let (chorus_l, chorus_r) = self.chorus_fx.process(
            saturated,
            self.params.chorus_mode,
            self.params.chorus_mix,
        );

        // 3. Stereo Tape Delay
        let (delay_l, delay_r) = self.delay_fx.process(
            chorus_l,
            chorus_r,
            self.params.delay_time,
            self.params.delay_feedback,
            self.params.delay_tone,
            self.params.delay_mix,
        );

        // 4. Master volume + soft limiting limiter
        let out_l = delay_l * self.params.master_volume;
        let out_r = delay_r * self.params.master_volume;

        (
            out_l / (1.0 + out_l.abs() * 0.1),
            out_r / (1.0 + out_r.abs() * 0.1),
        )
    }

    /// Render next mono sample (mixdown of stereo frame)
    #[inline]
    pub fn process_sample(&mut self) -> f32 {
        let (l, r) = self.process_frame();
        0.5 * (l + r)
    }

    pub fn active_note(&self) -> Option<u8> {
        self.active_note
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::lfo::LfoTarget;
    use crate::dsp::params::{SynthParams, Waveform};

    #[test]
    fn test_midi_to_freq_standard_pitches() {
        assert!((midi_to_freq(69, 0) - 440.0).abs() < 0.001);
        assert!((midi_to_freq(57, 0) - 220.0).abs() < 0.001);
        assert!((midi_to_freq(81, 0) - 880.0).abs() < 0.001);
        assert!((midi_to_freq(60, 0) - 261.6256).abs() < 0.01);
    }

    #[test]
    fn test_midi_to_freq_octave_shift() {
        let c4_base = midi_to_freq(60, 0);
        assert!((c4_base - midi_to_freq(48, 1)).abs() < 0.001);
        assert!((c4_base - midi_to_freq(72, -1)).abs() < 0.001);
    }

    #[test]
    fn test_dual_oscillator_mix_isolation() {
        let mut engine = SynthEngine::new(44100.0);
        let mut params = SynthParams {
            osc1_waveform: Waveform::Sine,
            osc2_waveform: Waveform::Saw,
            osc_mix: 0.0,
            ..Default::default()
        };
        engine.handle_event(SynthEvent::UpdateParams(params));
        engine.handle_event(SynthEvent::NoteOn { note: 69, velocity: 1.0 });
        let s1: Vec<f32> = (0..100).map(|_| engine.process_sample()).collect();

        params.osc_mix = 1.0;
        engine.handle_event(SynthEvent::UpdateParams(params));
        let s2: Vec<f32> = (0..100).map(|_| engine.process_sample()).collect();
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_dual_oscillator_interval_fifth() {
        let base = midi_to_freq(69, 0);
        let fifth = base * 2.0_f32.powf(7.0 / 12.0);
        assert!((fifth - 659.255).abs() < 0.01);
    }

    #[test]
    fn test_note_stack_legato_lifecycle() {
        let mut engine = SynthEngine::new(44100.0);
        assert_eq!(engine.active_note(), None);

        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.8 });
        assert_eq!(engine.active_note(), Some(60));
        assert!(engine.amp_envelope.is_active());

        engine.handle_event(SynthEvent::NoteOn { note: 64, velocity: 0.8 });
        assert_eq!(engine.active_note(), Some(64));

        engine.handle_event(SynthEvent::NoteOff { note: 64 });
        assert_eq!(engine.active_note(), Some(60));
        assert!(engine.amp_envelope.is_active());

        engine.handle_event(SynthEvent::NoteOff { note: 60 });
        assert_eq!(engine.active_note(), None);
    }

    #[test]
    fn test_note_stack_release_underneath_note() {
        let mut engine = SynthEngine::new(44100.0);
        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.8 });
        engine.handle_event(SynthEvent::NoteOn { note: 67, velocity: 0.8 });
        assert_eq!(engine.active_note(), Some(67));

        engine.handle_event(SynthEvent::NoteOff { note: 60 });
        assert_eq!(engine.active_note(), Some(67));

        engine.handle_event(SynthEvent::NoteOff { note: 67 });
        assert_eq!(engine.active_note(), None);
    }

    #[test]
    fn test_note_stack_duplicate_and_capacity() {
        let mut engine = SynthEngine::new(44100.0);
        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.8 });
        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.8 });
        assert_eq!(engine.note_stack_len, 1);

        for n in 1..25 {
            engine.handle_event(SynthEvent::NoteOn { note: n, velocity: 0.8 });
        }
        assert!(engine.note_stack_len <= 16);
    }

    #[test]
    fn test_synth_engine_audio_generation() {
        let mut engine = SynthEngine::new(44100.0);
        assert_eq!(engine.process_sample(), 0.0);

        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.9 });
        let mut has_audio = false;
        for _ in 0..1000 {
            let s = engine.process_sample();
            assert!(s.is_finite());
            if s.abs() > 0.001 { has_audio = true; }
        }
        assert!(has_audio);
    }

    #[test]
    fn test_synth_engine_velocity_scaling() {
        let mut quiet = SynthEngine::new(44100.0);
        let mut loud = SynthEngine::new(44100.0);
        quiet.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.2 });
        loud.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });

        let peak_q = (0..500).map(|_| quiet.process_sample().abs()).fold(0.0_f32, f32::max);
        let peak_l = (0..500).map(|_| loud.process_sample().abs()).fold(0.0_f32, f32::max);
        assert!(peak_l > peak_q * 2.0);
    }

    #[test]
    fn test_synth_engine_parameter_updates() {
        let mut engine = SynthEngine::new(44100.0);
        let params = SynthParams {
            osc1_waveform: Waveform::Sine,
            filter_cutoff: 100.0,
            ..Default::default()
        };
        engine.handle_event(SynthEvent::UpdateParams(params));
        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.8 });
        for _ in 0..500 {
            assert!(engine.process_sample().is_finite());
        }
    }

    /// Regression: release tail must stay on the released note's pitch, not
    /// jump to the middle-C fallback or any other unrelated note.
    #[test]
    fn test_release_tail_stays_on_correct_pitch() {
        let mut engine = SynthEngine::new(44100.0);
        engine.handle_event(SynthEvent::NoteOn { note: 36, velocity: 0.8 });
        for _ in 0..1000 { engine.process_sample(); }
        assert_eq!(engine.release_note, 36);

        engine.handle_event(SynthEvent::NoteOff { note: 36 });
        assert_eq!(engine.active_note, None);
        assert_eq!(engine.release_note, 36);

        let mut any_audio = false;
        for _ in 0..500 {
            let s = engine.process_sample();
            assert!(s.is_finite());
            if s.abs() > 1e-6 { any_audio = true; }
        }
        assert!(any_audio);
    }

    // ── Phase 3 feature tests ────────────────────────────────────────────────

    #[test]
    fn test_filter_envelope_modulation_opens_cutoff() {
        // With depth=2.0 (2 octaves), the filter should open noticeably wider
        // during the attack phase, producing a brighter sound than depth=0.
        let base_params = SynthParams {
            osc1_waveform: Waveform::Saw,
            filter_cutoff: 500.0,
            filter_resonance: 1.0,
            filter_attack: 0.001,
            filter_decay: 0.1,
            filter_sustain: 0.0,
            filter_release: 0.1,
            amp_attack: 0.001,
            amp_sustain: 1.0,
            amp_decay: 0.0,
            amp_release: 0.1,
            ..Default::default()
        };

        let mut engine_no_mod = SynthEngine::new(44100.0);
        engine_no_mod.handle_event(SynthEvent::UpdateParams(base_params));
        engine_no_mod.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });
        let peak_no_mod = (0..200).map(|_| engine_no_mod.process_sample().abs()).fold(0.0_f32, f32::max);

        let mut engine_with_mod = SynthEngine::new(44100.0);
        engine_with_mod.handle_event(SynthEvent::UpdateParams(SynthParams {
            filter_env_depth: 3.0,
            ..base_params
        }));
        engine_with_mod.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });
        let peak_with_mod = (0..200).map(|_| engine_with_mod.process_sample().abs()).fold(0.0_f32, f32::max);

        // Filter opening wider should pass more signal — peak must differ
        assert!(
            (peak_with_mod - peak_no_mod).abs() > 0.0001,
            "Filter env modulation should alter the output peak (no_mod={peak_no_mod:.4}, with_mod={peak_with_mod:.4})"
        );
    }

    #[test]
    fn test_lfo_pitch_modulation_changes_output() {
        let base = SynthParams {
            osc1_waveform: Waveform::Sine,
            lfo_rate: 5.0,
            lfo_target: LfoTarget::Pitch,
            ..Default::default()
        };

        let mut engine_no_lfo = SynthEngine::new(44100.0);
        engine_no_lfo.handle_event(SynthEvent::UpdateParams(SynthParams { lfo_depth: 0.0, ..base }));
        engine_no_lfo.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });
        let samples_no: Vec<f32> = (0..256).map(|_| engine_no_lfo.process_sample()).collect();

        let mut engine_lfo = SynthEngine::new(44100.0);
        engine_lfo.handle_event(SynthEvent::UpdateParams(SynthParams { lfo_depth: 1.0, ..base }));
        engine_lfo.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });
        let samples_lfo: Vec<f32> = (0..256).map(|_| engine_lfo.process_sample()).collect();

        assert_ne!(samples_no, samples_lfo, "LFO depth > 0 should alter the pitch output");
    }

    #[test]
    fn test_lfo_filter_modulation_changes_output() {
        let base = SynthParams {
            osc1_waveform: Waveform::Saw,
            lfo_rate: 3.0,
            lfo_target: LfoTarget::Filter,
            ..Default::default()
        };

        let mut no_lfo = SynthEngine::new(44100.0);
        no_lfo.handle_event(SynthEvent::UpdateParams(SynthParams { lfo_depth: 0.0, ..base }));
        no_lfo.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });
        let out_no: Vec<f32> = (0..256).map(|_| no_lfo.process_sample()).collect();

        let mut with_lfo = SynthEngine::new(44100.0);
        with_lfo.handle_event(SynthEvent::UpdateParams(SynthParams { lfo_depth: 1.0, ..base }));
        with_lfo.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });
        let out_lfo: Vec<f32> = (0..256).map(|_| with_lfo.process_sample()).collect();

        assert_ne!(out_no, out_lfo, "LFO filter modulation should alter the output");
    }

    #[test]
    fn test_portamento_glide_smooths_pitch_change() {
        let mut engine = SynthEngine::new(44100.0);
        engine.handle_event(SynthEvent::UpdateParams(SynthParams {
            glide_time: 0.05, // 50 ms glide
            osc1_waveform: Waveform::Sine,
            ..Default::default()
        }));

        // First note
        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });
        // Let it settle
        for _ in 0..2000 { engine.process_sample(); }
        let freq_at_c4 = engine.smoothed_freq;

        // Second note — with portamento the frequency should not snap instantly
        engine.handle_event(SynthEvent::NoteOn { note: 72, velocity: 1.0 }); // C5
        // Process a few samples so the smoother begins gliding toward target
        for _ in 0..10 { engine.process_sample(); }
        let freq_shortly_after = engine.smoothed_freq;

        // The glide smoother should not have reached the target yet
        let target = midi_to_freq(72, 0);
        assert!(
            (freq_shortly_after - target).abs() > 1.0,
            "Portamento should still be sliding toward target (got {freq_shortly_after:.1} Hz, target {target:.1} Hz)"
        );
        assert!(
            freq_shortly_after > freq_at_c4,
            "Frequency should be rising toward C5"
        );
    }

    #[test]
    fn test_portamento_off_is_instant() {
        let mut engine = SynthEngine::new(44100.0);
        engine.handle_event(SynthEvent::UpdateParams(SynthParams {
            glide_time: 0.0, // off
            osc1_waveform: Waveform::Sine,
            ..Default::default()
        }));

        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });
        for _ in 0..2000 { engine.process_sample(); }

        engine.handle_event(SynthEvent::NoteOn { note: 72, velocity: 1.0 });
        // One process call — with glide_time=0, alpha=1.0 so it should snap immediately
        engine.process_sample();
        let target = midi_to_freq(72, 0);
        assert!(
            (engine.smoothed_freq - target).abs() < 0.1,
            "With glide=0 pitch must snap instantly (got {:.1} Hz, expected {:.1} Hz)",
            engine.smoothed_freq, target
        );
    }

    // ── Phase 4 tests ────────────────────────────────────────────────────────

    #[test]
    fn test_polyphony_8_voices_independent_notes() {
        let mut engine = SynthEngine::new(44100.0);
        engine.handle_event(SynthEvent::UpdateParams(SynthParams {
            voice_mode: VoiceMode::Poly8,
            ..Default::default()
        }));

        // Trigger a 4-note chord: C4 (60), E4 (64), G4 (67), B4 (71)
        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.8 });
        engine.handle_event(SynthEvent::NoteOn { note: 64, velocity: 0.8 });
        engine.handle_event(SynthEvent::NoteOn { note: 67, velocity: 0.8 });
        engine.handle_event(SynthEvent::NoteOn { note: 71, velocity: 0.8 });

        assert_eq!(engine.active_voice_count(), 4, "Four distinct voices should be actively rendering");

        let mut peak = 0.0_f32;
        for _ in 0..500 {
            peak = peak.max(engine.process_sample().abs());
        }
        assert!(peak > 0.05, "Polyphonic chord must produce audible signal (got {peak})");

        // Release one note
        engine.handle_event(SynthEvent::NoteOff { note: 64 });
        // Drain note release (at 44.1kHz, 1 second = ~44k samples)
        for _ in 0..50000 { engine.process_sample(); }

        assert_eq!(engine.active_voice_count(), 3, "After 1 note released and decay ended, 3 voices should remain");
    }

    #[test]
    fn test_polyphony_voice_stealing() {
        let mut engine = SynthEngine::new(44100.0);
        engine.handle_event(SynthEvent::UpdateParams(SynthParams {
            voice_mode: VoiceMode::Poly8,
            ..Default::default()
        }));

        // Trigger 9 notes (exceeding MAX_VOICES = 8)
        for note in 60..69 {
            engine.handle_event(SynthEvent::NoteOn { note, velocity: 0.8 });
        }

        assert_eq!(engine.active_voice_count(), 8, "Active voices should cap at MAX_VOICES (8)");
        let (l, r) = engine.process_frame();
        assert!(l.is_finite() && r.is_finite());
    }

    #[test]
    fn test_stereo_process_frame_chorus() {
        use crate::dsp::effects::ChorusMode;
        let mut engine = SynthEngine::new(44100.0);
        engine.handle_event(SynthEvent::UpdateParams(SynthParams {
            chorus_mode: ChorusMode::Mode2,
            chorus_mix: 1.0,
            ..Default::default()
        }));
        engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.9 });

        let mut had_stereo_spread = false;
        for _ in 0..1000 {
            let (l, r) = engine.process_frame();
            if (l - r).abs() > 0.005 {
                had_stereo_spread = true;
                break;
            }
        }
        assert!(had_stereo_spread, "Stereo chorus should produce stereo field difference between left and right channels");
    }

    #[test]
    fn test_drive_saturation_in_engine() {
        let mut clean = SynthEngine::new(44100.0);
        clean.handle_event(SynthEvent::UpdateParams(SynthParams { drive: 0.0, ..Default::default() }));
        clean.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });

        let mut driven = SynthEngine::new(44100.0);
        driven.handle_event(SynthEvent::UpdateParams(SynthParams { drive: 0.9, ..Default::default() }));
        driven.handle_event(SynthEvent::NoteOn { note: 60, velocity: 1.0 });

        let s_clean: Vec<f32> = (0..256).map(|_| clean.process_sample()).collect();
        let s_driven: Vec<f32> = (0..256).map(|_| driven.process_sample()).collect();

        assert_ne!(s_clean, s_driven, "Drive should alter waveform harmonics");
    }
}
