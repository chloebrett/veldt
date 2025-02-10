use leptos::*;
use leptos::prelude::*;

use crate::ping::ping;
use crate::player::AudioPlayer;

#[component]
pub fn App() -> impl IntoView {
    let (player, set_player) = signal_local(None::<AudioPlayer>);

    let ping_action = Action::new_local(|input: &()| async {
        ping().await;
    });

    view! {
        <div>
            <p>"Hello, world!"</p>
            <button on:click=move |_| {
                ping_action.dispatch(());
            }>Ping</button>
            <button on:click=move |_| {
                set_player.set(AudioPlayer::new().unwrap().into());
            }>Play</button>
        </div>
    }
}

