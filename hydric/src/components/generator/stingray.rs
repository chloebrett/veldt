use super::{StingrayEnvelopeView, StingrayLfoView, StingrayLpfView, StingrayOscillatorView};
use crate::components::opacity_percentage_to_alpha;
use crate::components::{ModMatrixView, Piano, PianoOrientation};
use crate::playback::AudioPlayer;
use crate::view::View;
use crate::{GetSet, LocalState};
use egui::{Color32, Ui, Vec2};
use lazy_static::lazy_static;
use shared::{
    model::{PitchName, ScaleValue, StingrayConfig},
    types::PitchValue,
};
use state::{GeneratorSelector, Store};

pub struct StingrayView<'a, G: Fn()> {
    config: &'a StingrayConfig,
    on_release: G,
    store: &'a Store,
    local_state: &'a LocalState,
    generator_sel: &'a GeneratorSelector,
    audio_player: &'a mut AudioPlayer,
}

impl<'a, G: Fn()> StingrayView<'a, G> {
    pub fn new(
        config: &'a StingrayConfig,
        on_release: G,
        store: &'a Store,
        local_state: &'a LocalState,
        generator_sel: &'a GeneratorSelector,
        audio_player: &'a mut AudioPlayer,
    ) -> Self {
        Self {
            config,
            on_release,
            store,
            local_state,
            generator_sel,
            audio_player,
        }
    }

    fn draw_piano(&mut self, ui: &mut Ui) {
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
            Vec2::new(1330.0, 100.0),
            Some(self.audio_player),
            Some(*self.generator_sel),
        )
        .ui(ui);
    }
}

const CHART_FILL_ALPHA: u8 = opacity_percentage_to_alpha(44.0);
const HORIZONTAL_SPACE: f32 = 3.0;

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

impl<G: Fn()> View for StingrayView<'_, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let config = self.config;
        let on_release = &self.on_release;
        let gen_sel = self.generator_sel;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for oscillator_id in 0..3 {
                    let osc_sel = gen_sel.downcast_oscillator(oscillator_id);
                    let osc_dispatch = |action| self.store.dispatch(&osc_sel, action);
                    ui.push_id(oscillator_id, |ui| {
                        StingrayOscillatorView::new(
                            &config.oscillators[oscillator_id],
                            &osc_dispatch,
                            &on_release,
                            LINE_COLOURS[oscillator_id],
                            FILL_COLOURS[oscillator_id],
                        )
                        .ui(ui);
                    });
                }
            });

            ui.add_space(HORIZONTAL_SPACE); // space btwn oscillator and env + lfo

            ui.vertical(|ui| {
                let current_env_index = self.local_state.stingray_env_tab.get();
                let env_sel = gen_sel.downcast_envelope(current_env_index);
                let env_dispatch = |action| self.store.dispatch(&env_sel, action);

                StingrayEnvelopeView::new(config, env_dispatch, on_release, self.local_state)
                    .ui(ui);

                let current_lfo_index = self.local_state.stingray_lfo_tab.get();
                let lfo_sel = gen_sel.downcast_lfo(current_lfo_index);
                let lfo_dispatch = |action| self.store.dispatch(&lfo_sel, action);

                StingrayLfoView::new(config, lfo_dispatch, on_release, self.local_state).ui(ui);
            });

            ui.vertical(|ui| {
                ui.add_space(10.0); //adds spacing between osc, env, mod, and the top border
                ModMatrixView::new(
                    &config.matrix,
                    vec!["ENV 1", "ENV 2", "ENV 3", "LFO 1", "LFO 2", "LFO 3"],
                    vec!["OSC 1", "OSC 2", "OSC 3", "LPF"],
                    self.store,
                    gen_sel,
                    on_release,
                )
                .ui(ui); // must wrap in ui.vertical to stop the matrix from unnecessarily stretching vertically

                let lpf_index = 0;
                let lpf_sel = gen_sel.downcast_effect(lpf_index);
                let lpf_dispatch = |action| self.store.dispatch(&lpf_sel, action);

                ui.add_space(10.0); //space btwn mod and lpf
                StingrayLpfView::new(&config.lpf, lpf_dispatch, on_release).ui(ui);
            });
            ui.add_space(1.0); //spacing btwn osc + lpf and right border
        });

        ui.add_space(10.0); // spacing btwn elements and piano roll

        self.draw_piano(ui);
    }
}
