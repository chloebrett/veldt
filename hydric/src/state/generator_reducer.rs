use crate::state::Action;
use shared::model::{GeneratorInstance, GeneratorType};
use web_sys::console;

pub fn generator_reducer(generator: &mut GeneratorInstance, action: &Action) {
    console::log_1(&format!("generator_reducer processing: {:?}", action.clone()).into());

    let config = match &mut generator.kind {
        GeneratorType::SimpleWave { config } => config,
    };

    match action {
        Action::SetWave(wave) => {
            config.wave = *wave;
        }
        Action::SetOscCount(osc_count) => {
            config.osc_count = *osc_count;
        }
        Action::SetDetuneCents(detune_cents) => {
            config.detune_cents = *detune_cents;
        }
        Action::SetEnvelope(envelope) => {
            let mut envelope = envelope.clone();
            let headroom = 1.0 - envelope.attack - envelope.decay - envelope.release;
            let max_attack = headroom + envelope.attack;
            let max_decay = headroom + envelope.decay;
            let max_release = headroom + envelope.release;

            if envelope.attack > max_attack {
                envelope.attack = max_attack;
            }
            if envelope.decay > max_decay {
                envelope.decay = max_decay;
            }
            if envelope.release > max_release {
                envelope.release = max_release;
            }

            config.envelope = envelope;
        }
        _ => {}
    }
}
