use shared::model::{Mixer, ModMatrix, Project};
use state::StoreData;

pub fn empty_store_data() -> StoreData {
    StoreData {
        project: Project {
            name: "empty".into(),
            tracks: vec![],
            placements: vec![],
            samples: vec![],
            generators: vec![],
            mixer: Mixer::default(),
            bpm: 120.0,
            mod_matrix: ModMatrix::default(),
        },
        ..Default::default()
    }
}
