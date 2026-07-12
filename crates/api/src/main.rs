use std::net::SocketAddr;

use api::config::Config;
use api::listen_banner::format_listen_banner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let addr: SocketAddr = config.http_addr.parse()?;

    println!("{}", format_listen_banner(&config.env, addr));
    tracing::info!(%addr, env = %config.env, "indiebase api listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, api::app::router()).await?;

    Ok(())
}
