use leptos::logging::log;
use shared::render::{RenderRequest, RenderReply, render_client::RenderClient};
use tonic_web_wasm_client::Client;
use shared::model::wave_type::WaveType;
use mesic::DemoOption;
use mesic::create_demo_track;
use mesic::io::as_floats;

pub async fn render(demo_option: DemoOption, wave: WaveType) -> Vec<f32> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = RenderClient::new(wasm_client);

    let result = grpc
        .render(RenderRequest {
            track: Some(create_demo_track(demo_option, wave).into()),
        })
        .await;

    // TODO: proper error handling
    let audio = result.map(|it| it.into_inner().audio).unwrap_or(vec!());

    log!("{:?}", audio);

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    as_floats(&audio)
}
