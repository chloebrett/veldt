use leptos::*;

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

