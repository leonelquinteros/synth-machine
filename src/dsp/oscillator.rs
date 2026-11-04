use std::f32::consts::PI;
use super::params::Waveform;

/// Helper function for 2nd order Polynomial Bandlimited Step (PolyBLEP)
/// Corrects high-frequency aliasing around sharp discontinuities.
#[inline(always)]
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

pub struct Oscillator {
    phase: f32,
    sample_rate: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            sample_rate,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    /// Advance the oscillator by one sample and return the output signal in [-1.0, 1.0]
    pub fn process(&mut self, frequency: f32, waveform: Waveform, pulse_width: f32) -> f32 {
        let dt = (frequency / self.sample_rate).clamp(0.0, 0.499);
        let sample = match waveform {
            Waveform::Saw => {
                let mut out = 2.0 * self.phase - 1.0;
                out -= poly_blep(self.phase, dt);
                out
            }
            Waveform::Square => {
                let pw = pulse_width.clamp(0.05, 0.95);
                let mut out = if self.phase < pw { 1.0 } else { -1.0 };
                out += poly_blep(self.phase, dt);
                out -= poly_blep((self.phase + 1.0 - pw).fract(), dt);
                out
            }
            Waveform::Triangle => {
                // Leaky integrated PolyBLEP square or continuous triangle
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            Waveform::Sine => (self.phase * 2.0 * PI).sin(),
        };

        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_bounded() {
        let mut osc = Oscillator::new(44100.0);
        for &wf in &Waveform::ALL {
            for _ in 0..1000 {
                let sample = osc.process(440.0, wf, 0.5);
                assert!(
                    (-1.5..=1.5).contains(&sample),
                    "Sample out of bounds for {:?}: {}",
                    wf,
                    sample
                );
            }
        }
    }

    #[test]
    fn test_oscillator_square_pwm_duty_cycle() {
        let mut osc = Oscillator::new(1000.0);
        let freq = 10.0; // 100 samples per period

        // Test 20% pulse width vs 80% pulse width
        let mut positive_samples_20 = 0;
        let mut positive_samples_80 = 0;

        for _ in 0..100 {
            if osc.process(freq, Waveform::Square, 0.20) > 0.0 {
                positive_samples_20 += 1;
            }
        }

        osc.reset_phase();
        for _ in 0..100 {
            if osc.process(freq, Waveform::Square, 0.80) > 0.0 {
                positive_samples_80 += 1;
            }
        }

        assert!(
            positive_samples_20 < positive_samples_80,
            "PWM should produce fewer positive samples with 20% width than 80%: {} vs {}",
            positive_samples_20,
            positive_samples_80
        );
    }

    #[test]
    fn test_oscillator_sine_range() {
        let mut osc = Oscillator::new(48000.0);
        let mut min = 1.0_f32;
        let mut max = -1.0_f32;
        for _ in 0..4800 {
            let s = osc.process(100.0, Waveform::Sine, 0.5);
            min = min.min(s);
            max = max.max(s);
        }
        assert!((min - (-1.0)).abs() < 0.02, "Sine min should reach ~ -1.0, got {min}");
        assert!((max - 1.0).abs() < 0.02, "Sine max should reach ~ 1.0, got {max}");
    }

    #[test]
    fn test_oscillator_reset_phase() {
        let mut osc = Oscillator::new(44100.0);
        let s0 = osc.process(440.0, Waveform::Sine, 0.5);
        for _ in 0..50 {
            osc.process(440.0, Waveform::Sine, 0.5);
        }
        osc.reset_phase();
        let s_reset = osc.process(440.0, Waveform::Sine, 0.5);
        assert_eq!(s0, s_reset, "Phase reset must restore initial state");
    }

    #[test]
    fn test_oscillator_extreme_frequencies() {
        let mut osc = Oscillator::new(44100.0);
        let freqs = [0.0, 1.0, 20.0, 20000.0, 22050.0, -100.0];
        for &f in &freqs {
            for &wf in &Waveform::ALL {
                let sample = osc.process(f, wf, 0.5);
                assert!(sample.is_finite(), "Sample should be finite at freq {f}: {sample}");
            }
        }
    }

    #[test]
    fn test_sample_rate_adaptation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_sample_rate(96000.0);
        let sample = osc.process(440.0, Waveform::Saw, 0.5);
        assert!(sample.is_finite());
    }

    #[test]
    fn test_poly_blep_continuity() {
        let dt = 0.01;
        // Outside transition region: should return exactly 0.0
        assert_eq!(poly_blep(0.5, dt), 0.0);
        // At edges
        assert!(poly_blep(0.005, dt).is_finite());
        assert!(poly_blep(0.995, dt).is_finite());
    }
}
