use leptos::prelude::*;

mod audio_player;
mod audio_render;
mod components;
mod egui_app;
mod egui_init;
mod note_save;

use crate::components::App;
use crate::egui_init::egui_init;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <App /> });

    // Init the embedded immediate-mode canvas app
    egui_init();
}
