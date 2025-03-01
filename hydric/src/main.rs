use leptos::prelude::*;

pub mod app;
mod egui_app;
pub mod egui_init;
pub mod ping;
pub mod player;
pub mod render;

use crate::app::App;
use crate::egui_init::egui_init;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <App /> });

    // Init the embedded immediate-mode canvas app
    egui_init();
}
