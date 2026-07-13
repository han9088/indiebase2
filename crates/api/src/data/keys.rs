//! API Key lookup for the Data API SDK path.

use sqlx::PgPool;

use crate::constants::keys::KEY_STATUS_ACTIVE;
use crate::error::ApiError;
use crate::projects::keys::hash_api_key;

#[derive(Debug, Clone)]
pub struct ResolvedApiKey {
    pub project_id: String,
    pub key_type: String,
}

pub async fn lookup_active_api_key(
    pool: &PgPool,
    plaintext: &str,
) -> Result<Option<ResolvedApiKey>, ApiError> {
    let hash = hash_api_key(plaintext);
    let row = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT project_id, key_type
        FROM api_keys
        WHERE key_hash = $1
          AND status = $2
          AND deleted_at IS NULL
        "#,
    )
    .bind(&hash)
    .bind(KEY_STATUS_ACTIVE)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(project_id, key_type)| ResolvedApiKey {
        project_id,
        key_type,
    }))
}
