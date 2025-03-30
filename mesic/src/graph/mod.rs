use dasp_graph::{BoxedNode, NodeData};
use petgraph::stable_graph::StableGraph;

mod amp_node;
mod buffer_node;
mod compressor_node;
mod eq_node;
mod mixer_node;
mod render_graph;

pub use amp_node::*;
pub use buffer_node::*;
pub use compressor_node::*;
pub use eq_node::*;
pub use mixer_node::*;
pub use render_graph::*;

pub type Graph = StableGraph<NodeData<BoxedNode>, ()>;

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
