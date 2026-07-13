## ADDED Requirements

### Requirement: Dual-path Data API routing

The API SHALL expose a Data API gateway that proxies to internal PostgREST. Requests whose second path segment after `/api/data/` is a 26-character lowercase ULID SHALL be handled as the SDK path `/api/data/{project_id}/*`. All other `/api/data/*` requests SHALL be handled as the Dashboard path. ULID-shaped segments MUST NOT fall through to the Dashboard branch.

#### Scenario: SDK path with project ULID

- **WHEN** client requests `GET /api/data/{ulid}/users` with a valid Secret or Publishable Key for that project
- **THEN** the gateway resolves project from the URL ULID and proxies to PostgREST with `Accept-Profile: proj_{ulid}`

#### Scenario: Dashboard path without ULID segment

- **WHEN** client requests `GET /api/data/users` with a valid Dashboard Session and `X-Indiebase-Project-Id`
- **THEN** the gateway resolves project from the header and proxies with `Accept-Profile: proj_{ulid}`

### Requirement: Credential mutual exclusion

The gateway SHALL enforce architecture §6.2.3. Dashboard path MUST require Dashboard Session Bearer + `X-Indiebase-Project-Id` and MUST reject any `X-Indiebase-Api-Key`. SDK path MUST require `X-Indiebase-Api-Key` and MUST reject Dashboard Session Bearers. Illegal combinations SHALL return `403`.

#### Scenario: Dashboard path rejects API Key

- **WHEN** client calls `/api/data/users` with Dashboard Session and also `X-Indiebase-Api-Key`
- **THEN** response is 403

#### Scenario: SDK path rejects Dashboard Session

- **WHEN** client calls `/api/data/{project_id}/users` with Dashboard Session Bearer and without a valid API Key for that path
- **THEN** response is 403

### Requirement: API Key binding to URL project

On the SDK path, the gateway SHALL validate that the Publishable or Secret Key is active and bound to the URL `project_id`. Mismatch SHALL return `403`.

#### Scenario: Key project mismatch

- **WHEN** Publishable Key for project A is used on `/api/data/{project_B}/...`
- **THEN** response is 403

#### Scenario: Secret Key service mode

- **WHEN** a valid Secret Key is used on `/api/data/{project_id}/{table}`
- **THEN** gateway sets `auth_mode=service` and proxies successfully (subject to PostgREST/table existence)

### Requirement: Transparent PostgREST proxy

The gateway SHALL strip `/api/data` (Dashboard) or `/api/data/{project_id}` (SDK), forward query string and body unchanged, inject schema profile headers, replace Authorization with the PostgREST authenticator credential, and return PostgREST status and body. Client credentials MUST NOT be forwarded. The gateway SHALL forward `Prefer` and `Range` request headers and `Content-Range` responses when present.

#### Scenario: Query string preserved

- **WHEN** client requests `.../users?select=id,name&order=id.asc`
- **THEN** PostgREST receives the same query string on the stripped path

### Requirement: Internal-Context and SET ROLE

The gateway SHALL attach a signed `X-Indiebase-Internal-Context` describing at least `auth_mode`, `project_id`, and user identity fields when applicable. PostgREST `db-pre-request` SHALL verify the signature, `SET LOCAL` `app.*` settings, and `SET ROLE` to the tenant role for that `auth_mode` (`anon`, `authenticated`, `project_operator`, `project_operator_readonly`, `service`). Client-supplied Internal-Context headers MUST be ignored/overwritten.

#### Scenario: Dashboard owner maps to project_operator

- **WHEN** an owner uses Dashboard Data API path
- **THEN** `auth_mode` is `project_operator` and DB role is `project_operator`

#### Scenario: Publishable Key without App User Session is anon

- **WHEN** SDK path is called with Publishable Key and no App User Session
- **THEN** `auth_mode` is `anon`

#### Scenario: Publishable Key with App User Session is authenticated

- **WHEN** SDK path is called with Publishable Key and valid `app_user_session:` Bearer whose `project_id` matches the URL
- **THEN** `auth_mode` is `authenticated` and `app.user_id` is the session end user

### Requirement: App User Session project consistency

On the SDK path, when an App User Session Bearer is present, Redis lookup MUST use only `app_user_session:` (never `dashboard_session:`). Session `project_id` MUST equal URL `project_id` or the gateway SHALL return `403`.

#### Scenario: Cross-project App User Session

- **WHEN** App User Session for project A is sent to `/api/data/{project_B}/...`
- **THEN** response is 403
