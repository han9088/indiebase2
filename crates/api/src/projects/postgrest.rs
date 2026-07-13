//! Reload PostgREST schema list from the database (no static schema files).
//!
//! `public.indiebase_pre_config` (PostgREST `db-pre-config`) runs
//! `set_config('pgrst.db_schemas', …)` from `public.projects`. This module only NOTIFYs reload.

use sqlx::PgPool;

use crate::error::ApiError;

/// Ask PostgREST to re-run in-DB config (`indiebase_pre_config`) and refresh schema cache.
pub async fn sync_schemas_from_projects_and_reload(pool: &PgPool) -> Result<(), ApiError> {
    sqlx::query("SELECT pg_notify('pgrst', 'reload config')")
        .execute(pool)
        .await?;
    sqlx::query("SELECT pg_notify('pgrst', 'reload schema')")
        .execute(pool)
        .await?;
    Ok(())
}
