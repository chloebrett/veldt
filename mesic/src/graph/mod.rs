use dasp_frame::Stereo;
use dasp_graph::{BoxedNodeSend, NodeData};
use petgraph::stable_graph::StableGraph;
use state::StoreData;

mod note_tracker;
mod render_graph;

pub use note_tracker::*;
pub use render_graph::*;

#[derive(PartialEq)]
pub enum PlaybackMode {
    // Loads notes from tracks.
    Main,

    // Plays the preview buffer.
    // Switches back to Main and pauses once done.
    Preview,
}

pub struct ProcessContext {
    // TODO: don't keep a whole store here.
    // Have Project handle action receiving itself,
    // and then just store a project.
    // Then, StoreData doesn't need to be Clone anymore.
    pub store: StoreData,
    pub main_seek_pos: Option<usize>,
    pub preview_seek_pos: Option<usize>,
    pub playback_mode: PlaybackMode,
    pub note_events: NoteEventsByGenerator,

    // A buffer to play starting at sample 0.
    // Used for playing server-rendered audio, previewing samples, etc.
    pub preview_buffer: Vec<Stereo<f32>>,
}

impl ProcessContext {
    pub fn new(store: StoreData) -> Self {
        Self {
            store,
            main_seek_pos: None,
            preview_seek_pos: None,
            playback_mode: PlaybackMode::Main,
            note_events: vec![],
            preview_buffer: vec![],
        }
    }
}

pub type Graph = StableGraph<NodeData<BoxedNodeSend<ProcessContext>>, ()>;

pub type Processor = dasp_graph::Processor<Graph>;

// If these are exceeded then the graph will dynamically allocate.
const MAX_NODES: usize = 1024;
const MAX_EDGES: usize = 1024;

pub fn make_graph() -> Graph {
    Graph::with_capacity(MAX_NODES, MAX_EDGES)
}

pub fn make_processor() -> Processor {
    Processor::with_capacity(MAX_NODES)
}
