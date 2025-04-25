pub mod bytes;
pub mod consts;
pub mod model;
pub mod serialize;
mod testing;
pub mod types;

pub mod render {
    tonic::include_proto!("render");
}

pub mod load_sample {
    tonic::include_proto!("load_sample");
}

pub mod pmodel {
    tonic::include_proto!("pmodel");

    impl From<Option<f32>> for FloatOptionProto {
        fn from(item: Option<f32>) -> FloatOptionProto {
            FloatOptionProto { value: item }
        }
    }

    impl From<FloatOptionProto> for Option<f32> {
        fn from(item: FloatOptionProto) -> Option<f32> {
            item.value
        }
    }
}

pub mod save_load {
    tonic::include_proto!("save_load");
}

#[expect(clippy::module_inception)]
pub mod action_proto {
    tonic::include_proto!("action_proto");
}

pub mod broadcast_actions {
    tonic::include_proto!("broadcast_actions");
}

pub mod upload {
    tonic::include_proto!("upload");
}
