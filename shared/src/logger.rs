#[cfg(target_arch = "wasm32")]
use web_sys::console;

// Note: consider passing closures to these methods if it is more performant.

#[cfg(target_arch = "wasm32")]
pub fn log(message: &str) {
    console::log_1(&message.to_string().into());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(message: &str) {
    println!("{}", message);
}

#[cfg(target_arch = "wasm32")]
pub fn error(message: &str) {
    console::error_1(&message.to_string().into());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn error(message: &str) {
    println!("Error: {}", message);
}
