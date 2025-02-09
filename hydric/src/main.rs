use leptos::*;

pub mod ping;

use crate::ping::ping;

#[component]
pub fn App() -> impl IntoView {
    let ping_action = Action::new(|input: &()| async {
        ping();
    });

    view! {
        <html>
            <head>
                <title>My Leptos App</title>
            </head>
            <body>
                <p>"Hello, world!"</p>
                <input type="button" on:click=move |ev| {
                   ping_action.dispatch(()); 
                } />
                <script type="module" src="script.mjs"></script>
            </body>
        </html>
    }
}

fn main() {
    mount_to_body(|| view! { <App/> });
}
