use state::{EffectSelector, GeneratorSelector, MixerSelector};
use std::cmp::{Eq, Ord};
use std::collections::HashSet;
use std::hash::Hash;

pub struct MixerWindowState {
    pub visible: bool,
    // Currently active / shown channel.
    pub channel: MixerSelector,
}

/// Which windows are currently shown.
pub struct WindowState {
    pub mixer: MixerWindowState,
    pub effects: WindowStateField<EffectSelector>,
    pub generator_list: bool,
    pub generators: WindowStateField<GeneratorSelector>,
    pub scale: bool,
    pub sample_tree: bool,
    pub track_roll: bool,
    pub save: bool,
    pub microphone: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            mixer: MixerWindowState {
                visible: false,
                channel: MixerSelector(0),
            },
            effects: WindowStateField(HashSet::new()),
            generator_list: false,
            generators: WindowStateField(HashSet::new()),
            scale: false,
            sample_tree: false,
            track_roll: false,
            save: false,
            microphone: false,
        }
    }
}

#[derive(Clone)]
pub struct WindowStateField<T: Hash + Eq + Copy>(HashSet<T>);

impl<T: Hash + Ord + Copy> WindowStateField<T> {
    pub fn get(&self, index: T) -> bool {
        self.0.contains(&index)
    }

    pub fn set(&mut self, index: T, visible: bool) {
        let was_visible = self.get(index);
        if visible == was_visible {
            return;
        }
        if visible {
            self.0.insert(index);
        } else {
            self.0.retain(|it| *it != index);
        }
    }

    // Note: not necessarily sorted.
    pub fn as_vec(&self) -> Vec<T> {
        self.0.clone().into_iter().collect()
    }
}
