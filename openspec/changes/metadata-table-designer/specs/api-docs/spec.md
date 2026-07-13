## ADDED Requirements

### Requirement: Table designer paths in OpenAPI

The OpenAPI document SHALL include Manager table/column Designer paths and request/response schemas for create/list/update/delete and `allow_anon_read` updates.

#### Scenario: Spec lists table routes

- **WHEN** client fetches `/openapi.json`
- **THEN** `paths` includes the Manager table Designer endpoints introduced by this change
