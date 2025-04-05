use crate::consts::SAMPLE_RATE;
use dasp_graph::{Buffer, Input, Node};
use shared::model::CompressorConfig;
use std::cmp::max;
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
        // TODO: potentially we need two buffers with different lengths - one for tracking attack
        // and one for release.
        // Note: needs at least one frame or the ring buffer panics.
        let ring_buffer_frames = max(max(attack_frames, release_frames), 1);

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

#[inline]
/// Compresses an audio sample (in the amplitude sense, not the WinRAR sense) based on the output
/// of an amplitude detector, a compression threshold, and ratio (represented as a reciprocal).
/// The reciprocal is used to save on division.
fn compress(input: f32, detector: f32, threshold: f32, ratio_recip: f32) -> f32 {
    // TODO: use dB for threshold.
    if detector > threshold {
        // Apply the compression ratio.
        // The output vs input graph looks like the blue diagram in this article:
        // https://www.iconcollective.edu/audio-compressor-ratio-explained
        // TODO: This maths isn't quite right. Follow p516 in DAEP for the final gain calculation.
        input.signum() * (threshold + (input.abs() - threshold) * ratio_recip)
    } else {
        input
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
            out_buf.copy_from_slice(
                &in_buf
                    .iter()
                    .map(|it| {
                        let rms = self.detector.next(*it);

                        // TODO: also support using the compressor as a downward expander.
                        let pre_gain = compress(*it, rms, threshold, ratio_recip);

                        // TODO: use dB for makeup gain.
                        pre_gain * self.config.gain
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::RenderGraph;
    use crate::wave::freq;
    use assert_float_eq::assert_float_absolute_eq;
    use shared::model::{Effect, EffectInstance, EffectMeta, PitchName, ScaleValue};

    const FLOAT_THRES: f32 = 1e-6;

    #[test]
    fn compress_negative() {
        let input = -0.8;
        let detector = 1.0;
        let threshold = 0.5;
        let ratio_recip = 0.5;
        let output = compress(input, detector, threshold, ratio_recip);

        assert_eq!(output, -0.65);
    }

    #[test]
    fn identity_compressor() {
        // ARRANGE
        let input = generate_signal_seconds(1.0);
        let graph = make_graph(
            input.clone(),
            CompressorConfig {
                attack_ms: 10.0,
                release_ms: 10.0,
                gain: 1.0,
                ratio: 1.0, // this is what makes it 'identity' (no-op)
                threshold: 0.5,
            },
        );

        // ACT
        let output: Vec<_> = graph.collect();

        // ASSERT
        assert_signals_approx_eq(output, input);
    }

    #[test]
    fn instant_attack_and_release() {
        // Asserts that constructing a compressor with no attack/release doesn't panic.
        // Also asserts that the output is a pure function of the input.

        // ARRANGE
        let input = generate_signal_seconds(1.0);
        let threshold = 0.5;
        let ratio = 3.0;
        let graph = make_graph(
            input.clone(),
            CompressorConfig {
                attack_ms: 0.0,
                release_ms: 0.0,
                gain: 1.0,
                ratio,
                threshold,
            },
        );

        // ACT
        let output: Vec<_> = graph.collect();
        // Output should be equivalent to applying the compressor function straight to the input.
        let expected: Vec<_> = input
            .iter()
            .map(|it| {
                // Note using 'it' as both input and detector.
                // (with .abs() for detector).
                compress(*it, it.abs(), threshold, 1.0 / ratio)
            })
            .collect();

        // ASSERT
        assert_signals_approx_eq(output, expected);
    }

    #[test]
    #[ignore] // TODO: add final gain calc, which will fix this.
    fn standard_compressor_no_makeup_gain() {
        // ARRANGE
        let input = generate_signal_seconds(1.0);
        let graph = make_graph(
            input.clone(),
            CompressorConfig {
                attack_ms: 30.0,
                release_ms: 100.0,
                gain: 1.0,
                ratio: 5.5,
                threshold: 0.7,
            },
        );

        // ACT
        let output: Vec<_> = graph.collect();

        // ASSERT
        for (y, x) in output.into_iter().zip(input.into_iter()) {
            // Absolute output should always be less than absolute input.
            // Polarity should be the same.
            assert!(y.abs() < x.abs() + FLOAT_THRES, "{y}, {x}");
            assert_eq!(y.signum(), x.signum());
        }
    }

    #[test]
    #[ignore] // TODO: add final gain calc, which will fix this.
    fn standard_compressor_with_makeup_gain() {
        // ARRANGE
        let input = generate_signal_seconds(1.0);
        let makeup_gain = 1.5; // 1.5x multiplier
        let graph = make_graph(
            input.clone(),
            CompressorConfig {
                attack_ms: 30.0,
                release_ms: 100.0,
                gain: makeup_gain,
                ratio: 5.5,
                threshold: 0.7,
            },
        );

        // ACT
        let output: Vec<_> = graph.collect();

        // ASSERT
        for (y, x) in output.into_iter().zip(input.into_iter()) {
            assert!(
                y.abs() <= (x.abs() * makeup_gain) + FLOAT_THRES,
                "{y}, {x}, {makeup_gain}, {}",
                x * makeup_gain
            );
            assert_eq!(y.signum(), x.signum(), "{y}, {x}");
        }
    }

    #[test]
    fn hard_limiter() {
        // ARRANGE
        let input = generate_signal_seconds(1.0);
        let attack_ms = 5.0;
        let attack_samples = (attack_ms / 1000.0 * SAMPLE_RATE as f32) as usize;
        let threshold = 0.21;
        let graph = make_graph(
            input.clone(),
            CompressorConfig {
                attack_ms,
                // TODO: fix behaviour when release > attack, and test it.
                // Currently this test fails if release > attack.
                release_ms: 5.0,
                gain: 1.0,
                ratio: f32::INFINITY,
                threshold,
            },
        );

        // ACT
        let output: Vec<_> = graph.collect();

        // ASSERT
        for ((i, y), x) in output.into_iter().enumerate().zip(input.into_iter()) {
            if i < attack_samples {
                // While the limiter is kicking in, just assert that the output is less than or
                // equal to the input.
                assert!(y.abs() <= x.abs() + FLOAT_THRES, "{y}, {x}");
                assert_eq!(y.signum(), x.signum(), "{y}, {x}");
            } else {
                // Once it's kicked in, assert that the output is less than or equal to the
                // compressor threshold (plus or minus the float threshold).
                assert!(y.abs() <= threshold + FLOAT_THRES, "{y}, {x}, {threshold}");
                assert_eq!(y.signum(), x.signum(), "{y}, {x}");
            }
        }
    }

    // TODO:
    // * Test release.
    // * Test more complex input signals.

    fn make_graph(input: Vec<f32>, config: CompressorConfig) -> RenderGraph {
        let mut graph = RenderGraph::from_vec(input.clone());
        graph.add_effect_with_mixer(EffectInstance {
            effect: Effect::SimpleCompressor { config },
            meta: EffectMeta { id: 0, wet: 1.0 },
        });
        graph
    }

    fn assert_signals_approx_eq(first: Vec<f32>, second: Vec<f32>) {
        // TODO: make the errors for this more readable,
        // and perhaps make our own macro.
        for (a, b) in first.iter().zip(second.iter()) {
            assert_float_absolute_eq!(*a, *b, FLOAT_THRES);
        }
    }

    fn generate_signal_seconds(seconds: f32) -> Vec<f32> {
        let samples = (SAMPLE_RATE as f32 * seconds) as usize;
        generate_signal(samples)
    }

    /// Generates an A4 sine wave that lasts for the given number of samples.
    fn generate_signal(samples: usize) -> Vec<f32> {
        let pitch = PitchName {
            scale_value: ScaleValue::A,
            octave: 4,
        };
        (0..samples as usize)
            .map(|it| (it as f32 / SAMPLE_RATE as f32 * freq(pitch)).sin())
            .collect()
    }
}
