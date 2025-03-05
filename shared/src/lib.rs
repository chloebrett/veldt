pub mod bytes;
pub mod model;
pub mod serialize;
pub mod types;

pub mod render {
    tonic::include_proto!("render");
}

pub mod pmodel {
    tonic::include_proto!("pmodel");
}
