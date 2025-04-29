use super::ProcessContext;
use super::{extract_inputs_2, extract_outputs};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{EffectInstance, EffectMeta};

/// Mixes two inputs down to one in the given wet/dry ratio.
/// The first input is the dry signal. The second is the wet signal.
/// wet = 1.0 returns only the wet signal.
/// wet = 0.0 returns only the dry signal.
/// wet = 0.5 returns a 50/50 mix.
/// And so on.
pub struct MixerNode {
    mixer_index: usize,
    effect_index: usize,
    meta: EffectMeta,
}

impl MixerNode {
    pub fn new(mixer_index: usize, effect_index: usize, meta: EffectMeta) -> Self {
        Self {
            mixer_index,
            effect_index,
            meta: meta.clone(),
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

impl Node<ProcessContext> for MixerNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        // Apply any changes from the store if applicable.
        if let Some(mixer) = &payload.store.project.mixer.get(self.mixer_index) {
            if let Some(EffectInstance { meta, .. }) = &mixer.effects.get(self.effect_index) {
                if *meta != self.meta {
                    self.meta = meta.clone();
                }
            }
        }

        debug_assert!(self.meta.wet >= 0.0 && self.meta.wet <= 1.0);

        let (out_left, out_right) = extract_outputs(output);
        let [(dry_left, dry_right), (wet_left, wet_right)] = extract_inputs_2(inputs);

        self.process_channel(out_left, dry_left, wet_left);
        self.process_channel(out_right, dry_right, wet_right);
    }
}
