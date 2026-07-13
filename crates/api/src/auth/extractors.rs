use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::auth::session::{load_dashboard_session, DashboardSessionData};
use crate::constants::http::HEADER_PROJECT_ID;
use crate::error::ApiError;
use crate::projects::service::membership_role;
use crate::state::AppState;

/// Authenticated Dashboard Session from `Authorization: Bearer`.
#[derive(Debug, Clone)]
pub struct DashboardAuth {
    pub token: String,
    pub session: DashboardSessionData,
}

/// Dashboard Session + project context from `X-Indiebase-Project-Id`.
///
/// Used for project-scoped Manager / Data API routes that do not put `project_id` in the URL.
/// Membership is resolved per request via `project_members` (no separate Project Session).
#[derive(Debug, Clone)]
pub struct ProjectScopedAuth {
    pub token: String,
    pub session: DashboardSessionData,
    pub project_id: String,
    pub project_role: String,
}

fn extract_bearer(parts: &Parts) -> Result<&str, ApiError> {
    let header = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("missing Authorization header".into()))?;

    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or_else(|| ApiError::Unauthorized("expected Bearer token".into()))?;

    Ok(token)
}

fn extract_project_id_header(parts: &Parts) -> Result<&str, ApiError> {
    parts
        .headers
        .get(HEADER_PROJECT_ID)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::BadRequest("missing X-Indiebase-Project-Id header".into()))
}

impl FromRequestParts<AppState> for DashboardAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer(parts)?.to_string();
        let session = load_dashboard_session(state, &token)
            .await?
            .ok_or_else(|| ApiError::Unauthorized("invalid or expired dashboard session".into()))?;
        Ok(Self { token, session })
    }
}

impl FromRequestParts<AppState> for ProjectScopedAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer(parts)?.to_string();
        let session = load_dashboard_session(state, &token)
            .await?
            .ok_or_else(|| ApiError::Unauthorized("invalid or expired dashboard session".into()))?;

        let project_id = extract_project_id_header(parts)?.to_string();
        let project_role = membership_role(&state.pool, &session.user_id, &project_id)
            .await?
            .ok_or_else(|| ApiError::Forbidden("not a member of this project".into()))?;

        Ok(Self {
            token,
            session,
            project_id,
            project_role,
        })
    }
}
