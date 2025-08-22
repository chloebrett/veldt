use super::SimpleWaveVisualiser;
use crate::view::View;
use crate::widget::{
    TabDisplay, TabOrientation, get_set, inner_frame, outer_frame, selectable_value, slider,
};
use crate::{GetSet, LocalState};
use egui::{Color32, Ui, Vec2, Margin};
use shared::model::{StingrayConfig, WaveType};
use state::{Action, FloatField, TypeField};
use strum::IntoEnumIterator;

pub struct StingrayLfoView<'a, F: Fn(Action), G: Fn()> {
    config: &'a StingrayConfig,
    dispatch: F,
    on_release: G,
    local_state: &'a LocalState,
}

impl<'a, F: Fn(Action), G: Fn()> StingrayLfoView<'a, F, G> {
    pub fn new(
        config: &'a StingrayConfig,
        dispatch: F,
        on_release: G,
        local_state: &'a LocalState,
    ) -> Self {
        Self {
            config,
            dispatch,
            on_release,
            local_state,
        }
    }
}

const LFO_LINE_COLOUR: Color32 = Color32::from_rgb(166, 47, 250);
const LFO_FILL_COLOUR: Color32 = Color32::from_rgba_premultiplied(166, 47, 250, 40);

impl<F: Fn(Action), G: Fn()> View for StingrayLfoView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let config = self.config.clone();
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        let active_lfo_tab = self.local_state.stingray_lfo_tab.get();
        let handle_lfo_tab_click = |index| {
            self.local_state.stingray_lfo_tab.set(index);
        };

        outer_frame()
            .outer_margin(Margin {
                left: 0,
                right: 2,
                top: 5,
                bottom: 5,
            })
        .show(ui, |ui| {
            let original_spacing = ui.spacing().item_spacing; // store original spacing
            ui.spacing_mut().item_spacing = Vec2::ZERO; // set spacing to zero so that the tabs and associated content actually touch each other

            ui.horizontal(|ui| {
                ui.add_space(2.0);
                TabDisplay::new(
                    active_lfo_tab,
                    vec!["LFO 1", "LFO 2", "LFO 3"],
                    TabOrientation::Left,
                    handle_lfo_tab_click,
                )
                .ui(ui);

                inner_frame()
                    .inner_margin(Margin::same(20))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let current_lfo_config = &config.lfos[active_lfo_tab];

                        egui::ComboBox::from_label("")
                            .selected_text(current_lfo_config.wave.to_string())
                            .show_ui(ui, |ui| {
                                for wave in WaveType::iter() {
                                    selectable_value(
                                        ui,
                                        get_set(current_lfo_config.wave, |wave_type| {
                                            dispatch(Action::SetChild(TypeField::Wave(wave_type)))
                                        }),
                                        wave,
                                        wave.to_string(),
                                    );
                                }
                            });

                        ui.add_space(20.0);

                        SimpleWaveVisualiser::new(
                            current_lfo_config.wave,
                            LFO_LINE_COLOUR,
                            LFO_FILL_COLOUR,
                            current_lfo_config.frequency,
                            Vec2::new(340.0, 140.0),
                        )
                        .show(ui);

                        ui.add_space(4.0);

                        ui.spacing_mut().item_spacing = original_spacing; // reset ui spacing back to original

                        slider(
                            ui,
                            "Frequency",
                            current_lfo_config.frequency as f64,
                            |it| dispatch(Action::SetFloat(FloatField::LfoFreq, it as f32)),
                            1.0..=10.0,
                            on_release,
                        );
                    })
                })
            });
        });
    }
}
