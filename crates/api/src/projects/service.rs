use sqlx::{PgPool, Row};

use crate::constants::auth::PROJECT_ROLE_OWNER;
use crate::constants::keys::{KEY_STATUS_ACTIVE, KEY_TYPE_PUBLISHABLE, KEY_TYPE_SECRET};
use crate::error::ApiError;
use crate::ids::new_ulid;
use crate::projects::keys::{
    hash_api_key, key_prefix_for_display, mint_publishable_key, mint_secret_key,
};
use crate::projects::postgrest;

#[derive(Debug, Clone)]
pub struct CreatedProject {
    pub id: String,
    pub name: String,
    pub publishable_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone)]
pub struct ProjectListItem {
    pub id: String,
    pub name: String,
    pub role: String,
}

pub async fn create_project(
    pool: &PgPool,
    config: &crate::config::Config,
    owner_user_id: &str,
    name: &str,
) -> Result<CreatedProject, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }

    let project_id = new_ulid();
    let schema = format!("proj_{project_id}");
    let publishable = mint_publishable_key();
    let secret = mint_secret_key();
    let pub_hash = hash_api_key(&publishable);
    let sec_hash = hash_api_key(&secret);
    let pub_id = new_ulid();
    let sec_id = new_ulid();

    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO projects (id, name) VALUES ($1, $2)")
        .bind(&project_id)
        .bind(name)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO project_members (project_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(&project_id)
        .bind(owner_user_id)
        .bind(PROJECT_ROLE_OWNER)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO api_keys (id, project_id, key_type, key_hash, key_prefix, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&pub_id)
    .bind(&project_id)
    .bind(KEY_TYPE_PUBLISHABLE)
    .bind(&pub_hash)
    .bind(key_prefix_for_display(&publishable))
    .bind(KEY_STATUS_ACTIVE)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO api_keys (id, project_id, key_type, key_hash, key_prefix, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&sec_id)
    .bind(&project_id)
    .bind(KEY_TYPE_SECRET)
    .bind(&sec_hash)
    .bind(key_prefix_for_display(&secret))
    .bind(KEY_STATUS_ACTIVE)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    if let Err(err) = provision_schema(pool, &schema).await {
        tracing::error!(%project_id, %schema, error = %err, "schema provisioning failed; compensating project row");
        let _ = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(&project_id)
            .execute(pool)
            .await;
        return Err(err);
    }

    postgrest::register_schema_and_reload(pool, config, &schema).await?;

    Ok(CreatedProject {
        id: project_id,
        name: name.to_string(),
        publishable_key: publishable,
        secret_key: secret,
    })
}

async fn provision_schema(pool: &PgPool, schema: &str) -> Result<(), ApiError> {
    // Schema name is proj_{ulid} — safe identifier (lowercase alphanumeric + underscore).
    if !is_safe_schema_name(schema) {
        return Err(ApiError::Internal("invalid schema name".into()));
    }

    let create = format!("CREATE SCHEMA {schema}");
    sqlx::query(&create).execute(pool).await?;

    sqlx::query(&format!(
        "GRANT USAGE ON SCHEMA {schema} TO anon, authenticated, project_operator_readonly"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "GRANT ALL ON SCHEMA {schema} TO service, project_operator"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA {schema} GRANT SELECT ON TABLES TO anon, authenticated, project_operator_readonly"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA {schema} GRANT ALL ON TABLES TO service, project_operator"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA {schema} GRANT USAGE, SELECT ON SEQUENCES TO anon, authenticated, project_operator_readonly"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA {schema} GRANT ALL ON SEQUENCES TO service, project_operator"
    ))
    .execute(pool)
    .await?;

    Ok(())
}

fn is_safe_schema_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("proj_") else {
        return false;
    };
    rest.len() == 26
        && rest
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

pub async fn list_projects_for_user(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<ProjectListItem>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT p.id, p.name, pm.role
        FROM projects p
        INNER JOIN project_members pm ON pm.project_id = p.id
        WHERE pm.user_id = $1
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ProjectListItem {
            id: row.get("id"),
            name: row.get("name"),
            role: row.get("role"),
        })
        .collect())
}

pub async fn membership_role(
    pool: &PgPool,
    user_id: &str,
    project_id: &str,
) -> Result<Option<String>, ApiError> {
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM project_members WHERE user_id = $1 AND project_id = $2",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    Ok(role)
}
