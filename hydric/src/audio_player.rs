use chrono::{DateTime, Utc};
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp_frame::Stereo;
use log::error;
use mesic::graph::RenderGraph;
use std::sync::mpsc;
use wasm_thread::JoinHandle;

#[derive(Default)]
pub struct AudioPlayer {
    stream: Option<Stream>,
    producer_thread: Option<JoinHandle<()>>,
    pub start_timestamp: Option<DateTime<Utc>>,
}

impl AudioPlayer {
    pub fn init(&mut self, mut graph: RenderGraph) {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();
        let config: &cpal::StreamConfig = &config.into();

        let err_fn = |err| error!("an error occurred on stream: {}", err);
        let channels = config.channels as usize;

        // Total size of the audio buffer.
        let buffer_size = 5000;

        // Number of samples to render at a time.
        let chunk_size = 1000;

        // Don't start playing until this many samples have been produced.
        let buffer_threshold = 1000;

        let (tx, rx) = crossbeam_channel::bounded(buffer_size);

        log::info!(
            "Available parallelism: {:?}",
            wasm_thread::available_parallelism()
        );

        let producer_thread = wasm_thread::spawn(move || {
            log::info!("In producer_thread");

            loop {
                if tx.len() < buffer_size - chunk_size {
                    log::info!("Sent 1000 samples. Len: {}", tx.len());

                    for _ in 0..chunk_size {
                        let _ = tx.try_send(graph.next().unwrap_or([0.0; 2])).unwrap();
                    }
                } else {
                    sleep_ms(10);
                }
            }
        });

        // Tracks whether buffer_threshold has been reached.
        // Once this is true, it stays true.
        let mut latch = false;

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _| {
                    for frame in data.chunks_mut(channels) {
                        if !latch && rx.len() > buffer_threshold {
                            latch = true;
                        }

                        let value = if latch {
                            rx.try_recv().unwrap_or([0.0; 2])
                        } else {
                            [0.0; 2]
                        };

                        frame[0] = value[0]; // left
                        frame[1] = value[1]; // right
                    }
                },
                err_fn,
                None,
            )
            .unwrap();

        self.stream = Some(stream);
        self.producer_thread = Some(producer_thread);
    }

    pub fn play(&mut self) {
        self.stream.as_mut().expect("Call .init() first!").play().unwrap();
        self.start_timestamp = Some(chrono::offset::Utc::now());
    }
}

fn sleep_ms(ms: u32) {
    log::info!("Sleeping {} ms", ms);
    let secs = 0;
    let nanos = ms * 1000 * 1000;
    wasm_thread::sleep(std::time::Duration::new(secs, nanos));
}
