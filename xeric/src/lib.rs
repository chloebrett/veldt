use crate::load_sample::MyLoadSample;
use http::{HeaderValue, Method};
use mesic::render;
use save_load::SaveLoadContext;
use shared::bytes::as_bytes;
use shared::consts::{HYDRIC_URL, XERIC_SOCKET_ADDR};
use shared::load_sample::load_sample_server::LoadSampleServer;
use shared::render::render_server::{Render, RenderServer};
use shared::render::{RenderReply, RenderRequest};
use shared::save_load::load_project_list_server::LoadProjectListServer;
use shared::save_load::load_project_server::LoadProjectServer;
use shared::save_load::save_project_server::SaveProjectServer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::async_trait;
use tonic_web::GrpcWebLayer;
use tower_http::cors::AllowHeaders;

pub mod load_sample;
pub mod save_load;

struct RenderContext;

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

pub async fn start_server() -> anyhow::Result<()> {
    let render = RenderServer::new(RenderContext);
    let load_sample = LoadSampleServer::new(MyLoadSample);

    // Projects list gets shared when this is cloned.
    let save_load_context = SaveLoadContext {
        projects: Arc::new(Mutex::new(HashMap::new())),
    };

    let save_project = SaveProjectServer::new(save_load_context.clone());
    let load_project_list = LoadProjectListServer::new(save_load_context.clone());
    let load_project = LoadProjectServer::new(save_load_context.clone());

    tonic::transport::Server::builder()
        .accept_http1(true)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_origin(HYDRIC_URL.parse::<HeaderValue>().unwrap())
                .allow_headers(AllowHeaders::mirror_request())
                .allow_credentials(true),
        )
        .layer(GrpcWebLayer::new())
        .add_service(render)
        .add_service(load_sample)
        .add_service(save_project)
        .add_service(load_project)
        .add_service(load_project_list)
        .serve(*XERIC_SOCKET_ADDR)
        .await?;

    Ok(())
}
