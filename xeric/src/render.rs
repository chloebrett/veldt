use mesic::render;
use shared::bytes::as_bytes;
use shared::render::render_server::Render;
use shared::render::{RenderReply, RenderRequest};
use tonic::async_trait;

pub struct RenderContext;

#[async_trait]
impl Render for RenderContext {
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
        let bytes = as_bytes(&graph.collect());

        Ok(tonic::Response::new(RenderReply { audio: bytes }))
    }
}
