use crate::collab::CollabContext;
use crate::load_sample::LoadSampleContext;
use crate::render::RenderContext;
use crate::save_load::SaveLoadContext;
use http::{HeaderValue, Method};
use shared::broadcast_actions::broadcast_actions_server::BroadcastActionsServer;
use shared::consts::{HYDRIC_URL, XERIC_SOCKET_ADDR};
use shared::load_sample::load_sample_server::LoadSampleServer;
use shared::render::render_server::RenderServer;
use shared::save_load::save_load_server::SaveLoadServer;
use tonic_web::GrpcWebLayer;
use tower_http::cors::AllowHeaders;

mod collab;
mod load_sample;
mod render;
mod save_load;

pub async fn start_server() -> anyhow::Result<()> {
    let render = RenderServer::new(RenderContext);
    let load_sample = LoadSampleServer::new(LoadSampleContext);

    let save_load = SaveLoadServer::new(SaveLoadContext);

    // Broadcasting on the server version of the stack shouldn't do anything.
    // TODO: handle this better.
    let broadcast = |_| {};
    let collab_context = CollabContext::new(broadcast);
    let broadcast_actions = BroadcastActionsServer::new(collab_context);

    tonic::transport::Server::builder()
        .accept_http1(true)
        .layer(
            // Without this configuration, requests from the client will be blocked by the browser.
            // This is also the reason the URL in the browser must be 127.0.0.1:8080, *not* localhost:8080.
            tower_http::cors::CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_origin(HYDRIC_URL.parse::<HeaderValue>().unwrap())
                .allow_headers(AllowHeaders::mirror_request())
                .allow_credentials(true),
        )
        .layer(GrpcWebLayer::new())
        .add_service(render)
        .add_service(load_sample)
        .add_service(save_load)
        .add_service(broadcast_actions)
        .serve(*XERIC_SOCKET_ADDR)
        .await?;

    Ok(())
}
