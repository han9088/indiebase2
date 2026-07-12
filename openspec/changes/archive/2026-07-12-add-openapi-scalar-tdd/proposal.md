## Why

Phase 0 delivers a bare `/health` endpoint with no machine-readable contract or interactive docs. Adopting OpenAPI (code-first via utoipa) plus [Scalar](https://scalar.com/) gives developers and agents a live API reference from day one, and TDD keeps each endpoint behavior locked by tests before implementation.

## What Changes

- Add `utoipa` + `utoipa-scalar` to the workspace; annotate handlers with `#[utoipa::path]`.
- Serve OpenAPI JSON at `GET /openapi.json` and Scalar UI at `GET /docs`.
- Document `/health` in the OpenAPI spec (title, version, path, response schema).
- Write integration tests **before** wiring docs routes (TDD red-green-refactor).
- Update README with docs URLs; note TDD + OpenAPI as the default API development workflow in OpenSpec context.

## Capabilities

### New Capabilities

- `api-docs`: OpenAPI spec generation, Scalar interactive docs, and test coverage for doc endpoints.

### Modified Capabilities

- (none)

## Non-goals

- Swagger UI (Scalar only).
- Auth/security schemes in OpenAPI (Phase 1+).
- CI publish to external doc hosting.
- OpenAPI for Manager/Data API routes not yet implemented.

## Impact

- `Cargo.toml` (workspace deps), `crates/api` (new `openapi` module, route changes, tests).
- No runtime infra changes; docs served by the same Axum process.
