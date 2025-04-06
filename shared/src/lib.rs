pub mod bytes;
pub mod consts;
pub mod logger;
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
}

pub mod save_load {
    tonic::include_proto!("save_load");
}
