use lazy_static::lazy_static;
use std::net::SocketAddr;

pub const XERIC_URL: &str = "http://127.0.0.1:3000";
pub const HYDRIC_URL: &str = "http://127.0.0.1:8080";

lazy_static! {
    pub static ref XERIC_SOCKET_ADDR: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 3000));
}
