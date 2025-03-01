use shared::model::note::Note;
use shared::model::sequence::Sequence;

use crate::DemoOption;

pub struct Demo {
    // Required as waveform is currently seperately configurable::.
    // Replace in future.
    pub sequences: Vec<Sequence>,
}

impl Demo {
    pub fn new(demo_option: DemoOption) -> Self {
        match demo_option {
            DemoOption::Overworld=> { 
                Self {
                    sequences: vec! [
                        Sequence {
                            offset: 0.,
                            volume: 1.,
                            synth_index: 0,
                            notes: vec![
                                Note(7., 0.5),
                                Note(7., 1.),
                                Note(7., 1.),
                                Note(3., 0.5),
                                Note(7., 1.),
                                Note(10., 2.),
                                Note(-2., 2.),
                            ],
                        },
                        Sequence {
                            offset: 0.,
                                volume: 0.5,
                            synth_index: 0,
                            notes: vec![
                                Note(-19., 0.5),
                                Note(-19., 1.),
                                Note(-19., 1.),
                                Note(-19., 0.5),
                               Note(-19., 1.),
                                Note(-14., 2.),
                                Note(-26., 2.),
                            ],
                        }
                    ],
                }
            }
            DemoOption::FurElise=> {
                Self {
                    sequences: vec! [
                        Sequence {
                            offset: 0.,
                            volume: 1.,
                            synth_index: 0,
                            notes: vec![
                                Note(7., 0.5),
                                Note(6., 0.5),
                                Note(7., 0.5),
                                Note(6., 0.5),
                                Note(7., 0.5),
                                Note(2., 0.5),
                                Note(5., 0.5),
                                Note(3., 0.5),
                                Note(0., 2.0),
                            ],
                        },
                        Sequence {
                            offset: 0.,
                            volume: 0.5,
                            synth_index: 0,
                            notes: vec![
                                Note(-5., 2.0),
                                Note(-10., 2.0),
                                Note(-12., 2.0),
                            ],
                        }
                    ],
                }
            } 
        }
    }
}
