use crate::consts::SAMPLE_RATE;
use dasp_graph::{Buffer, Input, Node};
use shared::model::CompressorConfig;
use std::iter::repeat_n;

pub type EnvelopeDetector = dasp_envelope::Detector<f32, dasp_rms::Rms<f32, Vec<f32>>>;

pub struct CompressorNode {
    config: CompressorConfig,
    detector: EnvelopeDetector,
}

impl CompressorNode {
    pub fn new(config: CompressorConfig) -> Self {
        let attack_frames = (config.attack_ms / 1000.0 * SAMPLE_RATE as f32) as usize;
        let release_frames = (config.release_ms / 1000.0 * SAMPLE_RATE as f32) as usize;

        // TODO: consider whether this is the appropriate size for the ring buffer.
        let ring_buffer_frames = attack_frames + release_frames;

        // TODO: de-duplicate this logic, which is also repeated in graph.rs for creating delay
        // nodes.
        let mut vec = Vec::with_capacity(ring_buffer_frames);
        vec.extend(repeat_n(0.0, ring_buffer_frames));
        let buffer = dasp_ring_buffer::Fixed::from(vec);

        let rms = dasp_rms::Rms::new(buffer);
        CompressorNode {
            config,
            detector: dasp_envelope::Detector::new(
                rms,
                attack_frames as f32,
                release_frames as f32,
            ),
        }
    }
}

impl Node for CompressorNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        let threshold = self.config.threshold;
        let ratio_recip = 1.0 / self.config.ratio;

        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.first().expect("Expected one input").buffers())
        {
            let buf: Vec<f32> = in_buf
                .iter()
                .map(|it| {
                    let rms = self.detector.next(*it);

                    // TODO: use dB for threshold.
                    // TODO: also support using the compressor as a downward expander.
                    let pre_gain = if rms > threshold {
                        // Apply the compression ratio.
                        threshold + (it - threshold) * ratio_recip
                    } else {
                        *it
                    };

                    // TODO: use dB for makeup gain.
                    pre_gain * self.config.gain
                })
                .collect();
            out_buf.copy_from_slice(&buf);
        }
    }
}
