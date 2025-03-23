mod delay;
mod eq;

use crate::Sig;
use crate::sig::{EffectNode, MixerNode};
use shared::model::EffectInstance;
use std::cell::RefCell;
use std::rc::Rc;

/// Trait corresponding to something that knows how to apply an effect.
/// Implemented for the various effect config types by the various effect plugins.
pub trait ApplyEffect {
    fn apply(&self, input: &[f32]) -> Vec<f32>;
}

pub fn apply_effects(
    input: Rc<RefCell<dyn Sig>>,
    effects: &[EffectInstance],
) -> Rc<RefCell<dyn Sig>> {
    let mut last = input;

    for effect in effects {
        let effect_node = Rc::new(RefCell::new(EffectNode {
            input: last.clone(),
            effect: effect.clone(),
        }));
        let mixer_node = Rc::new(RefCell::new(MixerNode {
            dry: last.clone(),
            wet: effect_node,
            ratio: effect.clone().meta.wet,
        }));
        last = mixer_node;
    }

    last
}
