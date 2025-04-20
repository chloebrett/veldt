 use crate::widget::{Sequencer, get_set, int_slider, knob, selectable_value};
use eframe::egui;
use egui::{Ui, Color32, Rect, pos2};
use shared::model::{SubSynthConfig, WaveType};
use state::{Action, Store, TypeField};
use super::{SubsynthOscillator, OscillatorId};
use super::super::{Piano, PianoOrientation};
use crate::{
    view::View,
};
use shared::{
    model::{PitchName, ScaleValue},
    types::PitchValue,
};

pub fn subsynth_control<F, G>(config: &SubSynthConfig, dispatch: F, on_release: G, ui: &mut Ui, store: &Store)
where
    F: Fn(Action),
    G: Fn(),
{
    // Create a struct to track oscillator wave types
    #[derive(Clone)]
    struct OscillatorState {
        wave_types: [WaveType; 3]
    }
    
    // Get persisted state or create new
    let id = ui.id().with("subsynth_oscillators");
    let mut state = ui.ctx().data_mut(|data| 
        data.get_temp::<OscillatorState>(id)
            .unwrap_or(OscillatorState {
                wave_types: [
                    config.oscillators[0].wave,
                    config.oscillators[1].wave,
                    config.oscillators[2].wave,
                ]
            })
    );
    
    // OSC 1
    ui.push_id(OscillatorId::Osc1, |ui| {
        let mut osc1 = SubsynthOscillator::new(
            OscillatorId::Osc1,
            state.wave_types[0],
            Color32::from_rgb(119, 167, 43),
            Color32::from_rgba_unmultiplied(147, 175, 100, (0.44 * 255.0) as u8),
        );
        osc1.show(ui, &dispatch, &on_release);
        
        // Update state and trigger action if changed
        let new_wave_type = osc1.wave_type();
        if state.wave_types[0] != new_wave_type {
            state.wave_types[0] = new_wave_type;
            dispatch(Action::SetChild(TypeField::Wave(new_wave_type)));
            on_release();
        }
    });

    ui.add_space(8.0);

    // OSC 2
    ui.push_id(OscillatorId::Osc2, |ui| {
        let mut osc2 = SubsynthOscillator::new(
            OscillatorId::Osc2,
            state.wave_types[1],
            Color32::from_rgb(246, 83, 192),
            Color32::from_rgba_unmultiplied(212, 132, 170, (0.44 * 255.0) as u8),
        );
        osc2.show(ui, &dispatch, &on_release);
        
        // Update state and trigger action
        let new_wave_type = osc2.wave_type();
        if state.wave_types[1] != new_wave_type {
            state.wave_types[1] = new_wave_type;
            dispatch(Action::SetChild(TypeField::Wave(new_wave_type)));
            on_release();
        }
    });
    
    ui.add_space(8.0);

    // OSC 3
    ui.push_id(OscillatorId::Osc3, |ui| {
        let mut osc3 = SubsynthOscillator::new(
            OscillatorId::Osc3,
            state.wave_types[2],
            Color32::from_rgb(227, 172, 84),
            Color32::from_rgba_unmultiplied(215, 171, 53, (0.44 * 255.0) as u8),
        );
        osc3.show(ui, &dispatch, &on_release);
        
        // Update state and trigger action if changed
        let new_wave_type = osc3.wave_type();
        if state.wave_types[2] != new_wave_type {
            state.wave_types[2] = new_wave_type;
            dispatch(Action::SetChild(TypeField::Wave(new_wave_type)));
            on_release();
        }
    });

    let min_note: PitchValue = PitchName {
        scale_value: ScaleValue::A,
        octave: 1,
    }.into();
    let max_note: PitchValue = PitchName {
        scale_value: ScaleValue::C,
        octave: 8,
    }.into();
    Piano::new(max_note, min_note - 1).with_orientation(PianoOrientation::Horizontal).ui(ui);

    // // Store state for next frame
    ui.ctx().data_mut(|data| data.insert_temp(id, state));
}