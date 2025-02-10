use leptos::*;
use leptos::prelude::*;
use thaw::{Button, Space, ConfigProvider, ButtonAppearance};

use crate::ping::ping;
use crate::player::AudioPlayer;

#[component]
pub fn App() -> impl IntoView {
    let (player, set_player) = signal_local(None::<AudioPlayer>);

    let ping_action = Action::new_local(|_: &()| async {
        ping().await;
    });

    view! {
        <ConfigProvider>
            <p>"Veldt"</p>
            <Card>
                <Space>
                    <Button
                        appearance=ButtonAppearance::Primary
                        on_click=move |_| {
                            set_player.set(AudioPlayer::new().unwrap().into());
                        }
                    >
                        "Play"
                    </Button>
                    <Button
                        appearance=ButtonAppearance::Secondary
                        on_click=move |_| {
                            ping_action.dispatch(());
                        }
                    >
                        "Ping"
                    </Button>
                </Space>
            </Card>
        </ConfigProvider>
    }
}

