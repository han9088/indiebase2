## ADDED Requirements

### Requirement: Dashboard login issues opaque session

The API SHALL accept `POST /api/auth/login` with platform credentials and return an opaque Bearer token. The server SHALL store `dashboard_session:{token}` in Redis with at least `user_id` and expiry. No JWT SHALL be used.

#### Scenario: Successful dashboard login

- **WHEN** a valid platform user submits credentials to `POST /api/auth/login`
- **THEN** response is 200 with a session token
- **AND** Redis contains `dashboard_session:{token}` for that user

#### Scenario: Invalid credentials

- **WHEN** login credentials are wrong
- **THEN** response is 401 and no dashboard session is created

### Requirement: Dashboard logout revokes session

The API SHALL accept `POST /api/auth/logout` with a Dashboard Session Bearer and delete the corresponding Redis key.

#### Scenario: Logout

- **WHEN** client sends `POST /api/auth/logout` with a valid Dashboard Session
- **THEN** Redis `dashboard_session:{token}` is removed
- **AND** subsequent Manager requests with that token fail auth

### Requirement: Project login issues project-scoped session

The API SHALL accept `POST /api/auth/project/login` (body includes `project_id`) for a member of that project and store `project_session:{token}` in Redis with `user_id`, `project_id`, `project_role`, and expiry.

#### Scenario: Successful project login

- **WHEN** a Dashboard-authenticated (or otherwise authorized) member logs into a project they belong to
- **THEN** Redis `project_session:{token}` includes the correct `project_id` and `project_role`

#### Scenario: Non-member cannot project-login

- **WHEN** the user is not in `project_members` for the given `project_id`
- **THEN** response is 403 and no project session is created

### Requirement: Project logout revokes project session

The API SHALL accept `POST /api/auth/project/logout` and delete `project_session:{token}`.

#### Scenario: Project logout

- **WHEN** client sends `POST /api/auth/project/logout` with a valid Project Session
- **THEN** the Redis project session key is removed
