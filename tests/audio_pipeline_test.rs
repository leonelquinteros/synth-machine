use synth::dsp::params::{FilterMode, SynthEvent, SynthParams, Waveform};
use synth::dsp::voice::SynthEngine;

#[test]
fn test_end_to_end_note_lifecycle_and_energy() {
    let sample_rate = 44100.0;
    let mut engine = SynthEngine::new(sample_rate);

    // Block 1: Initial silence
    let mut buffer = [0.0_f32; 256];
    for sample in buffer.iter_mut() {
        *sample = engine.process_sample();
    }
    assert!(buffer.iter().all(|&s| s == 0.0), "Initial audio must be silent");

    // Block 2: Note On (A4 = 69)
    engine.handle_event(SynthEvent::NoteOn { note: 69, velocity: 0.9 });
    for sample in buffer.iter_mut() {
        *sample = engine.process_sample();
    }

    let rms = (buffer.iter().map(|&s| s * s).sum::<f32>() / buffer.len() as f32).sqrt();
    assert!(rms > 0.05, "Active note should generate substantial RMS energy: got {rms}");

    // Block 3: Dynamic parameter modulation during playback
    let params = SynthParams {
        filter_mode: FilterMode::LowPass,
        filter_cutoff: 300.0,
        filter_resonance: 8.0,
        ..Default::default()
    };
    engine.handle_event(SynthEvent::UpdateParams(params));

    for sample in buffer.iter_mut() {
        *sample = engine.process_sample();
        assert!(sample.is_finite(), "Modulated sample must be finite");
        assert!(sample.abs() <= 1.2, "Sample exceeded soft-clipped headroom: {}", sample);
    }

    // Block 4: Note Off and release decay to silence
    engine.handle_event(SynthEvent::NoteOff { note: 69 });

    // Render 1.5 seconds of release (at 44.1kHz, 1.5s = ~66k samples)
    let mut final_sample = 1.0;
    for _ in 0..66000 {
        final_sample = engine.process_sample();
    }

    assert_eq!(final_sample, 0.0, "Audio engine must fully return to zero after release completes");
}

#[test]
fn test_filter_sweep_stability() {
    let sample_rate = 48000.0;
    let mut engine = SynthEngine::new(sample_rate);

    engine.handle_event(SynthEvent::NoteOn { note: 60, velocity: 0.8 });

    // Rapidly sweep filter cutoff up and down over 10,000 samples
    for i in 0..10000 {
        let cutoff = 100.0 + (i as f32 * 0.1).sin().abs() * 15000.0;
        let params = SynthParams {
            filter_cutoff: cutoff,
            filter_resonance: 12.0,
            ..Default::default()
        };
        engine.handle_event(SynthEvent::UpdateParams(params));

        let s = engine.process_sample();
        assert!(s.is_finite(), "Filter sweep produced non-finite output at step {i}");
        assert!(s.abs() < 2.0, "Filter output blew up during sweep: {s}");
    }
}

#[test]
fn test_all_waveforms_audio_generation() {
    let sample_rate = 44100.0;
    let mut engine = SynthEngine::new(sample_rate);

    for &waveform in &Waveform::ALL {
        let params = SynthParams {
            osc1_waveform: waveform,
            osc1_pulse_width: 0.35,
            osc_mix: 0.0,
            ..Default::default()
        };
        engine.handle_event(SynthEvent::UpdateParams(params));

        engine.handle_event(SynthEvent::NoteOn { note: 64, velocity: 0.85 });

        let mut peak = 0.0_f32;
        for _ in 0..512 {
            peak = peak.max(engine.process_sample().abs());
        }

        assert!(peak > 0.05, "Waveform {:?} produced insufficient amplitude: {peak}", waveform);

        engine.handle_event(SynthEvent::NoteOff { note: 64 });
        // Drain note
        for _ in 0..20000 {
            engine.process_sample();
        }
    }
}
