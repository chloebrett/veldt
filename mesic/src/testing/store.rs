use shared::model::{Mixer, Project};
use state::StoreData;
use std::collections::HashMap;

pub fn empty_store_data() -> StoreData {
    StoreData {
        project: Project {
            name: "empty".into(),
            tracks: HashMap::new(),
            placements: HashMap::new(),
            samples: HashMap::new(),
            generators: HashMap::new(),
            effects: HashMap::new(),
            mixer: Mixer::default(),
            bpm: 120.0,
        },
        ..Default::default()
    }
}
