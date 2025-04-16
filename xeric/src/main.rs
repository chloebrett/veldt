use xeric::start_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    start_server().await
}
