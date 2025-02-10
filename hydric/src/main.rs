use leptos::prelude::*;

pub mod ping;
pub mod render;
pub mod app;
pub mod player;

use crate::app::App;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <App /> });
}
