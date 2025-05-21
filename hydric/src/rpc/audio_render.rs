use dasp_frame::Stereo;
use mesic::interleave_stereo;
use shared::bytes::as_floats;
use shared::consts::XERIC_URL;
use shared::model::Project;
use shared::render::{RenderRequest, render_client::RenderClient};
use tonic_web_wasm_client::Client;

pub async fn render(project: Project) -> Result<Vec<Stereo<f32>>, tonic::Status> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = RenderClient::new(client);

    let result = grpc
        .render(RenderRequest {
            project: Some(project.into()),
        })
        .await?;

    let result = result.into_inner().clone();
    let left = result.left;
    let right = result.right;

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    Ok(interleave_stereo(as_floats(&left), as_floats(&right)))
}
