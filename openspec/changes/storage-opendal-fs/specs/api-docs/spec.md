## ADDED Requirements

### Requirement: Storage paths in OpenAPI

The OpenAPI document SHALL include Manager Storage paths under `/api/projects/{project_id}/files*`.

#### Scenario: Spec lists file routes

- **WHEN** client fetches `/openapi.json`
- **THEN** `paths` includes list/upload and file-id download/delete operations
