use dasp_frame::Stereo;
use shared::bytes::as_floats;
use shared::consts::XERIC_URL;
use shared::model::Project;
use shared::render::{RenderRequest, render_client::RenderClient};
use tonic_web_wasm_client::Client;

pub async fn render(project: Project) -> Result<Vec<Stereo<f32>>, ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = RenderClient::new(client);

    let result = grpc
        .render(RenderRequest {
            project: Some(project.into()),
        })
        .await
        .map_err(|_| ())?;

    let result = result.into_inner().clone();
    let left = result.left;
    let right = result.right;

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    let data = interleave_stereo(as_floats(&left), as_floats(&right));
    Ok(data)
}

pub fn interleave_stereo(left: Vec<f32>, right: Vec<f32>) -> Vec<Stereo<f32>> {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| [*left, *right])
        .collect()
}
