use dasp_frame::Frame;
use mesic::render;
use shared::bytes::as_bytes;
use shared::render::render_server::Render;
use shared::render::{RenderReply, RenderRequest};
use tonic::async_trait;

// This is a stateless RPC: it accepts a project and returns audio bytes of the rendered project.
pub struct RenderContext;

#[async_trait]
impl Render for RenderContext {
    /// Renders a project and returns audio bytes.
    async fn render(
        &self,
        request: tonic::Request<RenderRequest>,
    ) -> Result<tonic::Response<RenderReply>, tonic::Status> {
        let project = request
            .into_inner()
            .project
            .ok_or(tonic::Status::invalid_argument("Project must be supplied"))?
            .into();
        let graph = &mut render(&project);
        // TODO: render out stereo audio.
        // Currently, this gets sent over the wire as mono, then put into a BufferNode, which will
        // cause it to play as stereo but only the left channel.
        let bytes = as_bytes(&graph.map(|it| *it.channel(0).unwrap()).collect());

        Ok(tonic::Response::new(RenderReply { audio: bytes }))
    }
}
