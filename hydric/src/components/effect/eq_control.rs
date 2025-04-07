use crate::widget::{FloatRange, knob, selectable_value};
use egui::Ui;
use shared::model::EqConfig;
use shared::model::EqType;
use state::{Action, get_set};
use strum::IntoEnumIterator;

pub fn eq_control<F>(config: &EqConfig, dispatch: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    knob(
        ui,
        "Freq",
        config.fc,
        |it| dispatch(Action::SetEqFc(it)),
        FloatRange(20.0, 20000.0), // TODO: logarithmic
    );

    knob(
        ui,
        "Q",
        config.q,
        |it| dispatch(Action::SetEqQ(it)),
        FloatRange(0.1, 100.0), // TODO: logarithmic
    );

    knob(
        ui,
        "Gain",
        config.gain,
        |it| dispatch(Action::SetEqGain(it)),
        FloatRange(-60.0, 60.0),
    );

    let eq_type = config.kind.clone();
    egui::ComboBox::from_label("EQ type")
        .selected_text(format!("{}", eq_type))
        .show_ui(ui, |ui| {
            for eq_type in EqType::iter() {
                selectable_value(
                    ui,
                    get_set(config.kind.clone(), |it| dispatch(Action::SetEqKind(it))),
                    eq_type.clone(),
                    eq_type.to_string(),
                );
            }
        });
}
