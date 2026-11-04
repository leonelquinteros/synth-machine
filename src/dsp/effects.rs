use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Roland Juno-style Chorus mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChorusMode {
    Off,
    Mode1,      // Mode I: Gentle ensemble chorus (~0.5 Hz)
    Mode2,      // Mode II: Deep, lush chorusing (~0.85 Hz)
    Mode1And2,  // Mode I+II: Fast, rich rotary vibrato (~8.0 Hz)
}

impl ChorusMode {
    pub const ALL: [ChorusMode; 4] = [
        ChorusMode::Off,
        ChorusMode::Mode1,
        ChorusMode::Mode2,
        ChorusMode::Mode1And2,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ChorusMode::Off => "Off",
            ChorusMode::Mode1 => "I (Ensemble)",
            ChorusMode::Mode2 => "II (Lush)",
            ChorusMode::Mode1And2 => "I+II (Vibe)",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  1. Soft Saturation / Overdrive
// ─────────────────────────────────────────────────────────────────────────────

pub struct DriveEffect;

impl DriveEffect {
    pub fn new() -> Self {
        Self
    }

    /// Process a sample through warm analog-modeled soft clipping.
    /// `drive` range: 0.0 (clean) to 1.0 (aggressive warm overdrive).
    #[inline]
    pub fn process(&self, input: f32, drive: f32) -> f32 {
        let drive = drive.clamp(0.0, 1.0);
        if drive <= 0.001 {
            return input;
        }

        // Drive gain: 1.0 (at drive 0) up to 5.0 (at drive 1)
        let gain = 1.0 + drive * 4.0;
        let x = input * gain;

        // Symmetric polynomial soft-clipper with smooth saturation knee
        let saturated = if x > 1.5 {
            1.0
        } else if x < -1.5 {
            -1.0
        } else {
            // Cubic saturation curve: f(x) = x - x^3 / 3
            x - (x * x * x) * (1.0 / 6.75)
        };

        // Automatic makeup attenuation to prevent deafening loudness surges
        let makeup = 1.0 / (1.0 + drive * 1.5);
        saturated * makeup
    }
}

impl Default for DriveEffect {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  2. Roland Juno-style Stereo Chorus
// ─────────────────────────────────────────────────────────────────────────────

const CHORUS_BUFFER_SIZE: usize = 2048;

pub struct StereoChorus {
    sample_rate: f32,
    buffer_left: [f32; CHORUS_BUFFER_SIZE],
    buffer_right: [f32; CHORUS_BUFFER_SIZE],
    write_pos: usize,
    lfo_phase: f32,
}

impl StereoChorus {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(1.0),
            buffer_left: [0.0; CHORUS_BUFFER_SIZE],
            buffer_right: [0.0; CHORUS_BUFFER_SIZE],
            write_pos: 0,
            lfo_phase: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn reset(&mut self) {
        self.buffer_left.fill(0.0);
        self.buffer_right.fill(0.0);
        self.write_pos = 0;
        self.lfo_phase = 0.0;
    }

    /// Process a mono input into a lush stereo field.
    #[inline]
    pub fn process(&mut self, input: f32, mode: ChorusMode, mix: f32) -> (f32, f32) {
        let mix = mix.clamp(0.0, 1.0);
        if mode == ChorusMode::Off || mix <= 0.001 {
            return (input, input);
        }

        // Store current input in delay lines
        self.buffer_left[self.write_pos] = input;
        self.buffer_right[self.write_pos] = input;

        // Mode settings derived from Roland Juno-60 BBD specs
        let (rate_hz, depth_ms, center_ms) = match mode {
            ChorusMode::Off => (0.0, 0.0, 0.0),
            ChorusMode::Mode1 => (0.51, 1.2, 5.2),
            ChorusMode::Mode2 => (0.86, 2.4, 5.8),
            ChorusMode::Mode1And2 => (8.0, 0.9, 4.0),
        };

        // Advance LFO
        self.lfo_phase += rate_hz / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        // Left channel: sin(phase), Right channel: cos(phase) / 180° inverted for wide stereo spread
        let lfo_left = (self.lfo_phase * TAU).sin();
        let lfo_right = (self.lfo_phase * TAU + std::f32::consts::PI).sin();

        let center_samples = center_ms * 0.001 * self.sample_rate;
        let depth_samples = depth_ms * 0.001 * self.sample_rate;

        let delay_left = center_samples + lfo_left * depth_samples;
        let delay_right = center_samples + lfo_right * depth_samples;

        let wet_left = self.read_interpolated(&self.buffer_left, delay_left);
        let wet_right = self.read_interpolated(&self.buffer_right, delay_right);

        // Advance write pointer
        self.write_pos = (self.write_pos + 1) % CHORUS_BUFFER_SIZE;

        // Equal-power dry/wet crossfade
        let dry_gain = 1.0 - mix * 0.35;
        let wet_gain = mix * 0.75;

        (
            input * dry_gain + wet_left * wet_gain,
            input * dry_gain + wet_right * wet_gain,
        )
    }

    #[inline(always)]
    fn read_interpolated(&self, buffer: &[f32; CHORUS_BUFFER_SIZE], delay_samples: f32) -> f32 {
        let delay = delay_samples.clamp(1.0, (CHORUS_BUFFER_SIZE - 2) as f32);
        let read_pos = (self.write_pos as f32) - delay + (CHORUS_BUFFER_SIZE as f32);
        let idx0 = (read_pos.floor() as usize) % CHORUS_BUFFER_SIZE;
        let idx1 = (idx0 + 1) % CHORUS_BUFFER_SIZE;
        let frac = read_pos.fract();

        // Linear interpolation
        buffer[idx0] * (1.0 - frac) + buffer[idx1] * frac
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  3. Stereo Tape Delay
// ─────────────────────────────────────────────────────────────────────────────

const MAX_DELAY_SECONDS: usize = 2;

pub struct TapeDelay {
    sample_rate: f32,
    buffer_left: Vec<f32>,
    buffer_right: Vec<f32>,
    capacity: usize,
    write_pos: usize,
    // 1-pole lowpass filter states for tape feedback damping
    filter_state_l: f32,
    filter_state_r: f32,
}

impl TapeDelay {
    pub fn new(sample_rate: f32) -> Self {
        let sample_rate = sample_rate.max(1.0);
        let capacity = (sample_rate as usize * MAX_DELAY_SECONDS).max(4096);
        Self {
            sample_rate,
            buffer_left: vec![0.0; capacity],
            buffer_right: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            filter_state_l: 0.0,
            filter_state_r: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let sample_rate = sample_rate.max(1.0);
        let new_capacity = (sample_rate as usize * MAX_DELAY_SECONDS).max(4096);
        if new_capacity != self.capacity {
            self.buffer_left.resize(new_capacity, 0.0);
            self.buffer_right.resize(new_capacity, 0.0);
            self.capacity = new_capacity;
        }
        self.sample_rate = sample_rate;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.buffer_left.fill(0.0);
        self.buffer_right.fill(0.0);
        self.write_pos = 0;
        self.filter_state_l = 0.0;
        self.filter_state_r = 0.0;
    }

    /// Process stereo input through tape-modeled delay with high-frequency damping
    /// and stereo ping-pong spread.
    #[inline]
    pub fn process(
        &mut self,
        in_left: f32,
        in_right: f32,
        time_seconds: f32,
        feedback: f32,
        tone: f32, // 0.1 (dark tape) .. 1.0 (bright)
        mix: f32,  // 0.0 .. 1.0
    ) -> (f32, f32) {
        let mix = mix.clamp(0.0, 1.0);
        if mix <= 0.001 {
            return (in_left, in_right);
        }

        let time = time_seconds.clamp(0.01, (MAX_DELAY_SECONDS as f32) - 0.05);
        let feedback = feedback.clamp(0.0, 0.85);

        // Ping-pong delay time offset for rich spatial stereo width:
        // Left channel delays at base time, Right channel delays at 3/4 time (dotted 8th feel)
        let delay_samples_l = (time * self.sample_rate).clamp(1.0, (self.capacity - 1) as f32);
        let delay_samples_r = (time * 0.75 * self.sample_rate).clamp(1.0, (self.capacity - 1) as f32);

        // Read interpolated taps
        let tap_l = self.read_interpolated(&self.buffer_left, delay_samples_l);
        let tap_r = self.read_interpolated(&self.buffer_right, delay_samples_r);

        // Analog tape tone damping filter: one-pole lowpass filter in feedback loop
        let alpha = (tone.clamp(0.1, 1.0) * 0.5).clamp(0.05, 0.95);
        self.filter_state_l += alpha * (tap_l - self.filter_state_l);
        self.filter_state_r += alpha * (tap_r - self.filter_state_r);

        // Soft-clip feedback to prevent runaway overload
        let fb_l = (self.filter_state_l * feedback).tanh();
        let fb_r = (self.filter_state_r * feedback).tanh();

        // Cross-feedback ping-pong injection
        self.buffer_left[self.write_pos] = in_left + fb_r * 0.7 + fb_l * 0.3;
        self.buffer_right[self.write_pos] = in_right + fb_l * 0.7 + fb_r * 0.3;

        self.write_pos = (self.write_pos + 1) % self.capacity;

        // Dry/wet balance
        let out_l = in_left * (1.0 - mix * 0.5) + tap_l * mix;
        let out_r = in_right * (1.0 - mix * 0.5) + tap_r * mix;

        (out_l, out_r)
    }

    #[inline(always)]
    fn read_interpolated(&self, buffer: &[f32], delay_samples: f32) -> f32 {
        let read_pos = (self.write_pos as f32) - delay_samples + (self.capacity as f32);
        let idx0 = (read_pos.floor() as usize) % self.capacity;
        let idx1 = (idx0 + 1) % self.capacity;
        let frac = read_pos.fract();

        buffer[idx0] * (1.0 - frac) + buffer[idx1] * frac
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_clean_passthrough() {
        let drive = DriveEffect::new();
        let sample = 0.5;
        let processed = drive.process(sample, 0.0);
        assert!((processed - sample).abs() < 1e-4);
    }

    #[test]
    fn test_drive_saturation_bounds() {
        let drive = DriveEffect::new();
        for &input in &[-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0] {
            let out = drive.process(input, 1.0);
            assert!(out.is_finite(), "Output must be finite");
            assert!(out.abs() <= 1.2, "Drive saturated output exceeded safety bounds: {out}");
        }
    }

    #[test]
    fn test_chorus_off_is_passthrough() {
        let mut chorus = StereoChorus::new(44100.0);
        let (l, r) = chorus.process(0.75, ChorusMode::Off, 1.0);
        assert_eq!(l, 0.75);
        assert_eq!(r, 0.75);

        let (l2, r2) = chorus.process(0.75, ChorusMode::Mode1, 0.0);
        assert_eq!(l2, 0.75);
        assert_eq!(r2, 0.75);
    }

    #[test]
    fn test_chorus_stereo_widening() {
        let mut chorus = StereoChorus::new(44100.0);
        // Feed an impulse through chorus
        let mut different_outputs = false;
        for i in 0..1000 {
            let input = if i == 0 { 1.0 } else { 0.0 };
            let (l, r) = chorus.process(input, ChorusMode::Mode2, 1.0);
            if (l - r).abs() > 0.01 {
                different_outputs = true;
            }
        }
        assert!(different_outputs, "Stereo chorus should produce stereo difference between channels");
    }

    #[test]
    fn test_tape_delay_echoes() {
        let mut delay = TapeDelay::new(44100.0);
        let delay_time = 0.01; // 10ms = 441 samples
        let mut echo_received = false;

        // Send an impulse
        delay.process(1.0, 1.0, delay_time, 0.5, 0.8, 1.0);

        // Step 600 samples
        for _ in 0..600 {
            let (l, r) = delay.process(0.0, 0.0, delay_time, 0.5, 0.8, 1.0);
            if l.abs() > 0.1 || r.abs() > 0.1 {
                echo_received = true;
                break;
            }
        }
        assert!(echo_received, "Tape delay should echo back the impulse");
    }

    #[test]
    fn test_tape_delay_stability() {
        let mut delay = TapeDelay::new(44100.0);
        for _ in 0..2000 {
            let (l, r) = delay.process(0.8, 0.8, 0.05, 0.85, 0.5, 1.0);
            assert!(l.is_finite() && r.is_finite(), "Delay output must remain finite");
            assert!(l.abs() < 2.5 && r.abs() < 2.5, "Delay must not explode with high feedback");
        }
    }
}
