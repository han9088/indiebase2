use axum::Json;
use serde_json::json;
use utoipa::openapi::extensions::Extensions;
use utoipa::{Modify, OpenApi};

use crate::routes::health::HealthResponse;

struct IndiebaseLogo;

impl Modify for IndiebaseLogo {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.info.extensions = Some(Extensions::from_iter([(
            "x-logo",
            json!({
                "url": "/logo.svg",
                "altText": "Indiebase",
                "href": "https://indiebase.deskbtm.com"
            }),
        )]));
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Indiebase API",
        version = "0.1.0",
        description = "Self-hosted BaaS — Manager API and Data API (MVP in progress)."
    ),
    modifiers(&IndiebaseLogo),
    paths(crate::routes::health::health),
    components(schemas(HealthResponse))
)]
pub struct ApiDoc;

pub async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
