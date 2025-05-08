use super::subsynth_oscillator::SubSynthOscillatorView;
use crate::components::{ModMatrixView, Piano, PianoOrientation};
use crate::view::View;
use crate::widget::{TabDisplay, TabOrientation};
use crate::{GetSet, LocalState};
use eframe::egui;
use egui::{Color32, Frame, Stroke, Ui, Vec2};
use lazy_static::lazy_static;
use shared::model::SubSynthConfig;
use shared::{
    model::{PitchName, ScaleValue},
    types::PitchValue,
};
use state::{GeneratorSelector, Store};

pub struct SubSynthView<'a, G: Fn()> {
    config: &'a SubSynthConfig,
    on_release: G,
    store: &'a Store,
    local_state: &'a LocalState,
    generator_sel: &'a GeneratorSelector,
}

impl<'a, G: Fn()> SubSynthView<'a, G> {
    pub fn new(
        config: &'a SubSynthConfig,
        on_release: G,
        store: &'a Store,
        local_state: &'a LocalState,
        generator_sel: &'a GeneratorSelector,
    ) -> Self {
        Self {
            config,
            on_release,
            store,
            local_state,
            generator_sel,
        }
    }

    fn draw_lfos(&self, ui: &mut Ui) {
        let outer_frame = Frame::new()
            .fill(Color32::from_rgb(50, 50, 50))
            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
            .corner_radius(8.0)
            .inner_margin(6.0);

        outer_frame.show(ui, |ui| {
            let original_spacing = ui.spacing().item_spacing; // store original spacing
            ui.spacing_mut().item_spacing = Vec2::ZERO; // set spacing to zero so that the tabs and associated content actually touch each other

            ui.horizontal(|ui| {
                // TODO move to separate lfo file
                let active_lfo_tab = self.local_state.subsynth_lfo_tab.get();
                let handle_lfo_tab_click = |index| {
                    self.local_state.subsynth_lfo_tab.set(index);
                };

                TabDisplay::new(
                    active_lfo_tab,
                    vec!["LFO 1", "LFO 2", "LFO 3"],
                    TabOrientation::Left,
                    handle_lfo_tab_click,
                )
                .ui(ui);
                let inner_frame = Frame::new()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(30, 30, 30)))
                    .corner_radius(8.0)
                    .inner_margin(15.0);
                inner_frame.show(ui, |ui| {
                    // TODO show the actual LFO controls depending on active_lfo_tab so everything in this block can be deleted it's just a placeholder to visualise the space
                    let size = egui::Vec2::new(200.0, 180.0);
                    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let painter = ui.painter_at(rect);
                    let rect_shape = egui::Shape::rect_filled(rect, 5.0, Color32::RED);
                    painter.add(rect_shape);
                })
            });
            ui.spacing_mut().item_spacing = original_spacing; // reset ui spacing back to original
        });
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
        Piano::new(
            max_note + 1,
            min_note,
            PianoOrientation::Horizontal,
            Vec2::new(1080.0, 50.0),
        )
        .ui(ui);
    }
}

const CHART_FILL_ALPHA: u8 = opacity_percentage_to_alpha(44.0);
const HORIZONTAL_SPACE: f32 = 3.0;

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
    pub static ref LINE_COLOURS: [Color32; 3] = [*GREEN_OUTLINE, *PINK_OUTLINE, *ORANGE_OUTLINE];
    pub static ref FILL_COLOURS: [Color32; 3] = [*GREEN_FILL, *PINK_FILL, *ORANGE_FILL];
}

impl<G: Fn()> View for SubSynthView<'_, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let config = self.config;
        let on_release = &self.on_release;
        let gen_sel = self.generator_sel;
        let gen_dispatch = |action| self.store.dispatch(gen_sel, action); // TODO: fix this for the mod_matrix

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for oscillator_id in 0..3 {
                    let osc_sel = gen_sel.downcast_oscillator(oscillator_id);
                    let osc_dispatch = |action| self.store.dispatch(&osc_sel, action);
                    ui.push_id(oscillator_id, |ui| {
                        SubSynthOscillatorView::new(
                            &config.oscillators[oscillator_id],
                            &osc_dispatch,
                            &on_release,
                            LINE_COLOURS[oscillator_id],
                            FILL_COLOURS[oscillator_id],
                        )
                        .ui(ui);
                        ui.add_space(4.0);
                    });
                }
            });

            ui.add_space(HORIZONTAL_SPACE);

            ui.vertical(|ui| {
                self.draw_lfos(ui);
            });

            ui.add_space(HORIZONTAL_SPACE);

            ui.vertical(|ui| {
                ModMatrixView::new(
                    &config.matrix,
                    vec!["ENV 1", "ENV 2", "ENV 3", "LFO 1", "LFO 2", "LFO 3"],
                    vec!["OSC 1", "OSC 2", "OSC 3"],
                    gen_dispatch, // TODO: need to change this dispatch so that actions for modmatrix work, currently takes GeneratorSelector
                    on_release,
                )
                .ui(ui); // must wrap in ui.vertical to stop the matrix from unnecessarily stretching vertically
            });
        });

        Self::draw_piano(ui);
    }
}
