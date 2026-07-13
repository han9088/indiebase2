## ADDED Requirements

### Requirement: Data API gateway secrets and PostgREST proxy config

The API SHALL load non-empty configuration for PostgREST proxy authentication and Internal-Context signing (e.g. authenticator/JWT secret and `INDIEBASE_INTERNAL_CONTEXT_SECRET` or equivalent names documented in `.env.example`). `POSTGREST_URL` remains required.

#### Scenario: Missing Internal-Context secret fails config

- **WHEN** Internal-Context signing secret is unset after dotenv load
- **THEN** config loading fails with a clear missing-variable error

#### Scenario: Example env documents new variables

- **WHEN** a developer opens `.env.example`
- **THEN** Data API / Internal-Context / PostgREST authenticator variables are listed with comments
