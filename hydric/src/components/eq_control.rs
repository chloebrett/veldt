use crate::state::{Action, Store};
use crate::widget::selectable_value;
use egui::Ui;
use shared::model::Effect;
use shared::model::EqType;
use shared::types::{Freq, KnobPosition};
use strum::IntoEnumIterator;

pub fn eq_control(store: &mut Store, effect_index: usize, ui: &mut Ui) {
    let effect_instance = store.project.mixer[0].effects[effect_index].clone();
    let config = match effect_instance.effect {
        Effect::SimpleEq { config } => config,
        _ => panic!(),
    };

    ui.label("Equalizer");

    ui.add(
        egui::Slider::from_get_set(20.0..=20000.0, |it| {
            it.map(|it| {
                store.dispatch(Action::SetEqFc {
                    channel_index: 0,
                    effect_index,
                    fc: it as Freq,
                })
            });
            config.fc.into()
        })
        .text("Resonant frequency")
        .logarithmic(true),
    );

    ui.add(
        egui::Slider::from_get_set(0.1..=100.0, |it| {
            it.map(|it| {
                store.dispatch(Action::SetEqQ {
                    channel_index: 0,
                    effect_index,
                    q: it as Freq,
                })
            });
            config.q.into()
        })
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
                    |it| {
                        it.map(|it| {
                            store.dispatch(Action::SetEqKind {
                                channel_index: 0,
                                effect_index,
                                kind: it,
                            });
                        });
                        config.kind.clone()
                    },
                    eq_type.clone(),
                    eq_type.to_string(),
                );
            }
        });

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
        .text("EQ wet"),
    );
}
