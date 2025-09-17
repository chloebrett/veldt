use super::extract_outputs;
use crate::graph::{NoteEventType, ProcessContext};
use dasp_graph::{Buffer, Input, Node};
use shared::{model::{DrumTrackPlacement, SampleId}, types::PitchValue};
use state::{PlacementSelector, SampleSelector};
use std::{cmp::max, collections::HashMap};
use shared::model::Sample;

#[derive(Clone, Copy)]
pub struct Hit {
    hit_start: usize,
    pitch: PitchValue
}

/// Node that plays a drum track.
pub struct DrumTrackPlacementNode {
    selector: PlacementSelector,
    hits: Vec<Hit>, // stores start index of pending hits
    resample_cache: HashMap<PitchValue, Sample>,
    current_sample: Option<SampleId>
}

impl DrumTrackPlacementNode {
    pub fn new(sel: PlacementSelector) -> Self {
        Self {
            selector: sel,
            hits: Vec::new(),
            resample_cache: HashMap::new(),
            current_sample: None
        }
    }
}

impl Node<ProcessContext> for DrumTrackPlacementNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let (out_left, out_right) = extract_outputs(output);
        out_left.fill(0.0);
        out_right.fill(0.0);

        let store = &payload.store;
        let Some(placement) = store.try_select(&self.selector) else {
            log::error!("Couldn't find placement: {:?}", self.selector);
            return;
        };

        let Some(drum_track_placement): &Option<&DrumTrackPlacement> = &placement.try_into().ok()
        else {
            log::error!(
                "Placement wasn't a drum track placement: {:?}",
                self.selector
            );
            return;
        };

        let sample_sel = SampleSelector(drum_track_placement.sample_id);
        let Some(sample) = store.try_select(&sample_sel) else {
            log::error!("Sample doesn't exist: {:?}", drum_track_placement.sample_id);
            return;
        };

        if let Some(current_sample) = self.current_sample {
            if current_sample != drum_track_placement.sample_id {
                self.current_sample = Some(drum_track_placement.sample_id);
                self.resample_cache.clear(); // if the sample for the placement changes then resampled samples need to be reprocessed
            }
        } else {
            self.current_sample = Some(drum_track_placement.sample_id);
        }

        let placement_id = self.selector.0;
        if let Some(events) = payload.drum_note_events.get(&placement_id) {
            for note_event in events {
                if note_event.kind == NoteEventType::On {
                    let start_index = payload.playback_pos + note_event.sample_index;
                    let pitch_value: i32 = note_event.pitch_name.into();
                    self.hits.push(Hit { hit_start: start_index, pitch: pitch_value });
                    if !self.resample_cache.contains_key(&pitch_value) {
                        self.resample_cache.insert(pitch_value, resample_to_pitch(sample, 60, pitch_value));
                    }
                }
            }
        }

        let playback_pos = payload.playback_pos;
        let buffer_len = out_left.len();

        let remaining_hits = self
            .hits
            .iter()
            .copied()
            .filter(|&hit| {
                // TODO: sometimes there is a cache miss making the below check/recalculation necessary. Investigate.
                if !self.resample_cache.contains_key(&hit.pitch) {
                    self.resample_cache.insert(hit.pitch, resample_to_pitch(sample, 60, hit.pitch));
                }
                let sample = &self.resample_cache[&hit.pitch];
                let sample_len = max(sample.left.len(), sample.right.len());
                hit.hit_start + sample_len > playback_pos as usize
            })
            .collect::<Vec<_>>();

        for i in 0..buffer_len {
            let sample_index: usize = playback_pos as usize + i;
            let mut left_acc = 0.0;
            let mut right_acc = 0.0;

            for &hit in &remaining_hits {
                let sample = &self.resample_cache[&hit.pitch];
                let sample_len = max(sample.left.len(), sample.right.len());
                let hit_end = hit.hit_start + sample_len;
                if sample_index >= hit.hit_start && sample_index < hit_end {
                    let sample_offset = sample_index - hit.hit_start;
                    left_acc += sample.left[sample_offset];
                    right_acc += sample.right[sample_offset];
                }
            }

            out_left[i] = left_acc;
            out_right[i] = right_acc;
        }

        self.hits = remaining_hits;
    }
}

pub fn resample_to_pitch(sample: &Sample, from_note: PitchValue, to_note: PitchValue) -> Sample {
    let semitones = (to_note - from_note) as f32;
    let ratio = semitone_ratio(semitones);

    let resampled_left = resample(&sample.left, sample.left.len(), ratio);
    let resampled_right = resample(&sample.right, sample.right.len(), ratio);

    Sample {left: resampled_left, right: resampled_right, sample_rate: sample.sample_rate, sample_name: sample.sample_name.clone()}
}

/// This function takes an audio sample and resamples it to a new playback rate.
/// Resampling here means we generate a new sample at the new playback rate using linear interpolation.
/// This is how we change the "pitch" of a drum hit or other short audio snippet: playing it faster raises the pitch and playing it
/// slower lowers the pitch. For percussive sounds like drum hits the simplest and most authentic way to change pitch is just to play them faster or slower.
/// More advanced algorithms (phase vocoder, elastique, etc.) can separate pitch from duration but they are heavier on CPU and often unnecessary for short samples.
pub fn resample(sample: &Vec<f32>, sample_len: usize, ratio: f32) -> Vec<f32> {
    let new_len = (sample_len as f32 / ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_index = i as f32 * ratio; // corresponding index in the original sample vec
        let idx = src_index.floor() as usize; // index of nearest position in original sample vec
        let frac = src_index - idx as f32; // offset used for interpolation

        if idx + 1 < sample_len {
            // Linear interpolation: s0 + frac * (s1 - s0)
            // This produces a smooth value between s0 and s1 which are the original sample values that get blended together
            let s0 = sample[idx];
            let s1 = sample[idx + 1];
            resampled.push(s0 + frac * (s1 - s0));
        } else {
            resampled.push(sample[idx]);
        }
    };
    resampled
}

/// Converts a semitone offset to a playback speed ratio. 12.0 semitones (one octave) will
/// double playback speed, -12.0 will halve it.
pub fn semitone_ratio(semitones: f32) -> f32 {
    2f32.powf(semitones / 12.0)
}

