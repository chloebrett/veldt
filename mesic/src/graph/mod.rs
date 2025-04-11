use dasp_graph::{BoxedNodeSend, Buffer, Input, NodeData};
use petgraph::stable_graph::StableGraph;

mod amp_node;
mod buffer_node;
mod compressor_node;
mod delay_node;
mod eq_node;
mod generator_node;
mod mixer_node;
mod mod_delay_node;
mod render_graph;

pub use amp_node::*;
pub use buffer_node::*;
use compressor_node::*;
use delay_node::*;
use eq_node::*;
pub use generator_node::*;
pub use mixer_node::*;
use mod_delay_node::*;
pub use render_graph::*;

pub type Graph = StableGraph<NodeData<BoxedNodeSend>, ()>;

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

/// Extracts the channels out of an input array and output buffer array.
/// Returns:
/// (left_out, left_in, right_out, right_in)
/// Panics if there aren't enough inputs/outputs.
fn dual_channel<'a>(
    inputs: &'a [Input],
    output: &'a mut [Buffer],
) -> (&'a mut Buffer, &'a Buffer, &'a mut Buffer, &'a Buffer) {
    let mut input = inputs
        .first()
        .expect("Expected one set of input channels")
        .buffers()
        .iter();
    let output = &mut output.iter_mut();

    let left_out = output.next().expect("Expected left output");
    let left_in = input.next().expect("Expected left input");
    let right_out = output.next().expect("Expected right output");
    let right_in = input.next().expect("Expected right input");

    (left_out, left_in, right_out, right_in)
}
