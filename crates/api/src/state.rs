use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::config::Config;

/// Shared Axum state: Postgres pool, Redis, and config.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(pool: PgPool, redis: ConnectionManager, config: Config) -> Self {
        Self {
            pool,
            redis,
            config: Arc::new(config),
        }
    }
}
