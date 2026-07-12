## Context

- **Current state**: Repo has Docker (Postgres 17, Redis 6, PostgREST on `:13000`), PRD docs, OpenSpec config—**no Rust application**.
- **Phase 0 goal** ([mvp-phases.md](../../docs/prd/mvp-phases.md)): compilable workspace, Axum `/health`, env config, ready for Phase 1 platform auth.
- **Constraints**: Idiomatic Rust; `cargo fmt` / `cargo clippy`; English in code/comments; align with [baas-platform-architecture.md](../../docs/prd/baas-platform-architecture.md) single Axum process hosting Manager + Data API later.

## Goals / Non-Goals

**Goals:**

- Root Cargo workspace + one server binary crate.
- Axum app with `GET /health`.
- Config struct from env (with `dotenvy` for local `.env`).
- Fail-fast if required URLs/addr missing.
- Integration test for `/health` via `axum::test` or `reqwest` against in-process server.
- Minimal run docs (README section or `crates/.../README.md`).

**Non-Goals:**

- sqlx pool usage, Redis client, PostgREST proxy, migrations, auth, business routes.
- Dockerizing the Axum server (host-run is fine for MVP).
- Workspace split into many crates (defer until Phase 2+ needs it).

## Decisions

### 1. Crate name and layout

- **Decision**: `crates/api` (package and binary name `api`).
- **Rationale**: Matches [mvp-phases.md](../../docs/prd/mvp-phases.md); aligns with repo `crates/api/` convention.
- **Alternatives**: `indiebase-server` (more product-specific); monolithic root package (harder to add tools crates later).

**Suggested tree:**

```text
Cargo.toml                 # workspace
crates/api/
  Cargo.toml
  src/
    main.rs                # tokio main, bind, serve
    config.rs              # env loading
    routes/
      mod.rs
      health.rs            # GET /health
    app.rs                 # Router builder (merge health; nest placeholders)
```

### 2. HTTP stack

- **Decision**: Axum 0.8.x + Tokio full runtime, `tower-http` for trace/cors later (trace optional in Phase 0).
- **Rationale**: PRD mandates Axum; standard ecosystem.

### 3. Configuration

- **Decision**: Dedicated `Config` struct; load with `dotenvy::dotenv().ok()` then `std::env::var`. Required fields validated at startup.
- **Env names** (align with existing `.env` where possible):

| Variable | Purpose | Example |
|----------|---------|---------|
| `INDIEBASE_HTTP_ADDR` | Bind | `0.0.0.0:8080` |
| `DATABASE_URL` | Postgres (Phase 1+) | `postgres://...@localhost:5432/indiebase-dev` |
| `REDIS_URL` | Redis (Phase 1+) | `redis://:password@localhost:6379` |
| `POSTGREST_URL` | Internal PostgREST | `http://localhost:13000` |

- **Rationale**: Fail-fast now; Phase 1 adds pools without renaming vars.
- **Alternatives**: `config` crate with files (overkill for Phase 0).

### 4. Health semantics

- **Decision**: Liveness only—always 200 if process is up. No DB ping in Phase 0.
- **Rationale**: [mvp-phases.md](../../docs/prd/mvp-phases.md) acceptance is `curl /health`; dependency readiness checks belong in Phase 1 (`/ready` optional later).

### 5. Router extension (Manager vs Data API)

- **Decision**: Single `Router` in `app.rs`; comment markers for future:

```rust
// Router::new()
//   .merge(health_routes())
//   // Phase 1+: .nest("/api", manager_api::router())
//   // Phase 2+: .nest("/api/data", data_api::router())
```

- **Rationale**: Matches PRD single-process architecture without implementing routes early.

### 6. Testing

- **Decision**: 
  - Unit test: config parsing / defaults.
  - Integration: spawn app with `axum::Router`, `tower::ServiceExt::oneshot` for `GET /health`.
- **Rationale**: No Docker required in CI for Phase 0 tests.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `.env` not committed; new devs missing vars | Add `.env.example` with keys matching compose |
| Env var naming drift vs docker-compose | Document mapping in README; reuse `POSTGRES_*` build for `DATABASE_URL` in example |
| Over-scoping Phase 0 with sqlx/redis connect | Spec explicitly excludes dependency checks in `/health` |
| Crate layout churn in Phase 2 | Reserve `routes/` + `app.rs` merge pattern now |

## Migration Plan

Greenfield—no migration. After merge:

1. `docker compose up -d`
2. Copy `.env.example` → `.env` if needed
3. `cargo run -p api`
4. `curl localhost:8080/health`

## Open Questions

- Final default port: `8080` vs `3000` (propose **8080** to avoid clash with PostgREST internal 3000 / host 13000).
- Whether to add `GET /ready` stub in Phase 0 (defer to Phase 1).
