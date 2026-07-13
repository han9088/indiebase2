## Purpose

Project lifecycle on the Manager API: platform tables, create/list projects, per-project `proj_{ulid}` schema provisioning, default API keys, and PostgREST schema registration.

## Requirements

### Requirement: Platform tables exist in public schema

The system SHALL provide platform tables in `public` including at least `users`, `projects` (id ULID), `project_members`, and `api_keys` (`key_type` publishable|secret, key hash, status, `project_id`). Soft-delete via `deleted_at` where applicable. Development MAY synchronize schema via SeaQuery; production MUST apply sqlx migrations.

#### Scenario: Schema available after startup sync or migrate

- **WHEN** the API starts against a configured Postgres database (dev sync or production migrate)
- **THEN** the platform tables exist and are queryable

### Requirement: Create project provisions schema and keys

`POST /api/projects` (Manager API, Dashboard Session required) SHALL create a project with a new ULID and:

1. Insert `public.projects` and owner membership in `project_members`
2. `CREATE SCHEMA proj_{ulid}`
3. Create tenant DB roles (`anon`, `authenticated`, `service` with BYPASSRLS, `project_operator`, `project_operator_readonly`) and grants for that schema
4. Insert default Publishable + Secret API key rows (hashes only; plaintext shown once in response)
5. Register the schema with PostgREST and trigger reload

#### Scenario: Create project succeeds

- **WHEN** an authenticated platform user creates a project with a non-empty name
- **THEN** DB has `proj_{ulid}` schema and two `api_keys` rows for that project
- **AND** response includes project id, name, and one-time plaintext keys

### Requirement: List projects for dashboard user

`GET /api/projects` SHALL return projects the Dashboard Session user is an active member of (excluding soft-deleted projects/memberships), each with the caller's `role`.

#### Scenario: List after create

- **WHEN** the creator lists projects with a valid Dashboard Session
- **THEN** the new project appears in the list with role `owner`

### Requirement: PostgREST can reach new schema after reload

After project create, PostgREST SHALL expose the new `proj_{ulid}` schema via the documented registration + reload path (schemas file + `pg_notify` / config reload).

#### Scenario: Schema registered

- **WHEN** project create completes
- **THEN** PostgREST configuration includes `proj_{ulid}` and reload has been requested
