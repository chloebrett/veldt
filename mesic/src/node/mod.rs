use dasp_graph::{Buffer, Input};
use shared::types::{KnobPosition, Volume};

mod amp_node;
mod buffer_node;
mod compressor_node;
mod delay_node;
mod eq_node;
mod mod_delay_node;
mod noise_generator_node;
mod simple_wave_generator_node;
mod stingray_node;
mod wet_dry_node;

pub use amp_node::*;
pub use buffer_node::*;
pub use compressor_node::*;
pub use delay_node::*;
pub use eq_node::*;
pub use mod_delay_node::*;
pub use noise_generator_node::*;
pub use simple_wave_generator_node::*;
pub use stingray_node::*;
pub use wet_dry_node::*;

/// Extracts left/right outputs from an outputs slice.
/// Panics if there aren't enough channels.
fn extract_outputs(output: &mut [Buffer]) -> (&mut Buffer, &mut Buffer) {
    let output = &mut output.iter_mut();
    let left = output.next().expect("Expected left output");
    let right = output.next().expect("Expected right output");
    (left, right)
}

/// Extracts left/right inputs from an inputs slice.
/// Panics if there aren't enough channels for any of the inputs.
fn extract_inputs(input: &[Input]) -> Vec<(&Buffer, &Buffer)> {
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
fn extract_inputs_2(input: &[Input]) -> [(&Buffer, &Buffer); 2] {
    debug_assert!(input.len() >= 2);
    let x = extract_inputs(input);
    [x[0], x[1]]
}

/// TODO: use exponential pan curves, instead of linear.
fn pan_multipliers(pan: KnobPosition) -> [Volume; 2] {
    let left = 0.5 * (1.0 - pan);
    let right = 0.5 * (1.0 + pan);
    [left, right]
}
