use crate::state::{Action, Store};
use egui::Ui;
use shared::model::Effect;
use shared::types::{KnobPosition, Milliseconds, Volume};

pub fn delay_control(store: &Store, effect_index: usize, ui: &mut Ui) {
    let effect_instance = store.get().project.mixer[0].effects[effect_index].clone();
    let config = match effect_instance.effect {
        Effect::SimpleDelay { config } => config,
        _ => panic!(),
    };

    ui.label("Delay");

    let amplitude = config.amplitude as f64;
    ui.add(
        egui::Slider::from_get_set(0.0..=1.0, |it| {
            it.map(|it| {
                if it != amplitude {
                    store.dispatch(Action::SetDelayAmplitude {
                        channel_index: 0,
                        effect_index,
                        amplitude: it as Volume,
                    })
                }
            });
            amplitude
        })
        .text("Delay amplitude"),
    );
    ui.add(
        egui::Slider::from_get_set(1.0..=1000.0, |it| {
            it.map(|it| {
                store.dispatch(Action::SetDelayMs {
                    channel_index: 0,
                    effect_index,
                    delay_ms: it as Milliseconds,
                })
            });
            config.delay_ms.into()
        })
        .text("Delay ms"),
    );
    ui.add(
        egui::Slider::from_get_set(0.0..=1.0, |it| {
            it.map(|it| {
                store.dispatch(Action::SetEffectWet {
                    channel_index: 0,
                    effect_index,
                    wet: it as KnobPosition,
                })
            });
            effect_instance.meta.wet.into()
        })
        .text("Delay wet"),
    );
}
