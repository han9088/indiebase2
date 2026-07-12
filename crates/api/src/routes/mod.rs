pub mod health;

use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::assets;
use crate::openapi::{self, ApiDoc};

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
