use super::{extract_inputs_2, extract_outputs};
use crate::graph::ProcessContext;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{EffectInstance, EffectMeta};
use state::EffectSelector;

/// Mixes two inputs down to one in the given wet/dry ratio.
/// The first input is the dry signal. The second is the wet signal.
/// wet = 1.0 returns only the wet signal.
/// wet = 0.0 returns only the dry signal.
/// wet = 0.5 returns a 50/50 mix.
/// And so on.
pub struct WetDryNode {
    selector: EffectSelector,
    meta: EffectMeta,
}

impl WetDryNode {
    pub fn new(selector: EffectSelector) -> Self {
        Self {
            selector,
            meta: EffectMeta::default(),
        }
    }

    fn process_channel(&self, out: &mut Buffer, dry: &Buffer, wet: &Buffer) {
        let d = 1.0 - self.meta.wet;

        if self.meta.mute {
            // TODO: make muting an effect temporarily short circuit it in the graph, so that
            // it doesn't run at all.
            out.copy_from_slice(dry);
        } else {
            for i in 0..Buffer::LEN {
                out[i] = dry[i] * d + wet[i] * self.meta.wet;
            }
        }
    }
}

impl Node<ProcessContext> for WetDryNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        // Apply changes from the store.
        if let Some(EffectInstance { meta, .. }) = &payload.store.try_select(&self.selector) {
            if *meta != self.meta {
                self.meta = meta.clone();
            }
        }

        debug_assert!(self.meta.wet >= 0.0 && self.meta.wet <= 1.0);

        let (out_left, out_right) = extract_outputs(output);
        let [(dry_left, dry_right), (wet_left, wet_right)] = extract_inputs_2(inputs);

        self.process_channel(out_left, dry_left, wet_left);
        self.process_channel(out_right, dry_right, wet_right);
    }
}
