pub mod auth;
pub mod health;
pub mod projects;

use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::assets;
use crate::openapi::{self, ApiDoc};
use crate::state::AppState;

pub fn health_routes() -> Router {
    Router::new().route("/health", get(health::health))
}

pub fn docs_routes() -> Router {
    Router::new()
        .route("/openapi.json", get(openapi::serve_openapi))
        .route("/favicon.ico", get(assets::favicon))
        .route("/logo.svg", get(assets::logo_svg))
        .route("/logo.png", get(assets::logo_png))
        .merge(
            Scalar::with_url("/docs", ApiDoc::openapi())
                .custom_html(include_str!("../../assets/scalar.html")),
        )
}

/// Manager API routes under `/api` (Dashboard Session / Project Session).
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/project/login", post(auth::project_login))
        .route("/auth/project/logout", post(auth::project_logout))
        .route("/projects", get(projects::list).post(projects::create))
}
