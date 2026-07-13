//! Development schema synchronize (TypeORM-like `synchronize: true`).
//!
//! When `INDIEBASE_ENV=development`, platform DDL is driven by SeaQuery definitions.
//! If the rendered DDL hash changes, tables are dropped and recreated (dev-only; no compat).

use sha2::{Digest, Sha256};
use sqlx::PgPool;

use super::gateway_sql::{self, gateway_sql_hash_input};
use super::schema::{platform_table_statements, TENANT_ROLES_SQL};

const SYNC_META_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS public._indiebase_schema_sync (
    id INT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    schema_hash TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
)
"#;

fn schema_hash(statements: &[String]) -> String {
    let mut hasher = Sha256::new();
    for stmt in statements {
        hasher.update(stmt.as_bytes());
        hasher.update(b"\n");
    }
    hasher.update(TENANT_ROLES_SQL.as_bytes());
    hasher.update(gateway_sql_hash_input().as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn ensure_tenant_roles_and_gateway(pool: &PgPool) -> Result<(), String> {
    sqlx::query(TENANT_ROLES_SQL)
        .execute(pool)
        .await
        .map_err(|err| format!("tenant roles ensure failed: {err}"))?;
    gateway_sql::apply_data_api_gateway_sql(pool).await?;
    Ok(())
}

/// Upsert the Internal-Context HMAC secret for `indiebase_pre_request`.
pub async fn upsert_internal_context_secret(pool: &PgPool, secret: &str) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO public.gateway_config (key, value, updated_at)
        VALUES ('internal_context_secret', $1, now())
        ON CONFLICT (key) DO UPDATE
        SET value = EXCLUDED.value,
            updated_at = now()
        "#,
    )
    .bind(secret)
    .execute(pool)
    .await
    .map_err(|err| format!("gateway_config secret upsert failed: {err}"))?;
    Ok(())
}

/// Recreate platform tables from SeaQuery when the DDL hash changes.
pub async fn synchronize_platform_schema(pool: &PgPool) -> Result<(), String> {
    let statements = platform_table_statements();
    let hash = schema_hash(&statements);

    sqlx::query(SYNC_META_SQL)
        .execute(pool)
        .await
        .map_err(|err| format!("schema sync meta table failed: {err}"))?;

    let current: Option<String> =
        sqlx::query_scalar("SELECT schema_hash FROM public._indiebase_schema_sync WHERE id = 1")
            .fetch_optional(pool)
            .await
            .map_err(|err| format!("schema sync hash lookup failed: {err}"))?;

    if current.as_deref() == Some(hash.as_str()) {
        tracing::debug!(%hash, "platform schema already synchronized");
        ensure_tenant_roles_and_gateway(pool).await?;
        return Ok(());
    }

    tracing::info!(
        previous = current.as_deref().unwrap_or("(none)"),
        %hash,
        "development schema synchronize: recreating platform tables"
    );

    for table in ["api_keys", "project_members", "projects", "users"] {
        let sql = format!("DROP TABLE IF EXISTS public.{table} CASCADE");
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|err| format!("drop {table} failed: {err}"))?;
    }

    for stmt in &statements {
        sqlx::query(stmt)
            .execute(pool)
            .await
            .map_err(|err| format!("schema sync apply failed: {err}\nSQL: {stmt}"))?;
    }

    ensure_tenant_roles_and_gateway(pool).await?;

    sqlx::query(
        r#"
        INSERT INTO public._indiebase_schema_sync (id, schema_hash, updated_at)
        VALUES (1, $1, now())
        ON CONFLICT (id) DO UPDATE
        SET schema_hash = EXCLUDED.schema_hash,
            updated_at = now()
        "#,
    )
    .bind(&hash)
    .execute(pool)
    .await
    .map_err(|err| format!("schema sync hash upsert failed: {err}"))?;

    Ok(())
}
