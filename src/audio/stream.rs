use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use crossbeam_channel::{Receiver, Sender};
use ringbuf::traits::{Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};

use crate::dsp::params::SynthEvent;
use crate::dsp::voice::SynthEngine;

pub struct AudioSystem {
    _stream: Stream,
    pub event_tx: Sender<SynthEvent>,
    pub viz_consumer: HeapCons<f32>,
    pub sample_rate: f32,
    pub device_name: String,
}

impl AudioSystem {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No default audio output device found")?;

        let device_name = device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|_| "Default Device".to_string());

        let default_config = device.default_output_config()?;
        let sample_format = default_config.sample_format();
        let stream_config: StreamConfig = default_config.into();

        let sample_rate = stream_config.sample_rate as f32;
        let channels = stream_config.channels as usize;

        let (event_tx, event_rx): (Sender<SynthEvent>, Receiver<SynthEvent>) =
            crossbeam_channel::unbounded();

        // 2048 sample ringbuffer for oscilloscope visualization
        let rb = HeapRb::<f32>::new(2048);
        let (viz_producer, viz_consumer) = rb.split();

        let engine = SynthEngine::new(sample_rate);

        let err_fn = |err| eprintln!("Audio stream error: {err}");

        let stream = match sample_format {
            SampleFormat::F32 => Self::build_stream::<f32>(
                &device,
                stream_config,
                channels,
                engine,
                event_rx,
                viz_producer,
                err_fn,
            )?,
            SampleFormat::I16 => Self::build_stream::<i16>(
                &device,
                stream_config,
                channels,
                engine,
                event_rx,
                viz_producer,
                err_fn,
            )?,
            SampleFormat::U16 => Self::build_stream::<u16>(
                &device,
                stream_config,
                channels,
                engine,
                event_rx,
                viz_producer,
                err_fn,
            )?,
            _ => return Err(format!("Unsupported audio sample format: {sample_format:?}").into()),
        };

        stream.play()?;

        Ok(Self {
            _stream: stream,
            event_tx,
            viz_consumer,
            sample_rate,
            device_name,
        })
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: StreamConfig,
        channels: usize,
        mut engine: SynthEngine,
        event_rx: Receiver<SynthEvent>,
        mut viz_prod: HeapProd<f32>,
        err_fn: impl FnMut(cpal::Error) + Send + 'static,
    ) -> Result<Stream, cpal::Error>
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let mut sample_counter: usize = 0;

        device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                // 1. Process all pending synth events without blocking
                while let Ok(event) = event_rx.try_recv() {
                    engine.handle_event(event);
                }

                // 2. Render audio frames directly (stereo output)
                for frame in data.chunks_mut(channels) {
                    let (left, right) = engine.process_frame();

                    if channels >= 2 {
                        frame[0] = T::from_sample(left);
                        frame[1] = T::from_sample(right);
                        for ch in frame.iter_mut().skip(2) {
                            *ch = T::from_sample(0.5 * (left + right));
                        }
                    } else if channels == 1 {
                        frame[0] = T::from_sample(0.5 * (left + right));
                    }

                    // Push mono mix to visualizer buffer every 2 samples (downsample for oscilloscope display)
                    sample_counter = sample_counter.wrapping_add(1);
                    if sample_counter.is_multiple_of(2) {
                        let _ = viz_prod.try_push(0.5 * (left + right));
                    }
                }
            },
            err_fn,
            None,
        )
    }
}
