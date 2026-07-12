use axum::Json;
use utoipa::OpenApi;

use crate::routes::health::HealthResponse;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Indiebase API",
        version = "0.1.0",
        description = "Self-hosted BaaS — Manager API and Data API (MVP in progress)."
    ),
    paths(crate::routes::health::health),
    components(schemas(HealthResponse))
)]
pub struct ApiDoc;

pub async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
