## 1. Workspace scaffold

- [x] 1.1 Add root `Cargo.toml` workspace with `members = ["crates/api"]` and shared workspace metadata (edition 2021, rust-version if desired)
- [x] 1.2 Create `crates/api/Cargo.toml` with dependencies: `axum`, `tokio`, `serde`, `serde_json`, `dotenvy`, `tracing`, `tracing-subscriber`
- [x] 1.3 Add `src/main.rs`, `src/config.rs`, `src/app.rs`, `src/routes/mod.rs`, `src/routes/health.rs` per design.md layout

## 2. Configuration

- [x] 2.1 Implement `Config` struct loading `INDIEBASE_HTTP_ADDR`, `DATABASE_URL`, `REDIS_URL`, `POSTGREST_URL` from env (after optional `dotenvy::dotenv()`)
- [x] 2.2 Fail fast with clear error when required vars missing or empty
- [x] 2.3 Add `.env.example` documenting vars aligned with `docker-compose.yaml` (ports 5432, 6379, 13000)

## 3. HTTP server

- [x] 3.1 Implement `GET /health` returning 200 JSON `{"status":"ok"}` (or equivalent)
- [x] 3.2 Wire `app::router()` merging health routes; add comments for future Manager API / Data API nests
- [x] 3.3 Implement `main`: load config, init tracing, bind `INDIEBASE_HTTP_ADDR`, serve Axum router

## 4. Tests and quality

- [x] 4.1 Unit test: config validation (missing env → error)
- [x] 4.2 Integration test: `GET /health` via in-process Axum (`tower::ServiceExt::oneshot`)
- [x] 4.3 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all pass

## 5. Documentation

- [x] 5.1 Add README section (or crate README): `docker compose up -d`, copy `.env.example`, `cargo run -p api`, `curl /health`
- [x] 5.2 Link to [docs/prd/mvp-phases.md](../../docs/prd/mvp-phases.md) Phase 0 as completed scope when change ships

## 6. Manual verification (Phase 0 acceptance)

- [x] 6.1 With compose up, start server and confirm `curl localhost:8080/health` returns 200 JSON
- [x] 6.2 Confirm server starts without connecting to Postgres/Redis (liveness-only health)
