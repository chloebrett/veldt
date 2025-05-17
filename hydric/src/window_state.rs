use egui::{Pos2, pos2, vec2};
use state::{EffectSelector, GeneratorSelector, MixerSelector};
use std::cmp::{Eq, Ord};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use strum::EnumIter;
use strum::IntoEnumIterator;

#[derive(Hash, Copy, Clone, EnumIter, PartialEq, Eq)]
pub enum WindowKind {
    Mixer,
    Effect(usize),
    GeneratorList,
    Generator(usize),
    Scale,
    SampleTree,
    TrackRoll,
    Save,
}

struct WindowData {
    visible: bool,
    pos: Pos2,
}

impl WindowData {
    pub fn default_from_window(window: WindowKind) -> Self {
        let pos = match window {
            WindowKind::Mixer => pos2(1000.0, 150.0),
            WindowKind::Effect(index) => {
                pos2(1000.0, 150.0) + vec2(50.0 * index as f32, 50.0 * index as f32)
            }
            WindowKind::GeneratorList => pos2(1100.0, 20.0),
            WindowKind::Generator(..) => pos2(1000.0, 150.0),
            WindowKind::Save => pos2(150.0, 150.0),
            WindowKind::Scale => pos2(50.0, 200.0),
            WindowKind::TrackRoll => pos2(30.0, 200.0),
            WindowKind::SampleTree => pos2(600.0, 20.0),
        };
        Self {
            visible: false,
            pos,
        }
    }
}

pub struct WindowState2(HashMap<WindowKind, WindowData>);

impl Default for WindowState2 {
    fn default() -> Self {
        let mut windows = HashMap::new();
        for window in WindowKind::iter() {
            windows.insert(window, WindowData::default_from_window(window));
        }
        Self(windows)
    }
}

impl WindowState2 {
    pub fn get_mut_visible(&mut self, window: WindowKind) -> &mut bool {
        &mut self
            .0
            .get_mut(&window)
            .expect("Windows should have been initialised.")
            .visible
    }

    pub fn get_pos(&self, window: WindowKind) -> Pos2 {
        self.0
            .get(&window)
            .expect("Windows should have been initialised.")
            .pos
    }
}

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
    pub save: bool,
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
            save: false,
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
