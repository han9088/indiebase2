## ADDED Requirements

### Requirement: Environment selection via INDIEBASE_ENV

The API SHALL read `INDIEBASE_ENV` as the active environment (Vite-style mode). When unset or empty, the environment SHALL default to `development`. Supported documented values are `development` and `production`.

#### Scenario: Default environment is development

- **WHEN** `INDIEBASE_ENV` is not set
- **THEN** `Config.env` is `development`

#### Scenario: Explicit production environment

- **WHEN** `INDIEBASE_ENV=production`
- **THEN** `Config.env` is `production`

### Requirement: Vite-style dotenv file layers

The API SHALL load env files in this order (later overrides earlier among files): `.env`, `.env.local`, `.env.[env]`, `.env.[env].local`, where `[env]` is the active `INDIEBASE_ENV` value. Variables already present in the process environment SHALL NOT be overwritten by dotenv files.

#### Scenario: Environment file overrides base .env

- **WHEN** `.env` sets `SHARED_KEY=from-base` and `.env.development` sets `SHARED_KEY=from-env` and environment is `development`
- **THEN** process env `SHARED_KEY` is `from-env`

#### Scenario: Process env wins

- **WHEN** process already has `POSTGRES_HOST=from-process` and `.env` sets `POSTGRES_HOST=from-file`
- **THEN** `POSTGRES_HOST` remains `from-process`

### Requirement: Discrete Postgres and Redis settings

The API SHALL load Postgres from `POSTGRES_HOST`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB` (optional `POSTGRES_PORT`, default `5432`) and Redis from `REDIS_HOST`, `REDIS_PASSWORD` (optional `REDIS_PORT`, default `6379`). Connection URIs SHALL be derived in code with password percent-encoding. The API MUST NOT require `DATABASE_URL` or `REDIS_URL`.

#### Scenario: Build database URL from Postgres fields

- **WHEN** env provides Orb host `postgres.indiebase2.orb.local`, user `postgres`, password `dev@indiebase.com`, db `indiebase-dev`
- **THEN** `Config.database_url()` equals `postgres://postgres:dev%40indiebase.com@postgres.indiebase2.orb.local:5432/indiebase-dev`

### Requirement: PostgREST base URL

The API SHALL require non-empty `POSTGREST_URL`.

#### Scenario: Missing POSTGREST_URL

- **WHEN** `POSTGREST_URL` is not set after dotenv load
- **THEN** config loading fails with a missing `POSTGREST_URL` error

### Requirement: Local Compose env-file convention

Local Docker Compose SHALL be documented and invoked with `--env-file .env.development` as the sole convention.

#### Scenario: Documented compose command

- **WHEN** a developer starts local infrastructure from project docs
- **THEN** the documented command is `docker compose --env-file .env.development up -d`
