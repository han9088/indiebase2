## Why

Indiebase is an early-stage Rust rewrite: Docker infra (Postgres, Redis, PostgREST) exists but there is **no application code**—no `Cargo.toml`, no Axum server. [mvp-phases.md](../../docs/prd/mvp-phases.md) Phase 0 blocks all later MVP work (platform auth, Data API gateway, SDK). We need a minimal, testable server shell that proves the repo can build, run, and connect to local dependencies.

## What Changes

- Add root **Cargo workspace** with primary server crate `crates/api`.
- Add **Axum** HTTP server with `GET /health` returning 200 JSON.
- Add **configuration** loaded from environment (`.env` / process env): listen address, Postgres URL, Redis URL, PostgREST internal URL (read-only for future use; no DB calls required in Phase 0).
- Add **startup wiring**: Tokio runtime, graceful structure for later Manager API + Data API routers (empty modules or comments only—no business routes yet).
- Add **CI-friendly tests**: unit/integration smoke for `/health`; `cargo fmt`, `cargo clippy`, `cargo test` pass.
- Document how to run the server alongside `docker compose up -d` (README snippet or crate README—minimal).

## Non-goals

- Platform migrations, auth, Manager API, Data API, PostgREST proxy, sqlx queries, Redis usage.
- Dashboard frontend, TS SDK, Storage (OpenDAL).
- Production deployment, TLS, or multi-crate split beyond workspace + one server crate.

## Capabilities

### New Capabilities

- `server-bootstrap`: Cargo workspace, Axum process, `/health`, env-based config, local run instructions.

### Modified Capabilities

- _(none — `openspec/specs/` is empty; greenfield)_

## Impact

- **New files**: root `Cargo.toml`, server crate under `crates/`, `.env.example` if missing.
- **Dependencies**: `axum`, `tokio`, `serde`, `serde_json`, config crate (e.g. `dotenvy` + manual env or `config`).
- **Docker**: no compose changes required; server runs on host and connects to published ports (5432, 6379, 13000).
- **Follow-up changes**: `platform-auth-project-lifecycle` (Phase 1), `data-api-gateway` (Phase 2), per [mvp-phases.md](../../docs/prd/mvp-phases.md).
