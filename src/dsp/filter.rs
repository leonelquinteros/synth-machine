use std::f32::consts::PI;
use super::params::FilterMode;

/// Cytomic / Andrew Simper State Variable Filter (SVF) with non-linear analog saturation.
pub struct StateVariableFilter {
    sample_rate: f32,
    ic1eq: f32,
    ic2eq: f32,
}

impl StateVariableFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// Process a single audio sample through the filter
    pub fn process(
        &mut self,
        input: f32,
        cutoff_hz: f32,
        resonance: f32,
        mode: FilterMode,
    ) -> f32 {
        // Clamp cutoff safely below Nyquist
        let nyquist = self.sample_rate * 0.49;
        let cutoff = cutoff_hz.clamp(20.0, nyquist);
        let q = resonance.clamp(0.5, 20.0);

        // Pre-warp cutoff frequency
        let g = (PI * cutoff / self.sample_rate).tan();
        let k = 1.0 / q;

        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let mut v1 = a1 * self.ic1eq + a2 * v3;

        // Subtle soft-saturation on internal resonance path to mimic analog saturation
        // and prevent infinite explosion at very high Q
        v1 = v1 / (1.0 + (v1 * 0.15).abs());

        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        match mode {
            FilterMode::LowPass => v2,
            FilterMode::BandPass => v1,
            FilterMode::HighPass => input - k * v1 - v2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compute_sine_rms(filter: &mut StateVariableFilter, freq: f32, cutoff: f32, q: f32, mode: FilterMode) -> f32 {
        let sample_rate = 44100.0;
        let mut sum_sq = 0.0;
        let n = 2000;
        for i in 0..n {
            let input = ((i as f32) * freq * 2.0 * PI / sample_rate).sin();
            let out = filter.process(input, cutoff, q, mode);
            // Ignore initial transient
            if i > 500 {
                sum_sq += out * out;
            }
        }
        (sum_sq / (n - 500) as f32).sqrt()
    }

    #[test]
    fn test_filter_stability_with_high_q() {
        let mut filter = StateVariableFilter::new(44100.0);
        for _ in 0..10000 {
            let out = filter.process(1.0, 1000.0, 20.0, FilterMode::LowPass);
            assert!(out.is_finite(), "Filter output diverged with high Q");
            assert!(out.abs() < 50.0, "Filter output blew up: {}", out);
        }
    }

    #[test]
    fn test_filter_lowpass_attenuation() {
        let mut filter = StateVariableFilter::new(44100.0);
        let cutoff = 200.0;
        let q = 0.707;

        let low_freq_rms = compute_sine_rms(&mut filter, 50.0, cutoff, q, FilterMode::LowPass);
        filter.reset();
        let high_freq_rms = compute_sine_rms(&mut filter, 5000.0, cutoff, q, FilterMode::LowPass);

        assert!(
            high_freq_rms < low_freq_rms * 0.1,
            "Low-pass should attenuate 5 kHz significantly more than 50 Hz: {} vs {}",
            high_freq_rms,
            low_freq_rms
        );
    }

    #[test]
    fn test_filter_highpass_attenuation() {
        let mut filter = StateVariableFilter::new(44100.0);
        let cutoff = 2000.0;
        let q = 0.707;

        let high_freq_rms = compute_sine_rms(&mut filter, 5000.0, cutoff, q, FilterMode::HighPass);
        filter.reset();
        let low_freq_rms = compute_sine_rms(&mut filter, 50.0, cutoff, q, FilterMode::HighPass);

        assert!(
            low_freq_rms < high_freq_rms * 0.1,
            "High-pass should attenuate 50 Hz significantly more than 5 kHz: {} vs {}",
            low_freq_rms,
            high_freq_rms
        );
    }

    #[test]
    fn test_filter_bandpass_peak() {
        let mut filter = StateVariableFilter::new(44100.0);
        let cutoff = 1000.0;
        let q = 2.0;

        let center_rms = compute_sine_rms(&mut filter, 1000.0, cutoff, q, FilterMode::BandPass);
        filter.reset();
        let low_rms = compute_sine_rms(&mut filter, 100.0, cutoff, q, FilterMode::BandPass);
        filter.reset();
        let high_rms = compute_sine_rms(&mut filter, 8000.0, cutoff, q, FilterMode::BandPass);

        assert!(center_rms > low_rms * 2.0, "Band-pass should boost center over low");
        assert!(center_rms > high_rms * 2.0, "Band-pass should boost center over high");
    }

    #[test]
    fn test_filter_reset_clears_state() {
        let mut filter = StateVariableFilter::new(44100.0);
        // Excite filter
        for _ in 0..500 {
            filter.process(1.0, 500.0, 5.0, FilterMode::LowPass);
        }
        assert_ne!(filter.ic1eq, 0.0);
        assert_ne!(filter.ic2eq, 0.0);

        filter.reset();
        assert_eq!(filter.ic1eq, 0.0);
        assert_eq!(filter.ic2eq, 0.0);
        // Clean zero input should produce zero output immediately
        let out = filter.process(0.0, 500.0, 5.0, FilterMode::LowPass);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_filter_sample_rate_adaptation() {
        let mut filter = StateVariableFilter::new(44100.0);
        filter.set_sample_rate(96000.0);
        let out = filter.process(0.5, 1000.0, 1.0, FilterMode::LowPass);
        assert!(out.is_finite());
    }
}
