use shared::bytes::as_floats;
use shared::consts::XERIC_URL;
use shared::model::Project;
use shared::render::{RenderRequest, render_client::RenderClient};
use tonic_web_wasm_client::Client;

pub async fn render(project: Project) -> Result<Vec<f32>, ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = RenderClient::new(client);

    let result = grpc
        .render(RenderRequest {
            project: Some(project.into()),
        })
        .await;

    let audio_result = result.map(|it| it.into_inner().audio);

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    audio_result.map(|it| as_floats(&it)).map_err(|_| ())
}
