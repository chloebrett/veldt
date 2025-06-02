use shared::model::{Mixer, Project};
use state::StoreData;
use std::collections::HashMap;

pub fn empty_store_data() -> StoreData {
    StoreData {
        project: Project {
            name: "empty".into(),
            tracks: vec![],
            placements: vec![],
            samples: vec![],
            generators: HashMap::new(),
            mixer: Mixer::default(),
            bpm: 120.0,
        },
        ..Default::default()
    }
}
