use super::extract_outputs;
use crate::graph::{NoteEventType, ProcessContext};
use dasp_graph::{Buffer, Input, Node};
use ordered_float::OrderedFloat;
use shared::model::Sample;
use shared::model::{DrumTrackPlacement, SampleId};
use state::{PlacementSelector, SampleSelector};
use std::{cmp::max, collections::HashMap};

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    hit_start: usize,
    sample: SampleId,
    pitch_offset: f32,
}

/// Node that plays a drum track.
pub struct DrumTrackPlacementNode {
    selector: PlacementSelector,
    hits: Vec<Hit>, // stores start index of pending hits
    sample_cache: HashMap<(SampleId, OrderedFloat<f32>), Sample>,
}

impl DrumTrackPlacementNode {
    pub fn new(sel: PlacementSelector) -> Self {
        Self {
            selector: sel,
            hits: Vec::new(),
            sample_cache: HashMap::new(),
        }
    }
}

// 60 is the MIDI value of C4 and is used below. The current code assumes that all drum samples are
// C4 by default. In future can possibly investigate calculating original pitch of sample to use instead
// or allowing user to adjust the root key.
const C4_MIDI: f32 = 60.0;

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

        let placement_id = self.selector.0;
        if let Some(events) = payload.drum_note_events.get(&placement_id) {
            for note_event in events {
                if note_event.kind == NoteEventType::On {
                    let start_index = payload.playback_pos + note_event.sample_index;
                    let pitch_value: i32 = note_event.pitch_name.into();
                    if let Some(sample_id) = drum_track_placement.pitch_sample_map.get(&pitch_value)
                    {
                        self.hits.push(Hit {
                            hit_start: start_index,
                            sample: *sample_id,
                            pitch_offset: note_event.pitch_offset,
                        });
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
                let sample_sel = SampleSelector(hit.sample);
                let Some(orig_sample) = store.try_select(&sample_sel) else {
                    log::error!("Sample doesn't exist: {:?}", hit.sample);
                    return false;
                };
                let mut sample_len = max(orig_sample.left.len(), orig_sample.right.len());
                if !self
                    .sample_cache
                    .contains_key(&(hit.sample, OrderedFloat(hit.pitch_offset)))
                {
                    if hit.pitch_offset != 0.0 {
                        let sample =
                            resample_to_pitch(orig_sample, C4_MIDI, C4_MIDI + hit.pitch_offset);
                        sample_len = max(sample.left.len(), sample.right.len());
                        self.sample_cache
                            .insert((hit.sample, OrderedFloat(hit.pitch_offset)), sample);
                    } else {
                        self.sample_cache.insert(
                            (hit.sample, OrderedFloat(hit.pitch_offset)),
                            orig_sample.clone(),
                        );
                    }
                }

                hit.hit_start + sample_len > playback_pos
            })
            .collect::<Vec<_>>();

        for i in 0..buffer_len {
            let sample_index = playback_pos + i;
            let mut left_acc = 0.0;
            let mut right_acc = 0.0;

            for &hit in &remaining_hits {
                let sample = self
                    .sample_cache
                    .get(&(hit.sample, OrderedFloat(hit.pitch_offset)))
                    .unwrap();
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

pub fn resample_to_pitch(sample: &Sample, from_note: f32, to_note: f32) -> Sample {
    let semitones = to_note - from_note;
    let ratio = semitone_ratio(semitones);

    let resampled_left = resample(&sample.left, sample.left.len(), ratio);
    let resampled_right = resample(&sample.right, sample.right.len(), ratio);

    Sample {
        left: resampled_left,
        right: resampled_right,
        sample_rate: sample.sample_rate,
        sample_name: sample.sample_name.clone(),
    }
}

/// This function takes a sample and resamples it to a new playback rate.
/// Resampling here means we generate a new sample at the new playback rate using linear interpolation.
/// This is how we change the "pitch" of a drum hit or other short audio snippet: playing it faster raises the pitch and playing it
/// slower lowers the pitch. For percussive sounds like drum hits the simplest and most authentic way to change pitch is just to play them faster or slower.
/// More advanced algorithms (phase vocoder, elastique, etc.) can separate pitch from duration but they are heavier on CPU and often unnecessary for short samples.
pub fn resample(sample: &[f32], sample_len: usize, ratio: f32) -> Vec<f32> {
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
    }
    resampled
}

/// Converts a semitone offset to a playback speed ratio. 12.0 semitones (one octave) will
/// double playback speed, -12.0 will halve it.
pub fn semitone_ratio(semitones: f32) -> f32 {
    2f32.powf(semitones / 12.0)
}
