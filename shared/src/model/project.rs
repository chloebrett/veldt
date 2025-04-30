use crate::bytes::{as_bytes, as_floats};
use crate::model::{
    EffectInstance, GeneratorInstance, ModMatrix, Placement, Track, TrackPlacement,
};
use crate::pmodel::*;
use crate::types::Beats;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;

pub type TrackId = u32;
type _SampleId = usize;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Project {
    pub name: String,

    #[proto_repeated]
    pub tracks: Vec<Track>,

    /// Ordered based on start_position.
    #[proto_repeated]
    pub placements: Vec<Placement>,

    #[proto_repeated]
    pub samples: Vec<Sample>,

    #[proto_repeated]
    pub generators: Vec<GeneratorInstance>,

    #[proto_repeated]
    pub mixer: Vec<MixerChannel>,

    pub bpm: Beats,

    #[proto_optional]
    pub mod_matrix: ModMatrix,
}

impl Project {
    pub fn duration(&self) -> OrderedFloat<f32> {
        let mut max = OrderedFloat(0.0);
        for placement in &self.placements {
            if let &Ok(&TrackPlacement { track_index, .. }) = &placement.try_into() {
                let track = &self.tracks[track_index];
                let offset = &placement.offset;
                let duration = placement
                    .clipped_duration
                    .unwrap_or(track.unclipped_duration());
                max = std::cmp::max(max, offset + duration);
            }
        }
        max
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub sample_rate: f32,
}

impl From<SampleProto> for Sample {
    fn from(item: SampleProto) -> Self {
        Self {
            left: as_floats(&item.left),
            right: as_floats(&item.right),
            sample_rate: item.sample_rate,
        }
    }
}

impl From<Sample> for SampleProto {
    fn from(item: Sample) -> Self {
        Self {
            left: as_bytes(&item.left),
            right: as_bytes(&item.right),
            sample_rate: item.sample_rate,
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct MixerChannel {
    #[proto_repeated]
    pub effects: Vec<EffectInstance>,
}

#[cfg(test)]
mod tests {
    use crate::{
        model::{
            AdsrEnvelope, AntiAliasingMode, DelayConfig, Effect, EffectMeta, EqConfig, EqType,
            GeneratorMeta, GeneratorType, ModDelayConfig, Note, PitchName, PlacedNote,
            PlacementType, ScaleValue, SimpleWaveConfig, WaveType,
        },
        testing::proto::proto_testing::assert_proto_round_trip,
    };
    use ordered_float::OrderedFloat;

    use super::*;

    #[test]
    fn project_proto_round_trip() {
        // Project must be fully populated for sufficient testing.
        let project = Project {
            name: "My Project".to_string(),
            tracks: vec![Track {
                notes: vec![PlacedNote {
                    note: Note {
                        pitch_name: PitchName {
                            scale_value: ScaleValue::A,
                            octave: 4,
                        },
                        beats: 1.0,
                    },
                    offset: OrderedFloat(0.0),
                }],
                offset: OrderedFloat(0.0),
            }],
            placements: vec![Placement {
                kind: PlacementType::Track(TrackPlacement {
                    track_index: 3,
                    generator_index: 3,
                }),
                offset: 2.5.into(),
                clipped_duration: Some(5.2.into()),
                visual_placement: 6,
            }],
            samples: vec![Sample {
                left: vec![0.0, 1.0, 3.0],
                right: vec![0.0, 1.0, 3.0],
                sample_rate: 1.0,
            }],
            generators: vec![GeneratorInstance {
                it: GeneratorType::SimpleWave(SimpleWaveConfig {
                    wave: WaveType::Sine,
                    envelope: AdsrEnvelope {
                        attack: 0.1,
                        decay: 0.1,
                        sustain: 0.8,
                        release: 0.1,
                    },
                    osc_count: 4,
                    detune_cents: 5.0,
                    anti_aliasing_mode: AntiAliasingMode::Off,
                    oversample_factor: 2,
                }),
                meta: GeneratorMeta {
                    volume: 1.0,
                    mute: false,
                    pan: 0.0,
                },
            }],
            mixer: vec![MixerChannel {
                effects: vec![
                    EffectInstance {
                        effect: Effect::SimpleEq {
                            config: EqConfig {
                                kind: EqType::SimpleResonator,
                                fc: 1000.0,
                                q: 1.0,
                                gain: 0.0,
                            },
                        },
                        meta: EffectMeta {
                            id: 0,
                            wet: 1.0,
                            mute: false,
                        },
                    },
                    EffectInstance {
                        effect: Effect::SimpleDelay {
                            config: DelayConfig {
                                delay_ms: 250.0,
                                feedback: 0.5,
                            },
                        },
                        meta: EffectMeta {
                            id: 1,
                            wet: 0.5,
                            mute: false,
                        },
                    },
                    EffectInstance {
                        effect: Effect::ModDelay {
                            config: ModDelayConfig {
                                min_depth: 100,
                                max_depth: 200,
                                freq: 10.0,
                                lfo_type: WaveType::Triangle,
                            },
                        },
                        meta: EffectMeta {
                            id: 1,
                            wet: 0.5,
                            mute: false,
                        },
                    },
                ],
            }],
            bpm: 120.0,
            mod_matrix: ModMatrix::default(),
        };
        assert_proto_round_trip::<Project, ProjectProto>(project);
    }
}
