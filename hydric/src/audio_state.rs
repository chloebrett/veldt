use crate::playback::AudioPlayer;
use dasp_frame::Stereo;
use mesic::graph::RenderGraph;

pub struct AudioState {
    pub audio: Vec<Stereo<f32>>,
    pub player: AudioPlayer,
}

impl AudioState {
    pub fn new(graph: RenderGraph) -> Self {
        Self {
            audio: vec![],
            player: AudioPlayer::new(graph),
        }
    }
}
