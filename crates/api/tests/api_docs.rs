use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn openapi_json_returns_spec_with_health_path() {
    let response = api::app::system_router()
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
    assert!(
        spec["paths"].get("/api/auth/login").is_some(),
        "openapi spec should document /api/auth/login"
    );
    assert!(
        spec["paths"].get("/api/projects").is_some(),
        "openapi spec should document /api/projects"
    );

    assert_eq!(
        spec["info"]["title"].as_str(),
        Some("Indiebase Manager API"),
        "OpenAPI info.title should be set"
    );
    assert!(
        spec["paths"]["/api/auth/login"]["post"]["summary"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "login path should have a summary"
    );
    assert_eq!(
        spec["paths"]["/api/auth/login"]["post"]["operationId"].as_str(),
        Some("auth_login")
    );

    let health_schema = &spec["components"]["schemas"]["HealthResponse"];
    assert_eq!(
        health_schema["properties"]["status"]["type"].as_str(),
        Some("string"),
        "HealthResponse schema should document status string field"
    );
    assert!(
        health_schema["properties"]["status"]
            .get("description")
            .and_then(|d| d.as_str())
            .is_some_and(|d| !d.is_empty()),
        "schema fields should include descriptions"
    );
    let login_schema = &spec["components"]["schemas"]["LoginRequest"];
    assert_eq!(
        login_schema["properties"]["email"]["example"].as_str(),
        Some("dev@indiebase.com")
    );
}

#[tokio::test]
async fn docs_returns_scalar_html() {
    let response = api::app::system_router()
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
    assert!(
        html.contains("/favicon.ico"),
        "docs page should reference favicon"
    );
    assert!(
        html.contains("/logo.svg"),
        "docs page should reference logo"
    );
}

#[tokio::test]
async fn serves_favicon() {
    let response = api::app::system_router()
        .oneshot(Request::get("/favicon.ico").body(Body::empty()).unwrap())
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("image/") || content_type.contains("icon"),
        "expected image content-type, got {content_type}"
    );
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    assert!(!body.is_empty(), "favicon should not be empty");
}

#[tokio::test]
async fn serves_logo_svg() {
    let response = api::app::system_router()
        .oneshot(Request::get("/logo.svg").body(Body::empty()).unwrap())
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("svg") || content_type.contains("image/"),
        "expected svg content-type, got {content_type}"
    );
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    assert!(!body.is_empty(), "logo.svg should not be empty");
}

#[tokio::test]
async fn openapi_includes_x_logo() {
    let response = api::app::system_router()
        .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
        .await
        .expect("router should respond");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let spec: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let logo = &spec["info"]["x-logo"];
    assert_eq!(logo["url"].as_str(), Some("/logo.svg"));
    assert_eq!(logo["altText"].as_str(), Some("Indiebase"));
}
