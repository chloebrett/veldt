use leptos::*;
use std::panic;

pub mod ping;

use crate::ping::ping;

#[component]
pub fn App() -> impl IntoView {
    let ping_action = Action::new(|_| async {
        ping().await;
    });

    view! {
        <div>
            <p>"Hello, world!"</p>
            <button on:click=move |_| {
                ping_action.dispatch(());
            }>Ping</button>
        </div>
    }
}

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    mount_to_body(|| view! { <App/> });
}
