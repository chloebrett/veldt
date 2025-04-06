use crate::widget::selectable_value;
use egui::Ui;
use shared::model::EqConfig;
use shared::model::EqType;
use shared::types::{Freq, GainDB, KnobPosition};
use state::{Action, get_set};
use strum::IntoEnumIterator;

pub fn eq_control<F>(config: EqConfig, dispatch_effect: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    ui.label("Equalizer");

    ui.add(
        egui::Slider::from_get_set(
            20.0..=20000.0,
            get_set(config.fc.into(), |it| {
                dispatch_effect(Action::SetEqFc(it as Freq))
            }),
        )
        .text("Resonant frequency")
        .logarithmic(true),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.1..=100.0,
            get_set(config.q.into(), |it| {
                dispatch_effect(Action::SetEqQ(it as KnobPosition))
            }),
        )
        .text("Q value")
        .logarithmic(true),
    );

    ui.add(
        egui::Slider::from_get_set(
            -60.0..=60.0,
            get_set(config.gain.into(), |it| {
                dispatch_effect(Action::SetEqGain(it as GainDB))
            }),
        )
        .text("Gain (dB)"),
    );

    let eq_type = config.kind.clone();
    egui::ComboBox::from_label("EQ type")
        .selected_text(format!("{}", eq_type))
        .show_ui(ui, |ui| {
            for eq_type in EqType::iter() {
                selectable_value(
                    ui,
                    get_set(config.kind.clone(), |it| {
                        dispatch_effect(Action::SetEqKind(it))
                    }),
                    eq_type.clone(),
                    eq_type.to_string(),
                );
            }
        });
}
