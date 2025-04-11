use super::dual_channel;
use crate::effect::ApplyFilter;
use dasp_graph::{Buffer, Input, Node};

pub struct EqNode {
    pub filter_left: Box<dyn ApplyFilter + Send>,
    pub filter_right: Box<dyn ApplyFilter + Send>,
}

impl Node for EqNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        let (left_out, left_in, right_out, right_in) = dual_channel(inputs, output);

        left_out.copy_from_slice(left_in);
        self.filter_left.apply(left_out);

        right_out.copy_from_slice(right_in);
        self.filter_right.apply(right_out);
    }
}
