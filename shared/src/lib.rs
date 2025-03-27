pub mod bytes;
pub mod consts;
pub mod model;
pub mod serialize;
pub mod types;
pub mod logger;

pub mod render {
    tonic::include_proto!("render");
}

pub mod pmodel {
    tonic::include_proto!("pmodel");
}

pub mod load_sample {
    tonic::include_proto!("load_sample");
}

pub mod save_track {
    tonic::include_proto!("save_track");
}
