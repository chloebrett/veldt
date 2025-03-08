use leptos::logging::log;
use shared::bytes::as_floats;
use shared::model::Track;
use shared::render::{RenderRequest, render_client::RenderClient};
use tonic_web_wasm_client::Client;

pub async fn render(track: Track) -> Vec<f32> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = RenderClient::new(wasm_client);

    let result = grpc
        .render(RenderRequest {
            track: Some(track.into()),
        })
        .await;

    // TODO: proper error handling
    let audio = result.map(|it| it.into_inner().audio).unwrap_or(vec![]);

    log!("{:?}", audio);

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    as_floats(&audio)
}
