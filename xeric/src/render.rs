use dasp_frame::Frame;
use mesic::graph::RenderGraph;
use shared::bytes::as_bytes;
use shared::render::render_server::Render;
use shared::render::{RenderReply, RenderRequest};
use state::StoreData;
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

        // TODO: use the StoreData from the collab context.
        let mut store = StoreData::default();
        store.project = project;

        let graph = RenderGraph::without_rx(&store);

        let audio: Vec<_> = graph.collect();
        let left = as_bytes(&audio.iter().map(|it| *it.channel(0).unwrap()).collect());
        let right = as_bytes(&audio.iter().map(|it| *it.channel(1).unwrap()).collect());

        Ok(tonic::Response::new(RenderReply { left, right }))
    }
}
