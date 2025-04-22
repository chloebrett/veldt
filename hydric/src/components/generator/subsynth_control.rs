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
    // opacity initialisation
    const CHART_FILL_ALPHA: u8 = opacity_percentage_to_alpha(44.0);

    const fn opacity_percentage_to_alpha(opacity_percentage: f32) -> u8 {
        ((opacity_percentage/100.0) * 255.0) as u8
    }

    // colour related 'constants' - from_rgba_unmultiplied is not a const function so for consistency these are all defined with 'let' and lower snake case
    let green_outline = Color32::from_rgb(119, 167, 43);
    let green_fill = Color32::from_rgba_unmultiplied(147, 175, 100, CHART_FILL_ALPHA);
    let pink_outline = Color32::from_rgb(246, 83, 192);
    let pink_fill = Color32::from_rgba_unmultiplied(212, 132, 170, CHART_FILL_ALPHA);
    let orange_outline = Color32::from_rgb(227, 172, 84);
    let orange_fill = Color32::from_rgba_unmultiplied(215, 171, 53, CHART_FILL_ALPHA);

    let line_colours = vec![green_outline, pink_outline, orange_outline];
    let fill_colours = vec![green_fill, pink_fill, orange_fill];

    for oscillator_id in vec![0, 1, 2] {
        ui.push_id(oscillator_id, |ui| {
            subsynth_oscillator(
                &config.oscillators[oscillator_id],
                ui,
                &dispatch,
                &on_release,
                line_colours[oscillator_id],
                fill_colours[oscillator_id],
            );
        });
        ui.add_space(4.0);    
    }

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
