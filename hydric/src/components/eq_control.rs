use crate::state::{Action, Store, get_set};
use crate::widget::selectable_value;
use egui::Ui;
use shared::model::Effect;
use shared::model::EqType;
use shared::types::{Freq, KnobPosition};
use strum::IntoEnumIterator;

pub fn eq_control(store: &Store, effect_index: usize, ui: &mut Ui) {
    let effect_instance = store.get().project.mixer[0].effects[effect_index].clone();
    let config = match effect_instance.effect {
        Effect::SimpleEq { config } => config,
        _ => panic!(),
    };

    ui.label("Equalizer");

    ui.add(
        egui::Slider::from_get_set(
            20.0..=20000.0,
            get_set(config.fc.into(), |it| {
                store.dispatch(Action::SetEqFc {
                    channel_index: 0,
                    effect_index,
                    fc: it as Freq,
                })
            }),
        )
        .text("Resonant frequency")
        .logarithmic(true),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.1..=100.0,
            get_set(config.q.into(), |it| {
                store.dispatch(Action::SetEqQ {
                    channel_index: 0,
                    effect_index,
                    q: it as Freq,
                })
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
                        store.dispatch(Action::SetEqKind {
                            channel_index: 0,
                            effect_index,
                            kind: it,
                        });
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
                store.dispatch(Action::SetEffectWet {
                    channel_index: 0,
                    effect_index,
                    wet: it as KnobPosition,
                })
            }),
        )
        .text("EQ wet"),
    );
}
