## Context

- **Current state**: `crates/api` serves `GET /health` with an integration test; no OpenAPI or docs UI.
- **Goal**: Code-first OpenAPI via [utoipa](https://docs.rs/utoipa) and interactive docs via [Scalar](https://scalar.com/) ([utoipa-scalar](https://docs.rs/utoipa-scalar)), developed with TDD.

## Goals / Non-Goals

**Goals:**

- `GET /openapi.json` returns OpenAPI 3.x JSON including `/health`.
- `GET /docs` (Scalar) renders interactive API reference pointing at the spec.
- `#[utoipa::path]` on handlers; central `ApiDoc` struct with `OpenApi` derive.
- Integration tests written **before** implementation (TDD).
- `cargo test`, `cargo clippy` pass.

**Non-Goals:**

- Swagger UI, external doc hosting, security schemes, Manager/Data routes in spec.

## Decisions

### 1. Libraries

- **Decision**: `utoipa = "5"` + `utoipa-scalar = { version = "0.3", features = ["axum"] }`.
- **Rationale**: Official Scalar bridge for utoipa; matches Axum 0.8 in workspace.

### 2. Routes

| Path | Purpose |
|------|---------|
| `GET /openapi.json` | Raw OpenAPI document (for tools, SDK gen, agents) |
| `GET /docs` | Scalar UI (merged via `Scalar::with_url`) |

- **Decision**: Explicit `/openapi.json` route on the app router **plus** Scalar at `/docs`.
- **Rationale**: Stable spec URL independent of Scalar asset paths; matches common conventions.

### 3. Module layout

```text
crates/api/src/
  openapi.rs       # ApiDoc, serve_openapi handler
  routes/
    mod.rs         # merge health + docs
    health.rs      # #[utoipa::path] on health handler
```

- `app.rs` merges health routes and docs (openapi + Scalar).

### 4. TDD workflow

1. **RED**: Add tests asserting `/openapi.json` and `/docs` behavior (404 initially).
2. **GREEN**: Add deps, `ApiDoc`, routes, Scalar merge.
3. **REFACTOR**: Annotate `/health` in OpenAPI; ensure spec lists the path.

Existing `/health` test stays; extend spec test to assert `paths["/health"]` exists.

### 5. OpenAPI metadata

- **title**: `Indiebase API`
- **version**: `0.1.0` (crate version)
- **description**: Brief MVP note (Manager + Data API to follow).

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Spec drift from handlers | Require `#[utoipa::path]` on every new route; test spec contains path |
| Scalar path quirks | Integration test checks `/docs` returns HTML 200 |

## Open Questions

- None for this change.
