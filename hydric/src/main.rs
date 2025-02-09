use leptos::*;
use std::panic;

pub mod ping;
pub mod app;

use crate::app::App;

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    mount_to_body(|| view! { <App/> });
}
