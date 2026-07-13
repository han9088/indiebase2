## Purpose

Dashboard authentication for the Manager API: Opaque Token + Redis sessions, with per-request project context via `X-Indiebase-Project-Id`. No JWT. No separate Project Session.

## Requirements

### Requirement: Dashboard login issues opaque session

The API SHALL accept `POST /api/auth/login` with platform email/password and return an opaque Bearer token. The server SHALL store `dashboard_session:{token}` in Redis with at least `user_id` and expiry. No JWT SHALL be used.

#### Scenario: Successful dashboard login

- **WHEN** a valid platform user submits credentials to `POST /api/auth/login`
- **THEN** response is 200 with a session token, `token_type` Bearer, and `expires_in`
- **AND** Redis contains `dashboard_session:{token}` for that user

#### Scenario: Invalid credentials

- **WHEN** login credentials are wrong or empty
- **THEN** response is 401 and no dashboard session is created

### Requirement: Dashboard logout revokes session

The API SHALL accept `POST /api/auth/logout` with a Dashboard Session Bearer and delete the corresponding Redis key.

#### Scenario: Logout

- **WHEN** client sends `POST /api/auth/logout` with a valid Dashboard Session
- **THEN** Redis `dashboard_session:{token}` is removed
- **AND** subsequent Manager requests with that token fail auth

### Requirement: Project context via header membership

The API SHALL accept `GET /api/auth/project-context` with a Dashboard Session Bearer and header `X-Indiebase-Project-Id`. The server SHALL resolve membership via `project_members` and return `{ project_id, role }` (`owner` | `admin` | `member`). The system MUST NOT use a second Project Session token.

#### Scenario: Member resolves project context

- **WHEN** a Dashboard-authenticated member sends `GET /api/auth/project-context` with a valid `X-Indiebase-Project-Id`
- **THEN** response is 200 with that `project_id` and the caller's `role`

#### Scenario: Non-member is forbidden

- **WHEN** the user is not an active member of the given project
- **THEN** response is 403

#### Scenario: Missing project header

- **WHEN** `X-Indiebase-Project-Id` is absent on a project-scoped Manager route
- **THEN** response is 400 or 401 as defined by the extractor (request rejected)
