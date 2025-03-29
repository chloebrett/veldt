mod delay;
mod eq;

use shared::model::EffectInstance;

/// Trait corresponding to something that knows how to apply an effect.
/// Implemented for the various effect config types by the various effect plugins.
pub trait ApplyEffect {
    fn apply(&self, input: &[f32]) -> Vec<f32>;
}

pub fn apply_effects(
    input: Vec<f32>,
    effects: &[EffectInstance],
) -> Vec<f32> {
    let last = input;

    //for effect in effects {
    //    let effect_node = Rc::new(RefCell::new(EffectNode {
    //        input: Rc::clone(&last),
    //        effect: effect.clone(),
    //    }));
    //    let mixer_node = Rc::new(RefCell::new(MixerNode {
    //        dry: Rc::clone(&last),
    //        wet: effect_node,
    //        ratio: effect.meta.wet,
    //    }));
    //    last = mixer_node;
    //}

    last
}
