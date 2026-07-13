//! Integration tests for Manager auth + project lifecycle.
//! Skips when Postgres/Redis from `.env.development` are unreachable.

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

async fn try_state() -> Option<api::state::AppState> {
    // `cargo test` uses the crate dir as CWD; Vite-style dotenv files live at the workspace root.
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let _ = std::env::set_current_dir(&workspace_root);

    let config = api::config::Config::from_env().ok()?;
    let pool = api::db::connect_pool(&config).await.ok()?;
    let redis = api::db::connect_redis(&config).await.ok()?;
    api::db::prepare_schema(&pool, &config).await.ok()?;
    api::db::ensure_dev_seed_user(&pool, &config).await.ok()?;
    Some(api::state::AppState::new(pool, redis, config))
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    serde_json::from_slice(&body)
        .unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&body) }))
}

async fn login_token(app: &axum::Router) -> String {
    let login = app
        .clone()
        .oneshot(
            Request::post("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "dev@indiebase.com",
                        "password": "dev@indiebase.com"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("login");
    assert_eq!(login.status(), StatusCode::OK);
    json_body(login).await["token"]
        .as_str()
        .expect("token")
        .to_string()
}

#[tokio::test]
async fn login_success_and_invalid_credentials() {
    let Some(state) = try_state().await else {
        eprintln!("skipping login_success_and_invalid_credentials: Postgres/Redis unavailable");
        return;
    };
    let app = api::app::router(state);

    let bad = app
        .clone()
        .oneshot(
            Request::post("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "dev@indiebase.com",
                        "password": "wrong-password"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);

    let ok = app
        .oneshot(
            Request::post("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "dev@indiebase.com",
                        "password": "dev@indiebase.com"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(ok.status(), StatusCode::OK);
    let body = json_body(ok).await;
    assert!(body["token"].as_str().unwrap_or("").len() >= 32);
    assert_eq!(body["token_type"].as_str(), Some("Bearer"));
}

#[tokio::test]
async fn project_context_forbidden_for_non_member() {
    let Some(state) = try_state().await else {
        eprintln!("skipping project_context_forbidden_for_non_member: Postgres/Redis unavailable");
        return;
    };
    let app = api::app::router(state);
    let token = login_token(&app).await;

    let forbidden = app
        .oneshot(
            Request::get("/api/auth/project-context")
                .header("authorization", format!("Bearer {token}"))
                .header("x-indiebase-project-id", "01AAAAAAAAAAAAAAAAAAAAAAAA")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("project context");
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_project_provisions_schema_keys_and_project_context() {
    let Some(state) = try_state().await else {
        eprintln!(
            "skipping create_project_provisions_schema_keys_and_project_context: infra unavailable"
        );
        return;
    };
    let pool = state.pool.clone();
    let app = api::app::router(state);
    let dash_token = login_token(&app).await;

    let create = app
        .clone()
        .oneshot(
            Request::post("/api/projects")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {dash_token}"))
                .body(Body::from(
                    json!({
                        "name": format!(
                            "test-{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos()
                        )
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("create");
    assert_eq!(create.status(), StatusCode::OK);
    let created = json_body(create).await;
    let project_id = created["id"].as_str().expect("id").to_string();
    assert!(created["keys"]["publishable"]
        .as_str()
        .unwrap_or("")
        .starts_with("ib_pub_"));
    assert!(created["keys"]["secret"]
        .as_str()
        .unwrap_or("")
        .starts_with("ib_sec_"));

    let schema = format!("proj_{project_id}");
    let schema_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM information_schema.schemata WHERE schema_name = $1)",
    )
    .bind(&schema)
    .fetch_one(&pool)
    .await
    .expect("schema check");
    assert!(schema_exists, "expected schema {schema}");

    let key_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE project_id = $1")
        .bind(&project_id)
        .fetch_one(&pool)
        .await
        .expect("key count");
    assert_eq!(key_count, 2);

    let list = app
        .clone()
        .oneshot(
            Request::get("/api/projects")
                .header("authorization", format!("Bearer {dash_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let listed = json_body(list).await;
    let projects = listed["projects"].as_array().expect("projects array");
    assert!(projects.iter().any(|p| p["id"] == project_id));

    let ctx = app
        .oneshot(
            Request::get("/api/auth/project-context")
                .header("authorization", format!("Bearer {dash_token}"))
                .header("x-indiebase-project-id", &project_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("project context");
    assert_eq!(ctx.status(), StatusCode::OK);
    let body = json_body(ctx).await;
    assert_eq!(body["project_id"].as_str(), Some(project_id.as_str()));
    assert_eq!(body["role"].as_str(), Some("owner"));
}
