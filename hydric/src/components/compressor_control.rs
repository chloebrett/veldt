use egui::Ui;
use shared::model::Effect;
use shared::types::KnobPosition;
use shared::types::Milliseconds;
use shared::types::Volume;
use state::{Action, Selector, Store, get_set};

pub fn compressor_control(store: &Store, effect_index: usize, ui: &mut Ui) {
    let mixer_index = 0;
    let sel = Selector::Effect(mixer_index, effect_index);

    let effect_instance = store.get().project.mixer[mixer_index].effects[effect_index].clone();
    let config = match effect_instance.effect {
        Effect::SimpleCompressor { config } => config,
        _ => panic!(),
    };

    ui.label("Compressor");

    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(config.threshold.into(), |it| {
                store.dispatch(&sel, Action::SetCompressorThreshold(it as Volume))
            }),
        )
        .text("Threshold"),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.0..=1000.0,
            get_set(config.attack_ms.into(), |it| {
                store.dispatch(&sel, Action::SetCompressorAttackMs(it as Milliseconds))
            }),
        )
        .text("Attack"),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.0..=1000.0,
            get_set(config.release_ms.into(), |it| {
                store.dispatch(&sel, Action::SetCompressorReleaseMs(it as Milliseconds))
            }),
        )
        .text("Release"),
    );

    ui.add(
        egui::Slider::from_get_set(
            1.0..=100.0,
            get_set(config.ratio.into(), |it| {
                store.dispatch(&sel, Action::SetCompressorRatio(it as KnobPosition))
            }),
        )
        .text("Ratio")
        .logarithmic(true),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(config.gain.into(), |it| {
                store.dispatch(&sel, Action::SetCompressorGain(it as Volume))
            }),
        )
        .text("Gain"),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(effect_instance.meta.wet.into(), |it| {
                store.dispatch(&sel, Action::SetEffectWet(it as KnobPosition));
            }),
        )
        .text("Compressor wet"),
    );
}
