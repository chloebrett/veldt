use chrono::{DateTime, Utc};
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use web_sys::console;

pub struct Handle {
    stream: Stream,
    pub start_timestamp: DateTime<Utc>,
}

pub fn play(audio: &[f32]) -> Handle {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), audio),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), audio),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), audio),
        // not all supported sample formats are included in this example
        _ => panic!("Unsupported sample format!"),
    }
}

fn run<'a, T>(device: &cpal::Device, config: &cpal::StreamConfig, audio: &'a [f32]) -> Handle
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;

    let mut sample_clock: usize = 0;
    let audio = audio.to_owned();
    let mut next_value = move || {
        if sample_clock >= audio.len() {
            0.0
        } else {
            let sample = audio[sample_clock];
            sample_clock += 1;
            sample
        }
    };

    // TODO: replace with egui logger
    let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_data(data, channels, &mut next_value),
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();
    Handle {
        stream,
        start_timestamp: chrono::offset::Utc::now(),
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
