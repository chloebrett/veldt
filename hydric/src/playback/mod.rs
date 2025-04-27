mod audio_player;
mod audio_processor;

pub use audio_player::*;
use audio_processor::*;

use dasp_frame::Stereo;
use shared::model::Project;
use shared::types::Volume;

// Total size of the audio buffer.
const BUFFER_SIZE: usize = 5000;

// Number of samples to render at a time.
const CHUNK_SIZE: usize = 1000;

// Don't start playing until this many samples have been produced.
const BUFFER_THRESHOLD: usize = 1000;

// For now, just samples. In future, consider supporting bars:beats, mins:secs, etc.
#[derive(Clone, Copy)]
pub struct PlaybackPosition {
    pub samples: usize,
}

// Messages that can be sent to the processor thread.
enum PlaybackMessage {
    SetProject(Box<Project>, Volume), // uses Box to keep enum size sane.
    SetAudio(Vec<Stereo<f32>>, Volume),
    Seek(PlaybackPosition),
    State(PlaybackState),
}

// Messages that can be received from the processor thread.
enum PlaybackUpdate {
    // Playback position changed.
    Pos(PlaybackPosition),

    // Playback state changed.
    State(PlaybackState),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PlaybackState {
    Play,
    Pause,
    Stop,
}
