use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::extractors::{DashboardAuth, ProjectAuth};
use crate::auth::password::verify_password;
use crate::auth::session::{
    delete_dashboard_session, delete_project_session, mint_opaque_token, store_dashboard_session,
    store_project_session,
};
use crate::error::ApiError;
use crate::projects::service::membership_role;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProjectLoginRequest {
    pub project_id: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Dashboard session created", body = TokenResponse),
        (status = 401, description = "Invalid credentials")
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

    let row: Option<(String, String)> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE email = $1")
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
    responses(
        (status = 200, description = "Dashboard session revoked"),
        (status = 401, description = "Missing or invalid session")
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    auth: DashboardAuth,
) -> Result<Json<serde_json::Value>, ApiError> {
    delete_dashboard_session(&state, &auth.token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[utoipa::path(
    post,
    path = "/api/auth/project/login",
    request_body = ProjectLoginRequest,
    responses(
        (status = 200, description = "Project session created", body = TokenResponse),
        (status = 401, description = "Missing dashboard session"),
        (status = 403, description = "Not a project member")
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn project_login(
    State(state): State<AppState>,
    auth: DashboardAuth,
    Json(body): Json<ProjectLoginRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let project_id = body.project_id.trim();
    if project_id.is_empty() {
        return Err(ApiError::BadRequest("project_id is required".into()));
    }

    let role = membership_role(&state.pool, &auth.session.user_id, project_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("not a member of this project".into()))?;

    let token = mint_opaque_token();
    store_project_session(&state, &token, &auth.session.user_id, project_id, &role).await?;

    Ok(Json(TokenResponse {
        token,
        token_type: "Bearer",
        expires_in: state.config.session_ttl_secs,
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/project/logout",
    responses(
        (status = 200, description = "Project session revoked"),
        (status = 401, description = "Missing or invalid project session")
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn project_logout(
    State(state): State<AppState>,
    auth: ProjectAuth,
) -> Result<Json<serde_json::Value>, ApiError> {
    delete_project_session(&state, &auth.token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
