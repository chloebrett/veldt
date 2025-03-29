use egui::Ui;
use shared::model::Effect;
use shared::types::{KnobPosition, Milliseconds, Volume};
use state::{Action, Selector, Store, get_set};

pub fn delay_control(store: &Store, effect_index: usize, ui: &mut Ui) {
    let mixer_index = 0;
    let sel = Selector::Effect(mixer_index, effect_index);

    let effect_instance = store.get().project.mixer[mixer_index].effects[effect_index].clone();
    let config = match effect_instance.effect {
        Effect::SimpleDelay { config } => config,
        _ => panic!(),
    };

    ui.label("Delay");

    let amplitude = config.amplitude as f64;
    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(amplitude, |it| {
                store.dispatch(&sel, Action::SetDelayAmplitude(it as Volume))
            }),
        )
        .text("Delay amplitude"),
    );
    ui.add(
        egui::Slider::from_get_set(
            1.0..=1000.0,
            get_set(config.delay_ms.into(), |it| {
                store.dispatch(&sel, Action::SetDelayMs(it as Milliseconds))
            }),
        )
        .text("Delay ms"),
    );
    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(effect_instance.meta.wet.into(), |it| {
                store.dispatch(&sel, Action::SetEffectWet(it as KnobPosition))
            }),
        )
        .text("Delay wet"),
    );
}
