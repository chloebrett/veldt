use crate::widget::{get_set, knob, selectable_value};
use egui::Ui;
use shared::model::EqConfig;
use shared::model::EqType;
use state::Action;
use strum::IntoEnumIterator;

pub fn eq_control<F, G>(config: &EqConfig, dispatch: F, on_release: G, ui: &mut Ui)
where
    F: Fn(Action),
    G: Fn(),
{
    knob(
        ui,
        "Freq",
        config.fc,
        |it| dispatch(Action::SetEqFc(it)),
        20.0..=20000.0, // TODO: logarithmic
        &on_release,
    );

    knob(
        ui,
        "Q",
        config.q,
        |it| dispatch(Action::SetEqQ(it)),
        0.1..=100.0, // TODO: logarithmic
        &on_release,
    );

    knob(
        ui,
        "Gain",
        config.gain,
        |it| dispatch(Action::SetEqGain(it)),
        -60.0..=60.0,
        &on_release,
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
