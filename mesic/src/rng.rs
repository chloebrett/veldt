use rand::Rng;
use wasm_bindgen::prelude::wasm_bindgen; // Needs to be kept for compiling for WASM

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Generates random number between [min, max]
#[wasm_bindgen]
pub fn generate_random_number_in_range(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

// WASM specific version
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_wasm_random_number_in_range(min: f32, max: f32) -> f32 {
    generate_random_number_in_range(min, max)
}
