use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Process liveness payload.
#[derive(Serialize, ToSchema)]
#[schema(example = json!({"status":"ok"}))]
pub struct HealthResponse {
    /// Always `ok` when the process is accepting HTTP.
    #[schema(example = "ok")]
    pub status: &'static str,
}

#[utoipa::path(
    get,
    path = "/health",
    operation_id = "system_health",
    summary = "Liveness check",
    description = "Returns `{ \"status\": \"ok\" }` when the API process is up. Does not verify \
        Postgres, Redis, or PostgREST connectivity.",
    responses(
        (status = 200, description = "Process is accepting requests", body = HealthResponse)
    ),
    tag = "system"
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::app;

    #[tokio::test]
    async fn health_returns_ok_json() {
        let response = app::system_router()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .expect("router should respond");

        assert_eq!(response.status(), http::StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should be readable")
            .to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("body should be valid json");
        assert_eq!(json["status"], "ok");
    }
}
