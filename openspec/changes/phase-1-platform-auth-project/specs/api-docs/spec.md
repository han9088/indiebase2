## ADDED Requirements

### Requirement: Manager auth and project paths in OpenAPI

The OpenAPI document at `GET /openapi.json` SHALL include paths for Dashboard login/logout, Project login/logout, and Project create/list under `/api/auth/*` and `/api/projects*`.

#### Scenario: Spec lists auth routes

- **WHEN** client fetches `/openapi.json`
- **THEN** `paths` includes `/api/auth/login` and `/api/projects`
