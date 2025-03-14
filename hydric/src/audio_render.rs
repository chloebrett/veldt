use shared::bytes::as_floats;
use shared::consts::XERIC_URL;
use shared::model::{EffectInstance, GeneratorInstance, Track};
use shared::render::{RenderRequest, render_client::RenderClient};
use shared::serialize::map_vec;
use shared::types::Beats;
use tonic_web_wasm_client::Client;
use web_sys::console;

pub async fn render(
    track: Track,
    effects: Vec<EffectInstance>,
    generator: GeneratorInstance,
    bpm: Beats,
) -> Option<Vec<f32>> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = RenderClient::new(client);

    let result = grpc
        .render(RenderRequest {
            track: Some(track.into()),
            effects: map_vec(effects.into()),
            generator: Some(generator.into()),
            bpm,
        })
        .await;

    let audio_result = result.map(|it| it.into_inner().audio);

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    match audio_result {
        Ok(audio) => Some(as_floats(&audio)),
        Err(_) => {
            console::error_1(&"Error rendering audio on server.".into());
            None
        }
    }
}
