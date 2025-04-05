use crate::effect::ApplyFilter;
use dasp_graph::{Buffer, Input, Node};

pub struct EqNode {
    pub filter: Box<dyn ApplyFilter>,
}

impl Node for EqNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.first().expect("Expected one input").buffers())
        {
            out_buf.copy_from_slice(&self.filter.apply(in_buf));
        }
    }
}
