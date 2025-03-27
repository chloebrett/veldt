#[cfg(target_arch = "wasm32")]
use web_sys::console;

// Note: consider passing closures to these methods if it is more performant.

#[cfg(target_arch = "wasm32")]
pub fn log(message: String) {
    console::log_1(&message.into());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(message: String) {
    println!("{}", message);
}

#[cfg(target_arch = "wasm32")]
pub fn error(message: String) {
    console::error_1(&message.into());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn error(message: String) {
    println!("Error: {}", message);
}
