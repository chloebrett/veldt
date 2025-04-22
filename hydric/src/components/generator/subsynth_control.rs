use super::super::{Piano, PianoOrientation};
use super::subsynth_oscillator;
use crate::view::View;
use eframe::egui;
use egui::{Color32, Ui};
use lazy_static::lazy_static;
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
        ((opacity_percentage / 100.0) * 255.0) as u8
    }

    lazy_static! {
        pub static ref GREEN_OUTLINE: Color32 = Color32::from_rgb(119, 167, 43);
        pub static ref GREEN_FILL: Color32 =
            Color32::from_rgba_unmultiplied(147, 175, 100, CHART_FILL_ALPHA);
        pub static ref PINK_OUTLINE: Color32 = Color32::from_rgb(246, 83, 192);
        pub static ref PINK_FILL: Color32 =
            Color32::from_rgba_unmultiplied(212, 132, 170, CHART_FILL_ALPHA);
        pub static ref ORANGE_OUTLINE: Color32 = Color32::from_rgb(227, 172, 84);
        pub static ref ORANGE_FILL: Color32 =
            Color32::from_rgba_unmultiplied(215, 171, 53, CHART_FILL_ALPHA);
        pub static ref LINE_COLOURS: [Color32; 3] =
            [*GREEN_OUTLINE, *PINK_OUTLINE, *ORANGE_OUTLINE,];
        pub static ref FILL_COLOURS: [Color32; 3] = [*GREEN_FILL, *PINK_FILL, *ORANGE_FILL,];
    }

    for oscillator_id in 0..3 {
        ui.push_id(oscillator_id, |ui| {
            subsynth_oscillator(
                &config.oscillators[oscillator_id],
                ui,
                &dispatch,
                &on_release,
                LINE_COLOURS[oscillator_id],
                FILL_COLOURS[oscillator_id],
            );
        });
        ui.add_space(4.0);
    }

    fn draw_piano(ui: &mut Ui) {
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

    draw_piano(ui);
}
