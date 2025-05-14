use rand::Rng;
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate_random_number() -> f64 {
    let mut rng = rand::thread_rng();
    (rng.r#gen::<f64>() * 2.0) - 1.0
}

#[wasm_bindgen]
pub fn generate_random_number_in_range(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_wasm_random_number() -> f64 {
    generate_random_number()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_wasm_random_number_in_range(min: f64, max: f64) -> f64 {
    generate_random_number_in_range(min, max)
}

/// White noise function
pub fn generate_white_noise() -> f64 {
    generate_random_number();
}