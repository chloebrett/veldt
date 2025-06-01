use super::{extract_inputs, extract_outputs};
use crate::eq::{ApplyFilter, eq_filter};
use crate::graph::ProcessContext;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Effect, EffectInstance, EqConfig};
use state::EffectSelector;
use crate::eq::filter::Filter;

pub struct EqNode {
    selector: EffectSelector,
    config: EqConfig,
    filter_left: Filter,
    filter_right: Filter,
}

impl EqNode {
    pub fn new(selector: EffectSelector) -> Self {
        let config = EqConfig::default();
        EqNode {
            selector,
            config: config.clone(),
            filter_left: eq_filter(&config),
            filter_right: eq_filter(&config),
        }
    }
}

impl Node<ProcessContext> for EqNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        // Apply changes from the store.
        if let Some(EffectInstance {
            it: Effect::SimpleEq(config),
            ..
        }) = &payload.store.try_select(&self.selector)
        {
            if *config != self.config {
                self.config = config.clone();
                self.filter_left = eq_filter(config);
                self.filter_right = eq_filter(config);
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
