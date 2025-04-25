use super::ProcessContext;
use super::{extract_inputs, extract_outputs};
use crate::effect::ApplyFilter;
use dasp_graph::{Buffer, Input, Node};

pub struct EqNode {
    pub filter_left: Box<dyn ApplyFilter + Send>,
    pub filter_right: Box<dyn ApplyFilter + Send>,
}

impl Node<ProcessContext> for EqNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], _payload: &ProcessContext) {
        let (out_left, out_right) = extract_outputs(output);
        let (in_left, in_right) = extract_inputs(inputs)[0];

        out_left.copy_from_slice(in_left);
        self.filter_left.apply(out_left);

        out_right.copy_from_slice(in_right);
        self.filter_right.apply(out_right);
    }
}
