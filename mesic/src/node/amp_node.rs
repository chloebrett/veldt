use super::{extract_inputs, extract_outputs};
use crate::graph::ProcessContext;
use dasp_graph::{Buffer, Input, Node};
use shared::types::Volume;
use state::{MixerMatrixCellSelector, MixerSelector};

enum AmpNodeSelector {
    // Main amp.
    // TODO: can maybe fold this into channel amp and just use the main channel amp.
    // But what about if we solo a channel?
    // Leave it for now.
    Main,
    // Amp for the output of a single channel.
    Channel(MixerSelector),
    // Amp for a single route between channels.
    Route(MixerMatrixCellSelector),
}

/// Node with a volume control.
/// Currently just reads from the overall project volume, but could be
/// made configurable.
/// Clips the post-gain signal.
pub struct AmpNode {
    selector: AmpNodeSelector,
}

impl AmpNode {
    fn new(selector: AmpNodeSelector) -> Self {
        Self { selector }
    }

    pub fn new_main() -> Self {
        Self::new(AmpNodeSelector::Main)
    }

    pub fn new_for_channel(sel: MixerSelector) -> Self {
        Self::new(AmpNodeSelector::Channel(sel))
    }

    pub fn new_for_route(sel: MixerMatrixCellSelector) -> Self {
        Self::new(AmpNodeSelector::Route(sel))
    }
}

impl AmpNode {
    fn process_channel(&self, volume: Volume, out: &mut Buffer) {
        for x in out.iter_mut() {
            // Apply the volume multiplier.
            let mut amped = *x * volume;

            // Clip the output so that the magnitude doesn't go above 1.
            amped = amped.clamp(-1.0, 1.0);

            *x = amped
        }
    }
}

impl Node<ProcessContext> for AmpNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let volume = match self.selector {
            AmpNodeSelector::Main => payload.store.volume,
            AmpNodeSelector::Channel(MixerSelector(channel)) => {
                if payload.store.project.mixer.channels[channel].mute {
                    0.0
                } else {
                    payload.store.project.mixer.channels[channel].volume
                }
            }
            AmpNodeSelector::Route(MixerMatrixCellSelector(row, col)) => {
                (*payload.store.project.mixer.matrix.get(row, col).unwrap()).into()
            }
        };

        let (out_left, out_right) = extract_outputs(output);
        let (in_left, in_right) = extract_inputs(inputs)[0];

        out_left.copy_from_slice(in_left);
        self.process_channel(volume, out_left);
        out_right.copy_from_slice(in_right);
        self.process_channel(volume, out_right);
    }
}
