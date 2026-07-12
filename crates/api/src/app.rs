use axum::Router;

use crate::routes;
use crate::state::AppState;

/// System routes that do not require AppState (health + docs).
pub fn system_router() -> Router {
    Router::new()
        .merge(routes::health_routes())
        .merge(routes::docs_routes())
}

/// Full application router with Manager API nested at `/api`.
pub fn router(state: AppState) -> Router {
    system_router().nest("/api", routes::api_routes().with_state(state))
}
