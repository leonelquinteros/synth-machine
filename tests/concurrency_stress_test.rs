use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use synth::dsp::params::{SynthEvent, SynthParams, Waveform};
use synth::dsp::voice::SynthEngine;

#[test]
fn test_live_performance_multithread_stress() {
    let (event_tx, event_rx): (Sender<SynthEvent>, Receiver<SynthEvent>) =
        crossbeam_channel::bounded(1024);

    let running = Arc::new(AtomicBool::new(true));

    // Audio thread simulator
    let running_audio = Arc::clone(&running);
    let audio_handle = thread::spawn(move || {
        let mut engine = SynthEngine::new(48000.0);
        let mut sample_count = 0usize;
        let mut sum = 0.0_f32;

        while running_audio.load(Ordering::Relaxed) || !event_rx.is_empty() {
            // Drain queue without blocking (exactly as in cpal callback)
            while let Ok(event) = event_rx.try_recv() {
                engine.handle_event(event);
            }

            // Process audio block (256 frames)
            for _ in 0..256 {
                let sample = engine.process_sample();
                assert!(sample.is_finite(), "Audio thread generated non-finite sample");
                sum += sample;
                sample_count += 1;
            }

            thread::yield_now();
        }

        (sample_count, sum)
    });

    // UI/MIDI thread simulator generating 10,000 rapid live events
    let event_count = 10_000;
    for i in 0..event_count {
        let note = (40 + (i % 48)) as u8;
        if i % 3 == 0 {
            let _ = event_tx.send(SynthEvent::NoteOn {
                note,
                velocity: 0.8,
            });
        } else if i % 3 == 1 {
            let _ = event_tx.send(SynthEvent::NoteOff { note });
        } else {
            let params = SynthParams {
                osc1_waveform: match (i / 100) % 4 {
                    0 => Waveform::Saw,
                    1 => Waveform::Square,
                    2 => Waveform::Triangle,
                    _ => Waveform::Sine,
                },
                osc2_waveform: match (i / 150) % 4 {
                    0 => Waveform::Square,
                    1 => Waveform::Saw,
                    2 => Waveform::Sine,
                    _ => Waveform::Triangle,
                },
                osc_mix: ((i % 100) as f32) / 100.0,
                filter_cutoff: 100.0 + ((i * 17) % 15000) as f32,
                ..Default::default()
            };
            let _ = event_tx.send(SynthEvent::UpdateParams(params));
        }

        if i % 1000 == 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }

    // Allow audio thread to finish draining
    thread::sleep(Duration::from_millis(50));
    running.store(false, Ordering::Relaxed);

    let (samples_rendered, _) = audio_handle.join().expect("Audio thread panicked");
    assert!(
        samples_rendered > 10_000,
        "Audio thread should have rendered thousands of samples: got {samples_rendered}"
    );
}
