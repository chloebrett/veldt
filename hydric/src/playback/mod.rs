mod audio_player;
mod audio_processor;

pub use audio_player::*;
use audio_processor::*;

use dasp_frame::Stereo;
use shared::model::PitchName;
use state::GeneratorSelector;

// Number of samples to process and send to the audio player at a time.
// This matches the value configured in CPAL.
const BUFFER_SIZE: usize = 2048;

type AudioBuffer = [Stereo<f32>; BUFFER_SIZE];

const EMPTY_BUFFER: AudioBuffer = [[0.0; 2]; BUFFER_SIZE];

// For now, just samples. In future, consider supporting bars:beats, mins:secs, etc.
#[derive(Clone, Copy)]
pub struct PlaybackPosition {
    pub samples: usize,
}

// Messages that can be sent to the processor thread.
enum PlaybackMessage {
    // Recreates the audio mixer.
    // Useful for debugging and for playback where the graph isn't perfectly dynamic (which it
    // currently isn't, e.g. some effects don't update live).
    RecreateMixer,
    // Sets some pre-rendered audio to be played by the graph.
    SetAudio(Vec<Stereo<f32>>),
    Seek(PlaybackPosition),
    State(PlaybackState),
    Loop(bool),
    NoteOn(GeneratorSelector, PitchName),
    NoteOff(GeneratorSelector, PitchName),
}

// Messages that can be received from the processor thread.
enum PlaybackUpdate {
    // Playback position changed.
    Pos(PlaybackPosition),

    // Latest playback buffer delay amount, in samples.
    Delay(usize),

    // Playback state changed.
    State(PlaybackState),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PlaybackState {
    Play,
    Pause,
    Finished,
}
