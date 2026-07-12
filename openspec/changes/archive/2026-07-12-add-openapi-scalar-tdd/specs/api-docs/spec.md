## ADDED Requirements

### Requirement: OpenAPI document endpoint

The API SHALL expose `GET /openapi.json` returning OpenAPI 3.x JSON with `Content-Type: application/json`.

#### Scenario: Fetch OpenAPI spec

- **WHEN** client sends `GET /openapi.json`
- **THEN** response status is 200
- **AND** body parses as JSON with `openapi` and `paths` keys
- **AND** `paths` includes `/health`

### Requirement: Scalar interactive documentation

The API SHALL serve Scalar API reference at `GET /docs` (or redirect to trailing-slash equivalent).

#### Scenario: Open docs UI

- **WHEN** client sends `GET /docs` or `GET /docs/`
- **THEN** response status is 200
- **AND** `Content-Type` indicates HTML

### Requirement: Health endpoint documented in OpenAPI

The OpenAPI spec SHALL describe `GET /health` with a JSON response containing a `status` string field.

#### Scenario: Health path metadata

- **WHEN** client fetches `/openapi.json`
- **THEN** `paths["/health"].get.responses` includes a 200 response
- **AND** response schema documents a `status` property

### Requirement: Test-driven API development

New API behavior in `crates/api` SHALL be covered by failing integration tests written before production code (red-green-refactor).

#### Scenario: Docs endpoints developed with TDD

- **WHEN** OpenAPI and Scalar routes are added
- **THEN** integration tests for `/openapi.json` and `/docs` exist and were used to drive implementation
