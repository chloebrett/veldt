use crate::playback::AudioPlayer;
use dasp_frame::Stereo;
use mesic::graph::RenderGraph;

pub struct AudioState {
    pub player: AudioPlayer,
}

impl AudioState {
    pub fn new(graph: RenderGraph) -> Self {
        Self {
            player: AudioPlayer::new(graph),
        }
    }
}
