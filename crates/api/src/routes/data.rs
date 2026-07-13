use axum::body::{to_bytes, Body};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method};
use axum::response::Response;

use crate::data::auth_matrix::resolve_data_auth;
use crate::data::proxy::proxy_to_postgrest;
use crate::error::ApiError;
use crate::state::AppState;

/// Max request body forwarded to PostgREST (16 MiB).
const MAX_PROXY_BODY_BYTES: usize = 16 * 1024 * 1024;

#[utoipa::path(
    get,
    path = "/api/data/{project_id}/{table}",
    operation_id = "data_api_sdk_get",
    summary = "Data API (SDK path)",
    description = "Proxy tenant CRUD via PostgREST on the SDK path.\n\n\
        **Required:** `X-Indiebase-Api-Key` (Publishable or Secret) bound to `{project_id}`.\n\
        **Optional:** `Authorization: Bearer` App User Session (`app_user_session:`) for \
        `auth_mode=authenticated`.\n\n\
        Dashboard Session and mismatched keys return **403**. Client credentials are not forwarded \
        to PostgREST; the gateway injects `Accept-Profile` and signed Internal-Context.",
    params(
        ("project_id" = String, Path, description = "Project ULID (26-char lowercase)"),
        ("table" = String, Path, description = "Tenant table / PostgREST resource name"),
    ),
    responses(
        (status = 200, description = "Proxied PostgREST success (body depends on resource)"),
        (status = 401, description = "Missing or invalid API Key / App User Session"),
        (status = 403, description = "Credential mutual exclusion or project mismatch"),
        (status = 404, description = "Upstream PostgREST not found")
    ),
    tag = "data-api"
)]
pub async fn data_api_sdk_get_docs() {}

#[utoipa::path(
    get,
    path = "/api/data/{table}",
    operation_id = "data_api_dashboard_get",
    summary = "Data API (Dashboard path)",
    description = "Proxy tenant CRUD via PostgREST on the Dashboard Row Viewer path.\n\n\
        **Required:** Dashboard Session Bearer + `X-Indiebase-Project-Id`.\n\
        **Forbidden:** any `X-Indiebase-Api-Key` (403).\n\n\
        `owner`/`admin` → `project_operator`; `member` → `project_operator_readonly` (writes rejected).",
    params(
        ("table" = String, Path, description = "Tenant table / PostgREST resource name"),
        ("X-Indiebase-Project-Id" = String, Header, description = "Project ULID for this request"),
    ),
    responses(
        (status = 200, description = "Proxied PostgREST success"),
        (status = 401, description = "Missing or invalid Dashboard Session"),
        (status = 403, description = "Not a member, member write, or API Key present"),
        (status = 404, description = "Upstream PostgREST not found")
    ),
    security(("bearer_auth" = [])),
    tag = "data-api"
)]
pub async fn data_api_dashboard_get_docs() {}

/// Dual-path Data API proxy: `/api/data/{*path}`.
pub async fn proxy_data(
    State(state): State<AppState>,
    method: Method,
    Path(path): Path<String>,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Body,
) -> Result<Response, ApiError> {
    let auth = resolve_data_auth(&state, &method, &headers, &path).await?;

    let collected = to_bytes(body, MAX_PROXY_BODY_BYTES)
        .await
        .map_err(|err| ApiError::BadRequest(format!("request body too large or invalid: {err}")))?;

    proxy_to_postgrest(&state, &method, uri.query(), collected, &headers, &auth).await
}
