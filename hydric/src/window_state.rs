use crate::local_state::{GetSet, LocalState};
use egui::{Pos2, pos2, vec2};
use state::{EffectSelector, GeneratorSelector, Store};
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
}

impl Default for WindowState2 {
    fn default() -> Self {
        let mut windows = HashMap::new();
        for window in WindowKind::iter() {
            windows.insert(
                window,
                Rc::new(RefCell::new(WindowData::default_from_window(window))),
            );
        }
        Self {
            windows,
            visible_effects: Rc::new(RefCell::new(HashSet::new())),
        }
    }
}

impl WindowState2 {
    /// Derive changable windows such as effects and generators from the store every frame.
    /// This way if other uses delete items, the window states will not go out of date.
    pub fn update(&mut self, store: &Store, local_state: &LocalState) {
        // Add any new effects from active mixer.
        if let Some(mixer_sel) = local_state.active_mixer_chanel.get() {
            let mixer = store.select(&mixer_sel);
            for effect_index in 0..mixer.effects.len() {
                let effect_sel = mixer_sel.downcast_effect(effect_index);
                let window = WindowKind::Effect(effect_sel);
                if !self.windows.contains_key(&window) {
                    self.windows.insert(
                        window,
                        Rc::new(RefCell::new(WindowData::default_from_window(window))),
                    );
                }
            }
        };
        // TODO add updating generators when implemented on generator views.
    }

    pub fn get_visible(&self, window: WindowKind) -> bool {
        self.windows
            .get(&window)
            .expect("Windows should have been initialised.")
            .get()
            .visible
    }

    pub fn set_visible(&self, window: WindowKind, open: bool) {
        // Keep track of open effect windows.
        if let WindowKind::Effect(effect_sel) = window {
            self.visible_effects.update(|mut it| {
                if open {
                    it.insert(effect_sel);
                } else {
                    it.remove(&effect_sel);
                }
                it
            })
        }

        // TODO: Keep track of open generator windows.

        self.windows
            .get(&window)
            .expect("Windows should have been initialised.")
            .update(|mut it| {
                it.visible = open;
                it
            })
    }

    pub fn get_pos(&self, window: WindowKind) -> Pos2 {
        self.windows
            .get(&window)
            .expect("Windows should have been initialised.")
            .get()
            .pos
    }

    pub fn visible_effect(&self) -> Vec<EffectSelector> {
        // TODO: Should only effects from active mixer channel be open?
        self.visible_effects.get().into_iter().collect()
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
