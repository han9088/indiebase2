use std::net::SocketAddr;

use api::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    let addr: SocketAddr = config.http_addr.parse()?;

    tracing::info!(%addr, mode = %config.mode, "indiebase api listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, api::app::router()).await?;

    Ok(())
}
