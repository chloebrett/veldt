use dasp_graph::{BoxedNodeSend, NodeData};
use petgraph::stable_graph::StableGraph;
use state::StoreData;

mod render_graph;

pub use render_graph::*;

#[derive(Default)]
pub struct ProcessContext {
    pub store: StoreData,
    pub seek_pos: Option<usize>,
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
