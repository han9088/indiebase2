## ADDED Requirements

### Requirement: Platform tables exist in public schema

The system SHALL provide migrations creating at least `public.users`, `public.projects` (id ULID CHAR(26)/TEXT), `public.project_members`, and `public.api_keys` (key_type publishable|secret, key hash, status, project_id).

#### Scenario: Migrations apply cleanly

- **WHEN** migrations are run against the configured Postgres database
- **THEN** the platform tables exist and are queryable

### Requirement: Create project provisions schema and keys

`POST /api/projects` (Manager API, Dashboard Session required) SHALL create a project with a new ULID and:

1. Insert `public.projects` and owner membership
2. `CREATE SCHEMA proj_{ulid}`
3. Create tenant DB roles (`anon`, `authenticated`, `service` with BYPASSRLS, `project_operator`, `project_operator_readonly`) and grants for that schema
4. Insert default Publishable + Secret API key rows (hashes only; plaintext shown once in response)
5. Register the schema with PostgREST and trigger reload

#### Scenario: Create project succeeds

- **WHEN** an authenticated platform user creates a project
- **THEN** DB has `proj_{ulid}` schema and two `api_keys` rows for that project
- **AND** response includes project id and one-time plaintext keys

### Requirement: List projects for dashboard user

`GET /api/projects` SHALL return projects the Dashboard Session user is a member of.

#### Scenario: List after create

- **WHEN** the creator lists projects with a valid Dashboard Session
- **THEN** the new project appears in the list

### Requirement: PostgREST can reach new schema after reload

After project create, PostgREST SHALL expose the new `proj_{ulid}` schema (verified by an internal smoke check or documented reload path).

#### Scenario: Schema registered

- **WHEN** project create completes
- **THEN** PostgREST configuration includes `proj_{ulid}` (or equivalent dynamic schemas) and reload has been requested
