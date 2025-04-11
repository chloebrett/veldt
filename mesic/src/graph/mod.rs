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

/// Extracts left/right outputs from an outputs slice.
/// Panics if there aren't enough channels.
fn extract_outputs<'a>(output: &'a mut [Buffer]) -> (&'a mut Buffer, &'a mut Buffer) {
    let output = &mut output.iter_mut();
    let left = output.next().expect("Expected left output");
    let right = output.next().expect("Expected right output");
    (left, right)
}

/// Extracts left/right inputs from an inputs slice.
/// Panics if there aren't enough channels for any of the inputs.
fn extract_inputs<'a>(input: &'a [Input]) -> Vec<(&'a Buffer, &'a Buffer)> {
    input
        .iter()
        .map(|input| {
            let mut input = input.buffers().iter();

            let left = input.next().expect("Expected left input");
            let right = input.next().expect("Expected right input");

            (left, right)
        })
        .collect()
}

/// Extracts exactly two sets of input channels.
fn extract_inputs_2<'a>(input: &'a [Input]) -> [(&'a Buffer, &'a Buffer); 2] {
    debug_assert!(input.len() >= 2);
    let x = extract_inputs(input);
    [x[0], x[1]]
}
