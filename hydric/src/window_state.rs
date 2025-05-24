use crate::local_state::GetSet;
use egui::{Pos2, pos2, vec2};
use state::{EffectSelector, GeneratorSelector, MixerSelector, Store};
use std::cell::RefCell;
use std::cmp::{Eq, Ord};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use strum::EnumIter;
use strum::IntoEnumIterator;

/// Windows variants that will appear on the UI.
// TODO: Move all windows to WindowKind
#[derive(Hash, Copy, Clone, Debug, EnumIter, PartialEq, Eq)]
pub enum WindowKind {
    Mixer,
    Effect(EffectSelector),
    GeneratorList,
    Generator(GeneratorSelector),
    Scale,
    SampleTree,
    TrackRoll,
    Save,
}

/// Information about a window needed to render on the UI.
#[derive(Debug, Clone, Copy)]
struct WindowData {
    visible: bool,
    pos: Pos2,
}

impl WindowData {
    pub fn default_from_window(window: WindowKind) -> Self {
        let pos = match window {
            WindowKind::Mixer => pos2(1000.0, 150.0),
            WindowKind::Effect(EffectSelector(.., index)) => {
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

#[derive(Debug, Clone)]
pub struct WindowState2 {
    windows: HashMap<WindowKind, Rc<RefCell<WindowData>>>,
    visible_effects: Rc<RefCell<HashSet<EffectSelector>>>,
    effect_windows: HashMap<WindowKind, Rc<RefCell<WindowData>>>,
}

impl Default for WindowState2 {
    fn default() -> Self {
        let mut windows = HashMap::new();
        let effect_windows = HashMap::new();
        for window in WindowKind::iter() {
            let window_data = Rc::new(RefCell::new(WindowData::default_from_window(window)));
            match window {
                WindowKind::Effect(..) => continue,
                _ => windows.insert(window, window_data),
            };
        }
        Self {
            windows,
            effect_windows,
            visible_effects: Rc::new(RefCell::new(HashSet::new())),
        }
    }
}

impl WindowState2 {
    /// Derive windows that may change in count such as effects and generators from the store every frame.
    /// This way if other uses delete or adds items, the window states will not go out of date.
    pub fn update(&mut self, store: &Store) {
        // Update effect windows based on the store.
        // Get all effects from the store
        let effects: Vec<EffectSelector> = store
            .get()
            .project
            .mixer
            .channels
            .iter()
            .enumerate()
            .map(|(index, channel)| {
                let mixer_sel = MixerSelector(index);
                let effect_sels: Vec<EffectSelector> = channel
                    .effects
                    .iter()
                    .enumerate()
                    .map(|(effect_index, _)| mixer_sel.downcast_effect(effect_index))
                    .collect();
                effect_sels
            })
            .flatten()
            .collect();
        let store_effects: HashSet<EffectSelector> = HashSet::from_iter(effects);
        let effect_windows = self.effect_windows.clone();
        // Remove effects no longer on the store.
        for effect in effect_windows.keys() {
            let WindowKind::Effect(effect_sel) = effect else {
                continue;
            };
            if !store_effects.contains(&effect_sel) {
                self.effect_windows.remove(&effect);
            }
        }
        // Add effects new to the store.
        for effect in store_effects {
            let window = WindowKind::Effect(effect);
            if !self.effect_windows.contains_key(&window) {
                self.effect_windows.insert(
                    window,
                    Rc::new(RefCell::new(WindowData::default_from_window(window))),
                );
            }
        }
        // TODO add updating generators when implemented on generator views.
    }

    pub fn get_visible(&self, window: WindowKind) -> bool {
        let windows = match window {
            WindowKind::Effect(..) => &self.effect_windows,
            _ => &self.windows,
        };
        windows
            .get(&window)
            .expect("Windows should have been initialised.")
            .get()
            .visible
    }

    pub fn set_visible(&self, window: WindowKind, open: bool) {
        let windows = match window {
            WindowKind::Effect(..) => &self.effect_windows,
            _ => &self.windows,
        };

        // Keep track of open changeable windows.
        self.update_visible(window, open);

        windows
            .get(&window)
            .expect("Windows should have been initialised.")
            .update(|mut it| {
                it.visible = open;
                it
            })
    }

    pub fn get_pos(&self, window: WindowKind) -> Pos2 {
        let windows = match window {
            WindowKind::Effect(..) => &self.effect_windows,
            _ => &self.windows,
        };
        windows
            .get(&window)
            .expect("Windows should have been initialised.")
            .get()
            .pos
    }

    pub fn visible_effect(&self) -> Vec<EffectSelector> {
        self.visible_effects.get().into_iter().collect()
    }

    /// Keep track of open changeable windows such as effects and generators.
    fn update_visible(&self, window: WindowKind, open: bool) {
        let (visible_windows, sel) = match window {
            WindowKind::Effect(sel) => (&self.visible_effects, sel),
            // TODO: implement for generators.
            _ => return,
        };
        visible_windows.update(|mut it| {
            if open {
                it.insert(sel);
            } else {
                it.remove(&sel);
            }
            it
        });
    }
}

/// Which windows are currently shown.
pub struct WindowState {
    pub generator_list: bool,
    pub generators: WindowStateField<GeneratorSelector>,
    pub scale: bool,
    pub sample_tree: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            generator_list: false,
            generators: WindowStateField(HashSet::new()),
            scale: false,
            sample_tree: false,
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
