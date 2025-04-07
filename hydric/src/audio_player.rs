use chrono::{DateTime, Utc};
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp_frame::Stereo;
use mesic::graph::RenderGraph;
use shared::logger::error;
use std::sync::mpsc;

pub struct Handle {
    _stream: Stream,
    pub start_timestamp: DateTime<Utc>,
}

pub fn play(mut graph: RenderGraph, pre_render: bool) -> Handle {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    let config: &cpal::StreamConfig = &config.into();

    let err_fn = |err| error(&format!("an error occurred on stream: {}", err));
    let channels = config.channels as usize;

    let (tx, rx) = mpsc::channel();

    // TODO: maybe this could avoid an allocation.
    let mut next_sample: Box<dyn FnMut() -> Stereo<f32> + Send> = if pre_render {
        for frame in graph {
            let _ = tx.send(frame);
        }

        Box::new(move || rx.recv().unwrap_or([0.0; 2]))
    } else {
        Box::new(move || graph.next().unwrap_or([0.0; 2]))
    };

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [f32], _| {
                for frame in data.chunks_mut(channels) {
                    let value = next_sample();
                    frame[0] = value[0]; // left
                    frame[1] = value[1]; // right
                }
            },
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();
    Handle {
        _stream: stream,
        start_timestamp: chrono::offset::Utc::now(),
    }
}
