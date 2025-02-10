pub mod model;
pub mod serialize;

pub mod echo {
    tonic::include_proto!("echo");
}

pub mod pmodel {
    tonic::include_proto!("pmodel");
}
