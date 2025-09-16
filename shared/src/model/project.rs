use crate::model::{
    EffectId, EffectInstance, GeneratorId, GeneratorInstance, Mixer, Placement, PlacementId,
    PlacementType, Sample, SampleId, Track, TrackId, samples_to_beats,
};
use crate::pmodel::*;
use crate::types::Beats;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto, Default)]
pub struct Project {
    pub name: String,

    #[proto_hashmap]
    pub tracks: HashMap<TrackId, Track>,

    #[proto_hashmap]
    pub placements: HashMap<PlacementId, Placement>,

    #[proto_hashmap]
    pub samples: HashMap<SampleId, Sample>,

    #[proto_hashmap]
    pub generators: HashMap<GeneratorId, GeneratorInstance>,

    #[proto_hashmap]
    pub effects: HashMap<EffectId, EffectInstance>,

    #[proto_optional]
    pub mixer: Mixer,

    pub bpm: Beats,
}

impl Project {
    pub fn duration(&self) -> OrderedFloat<f32> {
        let mut max = OrderedFloat(0.0);
        for placement in self.placements.values() {
            let offset = &placement.offset;
            let duration = match &placement.kind {
                PlacementType::Track(track_placement) => {
                    let track = &self.tracks[&track_placement.track_id];
                    placement
                        .clipped_duration
                        .unwrap_or(track.unclipped_duration())
                }
                PlacementType::Sample(sample_placement) => {
                    if let Some(sample) = self.samples.get(&sample_placement.sample_id) {
                        placement
                            .clipped_duration
                            .unwrap_or(OrderedFloat(samples_to_beats(
                                std::cmp::max(sample.left.len(), sample.right.len()),
                                self.bpm,
                            )))
                    } else {
                        OrderedFloat(0.0)
                    }
                }
                PlacementType::DrumTrack(drum_track_placement) => {
                    drum_track_placement.duration(self)
                }
            };
            max = std::cmp::max(max, offset + duration);
        }
        max
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        model::{
            AdsrEnvelope, AntiAliasingMode, Colour, DelayConfig, Effect, EffectInstance,
            EffectMeta, EqConfig, EqType, Generator, GeneratorMeta, MixerChannel, MixerMatrix,
            ModDelayConfig, Note, PitchName, PlacedNote, PlacementType, PolyphonyMode, ScaleValue,
            SimpleWaveConfig, TrackPlacement, WaveType,
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
            tracks: HashMap::from([(
                TrackId(0),
                Track {
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
                },
            )]),
            placements: HashMap::from([(
                PlacementId(0),
                Placement {
                    kind: PlacementType::Track(TrackPlacement {
                        track_id: 3.into(),
                        generator_id: 3.into(),
                    }),
                    offset: 2.5.into(),
                    clipped_duration: Some(5.2.into()),
                    visual_placement: 6,
                    colour: Colour::from_8bit(67, 206, 222),
                },
            )]),
            samples: HashMap::from([(
                SampleId(0),
                Sample {
                    left: vec![0.0, 1.0, 3.0],
                    right: vec![0.0, 1.0, 3.0],
                    sample_rate: 1.0,
                    sample_name: "default".to_string(),
                },
            )]),
            generators: HashMap::from([(
                GeneratorId(0),
                GeneratorInstance {
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
                        name: "".to_string(),
                    },
                },
            )]),
            effects: HashMap::from([
                (
                    EffectId(0),
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
                ),
                (
                    EffectId(1),
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
                ),
                (
                    EffectId(2),
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
                ),
            ]),
            mixer: Mixer {
                matrix: MixerMatrix::with_channels(3),
                channels: vec![MixerChannel {
                    volume: 1.0,
                    mute: false,
                    effect_ids: vec![0.into(), 1.into(), 2.into()],
                }],
            },
            bpm: 120.0,
        };
        assert_proto_round_trip::<Project, ProjectProto>(project);
    }
}
