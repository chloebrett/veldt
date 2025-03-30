use dasp_graph::{BoxedNode, NodeData};
use petgraph::stable_graph::StableGraph;

mod node;
mod render_graph;

pub use node::*;
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
