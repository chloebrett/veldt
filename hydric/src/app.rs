use leptos::*;
use leptos::prelude::*;
use thaw::{Card, Button, Space, ConfigProvider, ButtonAppearance, Select};
use shared::model::wave_type::WaveType;
use std::str::FromStr;

use crate::ping::ping;
use crate::player::AudioPlayer;

#[component]
pub fn App() -> impl IntoView {
    let (player, set_player) = signal_local(None::<AudioPlayer>);
    let wave_string = RwSignal::new(WaveType::Sine.to_string());

    // TODO: instead of using a dependent signal, consider implementing
    // the appropriate From trait.
    let wave = move || WaveType::from_str(&wave_string.get()).unwrap();

    let ping_action = Action::new_local(|_: &()| async {
        ping().await;
    });

    view! {
        <ConfigProvider>
            <h1>"Veldt"</h1>
            <Card>
                <Space>
                    <Button
                        appearance=ButtonAppearance::Primary
                        on_click=move |_| {
                            set_player.set(AudioPlayer::new(wave()).unwrap().into());
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
                    <Select value=wave_string>
                    <option>Sine</option>
                    <option>Square</option>
                    <option>Saw</option>
                    <option>Triangle</option>
                    </Select>
                </Space>
            </Card>
        </ConfigProvider>
    }
}

