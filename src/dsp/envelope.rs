#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct AdsrEnvelope {
    sample_rate: f32,
    state: EnvelopeState,
    level: f32,
}

impl AdsrEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            state: EnvelopeState::Idle,
            level: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn gate_on(&mut self) {
        self.state = EnvelopeState::Attack;
    }

    pub fn gate_off(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }

    pub fn state(&self) -> EnvelopeState {
        self.state
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    /// Advance one sample and return the envelope level in [0.0, 1.0]
    pub fn process(&mut self, attack_s: f32, decay_s: f32, sustain_lvl: f32, release_s: f32) -> f32 {
        let sustain = sustain_lvl.clamp(0.0, 1.0);

        match self.state {
            EnvelopeState::Idle => {
                self.level = 0.0;
            }
            EnvelopeState::Attack => {
                let attack_time = attack_s.max(0.001);
                // Linear or smooth rise to 1.0
                let step = 1.0 / (attack_time * self.sample_rate);
                self.level += step;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            }
            EnvelopeState::Decay => {
                let decay_time = decay_s.max(0.001);
                let decay_coef = (-1.0 / (decay_time * self.sample_rate)).exp();
                self.level = sustain + (self.level - sustain) * decay_coef;
                if (self.level - sustain).abs() < 0.0001 {
                    self.level = sustain;
                    self.state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Sustain => {
                self.level = sustain;
            }
            EnvelopeState::Release => {
                let release_time = release_s.max(0.001);
                let release_coef = (-1.0 / (release_time * self.sample_rate * 0.25)).exp();
                self.level *= release_coef;
                if self.level < 0.0001 {
                    self.level = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
        }

        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_lifecycle() {
        let mut env = AdsrEnvelope::new(1000.0);
        assert_eq!(env.state(), EnvelopeState::Idle);
        assert!(!env.is_active());

        env.gate_on();
        assert_eq!(env.state(), EnvelopeState::Attack);
        assert!(env.is_active());

        // Process through attack (0.01s = 10 samples)
        for _ in 0..15 {
            env.process(0.01, 0.05, 0.5, 0.05);
        }
        assert!(env.state() == EnvelopeState::Decay || env.state() == EnvelopeState::Sustain);

        // Process to reach sustain level
        for _ in 0..500 {
            env.process(0.01, 0.05, 0.5, 0.05);
        }
        assert_eq!(env.state(), EnvelopeState::Sustain);
        assert!((env.level() - 0.5).abs() < 0.01);

        env.gate_off();
        assert_eq!(env.state(), EnvelopeState::Release);

        for _ in 0..150 {
            env.process(0.01, 0.05, 0.5, 0.01);
        }
        assert_eq!(env.state(), EnvelopeState::Idle);
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn test_envelope_retrigger_without_clicks() {
        let mut env = AdsrEnvelope::new(1000.0);
        env.gate_on();
        for _ in 0..10 {
            env.process(0.01, 0.05, 0.5, 0.05);
        }
        env.gate_off();
        // Process partially into release
        for _ in 0..5 {
            env.process(0.01, 0.05, 0.5, 0.05);
        }
        let level_before_retrigger = env.level();
        assert!(level_before_retrigger > 0.0);

        // Retrigger
        env.gate_on();
        assert_eq!(env.state(), EnvelopeState::Attack);
        let next_sample = env.process(0.01, 0.05, 0.5, 0.05);
        // Level should smoothly rise from previous level, not jump to 0.0 (prevents clicks)
        assert!(next_sample >= level_before_retrigger);
    }

    #[test]
    fn test_envelope_sustain_level_stability() {
        let mut env = AdsrEnvelope::new(1000.0);
        env.gate_on();
        // Fast attack and decay
        for _ in 0..100 {
            env.process(0.001, 0.005, 0.42, 0.1);
        }
        assert_eq!(env.state(), EnvelopeState::Sustain);
        // While gate stays on, sustain level shouldn't drift
        for _ in 0..1000 {
            let l = env.process(0.001, 0.005, 0.42, 0.1);
            assert!((l - 0.42).abs() < 0.001);
        }
    }

    #[test]
    fn test_envelope_bounds_always_valid() {
        let mut env = AdsrEnvelope::new(44100.0);
        env.gate_on();
        for _ in 0..5000 {
            let l = env.process(0.02, 0.05, 0.7, 0.05);
            assert!((0.0..=1.0).contains(&l), "Envelope level out of bounds: {}", l);
        }
        env.gate_off();
        for _ in 0..5000 {
            let l = env.process(0.02, 0.05, 0.7, 0.05);
            assert!((0.0..=1.0).contains(&l), "Envelope level out of bounds: {}", l);
        }
    }
}
