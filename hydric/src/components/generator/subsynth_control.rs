use super::super::{Piano, PianoOrientation};
use super::subsynth_oscillator;
use crate::view::View;
use eframe::egui;
use egui::{Color32, Ui};
use shared::model::SubSynthConfig;
use shared::{
    model::{PitchName, ScaleValue},
    types::PitchValue,
};
use state::Action;

pub fn subsynth_control<F, G>(config: &SubSynthConfig, dispatch: F, on_release: G, ui: &mut Ui)
where
    F: Fn(Action),
    G: Fn(),
{
    // OSC 1
    ui.push_id(0, |ui| {
        subsynth_oscillator(
            &config.oscillators[0],
            ui,
            &dispatch,
            &on_release,
            Color32::from_rgb(119, 167, 43),
            Color32::from_rgba_unmultiplied(147, 175, 100, (0.44 * 255.0) as u8),
        );
    });
    ui.add_space(4.0);

    // OSC 2
    ui.push_id(1, |ui| {
        subsynth_oscillator(
            &config.oscillators[1],
            ui,
            &dispatch,
            &on_release,
            Color32::from_rgb(246, 83, 192),
            Color32::from_rgba_unmultiplied(212, 132, 170, (0.44 * 255.0) as u8),
        );
    });
    ui.add_space(4.0);

    // OSC 3
    ui.push_id(2, |ui| {
        subsynth_oscillator(
            &config.oscillators[2],
            ui,
            &dispatch,
            &on_release,
            Color32::from_rgb(227, 172, 84),
            Color32::from_rgba_unmultiplied(215, 171, 53, (0.44 * 255.0) as u8),
        );
    });

    ui.add_space(4.0);
    // piano stuff
    let min_note: PitchValue = PitchName {
        scale_value: ScaleValue::A,
        octave: 1,
    }
    .into();
    let max_note: PitchValue = PitchName {
        scale_value: ScaleValue::C,
        octave: 8,
    }
    .into();
    Piano::new(max_note, min_note - 1)
        .with_orientation(PianoOrientation::Horizontal)
        .ui(ui);
}
