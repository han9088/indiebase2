//! Dual-path credential resolution for the Data API gateway (§6.2.3).

use axum::http::{HeaderMap, Method};
use redis::AsyncCommands;
use serde::Deserialize;

use crate::auth::session::load_dashboard_session;
use crate::constants::auth::{PROJECT_ROLE_ADMIN, PROJECT_ROLE_MEMBER, PROJECT_ROLE_OWNER};
use crate::constants::data_api::{
    AUTH_MODE_ANON, AUTH_MODE_AUTHENTICATED, AUTH_MODE_PROJECT_OPERATOR,
    AUTH_MODE_PROJECT_OPERATOR_READONLY, AUTH_MODE_SERVICE,
};
use crate::constants::http::HEADER_API_KEY;
use crate::constants::keys::{KEY_TYPE_PUBLISHABLE, KEY_TYPE_SECRET};
use crate::constants::session::APP_USER_SESSION_PREFIX;
use crate::data::keys::lookup_active_api_key;
use crate::error::ApiError;
use crate::projects::service::membership_role;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct ResolvedDataAuth {
    pub project_id: String,
    pub auth_mode: String,
    pub user_id: Option<String>,
    pub project_role: Option<String>,
    /// Remaining PostgREST path after stripping `/api/data` or `/api/data/{project_id}`.
    pub rest_path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AppUserSessionData {
    pub end_user_id: String,
    pub project_id: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub exp: u64,
}

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|t| !t.is_empty())
}

fn extract_api_key(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(HEADER_API_KEY)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn extract_project_id_header(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get(crate::constants::http::HEADER_PROJECT_ID)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::BadRequest("missing X-Indiebase-Project-Id header".into()))
}

fn is_write_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

async fn load_app_user_session(
    state: &AppState,
    token: &str,
) -> Result<Option<AppUserSessionData>, ApiError> {
    let key = format!("{APP_USER_SESSION_PREFIX}{token}");
    let mut redis = state.redis.clone();
    let value: Option<String> = redis.get(key).await?;
    match value {
        Some(raw) => Ok(Some(serde_json::from_str(&raw)?)),
        None => Ok(None),
    }
}

/// Resolve auth for a request whose path under `/api/data/` is `path` (no leading slash required).
pub async fn resolve_data_auth(
    state: &AppState,
    method: &Method,
    headers: &HeaderMap,
    path: &str,
) -> Result<ResolvedDataAuth, ApiError> {
    let path = path.trim_start_matches('/');
    let (first, rest_after_first) = match path.split_once('/') {
        Some((a, b)) => (a, b),
        None => (path, ""),
    };

    if crate::data::ulid_path::is_project_ulid(first) {
        resolve_sdk_path(state, headers, first, rest_after_first).await
    } else {
        resolve_dashboard_path(state, method, headers, path).await
    }
}

async fn resolve_sdk_path(
    state: &AppState,
    headers: &HeaderMap,
    project_id: &str,
    rest_path: &str,
) -> Result<ResolvedDataAuth, ApiError> {
    let api_key = extract_api_key(headers)
        .ok_or_else(|| ApiError::Unauthorized("missing X-Indiebase-Api-Key".into()))?;

    if let Some(bearer) = extract_bearer(headers) {
        // SDK path must never accept Dashboard Session (§6.2.3).
        if load_dashboard_session(state, bearer).await?.is_some() {
            return Err(ApiError::Forbidden(
                "Dashboard Session cannot be used on SDK Data API path".into(),
            ));
        }
    }

    let key = lookup_active_api_key(&state.pool, api_key)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid API key".into()))?;

    if key.project_id != project_id {
        return Err(ApiError::Forbidden(
            "API key is not bound to this project".into(),
        ));
    }

    if key.key_type == KEY_TYPE_SECRET {
        tracing::info!(%project_id, "data API secret key request");
        return Ok(ResolvedDataAuth {
            project_id: project_id.to_string(),
            auth_mode: AUTH_MODE_SERVICE.to_string(),
            user_id: None,
            project_role: None,
            rest_path: rest_path.to_string(),
        });
    }

    if key.key_type != KEY_TYPE_PUBLISHABLE {
        return Err(ApiError::Forbidden("unsupported API key type".into()));
    }

    let mut auth_mode = AUTH_MODE_ANON.to_string();
    let mut user_id = None;

    if let Some(bearer) = extract_bearer(headers) {
        match load_app_user_session(state, bearer).await? {
            Some(session) => {
                if session.project_id != project_id {
                    return Err(ApiError::Forbidden(
                        "App User Session project mismatch".into(),
                    ));
                }
                auth_mode = AUTH_MODE_AUTHENTICATED.to_string();
                user_id = Some(session.end_user_id);
                let _ = session.role;
            }
            None => {
                return Err(ApiError::Unauthorized(
                    "invalid or expired App User Session".into(),
                ));
            }
        }
    }

    Ok(ResolvedDataAuth {
        project_id: project_id.to_string(),
        auth_mode,
        user_id,
        project_role: None,
        rest_path: rest_path.to_string(),
    })
}

async fn resolve_dashboard_path(
    state: &AppState,
    method: &Method,
    headers: &HeaderMap,
    rest_path: &str,
) -> Result<ResolvedDataAuth, ApiError> {
    if extract_api_key(headers).is_some() {
        return Err(ApiError::Forbidden(
            "X-Indiebase-Api-Key cannot be used on Dashboard Data API path".into(),
        ));
    }

    let bearer = extract_bearer(headers)
        .ok_or_else(|| ApiError::Unauthorized("missing Authorization header".into()))?;

    let session = load_dashboard_session(state, bearer)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("invalid or expired dashboard session".into()))?;

    let project_id = extract_project_id_header(headers)?.to_string();
    let project_role = membership_role(&state.pool, &session.user_id, &project_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("not a member of this project".into()))?;

    let auth_mode = match project_role.as_str() {
        PROJECT_ROLE_OWNER | PROJECT_ROLE_ADMIN => AUTH_MODE_PROJECT_OPERATOR,
        PROJECT_ROLE_MEMBER => AUTH_MODE_PROJECT_OPERATOR_READONLY,
        _ => {
            return Err(ApiError::Forbidden("unknown project role".into()));
        }
    };

    if auth_mode == AUTH_MODE_PROJECT_OPERATOR_READONLY && is_write_method(method) {
        return Err(ApiError::Forbidden(
            "project member cannot write via Data API".into(),
        ));
    }

    Ok(ResolvedDataAuth {
        project_id,
        auth_mode: auth_mode.to_string(),
        user_id: Some(session.user_id),
        project_role: Some(project_role),
        rest_path: rest_path.to_string(),
    })
}
