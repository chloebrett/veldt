use crate::consts::REFERENCE_PITCH;
use crate::effect::ApplyEffect;
use shared::model::Effect;
use shared::model::{EffectInstance, PitchName};
use shared::types::{Freq, KnobPosition, PitchValue, Volume};
use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;

/// TODO: use a single shared graph type to control ownership of nodes.
pub type SigRef = Rc<RefCell<dyn Sig>>;

pub trait Sig {
    /// Outputs a buffer containing the specified number of frames.
    // TODO: consider passing in a single buffer for reuse.
    fn buffer(&mut self, num_samples: u32) -> Vec<f32>;

    // TODO: consider storing a last_buffer?
}

/// Node with zero inputs and one output.
pub struct GeneratorNode {}

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
}

impl Sig for OutputNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        let mut output = vec![0.0; num_samples as usize];
        let remaining = self.buffer.len() as i32 - self.index as i32;
        for i in 0i32..min(num_samples as i32, remaining) {
            output[i as usize] = self.buffer[i as usize + self.index];
        }
        //TODO: state
        //self.index += num_samples as usize;
        output
    }
}

/// Node with one input and one output.
pub struct EffectNode {
    pub input: SigRef,
    pub effect: EffectInstance,
}

impl Sig for EffectNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        let dry = self.input.as_ref().borrow_mut().buffer(num_samples);
        match &self.effect.effect {
            Effect::SimpleDelay { config } => config.apply(&dry),
            Effect::SimpleEq { config } => config.apply(&dry),
            _ => panic!("Effect not implemented yet!"),
        }
    }
}

/// Node with one input and one output and a volume control.
/// Can clip the post-gain signal if desired.
pub struct AmpNode {
    pub input: SigRef,
    pub volume: Volume,
    pub should_clip: bool,
}

impl Sig for AmpNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        // TODO
        self.input
            .as_ref()
            .borrow_mut()
            .buffer(num_samples)
            .into_iter()
            .map(|it| {
                let mut amped = it * self.volume;
                if self.should_clip {
                    amped = amped.clamp(-1.0, 1.0);
                }
                amped
            })
            .collect()
    }
}

/// Node with two inputs and one output. Mixes its inputs in a specified ratio.
pub struct MixerNode {
    pub dry: SigRef,
    pub wet: SigRef,
    pub ratio: KnobPosition,
}

impl Sig for MixerNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        let dry = self.dry.as_ref().borrow_mut().buffer(num_samples);
        let wet = self.wet.as_ref().borrow_mut().buffer(num_samples);

        let out = sum(&mult(&dry, 1.0 - self.ratio), &mult(&wet, self.ratio));
        out
    }
}

/// Node with N inputs and one output. Adds its inputs to form the output.
pub struct AdderNode {
    pub inputs: Vec<SigRef>,
}

impl Sig for AdderNode {
    fn buffer(&mut self, num_samples: u32) -> Vec<f32> {
        let mut output = Vec::with_capacity(num_samples as usize);

        for input in &mut self.inputs {
            output = sum(&output, &input.as_ref().borrow_mut().buffer(num_samples));
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
