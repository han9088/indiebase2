use redis::aio::ConnectionManager;
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Config;
use crate::constants::auth::{DEV_SEED_EMAIL, DEV_SEED_PASSWORD};

pub mod gateway_sql;
pub mod schema;
pub mod sync;

/// Connect to Postgres and fail fast if unreachable.
pub async fn connect_pool(config: &Config) -> Result<PgPool, String> {
    let url = config.database_url();
    PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await
        .map_err(|err| format!("failed to connect to Postgres: {err}"))
}

/// Connect to Redis and fail fast if unreachable.
pub async fn connect_redis(config: &Config) -> Result<ConnectionManager, String> {
    let client = Client::open(config.redis_url())
        .map_err(|err| format!("failed to create Redis client: {err}"))?;
    ConnectionManager::new(client)
        .await
        .map_err(|err| format!("failed to connect to Redis: {err}"))
}

/// Apply embedded sqlx migrations (production / non-development).
pub async fn run_migrations(pool: &PgPool) -> Result<(), String> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|err| format!("migration failed: {err}"))
}

/// Prepare platform schema: SeaQuery synchronize in development, sqlx migrations otherwise.
///
/// Development mirrors TypeORM `synchronize: true` — DDL hash changes recreate platform tables
/// (no backward-compat). Production uses versioned sqlx migrations only.
/// Always ensures tenant roles, `db-pre-request`, and Internal-Context secret.
pub async fn prepare_schema(pool: &PgPool, config: &Config) -> Result<(), String> {
    if config.env == "development" {
        sync::synchronize_platform_schema(pool).await?;
    } else {
        run_migrations(pool).await?;
        sync::ensure_tenant_roles_and_gateway(pool).await?;
    }
    sync::upsert_internal_context_secret(pool, &config.internal_context_secret).await?;
    Ok(())
}

/// Seed a local platform user in development when none exists for the seed email.
pub async fn ensure_dev_seed_user(pool: &PgPool, config: &Config) -> Result<(), String> {
    if config.env != "development" {
        return Ok(());
    }

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND deleted_at IS NULL)",
    )
    .bind(DEV_SEED_EMAIL)
    .fetch_one(pool)
    .await
    .map_err(|err| format!("seed user lookup failed: {err}"))?;

    if exists {
        return Ok(());
    }

    let password_hash = crate::auth::password::hash_password(DEV_SEED_PASSWORD)
        .map_err(|err| format!("seed password hash failed: {err}"))?;
    let user_id = crate::ids::new_ulid();

    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(&user_id)
        .bind(DEV_SEED_EMAIL)
        .bind(&password_hash)
        .execute(pool)
        .await
        .map_err(|err| format!("seed user insert failed: {err}"))?;

    tracing::info!(
        email = DEV_SEED_EMAIL,
        "seeded development platform user (password documented in README / .env.example)"
    );
    Ok(())
}
