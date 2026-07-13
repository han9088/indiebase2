use std::net::SocketAddr;

use api::config::Config;
use api::db::{connect_pool, connect_redis, ensure_dev_seed_user, prepare_schema};
use api::listen_banner::format_listen_banner;
use api::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let migrate_only = std::env::args().any(|arg| arg == "--migrate-only");

    let config = Config::from_env()?;
    let pool = connect_pool(&config).await.map_err(|err| {
        eprintln!("{err}");
        err
    })?;

    prepare_schema(&pool, &config).await?;
    ensure_dev_seed_user(&pool, &config).await?;

    if migrate_only {
        tracing::info!("schema prepared; exiting (--migrate-only)");
        return Ok(());
    }

    let redis = connect_redis(&config).await.map_err(|err| {
        eprintln!("{err}");
        err
    })?;

    let state = AppState::new(pool, redis, config.clone());
    let addr: SocketAddr = config.http_addr.parse()?;

    println!("{}", format_listen_banner(&config.env, addr));
    tracing::info!(%addr, env = %config.env, "indiebase api listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, api::app::router(state)).await?;

    Ok(())
}
