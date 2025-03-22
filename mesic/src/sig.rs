use crate::consts::REFERENCE_PITCH;
use shared::model::PitchName;
use shared::types::{Freq, KnobPosition, PitchValue};
use std::cmp::{max, min};

pub trait Sig {
    /// Outputs a buffer containing the specified number of frames.
    // TODO: consider passing in a single buffer for reuse.
    fn buffer(&mut self, num_samples: u32) -> Vec<f32>;

    // TODO: consider storing a last_buffer?
}

/// Node with zero inputs and one output.
pub struct GeneratorNode {
    pub output: Option<Box<dyn Sig>>,
}

impl Sig for GeneratorNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        // TODO
        vec![0.0; num_samples as usize]
    }
}

/// Node with zero inputs and one output. Outputs from its buffer of already-rendered audio.
pub struct OutputNode {
    pub buffer: Vec<f32>,
    pub index: usize,
    pub output: Option<Box<dyn Sig>>,
}

impl Sig for OutputNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        let mut output = vec![0.0; num_samples as usize];
        for i in 0..min(num_samples as usize, self.buffer.len() - self.index) {
            output[i] = self.buffer[i + self.index];
        }
        self.index += num_samples as usize;
        output
    }
}

/// Node with one input and one output.
pub struct EffectNode {
    pub input: Box<dyn Sig>,
    pub output: Option<Box<dyn Sig>>,
}

impl Sig for EffectNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        // TODO
        self.input.buffer(num_samples)
    }
}

/// Node with two inputs and one output. Mixes its inputs in a specified ratio.
pub struct MixerNode {
    pub dry: Box<dyn Sig>,
    pub wet: Box<dyn Sig>,
    pub ratio: KnobPosition,
    pub output: Option<Box<dyn Sig>>,
}

impl Sig for MixerNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        sum(
            &mult(&self.wet.buffer(num_samples), self.ratio),
            &mult(&self.dry.buffer(num_samples), 1.0 - self.ratio),
        )
    }
}

/// Node with N inputs and one output. Adds its inputs to form the output.
pub struct AdderNode {
    pub inputs: Vec<Box<dyn Sig>>,
    pub output: Box<dyn Sig>,
}

impl Sig for AdderNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        let mut output = Vec::with_capacity(num_samples as usize);

        for input in &mut self.inputs {
            output = sum(&output, &input.buffer(num_samples));
        }

        output
    }
}

pub fn sum(a: &[f32], b: &[f32]) -> Vec<f32> {
    let max_len = max(a.len(), b.len());
    let mut output: Vec<f32> = vec![0.0; max_len];

    for (i, item) in output.iter_mut().enumerate() {
        let ai = a.get(i).unwrap_or(&0.0);
        let bi = b.get(i).unwrap_or(&0.0);
        *item = ai + bi;
    }

    output
}

pub fn mult(vec: &[f32], scalar: f32) -> Vec<f32> {
    vec.iter().map(|it| it * scalar).collect()
}

// Returns the frequency based on the distance from reference pitch.
pub fn freq(pitch_name: PitchName) -> Freq {
    let pitch: PitchValue = pitch_name.into();
    let reference: PitchValue = (*REFERENCE_PITCH.pitch_name).into();
    let interval: PitchValue = pitch - reference;

    // powf can't be run at compile time.
    // TODO: make this only run once.
    let semitone_increment: f32 = 2.0_f32.powf(1.0 / 12.0);
    REFERENCE_PITCH.frequency * semitone_increment.powf(interval as f32)
}
