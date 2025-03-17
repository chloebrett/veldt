use crate::state::{Action, Selector, Store, get_set};
use crate::widget::selectable_value;
use egui::Ui;
use shared::model::Effect;
use shared::model::EqType;
use shared::types::{Freq, KnobPosition};
use strum::IntoEnumIterator;

pub fn eq_control(store: &Store, effect_index: usize, ui: &mut Ui) {
    let mixer_index = 0;
    let sel = Selector::Effect(mixer_index, effect_index);

    let effect_instance = store.get().project.mixer[mixer_index].effects[effect_index].clone();
    let config = match effect_instance.effect {
        Effect::SimpleEq { config } => config,
        _ => panic!(),
    };

    ui.label("Equalizer");

    ui.add(
        egui::Slider::from_get_set(
            20.0..=20000.0,
            get_set(config.fc.into(), |it| {
                store.dispatch(&sel, Action::SetEqFc(it as Freq))
            }),
        )
        .text("Resonant frequency")
        .logarithmic(true),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.1..=100.0,
            get_set(config.q.into(), |it| {
                store.dispatch(&sel, Action::SetEqQ(it as KnobPosition))
            }),
        )
        .text("Q value")
        .logarithmic(true),
    );

    let eq_type = config.kind.clone();
    egui::ComboBox::from_label("EQ type")
        .selected_text(format!("{}", eq_type))
        .show_ui(ui, |ui| {
            for eq_type in EqType::iter() {
                selectable_value(
                    ui,
                    get_set(config.kind.clone(), |it| {
                        store.dispatch(&sel, Action::SetEqKind(it))
                    }),
                    eq_type.clone(),
                    eq_type.to_string(),
                );
            }
        });

    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(effect_instance.meta.wet.into(), |it| {
                store.dispatch(&sel, Action::SetEffectWet(it as KnobPosition))
            }),
        )
        .text("EQ wet"),
    );
}
