use crate::model::{GeneratorInstance, Mixer, Placement, Sample, Track, TrackPlacement};
use crate::pmodel::*;
use crate::types::Beats;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto, Default)]
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

    #[proto_optional]
    pub mixer: Mixer,

    pub bpm: Beats,
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

#[cfg(test)]
mod tests {
    use crate::{
        model::{
            AdsrEnvelope, AntiAliasingMode, DelayConfig, Effect, EffectInstance, EffectMeta,
            EqConfig, EqType, Generator, GeneratorMeta, MixerChannel, MixerMatrix, ModDelayConfig,
            Note, PitchName, PlacedNote, PlacementType, PolyphonyMode, ScaleValue,
            SimpleWaveConfig, WaveType,
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
                sample_name: "default".to_string(),
            }],
            generators: vec![GeneratorInstance {
                it: Generator::SimpleWave(SimpleWaveConfig {
                    wave: WaveType::Sine,
                    envelope: AdsrEnvelope {
                        attack: 0.1,
                        decay: 0.1,
                        sustain: 0.8,
                        release: 0.1,
                    },
                    osc_count: 4,
                    detune_cents: 5.0,
                    anti_aliasing_mode: AntiAliasingMode::Additive,
                    oversample_factor: 2,
                    polyphony_mode: PolyphonyMode::Polyphonic,
                    polyphony_limit: 0,
                }),
                meta: GeneratorMeta {
                    volume: 1.0,
                    mute: false,
                    pan: 0.0,
                    mixer_channel: 0,
                },
            }],
            mixer: Mixer {
                matrix: MixerMatrix::with_channels(3),
                channels: vec![MixerChannel {
                    volume: 1.0,
                    effects: vec![
                        EffectInstance {
                            it: Effect::SimpleEq(EqConfig {
                                kind: EqType::SimpleResonator,
                                fc: 1000.0,
                                q: 1.0,
                                gain: 0.0,
                            }),
                            meta: EffectMeta {
                                wet: 1.0,
                                mute: false,
                            },
                        },
                        EffectInstance {
                            it: Effect::Delay(DelayConfig {
                                delay_ms: 250.0,
                                feedback: 0.5,
                            }),
                            meta: EffectMeta {
                                wet: 0.5,
                                mute: false,
                            },
                        },
                        EffectInstance {
                            it: Effect::ModDelay(ModDelayConfig {
                                min_depth: 100,
                                max_depth: 200,
                                freq: 10.0,
                                lfo_type: WaveType::Triangle,
                            }),
                            meta: EffectMeta {
                                wet: 0.5,
                                mute: false,
                            },
                        },
                    ],
                }],
            },
            bpm: 120.0,
        };
        assert_proto_round_trip::<Project, ProjectProto>(project);
    }
}
