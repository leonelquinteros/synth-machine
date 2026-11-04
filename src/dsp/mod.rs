pub mod effects;
pub mod envelope;
pub mod filter;
pub mod lfo;
pub mod oscillator;
pub mod params;
pub mod presets;
pub mod voice;

pub use effects::{ChorusMode, DriveEffect, StereoChorus, TapeDelay};
pub use envelope::AdsrEnvelope;
pub use filter::StateVariableFilter;
pub use lfo::{Lfo, LfoTarget, LfoWaveform};
pub use oscillator::Oscillator;
pub use params::{FilterMode, SynthEvent, SynthParams, VoiceMode, Waveform};
pub use presets::{Preset, PresetManager};
pub use voice::{SynthEngine, SynthVoice, MAX_VOICES};
