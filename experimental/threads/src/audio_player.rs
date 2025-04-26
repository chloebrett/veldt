use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wasm_thread;
use wasm_thread::JoinHandle;

pub struct AudioPlayer {
    freq_rx: crossbeam_channel::Receiver<f64>,
    producer_thread: Option<JoinHandle<()>>,
    consumer_stream: Option<Stream>,
}

impl AudioPlayer {
    pub fn new(rx: crossbeam_channel::Receiver<f64>) -> Self {
        Self {
            freq_rx: rx,
            producer_thread: None,
            consumer_stream: None,
        }
    }

    pub fn init(&mut self) {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let output_config = device.default_output_config().unwrap();
        let config: &cpal::StreamConfig = &output_config.clone().into();

        let err_fn = |err| error!("an error occurred on stream: {}", err);
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0 as f32;

        let buffer_size = 5000;
        let chunk_size = 1000;
        let (tx, rx) = crossbeam_channel::bounded(buffer_size);

        log::info!(
            "Available parallelism: {:?}",
            wasm_thread::available_parallelism()
        );

        // Clone the frequency receiver so that the producer thread can take ownership.
        // Thread closures require a static lifetime.
        let freq_rx = self.freq_rx.clone();

        // Producer thread.
        self.producer_thread = Some(wasm_thread::spawn(move || {
            log::info!("In producer_thread");

            // Produce a sinusoid of maximum amplitude.
            let mut phase = 0f32;
            let phase_inc = 1.0 / sample_rate;
            let mut next_sample = |freq: f32| {
                phase += phase_inc * freq;
                phase = phase % 1.0;
                (phase * std::f32::consts::TAU).sin()
            };

            let mut freq = 0.0;

            loop {
                if tx.len() < buffer_size - chunk_size {
                    log::info!("Sent 1000 samples. Len: {}", tx.len());

                    while let Ok(new_freq) = freq_rx.try_recv() {
                        log::info!("Updated freq! {}", new_freq);
                        freq = new_freq as f32;
                    }

                    for _ in 0..chunk_size {
                        let _ = tx.try_send(next_sample(freq)).unwrap();
                    }
                } else {
                    let ms = 10;
                    log::info!("Sleeping {} ms", ms);
                    let secs = 0;
                    let nanos = ms * 1000 * 1000;
                    wasm_thread::sleep(std::time::Duration::new(secs, nanos));
                }
            }
        }));

        // Consumer / audio player.
        let config: &cpal::StreamConfig = &output_config.clone().into();
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _| {
                    for frame in data.chunks_mut(channels) {
                        let value = rx.try_recv().unwrap_or(0.0);
                        frame[0] = value;
                        frame[1] = value;
                    }
                },
                err_fn,
                None,
            )
            .unwrap();
        let _ = stream.play();
        self.consumer_stream = Some(stream);
    }
}
