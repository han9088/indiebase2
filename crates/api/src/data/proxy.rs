//! Transparent PostgREST HTTP proxy for the Data API gateway.

use axum::body::Body;
use axum::body::Bytes;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::constants::http::HEADER_INTERNAL_CONTEXT_CANONICAL;
use crate::data::auth_matrix::ResolvedDataAuth;
use crate::data::internal_context::sign_internal_context;
use crate::data::jwt::mint_authenticator_jwt;
use crate::error::ApiError;
use crate::state::AppState;

const FORWARD_REQUEST_HEADERS: &[&str] = &[
    "accept",
    "prefer",
    "range",
    "content-type",
    "content-profile",
];

const FORWARD_RESPONSE_HEADERS: &[&str] = &[
    "content-type",
    "content-range",
    "content-location",
    "location",
    "prefer",
    "preference-applied",
];

pub async fn proxy_to_postgrest(
    state: &AppState,
    method: &Method,
    query: Option<&str>,
    body: Bytes,
    inbound_headers: &HeaderMap,
    auth: &ResolvedDataAuth,
) -> Result<Response, ApiError> {
    let schema = format!("proj_{}", auth.project_id);
    let context = sign_internal_context(
        &state.config.internal_context_secret,
        &auth.auth_mode,
        &auth.project_id,
        auth.user_id.as_deref(),
        auth.project_role.as_deref(),
    )?;
    let jwt = mint_authenticator_jwt(&state.config.postgrest_jwt_secret)?;

    let mut url = format!(
        "{}/{}",
        state.config.postgrest_url.trim_end_matches('/'),
        auth.rest_path.trim_start_matches('/')
    );
    if let Some(q) = query.filter(|q| !q.is_empty()) {
        url.push('?');
        url.push_str(q);
    }

    let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes())
        .map_err(|err| ApiError::Internal(format!("unsupported method for proxy: {err}")))?;

    let mut req = state
        .http
        .request(reqwest_method, &url)
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {jwt}"))
        .header(HEADER_INTERNAL_CONTEXT_CANONICAL, &context)
        .header("Accept-Profile", &schema)
        .header("Content-Profile", &schema);

    for name in FORWARD_REQUEST_HEADERS {
        if let Some(value) = inbound_headers.get(*name) {
            if let Ok(v) = value.to_str() {
                req = req.header(*name, v);
            }
        }
    }

    if !body.is_empty() {
        req = req.body(body.to_vec());
    }

    let upstream = req.send().await.map_err(|err| {
        tracing::error!(error = %err, "PostgREST proxy request failed");
        ApiError::Internal("PostgREST proxy failed".into())
    })?;

    let status =
        StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let upstream_headers = upstream.headers().clone();
    let bytes = upstream
        .bytes()
        .await
        .map_err(|err| ApiError::Internal(format!("failed to read PostgREST response: {err}")))?;

    let mut response = Response::builder().status(status);
    let headers = response.headers_mut().expect("response headers");
    for name in FORWARD_RESPONSE_HEADERS {
        if let Some(value) = upstream_headers.get(*name) {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_bytes(value.as_bytes()),
            ) {
                headers.insert(hn, hv);
            }
        }
    }

    response
        .body(Body::from(bytes))
        .map_err(|err| ApiError::Internal(format!("proxy response build failed: {err}")))
}

#[allow(dead_code)]
pub fn proxy_error_response(err: ApiError) -> Response {
    err.into_response()
}
