//! Integration tests for Manager auth + project lifecycle.
//! Skips when Postgres/Redis from `.env.development` are unreachable.

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use redis::AsyncCommands;
use serde_json::json;
use tower::ServiceExt;

async fn try_state() -> Option<api::state::AppState> {
    // `cargo test` uses the crate dir as CWD; Vite-style dotenv files live at the workspace root.
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let _ = std::env::set_current_dir(&workspace_root);

    let config = api::config::Config::from_env().ok()?;
    let pool = api::db::connect_pool(&config).await.ok()?;
    let redis = api::db::connect_redis(&config).await.ok()?;
    api::db::run_migrations(&pool).await.ok()?;
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
                        "email": "admin@indiebase.local",
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
                        "email": "admin@indiebase.local",
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
async fn project_login_forbidden_for_non_member() {
    let Some(state) = try_state().await else {
        eprintln!("skipping project_login_forbidden_for_non_member: Postgres/Redis unavailable");
        return;
    };
    let app = api::app::router(state);

    let login = app
        .clone()
        .oneshot(
            Request::post("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "admin@indiebase.local",
                        "password": "dev@indiebase.com"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("login");
    assert_eq!(login.status(), StatusCode::OK);
    let token = json_body(login).await["token"]
        .as_str()
        .expect("token")
        .to_string();

    // ULID-shaped id the seed user is not a member of → 403
    let forbidden = app
        .oneshot(
            Request::post("/api/auth/project/login")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(
                    json!({ "project_id": "01AAAAAAAAAAAAAAAAAAAAAAAA" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("project login");
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_project_provisions_schema_keys_and_project_session() {
    let Some(state) = try_state().await else {
        eprintln!(
            "skipping create_project_provisions_schema_keys_and_project_session: infra unavailable"
        );
        return;
    };
    let pool = state.pool.clone();
    let mut redis = state.redis.clone();
    let app = api::app::router(state);

    let login = app
        .clone()
        .oneshot(
            Request::post("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "admin@indiebase.local",
                        "password": "dev@indiebase.com"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("login");
    assert_eq!(login.status(), StatusCode::OK);
    let dash_token = json_body(login).await["token"]
        .as_str()
        .expect("token")
        .to_string();

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

    let proj_login = app
        .oneshot(
            Request::post("/api/auth/project/login")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {dash_token}"))
                .body(Body::from(json!({ "project_id": project_id }).to_string()))
                .unwrap(),
        )
        .await
        .expect("project login");
    assert_eq!(proj_login.status(), StatusCode::OK);
    let proj_token = json_body(proj_login).await["token"]
        .as_str()
        .expect("project token")
        .to_string();

    let redis_key = format!("project_session:{proj_token}");
    let raw: Option<String> = redis.get(&redis_key).await.expect("redis get");
    let payload: serde_json::Value =
        serde_json::from_str(&raw.expect("project session missing in redis")).unwrap();
    assert_eq!(payload["project_id"].as_str(), Some(project_id.as_str()));
    assert_eq!(payload["project_role"].as_str(), Some("owner"));
}
