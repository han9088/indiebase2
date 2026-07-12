pub mod health;

use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::openapi::{self, ApiDoc};

pub fn health_routes() -> Router {
    Router::new().route("/health", get(health::health))
}

pub fn docs_routes() -> Router {
    Router::new()
        .route("/openapi.json", get(openapi::serve_openapi))
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
}
