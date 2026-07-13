## ADDED Requirements

### Requirement: App user records in tenant schema

Each project schema SHALL have an `app_users` table (ULID id, email, password hash, soft delete) provisioned for Project Auth. Platform `public.users` MUST NOT be used as App end users.

#### Scenario: New project has app_users

- **WHEN** a project is created (or App Auth provisioning runs)
- **THEN** `proj_{ulid}.app_users` exists

### Requirement: App login issues opaque session

The API SHALL accept `POST /api/auth/app/login` with `project_id` + credentials and a valid Publishable Key for that project, and return an opaque Bearer token. The server SHALL store `app_user_session:{token}` in Redis with `end_user_id`, `project_id`, `role`, and expiry. No JWT SHALL be used.

#### Scenario: Successful app login

- **WHEN** a valid app user logs in with matching Publishable Key
- **THEN** response is 200 with token and Redis contains `app_user_session:{token}` for that project

#### Scenario: Wrong password

- **WHEN** password is invalid
- **THEN** response is 401 and no session is created

#### Scenario: Publishable Key required

- **WHEN** login is attempted without a valid Publishable Key for the project
- **THEN** response is 401 or 403

### Requirement: App logout revokes session

The API SHALL accept `POST /api/auth/app/logout` with an App User Session Bearer and delete the Redis key.

#### Scenario: Logout

- **WHEN** client logs out with a valid App User Session
- **THEN** Redis key is removed and subsequent Data API calls with that token are unauthenticated/forbidden as applicable

### Requirement: Session usable on Data API

A valid App User Session combined with Publishable Key on `/api/data/{project_id}/*` SHALL yield `auth_mode=authenticated` when the gateway is present. Session `project_id` MUST match the URL.

#### Scenario: Authenticated Data API access

- **WHEN** client calls SDK Data API with Publishable Key + App User Session for the same project
- **THEN** gateway treats the request as authenticated (subject to RLS)
