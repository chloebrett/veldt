mod audio_player;
mod audio_processor;

pub use audio_player::*;
use audio_processor::*;

use dasp_frame::Stereo;
use shared::model::Project;

// Total size of the audio buffer.
const BUFFER_SIZE: usize = 5000;

// Number of samples to render at a time.
const CHUNK_SIZE: usize = 1000;

// For now, just samples. In future, consider supporting bars:beats, mins:secs, etc.
#[derive(Clone, Copy)]
pub struct PlaybackPosition {
    pub samples: usize,
}

// Messages that can be sent to the processor thread.
enum PlaybackMessage {
    SetProject(Box<Project>), // uses Box to keep enum size sane.
    SetAudio(Vec<Stereo<f32>>),
    Seek(PlaybackPosition),
    State(PlaybackState),
    Loop(bool),
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
