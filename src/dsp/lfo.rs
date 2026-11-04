use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// LFO waveform shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfoWaveform {
    Sine,
    Triangle,
    Square,
}

impl LfoWaveform {
    pub const ALL: [LfoWaveform; 3] = [LfoWaveform::Sine, LfoWaveform::Triangle, LfoWaveform::Square];
}

/// LFO modulation target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfoTarget {
    /// Modulates oscillator pitch (vibrato)
    Pitch,
    /// Modulates filter cutoff (auto-wah / wobble)
    Filter,
}

impl LfoTarget {
    pub const ALL: [LfoTarget; 2] = [LfoTarget::Pitch, LfoTarget::Filter];
}

/// Low-Frequency Oscillator — produces a unipolar [−1, +1] modulation signal.
/// Designed to be called once per audio sample; no allocation in hot path.
pub struct Lfo {
    sample_rate: f32,
    phase: f32, // 0.0 .. 1.0
}

impl Lfo {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate, phase: 0.0 }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// Advance one sample and return modulation value in [-1.0, +1.0].
    #[inline(always)]
    pub fn process(&mut self, rate_hz: f32, waveform: LfoWaveform) -> f32 {
        let rate = rate_hz.clamp(0.01, 30.0);
        self.phase += rate / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        match waveform {
            LfoWaveform::Sine => (self.phase * TAU).sin(),
            LfoWaveform::Triangle => {
                // 0→1 first half, 1→-1 second half, linear
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            LfoWaveform::Square => {
                if self.phase < 0.5 { 1.0 } else { -1.0 }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfo_sine_bounded() {
        let mut lfo = Lfo::new(44100.0);
        for _ in 0..44100 {
            let v = lfo.process(1.0, LfoWaveform::Sine);
            assert!((-1.0..=1.0).contains(&v), "LFO sine out of bounds: {}", v);
        }
    }

    #[test]
    fn test_lfo_triangle_bounded() {
        let mut lfo = Lfo::new(44100.0);
        for _ in 0..44100 {
            let v = lfo.process(2.5, LfoWaveform::Triangle);
            assert!((-1.0..=1.0).contains(&v), "LFO triangle out of bounds: {}", v);
        }
    }

    #[test]
    fn test_lfo_square_values() {
        let mut lfo = Lfo::new(100.0);
        // At rate=1Hz, 100 samples = one full cycle
        let mut saw_high = false;
        let mut saw_low = false;
        for _ in 0..100 {
            let v = lfo.process(1.0, LfoWaveform::Square);
            assert!(v == 1.0 || v == -1.0, "Square LFO must be exactly ±1");
            if v == 1.0 { saw_high = true; }
            if v == -1.0 { saw_low = true; }
        }
        assert!(saw_high && saw_low, "Square LFO must produce both +1 and -1");
    }

    #[test]
    fn test_lfo_reset_restarts_phase() {
        let mut lfo = Lfo::new(44100.0);
        for _ in 0..1000 {
            lfo.process(5.0, LfoWaveform::Sine);
        }
        lfo.reset();
        // After reset the sine should be near 0.0 (sin(0) = 0)
        // Advance one sample to get first output after reset
        let first = lfo.process(5.0, LfoWaveform::Sine);
        assert!(first.abs() < 0.01, "After reset, first LFO sample should be near 0");
    }

    #[test]
    fn test_lfo_rate_clamp() {
        let mut lfo = Lfo::new(44100.0);
        // Very low / very high rates must not panic or produce NaN
        let v1 = lfo.process(0.0, LfoWaveform::Sine);
        let v2 = lfo.process(1000.0, LfoWaveform::Sine);
        assert!(v1.is_finite() && v2.is_finite());
    }
}
