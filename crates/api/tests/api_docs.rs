use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn openapi_json_returns_spec_with_health_path() {
    let response = api::app::router()
        .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("application/json"),
        "expected application/json, got {content_type}"
    );

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should be readable")
        .to_bytes();
    let spec: serde_json::Value =
        serde_json::from_slice(&body).expect("body should be valid openapi json");

    assert!(
        spec.get("openapi").is_some(),
        "missing openapi version field"
    );
    assert!(spec.get("paths").is_some(), "missing paths field");
    assert!(
        spec["paths"].get("/health").is_some(),
        "openapi spec should document /health"
    );

    let health_schema = &spec["components"]["schemas"]["HealthResponse"];
    assert_eq!(
        health_schema["properties"]["status"]["type"].as_str(),
        Some("string"),
        "HealthResponse schema should document status string field"
    );
}

#[tokio::test]
async fn docs_returns_scalar_html() {
    let response = api::app::router()
        .oneshot(Request::get("/docs").body(Body::empty()).unwrap())
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("text/html"),
        "expected text/html, got {content_type}"
    );

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should be readable")
        .to_bytes();
    let html = String::from_utf8(body.to_vec()).expect("docs body should be utf-8");
    assert!(
        html.contains("scalar") || html.contains("Scalar"),
        "docs page should reference Scalar"
    );
}
