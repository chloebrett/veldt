use super::SimpleWaveVisualiser;
use crate::view::View;
use crate::widget::{
    add_typable_knob, get_set, inner_frame, int_slider, outer_frame, selectable_value, styled_knob,
};
use eframe::egui;
use egui::{Color32, Margin, Ui, Vec2};
use shared::model::{Oscillator, WaveType};
use state::{Action, FloatField, TypeField, UintField};
use strum::IntoEnumIterator;

pub struct StingrayOscillatorView<'a, F: Fn(Action), G: Fn()> {
    config: &'a Oscillator,
    dispatch: F,
    on_release: G,
    line_colour: Color32,
    fill_colour: Color32,
}

impl<'a, F: Fn(Action), G: Fn()> StingrayOscillatorView<'a, F, G> {
    pub fn new(
        config: &'a Oscillator,
        dispatch: F,
        on_release: G,
        line_colour: Color32,
        fill_colour: Color32,
    ) -> Self {
        Self {
            config,
            dispatch,
            on_release,
            line_colour,
            fill_colour,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for StingrayOscillatorView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config,
            ref dispatch,
            ref on_release,
            line_colour,
            fill_colour,
        } = *self;

        fn draw_wave_selection<F>(
            ui: &mut Ui,
            config: &Oscillator,
            dispatch: &F,
            line_colour: Color32,
            fill_colour: Color32,
        ) where
            F: Fn(Action),
        {
            inner_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    egui::ComboBox::from_label("")
                        .selected_text(config.wave.to_string())
                        .show_ui(ui, |ui| {
                            for wave in WaveType::iter() {
                                selectable_value(
                                    ui,
                                    get_set(config.wave, |wave_type| {
                                        dispatch(Action::SetChild(TypeField::Wave(wave_type)))
                                    }),
                                    wave,
                                    wave.to_string(),
                                );
                            }
                        });
                    ui.add_space(20.0);
                    let visualiser = SimpleWaveVisualiser::new(
                        config.wave,
                        line_colour,
                        fill_colour,
                        1.0,
                        Vec2::new(130.0, 70.0),
                    );

                    visualiser.show(ui);
                    ui.add_space(8.0);
                });
            });
        }

        fn draw_oscillator_controls<F, G>(
            ui: &mut Ui,
            config: &Oscillator,
            dispatch: &F,
            on_release: &G,
        ) where
            F: Fn(Action),
            G: Fn(),
        {
            const KNOB_SPACE: f32 = 4.0;
            let volume_knob = styled_knob(
                config.volume,
                |it| dispatch(Action::SetFloat(FloatField::Volume, it)),
                0.0..=1.0,
            )
            .with_neutral(1.0);
            let pan_knob = styled_knob(
                config.pan,
                |it| dispatch(Action::SetFloat(FloatField::Pan, it)),
                -1.0..=1.0,
            )
            .with_neutral(0.0);
            let coarse_det_knob = styled_knob(
                config.coarse_detune / 100.0,
                |it| {
                    dispatch(Action::SetFloat(
                        FloatField::OscillatorCoarseDetune,
                        it * 100.0,
                    ))
                },
                -24.0..=24.0,
            )
            .with_neutral(0.0)
            .with_step(1.0);
            let fine_det_knob = styled_knob(
                config.fine_detune,
                |it| dispatch(Action::SetFloat(FloatField::OscillatorFineDetune, it)),
                -100.0..=100.0,
            )
            .with_neutral(0.0);

            inner_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    add_typable_knob(
                        ui,
                        volume_knob,
                        "Volume",
                        config.volume,
                        |it| dispatch(Action::SetFloat(FloatField::Volume, it)),
                        0.0..=1.0,
                        on_release,
                    );
                    ui.add_space(KNOB_SPACE);

                    add_typable_knob(
                        ui,
                        pan_knob,
                        "Pan",
                        config.pan,
                        |it| dispatch(Action::SetFloat(FloatField::Pan, it)),
                        -1.0..=1.0,
                        on_release,
                    );
                    ui.add_space(KNOB_SPACE);

                    add_typable_knob(
                        ui,
                        coarse_det_knob,
                        "Coarse",
                        config.coarse_detune / 100.0,
                        |it| {
                            dispatch(Action::SetFloat(
                                FloatField::OscillatorCoarseDetune,
                                it * 100.0,
                            ))
                        },
                        -24.0..=24.0,
                        on_release,
                    );
                    ui.add_space(KNOB_SPACE);

                    add_typable_knob(
                        ui,
                        fine_det_knob,
                        "Fine",
                        config.fine_detune,
                        |it| dispatch(Action::SetFloat(FloatField::OscillatorFineDetune, it)),
                        -100.0..=100.0,
                        on_release,
                    );
                });
            });
        }

        fn draw_stacking_controls<F, G>(
            ui: &mut Ui,
            config: &Oscillator,
            dispatch: &F,
            on_release: &G,
        ) where
            F: Fn(Action),
            G: Fn(),
        {
            inner_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label("Stacking");
                    ui.add_space(4.0);
                    int_slider(
                        ui,
                        "Unison",
                        config.osc_count as f64,
                        |it| dispatch(Action::SetUint(UintField::OscCount, it as u32)),
                        1..=24,
                        on_release,
                    );
                    ui.add_space(6.0);
                    let unison_det_knob = styled_knob(
                        config.unison_detune,
                        |it| dispatch(Action::SetFloat(FloatField::UnisonDetune, it)),
                        0.0..=100.0,
                    )
                    .with_neutral(0.0);
                    add_typable_knob(
                        ui,
                        unison_det_knob,
                        "Unison detune",
                        config.unison_detune,
                        |it| dispatch(Action::SetFloat(FloatField::UnisonDetune, it)),
                        0.0..=100.0,
                        on_release,
                    );
                });
            });
        }

        outer_frame()
            .outer_margin(Margin {
                left: 5,
                right: 0,
                top: 10,
                bottom: 0,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    draw_wave_selection(ui, config, dispatch, line_colour, fill_colour);
                    draw_oscillator_controls(ui, config, dispatch, on_release);
                    draw_stacking_controls(ui, config, dispatch, on_release);
                });
            });
    }
}
