use std::sync::Arc;

use redis::aio::ConnectionManager;
use reqwest::Client;
use sqlx::PgPool;

use crate::config::Config;

/// Shared Axum state: Postgres pool, Redis, HTTP client, and config.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub http: Client,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(pool: PgPool, redis: ConnectionManager, config: Config) -> Self {
        Self {
            pool,
            redis,
            http: Client::new(),
            config: Arc::new(config),
        }
    }
}
