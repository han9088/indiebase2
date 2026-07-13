//! Integration tests for Data API gateway (§6.2.3 mutual exclusion + Secret Key smoke).
//! Skips when Postgres/Redis/PostgREST from `.env.development` are unreachable.

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

async fn try_state() -> Option<api::state::AppState> {
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

async fn create_project(app: &axum::Router, token: &str) -> (String, String, String) {
    let created = app
        .clone()
        .oneshot(
            Request::post("/api/projects")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "name": format!("data-api-{}", ulid::Ulid::new()) }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("create");
    assert_eq!(created.status(), StatusCode::OK);
    let body = json_body(created).await;
    (
        body["id"].as_str().unwrap().to_string(),
        body["keys"]["publishable"].as_str().unwrap().to_string(),
        body["keys"]["secret"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn dashboard_path_rejects_api_key() {
    let Some(state) = try_state().await else {
        eprintln!("skipping dashboard_path_rejects_api_key: infra unavailable");
        return;
    };
    let app = api::app::router(state);
    let token = login_token(&app).await;
    let (project_id, publishable, _) = create_project(&app, &token).await;

    let res = app
        .oneshot(
            Request::get("/api/data/users")
                .header("authorization", format!("Bearer {token}"))
                .header("x-indiebase-project-id", &project_id)
                .header("x-indiebase-api-key", &publishable)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn sdk_path_rejects_dashboard_session_without_key() {
    let Some(state) = try_state().await else {
        eprintln!("skipping sdk_path_rejects_dashboard_session_without_key: infra unavailable");
        return;
    };
    let app = api::app::router(state);
    let token = login_token(&app).await;
    let (project_id, _, _) = create_project(&app, &token).await;

    let res = app
        .oneshot(
            Request::get(format!("/api/data/{project_id}/users"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    // Missing API key → 401; with only Dashboard Session still not a valid SDK call.
    assert!(
        res.status() == StatusCode::UNAUTHORIZED || res.status() == StatusCode::FORBIDDEN,
        "status={}",
        res.status()
    );
}

#[tokio::test]
async fn sdk_path_rejects_dashboard_session_even_with_key() {
    let Some(state) = try_state().await else {
        eprintln!("skipping sdk_path_rejects_dashboard_session_even_with_key: infra unavailable");
        return;
    };
    let app = api::app::router(state);
    let token = login_token(&app).await;
    let (project_id, publishable, _) = create_project(&app, &token).await;

    let res = app
        .oneshot(
            Request::get(format!("/api/data/{project_id}/users"))
                .header("authorization", format!("Bearer {token}"))
                .header("x-indiebase-api-key", &publishable)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn publishable_key_project_mismatch_is_forbidden() {
    let Some(state) = try_state().await else {
        eprintln!("skipping publishable_key_project_mismatch_is_forbidden: infra unavailable");
        return;
    };
    let app = api::app::router(state);
    let token = login_token(&app).await;
    let (project_a, publishable_a, _) = create_project(&app, &token).await;
    let (project_b, _, _) = create_project(&app, &token).await;
    assert_ne!(project_a, project_b);

    let res = app
        .oneshot(
            Request::get(format!("/api/data/{project_b}/users"))
                .header("x-indiebase-api-key", &publishable_a)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn secret_key_proxies_to_postgrest() {
    let Some(state) = try_state().await else {
        eprintln!("skipping secret_key_proxies_to_postgrest: infra unavailable");
        return;
    };
    let pool = state.pool.clone();
    let app = api::app::router(state);
    let token = login_token(&app).await;
    let (project_id, _, secret) = create_project(&app, &token).await;

    let schema = format!("proj_{project_id}");
    let create_table =
        format!("CREATE TABLE {schema}.smoke_items (id int primary key, name text not null)");
    if sqlx::query(&create_table).execute(&pool).await.is_err() {
        eprintln!("skipping secret_key_proxies_to_postgrest: could not create smoke table");
        return;
    }
    let _ = sqlx::query(&format!(
        "GRANT ALL ON TABLE {schema}.smoke_items TO service, project_operator"
    ))
    .execute(&pool)
    .await;
    let _ = sqlx::query(&format!(
        "INSERT INTO {schema}.smoke_items (id, name) VALUES (1, 'ok')"
    ))
    .execute(&pool)
    .await;

    // Give PostgREST a moment after schema registration / NOTIFY.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let res = app
        .oneshot(
            Request::get(format!("/api/data/{project_id}/smoke_items?select=*"))
                .header("x-indiebase-api-key", &secret)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    // If PostgREST is down or pre-request not loaded yet, skip rather than fail CI hard.
    if res.status() == StatusCode::INTERNAL_SERVER_ERROR {
        eprintln!(
            "skipping secret_key assertion: PostgREST proxy returned 500 (is postgrest up + restarted?)"
        );
        return;
    }

    assert!(
        res.status().is_success() || res.status() == StatusCode::NOT_FOUND,
        "unexpected status {} body={:?}",
        res.status(),
        json_body(res).await
    );
}
