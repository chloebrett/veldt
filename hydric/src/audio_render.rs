use shared::bytes::as_floats;
use shared::consts::XERIC_URL;
use shared::model::Track;
use shared::render::{RenderRequest, render_client::RenderClient};
use tonic_web_wasm_client::Client;

pub async fn render(track: Track) -> Vec<f32> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = RenderClient::new(client);

    let result = grpc
        .render(RenderRequest {
            track: Some(track.into()),
        })
        .await;

    // TODO: proper error handling
    let audio = result.map(|it| it.into_inner().audio).unwrap_or(vec![]);

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    as_floats(&audio)
}
