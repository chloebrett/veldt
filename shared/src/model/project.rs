use crate::bytes::{as_bytes, as_floats};
use crate::model::{EffectInstance, GeneratorInstance, Track};
use crate::pmodel::*;
use crate::types::Beats;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

type _TrackId = usize;
type _SampleId = usize;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Project {
    pub name: String,

    #[proto_repeated]
    pub tracks: Vec<Track>,

    /// Ordered based on start_position.
    #[proto_repeated]
    pub track_placements: Vec<TrackPlacement>,

    #[proto_repeated]
    pub samples: Vec<Sample>,

    #[proto_repeated]
    pub generators: Vec<GeneratorInstance>,

    #[proto_repeated]
    pub mixer: Vec<MixerChannel>,

    pub bpm: Beats,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackPlacement {
    track_id: _TrackId,

    /// The time that the track starts within the arrangement.
    start_position: OrderedFloat<Beats>,

    /// If None, then duration is not clipped.
    clipped_duration: Option<OrderedFloat<Beats>>,

    /// Position that the track should be displayed visually, useful if there are overlapping tracks.
    /// Zero is the top.
    visual_placement: u32,
}

impl PartialOrd for TrackPlacement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TrackPlacement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_position.cmp(&other.start_position)
    }
}

impl From<TrackPlacementProto> for TrackPlacement {
    fn from(item: TrackPlacementProto) -> Self {
        TrackPlacement {
            track_id: item.track_id as usize,
            start_position: item.start_position.into(),
            clipped_duration: item.clipped_duration.map(OrderedFloat),
            visual_placement: item.visual_placement,
        }
    }
}

impl From<TrackPlacement> for TrackPlacementProto {
    fn from(item: TrackPlacement) -> Self {
        TrackPlacementProto {
            track_id: item.track_id as u32,
            start_position: *item.start_position,
            clipped_duration: item.clipped_duration.map(|it| *it),
            visual_placement: item.visual_placement,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub data: Vec<f32>,
    pub sample_rate: f32,
}

impl From<SampleProto> for Sample {
    fn from(item: SampleProto) -> Self {
        Sample {
            data: as_floats(&item.data),
            sample_rate: item.sample_rate,
        }
    }
}

impl From<Sample> for SampleProto {
    fn from(item: Sample) -> Self {
        SampleProto {
            data: as_bytes(&item.data),
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
            AdsrEnvelope, DelayConfig, Effect, EffectMeta, EqConfig, EqType, GeneratorMeta,
            GeneratorType, Note, PitchName, PlacedNote, ScaleValue, SimpleWaveConfig, WaveType,
        },
        testing::proto::proto_testing::assert_proto_round_trip,
    };

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
            }],
            track_placements: vec![TrackPlacement {
                track_id: 3,
                start_position: 2.5.into(),
                clipped_duration: Some(5.2.into()),
                visual_placement: 6,
            }],
            samples: vec![Sample {
                data: vec![0.0, 1.0, 3.0],
                sample_rate: 1.0,
            }],
            generators: vec![GeneratorInstance {
                id: 0,
                kind: GeneratorType::SimpleWave {
                    config: SimpleWaveConfig {
                        wave: WaveType::Sine,
                        envelope: AdsrEnvelope {
                            attack: 0.1,
                            decay: 0.1,
                            sustain: 0.8,
                            release: 0.1,
                        },
                        osc_count: 4,
                        detune_cents: 5.0,
                    },
                },
                meta: GeneratorMeta { volume: 1.0 },
            }],
            mixer: vec![MixerChannel {
                effects: vec![
                    EffectInstance {
                        effect: Effect::SimpleEq {
                            config: EqConfig {
                                kind: EqType::SimpleResonator,
                                fc: 1000.0,
                                q: 1.0,
                            },
                        },
                        meta: EffectMeta { id: 0, wet: 1.0 },
                    },
                    EffectInstance {
                        effect: Effect::SimpleDelay {
                            config: DelayConfig {
                                amplitude: 0.5,
                                delay_ms: 250.0,
                            },
                        },
                        meta: EffectMeta { id: 1, wet: 0.5 },
                    },
                ],
            }],
            bpm: 120.0,
        };
        assert_proto_round_trip::<Project, ProjectProto>(project);
    }
}
