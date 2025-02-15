use strum::{Display, EnumString};

use shared::model::note::Note;
use shared::model::sequence::Sequence;

#[derive(EnumString, Display)]
pub enum DemoOption {
    Demo1,
    Demo2,
}

pub struct Demo {
    pub sequences: Vec<Sequence>,
    pub bpm: f32,
}

impl Demo {
    pub fn new(demo_option: DemoOption) -> Self {
        match demo_option {
            DemoOption::Demo1 => { 
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
                    bpm:160.
                }
            }
            DemoOption::Demo2 => {
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
                    bpm: 120.
                }
            } 
        }
    }
}
