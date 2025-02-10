use leptos::logging::log;
use shared::render::{RenderRequest, RenderReply, render_client::RenderClient};
use tonic_web_wasm_client::Client;
use shared::model::wave_type::WaveType;
use mesic::create_demo_track;
use mesic::io::as_floats;

pub async fn render(wave: WaveType) -> Vec<f32> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = RenderClient::new(wasm_client);

    let result: RenderReply = grpc
        .render(RenderRequest {
            track: Some(create_demo_track(wave).into()),
        })
        .await
        .unwrap().into_inner();

    log!("{:?}", result);

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    as_floats(&result.audio)
}
