use super::ProcessContext;
use super::{extract_inputs, extract_outputs};
use crate::effect::{ApplyFilter, eq_filter};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Effect, EffectInstance, EqConfig};

pub struct EqNode {
    mixer_index: usize,
    effect_index: usize,
    config: EqConfig,
    filter_left: Box<dyn ApplyFilter + Send>,
    filter_right: Box<dyn ApplyFilter + Send>,
}

impl EqNode {
    pub fn new(mixer_index: usize, effect_index: usize, config: EqConfig) -> Self {
        EqNode {
            mixer_index,
            effect_index,
            config: config.clone(),
            filter_left: eq_filter(&config),
            filter_right: eq_filter(&config),
        }
    }
}

impl Node<ProcessContext> for EqNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        if let Some(mixer) = &payload.store.project.mixer.get(self.mixer_index) {
            if let Some(EffectInstance {
                effect: Effect::SimpleEq { config, .. },
                ..
            }) = mixer.effects.get(self.effect_index)
            {
                if *config != self.config {
                    self.config = config.clone();
                    self.filter_left = eq_filter(config);
                    self.filter_right = eq_filter(config);
                }
            }
        }

        let (out_left, out_right) = extract_outputs(output);
        let (in_left, in_right) = extract_inputs(inputs)[0];

        out_left.copy_from_slice(in_left);
        self.filter_left.apply(out_left);

        out_right.copy_from_slice(in_right);
        self.filter_right.apply(out_right);
    }
}
