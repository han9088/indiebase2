## ADDED Requirements

### Requirement: App Auth paths in OpenAPI

The OpenAPI document SHALL include `POST /api/auth/app/login` and `POST /api/auth/app/logout` (or the final path names chosen) with security requirements for Publishable Key and Bearer session.

#### Scenario: Spec lists app auth routes

- **WHEN** client fetches `/openapi.json`
- **THEN** `paths` includes the App Auth login and logout operations
