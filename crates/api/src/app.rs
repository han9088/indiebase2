use axum::Router;

use crate::routes;

pub fn router() -> Router {
    Router::new()
        .merge(routes::health_routes())
        .merge(routes::docs_routes())
    // Phase 1+: .nest("/api", manager_api::router())
    // Phase 2+: Data API gateway routes under /api/data
}
