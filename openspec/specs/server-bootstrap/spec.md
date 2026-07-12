## Purpose

Phase 0 Axum server bootstrap: workspace, health, config seams for Manager API and Data API.

## Requirements

### Requirement: Cargo workspace builds and lints cleanly

The repository SHALL provide a root Cargo workspace that includes at least one binary server crate. The workspace MUST pass `cargo fmt --check`, `cargo clippy -- -D warnings` (workspace-default), and `cargo test` without manual patches.

#### Scenario: Developer runs quality checks

- **WHEN** a developer runs `cargo fmt`, `cargo clippy`, and `cargo test` at the repository root
- **THEN** all commands complete successfully with exit code 0

### Requirement: Health endpoint is available

The server SHALL expose `GET /health` on the configured listen address. A successful response MUST use HTTP status 200 and a JSON body indicating healthy status (e.g. `{"status":"ok"}`).

#### Scenario: Health check succeeds

- **WHEN** a client sends `GET /health` to a running server instance
- **THEN** the response status is 200 and the body is valid JSON with a healthy status field

#### Scenario: Health check without dependencies

- **WHEN** Postgres, Redis, or PostgREST are unreachable or not running
- **THEN** `GET /health` still returns 200 (liveness only; dependency checks are out of scope for Phase 0)

### Requirement: Configuration loads from environment

The server SHALL read configuration from process environment variables and Vite-style dotenv layers (see capability `env-config`). At minimum:

- `INDIEBASE_HTTP_ADDR` — bind address, default `0.0.0.0:8080`
- Discrete Postgres / Redis fields and `POSTGREST_URL` as defined by `env-config`

#### Scenario: Server starts with valid env

- **WHEN** all required environment variables are set (or dotenv files provide them)
- **THEN** the server process starts and listens on the configured address

#### Scenario: Server fails fast on missing config

- **WHEN** a required environment variable is missing or empty
- **THEN** the server exits during startup with a clear error message (non-zero exit code)

### Requirement: Server runs alongside local Docker stack

The server SHALL be runnable on the host while Docker Compose provides Postgres, Redis, and PostgREST. Documentation MUST use `docker compose --env-file .env.development up -d` as the sole local compose convention.

#### Scenario: Local development startup

- **WHEN** Docker compose services are up and the developer starts the Axum server with repo-standard env
- **THEN** `GET /health` responds successfully on the configured port

### Requirement: Module layout reserves Manager and Data API seams

The server crate SHALL organize code so Phase 1+ can add routers without restructuring: e.g. `main.rs` (or `lib.rs` + bin) wires Axum; placeholder modules or router merge points for **Manager API** and **Data API gateway** are documented in design or code comments.

#### Scenario: Future router extension

- **WHEN** a follow-up change adds `/api/auth/*` or `/api/data/*` routes
- **THEN** new routers can nest into the existing Axum app without renaming the crate or moving the workspace root
