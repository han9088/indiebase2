## 1. Dependencies

- [x] 1.1 Add `utoipa` and `utoipa-scalar` (axum feature) to workspace `Cargo.toml` and `crates/api/Cargo.toml`

## 2. TDD — OpenAPI endpoint (RED → GREEN)

- [x] 2.1 Write failing integration test: `GET /openapi.json` returns 200, valid OpenAPI JSON, includes `/health` path
- [x] 2.2 Implement `openapi.rs` with `ApiDoc` (`OpenApi` derive), `serve_openapi` handler, wire route in router
- [x] 2.3 Annotate `health` handler with `#[utoipa::path]` and register in `ApiDoc`

## 3. TDD — Scalar UI (RED → GREEN)

- [x] 3.1 Write failing integration test: `GET /docs` or `/docs/` returns 200 HTML
- [x] 3.2 Merge `Scalar::with_url("/docs", ApiDoc::openapi())` in app router

## 4. Verification & docs

- [x] 4.1 Run `cargo fmt`, `cargo clippy`, `cargo test -p api`
- [x] 4.2 Update README and `docs/prd/mvp-phases.md`: `/docs` and `/openapi.json`
- [x] 4.3 Add `.cursor/rules/backend-tdd-prd.mdc` and OpenSpec context for TDD + PRD sync
