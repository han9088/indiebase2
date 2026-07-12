use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::auth::session::{
    load_dashboard_session, load_project_session, DashboardSessionData, ProjectSessionData,
};
use crate::error::ApiError;
use crate::state::AppState;

/// Authenticated Dashboard Session extracted from `Authorization: Bearer`.
#[derive(Debug, Clone)]
pub struct DashboardAuth {
    pub token: String,
    pub session: DashboardSessionData,
}

/// Authenticated Project Session extracted from `Authorization: Bearer`.
#[derive(Debug, Clone)]
pub struct ProjectAuth {
    pub token: String,
    pub session: ProjectSessionData,
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

impl FromRequestParts<AppState> for ProjectAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer(parts)?.to_string();
        let session = load_project_session(state, &token)
            .await?
            .ok_or_else(|| ApiError::Unauthorized("invalid or expired project session".into()))?;
        Ok(Self { token, session })
    }
}
