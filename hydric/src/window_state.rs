use crate::local_state::GetSet;
use egui::{Id, Pos2, pos2, vec2};
use state::{EffectSelector, GeneratorSelector, Store};
use std::cell::RefCell;
use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use strum::IntoEnumIterator;
use strum::{Display, EnumIter};

/// Windows variants that will appear on the UI.
#[derive(Hash, Copy, Clone, Display, Debug, EnumIter, PartialEq, Eq)]
pub enum WindowKind {
    Mixer,
    Effect(EffectSelector),
    ChannelEffect,
    GeneratorList,
    Generator(GeneratorSelector),
    Scale,
    SampleTree,
    TrackRoll,
    Save,
    Microphone,
    NoteRoll,
    Placement,
    Note,
    GraphDebug,
}

/// Information about a window needed to render on the UI.
#[derive(Debug, Clone, Copy)]
struct WindowData {
    visible: bool,
    pos: Pos2,
    id: Id,
}

impl WindowData {
    pub fn default_from_window(window: WindowKind) -> Self {
        let pos = match window {
            WindowKind::Mixer => pos2(1000.0, 150.0),
            WindowKind::Effect(EffectSelector(effect_id)) => {
                pos2(1000.0, 150.0) + vec2(50.0 * *effect_id as f32, 50.0 * *effect_id as f32)
            }
            WindowKind::ChannelEffect => pos2(800.0, 150.0),
            WindowKind::GeneratorList => pos2(1100.0, 20.0),
            WindowKind::Generator(..) => pos2(1000.0, 150.0),
            WindowKind::Save => pos2(150.0, 150.0),
            WindowKind::Scale => pos2(50.0, 200.0),
            WindowKind::TrackRoll => pos2(30.0, 200.0),
            WindowKind::SampleTree => pos2(600.0, 20.0),
            WindowKind::Microphone => pos2(400.0, 40.0),
            WindowKind::NoteRoll => pos2(600.0, 20.0),
            WindowKind::Placement => pos2(100.0, 20.0),
            WindowKind::Note => pos2(600.0, 20.0),
            WindowKind::GraphDebug => pos2(600.0, 20.0),
        };
        // Create unique IDs for `WindowKind` that could have multiple variants.
        let id_string = match window {
            WindowKind::Effect(EffectSelector(effect_id)) => {
                format!("{}", *effect_id)
            }
            WindowKind::Generator(GeneratorSelector(generator_id)) => {
                format!("{}", *generator_id)
            }
            _ => "".to_string(),
        };
        Self {
            visible: false,
            pos,
            id: Id::new(format!("window_{window}_{id_string}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowState {
    windows: HashMap<WindowKind, Rc<RefCell<WindowData>>>,
    visible_effects: Rc<RefCell<HashSet<EffectSelector>>>,
    visible_generators: Rc<RefCell<HashSet<GeneratorSelector>>>,
    effect_windows: HashMap<WindowKind, Rc<RefCell<WindowData>>>,
    generator_windows: HashMap<WindowKind, Rc<RefCell<WindowData>>>,
}

impl Default for WindowState {
    fn default() -> Self {
        let mut windows = HashMap::new();
        for window in WindowKind::iter() {
            let window_data = Rc::new(RefCell::new(WindowData::default_from_window(window)));
            match window {
                WindowKind::Effect(..) | WindowKind::Generator(..) => continue,
                _ => windows.insert(window, window_data),
            };
        }
        Self {
            windows,
            effect_windows: HashMap::new(),
            generator_windows: HashMap::new(),
            visible_effects: Rc::new(RefCell::new(HashSet::new())),
            visible_generators: Rc::new(RefCell::new(HashSet::new())),
        }
    }
}

impl WindowState {
    // Update state of variable windows such as effects and generators from store.
    // Should be called every frame.
    pub fn update(&mut self, store: &Store) {
        // Update effect windows based on the store.
        // Get all effects from the store
        let effects: HashSet<EffectSelector> = store
            .get()
            .project
            .effects
            .keys()
            .map(|id| EffectSelector(*id))
            .collect();
        let effect_windows = self.effect_windows.clone();
        // Remove effects no longer on the store.
        for effect in effect_windows.keys() {
            let WindowKind::Effect(effect_sel) = effect else {
                continue;
            };
            if !effects.contains(effect_sel) {
                self.effect_windows.remove(effect);
            }
        }
        // Add effects new to the store.
        for effect in effects {
            let window = WindowKind::Effect(effect);
            self.effect_windows
                .entry(window)
                .or_insert_with(|| Rc::new(RefCell::new(WindowData::default_from_window(window))));
        }

        // Update generator windows based on the store.
        // Get all generators from the store
        let generators: Vec<GeneratorSelector> = store
            .get()
            .project
            .generators
            .keys()
            .map(|id| GeneratorSelector(*id))
            .collect();
        let store_gens: HashSet<GeneratorSelector> = HashSet::from_iter(generators);
        let gen_windows = self.generator_windows.clone();
        // Remove generators no longer on the store.
        for generator in gen_windows.keys() {
            let WindowKind::Generator(gen_sel) = generator else {
                continue;
            };
            if !store_gens.contains(gen_sel) {
                self.generator_windows.remove(generator);
            }
        }
        // Add generators new to the store.
        for gen_sel in store_gens {
            let window = WindowKind::Generator(gen_sel);
            self.generator_windows
                .entry(window)
                .or_insert_with(|| Rc::new(RefCell::new(WindowData::default_from_window(window))));
        }
    }

    pub fn get_id(&self, window: WindowKind) -> Id {
        let windows = match window {
            WindowKind::Effect(..) => &self.effect_windows,
            WindowKind::Generator(..) => &self.generator_windows,
            _ => &self.windows,
        };
        windows
            .get(&window)
            .expect("Windows should have been initialised.")
            .get()
            .id
    }

    pub fn get_visible(&self, window: WindowKind) -> bool {
        let windows = match window {
            WindowKind::Effect(..) => &self.effect_windows,
            WindowKind::Generator(..) => &self.generator_windows,
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
            WindowKind::Generator(..) => &self.generator_windows,
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
            WindowKind::Generator(..) => &self.generator_windows,
            _ => &self.windows,
        };
        windows
            .get(&window)
            .expect("Windows should have been initialised.")
            .get()
            .pos
    }

    pub fn visible_effects(&self) -> Vec<EffectSelector> {
        self.visible_effects.get().into_iter().collect()
    }

    pub fn visible_generators(&self) -> Vec<GeneratorSelector> {
        self.visible_generators.get().into_iter().collect()
    }

    /// Keep track of open changeable windows such as effects and generators.
    fn update_visible(&self, window: WindowKind, open: bool) {
        match window {
            WindowKind::Effect(sel) => self.visible_effects.update(|mut it| {
                if open {
                    it.insert(sel);
                } else {
                    it.remove(&sel);
                }
                it
            }),
            WindowKind::Generator(sel) => self.visible_generators.update(|mut it| {
                if open {
                    it.insert(sel);
                } else {
                    it.remove(&sel);
                }
                it
            }),
            _ => (),
        };
    }
}
