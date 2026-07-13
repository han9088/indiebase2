## ADDED Requirements

### Requirement: Data API paths in OpenAPI

The OpenAPI document at `GET /openapi.json` SHALL document Data API gateway routes for Dashboard `/api/data/*` and SDK `/api/data/{project_id}/*`, including auth headers and representative 401/403 responses.

#### Scenario: Spec lists Data API paths

- **WHEN** client fetches `/openapi.json`
- **THEN** `paths` includes entries under `/api/data` covering Dashboard and SDK project-scoped patterns
