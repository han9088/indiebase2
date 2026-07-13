use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::extractors::{DashboardAuth, ProjectScopedAuth};
use crate::auth::password::verify_password;
use crate::auth::session::{delete_dashboard_session, mint_opaque_token, store_dashboard_session};
use crate::error::ApiError;
use crate::state::AppState;

/// Credentials for creating a Dashboard Session.
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"email":"dev@indiebase.com","password":"dev@indiebase.com"}))]
pub struct LoginRequest {
    /// Platform user email. Trimmed and lowercased before lookup.
    #[schema(example = "dev@indiebase.com", format = "email")]
    pub email: String,
    /// Account password (never logged). Dev seed uses the same value as the email.
    #[schema(example = "dev@indiebase.com", format = "password")]
    pub password: String,
}

/// Opaque session token payload returned by login.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "token": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
    "token_type": "Bearer",
    "expires_in": 86400
}))]
pub struct TokenResponse {
    /// Opaque Dashboard Session token. Send as `Authorization: Bearer <token>`.
    #[schema(example = "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456")]
    pub token: String,
    /// Always `Bearer` for HTTP Authorization scheme compatibility.
    #[schema(example = "Bearer")]
    pub token_type: &'static str,
    /// Token lifetime in seconds (from `SESSION_TTL_SECS`, default 86400).
    #[schema(example = 86400)]
    pub expires_in: u64,
}

/// Generic success body for logout and similar mutations.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({"ok": true}))]
pub struct OkResponse {
    /// Always `true` on success.
    #[schema(example = true)]
    pub ok: bool,
}

/// Resolved project context for the current Dashboard user (via header).
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "project_id": "01jcqz4sxf7k2m8n3p5r6t9vwx",
    "role": "owner"
}))]
pub struct ProjectContextResponse {
    /// Project ULID from `X-Indiebase-Project-Id`.
    #[schema(example = "01jcqz4sxf7k2m8n3p5r6t9vwx")]
    pub project_id: String,
    /// Caller's role: `owner` | `admin` | `member`.
    #[schema(example = "owner")]
    pub role: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    operation_id = "auth_login",
    summary = "Create Dashboard Session",
    description = "Authenticate a platform user with email + password and mint an **Opaque Token** \
        stored in Redis (`dashboard_session:`). Use the returned token as \
        `Authorization: Bearer <token>` on Manager and Dashboard Data API routes.\n\n\
        Project context is **not** in this token. For project-scoped routes send \
        `X-Indiebase-Project-Id` on each request; membership is checked via `project_members`.\n\n\
        Soft-deleted users cannot log in. Invalid credentials always return 401.",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Dashboard Session created", body = TokenResponse),
        (status = 401, description = "Invalid email or password")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let email = body.email.trim().to_ascii_lowercase();
    if email.is_empty() || body.password.is_empty() {
        return Err(ApiError::Unauthorized("invalid credentials".into()));
    }

    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE email = $1 AND deleted_at IS NULL",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await?;

    let Some((user_id, password_hash)) = row else {
        return Err(ApiError::Unauthorized("invalid credentials".into()));
    };

    let ok = verify_password(&body.password, &password_hash).map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::Unauthorized("invalid credentials".into()));
    }

    let token = mint_opaque_token();
    store_dashboard_session(&state, &token, &user_id).await?;

    Ok(Json(TokenResponse {
        token,
        token_type: "Bearer",
        expires_in: state.config.session_ttl_secs,
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    operation_id = "auth_logout",
    summary = "Revoke Dashboard Session",
    description = "Deletes the current Dashboard Session from Redis.",
    responses(
        (status = 200, description = "Session revoked", body = OkResponse),
        (status = 401, description = "Missing or invalid Dashboard Session")
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    auth: DashboardAuth,
) -> Result<Json<OkResponse>, ApiError> {
    delete_dashboard_session(&state, &auth.token).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/auth/project-context",
    operation_id = "auth_project_context",
    summary = "Resolve project context",
    description = "Validates the Dashboard Session and `X-Indiebase-Project-Id` header, then returns \
        the caller's `project_role` from `project_members`. Used by Dashboard clients and as the \
        pattern for future Data API (`/api/data/*`) authorization.\n\n\
        No second login / Project Session — same Bearer token as Manager API.",
    responses(
        (status = 200, description = "Caller is a member; role returned", body = ProjectContextResponse),
        (status = 400, description = "Missing `X-Indiebase-Project-Id`"),
        (status = 401, description = "Missing or invalid Dashboard Session"),
        (status = 403, description = "Not a member of this project")
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn project_context(
    auth: ProjectScopedAuth,
) -> Result<Json<ProjectContextResponse>, ApiError> {
    Ok(Json(ProjectContextResponse {
        project_id: auth.project_id,
        role: auth.project_role,
    }))
}
